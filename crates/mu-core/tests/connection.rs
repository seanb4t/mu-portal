//! The scenarios of the world-connection capability that run on the engine with a paused clock.

use std::io;
use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use mu_core::connection::Quiet;
use mu_core::telnet::{IAC, MARK, WILL, opt};
use mu_core::{
    ByteStream, EventKind, Keepalive, KeepaliveMode, Portal, PortalError, Reconnect, ResolvedWorld,
    Secret, Transport, WorldId, WorldSetup, WorldState,
};
use mu_fake_server::{Script, ScriptedGame};
use tokio::time::Instant;

use support::*;

mod support;

const PW: &str = "s3cret-pw";

/// The login line of `alpha/aria` with the default template.
const LOGIN: &[u8] = b"connect aria s3cret-pw\r\n";

/// The answer of a game to a mark.
const MARK_ANSWER: [u8; 3] = [IAC, WILL, opt::TIMING_MARK];

/// `alpha/aria` on `game`, which waits for a quiet second before its login. With `login` false,
/// the world has no login.
fn quiet_world(game: &ScriptedGame, login: bool) -> WorldSetup {
    let aria = id("alpha/aria");
    let (world, password) = if login {
        let world = ResolvedWorld::with_default_login(aria, "env:PW".parse().unwrap());
        (world, Some(Secret::new(PW.to_owned())))
    } else {
        (ResolvedWorld::new(aria), None)
    };
    WorldSetup::new(world, password, Arc::new(game.clone()))
        .with_login_moment(Arc::new(Quiet::default()))
}

/// Starts a portal with `alpha/aria` on `game`, and connects it. Gives the portal and the time
/// of the connect.
async fn connected(game: &ScriptedGame, login: bool) -> (Portal, Instant) {
    let portal = portal(vec![quiet_world(game, login)], Vec::new());
    let start = Instant::now();
    portal.connect(&id("alpha/aria")).await.unwrap();
    (portal, start)
}

/// Waits for the login event, and gives the time from `start`.
async fn login_after(portal: &Portal, start: Instant) -> Duration {
    wait_event(portal, |event| event.kind == EventKind::Login).await;
    start.elapsed()
}

