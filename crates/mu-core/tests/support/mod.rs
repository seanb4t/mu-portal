//! Helpers that the engine tests share. Each test runs on the paused clock.

#![allow(dead_code, reason = "each test binary uses a part of this module")]

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use futures::future::BoxFuture;
use mu_core::{
    ByteStream, Command, Event, EventKind, Handler, MemoryJournal, Portal, PortalError, Reconnect,
    ResolvedWorld, Secret, Seq, Transport, WorldId, WorldSetup, WorldState,
};
use mu_fake_server::{FakeGame, FakeTransport};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// The bound of each wait. On the paused clock it passes only when nothing else can run.
pub const BOUND: Duration = Duration::from_secs(30);

pub fn id(text: &str) -> WorldId {
    text.parse().unwrap()
}

/// A world on its own fake game, with `password` and the default login.
pub fn world(text: &str, password: &str) -> (WorldSetup, FakeGame) {
    world_on(
        text,
        password,
        FakeTransport::new(id(text).character(), password),
    )
}

pub fn world_on(text: &str, password: &str, transport: FakeTransport) -> (WorldSetup, FakeGame) {
    let game = transport.game();
    let resolved = ResolvedWorld {
        reconnect: Reconnect::Off,
        ..ResolvedWorld::with_default_login(id(text), "env:PW".parse().unwrap())
    };
    let setup = WorldSetup::new(
        resolved,
        Some(Secret::new(password.to_owned())),
        Arc::new(transport),
    );
    (setup, game)
}

pub fn portal(worlds: Vec<WorldSetup>, server_handlers: Vec<Arc<dyn Handler>>) -> Portal {
    Portal::start(
        Arc::new(MemoryJournal::new(mu_core::journal::DEFAULT_CAPACITY).unwrap()),
        worlds,
        server_handlers,
    )
}

/// Starts the portal, connects each world and waits for each greeting.
pub async fn started(worlds: Vec<WorldSetup>) -> Portal {
    let portal = portal(worlds, Vec::new());
    portal.connect_all().await;
    for (world, _) in portal.worlds() {
        let greeting = format!("Welcome, {}.", world.character());
        wait_event(&portal, |event| {
            event.world == world && event.text == greeting
        })
        .await;
    }
    portal
}

/// Every event in the journal now.
pub fn events(portal: &Portal) -> Vec<Event> {
    let mut subscription = portal.subscribe(Seq(1)).unwrap();
    std::iter::from_fn(|| subscription.try_next())
        .map(|item| item.unwrap())
        .collect()
}

/// Waits for the first event, from the start, that `check` accepts.
pub async fn wait_event(portal: &Portal, check: impl Fn(&Event) -> bool) -> Event {
    let mut subscription = portal.subscribe(Seq(1)).unwrap();
    let found = tokio::time::timeout(BOUND, async {
        while let Some(item) = subscription.next().await {
            let event = item.unwrap();
            if check(&event) {
                return Some(event);
            }
        }
        None
    })
    .await;
    match found {
        Ok(Some(event)) => event,
        _ => panic!("no such event; the journal holds {:#?}", events(portal)),
    }
}

/// Waits until `world` is in `state`.
pub async fn wait_state(portal: &Portal, world: &WorldId, state: WorldState) {
    let mut receiver = portal.state(world).unwrap();
    tokio::time::timeout(BOUND, receiver.wait_for(|now| *now == state))
        .await
        .unwrap_or_else(|_| panic!("{world} never got to {state:?}"))
        .unwrap();
}

/// Waits until the engine has no work left. Panics when the wait fails or passes `BOUND`.
pub async fn idle(portal: &Portal) {
    try_idle(portal).await.expect("the engine wait failed");
}

/// Like [`idle`], but gives the result of the wait.
pub async fn try_idle(portal: &Portal) -> Result<(), PortalError> {
    tokio::time::timeout(BOUND, portal.wait_idle())
        .await
        .unwrap_or_else(|_| {
            panic!(
                "the engine never went idle; the journal holds {:#?}",
                events(portal)
            )
        })
}

/// Runs `scenario` as a task, in line with the tasks of the engine.
pub async fn as_task<T: Send + 'static>(scenario: impl Future<Output = T> + Send + 'static) -> T {
    tokio::spawn(scenario).await.unwrap()
}

/// The events of `kind`.
pub fn of_kind(portal: &Portal, kind: EventKind) -> Vec<Event> {
    events(portal)
        .into_iter()
        .filter(|event| event.kind == kind)
        .collect()
}

/// A handler from a closure.
struct On<F>(F);

impl<F: Fn(&Event) -> Vec<Command> + Send + Sync + 'static> Handler for On<F> {
    fn on_event(&self, event: &Event) -> Vec<Command> {
        (self.0)(event)
    }
}

pub fn on(f: impl Fn(&Event) -> Vec<Command> + Send + Sync + 'static) -> Arc<dyn Handler> {
    Arc::new(On(f))
}

/// A transport whose stream lets `writes_allowed` writes through and then fails every write.
/// Once those writes are through, the stream gives `incoming` as one read and then no more.
pub struct BrokenTransport {
    writes_allowed: usize,
    incoming: &'static [u8],
}

