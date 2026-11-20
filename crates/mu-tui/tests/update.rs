//! `update` for each default entry of the keymap, with no terminal.

use mu_tui::{Effect, Model, Msg, update};
use termina::Event as TermEvent;
use termina::event::{KeyCode, KeyEvent, Modifiers};

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
    let mut model = model();
    let effects = press(&mut model, KeyCode::Char('p'), Modifiers::CONTROL);
    assert_eq!(effects, []);
    assert_eq!(model.focused(), Some("beta/bryn"));
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
