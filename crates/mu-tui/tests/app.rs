//! The scenarios of the terminal-client capability on `update` and `view`, drawn on a
//! `TestBackend`.

use mu_client::ClientEvent;
use mu_proto::{Event, EventKind, Source};
use mu_tui::{Effect, Model, Msg, Reply, update, view};
use proptest::prelude::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use termina::Event as TermEvent;
use termina::event::{KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseEventKind};

fn app(worlds: &[&str]) -> Model {
    Model::new(worlds.iter().map(|world| (*world).to_owned()))
}

/// The rows of an 80 by 24 screen, with no trailing spaces.
fn screen(app: &mut Model) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| view(app, frame)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..buffer.area.height)
        .map(|y| {
            let row: String = (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect();
            row.trim_end().to_owned()
        })
        .collect()
}

fn top(app: &mut Model) -> String {
    screen(app).remove(0)
}

/// The rows between the top row and the input line, with no empty row.
fn rows(app: &mut Model) -> Vec<String> {
    let screen = screen(app);
    screen[1..screen.len() - 1]
        .iter()
        .filter(|row| !row.is_empty())
        .cloned()
        .collect()
}

fn event(world: &str, source: Source, text: &str) -> ClientEvent {
    let kind = match source {
        Source::SOURCE_PLAYER => EventKind::EVENT_KIND_INPUT,
        Source::SOURCE_CREDENTIAL => EventKind::EVENT_KIND_LOGIN,
        _ => EventKind::EVENT_KIND_LINE,
    };
    ClientEvent::Event(Event {
        world: world.to_owned(),
        kind: kind.into(),
        source: source.into(),
        text: text.to_owned(),
        ..Default::default()
    })
}

fn game(world: &str, text: &str) -> ClientEvent {
    event(world, Source::SOURCE_GAME, text)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, Modifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), Modifiers::CONTROL)
}

/// Presses `key` and returns the effects.
fn press(app: &mut Model, key: KeyEvent) -> Vec<Effect> {
    update(app, Msg::Terminal(TermEvent::Key(key)))
}

fn on_event(app: &mut Model, item: ClientEvent) {
    assert_eq!(update(app, Msg::Client(item)), []);
}

fn on_reply(app: &mut Model, reply: Reply) {
    assert_eq!(update(app, Msg::Reply(reply)), []);
}

fn type_text(app: &mut Model, text: &str) {
    for c in text.chars() {
        assert_eq!(press(app, key(KeyCode::Char(c))), []);
    }
}

/// Two worlds, each with its greeting.
fn two_worlds() -> Model {
    let mut app = app(&["alpha/aria", "beta/bryn"]);
    on_event(&mut app, game("alpha/aria", "Welcome, aria."));
    on_event(&mut app, game("beta/bryn", "Welcome, bryn."));
    app
}

#[test]
fn first_view() {
    let mut app = two_worlds();
    assert!(
        top(&mut app).starts_with("[alpha/aria] beta/bryn"),
        "{}",
        top(&mut app)
    );
    assert_eq!(rows(&mut app), ["Welcome, aria."]);
}

#[test]
fn first_view_snapshot() {
    let mut app = two_worlds();
    type_text(&mut app, "look");
    let mut terminal = Terminal::new(TestBackend::new(40, 6)).unwrap();
    terminal.draw(|frame| view(&mut app, frame)).unwrap();
    insta::assert_snapshot!(terminal.backend());
}

#[test]
fn world_order() {
    let mut app = app(&["alpha-2/cy", "alpha/bea", "alpha/aria"]);
    assert_eq!(top(&mut app), "[alpha/aria] alpha/bea alpha-2/cy");
}

#[test]
fn no_worlds() {
    assert_eq!(top(&mut app(&[])), "no worlds");
}

#[test]
fn line_prefixes() {
    let mut app = app(&["alpha/aria"]);
    on_event(
        &mut app,
        event("alpha/aria", Source::SOURCE_CREDENTIAL, "connect aria ****"),
    );
    on_event(
        &mut app,
        event("alpha/aria", Source::SOURCE_PLAYER, "hello"),
    );
    on_event(&mut app, game("alpha/aria", "You say, \"hello\""));
    assert_eq!(
        rows(&mut app),
        ["-- connect aria ****", "> hello", "You say, \"hello\""]
    );
}

