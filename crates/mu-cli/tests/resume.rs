//! The resume of `PortalClient` after a cut connection, a restart and a clean end, through
//! the relay.

use std::pin::Pin;

use futures::{Stream, StreamExt as _};
use mu_client::{ClientEvent, PortalClient};
use rustix::process::Signal;

use support::process::{BOUND, PW, Relay, ServerBuilder, ServerProcess, wait_until};

mod support;

type Events = Pin<Box<dyn Stream<Item = ClientEvent> + Send>>;

fn builder() -> ServerBuilder {
    ServerBuilder::new().fake_world("alpha/aria", "ALPHA_PW", PW)
}

/// The server, a relay in front of it, and a client stream through the relay.
struct Rig {
    server: ServerProcess,
    relay: Relay,
    events: Events,
    seen: Vec<ClientEvent>,
}

impl Rig {
    async fn start() -> Self {
        let server = builder().start().await;
        let relay = Relay::start(&server.dir().path().join("relay.sock"), server.socket());
        let events = Box::pin(PortalClient::unix(relay.socket()).events());
        Self {
            server,
            relay,
            events,
            seen: Vec::new(),
        }
    }

    /// Reads the stream to the first item that `check` accepts, and returns it.
    async fn until(&mut self, what: &str, check: impl Fn(&ClientEvent) -> bool) -> ClientEvent {
        wait_until(BOUND, what, async {
            loop {
                let item = self.events.next().await.expect("the stream has no end");
                self.seen.push(item.clone());
                if check(&item) {
                    return item;
                }
            }
        })
        .await
    }

    async fn line(&mut self, text: &str) {
        self.until(
            text,
            |item| matches!(item, ClientEvent::Event(event) if event.text == text),
        )
        .await;
    }

    /// Sends `text` to the world on a direct client, not through the relay.
    async fn send(&self, text: &str) {
        self.server.client().send("alpha/aria", text).await.unwrap();
    }

    fn seqs(items: &[ClientEvent]) -> Vec<u64> {
        items
            .iter()
            .filter_map(|item| match item {
                ClientEvent::Event(event) => Some(event.seq),
                _ => None,
            })
            .collect()
    }
}

fn instance(item: &ClientEvent) -> String {
    match item {
        ClientEvent::Reset { instance_id } => instance_id.clone(),
        other => panic!("not a reset: {other:?}"),
    }
}

#[tokio::test]
async fn resume_after_cut_connection() {
    let mut rig = Rig::start().await;
    rig.line("Welcome, aria.").await;
    // The hold keeps the client out until the server recorded the events of the cut.
    rig.relay.hold();
    assert!(rig.relay.cut() >= 1, "no connection to cut");
    rig.until("the loss", |item| *item == ClientEvent::Lost)
        .await;
    rig.send("one").await;
    rig.send("two").await;
    let during = rig.server.client().list_worlds().await.unwrap();
    assert_eq!(during.len(), 1);
    rig.relay.release();
    rig.until("the resume", |item| *item == ClientEvent::Resumed)
        .await;
    rig.line("You say, \"one\"").await;
    rig.line("You say, \"two\"").await;
    let before = Rig::seqs(
        &rig.seen[..rig
            .seen
            .iter()
            .position(|item| *item == ClientEvent::Lost)
            .unwrap()],
    );
    let after = Rig::seqs(
        &rig.seen[rig
            .seen
            .iter()
            .position(|item| *item == ClientEvent::Lost)
            .unwrap()..],
    );
    assert!(
        after.len() >= 4,
        "the client read the events of the cut after the resume: {after:?}"
    );
    assert!(before.iter().all(|seq| !after.contains(seq)));
    let seqs = Rig::seqs(&rig.seen);
    let want: Vec<u64> = (1..=seqs.len() as u64).collect();
    assert_eq!(seqs, want, "each number once, in order, with no gap");
}

/// Reads events 1 to 5, with `before`, stops the server with `signal`, starts a new one on
/// the same socket, and checks that the client resets and holds only the new events.
async fn resets_after(signal: Signal) {
    let mut rig = Rig::start().await;
    let first = rig
        .until("the first hello", |item| {
            matches!(item, ClientEvent::Reset { .. })
        })
        .await;
    rig.send("before").await;
    rig.line("You say, \"before\"").await;
    assert_eq!(Rig::seqs(&rig.seen), [1, 2, 3, 4, 5]);

    rig.server.signal(signal);
    rig.server.wait_exit().await;
    // The old stream ends first. On SIGTERM it carries the disconnect of the world.
    rig.until("the loss", |item| *item == ClientEvent::Lost)
        .await;
    let dir = std::sync::Arc::clone(rig.server.dir());
    rig.server = builder().in_dir(&dir).start().await;
    rig.send("after").await;

    let mark = rig.seen.len();
    let reset = rig
        .until("the reset", |item| {
            matches!(item, ClientEvent::Reset { .. })
        })
        .await;
    assert_ne!(instance(&reset), instance(&first));
    rig.line("You say, \"after\"").await;
    let after = &rig.seen[mark..];
    assert_eq!(Rig::seqs(after).first(), Some(&1));
    assert!(
        after.iter().all(
            |item| !matches!(item, ClientEvent::Event(event) if event.text.contains("before"))
        )
    );
}

#[tokio::test]
async fn resume_after_server_restart() {
    resets_after(Signal::KILL).await;
}

#[tokio::test]
async fn resume_after_clean_end() {
    resets_after(Signal::TERM).await;
}
