//! The scenarios of the world-sessions and event-journal capabilities, run on the engine
//! with the fake game.

use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use mu_core::{
    ByteStream, EventKind, FakeTransport, PortalError, ResolvedWorld, Source, Transport, WorldId,
    WorldSetup, WorldState,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

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
    settle().await;
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

/// A stream that never gives a byte, and whose every write fails.
struct BrokenWrite;

impl AsyncRead for BrokenWrite {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        _: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Poll::Pending
    }
}

impl AsyncWrite for BrokenWrite {
    fn poll_write(self: Pin<&mut Self>, _: &mut Context<'_>, _: &[u8]) -> Poll<io::Result<usize>> {
        Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

struct BrokenTransport;

impl Transport for BrokenTransport {
    fn connect(&self, _: &WorldId) -> BoxFuture<'static, io::Result<Box<dyn ByteStream>>> {
        Box::pin(async { Ok(Box::new(BrokenWrite) as Box<dyn ByteStream>) })
    }
}

#[tokio::test(start_paused = true)]
async fn failed_write_records_disconnect() {
    let aria = id("alpha/aria");
    let world = ResolvedWorld {
        id: aria.clone(),
        login: None,
    };
    let setup = WorldSetup::new(world, None, Arc::new(BrokenTransport));
    let portal = portal(vec![setup], Vec::new());
    portal.connect_all().await;
    wait_state(&portal, &aria, WorldState::Connected).await;
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
    settle().await;
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
