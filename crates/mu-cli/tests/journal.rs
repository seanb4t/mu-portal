//! The scenarios of the event-journal capability, run on the built binary.

use mu_proto::__buffa::oneof::subscribe_response::Message;
use mu_proto::{EventKind, Source, SubscribeResponse};

use support::process::{BOUND, PW, ServerBuilder, Session, TWO_WORLDS, wait_until};
use support::{rule, stderr};

mod support;

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
    let session = Session::with_worlds(TWO_WORLDS).await;
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
    let mut session = Session::with_worlds(TWO_WORLDS).await;
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
    let mut session = Session::with_worlds(&["alpha/aria"]).await;
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
    let session = Session::with_worlds(&["alpha/aria"]).await;
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
    let mut session = Session::with_worlds(&["alpha/aria"]).await;
    session
        .client
        .send("alpha/aria", &format!("my pw is {PW}"))
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
    let mut session =
        Session::start(ServerBuilder::new().fake_world("alpha/aria", "ARIA_PW", "a*")).await;
    session.welcome("alpha/aria").await;
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

#[tokio::test]
async fn password_of_stars_is_refused() {
    let output = ServerBuilder::new()
        .fake_world("alpha/aria", "ALPHA_PW", "**")
        .output()
        .await;
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

// The ids and the streams of the events.

fn ids_of(events: &[mu_proto::Event]) -> std::collections::BTreeSet<String> {
    events.iter().map(|event| event.id.clone()).collect()
}

#[tokio::test]
async fn ids_in_one_stream() {
    let mut session = Session::with_worlds(&["alpha/aria"]).await;
    for text in ["one", "two", "three"] {
        session.client.send("alpha/aria", text).await.unwrap();
    }
    session.line("alpha/aria", "You say, \"three\"").await;
    let seen = session.seen();
    assert!(seen.iter().all(|event| event.id.len() == 26), "{seen:?}");
    assert_eq!(ids_of(seen).len(), seen.len(), "{seen:?}");
}

#[tokio::test]
async fn ids_after_a_restart() {
    let dir = support::ConfigDir::empty();
    let first = on_config(&dir).await;
    let before = ids_of(first.seen());
    drop(first);
    let mut second = on_config(&dir).await;
    second.client.send("alpha/aria", "hi").await.unwrap();
    second.line("alpha/aria", "You say, \"hi\"").await;
    let after = ids_of(second.seen());
    assert!(
        !before.is_empty() && before.is_disjoint(&after),
        "{before:?} {after:?}"
    );
}

#[tokio::test]
async fn no_event_without_a_stream() {
    let mut session = Session::with_worlds(&["alpha/aria"]).await;
    session.client.send("alpha/aria", "hi").await.unwrap();
    session.line("alpha/aria", "You say, \"hi\"").await;
    session.client.send("alpha/aria", "@drop").await.unwrap();
    session
        .kind("alpha/aria", EventKind::EVENT_KIND_DISCONNECTED, 0)
        .await;
    let seen = session.seen();
    assert!(
        seen.iter().all(|event| !event.streams.is_empty()),
        "{seen:?}"
    );
}

// The line rules on the events of a server.

/// A server with `global.toml` of `global` and the fake world `alpha/aria`, after its greeting.
/// The directory lives as long as the session.
async fn with_global(global: &str) -> (Session, support::ConfigDir) {
    let dir = support::ConfigDir::empty().file("global.toml", global);
    (on_config(&dir).await, dir)
}

/// A server on the configuration `dir` with the fake world `alpha/aria`, after its greeting.
async fn on_config(dir: &support::ConfigDir) -> Session {
    let builder = ServerBuilder::new()
        .arg("--config")
        .arg(dir.path())
        .fake_worlds(&["alpha/aria"]);
    let mut session = Session::start(builder).await;
    session.welcome("alpha/aria").await;
    session
}

fn streams_of(event: &mu_proto::Event) -> Vec<&str> {
    event.streams.iter().map(String::as_str).collect()
}

fn script_errors(session: &Session) -> Vec<&mu_proto::Event> {
    session
        .seen()
        .iter()
        .filter(|event| event.kind == EventKind::EVENT_KIND_SCRIPT_ERROR)
        .collect()
}

#[tokio::test]
async fn the_line_that_the_player_sends() {
    let (mut session, _dir) = with_global(&rule(
        "chan",
        "pattern = '\\[Public\\]'\nstream = \"public\"",
    ))
    .await;
    session
        .client
        .send("alpha/aria", "[Public] hi")
        .await
        .unwrap();
    let line = session.line("alpha/aria", "You say, \"[Public] hi\"").await;
    let input = session
        .until("the player event", |event| {
            event.kind == EventKind::EVENT_KIND_INPUT && event.text == "[Public] hi"
        })
        .await;
    assert_eq!(
        (streams_of(&input), streams_of(&line)),
        (vec!["main"], vec!["public"])
    );
}

#[tokio::test]
async fn two_bad_names() {
    let (mut session, _dir) = with_global(&rule(
        "chan",
        "pattern = '^You say, \"\\[([^\\]]+)\\] '\nstream = \"chan-$1\"",
    ))
    .await;
    session
        .client
        .send("alpha/aria", "[Role Play] one")
        .await
        .unwrap();
    session
        .client
        .send("alpha/aria", "[Role Play] two")
        .await
        .unwrap();
    let one = session
        .line("alpha/aria", "You say, \"[Role Play] one\"")
        .await;
    let two = session
        .line("alpha/aria", "You say, \"[Role Play] two\"")
        .await;
    let errors = script_errors(&session);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].world == "alpha/aria"
            && errors[0].text.contains("chan")
            && errors[0].text.contains("chan-role play")
            && errors[0].seq < one.seq,
        "{errors:?}"
    );
    assert_eq!(
        (streams_of(&one), streams_of(&two)),
        (vec!["main"], vec!["main"])
    );
}