#[test]
fn control_characters_are_replaced() {
    let mut app = app(&["alpha/aria"]);
    on_event(
        &mut app,
        game("alpha/aria", "\u{1b}[2Jhi\tthere\u{7}\u{7f}\u{9b}~\u{a0}."),
    );
    on_reply(
        &mut app,
        Reply::Refused {
            world: "alpha/aria".to_owned(),
            text: String::new(),
            reason: "bad\u{1b}]0;title\u{7}".to_owned(),
        },
    );
    assert_eq!(
        rows(&mut app),
        [
            "\u{fffd}[2Jhi there\u{fffd}\u{fffd}\u{fffd}~\u{a0}.",
            "-- bad\u{fffd}]0;title\u{fffd}"
        ]
    );
}

#[test]
fn control_characters_are_replaced_in_every_part() {
    let world = "alpha/a\u{1b}x";
    let mut app = app(&[world]);
    on_event(&mut app, event(world, Source::SOURCE_PLAYER, "hi\u{7}"));
    on_event(&mut app, event(world, Source::SOURCE_SYSTEM, "up\u{9b}"));
    on_event(
        &mut app,
        event(world, Source::SOURCE_CREDENTIAL, "pw\u{7f}"),
    );
    assert_eq!(top(&mut app), "[alpha/a\u{fffd}x]");
    assert_eq!(
        rows(&mut app),
        ["> hi\u{fffd}", "-- up\u{fffd}", "-- pw\u{fffd}"]
    );
}

#[test]
fn enter_sends_to_world_in_focus() {
    let mut app = two_worlds();
    press(&mut app, ctrl('n'));
    type_text(&mut app, "hi");
    let action = press(&mut app, key(KeyCode::Enter));
    assert_eq!(
        action,
        [Effect::Send {
            world: "beta/bryn".to_owned(),
            text: "hi".to_owned()
        }]
    );
    assert_eq!(app.input(), "");
}

#[test]
fn empty_enter_sends_empty_line() {
    let mut app = two_worlds();
    let action = press(&mut app, key(KeyCode::Enter));
    assert_eq!(
        action,
        [Effect::Send {
            world: "alpha/aria".to_owned(),
            text: String::new()
        }]
    );
}

#[test]
fn enter_with_no_world() {
    let mut app = app(&[]);
    type_text(&mut app, "hi");
    assert_eq!(press(&mut app, key(KeyCode::Enter)), []);
    let rows = rows(&mut app);
    assert!(
        rows.iter()
            .any(|row| row.starts_with("-- ") && row.contains("no world")),
        "{rows:?}"
    );
    assert_eq!(app.input(), "hi");
}

#[test]
fn refused_line_keeps_input() {
    let mut app = two_worlds();
    type_text(&mut app, "hi");
    assert_ne!(press(&mut app, key(KeyCode::Enter)), [] as [Effect; 0]);
    on_reply(
        &mut app,
        Reply::Refused {
            world: "alpha/aria".to_owned(),
            text: "hi".to_owned(),
            reason: "alpha/aria is not connected".to_owned(),
        },
    );
    assert_eq!(
        rows(&mut app).last().unwrap(),
        "-- alpha/aria is not connected"
    );
    assert_eq!(app.input(), "hi");
    assert_eq!(screen(&mut app).last().unwrap(), "hi");
}

#[test]
fn ctrl_n_moves_focus() {
    let mut app = two_worlds();
    press(&mut app, ctrl('n'));
    assert_eq!(top(&mut app), "alpha/aria [beta/bryn]");
    assert_eq!(rows(&mut app), ["Welcome, bryn."]);
}

#[test]
fn ctrl_p_wraps_around() {
    let mut app = two_worlds();
    press(&mut app, ctrl('p'));
    assert_eq!(top(&mut app), "alpha/aria [beta/bryn]");
}

#[test]
fn ctrl_n_wraps_at_the_end() {
    let mut app = two_worlds();
    press(&mut app, ctrl('n'));
    press(&mut app, ctrl('n'));
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn");
}

#[test]
fn keys_with_no_world() {
    let mut app = app(&[]);
    assert_eq!(press(&mut app, ctrl('n')), []);
    assert_eq!(press(&mut app, ctrl('p')), []);
    assert_eq!(top(&mut app), "no worlds");
}

#[test]
fn one_world_keeps_focus() {
    let mut app = app(&["alpha/aria"]);
    press(&mut app, ctrl('n'));
    press(&mut app, ctrl('p'));
    assert_eq!(top(&mut app), "[alpha/aria]");
}

#[test]
fn activity_mark() {
    let mut app = app(&["alpha/aria", "beta/bryn"]);
    on_event(&mut app, game("beta/bryn", "You say, \"ping\""));
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn*");
    press(&mut app, ctrl('n'));
    assert_eq!(top(&mut app), "alpha/aria [beta/bryn]");
}

