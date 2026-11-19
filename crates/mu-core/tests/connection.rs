//! The scenarios of the world-connection capability that run on the engine with a paused clock.

use std::ffi::OsString;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::future::BoxFuture;
use mu_core::connection::Quiet;
use mu_core::telnet::{IAC, MARK, WILL, opt};
use mu_core::{
    Backoff, ByteStream, Command, EnvSecretStore, Envelope, EventKind, Keepalive, KeepaliveMode,
    Portal, PortalError, Reconnect, ResolvedWorld, Secret, SecretRef, SecretSource, Transport,
    WorldId, WorldSetup, WorldState,
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

/// The longest wait of a test on the paused clock.
const LONG_BOUND: Duration = Duration::from_secs(3_600);

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
    world.keepalive = Keepalive::new("IDLE", 60).unwrap().with_mode(mode);
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
    tokio::time::timeout(LONG_BOUND, enough).await.unwrap();
    (portal, idle_times(&game.reads(), start))
}

fn seconds(list: &[u64]) -> Vec<Duration> {
    list.iter().copied().map(Duration::from_secs).collect()
}

#[tokio::test(start_paused = true)]
async fn keepalive_idle() {
    // The send at 30 moves the first line to 90, and each keepalive line starts the idle time
    // again.
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
    tokio::time::timeout(LONG_BOUND, read).await.unwrap();
    let (time, bytes) = game.reads()[0][0].clone();
    assert_eq!(time - start, Duration::from_secs(10));
    assert!(bytes.starts_with(LOGIN), "{bytes:?}");
}

/// The tokio timer fires on a whole millisecond, so a wait can end up to this much later.
const TICK: Duration = Duration::from_millis(1);

/// More refusals than any test reaches.
const REFUSE_ALL: usize = 1_000;

/// A reconnect from 1 second up to 60, which stops after `attempts`.
fn backoff(attempts: u32) -> Reconnect {
    Reconnect::Backoff(Backoff::new(1, 60, attempts).unwrap())
}

/// Starts a portal with `alpha/aria`, which has no login and `reconnect`, on `game`.
fn reconnect_portal(game: &ScriptedGame, reconnect: Reconnect) -> Portal {
    let world = ResolvedWorld {
        reconnect,
        ..ResolvedWorld::new(id("alpha/aria"))
    };
    portal(
        vec![WorldSetup::new(world, None, Arc::new(game.clone()))],
        Vec::new(),
    )
}

/// A portal whose world waits for attempt 1, after its game refused the first connect. The game
/// refuses every connect.
async fn waiting(attempts: u32) -> (Portal, ScriptedGame) {
    let game = ScriptedGame::new(Script::new());
    game.refuse(REFUSE_ALL);
    let portal = reconnect_portal(&game, backoff(attempts));
    portal.connect(&id("alpha/aria")).await.unwrap();
    game.wait_for_refused(1).await;
    wait_state(&portal, &id("alpha/aria"), WorldState::Reconnecting).await;
    (portal, game)
}

/// Waits for the refused connects up to `count` in all, and gives the time of each from `start`.
async fn refusal_times(game: &ScriptedGame, start: Instant, count: usize) -> Vec<Duration> {
    let mut times = Vec::new();
    for refused in 1..=count {
        tokio::time::timeout(LONG_BOUND, game.wait_for_refused(refused))
            .await
            .unwrap();
        times.push(start.elapsed());
    }
    times
}

/// Whether `wait` is a wait before attempt `k` of [`backoff`]: from `min(2^(k-1), 60)` seconds
/// up to 1 + `min(0.05 * k, 0.20)` times that.
fn within_bounds(wait: Duration, k: u32) -> bool {
    let base = Duration::from_secs(2_u64.pow(k - 1).min(60));
    let share = (0.05 * f64::from(k)).min(0.20);
    base <= wait && wait <= base.mul_f64(1.0 + share) + TICK
}

