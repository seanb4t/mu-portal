//! Helpers that the engine tests share. Each test runs on the paused clock.

#![allow(dead_code, reason = "each test binary uses a part of this module")]

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use mu_core::{
    Event, EventKind, FakeGame, FakeTransport, Handler, MemoryJournal, Portal, PortalError,
    ResolvedWorld, Secret, Seq, WorldId, WorldSetup, WorldState,
};

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
    let resolved = ResolvedWorld::with_default_login(id(text), "env:PW".parse().unwrap());
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
