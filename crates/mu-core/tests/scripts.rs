//! The scenarios of hooks and handlers, with the depth limit and the rate limit.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use mu_core::{
    Command, DEPTH_LIMIT, Envelope, Event, EventKind, Framer, Handler, Hook, HookAction, Inbound,
    LineFramer, MemoryJournal, Origin, Portal, Source, WorldState, WorldStatus,
};

use support::*;

mod support;

const PW: &str = "s3cret-pw";

/// The rate limit of the spec: 20 commands from handlers in one second.
const RATE: usize = 20;

/// The depth limit of the spec.
const DEPTH: usize = 8;

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
    idle(&portal).await;
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

/// A portal whose server handler emits each custom event again, so that a loop runs to the
/// depth limit.
fn echo_custom_loop() -> Portal {
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
    portal(vec![aria], vec![echo_custom])
}

#[tokio::test(start_paused = true)]
async fn handler_loop_stops_at_depth_limit() {
    let portal = echo_custom_loop();
    portal.submit(emit("ping")).await.unwrap();
    let error = wait_event(&portal, |event| event.kind == EventKind::ScriptError).await;
    idle(&portal).await;
    assert_eq!(of_kind(&portal, EventKind::Custom).len(), 1 + DEPTH);
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
    idle(&portal).await;
    assert_eq!(of_kind(&portal, EventKind::ScriptError).len(), 1);
    assert_eq!(portal.journal().last(), error.seq);
}

/// A world whose handler sends `again` for each echo of the game.
fn again_loop() -> (mu_core::WorldSetup, mu_fake_server::FakeGame) {
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
    idle(&portal).await;
    assert_eq!(count_lines(&game.lines(), "again"), RATE);
    assert_eq!(of_kind(&portal, EventKind::ScriptError).len(), 1);
}

#[tokio::test(start_paused = true)]
async fn rate_limit_refills() {
    let (aria, game) = again_loop();
    let portal = started(vec![aria]).await;
    portal.send(&id("alpha/aria"), "start").await.unwrap();
    wait_event(&portal, |event| event.kind == EventKind::ScriptError).await;
    let mut right_after = emit("dropped");
    right_after.command = send("alpha/aria", "again");
    assert_eq!(portal.submit(right_after).await, Ok(None));
    assert_eq!(count_lines(&game.lines(), "again"), RATE);

    tokio::time::advance(Duration::from_millis(50)).await;
    portal.send(&id("alpha/aria"), "start").await.unwrap();
    // The login, `start`, the burst, the second `start` and one `again` from the new budget.
    game.wait_for_lines(RATE + 4).await;
    idle(&portal).await;
    assert_eq!(count_lines(&game.lines(), "again"), RATE + 1);
}

#[tokio::test(start_paused = true)]
async fn two_drops_give_one_script_error() {
    let burst = on(move |event| {
        if event.kind == EventKind::Line && event.text == "go" {
            (0..22).map(|_| send("alpha/aria", "x")).collect()
        } else {
            Vec::new()
        }
    });
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria.with_handler(burst)]).await;
    assert!(game.say("go"));
    wait_event(&portal, |event| event.kind == EventKind::ScriptError).await;
    idle(&portal).await;
    assert_eq!(count_lines(&game.lines(), "x"), RATE);
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

async fn line_break_from_hook(text: &str) {
    let hook = PlayerHook {
        from: "hi",
        action: HookAction::Replace(text.to_owned()),
    };
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria.with_hook(Arc::new(hook))]).await;
    assert_eq!(portal.send(&id("alpha/aria"), "hi").await, Ok(None));
    idle(&portal).await;
    assert_eq!(game.lines().len(), 1, "only the login reached the game");
    assert_eq!(of_kind(&portal, EventKind::ScriptError).len(), 1);
    assert_eq!(of_kind(&portal, EventKind::Input), []);
}