/// Waits for the line `text` from the game.
async fn line(portal: &Portal, text: &str) {
    wait_event(portal, |event| {
        event.kind == EventKind::Line && event.text == text
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn login_after_banner_and_quiet() {
    let game = ScriptedGame::new(
        Script::new()
            .send("one\r\n")
            .silent_for(Duration::from_millis(200))
            .send("two\r\n")
            .silent_for(Duration::from_millis(200))
            .send("three\r\n")
            .wait_for(LOGIN)
            .send("ready\r\n"),
    );
    let (portal, start) = connected(&game, true).await;
    assert_eq!(
        login_after(&portal, start).await,
        Duration::from_millis(1_400)
    );
    line(&portal, "ready").await;
    assert!(game.written()[0].starts_with(LOGIN), "{:?}", game.written());
}

#[tokio::test(start_paused = true)]
async fn login_after_a_silent_second() {
    let game = ScriptedGame::new(Script::new());
    let (portal, start) = connected(&game, true).await;
    assert_eq!(login_after(&portal, start).await, Duration::from_secs(1));
}

#[tokio::test(start_paused = true)]
async fn login_cap_for_a_noisy_game() {
    let noise = (0..40).fold(Script::new(), |script, _| {
        script
            .send("noise\r\n")
            .silent_for(Duration::from_millis(500))
    });
    let (portal, start) = connected(&ScriptedGame::new(noise), true).await;
    assert_eq!(login_after(&portal, start).await, Duration::from_secs(10));
}

#[tokio::test(start_paused = true)]
async fn send_during_quiet_second_is_refused() {
    let game = ScriptedGame::new(Script::new().wait_for(LOGIN).send("ready\r\n"));
    let (portal, _) = connected(&game, true).await;
    tokio::time::advance(Duration::from_millis(500)).await;
    let refused = portal.send(&id("alpha/aria"), "hello").await;
    assert!(
        matches!(refused, Err(PortalError::NotConnected(_))),
        "{refused:?}"
    );
    line(&portal, "ready").await;
    let written = &game.written()[0];
    assert!(written.starts_with(LOGIN), "{written:?}");
    assert!(
        !written.windows(5).any(|bytes| bytes == b"hello"),
        "{written:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn no_login_sends_no_line() {
    let game = ScriptedGame::new(Script::new().wait_for(MARK).send(MARK_ANSWER));
    let (portal, start) = connected(&game, false).await;
    wait_state(&portal, &id("alpha/aria"), WorldState::Connected).await;
    assert_eq!(start.elapsed(), Duration::ZERO);
    tokio::time::advance(Duration::from_secs(20)).await;
    idle(&portal).await;
    assert_eq!(game.written(), [MARK.to_vec()]);
}

#[tokio::test(start_paused = true)]
async fn wait_covers_pending_login() {
    // The wait marks the world once, and once more because the login added events.
    let game = ScriptedGame::new(
        Script::new()
            .wait_for(LOGIN)
            .wait_for(MARK)
            .send(MARK_ANSWER)
            .wait_for(MARK)
            .send(MARK_ANSWER),
    );
    let (portal, _) = connected(&game, true).await;
    idle(&portal).await;
    assert_eq!(of_kind(&portal, EventKind::Login).len(), 1);
}

/// The keepalive line on the wire.
const IDLE: &[u8] = b"IDLE\r\n";

/// The longest wait of a keepalive test on the paused clock.
const KEEPALIVE_BOUND: Duration = Duration::from_secs(300);

/// The time from `start` of each read of the game that holds the keepalive line.
fn idle_times(reads: &[Vec<(Instant, Vec<u8>)>], start: Instant) -> Vec<Duration> {
    reads
        .iter()
        .flatten()
        .filter(|(_, bytes)| bytes.windows(IDLE.len()).any(|window| window == IDLE))
        .map(|(time, _)| *time - start)
        .collect()
}

/// Starts a portal with `alpha/aria`, which has no login and the keepalive `IDLE` every 60
/// seconds in `mode`, on a game that sends nothing. Connects it, and gives the time of the move
/// to `connected`.
async fn keepalive_world(game: &ScriptedGame, mode: KeepaliveMode) -> (Portal, Instant) {
    let mut world = ResolvedWorld::new(id("alpha/aria"));
    world.keepalive = Some(Keepalive::new("IDLE", 60).unwrap().with_mode(mode));
    let portal = portal(
        vec![WorldSetup::new(world, None, Arc::new(game.clone()))],
        Vec::new(),
    );
    let start = Instant::now();
    portal.connect(&id("alpha/aria")).await.unwrap();
    wait_state(&portal, &id("alpha/aria"), WorldState::Connected).await;
    assert_eq!(start.elapsed(), Duration::ZERO);
    (portal, start)
}

/// Connects the keepalive world in `mode` at the time 0, sends `hello` at 30, and waits until
/// the game read `count` keepalive lines. Gives the portal and the time of each keepalive line.
async fn keepalive_times(mode: KeepaliveMode, count: usize) -> (Portal, Vec<Duration>) {
    let game = ScriptedGame::new(Script::new());
    let (portal, start) = keepalive_world(&game, mode).await;
    tokio::time::advance(Duration::from_secs(30)).await;
    portal.send(&id("alpha/aria"), "hello").await.unwrap();
    let enough = game.wait_reads(|reads| idle_times(reads, start).len() >= count);
    tokio::time::timeout(KEEPALIVE_BOUND, enough).await.unwrap();
    (portal, idle_times(&game.reads(), start))
}

fn seconds(list: &[u64]) -> Vec<Duration> {
    list.iter().copied().map(Duration::from_secs).collect()
}

#[tokio::test(start_paused = true)]
async fn keepalive_idle() {
    // The third line at 210 shows that no line came between 150 and 200.
    let (_, times) = keepalive_times(KeepaliveMode::Idle, 3).await;
    assert_eq!(times, seconds(&[90, 150, 210]));
}

#[tokio::test(start_paused = true)]
async fn keepalive_interval() {
    let (_, times) = keepalive_times(KeepaliveMode::Interval, 4).await;
    let every_minute: Vec<Duration> = (1..=4).map(|n| Duration::from_secs(60 * n)).collect();
    assert_eq!(times, every_minute);
}

#[tokio::test(start_paused = true)]
async fn keepalive_off() {
    let game = ScriptedGame::new(Script::new());
    let (_, start) = keepalive_world(&game, KeepaliveMode::Off).await;
    let any = game.wait_reads(|reads| !idle_times(reads, start).is_empty());
    let waited = tokio::time::timeout(Duration::from_secs(200), any).await;
    assert!(waited.is_err(), "{:?}", game.reads());
}

#[tokio::test(start_paused = true)]
async fn journal_holds_no_keepalive() {
    let (portal, _) = keepalive_times(KeepaliveMode::Idle, 3).await;
    let journal = events(&portal);
    assert!(
        journal.iter().any(|event| event.text.contains("hello")),
        "{journal:#?}"
    );
    assert!(
        !journal.iter().any(|event| event.text.contains("IDLE")),
        "{journal:#?}"
    );
}

#[tokio::test(start_paused = true)]
async fn no_keepalive_before_login() {
    let noise = (0..40).fold(Script::new(), |script, _| {
        script
            .send("noise\r\n")
            .silent_for(Duration::from_millis(500))
    });
    let game = ScriptedGame::new(noise);
    let mut world = ResolvedWorld::with_default_login(id("alpha/aria"), "env:PW".parse().unwrap());
    world.keepalive = Some(Keepalive::new("IDLE", 1).unwrap());
    let setup = WorldSetup::new(
        world,
        Some(Secret::new(PW.to_owned())),
        Arc::new(game.clone()),
    )
    .with_login_moment(Arc::new(Quiet::default()));
    let portal = portal(vec![setup], Vec::new());
    let start = Instant::now();
    portal.connect(&id("alpha/aria")).await.unwrap();
    let read = game.wait_reads(|reads| reads.first().is_some_and(|reads| !reads.is_empty()));
    tokio::time::timeout(KEEPALIVE_BOUND, read).await.unwrap();
    let (time, bytes) = game.reads()[0][0].clone();
    assert_eq!(time - start, Duration::from_secs(10));
    assert!(bytes.starts_with(LOGIN), "{bytes:?}");
}

/// A transport whose connect never ends.
struct Hanging;

impl Transport for Hanging {
    fn connect(&self, _: &WorldId) -> BoxFuture<'static, io::Result<Box<dyn ByteStream>>> {
        Box::pin(std::future::pending())
    }
}

#[tokio::test(start_paused = true)]
async fn connect_that_hangs() {
    let mut world = ResolvedWorld::new(id("alpha/aria"));
    world.reconnect = Reconnect::Off;
    let portal = portal(
        vec![WorldSetup::new(world, None, Arc::new(Hanging))],
        Vec::new(),
    );
    let start = Instant::now();
    portal.connect(&id("alpha/aria")).await.unwrap();
    let down = wait_event(&portal, |event| event.kind == EventKind::Disconnected).await;
    assert_eq!(start.elapsed(), Duration::from_secs(10));
    assert!(
        down.text.starts_with("the connect failed:") && down.text.contains("timed out"),
        "{}",
        down.text
    );
}
