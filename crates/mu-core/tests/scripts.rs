//! The scenarios of hooks and handlers, with the depth limit and the rate limit.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use mu_core::{
    Command, DEPTH_LIMIT, Envelope, Event, EventKind, Handler, Hook, HookAction, Origin,
    RATE_PER_SECOND, Source, WorldState,
};

use support::*;

mod support;

const PW: &str = "s3cret-pw";

struct GameGag(&'static str);

impl Hook for GameGag {
    fn on_game_line(&self, line: &str) -> HookAction {
        if line.contains(self.0) {
            HookAction::Gag
        } else {
            HookAction::Pass
        }
    }
}

struct PlayerHook {
    from: &'static str,
    action: HookAction,
}

impl Hook for PlayerHook {
    fn on_player_line(&self, line: &str) -> HookAction {
        if line == self.from {
            self.action.clone()
        } else {
            HookAction::Pass
        }
    }
}

/// A handler from a closure.
struct On<F>(F);

impl<F: Fn(&Event) -> Vec<Command> + Send + Sync + 'static> Handler for On<F> {
    fn on_event(&self, event: &Event) -> Vec<Command> {
        (self.0)(event)
    }
}

fn on(f: impl Fn(&Event) -> Vec<Command> + Send + Sync + 'static) -> Arc<dyn Handler> {
    Arc::new(On(f))
}

fn send(world: &str, text: &str) -> Command {
    Command::Send {
        world: id(world),
        text: text.to_owned(),
    }
}

fn emit(text: &str) -> Envelope {
    Envelope {
        command: Command::Emit {
            world: id("alpha/aria"),
            text: text.to_owned(),
        },
        origin: Origin::Script,
        depth: 0,
    }
}

fn count_lines(lines: &[String], text: &str) -> usize {
    lines.iter().filter(|line| *line == text).count()
}

#[tokio::test(start_paused = true)]
async fn hook_gags_game_line() {
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria.with_hook(Arc::new(GameGag("spam")))]).await;
    assert!(game.say("buy spam now"));
    assert!(game.say("hello"));
    wait_event(&portal, |event| {
        event.kind == EventKind::Line && event.text == "hello"
    })
    .await;
    assert!(
        events(&portal)
            .iter()
            .all(|event| !event.text.contains("spam"))
    );
}

#[tokio::test(start_paused = true)]
async fn hook_replaces_player_line() {
    let hook = PlayerHook {
        from: "hi",
        action: HookAction::Replace("say hi".to_owned()),
    };
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria.with_hook(Arc::new(hook))]).await;
    let seq = portal.send(&id("alpha/aria"), "hi").await.unwrap().unwrap();
    assert_eq!(game.wait_for_lines(2).await[1], "say hi");
    let event = wait_event(&portal, |event| event.seq == seq).await;
    assert_eq!(
        (event.kind, event.text.as_str()),
        (EventKind::Input, "say hi")
    );
}

#[tokio::test(start_paused = true)]
async fn hook_gags_player_line() {
    let hook = PlayerHook {
        from: "secret plan",
        action: HookAction::Gag,
    };
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria.with_hook(Arc::new(hook))]).await;
    assert_eq!(
        portal.send(&id("alpha/aria"), "secret plan").await,
        Ok(None)
    );
    settle().await;
    assert_eq!(game.lines().len(), 1);
    assert!(
        events(&portal)
            .iter()
            .all(|event| !event.text.contains("secret plan"))
    );
}

#[tokio::test(start_paused = true)]
async fn handler_answers_line() {
    let handler = on(|event| {
        if event.kind == EventKind::Line && event.text == "Hello?" {
            vec![send("alpha/aria", "wave")]
        } else {
            Vec::new()
        }
    });
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria.with_handler(handler)]).await;
    assert!(game.say("Hello?"));
    assert_eq!(game.wait_for_lines(2).await[1], "wave");
    let event = wait_event(&portal, |event| event.kind == EventKind::Input).await;
    assert_eq!(
        (event.text.as_str(), event.source),
        ("wave", Source::Script)
    );
}

#[tokio::test(start_paused = true)]
async fn handler_loop_stops_at_depth_limit() {
    let echo_custom = on(|event| {
        if event.kind == EventKind::Custom {
            vec![Command::Emit {
                world: event.world.clone(),
                text: event.text.clone(),
            }]
        } else {
            Vec::new()
        }
    });
    let (aria, _) = world("alpha/aria", PW);
    let portal = portal(vec![aria], vec![echo_custom]);
    portal.submit(emit("ping")).await.unwrap();
    let error = wait_event(&portal, |event| event.kind == EventKind::ScriptError).await;
    settle().await;
    assert_eq!(
        of_kind(&portal, EventKind::Custom).len(),
        1 + usize::from(DEPTH_LIMIT)
    );
    let errors = of_kind(&portal, EventKind::ScriptError);
    assert_eq!(errors.len(), 1);
    assert!(error.text.contains("depth limit"));
}

