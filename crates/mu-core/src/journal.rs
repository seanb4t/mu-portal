//! The journal: one sequence of events for all worlds, and its subscribers.

use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::SystemTime;

use futures::Stream;
use thiserror::Error;
use tokio::sync::broadcast::{self, error::RecvError, error::TryRecvError};

use crate::error::PortalError;
use crate::event::{Event, NewEvent, Seq};

/// The capacity of the live channel of the server.
pub const DEFAULT_CAPACITY: usize = 1024;

/// The largest capacity that [`MemoryJournal::new`] accepts.
pub const MAX_CAPACITY: usize = 65_536;

/// The record of every event of the server, in order.
pub trait Journal: Send + Sync + 'static {
    /// Numbers the event, records it and sends it to each live subscriber.
    fn append(&self, event: NewEvent) -> Event;

    /// The recorded events from `from`, then each new event.
    ///
    /// 0 and 1 both mean the first event. A `from` past the last number plus 1 fails with
    /// [`PortalError::PastEnd`], which carries the last number.
    ///
    /// # Errors
    ///
    /// [`PortalError::PastEnd`] for a `from` past the next number.
    fn subscribe(&self, from: Seq) -> Result<Subscription, PortalError>;

    /// The last sequence number, or 0 for an empty journal.
    fn last(&self) -> Seq;
}

/// A subscriber fell more live events behind than the channel holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
#[error("the subscriber fell {0} events behind")]
pub struct Lagged(pub u64);

/// The events of one subscriber: the backlog at the call, then the live events.
#[derive(Debug)]
pub struct Subscription {
    last: Seq,
    backlog: VecDeque<Event>,
    live: broadcast::Receiver<Event>,
}

impl Subscription {
    /// The last sequence number in the journal when the subscription started.
    #[must_use]
    pub fn last(&self) -> Seq {
        self.last
    }

    /// The next event. `None` means the journal closed.
    ///
    /// # Errors
    ///
    /// [`Lagged`] if the subscriber fell behind the live channel.
    pub async fn next(&mut self) -> Option<Result<Event, Lagged>> {
        if let Some(event) = self.backlog.pop_front() {
            return Some(Ok(event));
        }
        match self.live.recv().await {
            Ok(event) => Some(Ok(event)),
            Err(RecvError::Lagged(n)) => Some(Err(Lagged(n))),
            Err(RecvError::Closed) => None,
        }
    }

    /// The next event if one is ready now.
    ///
    /// # Errors
    ///
    /// [`Lagged`] if the subscriber fell behind the live channel.
    pub fn try_next(&mut self) -> Option<Result<Event, Lagged>> {
        if let Some(event) = self.backlog.pop_front() {
            return Some(Ok(event));
        }
        match self.live.try_recv() {
            Ok(event) => Some(Ok(event)),
            Err(TryRecvError::Lagged(n)) => Some(Err(Lagged(n))),
            Err(TryRecvError::Empty | TryRecvError::Closed) => None,
        }
    }

    /// The subscription as a stream. The stream ends after a lag.
    pub fn into_stream(self) -> impl Stream<Item = Result<Event, Lagged>> + Send + 'static {
        futures::stream::unfold(Some(self), |state| async move {
            let mut subscription = state?;
            let item = subscription.next().await?;
            let state = item.is_ok().then_some(subscription);
            Some((item, state))
        })
    }
}

/// A capacity that [`MemoryJournal::new`] refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
#[error("the capacity {0} is not a power of two from 1 to {MAX_CAPACITY}")]
pub struct BadCapacity(pub usize);

struct Inner {
    events: Vec<Event>,
    live: broadcast::Sender<Event>,
    /// The maker of the ids, which keeps them in the order of the events.
    ids: ulid::Generator,
}

/// A journal in memory. A restart of the server loses it.
pub struct MemoryJournal {
    inner: Mutex<Inner>,
}

