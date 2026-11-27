//! Helpers that the tests of `update` and `view` share.

#![allow(dead_code, reason = "each test binary uses a part of this module")]

use mu_config::layout::BlockId;
use mu_tui::{Effect, Model, Msg, update, view};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use termina::Event as TermEvent;
use termina::event::{KeyCode, KeyEvent, Modifiers};

/// The block id `id`.
pub fn block(id: &str) -> BlockId {
    BlockId::new(id).expect("an id")
}

/// The rectangle of the pane of `id` on an 80 by 24 screen, as `(first column, last column,
/// first row, last row)`, each from 1.
pub fn rect_of(app: &Model, id: &str) -> (u16, u16, u16, u16) {
    let solved = mu_tui::solved(app, 80, 24);
    let area = solved
        .pane_of(&block(id))
        .unwrap_or_else(|| panic!("no pane of {id}"))
        .area;
    (
        area.x + 1,
        area.x + area.width,
        area.y + 1,
        area.y + area.height,
    )
}

/// The rows of the pane of `id` on an 80 by 24 screen, with no trailing spaces and no empty
/// row.
pub fn block_rows(app: &mut Model, id: &str) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| view(app, frame)).unwrap();
    let buffer = terminal.backend().buffer();
    let (left, right, top, bottom) = rect_of(app, id);
    (top - 1..bottom)
        .map(|y| {
            let row: String = (left - 1..right).map(|x| buffer[(x, y)].symbol()).collect();
            row.trim_end().to_owned()
        })
        .filter(|row| !row.is_empty())
        .collect()
}

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
