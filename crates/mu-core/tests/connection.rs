//! The scenarios of the world-connection capability that run on the engine with a paused clock.

use std::io;
use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use mu_core::connection::Quiet;
use mu_core::telnet::{IAC, MARK, WILL, opt};
use mu_core::{
    ByteStream, EventKind, Portal, PortalError, Reconnect, ResolvedWorld, Secret, Transport,
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