#[tokio::test(start_paused = true)]
async fn hook_line_break_is_dropped() {
    line_break_from_hook("say hi\nsay bye").await;
    line_break_from_hook("say hi\rsay bye").await;
}

/// A line framer that sends each line in capitals.
struct Shout(LineFramer);

impl Framer for Shout {
    fn feed(&mut self, bytes: &[u8]) -> Vec<Inbound> {
        self.0.feed(bytes)
    }

    fn encode(&self, line: &str) -> Vec<u8> {
        self.0.encode(&line.to_uppercase())
    }
}

#[tokio::test(start_paused = true)]
async fn world_uses_its_framer() {
    let (aria, game) = world("alpha/aria", PW);
    let made = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&made);
    let aria = aria.with_framer(Arc::new(move || {
        count.fetch_add(1, Ordering::Relaxed);
        Box::new(Shout(LineFramer::new()))
    }));
    let portal = portal(vec![aria], Vec::new());
    portal.connect_at_start().await;
    let login = format!("CONNECT ARIA {}", PW.to_uppercase());
    assert_eq!(game.wait_for_lines(1).await[0], login);
    // A new connection gets a new framer from the same factory.
    portal.disconnect(&id("alpha/aria")).await.unwrap();
    wait_state(&portal, &id("alpha/aria"), WorldState::Disconnected).await;
    portal.connect(&id("alpha/aria")).await.unwrap();
    assert_eq!(game.wait_for_lines(2).await, [login.clone(), login]);
    assert_eq!(made.load(Ordering::Relaxed), 2);
}

/// A server handler that records each event it sees.
fn recorder() -> (Arc<dyn Handler>, Arc<Mutex<Vec<Event>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let handler = {
        let seen = Arc::clone(&seen);
        on(move |event| {
            seen.lock().unwrap().push(event.clone());
            Vec::new()
        })
    };
    (handler, seen)
}

fn seqs(events: &[Event]) -> Vec<u64> {
    events.iter().map(|event| event.seq.0).collect()
}

#[tokio::test(start_paused = true)]
async fn server_handler_sees_every_world() {
    let (handler, seen) = recorder();
    let (aria, _) = world("alpha/aria", PW);
    let (bryn, _) = world("beta/bryn", PW);
    let portal = portal(vec![aria, bryn], vec![handler]);
    portal.connect_at_start().await;
    for greeting in ["Welcome, aria.", "Welcome, bryn."] {
        wait_event(&portal, |event| event.text == greeting).await;
    }
    idle(&portal).await;
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seqs(&seen), seqs(&events(&portal)));
    for world in ["alpha/aria", "beta/bryn"] {
        assert!(seen.iter().any(|event| event.world == id(world)), "{world}");
    }
}

#[tokio::test(start_paused = true)]
async fn server_handler_resumes_after_lag() {
    // A world handler answers one event with ten, all before the server handler runs, so
    // the server handler falls behind a live channel of four events.
    let burst = on(|event| {
        if event.kind == EventKind::Custom && event.text == "burst" {
            (0..10)
                .map(|n| Command::Emit {
                    world: event.world.clone(),
                    text: n.to_string(),
                })
                .collect()
        } else {
            Vec::new()
        }
    });
    let (handler, seen) = recorder();
    let (aria, _) = world("alpha/aria", PW);
    let portal = Portal::start(
        Arc::new(MemoryJournal::new(4).unwrap()),
        vec![aria.with_handler(burst)],
        vec![handler],
    );
    portal.submit(emit("burst")).await.unwrap();
    idle(&portal).await;
    let seen = seqs(&seen.lock().unwrap());
    assert_eq!(
        seen,
        (1..=11).collect::<Vec<_>>(),
        "each event once, in order"
    );
}