#[tokio::test]
async fn a_drop_during_a_capture() {
    let (mut session, _dir) = with_global(&rule(
        "who",
        "pattern = '^You say, \"WHO'\nuntil = '^End'\nstream = \"who\"",
    ))
    .await;
    session.client.send("alpha/aria", "WHO").await.unwrap();
    let who = session.line("alpha/aria", "You say, \"WHO\"").await;
    session.client.send("alpha/aria", "@drop").await.unwrap();
    let gone = session
        .kind("alpha/aria", EventKind::EVENT_KIND_DISCONNECTED, who.seq)
        .await;
    session.client.connect("alpha/aria").await.unwrap();
    session
        .until("the greeting after the connect", |event| {
            event.seq > gone.seq && event.text == support::process::greeting("alpha/aria")
        })
        .await;
    session.client.send("alpha/aria", "hi").await.unwrap();
    let hi = session.line("alpha/aria", "You say, \"hi\"").await;
    assert_eq!(
        (streams_of(&who), streams_of(&hi)),
        (vec!["who"], vec!["main"])
    );
}

#[tokio::test]
async fn a_pattern_that_backtracks_too_much() {
    let global = format!(
        "{}{}",
        rule(
            "slow",
            "pattern = '^You say, \"(a|aa)+\\1b'\nstream = \"slow\""
        ),
        rule("all", "pattern = '^You say, \"a'\nstream = \"rest\"")
    );
    let (mut session, _dir) = with_global(&global).await;
    let text = "a".repeat(40);
    session.client.send("alpha/aria", &text).await.unwrap();
    let line = session
        .line("alpha/aria", &format!("You say, \"{text}\""))
        .await;
    let errors = script_errors(&session);
    assert_eq!(streams_of(&line), ["rest"]);
    assert!(
        errors.len() == 1 && errors[0].text.contains("slow"),
        "{errors:?}"
    );
}

#[tokio::test]
async fn a_fake_world_with_a_rule() {
    let (mut session, _dir) = with_global(&rule(
        "chan",
        "pattern = '^You say, \"\\['\nstream = \"chat\"",
    ))
    .await;
    session
        .client
        .send("alpha/aria", "[Public] hi")
        .await
        .unwrap();
    let line = session.line("alpha/aria", "You say, \"[Public] hi\"").await;
    assert_eq!(streams_of(&line), ["chat"]);
}

#[tokio::test]
async fn a_fake_world_with_no_rules() {
    let (mut session, _dir) = with_global("").await;
    session
        .client
        .send("alpha/aria", "[Public] hi")
        .await
        .unwrap();
    let line = session.line("alpha/aria", "You say, \"[Public] hi\"").await;
    assert_eq!(streams_of(&line), ["main"]);
}

/// Reads `stream` from its start to the game line `text`, and gives every event it read.
async fn read_to(stream: &mut mu_client::EventStream, text: &str) -> Vec<mu_proto::Event> {
    let mut read = Vec::new();
    wait_until(BOUND, text, async {
        loop {
            let message = stream.next().await.unwrap();
            if let Some(Message::Event(event)) = message.and_then(|message| message.message) {
                let done = event.kind == EventKind::EVENT_KIND_LINE && event.text == text;
                read.push(*event);
                if done {
                    return;
                }
            }
        }
    })
    .await;
    read
}

#[tokio::test]
async fn a_live_event_and_a_replay() {
    let (mut session, _dir) = with_global(&rule(
        "chan",
        "pattern = '^You say, \"\\[Public\\]'\nstream = [\"public\", \"main\"]",
    ))
    .await;
    session
        .client
        .send("alpha/aria", "[Public] hi")
        .await
        .unwrap();
    let said = "You say, \"[Public] hi\"";
    let live = session.line("alpha/aria", said).await;
    let live_welcome = session.welcome("alpha/aria").await;
    let other = session.server.client();
    let mut stream = other.subscribe(1).await.unwrap();
    let replay = read_to(&mut stream, said).await;
    let find = |text: &str| {
        replay
            .iter()
            .find(|event| event.kind == EventKind::EVENT_KIND_LINE && event.text == text)
            .unwrap()
            .clone()
    };
    let replayed = find(said);
    let replayed_welcome = find(&support::process::greeting("alpha/aria"));
    assert_eq!(
        (
            (streams_of(&live), streams_of(&replayed)),
            live.id == replayed.id,
            (streams_of(&live_welcome), streams_of(&replayed_welcome)),
        ),
        (
            (vec!["public", "main"], vec!["public", "main"]),
            true,
            (vec!["main"], vec!["main"]),
        )
    );
}
