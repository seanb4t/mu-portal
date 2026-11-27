//! Helpers that the tests of `update` and `view` share.

#![allow(dead_code, reason = "each test binary uses a part of this module")]

use mu_client::ClientEvent;
use mu_config::layout::BlockId;
use mu_proto::{Event, EventKind, Source, World};
use mu_tui::{Effect, Model, Msg, Reply, update, view};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use termina::Event as TermEvent;
use termina::event::{KeyCode, KeyEvent, Modifiers};

/// The lines of the built-in layout `sidebar`.
pub fn sidebar_lines() -> &'static [&'static str] {
    mu_config::layout::BuiltIn::named("sidebar")
        .expect("mu ships the layout sidebar")
        .lines
}

/// The world `id` as the server lists it.
pub fn world_of(id: &str) -> World {
    World {
        id: id.to_owned(),
        ..Default::default()
    }
}

/// The reply that refuses the line `text` of `world` for `reason`.
pub fn refused(world: &str, text: &str, reason: &str) -> Reply {
    Reply::Refused {
        world: world.to_owned(),
        text: text.to_owned(),
        reason: reason.to_owned(),
    }
}

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

/// Types `text` and presses Enter, and returns the effects.
pub fn run_command(model: &mut Model, text: &str) -> Vec<Effect> {
    type_text(model, text);
    press(model, KeyCode::Enter, Modifiers::NONE)
}

/// The line `text` from the game of `world`, in the streams `streams`.
pub fn game_event(world: &str, text: &str, streams: &[&str]) -> ClientEvent {
    ClientEvent::Event(Event {
        world: world.to_owned(),
        kind: EventKind::EVENT_KIND_LINE.into(),
        source: Source::SOURCE_GAME.into(),
        text: text.to_owned(),
        streams: streams.iter().map(ToString::to_string).collect(),
        ..Default::default()
    })
}

/// The message of [`game_event`].
pub fn game_line(world: &str, text: &str, streams: &[&str]) -> Msg {
    Msg::Client(game_event(world, text, streams))
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