#[test]
fn lost_stream_resumes() {
    let mut app = two_worlds();
    on_event(&mut app, ClientEvent::Lost);
    assert!(top(&mut app).contains("disconnected"));
    on_event(&mut app, ClientEvent::Resumed);
    on_event(&mut app, game("alpha/aria", "You say, \"after the cut\""));
    let rows = rows(&mut app);
    assert_eq!(
        rows.iter()
            .filter(|row| *row == "You say, \"after the cut\"")
            .count(),
        1
    );
    assert!(!top(&mut app).contains("disconnected"));
}

#[test]
fn resumed_clears_disconnected() {
    let mut app = two_worlds();
    on_event(&mut app, ClientEvent::Lost);
    assert!(top(&mut app).contains("disconnected"));
    on_event(&mut app, ClientEvent::Resumed);
    assert!(!top(&mut app).contains("disconnected"));
}

/// A prompt event of the game: text with no line end.
fn prompt(world: &str, text: &str) -> ClientEvent {
    ClientEvent::Event(Event {
        world: world.to_owned(),
        kind: EventKind::EVENT_KIND_PROMPT.into(),
        source: Source::SOURCE_GAME.into(),
        text: text.to_owned(),
        ..Default::default()
    })
}

#[test]
fn press_on_the_unfinished_line() {
    let mut app = app(&["alpha/aria"]);
    on_event(&mut app, game("alpha/aria", "Welcome, aria."));
    on_event(&mut app, prompt("alpha/aria", "Name: "));
    assert_eq!(screen(&mut app)[22], "Name:");
    let press = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 1,
        row: 22,
        modifiers: Modifiers::NONE,
    };
    assert_eq!(update(&mut app, Msg::Terminal(TermEvent::Mouse(press))), []);
    assert_eq!(app.pane(app.focused_pane()).unwrap().selection(), None);
}

#[test]
fn ctrl_c_quits() {
    assert_eq!(press(&mut app(&["alpha/aria"]), ctrl('c')), [Effect::Quit]);
}

/// The view of `text` on the screen: a tab as a space and each other control character as
/// U+FFFD.
fn shown(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\t' => ' ',
            c if c.is_control() => '\u{fffd}',
            c => c,
        })
        .collect()
}

proptest! {
    #[test]
    fn no_control_character_is_drawn(
        character in "[a\\x00-\\x1f\\x7f\\u{80}-\\u{9f}]{1,10}",
        line in "[a \\x00-\\x1f\\x7f\\u{80}-\\u{9f}]{0,40}",
        source in prop::sample::select(vec![
            Source::SOURCE_GAME,
            Source::SOURCE_PLAYER,
            Source::SOURCE_SYSTEM,
            Source::SOURCE_CREDENTIAL,
        ]),
    ) {
        let world = format!("alpha/{character}");
        let mut app = app(&[&world]);
        on_event(&mut app, event(&world, source, &line));
        let prefix = match source {
            Source::SOURCE_PLAYER => "> ",
            Source::SOURCE_SYSTEM | Source::SOURCE_CREDENTIAL => "-- ",
            _ => "",
        };
        let want = format!("{prefix}{}", shown(&line));
        let drawn = rows(&mut app).first().cloned().unwrap_or_default();
        prop_assert_eq!(drawn, want.trim_end());
        let top_want = format!("[alpha/{}]", shown(&character));
        prop_assert_eq!(top(&mut app), top_want.trim_end());
    }
}

#[derive(Clone, Debug)]
enum Step {
    Next,
    Previous,
    Enter,
    Line(usize),
    Reset,
    Lost,
}

const IDS: [&str; 4] = ["alpha/aria", "beta/bryn", "alpha-2/cy", "alpha/bea"];

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        Just(Step::Next),
        Just(Step::Previous),
        Just(Step::Enter),
        (0..IDS.len()).prop_map(Step::Line),
        Just(Step::Reset),
        Just(Step::Lost),
    ]
}

proptest! {
    #[test]
    fn focus_is_always_valid(start in 0..=IDS.len(), steps in proptest::collection::vec(step(), 0..40)) {
        let mut app = app(&IDS[..start]);
        for step in steps {
            match step {
                Step::Next => { press(&mut app, ctrl('n')); }
                Step::Previous => { press(&mut app, ctrl('p')); }
                Step::Enter => { press(&mut app, key(KeyCode::Enter)); }
                Step::Line(n) => on_event(&mut app, game(IDS[n], "line")),
                Step::Reset => on_event(&mut app, ClientEvent::Reset { instance_id: "i".to_owned() }),
                Step::Lost => on_event(&mut app, ClientEvent::Lost),
            }
            let ids = app.world_ids();
            match app.focused() {
                Some(focus) => prop_assert!(ids.contains(&focus)),
                None => prop_assert!(ids.is_empty()),
            }
        }
    }
}
