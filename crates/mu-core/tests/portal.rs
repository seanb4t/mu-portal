//! The scenarios of the world-sessions and event-journal capabilities, run on the engine
//! with the fake game.

use std::sync::Arc;

use mu_core::{
    ConnectGate, EventKind, FakeTransport, Journal, MemoryJournal, NewEvent, Portal, PortalError,
    Source, WorldId, WorldState,
};

use support::*;

mod support;

const PW: &str = "s3cret-pw";

#[tokio::test(start_paused = true)]
async fn two_characters_on_one_game() {
    let (aria, _) = world("alpha/aria", PW);
    let (ash, _) = world("alpha/ash", PW);
    let portal = started(vec![aria, ash]).await;
    portal.send(&id("alpha/aria"), "one").await.unwrap();
    wait_event(&portal, |event| {
        event.world == id("alpha/aria") && event.text == "You say, \"one\""
    })
    .await;
    assert!(
        events(&portal)
            .iter()
            .filter(|event| event.world == id("alpha/ash"))
            .all(|event| !event.text.contains("one"))
    );
}

#[tokio::test(start_paused = true)]
async fn send_to_unknown_world() {
    let (aria, _) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    let last = portal.journal().last();
    assert_eq!(
        portal.send(&id("nowhere/nobody"), "hello").await,
        Err(PortalError::UnknownWorld(id("nowhere/nobody")))
    );
    assert_eq!(portal.journal().last(), last);
}

#[tokio::test(start_paused = true)]
async fn send_to_disconnected_world() {
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    portal.disconnect(&id("alpha/aria")).await.unwrap();
    wait_state(&portal, &id("alpha/aria"), WorldState::Disconnected).await;
    let last = portal.journal().last();
    let lines = game.lines().len();
    assert_eq!(
        portal.send(&id("alpha/aria"), "hello").await,
        Err(PortalError::NotConnected(id("alpha/aria")))
    );
    assert_eq!(portal.journal().last(), last);
    assert_eq!(game.lines().len(), lines);
}

#[tokio::test(start_paused = true)]
async fn send_to_connecting_world() {
    let (transport, _gate) = FakeTransport::new("aria", PW).held();
    let (aria, _) = world_on("alpha/aria", PW, transport);
    let portal = portal(vec![aria], Vec::new());
    portal.connect_all().await;
    wait_state(&portal, &id("alpha/aria"), WorldState::Connecting).await;
    assert_eq!(
        portal.send(&id("alpha/aria"), "hello").await,
        Err(PortalError::NotConnected(id("alpha/aria")))
    );
    assert_eq!(portal.journal().last().0, 0);
}

#[tokio::test(start_paused = true)]
async fn empty_send_reaches_game() {
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    let seq = portal.send(&id("alpha/aria"), "").await.unwrap().unwrap();
    assert_eq!(game.wait_for_lines(2).await[1], "");
    let event = wait_event(&portal, |event| event.seq == seq).await;
    assert_eq!(
        (event.kind, event.source, event.text.as_str()),
        (EventKind::Input, Source::Player, "")
    );
}

async fn refused_line_break(text: &str) {
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    let last = portal.journal().last();
    assert_eq!(
        portal.send(&id("alpha/aria"), text).await,
        Err(PortalError::LineBreak)
    );
    idle(&portal).await;
    assert_eq!(portal.journal().last(), last);
    assert_eq!(game.lines(), [format!("connect aria {PW}")]);
}

#[tokio::test(start_paused = true)]
async fn send_with_lf_is_refused() {
    refused_line_break("one\ntwo").await;
}

#[tokio::test(start_paused = true)]
async fn send_with_cr_is_refused() {
    refused_line_break("one\rtwo").await;
}

