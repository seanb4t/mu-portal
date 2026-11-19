//! The scenarios of the world-connection capability that run on the built binary, with a
//! scripted fake game over TCP.

use mu_core::telnet::{DO, IAC, SB, SE, WILL, opt};
use mu_fake_server::{Listening, Script, ScriptedGame};
use mu_proto::{EventKind, WorldState};

use support::ConfigDir;
use support::process::{BOUND, PW, ServerBuilder, Session};

mod support;

/// Serves `script` over TCP.
async fn serve(script: Script) -> (ScriptedGame, Listening) {
    let game = ScriptedGame::new(script);
    let listening = game.listen().await.unwrap();
    (game, listening)
}

/// Starts a server with the configuration `conf`, the fake world `beta/bryn`, and the
/// environment `env`. Waits for the greeting of `beta/bryn`.
async fn start(conf: &ConfigDir, env: &[(&str, Option<&str>)]) -> Session {
    let mut builder = ServerBuilder::new()
        .fake_world("beta/bryn", "PW", PW)
        .arg("--config")
        .arg(conf.path());
    for (name, value) in env {
        builder = builder.env(*name, *value);
    }
    let mut session = Session::start(builder).await;
    session.welcome("beta/bryn").await;
    session
}

/// Sends `mark` to `beta/bryn` and checks that its input event comes right after the last event
/// seen, so the server recorded no event in between.
async fn no_event_since_start(session: &mut Session) {
    let last = session.seen().last().map_or(0, |event| event.seq);
    let seq = session
        .client
        .send("beta/bryn", "mark")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(seq, last + 1, "an event came after {last}");
}

/// Checks that `ListWorlds` gives `alpha/aria` in `state`.
async fn assert_state(session: &Session, state: WorldState) {
    let worlds = session.client.list_worlds().await.unwrap();
    let aria = worlds
        .into_iter()
        .find(|entry| entry.id == "alpha/aria")
        .unwrap();
    assert!(aria.state == state, "{:?}", aria.state);
}

/// Waits until the game read `bytes` on its first connection.
async fn game_got(game: &ScriptedGame, bytes: &[u8]) {
    let got = game.wait_reads(|reads| {
        reads.first().is_some_and(|reads| {
            let all: Vec<u8> = reads.iter().flat_map(|(_, read)| read.clone()).collect();
            all.windows(bytes.len()).any(|window| window == bytes)
        })
    });
    tokio::time::timeout(BOUND, got)
        .await
        .unwrap_or_else(|_| panic!("the game got no {bytes:?}: {:?}", game.written()));
}

#[tokio::test]
async fn world_reaches_its_game() {
    let (_game, listening) = serve(Script::new()).await;
    let conf = ConfigDir::tcp_world(listening.addr().port(), "");
    let mut session = start(&conf, &[]).await;
    session.client.connect("alpha/aria").await.unwrap();
    session
        .kind("alpha/aria", EventKind::EVENT_KIND_CONNECTED, 0)
        .await;
    assert_state(&session, WorldState::WORLD_STATE_CONNECTED).await;
}

#[tokio::test]
async fn nothing_listens_on_the_port() {
    let (_game, listening) = serve(Script::new()).await;
    let port = listening.addr().port();
    listening.stop().await;
    let conf = ConfigDir::tcp_world(port, "[reconnect]\nmode = \"off\"\n");
    let mut session = start(&conf, &[]).await;
    session.client.connect("alpha/aria").await.unwrap();
    let down = session
        .kind("alpha/aria", EventKind::EVENT_KIND_DISCONNECTED, 0)
        .await;
    assert!(
        down.text.starts_with("the connect failed:"),
        "{}",
        down.text
    );
    assert_state(&session, WorldState::WORLD_STATE_DISCONNECTED).await;
}

#[tokio::test]
async fn tls_world_is_refused() {
    let (game, listening) = serve(Script::new()).await;
    let conf = ConfigDir::tcp_world(listening.addr().port(), "tls = true\n");
    let mut session = start(&conf, &[]).await;
    let error = session.client.connect("alpha/aria").await.unwrap_err();
    assert_eq!(error.code, "failed_precondition");
    assert!(
        error.message.contains("alpha/aria") && error.message.contains("TLS"),
        "{}",
        error.message
    );
    no_event_since_start(&mut session).await;
    assert_state(&session, WorldState::WORLD_STATE_DISCONNECTED).await;
    assert_eq!(game.connects(), 0);
}

