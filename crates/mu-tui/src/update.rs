//! [`update`]: the only function that changes the [`Model`]. It does no I/O.

use std::sync::Arc;

use mu_client::ClientEvent;
use mu_proto::{EventKind, Source};
use ratatui::style::Style;
use ratatui_textarea::{Input, Key};
use termina::Event as TermEvent;
use termina::event::{
    KeyCode, KeyEvent, KeyEventKind, Modifiers, MouseButton, MouseEvent, MouseEventKind,
};

use crate::convert::convert;
use crate::model::{Model, PaneId};
use crate::pane::PaneState;
use crate::text::{clean, line};

/// One input of the model.
#[derive(Clone, Debug)]
pub enum Msg {
    /// An event of the terminal.
    Terminal(TermEvent),
    /// An action from a source other than the keymap.
    Action(Action),
    /// An item of the event stream of the server.
    Client(ClientEvent),
    /// The reply of a call.
    Reply(Reply),
}

/// What a key or a client command means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Leave the client.
    Quit,
    /// Give the focus to the next world, after the last world the first.
    FocusNext,
    /// Give the focus to the previous world, before the first world the last.
    FocusPrev,
    /// Send the input line to the world in focus, and return its pane to the bottom.
    Submit,
    /// Scroll the pane in focus by this many pages: back for a negative count, forward for a
    /// positive one. A page is the height of the pane less one row, and at least one row.
    ScrollPage(i32),
    /// Scroll the pane in focus by this many rows: back for a negative count, forward for a
    /// positive one.
    ScrollRows(i32),
}

/// The rows that one step of the mouse wheel scrolls.
const WHEEL_ROWS: i32 = 3;

/// What the loop must do after an update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Send `text` to `world`, and give the result back as a [`Msg::Reply`].
    Send {
        /// The world in focus.
        world: String,
        /// The text of the input line.
        text: String,
    },
    /// Leave the client.
    Quit,
    /// Write `text` to the clipboard of the terminal.
    ClipboardWrite(String),
}

/// The result of an [`Effect::Send`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    /// The server took the line.
    Sent,
    /// The server refused the line.
    Refused {
        /// The world of the line.
        world: String,
        /// The text of the line, which goes back to the input line.
        text: String,
        /// Why the server refused it.
        reason: String,
    },
}

/// Changes `model` for `msg` and returns the effects for the loop to perform, in order.
pub fn update(model: &mut Model, msg: Msg) -> Vec<Effect> {
    match msg {
        Msg::Terminal(TermEvent::Key(key)) => on_key(model, key),
        Msg::Terminal(TermEvent::Mouse(mouse)) => on_mouse(model, mouse),
        Msg::Terminal(_) => Vec::new(),
        Msg::Action(action) => act(model, action),
        Msg::Client(item) => {
            on_client(model, item);
            Vec::new()
        }
        Msg::Reply(reply) => {
            on_reply(model, reply);
            Vec::new()
        }
    }
}

fn on_key(model: &mut Model, key: KeyEvent) -> Vec<Effect> {
    if key.kind == KeyEventKind::Release {
        return Vec::new();
    }
    if let Some(action) = model.keymap.get(&key) {
        act(model, action)
    } else {
        model.input.input(input_key(key));
        Vec::new()
    }
}