#[tokio::test(start_paused = true)]
async fn one_world_drops() {
    let (aria, _) = world("alpha/aria", PW);
    let (bryn, _) = world("beta/bryn", PW);
    let portal = started(vec![aria, bryn]).await;
    portal.disconnect(&id("alpha/aria")).await.unwrap();
    wait_event(&portal, |event| {
        event.world == id("alpha/aria") && event.kind == EventKind::Disconnected
    })
    .await;
    portal.send(&id("beta/bryn"), "still here").await.unwrap();
    wait_event(&portal, |event| {
        event.world == id("beta/bryn") && event.text == "You say, \"still here\""
    })
    .await;
    assert_eq!(
        portal.worlds(),
        [
            (id("alpha/aria"), WorldState::Disconnected),
            (id("beta/bryn"), WorldState::Connected)
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn world_connects_again() {
    let (aria, _) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    portal.disconnect(&id("alpha/aria")).await.unwrap();
    let gone = wait_event(&portal, |event| event.kind == EventKind::Disconnected).await;
    portal.connect(&id("alpha/aria")).await.unwrap();
    wait_event(&portal, |event| {
        event.seq > gone.seq && event.kind == EventKind::Connected
    })
    .await;
    wait_event(&portal, |event| {
        event.seq > gone.seq && event.kind == EventKind::Login
    })
    .await;
    wait_state(&portal, &id("alpha/aria"), WorldState::Connected).await;
}

#[tokio::test(start_paused = true)]
async fn connect_from_api_has_source_system() {
    let (aria, _) = world("alpha/aria", PW);
    let portal = portal(vec![aria], Vec::new());
    portal.connect(&id("alpha/aria")).await.unwrap();
    let event = wait_event(&portal, |event| event.kind == EventKind::Connected).await;
    assert_eq!(event.source, Source::System);
}

#[tokio::test(start_paused = true)]
async fn sources_of_a_session() {
    let (aria, _) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    portal.send(&id("alpha/aria"), "hello").await.unwrap();
    wait_event(&portal, |event| event.text == "You say, \"hello\"").await;
    let source = |kind, text: Option<&str>| {
        events(&portal)
            .into_iter()
            .find(|event| event.kind == kind && text.is_none_or(|text| event.text == text))
            .unwrap()
            .source
    };
    assert_eq!(source(EventKind::Connected, None), Source::System);
    assert_eq!(source(EventKind::Login, None), Source::Credential);
    assert_eq!(source(EventKind::Input, Some("hello")), Source::Player);
    assert_eq!(
        source(EventKind::Line, Some("You say, \"hello\"")),
        Source::Game
    );
}

#[tokio::test(start_paused = true)]
async fn login_event_is_redacted() {
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    assert_eq!(game.lines(), [format!("connect aria {PW}")]);
    let login = of_kind(&portal, EventKind::Login);
    assert_eq!(login.len(), 1);
    assert_eq!(login[0].text, "connect aria ****");
    assert!(events(&portal).iter().all(|event| !event.text.contains(PW)));
}

#[tokio::test(start_paused = true)]
async fn typed_password_is_masked() {
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    portal
        .send(&id("alpha/aria"), "my pw is s3cret-pw")
        .await
        .unwrap();
    wait_event(&portal, |event| event.text == "You say, \"my pw is ****\"").await;
    assert_eq!(game.lines()[1], "my pw is s3cret-pw");
    assert!(
        of_kind(&portal, EventKind::Input)
            .iter()
            .any(|event| event.text == "my pw is ****")
    );
    assert!(events(&portal).iter().all(|event| !event.text.contains(PW)));
}

#[tokio::test(start_paused = true)]
async fn password_with_star_masks_whole_text() {
    let (aria, _) = world("alpha/aria", "a*");
    let portal = started(vec![aria]).await;
    let seq = portal
        .send(&id("alpha/aria"), "aa*")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        wait_event(&portal, |event| event.seq == seq).await.text,
        "****"
    );
    wait_event(&portal, |event| {
        event.kind == EventKind::Line && event.text == "****"
    })
    .await;
    assert!(
        events(&portal)
            .iter()
            .all(|event| !event.text.contains("a*"))
    );
}

#[tokio::test(start_paused = true)]
async fn the_game_drops_the_world() {
    let (aria, _) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    portal.send(&id("alpha/aria"), "@drop").await.unwrap();
    wait_state(&portal, &id("alpha/aria"), WorldState::Disconnected).await;
    let gone = of_kind(&portal, EventKind::Disconnected);
    assert_eq!((gone.len(), gone[0].source), (1, Source::System));
}

#[tokio::test(start_paused = true)]
async fn failed_write_records_disconnect() {
    let (portal, aria) = broken_world(bare_world(BrokenTransport::refusing_writes())).await;
    assert_eq!(
        portal.send(&aria, "hello").await,
        Err(PortalError::NotConnected(aria.clone()))
    );
    assert_eq!(of_kind(&portal, EventKind::Disconnected).len(), 1);
    assert_eq!(of_kind(&portal, EventKind::Input), []);
    wait_state(&portal, &aria, WorldState::Disconnected).await;
}

#[tokio::test(start_paused = true)]
async fn stop_disconnects_every_world_and_refuses_connect() {
    let (aria, _) = world("alpha/aria", PW);
    let (bryn, _) = world("beta/bryn", PW);
    let portal = started(vec![aria, bryn]).await;
    portal.stop().await;
    assert_eq!(
        portal.worlds(),
        [
            (id("alpha/aria"), WorldState::Disconnected),
            (id("beta/bryn"), WorldState::Disconnected)
        ]
    );
    let gone: Vec<(WorldId, Source)> = of_kind(&portal, EventKind::Disconnected)
        .into_iter()
        .map(|event| (event.world, event.source))
        .collect();
    assert_eq!(
        gone,
        [
            (id("alpha/aria"), Source::System),
            (id("beta/bryn"), Source::System)
        ]
    );
    let last = portal.journal().last();
    let refused = portal.connect(&id("alpha/aria")).await;
    assert!(matches!(refused, Err(PortalError::Stopped)), "{refused:?}");
    idle(&portal).await;
    assert_eq!(
        portal.journal().last(),
        last,
        "a connect after the stop records no event"
    );
    assert_eq!(
        portal.worlds()[0],
        (id("alpha/aria"), WorldState::Disconnected)
    );
}

/// Returns after the session of `world` handled each request sent before it. While the connect
/// of `world` is held, a send gets `NotConnected`.
async fn queue_barrier(portal: &Portal, world: &WorldId) {
    assert_eq!(
        portal.send(world, "barrier").await,
        Err(PortalError::NotConnected(world.clone()))
    );
}

/// A world on a held connect, with a portal that connected it.
async fn world_in_held_connect() -> (Portal, ConnectGate) {
    let (transport, gate) = FakeTransport::new("aria", PW).held();
    let (aria, _) = world_on("alpha/aria", PW, transport);
    let portal = portal(vec![aria], Vec::new());
    portal.connect_all().await;
    (portal, gate)
}

fn has_text(portal: &Portal, text: &str) -> bool {
    events(portal).iter().any(|event| event.text == text)
}

#[tokio::test(start_paused = true)]
async fn wait_covers_game_echo() {
    let (aria, _) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    as_task(async move {
        portal.send(&id("alpha/aria"), "one").await.unwrap();
        idle(&portal).await;
        assert!(has_text(&portal, "You say, \"one\""));
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn wait_adds_nothing() {
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    let last = portal.journal().last();
    let lines = game.lines();
    idle(&portal).await;
    assert_eq!(portal.journal().last(), last);
    assert_eq!(game.lines(), lines);
}

#[tokio::test(start_paused = true)]
async fn wait_on_world_that_is_down() {
    let (aria, _) = world("alpha/aria", PW);
    let portal = portal(vec![aria], Vec::new());
    idle(&portal).await;
    assert_eq!(events(&portal), []);
}

#[tokio::test(start_paused = true)]
async fn wait_outlasts_connect() {
    let (portal, gate) = world_in_held_connect().await;
    let aria = id("alpha/aria");
    let (waited, ()) = tokio::join!(try_idle(&portal), async {
        queue_barrier(&portal, &aria).await;
        gate.open();
    });
    assert_eq!(waited, Ok(()));
    assert!(has_text(&portal, "Welcome, aria."));
}

#[tokio::test(start_paused = true)]
async fn wait_survives_drop() {
    let (aria, _) = world("alpha/aria", PW);
    let portal = started(vec![aria]).await;
    let aria = id("alpha/aria");
    let (sent, waited) = tokio::join!(portal.send(&aria, "@drop"), try_idle(&portal));
    assert!(sent.is_ok());
    assert_eq!(waited, Ok(()));
    let gone = of_kind(&portal, EventKind::Disconnected);
    assert_eq!(gone.len(), 1);
    assert_eq!(gone[0].world, aria);
}

#[tokio::test(start_paused = true)]
async fn wait_survives_failed_connect() {
    let (portal, gate) = world_in_held_connect().await;
    let aria = id("alpha/aria");
    let (waited, ()) = tokio::join!(try_idle(&portal), async {
        queue_barrier(&portal, &aria).await;
        drop(gate);
    });
    assert_eq!(waited, Ok(()));
    let gone = of_kind(&portal, EventKind::Disconnected);
    assert_eq!(gone.len(), 1);
    assert_eq!(gone[0].world, aria);
    assert_eq!(gone[0].text, "the connect failed: the connect gate closed");
}

#[tokio::test(start_paused = true)]
async fn wait_survives_failed_mark_write() {
    let (portal, aria) = broken_world(bare_world(BrokenTransport::refusing_writes())).await;
    assert_eq!(try_idle(&portal).await, Ok(()));
    let gone = of_kind(&portal, EventKind::Disconnected);
    assert_eq!(gone.len(), 1);
    assert_eq!(gone[0].world, aria);
}

#[tokio::test(start_paused = true)]
async fn two_waits_on_one_world() {
    let (aria, _) = world("alpha/aria", PW);
    let connected = started(vec![aria]).await;
    let (first, second) = tokio::join!(try_idle(&connected), try_idle(&connected));
    assert_eq!((first, second), (Ok(()), Ok(())));

    let (held, gate) = world_in_held_connect().await;
    let aria = id("alpha/aria");
    let (first, second, ()) = tokio::join!(try_idle(&held), try_idle(&held), async {
        queue_barrier(&held, &aria).await;
        gate.open();
    });
    assert_eq!((first, second), (Ok(()), Ok(())));
    assert!(has_text(&held, "Welcome, aria."));
}

#[tokio::test(start_paused = true)]
async fn wait_on_journal_with_events_at_start() {
    let journal = Arc::new(MemoryJournal::new(mu_core::journal::DEFAULT_CAPACITY).unwrap());
    journal.append(NewEvent {
        world: id("alpha/aria"),
        kind: EventKind::Custom,
        source: Source::Script,
        text: "early".to_owned(),
        depth: 0,
    });
    let (aria, _) = world("alpha/aria", PW);
    let portal = Portal::start(journal, vec![aria], vec![on(|_| Vec::new())]);
    assert_eq!(try_idle(&portal).await, Ok(()));
}
