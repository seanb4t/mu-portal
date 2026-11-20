//! `update` for each default entry of the keymap, with no terminal.

use mu_client::ClientEvent;
use mu_proto::{Event, EventKind, Source};
use mu_tui::store::CAPACITY;
use mu_tui::{Effect, Model, Msg, Position, update, view};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use termina::Event as TermEvent;
use termina::event::{KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseEventKind};

fn model() -> Model {
    Model::new(["alpha/aria".to_owned(), "beta/bryn".to_owned()])
}

fn press(model: &mut Model, code: KeyCode, modifiers: Modifiers) -> Vec<Effect> {
    update(
        model,
        Msg::Terminal(TermEvent::Key(KeyEvent::new(code, modifiers))),
    )
}

fn type_text(model: &mut Model, text: &str) {
    for c in text.chars() {
        assert_eq!(press(model, KeyCode::Char(c), Modifiers::NONE), []);
    }
}

#[test]
fn ctrl_c_returns_quit_and_keeps_the_model() {
    let mut model = model();
    type_text(&mut model, "hi");
    let effects = press(&mut model, KeyCode::Char('c'), Modifiers::CONTROL);
    assert_eq!(effects, [Effect::Quit]);
    assert_eq!((model.focused(), model.input()), (Some("alpha/aria"), "hi"));
}

#[test]
fn ctrl_n_moves_the_focus_with_no_effect() {
    let mut model = model();
    let effects = press(&mut model, KeyCode::Char('n'), Modifiers::CONTROL);
    assert_eq!(effects, []);
    assert_eq!(model.focused(), Some("beta/bryn"));
}

#[test]
fn ctrl_p_moves_the_focus_back_with_no_effect() {
    let mut model = Model::new(["alpha/aria", "beta/bryn", "gamma/gale"].map(str::to_owned));
    let effects = press(&mut model, KeyCode::Char('p'), Modifiers::CONTROL);
    assert_eq!((effects, model.focused()), (vec![], Some("gamma/gale")));
}

#[test]
fn enter_returns_send_and_clears_the_input_line() {
    let mut model = model();
    type_text(&mut model, "look");
    let effects = press(&mut model, KeyCode::Enter, Modifiers::NONE);
    assert_eq!(
        effects,
        [Effect::Send {
            world: "alpha/aria".to_owned(),
            text: "look".to_owned()
        }]
    );
    assert_eq!(model.input(), "");
}

fn game_lines(model: &mut Model, lines: std::ops::Range<usize>) {
    for n in lines {
        let event = Event {
            world: "alpha/aria".to_owned(),
            kind: EventKind::EVENT_KIND_LINE.into(),
            source: Source::SOURCE_GAME.into(),
            text: n.to_string(),
            ..Default::default()
        };
        assert_eq!(update(model, Msg::Client(ClientEvent::Event(event))), []);
    }
}

#[test]
fn press_after_a_trim() {
    let mut model = model();
    game_lines(&mut model, 0..CAPACITY);
    let mut terminal = Terminal::new(TestBackend::new(20, 5)).unwrap();
    terminal.draw(|frame| view(&mut model, frame)).unwrap();
    game_lines(&mut model, 0..CAPACITY);
    let press = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 0,
        row: 1,
        modifiers: Modifiers::NONE,
    };
    assert_eq!(
        update(&mut model, Msg::Terminal(TermEvent::Mouse(press))),
        []
    );
    let pane = model.pane(model.focused_pane()).unwrap();
    let oldest = Position {
        line: pane.store().first_id(),
        byte: 0,
    };
    assert_eq!(pane.selection(), Some((oldest, oldest)));
}
