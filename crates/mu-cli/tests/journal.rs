//! The scenarios of the event-journal capability, run on the built binary.

use mu_proto::__buffa::oneof::subscribe_response::Message;
use mu_proto::{EventKind, Source, SubscribeResponse};

use support::process::{BOUND, Session, private_tempdir, wait_until};
use support::{mu_with, stderr};

mod support;

const PW: &str = "s3cret-pw";

fn hello(message: Option<SubscribeResponse>) -> (String, u64) {
    match message.and_then(|message| message.message) {
        Some(Message::Hello(hello)) => (hello.instance_id, hello.last_seq),
        other => panic!("not a Hello: {other:?}"),
    }
}

fn seq(message: Option<SubscribeResponse>) -> u64 {
    match message.and_then(|message| message.message) {
        Some(Message::Event(event)) => event.seq,
        other => panic!("not an event: {other:?}"),
    }
}

/// A server whose journal holds the events 1 to 6: the connect, login and greeting of two
/// worlds.
async fn six_events() -> Session {
    let session = Session::with_worlds(&[("alpha/aria", PW), ("beta/bryn", PW)]).await;
    assert_eq!(session.seen().last().map(|event| event.seq), Some(6));
    session
}

fn no_password(session: &Session, password: &str) {
    for event in session.seen() {
        assert!(
            !event.text.contains(password),
            "event {} holds the password: {:?}",
            event.seq,
            event.text
        );
    }
}

#[tokio::test]
async fn events_of_two_worlds_share_one_sequence() {
    let mut session = Session::with_worlds(&[("alpha/aria", PW), ("beta/bryn", PW)]).await;
    session.client.send("alpha/aria", "one").await.unwrap();
    session.client.send("beta/bryn", "two").await.unwrap();
    session.line("alpha/aria", "You say, \"one\"").await;
    session.line("beta/bryn", "You say, \"two\"").await;
    let seqs: Vec<u64> = session.seen().iter().map(|event| event.seq).collect();
    let want: Vec<u64> = (1..=seqs.len() as u64).collect();
    assert_eq!(seqs, want);
    assert!(
        session
            .seen()
            .iter()
            .all(|event| event.world == "alpha/aria" || event.world == "beta/bryn")
    );
}

#[tokio::test]
async fn sources_of_a_session() {
    let mut session = Session::with_worlds(&[("alpha/aria", PW)]).await;
    session.client.send("alpha/aria", "hello").await.unwrap();
    let echo = session.line("alpha/aria", "You say, \"hello\"").await;
    let source = |kind: EventKind, text: Option<&str>| {
        session
            .seen()
            .iter()
            .find(|event| event.kind == kind && text.is_none_or(|text| event.text == text))
            .unwrap()
            .source
    };
    assert_eq!(
        source(EventKind::EVENT_KIND_CONNECTED, None),
        Source::SOURCE_SYSTEM
    );
    assert_eq!(
        source(EventKind::EVENT_KIND_LOGIN, None),
        Source::SOURCE_CREDENTIAL
    );
    assert_eq!(
        source(EventKind::EVENT_KIND_INPUT, Some("hello")),
        Source::SOURCE_PLAYER
    );
    assert_eq!(echo.source, Source::SOURCE_GAME);
}

#[tokio::test]
async fn password_is_replaced() {
    let session = Session::with_worlds(&[("alpha/aria", PW)]).await;
    let login: Vec<&str> = session
        .seen()
        .iter()
        .filter(|event| event.kind == EventKind::EVENT_KIND_LOGIN)
        .map(|event| event.text.as_str())
        .collect();
    assert_eq!(login, ["connect aria ****"]);
    no_password(&session, PW);
    assert!(
        session
            .server
            .stderr()
            .iter()
            .all(|line| !line.contains(PW))
    );
}

#[tokio::test]
async fn typed_password_is_masked() {
    let mut session = Session::with_worlds(&[("alpha/aria", PW)]).await;
    session
        .client
        .send("alpha/aria", "my pw is s3cret-pw")
        .await
        .unwrap();
    session
        .line("alpha/aria", "You say, \"my pw is ****\"")
        .await;
    assert!(session.seen().iter().any(|event| {
        event.kind == EventKind::EVENT_KIND_INPUT && event.text == "my pw is ****"
    }));
    no_password(&session, PW);
}

#[tokio::test]
async fn password_with_star_masks_whole_text() {
    let mut session = Session::with_worlds(&[("alpha/aria", "a*")]).await;
    let seq = session
        .client
        .send("alpha/aria", "aa*")
        .await
        .unwrap()
        .unwrap();
    let input = session.until("the input", |event| event.seq == seq).await;
    assert_eq!(input.text, "****");
    session.line("alpha/aria", "****").await;
    no_password(&session, "a*");
}

#[test]
fn password_of_stars_is_refused() {
    let dir = private_tempdir();
    let socket = dir.path().join("s.sock").display().to_string();
    let output = mu_with(
        &[
            "server",
            "--socket",
            &socket,
            "--fake-world",
            "alpha/aria=env:ALPHA_PW",
        ],
        &[("ALPHA_PW", Some("**"))],
    );
    let stderr = stderr(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr.contains("alpha/aria") && stderr.contains("ALPHA_PW"),
        "{stderr}"
    );
}

#[tokio::test]
async fn stream_from_the_middle() {
    let session = six_events().await;
    let mut stream = session.client.subscribe(4).await.unwrap();
    let (instance, last) = hello(stream.next().await.unwrap());
    assert_ne!(instance, "");
    assert_eq!(last, 6);
    for want in 4..=6 {
        assert_eq!(seq(stream.next().await.unwrap()), want);
    }
    session.client.send("alpha/aria", "next").await.unwrap();
    assert_eq!(
        seq(wait_until(BOUND, "event 7", stream.next()).await.unwrap()),
        7
    );
}

#[tokio::test]
async fn stream_from_zero() {
    let session = six_events().await;
    let mut stream = session.client.subscribe(0).await.unwrap();
    assert_eq!(hello(stream.next().await.unwrap()).1, 6);
    for want in 1..=6 {
        assert_eq!(seq(stream.next().await.unwrap()), want);
    }
}

#[tokio::test]
async fn stream_from_the_next_number() {
    let session = six_events().await;
    let mut stream = session.client.subscribe(7).await.unwrap();
    assert_eq!(hello(stream.next().await.unwrap()).1, 6);
    session.client.send("alpha/aria", "next").await.unwrap();
    assert_eq!(
        seq(wait_until(BOUND, "event 7", stream.next()).await.unwrap()),
        7
    );
}

#[tokio::test]
async fn stream_past_the_end() {
    let session = six_events().await;
    let mut stream = session.client.subscribe(8).await.unwrap();
    let (instance, last) = hello(stream.next().await.unwrap());
    assert_ne!(instance, "");
    assert_eq!(last, 6);
    assert_eq!(stream.next().await.unwrap_err().code, "out_of_range");
}