#[tokio::test(start_paused = true)]
async fn handler_on_every_event_stops() {
    let answer_all = on(|event| {
        vec![Command::Emit {
            world: event.world.clone(),
            text: "again".to_owned(),
        }]
    });
    let (aria, _) = world("alpha/aria", PW);
    let portal = portal(vec![aria], vec![answer_all]);
    portal.submit(emit("ping")).await.unwrap();
    let error = wait_event(&portal, |event| event.kind == EventKind::ScriptError).await;
    settle().await;
    assert_eq!(of_kind(&portal, EventKind::ScriptError).len(), 1);
    assert_eq!(portal.journal().last(), error.seq);
}

/// A world whose handler sends `again` for each echo of the game.
fn again_loop() -> (mu_core::WorldSetup, mu_core::FakeGame) {
    let handler = on(|event| {
        if event.kind == EventKind::Line && event.text.starts_with("You say") {
            vec![send("alpha/aria", "again")]
        } else {
            Vec::new()
        }
    });
    let (aria, game) = world("alpha/aria", PW);
    (aria.with_handler(handler), game)
}

#[tokio::test(start_paused = true)]
async fn handler_loop_through_game_is_rate_limited() {
    let (aria, game) = again_loop();
    let portal = started(vec![aria]).await;
    let start = tokio::time::Instant::now();
    portal.send(&id("alpha/aria"), "start").await.unwrap();
    wait_event(&portal, |event| event.kind == EventKind::ScriptError).await;
    assert!(start.elapsed() < Duration::from_secs(1));
    settle().await;
    let rate = usize::try_from(RATE_PER_SECOND).unwrap();
    assert_eq!(count_lines(&game.lines(), "again"), rate);
    assert_eq!(of_kind(&portal, EventKind::ScriptError).len(), 1);
}

#[tokio::test(start_paused = true)]
async fn rate_limit_refills() {
    let (aria, game) = again_loop();
    let portal = started(vec![aria]).await;
    portal.send(&id("alpha/aria"), "start").await.unwrap();
    wait_event(&portal, |event| event.kind == EventKind::ScriptError).await;
    let rate = usize::try_from(RATE_PER_SECOND).unwrap();
    let mut right_after = emit("dropped");
    right_after.command = send("alpha/aria", "again");
    assert_eq!(portal.submit(right_after).await, Ok(None));
    assert_eq!(count_lines(&game.lines(), "again"), rate);

    tokio::time::advance(Duration::from_millis(50)).await;
    portal.send(&id("alpha/aria"), "start").await.unwrap();
    // The login, `start`, the burst, the second `start` and one `again` from the new budget.
    game.wait_for_lines(rate + 4).await;
    settle().await;
    assert_eq!(count_lines(&game.lines(), "again"), rate + 1);
}

#[tokio::test(start_paused = true)]
async fn two_drops_give_one_script_error() {
    let rate = usize::try_from(RATE_PER_SECOND).unwrap();
    let burst = on(move |event| {
        if event.kind == EventKind::Line && event.text == "go" {
            (0..rate + 2).map(|_| send("alpha/aria", "x")).collect()
        } else {
            Vec::new()
        }
    });
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria.with_handler(burst)]).await;
    assert!(game.say("go"));
    wait_event(&portal, |event| event.kind == EventKind::ScriptError).await;
    settle().await;
    assert_eq!(count_lines(&game.lines(), "x"), rate);
    assert_eq!(of_kind(&portal, EventKind::ScriptError).len(), 1);
}

#[tokio::test(start_paused = true)]
async fn world_handler_skips_script_error() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let counter = {
        let seen = Arc::clone(&seen);
        on(move |event| {
            seen.lock().unwrap().push(event.kind);
            Vec::new()
        })
    };
    let (aria, _) = world("alpha/aria", PW);
    let portal = portal(vec![aria.with_handler(counter)], Vec::new());
    let mut too_deep = emit("deep");
    too_deep.depth = DEPTH_LIMIT + 1;
    portal.submit(too_deep).await.unwrap();
    portal.submit(emit("shallow")).await.unwrap();
    assert_eq!(of_kind(&portal, EventKind::ScriptError).len(), 1);
    assert_eq!(*seen.lock().unwrap(), [EventKind::Custom]);
}

#[tokio::test(start_paused = true)]
async fn handler_connect_has_source_system() {
    let reconnect = on(|event| {
        if event.kind == EventKind::Disconnected {
            vec![Command::Connect {
                world: event.world.clone(),
            }]
        } else {
            Vec::new()
        }
    });
    let (aria, _) = world("alpha/aria", PW);
    let portal = started(vec![aria.with_handler(reconnect)]).await;
    portal.send(&id("alpha/aria"), "@drop").await.unwrap();
    let gone = wait_event(&portal, |event| event.kind == EventKind::Disconnected).await;
    let back = wait_event(&portal, |event| {
        event.seq > gone.seq && event.kind == EventKind::Connected
    })
    .await;
    assert_eq!(back.source, Source::System);
    wait_state(&portal, &id("alpha/aria"), WorldState::Connected).await;
}