impl MemoryJournal {
    /// A journal whose live channel holds `capacity` events.
    ///
    /// # Errors
    ///
    /// [`BadCapacity`] for 0, a capacity that is not a power of two, or one above
    /// [`MAX_CAPACITY`].
    pub fn new(capacity: usize) -> Result<Self, BadCapacity> {
        if !capacity.is_power_of_two() || capacity > MAX_CAPACITY {
            return Err(BadCapacity(capacity));
        }
        let (live, _) = broadcast::channel(capacity);
        Ok(Self {
            inner: Mutex::new(Inner {
                events: Vec::new(),
                live,
                ids: ulid::Generator::new(),
            }),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        // A panic under the lock leaves the Vec whole, so the data stays usable.
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn last_of(events: &[Event]) -> Seq {
    events.last().map_or(Seq(0), |event| event.seq)
}

impl Journal for MemoryJournal {
    fn append(&self, event: NewEvent) -> Event {
        let mut inner = self.lock();
        let id = match inner.ids.generate() {
            Ok(id) => id,
            Err(overflow) => overflow.commit_overflow_increment(),
        };
        let event = Event {
            seq: last_of(&inner.events).next(),
            id,
            world: event.world,
            kind: event.kind,
            source: event.source,
            text: event.text,
            time: SystemTime::now(),
            depth: event.depth,
            streams: event.streams,
        };
        inner.events.push(event.clone());
        // No receiver is not an error: the event stays in the journal.
        let _ = inner.live.send(event.clone());
        event
    }

    fn subscribe(&self, from: Seq) -> Result<Subscription, PortalError> {
        let inner = self.lock();
        let last = last_of(&inner.events);
        let from = from.max(Seq(1));
        if from > last.next() {
            return Err(PortalError::PastEnd { from, last });
        }
        // The backlog and the receiver come from one lock, so no event falls between them.
        let start = usize::try_from(from.0 - 1).unwrap_or(usize::MAX);
        let backlog = inner.events.get(start..).unwrap_or_default().to_vec();
        Ok(Subscription {
            last,
            backlog: backlog.into(),
            live: inner.live.subscribe(),
        })
    }

    fn last(&self) -> Seq {
        last_of(&self.lock().events)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures::StreamExt as _;
    use proptest::prelude::*;

    use super::*;
    use crate::event::{EventKind, Source};
    use crate::id::WorldId;
    use mu_config::stream::Streams;

    fn new_event(world: &str, text: &str) -> NewEvent {
        NewEvent {
            world: world.parse::<WorldId>().unwrap(),
            kind: EventKind::Line,
            source: Source::Game,
            text: text.to_owned(),
            depth: 0,
            streams: Streams::main(),
        }
    }

    /// A source of random bits that gives only one bits.
    struct Ones;

    impl rand_core::TryRng for Ones {
        type Error = std::convert::Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            Ok(u32::MAX)
        }

        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            Ok(u64::MAX)
        }

        fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
            dst.fill(!0);
            Ok(())
        }
    }

    #[test]
    fn an_id_after_a_full_millisecond_keeps_the_order() {
        let journal = MemoryJournal::new(DEFAULT_CAPACITY).unwrap();
        let later = SystemTime::now() + std::time::Duration::from_secs(3600);
        let full = journal
            .lock()
            .ids
            .generate_from_datetime_with_source(later, &mut Ones)
            .unwrap();
        let first = journal.append(new_event("alpha/aria", "one")).id;
        let second = journal.append(new_event("alpha/aria", "two")).id;
        assert!(full < first && first < second, "{full} {first} {second}");
    }

    fn journal_of(n: u64) -> MemoryJournal {
        let journal = MemoryJournal::new(DEFAULT_CAPACITY).unwrap();
        for i in 1..=n {
            journal.append(new_event("alpha/aria", &i.to_string()));
        }
        journal
    }

    /// Every event that is ready now, as sequence numbers.
    fn ready(subscription: &mut Subscription) -> Vec<u64> {
        std::iter::from_fn(|| subscription.try_next())
            .map(|item| item.expect("no lag").seq.0)
            .collect()
    }

    #[test]
    fn sequence_has_no_gap() {
        let journal = MemoryJournal::new(DEFAULT_CAPACITY).unwrap();
        let seqs: Vec<u64> = (0..10)
            .map(|i| {
                let world = if i % 2 == 0 {
                    "alpha/aria"
                } else {
                    "beta/bryn"
                };
                journal.append(new_event(world, "x")).seq.0
            })
            .collect();
        assert_eq!(seqs, (1..=10).collect::<Vec<_>>());
        assert_eq!(journal.last(), Seq(10));
    }

    #[test]
    fn subscribe_from_middle() {
        let journal = journal_of(6);
        let mut subscription = journal.subscribe(Seq(4)).unwrap();
        assert_eq!(subscription.last(), Seq(6));
        assert_eq!(ready(&mut subscription), [4, 5, 6]);
        journal.append(new_event("alpha/aria", "7"));
        assert_eq!(ready(&mut subscription), [7]);
        assert_eq!(subscription.last(), Seq(6));
    }

    #[test]
    fn subscribe_from_zero() {
        let journal = journal_of(6);
        let mut subscription = journal.subscribe(Seq(0)).unwrap();
        assert_eq!(ready(&mut subscription), [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn subscribe_from_next() {
        let journal = journal_of(6);
        let mut subscription = journal.subscribe(Seq(7)).unwrap();
        assert_eq!(subscription.last(), Seq(6));
        assert_eq!(ready(&mut subscription), Vec::<u64>::new());
        journal.append(new_event("alpha/aria", "7"));
        assert_eq!(ready(&mut subscription), [7]);
    }

    #[test]
    fn subscribe_past_end() {
        let journal = journal_of(6);
        assert_eq!(
            journal.subscribe(Seq(8)).unwrap_err(),
            PortalError::PastEnd {
                from: Seq(8),
                last: Seq(6)
            }
        );
    }

    #[test]
    fn capacity_rules() {
        for (capacity, accepted) in [
            (0, false),
            (3, false),
            (4, true),
            (65_536, true),
            (131_072, false),
        ] {
            assert_eq!(
                MemoryJournal::new(capacity).is_ok(),
                accepted,
                "capacity {capacity}"
            );
        }
    }

    /// A subscription from 1 that read nothing while `events` events went in.
    fn unread_after(events: u64) -> Subscription {
        let journal = MemoryJournal::new(4).unwrap();
        let subscription = journal.subscribe(Seq(1)).unwrap();
        for i in 0..events {
            journal.append(new_event("alpha/aria", &i.to_string()));
        }
        subscription
    }

    #[test]
    fn receiver_at_capacity_is_not_cut() {
        let mut subscription = unread_after(4);
        let items: Vec<_> = std::iter::from_fn(|| subscription.try_next()).collect();
        let seqs: Vec<Seq> = items.into_iter().map(|item| item.unwrap().seq).collect();
        assert_eq!(seqs, [Seq(1), Seq(2), Seq(3), Seq(4)]);
    }

    #[test]
    fn receiver_past_capacity_is_cut() {
        assert_eq!(unread_after(5).try_next().unwrap().unwrap_err(), Lagged(1));
    }

    #[test]
    fn stream_ends_after_a_lag() {
        let journal = MemoryJournal::new(4).unwrap();
        let subscription = journal.subscribe(Seq(1)).unwrap();
        for i in 0..5 {
            journal.append(new_event("alpha/aria", &i.to_string()));
        }
        drop(journal);
        let items: Vec<_> = futures::executor::block_on(subscription.into_stream().collect());
        assert_eq!(items, [Err(Lagged(1))]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn subscribe_takes_backlog_and_live_under_one_lock() {
        const TOTAL: u64 = 2_000;
        let journal = Arc::new(MemoryJournal::new(4_096).unwrap());
        let writer = {
            let journal = Arc::clone(&journal);
            tokio::task::spawn_blocking(move || {
                for i in 0..TOTAL {
                    journal.append(new_event("alpha/aria", &i.to_string()));
                }
            })
        };
        let mut subscriptions = Vec::new();
        while subscriptions.len() < 64 && !writer.is_finished() {
            subscriptions.push(journal.subscribe(Seq(1)).unwrap());
            tokio::task::yield_now().await;
        }
        // The writer can finish first. A subscription after the end still checks the backlog.
        subscriptions.push(journal.subscribe(Seq(1)).unwrap());
        writer.await.unwrap();
        for mut subscription in subscriptions {
            assert_eq!(ready(&mut subscription), (1..=TOTAL).collect::<Vec<_>>());
        }
    }

    #[derive(Clone, Debug)]
    enum Op {
        Append,
        Subscribe(u64),
    }

    proptest! {
        #[test]
        fn ids_sort_in_the_order_of_seq(count in 1_u64..2000) {
            let journal = journal_of(count);
            let events = journal.lock().events.clone();
            prop_assert!(events.windows(2).all(|pair| pair[0].id < pair[1].id));
        }

        #[test]
        fn resume_has_no_gap_for_any_cut(total in 0_u64..60, cut in 0_u64..60, later in 0_u64..10) {
            let cut = cut.min(total);
            let journal = journal_of(total);
            let mut first = journal.subscribe(Seq(1)).unwrap();
            let mut seen: Vec<u64> = ready(&mut first).into_iter().take(usize::try_from(cut).unwrap()).collect();
            drop(first);
            for i in 0..later {
                journal.append(new_event("beta/bryn", &i.to_string()));
            }
            let mut second = journal.subscribe(Seq(seen.last().copied().unwrap_or(0) + 1)).unwrap();
            seen.extend(ready(&mut second));
            prop_assert_eq!(seen, (1..=total + later).collect::<Vec<_>>());
        }

        #[test]
        fn interleaved_appends_and_subscribes_keep_order(
            ops in prop::collection::vec(
                prop_oneof![Just(Op::Append), (0_u64..40).prop_map(Op::Subscribe)],
                0..120,
            )
        ) {
            let journal = MemoryJournal::new(DEFAULT_CAPACITY).unwrap();
            let mut subscriptions = Vec::new();
            for op in ops {
                match op {
                    Op::Append => {
                        journal.append(new_event("alpha/aria", "x"));
                    }
                    Op::Subscribe(from) => {
                        if let Ok(subscription) = journal.subscribe(Seq(from)) {
                            subscriptions.push((from.max(1), subscription));
                        } else {
                            prop_assert!(from > journal.last().0 + 1);
                        }
                    }
                }
            }
            let last = journal.last().0;
            for (from, mut subscription) in subscriptions {
                prop_assert_eq!(ready(&mut subscription), (from..=last).collect::<Vec<_>>());
            }
        }
    }
}
