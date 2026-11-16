//! The scenarios of the world-sessions capability, run on the built binary.

use mu_proto::{EventKind, Source, WorldState};

use support::process::{PW, ServerBuilder, Session, TWO_WORLDS};
use support::{mu, stderr, stdout};

mod support;

fn last(session: &Session) -> u64 {
    session.seen().last().map_or(0, |event| event.seq)
}

/// Sends `mark` to `world` and checks that its input event comes right after `after`, so the
/// server recorded no event in between.
async fn no_event_since(session: &mut Session, world: &str, after: u64) {
    let seq = session.client.send(world, "mark").await.unwrap().unwrap();
    assert_eq!(seq, after + 1, "an event came between {after} and the mark");
}

#[tokio::test]
async fn bad_world_id_is_refused() {
    let output = ServerBuilder::new()
        .fake_world("Alpha_1", "PW", PW)
        .output()
        .await;
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("Alpha_1"));
}

#[tokio::test]
async fn fake_world_answers() {
    let mut session = Session::with_worlds(&["alpha/aria"]).await;
    let welcome = session.welcome("alpha/aria").await;
    assert_eq!(welcome.source, Source::SOURCE_GAME);
    session.client.send("alpha/aria", "hello").await.unwrap();
    let answer = session.line("alpha/aria", "You say, \"hello\"").await;
    assert_eq!(answer.source, Source::SOURCE_GAME);
    assert!(answer.seq > welcome.seq);
}

#[test]
fn fake_world_flag_is_hidden() {
    let output = mu(&["server", "--help"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(!stdout(&output).contains("fake-world"));
}

#[tokio::test]
async fn missing_password_variable() {
    let output = ServerBuilder::new()
        .arg("--fake-world")
        .arg("alpha/aria=env:ALPHA_PW")
        .env("ALPHA_PW", None::<&str>)
        .output()
        .await;
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("ALPHA_PW"));
}

#[tokio::test]
async fn two_worlds_connect() {
    let mut session = Session::start(ServerBuilder::new().fake_worlds(TWO_WORLDS)).await;
    session
        .kind("alpha/aria", EventKind::EVENT_KIND_CONNECTED, 0)
        .await;
    session
        .kind("beta/bryn", EventKind::EVENT_KIND_CONNECTED, 0)
        .await;
    let worlds = session.client.list_worlds().await.unwrap();
    let states: Vec<(String, bool)> = worlds
        .iter()
        .map(|world| {
            (
                world.id.clone(),
                world.state == WorldState::WORLD_STATE_CONNECTED,
            )
        })
        .collect();
    assert_eq!(
        states,
        [
            ("alpha/aria".to_owned(), true),
            ("beta/bryn".to_owned(), true)
        ]
    );
}

#[tokio::test]
async fn two_characters_on_one_game() {
    let mut session = Session::with_worlds(&["alpha/aria", "alpha/ash"]).await;
    session.client.send("alpha/aria", "one").await.unwrap();
    session.line("alpha/aria", "You say, \"one\"").await;
    session.client.send("alpha/ash", "two").await.unwrap();
    session.line("alpha/ash", "You say, \"two\"").await;
    let ash = session
        .seen()
        .iter()
        .filter(|event| event.world == "alpha/ash");
    assert!(ash.clone().count() > 0);
    assert!(ash.into_iter().all(|event| !event.text.contains("one")));
}

#[tokio::test]
async fn send_to_unknown_world() {
    let mut session = Session::with_worlds(&["alpha/aria"]).await;
    let after = last(&session);
    let error = session
        .client
        .send("nowhere/nobody", "hello")
        .await
        .unwrap_err();
    assert_eq!(error.code, "not_found");
    no_event_since(&mut session, "alpha/aria", after).await;
}

#[tokio::test]
async fn send_to_disconnected_world() {
    let mut session = Session::with_worlds(&["alpha/aria"]).await;
    session.client.disconnect("alpha/aria").await.unwrap();
    let gone = session
        .kind("alpha/aria", EventKind::EVENT_KIND_DISCONNECTED, 0)
        .await;
    let error = session
        .client
        .send("alpha/aria", "hello")
        .await
        .unwrap_err();
    assert_eq!(error.code, "failed_precondition");
    session.client.connect("alpha/aria").await.unwrap();
    let back = session
        .kind("alpha/aria", EventKind::EVENT_KIND_CONNECTED, gone.seq)
        .await;
    assert_eq!(
        back.seq,
        gone.seq + 1,
        "an event came between the disconnect and the connect"
    );
}

async fn line_break_is_refused(text: &str) {
    let mut session = Session::with_worlds(&["alpha/aria"]).await;
    let after = last(&session);
    let error = session.client.send("alpha/aria", text).await.unwrap_err();
    assert_eq!(error.code, "invalid_argument");
    no_event_since(&mut session, "alpha/aria", after).await;
    let echo = session.line("alpha/aria", "You say, \"mark\"").await;
    let lines: Vec<&str> = session
        .seen()
        .iter()
        .filter(|event| event.seq > after && event.seq <= echo.seq)
        .map(|event| event.text.as_str())
        .collect();
    assert_eq!(
        lines,
        ["mark", "You say, \"mark\""],
        "the game got only the mark"
    );
}

#[tokio::test]
async fn send_with_lf_is_refused() {
    line_break_is_refused("one\ntwo").await;
}

#[tokio::test]
async fn send_with_cr_is_refused() {
    line_break_is_refused("one\rtwo").await;
}

#[tokio::test]
async fn empty_send_reaches_game() {
    let mut session = Session::with_worlds(&["alpha/aria"]).await;
    let seq = session
        .client
        .send("alpha/aria", "")
        .await
        .unwrap()
        .unwrap();
    let input = session
        .until("the empty input", |event| event.seq == seq)
        .await;
    assert_eq!(
        (input.kind, input.source, input.text.as_str()),
        (
            EventKind::EVENT_KIND_INPUT.into(),
            Source::SOURCE_PLAYER.into(),
            ""
        )
    );
    session.line("alpha/aria", "You say, \"\"").await;
}

#[tokio::test]
async fn one_world_drops() {
    let mut session = Session::with_worlds(TWO_WORLDS).await;
    session.client.disconnect("alpha/aria").await.unwrap();
    session
        .kind("alpha/aria", EventKind::EVENT_KIND_DISCONNECTED, 0)
        .await;
    session
        .client
        .send("beta/bryn", "still here")
        .await
        .unwrap();
    session.line("beta/bryn", "You say, \"still here\"").await;
}

#[tokio::test]
async fn world_connects_again() {
    let mut session = Session::with_worlds(&["alpha/aria"]).await;
    session.client.disconnect("alpha/aria").await.unwrap();
    let gone = session
        .kind("alpha/aria", EventKind::EVENT_KIND_DISCONNECTED, 0)
        .await;
    session.client.connect("alpha/aria").await.unwrap();
    session
        .kind("alpha/aria", EventKind::EVENT_KIND_CONNECTED, gone.seq)
        .await;
    session
        .kind("alpha/aria", EventKind::EVENT_KIND_LOGIN, gone.seq)
        .await;
    let worlds = session.client.list_worlds().await.unwrap();
    assert_eq!(worlds[0].state, WorldState::WORLD_STATE_CONNECTED);
}
