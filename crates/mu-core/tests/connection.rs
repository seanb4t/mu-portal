//! The scenarios of the world-connection capability that run on the engine with a paused clock.

use std::io;
use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use mu_core::{ByteStream, EventKind, Reconnect, ResolvedWorld, Transport, WorldId, WorldSetup};
use tokio::time::Instant;

use support::*;

mod support;

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