/// Waits for a disconnect event that names the count of attempts.
async fn gave_up(portal: &Portal, attempts: u32) {
    let count = format!("{attempts} attempts");
    wait_event(portal, |event| {
        event.kind == EventKind::Disconnected && event.text.contains(&count)
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn waits_grow_until_the_last_attempt() {
    let game = ScriptedGame::new(Script::new());
    game.refuse(REFUSE_ALL);
    let portal = reconnect_portal(&game, backoff(10));
    let start = Instant::now();
    portal.connect(&id("alpha/aria")).await.unwrap();
    let times = refusal_times(&game, start, 11).await;
    for (k, pair) in (1..).zip(times.windows(2)) {
        let wait = pair[1].checked_sub(pair[0]).unwrap();
        assert!(within_bounds(wait, k), "attempt {k} waited {wait:?}");
    }
    wait_state(&portal, &id("alpha/aria"), WorldState::Disconnected).await;
}

#[tokio::test(start_paused = true)]
async fn world_stops_after_last_attempt() {
    let game = ScriptedGame::new(Script::new());
    game.refuse(REFUSE_ALL);
    let portal = reconnect_portal(&game, backoff(3));
    portal.connect(&id("alpha/aria")).await.unwrap();
    gave_up(&portal, 3).await;
    assert_eq!(
        portal.state(&id("alpha/aria")).unwrap().borrow().state,
        WorldState::Disconnected
    );
    let more = tokio::time::timeout(Duration::from_secs(120), game.wait_for_refused(5)).await;
    assert!(more.is_err());
    assert_eq!((game.refused(), game.connects()), (4, 0));
}

#[tokio::test(start_paused = true)]
async fn short_connections_count_as_attempts() {
    let game = ScriptedGame::new(
        Script::new()
            .wait_for(LOGIN)
            .silent_for(Duration::from_secs(2))
            .close(),
    );
    let world = ResolvedWorld {
        reconnect: backoff(3),
        ..ResolvedWorld::with_default_login(id("alpha/aria"), "env:PW".parse().unwrap())
    };
    let setup = WorldSetup::new(
        world,
        Some(Secret::new(PW.to_owned())),
        Arc::new(game.clone()),
    );
    let portal = portal(vec![setup], Vec::new());
    portal.connect(&id("alpha/aria")).await.unwrap();
    gave_up(&portal, 3).await;
    assert_eq!(game.connects(), 4);
}

#[tokio::test(start_paused = true)]
async fn attempts_zero_never_stops() {
    let (portal, game) = waiting(0).await;
    tokio::time::timeout(LONG_BOUND, game.wait_for_refused(41))
        .await
        .unwrap();
    wait_state(&portal, &id("alpha/aria"), WorldState::Reconnecting).await;
}

/// The wait before the attempt after a connection that stays up for `up`. The world reconnects
/// after 3 failed attempts, and its game closes each connection after `up`.
async fn wait_after_up_for(up: Duration) -> Duration {
    let game = ScriptedGame::new(Script::new().silent_for(up).close());
    // The first connect and 3 attempts.
    game.refuse(4);
    let portal = reconnect_portal(&game, backoff(10));
    portal.connect(&id("alpha/aria")).await.unwrap();
    tokio::time::timeout(LONG_BOUND, game.wait_for_connects(1))
        .await
        .unwrap();
    let opened = Instant::now();
    tokio::time::timeout(LONG_BOUND, game.wait_for_connects(2))
        .await
        .unwrap();
    opened.elapsed().checked_sub(up).unwrap()
}

#[tokio::test(start_paused = true)]
async fn stable_connection_resets_the_count() {
    let wait = wait_after_up_for(Duration::from_secs(60)).await;
    assert!(wait <= Duration::from_millis(1_050) + TICK, "{wait:?}");
}

#[tokio::test(start_paused = true)]
async fn short_connection_keeps_the_count() {
    let wait = wait_after_up_for(Duration::from_secs(59)).await;
    assert!(wait >= Duration::from_secs(8), "{wait:?}");
}

#[tokio::test(start_paused = true)]
async fn secret_gone_at_reconnect_stops() {
    let values = Mutex::new(vec![None, Some(OsString::from(PW))]);
    let store = EnvSecretStore::with_lookup(move |_| values.lock().unwrap().pop().flatten());
    let reference: SecretRef = "env:PW".parse().unwrap();
    let world = ResolvedWorld {
        reconnect: backoff(10),
        ..ResolvedWorld::with_default_login(id("alpha/aria"), reference.clone())
    };
    let game = ScriptedGame::new(Script::new().wait_for(LOGIN).close());
    let setup = WorldSetup::new(world, None, Arc::new(game.clone()))
        .with_secret_source(SecretSource::Store(reference, Arc::new(store)));
    let portal = portal(vec![setup], Vec::new());
    portal.connect(&id("alpha/aria")).await.unwrap();
    wait_event(&portal, |event| {
        event.kind == EventKind::Disconnected && event.text.contains("env:PW")
    })
    .await;
    assert_eq!(
        portal.state(&id("alpha/aria")).unwrap().borrow().state,
        WorldState::Disconnected
    );
    assert_eq!((game.connects(), game.refused()), (1, 0));
}

#[tokio::test(start_paused = true)]
async fn connect_during_a_wait() {
    let (portal, game) = waiting(10).await;
    // The first connect and 4 attempts, so the world waits for attempt 5.
    tokio::time::timeout(LONG_BOUND, game.wait_for_refused(5))
        .await
        .unwrap();
    wait_state(&portal, &id("alpha/aria"), WorldState::Reconnecting).await;
    let call = Instant::now();
    portal.connect(&id("alpha/aria")).await.unwrap();
    tokio::time::timeout(LONG_BOUND, game.wait_for_refused(6))
        .await
        .unwrap();
    assert!(
        call.elapsed() < Duration::from_secs(1),
        "{:?}",
        call.elapsed()
    );
    let refused = Instant::now();
    tokio::time::timeout(LONG_BOUND, game.wait_for_refused(7))
        .await
        .unwrap();
    let wait = refused.elapsed();
    assert!(wait <= Duration::from_millis(1_050) + TICK, "{wait:?}");
}

#[tokio::test(start_paused = true)]
async fn disconnect_during_a_wait() {
    let (portal, game) = waiting(10).await;
    portal.disconnect(&id("alpha/aria")).await.unwrap();
    let again = tokio::time::timeout(Duration::from_secs(120), game.wait_for_refused(2)).await;
    assert!(again.is_err());
    assert_eq!(
        portal.state(&id("alpha/aria")).unwrap().borrow().state,
        WorldState::Disconnected
    );
}

#[tokio::test(start_paused = true)]
async fn wait_returns_during_reconnect_wait() {
    let (portal, game) = waiting(10).await;
    let start = Instant::now();
    idle(&portal).await;
    assert_eq!(start.elapsed(), Duration::ZERO);
    assert_eq!(game.refused(), 1);
}

#[tokio::test(start_paused = true)]
async fn send_to_reconnecting_world() {
    let (portal, _game) = waiting(10).await;
    let before = events(&portal).len();
    let refused = portal.send(&id("alpha/aria"), "hello").await;
    assert!(
        matches!(refused, Err(PortalError::NotConnected(_))),
        "{refused:?}"
    );
    assert_eq!(events(&portal).len(), before);
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

#[tokio::test(start_paused = true)]
async fn write_that_hangs() {
    let (portal, aria) = broken_world(bare_world(BrokenTransport::stalling_writes())).await;
    let start = Instant::now();
    assert_eq!(
        portal.send(&aria, "hello").await,
        Err(PortalError::NotConnected(aria.clone()))
    );
    assert_eq!(start.elapsed(), Duration::from_secs(10));
    let down = wait_event(&portal, |event| event.kind == EventKind::Disconnected).await;
    assert_eq!(
        down.text,
        "the connection failed: the write timed out after 10 seconds"
    );
    wait_state(&portal, &aria, WorldState::Disconnected).await;
}

#[tokio::test(start_paused = true)]
async fn failed_login_write_never_connects() {
    let world = ResolvedWorld {
        reconnect: Reconnect::Off,
        ..ResolvedWorld::with_default_login(id("alpha/aria"), "env:PW".parse().unwrap())
    };
    let setup = WorldSetup::new(
        world,
        Some(Secret::new(PW.to_owned())),
        Arc::new(BrokenTransport::refusing_writes()),
    );
    let portal = portal(vec![setup], Vec::new());
    portal.connect(&id("alpha/aria")).await.unwrap();
    wait_event(&portal, |event| event.kind == EventKind::Disconnected).await;
    let connected = of_kind(&portal, EventKind::Connected);
    assert!(connected.is_empty(), "{:#?}", events(&portal));
}

/// Echoes `text` to `alpha/aria` and gives the text of its event.
async fn echo(portal: &Portal, text: &str) -> String {
    let command = Command::Echo {
        world: id("alpha/aria"),
        text: text.to_owned(),
    };
    let seq = portal
        .submit(Envelope::system(command))
        .await
        .unwrap()
        .unwrap();
    wait_event(portal, |event| event.seq == seq).await.text
}

#[tokio::test(start_paused = true)]
async fn secret_from_the_store_lasts_one_connection() {
    let store = EnvSecretStore::with_lookup(|_| Some(OsString::from(PW)));
    let reference: SecretRef = "env:PW".parse().unwrap();
    let world = ResolvedWorld {
        reconnect: Reconnect::Off,
        ..ResolvedWorld::with_default_login(id("alpha/aria"), reference.clone())
    };
    let game = ScriptedGame::new(Script::new().wait_for(LOGIN));
    let setup = WorldSetup::new(world, None, Arc::new(game.clone()))
        .with_secret_source(SecretSource::Store(reference, Arc::new(store)));
    let portal = portal(vec![setup], Vec::new());
    portal.connect(&id("alpha/aria")).await.unwrap();
    wait_state(&portal, &id("alpha/aria"), WorldState::Connected).await;
    assert!(!echo(&portal, PW).await.contains(PW));
    portal.disconnect(&id("alpha/aria")).await.unwrap();
    wait_state(&portal, &id("alpha/aria"), WorldState::Disconnected).await;
    // With no connection, the session holds no secret to mask the text with.
    assert_eq!(echo(&portal, PW).await, PW);
}
