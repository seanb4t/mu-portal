//! Helpers that the tests of `update` and `view` share.

#![allow(dead_code, reason = "each test binary uses a part of this module")]

use mu_tui::{Effect, Model, Msg, update};
use termina::Event as TermEvent;
use termina::event::{KeyCode, KeyEvent, Modifiers};

/// Presses the key `code` with `modifiers`, and returns the effects.
pub fn press(model: &mut Model, code: KeyCode, modifiers: Modifiers) -> Vec<Effect> {
    update(
        model,
        Msg::Terminal(TermEvent::Key(KeyEvent::new(code, modifiers))),
    )
}

/// Types each char of `text`.
pub fn type_text(model: &mut Model, text: &str) {
    for c in text.chars() {
        assert_eq!(press(model, KeyCode::Char(c), Modifiers::NONE), []);
    }
}

/// Types each line, with Ctrl+J between each two.
pub fn type_lines(model: &mut Model, lines: &[&str]) {
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            assert_eq!(press(model, KeyCode::Char('j'), Modifiers::CONTROL), []);
        }
        type_text(model, line);
    }
}