#[tokio::test(start_paused = true)]
async fn handler_cannot_connect_after_stop() {
    let (aria, _) = world("alpha/aria", PW);
    let reconnect = on(|event| {
        if event.kind == EventKind::Disconnected {
            vec![Command::Connect {
                world: event.world.clone(),
            }]
        } else {
            Vec::new()
        }
    });
    let portal = started(vec![aria.with_handler(reconnect)]).await;
    portal.stop().await;
    idle(&portal).await;
    let connects = of_kind(&portal, EventKind::Connected).len();
    assert_eq!(connects, 1, "the handler brought the world back up");
    assert_eq!(
        portal.worlds(),
        [(id("alpha/aria"), WorldStatus::of(WorldState::Disconnected))]
    );
}

#[tokio::test(start_paused = true)]
async fn stray_mark_is_ignored() {
    let (aria, game) = world("alpha/aria", PW);
    let aria = aria.with_framer(on_line("mark", || Inbound::Mark));
    let portal = started(vec![aria]).await;
    assert!(game.say("mark"));
    assert!(game.say("after"));
    wait_event(&portal, |event| {
        event.kind == EventKind::Line && event.text == "after"
    })
    .await;
    assert_eq!(
        portal.worlds(),
        [(id("alpha/aria"), WorldStatus::of(WorldState::Connected))]
    );
}

/// A line framer that gives `becomes` for the game line `trigger`.
struct OnLine {
    inner: LineFramer,
    trigger: &'static str,
    becomes: fn() -> Inbound,
}

impl Framer for OnLine {
    fn feed(&mut self, bytes: &[u8]) -> Vec<Inbound> {
        self.inner
            .feed(bytes)
            .into_iter()
            .map(|inbound| match inbound {
                Inbound::Line(line) if line == self.trigger => (self.becomes)(),
                other => other,
            })
            .collect()
    }

    fn encode(&self, line: &str) -> Vec<u8> {
        self.inner.encode(line)
    }

    fn mark(&self) -> Vec<u8> {
        self.inner.mark()
    }
}

fn on_line(trigger: &'static str, becomes: fn() -> Inbound) -> mu_core::NewFramer {
    Arc::new(move || {
        Box::new(OnLine {
            inner: LineFramer::new(),
            trigger,
            becomes,
        })
    })
}

fn reply_on_ask() -> mu_core::NewFramer {
    on_line("ask", || Inbound::Reply(b"reply\n".to_vec()))
}

#[tokio::test(start_paused = true)]
async fn reply_of_the_framer_reaches_the_game() {
    let (aria, game) = world("alpha/aria", PW);
    let portal = started(vec![aria.with_framer(reply_on_ask())]).await;
    assert!(game.say("ask"));
    assert_eq!(game.wait_for_lines(2).await[1], "reply");
    idle(&portal).await;
    assert!(events(&portal).iter().all(|event| event.text != "ask"));
    assert_eq!(
        portal.worlds(),
        [(id("alpha/aria"), WorldStatus::of(WorldState::Connected))]
    );
}

#[tokio::test(start_paused = true)]
async fn failed_reply_write_ends_the_connection() {
    // The write of the mark gets through. Then the game says `ask` and `after`, and the
    // write of the reply fails.
    let transport = BrokenTransport::after_writes(1, b"ask\nafter\n");
    let (portal, _) = broken_world(bare_world(transport).with_framer(reply_on_ask())).await;
    assert_eq!(try_idle(&portal).await, Ok(()));
    assert_eq!(of_kind(&portal, EventKind::Disconnected).len(), 1);
    assert_eq!(of_kind(&portal, EventKind::Line), []);
    assert_eq!(
        portal.worlds(),
        [(id("alpha/aria"), WorldStatus::of(WorldState::Disconnected))]
    );
}

#[tokio::test(start_paused = true)]
async fn wait_covers_server_handler_loop() {
    let portal = echo_custom_loop();
    as_task(async move {
        portal.submit(emit("ping")).await.unwrap();
        idle(&portal).await;
        assert_eq!(of_kind(&portal, EventKind::Custom).len(), 1 + DEPTH);
        assert_eq!(of_kind(&portal, EventKind::ScriptError).len(), 1);
    })
    .await;
}
