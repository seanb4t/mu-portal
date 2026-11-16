//! The scenarios of the terminal-client capability on `App`, drawn on a `TestBackend`.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mu_client::ClientEvent;
use mu_proto::{Event, EventKind, Source};
use mu_tui::{Action, App, Reply};
use proptest::prelude::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn app(worlds: &[&str]) -> App {
    App::new(worlds.iter().map(|world| (*world).to_owned()))
}

/// The rows of an 80 by 24 screen, with no trailing spaces.
fn screen(app: &App) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
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

fn top(app: &App) -> String {
    screen(app).remove(0)
}

/// The rows between the top row and the input line, with no empty row.
fn rows(app: &App) -> Vec<String> {
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
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        assert_eq!(app.on_key(key(KeyCode::Char(c))), None);
    }
}

/// Two worlds, each with its greeting.
fn two_worlds() -> App {
    let mut app = app(&["alpha/aria", "beta/bryn"]);
    app.on_event(game("alpha/aria", "Welcome, aria."));
    app.on_event(game("beta/bryn", "Welcome, bryn."));
    app
}

#[test]
fn first_view() {
    let app = two_worlds();
    assert!(
        top(&app).starts_with("[alpha/aria] beta/bryn"),
        "{}",
        top(&app)
    );
    assert_eq!(rows(&app), ["Welcome, aria."]);
}

#[test]
fn world_order() {
    let app = app(&["alpha-2/cy", "alpha/bea", "alpha/aria"]);
    assert_eq!(top(&app), "[alpha/aria] alpha/bea alpha-2/cy");
}

#[test]
fn no_worlds() {
    assert_eq!(top(&app(&[])), "no worlds");
}

#[test]
fn line_prefixes() {
    let mut app = app(&["alpha/aria"]);
    app.on_event(event(
        "alpha/aria",
        Source::SOURCE_CREDENTIAL,
        "connect aria ****",
    ));
    app.on_event(event("alpha/aria", Source::SOURCE_PLAYER, "hello"));
    app.on_event(game("alpha/aria", "You say, \"hello\""));
    assert_eq!(
        rows(&app),
        ["-- connect aria ****", "> hello", "You say, \"hello\""]
    );
}

#[test]
fn enter_sends_to_world_in_focus() {
    let mut app = two_worlds();
    app.on_key(ctrl('n'));
    type_text(&mut app, "hi");
    let action = app.on_key(key(KeyCode::Enter));
    assert_eq!(
        action,
        Some(Action::Send {
            world: "beta/bryn".to_owned(),
            text: "hi".to_owned()
        })
    );
    assert_eq!(app.input(), "");
}

#[test]
fn empty_enter_sends_empty_line() {
    let mut app = two_worlds();
    let action = app.on_key(key(KeyCode::Enter));
    assert_eq!(
        action,
        Some(Action::Send {
            world: "alpha/aria".to_owned(),
            text: String::new()
        })
    );
}

#[test]
fn enter_with_no_world() {
    let mut app = app(&[]);
    type_text(&mut app, "hi");
    assert_eq!(app.on_key(key(KeyCode::Enter)), None);
    let rows = rows(&app);
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
    assert!(app.on_key(key(KeyCode::Enter)).is_some());
    app.on_reply(Reply::Refused {
        world: "alpha/aria".to_owned(),
        text: "hi".to_owned(),
        reason: "alpha/aria is not connected".to_owned(),
    });
    assert_eq!(rows(&app).last().unwrap(), "-- alpha/aria is not connected");
    assert_eq!(app.input(), "hi");
    assert_eq!(screen(&app).last().unwrap(), "hi");
}

#[test]
fn ctrl_n_moves_focus() {
    let mut app = two_worlds();
    app.on_key(ctrl('n'));
    assert_eq!(top(&app), "alpha/aria [beta/bryn]");
    assert_eq!(rows(&app), ["Welcome, bryn."]);
}

#[test]
fn ctrl_p_wraps_around() {
    let mut app = two_worlds();
    app.on_key(ctrl('p'));
    assert_eq!(top(&app), "alpha/aria [beta/bryn]");
}

#[test]
fn ctrl_n_wraps_at_the_end() {
    let mut app = two_worlds();
    app.on_key(ctrl('n'));
    app.on_key(ctrl('n'));
    assert_eq!(top(&app), "[alpha/aria] beta/bryn");
}

#[test]
fn keys_with_no_world() {
    let mut app = app(&[]);
    assert_eq!(app.on_key(ctrl('n')), None);
    assert_eq!(app.on_key(ctrl('p')), None);
    assert_eq!(top(&app), "no worlds");
}

#[test]
fn one_world_keeps_focus() {
    let mut app = app(&["alpha/aria"]);
    app.on_key(ctrl('n'));
    app.on_key(ctrl('p'));
    assert_eq!(top(&app), "[alpha/aria]");
}

#[test]
fn activity_mark() {
    let mut app = app(&["alpha/aria", "beta/bryn"]);
    app.on_event(game("beta/bryn", "You say, \"ping\""));
    assert_eq!(top(&app), "[alpha/aria] beta/bryn*");
    app.on_key(ctrl('n'));
    assert_eq!(top(&app), "alpha/aria [beta/bryn]");
}

#[test]
fn lost_stream_resumes() {
    let mut app = two_worlds();
    app.on_event(ClientEvent::Lost);
    assert!(top(&app).contains("disconnected"));
    app.on_event(ClientEvent::Resumed);
    app.on_event(game("alpha/aria", "You say, \"after the cut\""));
    let rows = rows(&app);
    assert_eq!(
        rows.iter()
            .filter(|row| *row == "You say, \"after the cut\"")
            .count(),
        1
    );
    assert!(!top(&app).contains("disconnected"));
}

#[test]
fn resumed_clears_disconnected() {
    let mut app = two_worlds();
    app.on_event(ClientEvent::Lost);
    assert!(top(&app).contains("disconnected"));
    app.on_event(ClientEvent::Resumed);
    assert!(!top(&app).contains("disconnected"));
}

#[test]
fn ctrl_c_quits() {
    assert_eq!(app(&["alpha/aria"]).on_key(ctrl('c')), Some(Action::Quit));
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
                Step::Next => { app.on_key(ctrl('n')); }
                Step::Previous => { app.on_key(ctrl('p')); }
                Step::Enter => { app.on_key(key(KeyCode::Enter)); }
                Step::Line(n) => app.on_event(game(IDS[n], "line")),
                Step::Reset => app.on_event(ClientEvent::Reset { instance_id: "i".to_owned() }),
                Step::Lost => app.on_event(ClientEvent::Lost),
            }
            let ids = app.world_ids();
            match app.focused() {
                Some(focus) => prop_assert!(ids.contains(&focus)),
                None => prop_assert!(ids.is_empty()),
            }
        }
    }
}