/// The wheel scrolls the pane in focus. A drag of the left button selects in the pane in focus,
/// and its release copies the selection. A press on no position clears the selection.
fn on_mouse(model: &mut Model, mouse: MouseEvent) -> Vec<Effect> {
    match mouse.kind {
        MouseEventKind::ScrollUp => act(model, Action::ScrollRows(-WHEEL_ROWS)),
        MouseEventKind::ScrollDown => act(model, Action::ScrollRows(WHEEL_ROWS)),
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(state) = model.panes.get_mut(&model.focused_pane()) {
                match state.position(mouse.column, mouse.row) {
                    Some(position) => state.press(position),
                    None => state.clear_selection(),
                }
            }
            Vec::new()
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if let Some(state) = model.panes.get_mut(&model.focused_pane())
                && let Some(position) = state.position(mouse.column, mouse.row)
            {
                state.drag(position);
            }
            Vec::new()
        }
        MouseEventKind::Up(MouseButton::Left) => model
            .panes
            .get_mut(&model.focused_pane())
            .and_then(PaneState::release)
            .map(Effect::ClipboardWrite)
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn act(model: &mut Model, action: Action) -> Vec<Effect> {
    match action {
        Action::Quit => vec![Effect::Quit],
        Action::FocusNext => {
            move_focus(model, true);
            Vec::new()
        }
        Action::FocusPrev => {
            move_focus(model, false);
            Vec::new()
        }
        Action::Submit => {
            let effects = submit(model).into_iter().collect();
            if let Some(state) = model.panes.get_mut(&model.focused_pane()) {
                state.to_bottom();
            }
            effects
        }
        Action::ScrollPage(pages) => {
            if let Some(state) = model.panes.get_mut(&model.focused_pane()) {
                state.scroll_pages(pages);
            }
            Vec::new()
        }
        Action::ScrollRows(rows) => {
            if let Some(state) = model.panes.get_mut(&model.focused_pane()) {
                state.scroll_rows(rows);
            }
            Vec::new()
        }
    }
}

fn submit(model: &mut Model) -> Option<Effect> {
    let Some(world) = model.focused().map(str::to_owned) else {
        model.push(
            PaneId::NOTICES,
            "-- no world has the focus, so the line goes nowhere",
        );
        return None;
    };
    let text = model.input().to_owned();
    model.set_input(String::new());
    Some(Effect::Send { world, text })
}

fn move_focus(model: &mut Model, forward: bool) {
    let count = model.worlds.len();
    let Some(focus) = model.focus.filter(|focus| *focus < count) else {
        return;
    };
    let next = if forward {
        if focus + 1 == count { 0 } else { focus + 1 }
    } else if focus == 0 {
        count - 1
    } else {
        focus - 1
    };
    model.focus = Some(next);
    model.worlds[next].unseen = false;
}

fn on_client(model: &mut Model, item: ClientEvent) {
    match item {
        ClientEvent::Reset { .. } => {
            model.lost = false;
            for world in &mut model.worlds {
                world.unseen = false;
                world.style = Style::default();
                if let Some(pane) = model.panes.get_mut(&world.pane) {
                    pane.clear();
                }
            }
        }
        ClientEvent::Resumed => model.lost = false,
        ClientEvent::Lost => model.lost = true,
        ClientEvent::Event(event) => {
            model.lost = false;
            let index = model.world(&event.world);
            let unseen = model.focus != Some(index);
            let world = &mut model.worlds[index];
            world.unseen |= unseen;
            let pane = model
                .panes
                .entry(world.pane)
                .or_insert_with(|| PaneState::with_profile(Arc::clone(&model.profile)));
            let prompt = event.kind.as_known() == Some(EventKind::EVENT_KIND_PROMPT);
            match event.source.as_known() {
                Some(Source::SOURCE_PLAYER | Source::SOURCE_SYSTEM | Source::SOURCE_CREDENTIAL) => {
                    pane.push(&line(&event));
                }
                _ if prompt => {
                    let mut style = world.style;
                    pane.set_unfinished(Some(convert(&event.text, &mut style)));
                }
                _ => {
                    let converted = convert(&event.text, &mut world.style);
                    pane.push_styled(&converted.text, &converted.spans);
                }
            }
        }
    }
}

fn on_reply(model: &mut Model, reply: Reply) {
    let Reply::Refused {
        world,
        text,
        reason,
    } = reply
    else {
        return;
    };
    let notice = format!("-- {}", clean(&reason));
    let pane = model
        .worlds
        .iter()
        .find(|each| each.id == world)
        .map_or(PaneId::NOTICES, |world| world.pane);
    model.push(pane, &notice);
    model.set_input(text);
}

/// `key` as the input line reads it. A key that the input line does not know becomes
/// `Key::Null`, which it ignores.
fn input_key(key: KeyEvent) -> Input {
    let key_code = match key.code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Function(n) => Key::F(n),
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Tab => Key::Tab,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Escape => Key::Esc,
        _ => Key::Null,
    };
    Input {
        key: key_code,
        ctrl: key.modifiers.contains(Modifiers::CONTROL),
        alt: key.modifiers.contains(Modifiers::ALT),
        shift: key.modifiers.contains(Modifiers::SHIFT),
    }
}