#[tokio::test]
async fn latin1_game() {
    let cafe = b"caf\xe9\r\n";
    let (game, listening) = serve(Script::new().send(cafe)).await;
    let conf = ConfigDir::tcp_world(listening.addr().port(), "charset = \"iso-8859-1\"\n");
    let mut session = start(&conf, &[]).await;
    session.client.connect("alpha/aria").await.unwrap();
    session.line("alpha/aria", "café").await;
    session.client.send("alpha/aria", "café").await.unwrap();
    game_got(&game, cafe).await;
}

#[tokio::test]
async fn window_size_of_the_world() {
    let (game, listening) = serve(Script::new().send([IAC, DO, opt::NAWS])).await;
    let conf = ConfigDir::tcp_world(listening.addr().port(), "columns = 100\nrows = 30\n");
    let session = start(&conf, &[]).await;
    session.client.connect("alpha/aria").await.unwrap();
    game_got(&game, &[IAC, WILL, opt::NAWS]).await;
    game_got(&game, &[IAC, SB, opt::NAWS, 0, 100, 0, 30, IAC, SE]).await;
}

/// The character file of `alpha/aria` with the login name `Aria` and the secret `env:ARIA_PW`.
const ARIA_LOGIN: &str = "[login]\nname = \"Aria\"\nsecret = \"env:ARIA_PW\"\n";

#[tokio::test]
async fn missing_secret_variable_at_connect() {
    let (game, listening) = serve(Script::new()).await;
    let conf = ConfigDir::tcp_world(listening.addr().port(), ARIA_LOGIN);
    let mut session = start(&conf, &[("ARIA_PW", None)]).await;
    let error = session.client.connect("alpha/aria").await.unwrap_err();
    assert_eq!(error.code, "failed_precondition");
    assert!(error.message.contains("ARIA_PW"), "{}", error.message);
    no_event_since_start(&mut session).await;
    assert_eq!(game.connects(), 0);
}

#[tokio::test]
async fn login_line_hides_the_password() {
    let login = format!("connect Aria {PW}\r\n");
    let (_game, listening) = serve(Script::new().wait_for(&login).send("welcome\r\n")).await;
    let conf = ConfigDir::tcp_world(listening.addr().port(), ARIA_LOGIN);
    let mut session = start(&conf, &[("ARIA_PW", Some(PW))]).await;
    session.client.connect("alpha/aria").await.unwrap();
    // The game sends the line only after it got the login line with the password.
    session.line("alpha/aria", "welcome").await;
    let leaks: Vec<_> = session
        .seen()
        .iter()
        .filter(|event| event.text.contains(PW))
        .collect();
    assert!(leaks.is_empty(), "{leaks:?}");
}

#[tokio::test]
async fn game_close_reconnects() {
    let (_game, listening) = serve(Script::new().close()).await;
    let conf = ConfigDir::tcp_world(listening.addr().port(), "");
    let mut session = start(&conf, &[]).await;
    session.client.connect("alpha/aria").await.unwrap();
    let drop = session
        .kind("alpha/aria", EventKind::EVENT_KIND_DISCONNECTED, 0)
        .await;
    // The world sets its state before it records the drop, and then waits at least 1 second.
    assert_state(&session, WorldState::WORLD_STATE_RECONNECTING).await;
    session
        .kind("alpha/aria", EventKind::EVENT_KIND_CONNECTED, drop.seq)
        .await;
}

#[tokio::test]
async fn player_disconnect_does_not_reconnect() {
    let (game, listening) = serve(Script::new()).await;
    let conf = ConfigDir::tcp_world(listening.addr().port(), "");
    let mut session = start(&conf, &[]).await;
    session.client.connect("alpha/aria").await.unwrap();
    session
        .kind("alpha/aria", EventKind::EVENT_KIND_CONNECTED, 0)
        .await;
    session.client.disconnect("alpha/aria").await.unwrap();
    let gone = session
        .kind("alpha/aria", EventKind::EVENT_KIND_DISCONNECTED, 0)
        .await;
    assert_state(&session, WorldState::WORLD_STATE_DISCONNECTED).await;
    session.no_connect_after("alpha/aria", gone.seq).await;
    assert_eq!(game.connects(), 1);
}

#[tokio::test]
async fn reconnect_off_stays_disconnected() {
    let (_game, listening) = serve(Script::new().close()).await;
    let conf = ConfigDir::tcp_world(listening.addr().port(), "[reconnect]\nmode = \"off\"\n");
    let mut session = start(&conf, &[]).await;
    session.client.connect("alpha/aria").await.unwrap();
    session
        .kind("alpha/aria", EventKind::EVENT_KIND_DISCONNECTED, 0)
        .await;
    assert_state(&session, WorldState::WORLD_STATE_DISCONNECTED).await;
}