impl BrokenTransport {
    /// A stream that gives no byte and fails every write.
    pub fn refusing_writes() -> Self {
        Self::after_writes(0, b"")
    }

    pub fn after_writes(writes_allowed: usize, incoming: &'static [u8]) -> Self {
        Self {
            writes_allowed,
            incoming,
        }
    }
}

impl Transport for BrokenTransport {
    fn connect(&self, _: &WorldId) -> BoxFuture<'static, io::Result<Box<dyn ByteStream>>> {
        let stream = BrokenStream {
            writes_left: self.writes_allowed,
            incoming: self.incoming,
            reader: None,
        };
        Box::pin(async move { Ok(Box::new(stream) as Box<dyn ByteStream>) })
    }
}

struct BrokenStream {
    writes_left: usize,
    incoming: &'static [u8],
    reader: Option<Waker>,
}

impl AsyncRead for BrokenStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.writes_left > 0 || self.incoming.is_empty() {
            self.reader = Some(cx.waker().clone());
            return Poll::Pending;
        }
        buf.put_slice(std::mem::take(&mut self.incoming));
        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for BrokenStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        if self.writes_left == 0 {
            return Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()));
        }
        self.writes_left -= 1;
        if self.writes_left == 0
            && let Some(reader) = self.reader.take()
        {
            reader.wake();
        }
        Poll::Ready(Ok(bytes.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

/// The world `alpha/aria` on `transport`, with no login and no password.
pub fn bare_world(transport: impl Transport) -> WorldSetup {
    let world = ResolvedWorld {
        reconnect: Reconnect::Off,
        ..ResolvedWorld::new(id("alpha/aria"))
    };
    WorldSetup::new(world, None, Arc::new(transport))
}

/// A started portal whose only world is up on `setup`, and the id of that world.
pub async fn broken_world(setup: WorldSetup) -> (Portal, WorldId) {
    let aria = id("alpha/aria");
    let portal = portal(vec![setup], Vec::new());
    portal.connect_all().await;
    wait_state(&portal, &aria, WorldState::Connected).await;
    (portal, aria)
}

/// `bytes` split into reads that end at `points`. Points out of order or past the end are
/// sorted and clamped.
pub fn reads<'a>(bytes: &'a [u8], points: &[usize]) -> Vec<&'a [u8]> {
    let mut points: Vec<usize> = points
        .iter()
        .map(|point| (*point).min(bytes.len()))
        .collect();
    points.sort_unstable();
    let mut out = Vec::new();
    let mut start = 0;
    for point in points.into_iter().chain([bytes.len()]) {
        out.push(&bytes[start..point]);
        start = point;
    }
    out
}

/// The reads of the game in a transcript of the telnet spike, in order.
///
/// Each record is one line: `<` for a read of the game or `>` for a write to it, the seconds
/// since the connect, and the hex of the bytes. A line that starts with `#` is a note.
pub fn transcript_reads(text: &str) -> Vec<Vec<u8>> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let mut fields = line.splitn(3, ' ');
            let direction = fields.next().unwrap();
            assert!(matches!(direction, "<" | ">"), "bad record: {line}");
            let _seconds: f64 = fields.next().unwrap().parse().unwrap();
            let bytes = hex::decode(fields.next().unwrap_or_default()).unwrap();
            (direction == "<").then_some(bytes)
        })
        .collect()
}

/// The text of each transcript under `tests/fixtures/telnet`, with its file name.
pub fn transcripts() -> Vec<(String, String)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/telnet");
    let mut files: Vec<(String, String)> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            (name, std::fs::read_to_string(&path).unwrap())
        })
        .collect();
    files.sort();
    files
}

/// The telnet events of `events`, with each run of adjacent data events joined into one.
pub fn join_data(
    events: impl IntoIterator<Item = mu_core::telnet::Event>,
) -> Vec<mu_core::telnet::Event> {
    use mu_core::telnet::Event::Data;
    let mut out = Vec::new();
    for event in events {
        match (out.last_mut(), event) {
            (Some(Data(last)), Data(more)) => last.extend(more),
            (_, event) => out.push(event),
        }
    }
    out
}

/// The bytes of a subnegotiation: `IAC SB option`, the escaped payload, `IAC SE`.
pub fn subnegotiation(option: u8, payload: &[u8]) -> Vec<u8> {
    use mu_core::telnet::{IAC, SB, SE, escape};
    [&[IAC, SB, option][..], &escape(payload), &[IAC, SE]].concat()
}

/// Gives each of `reads` to `parser` in order, and gives the joined events and the sent bytes.
pub fn parse_reads(
    parser: &mut mu_core::telnet::Parser,
    reads: &[&[u8]],
) -> (Vec<mu_core::telnet::Event>, Vec<u8>) {
    let mut events = Vec::new();
    let mut output = Vec::new();
    for read in reads {
        parser.receive(read);
        events.extend(std::iter::from_fn(|| parser.poll_event()));
        output.extend(parser.take_output());
    }
    (join_data(events), output)
}
