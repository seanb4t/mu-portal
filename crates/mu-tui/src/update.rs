//! [`update`]: the function that changes the [`Model`] for each message. It does no I/O. Only
//! the render of [`view`](crate::view) also changes the model, as the module doc of
//! [`model`](crate::model) says.

use std::collections::VecDeque;
use std::sync::Arc;

use mu_client::ClientEvent;
use mu_proto::{EventKind, Source};
use ratatui::style::Style;
use ratatui_textarea::{Input, Key};
use termina::Event as TermEvent;
use termina::escape::csi::{Csi, DecPrivateMode, DecPrivateModeCode, Device, Mode};
use termina::event::{
    KeyCode, KeyEvent, KeyEventKind, Modifiers, MouseButton, MouseEvent, MouseEventKind,
};

use crate::convert::convert;
use crate::keymap::Wheel;
use crate::model::{Late, Model, PaneId};
use crate::pane::{Caps, PaneState};
use crate::probe::{self, Outcome};
use crate::store::Image;
use crate::text::{clean, clean_paste, line};

/// One input of the model.
#[derive(Clone, Debug)]
pub enum Msg {
    /// An event of the terminal.
    Terminal(TermEvent),
    /// An action from a source other than the keymap.
    Action(Action),
    /// An item of the event stream of the server.
    Client(ClientEvent),
    /// The result of the startup probe.
    Probe(Outcome),
    /// The bytes of an answer that the end of the probe cut, after the early input. The model
    /// holds them if it waits for late answers, and drops them if it does not.
    CutReply(Vec<u8>),
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
    /// Send the text of the pose editor to the world in focus as one command, and return its
    /// pane to the bottom.
    Submit,
    /// Send each line of the pose editor to the world in focus as its own command, in order,
    /// and return its pane to the bottom.
    SubmitLines,
    /// Insert a line break in the pose editor.
    Newline,
    /// Replace each line break of the pose editor with the line break of the world in focus.
    JoinLines,
    /// Delete the selection of the pose editor. With no selection, delete the text from the
    /// start of the line to the cursor.
    DeleteToLineStart,
    /// Undo the last edit of the pose editor.
    Undo,
    /// Bring back the edit that the last undo removed.
    Redo,
    /// Do nothing. An entry with it keeps a key from the pose editor.
    Ignore,
    /// Scroll the pane in focus by this many pages: back for a negative count, forward for a
    /// positive one. A page is the height of the text area less one row, and at least one row. The text
    /// area has no row of the unfinished line.
    ScrollPage(i32),
    /// Scroll the pane in focus by this many rows: back for a negative count, forward for a
    /// positive one.
    ScrollRows(i32),
}

/// What the loop must do after an update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Send `text` to `world`, and give the result back as a [`Msg::Reply`].
    Send {
        /// The world to send to: the world that had the focus at the key.
        world: String,
        /// The text of the pose editor, or one line of it.
        text: String,
    },
    /// Leave the client.
    Quit,
    /// Write `text` to the clipboard of the terminal.
    ClipboardWrite(String),
    /// Send the PNG `png` to the terminal as `image`, with a virtual placement of its size,
    /// before the next frame.
    TransmitImage {
        /// The id and the size of the image.
        image: Image,
        /// The bytes of a PNG file.
        png: Vec<u8>,
    },
}

/// The result of an [`Effect::Send`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    /// The server took the line.
    Sent {
        /// The world of the line.
        world: String,
        /// The text of the line.
        text: String,
    },
    /// The server refused the line.
    Refused {
        /// The world of the line.
        world: String,
        /// The text of the line, which goes back to the pose editor.
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
        Msg::Terminal(TermEvent::Paste(text)) => {
            // A paste replaces a selection, and one undo removes its text.
            model.pose().textarea().insert_str(clean_paste(&text));
            Vec::new()
        }
        Msg::Terminal(TermEvent::Csi(Csi::Device(Device::DeviceAttributes(())))) => {
            model.late = Late::None;
            Vec::new()
        }
        Msg::Terminal(TermEvent::Csi(Csi::Mode(Mode::ReportDecPrivateMode {
            mode: DecPrivateMode::Code(DecPrivateModeCode::GraphemeClustering),
            setting,
        }))) => {
            // Only the probe asks for the mode, and its reply comes before DA1.
            if model.late != Late::None {
                model.keeps_grapheme_mode = probe::is_set(setting);
            }
            Vec::new()
        }
        Msg::Terminal(_) => Vec::new(),
        Msg::Action(action) => act(model, action),
        Msg::Client(item) => {
            on_client(model, item);
            Vec::new()
        }
        Msg::Probe(outcome) => {
            on_probe(model, &outcome);
            Vec::new()
        }
        Msg::CutReply(bytes) => {
            if model.late == Late::Waiting && !bytes.is_empty() {
                model.hold_cut_reply(bytes);
            }
            Vec::new()
        }
        Msg::Reply(reply) => on_reply(model, reply),
    }
}

/// The notice when the probe gives no character widths.
const WIDTH_NOTICE: &str =
    "the terminal gave no character widths, so wide characters can be out of line";

fn on_probe(model: &mut Model, outcome: &Outcome) {
    model.set_caps(Caps::from(outcome));
    if !outcome.complete {
        model.await_late_replies();
    }
    if !outcome.measured {
        model.notice(WIDTH_NOTICE);
    }
}

/// The most keys of a late graphics reply. A reply that is longer ends here.
const GRAPHICS_KEYS: usize = 256;

/// The character of a key that is text: no modifier but Shift.
fn text(key: KeyEvent) -> Option<char> {
    match key.code {
        KeyCode::Char(c) if (key.modifiers - Modifiers::SHIFT).is_empty() => Some(c),
        _ => None,
    }
}

fn on_key(model: &mut Model, key: KeyEvent) -> Vec<Effect> {
    if key.kind == KeyEventKind::Release {
        return Vec::new();
    }
    let alt = |c: char| key.code == KeyCode::Char(c) && key.modifiers == Modifiers::ALT;
    match model.late {
        Late::None => {}
        Late::Waiting => {
            if alt('_') {
                model.late = Late::Graphics { keys: 0 };
                return Vec::new();
            }
        }
        Late::Graphics { keys } => {
            if alt('\\') {
                model.late = Late::Waiting;
                return Vec::new();
            }
            if key.code == KeyCode::Escape && key.modifiers.is_empty() {
                model.late = Late::GraphicsEscape;
                return Vec::new();
            }
            // A graphics reply starts with `G`. Any other key after Alt+`_` is the user's.
            if text(key).is_some()
                && (keys > 0 || key.code == KeyCode::Char('G'))
                && keys < GRAPHICS_KEYS
            {
                model.late = Late::Graphics { keys: keys + 1 };
                return Vec::new();
            }
            return not_a_graphics_reply(model, key);
        }
        Late::GraphicsEscape => {
            if text(key) == Some('\\') {
                model.late = Late::Waiting;
                return Vec::new();
            }
            return not_a_graphics_reply(model, key);
        }
        Late::Csi(ref mut bytes) => {
            let bytes = std::mem::take(bytes);
            return on_cut_reply_key(model, bytes, key);
        }
    }
    on_input_key(model, key)
}

/// `key` after the start of a graphics answer that did not go on. The model waits for late
/// answers again, and the Alt+`_` that started the answer and `key` are the user's.
fn not_a_graphics_reply(model: &mut Model, key: KeyEvent) -> Vec<Effect> {
    model.late = Late::Waiting;
    let mut effects = on_input_key(model, KeyEvent::new(KeyCode::Char('_'), Modifiers::ALT));
    effects.extend(on_input_key(model, key));
    effects
}

/// A key while the model holds `bytes` of an answer that the probe cut. A key that is a CSI byte
/// as text adds its byte. At the final byte, the bytes go to the decoder of the probe, and each
/// event is handled as a terminal event. After a lone ESC, `[` continues the CSI, and `_` starts
/// a graphics answer. Any other key, or a byte past [`probe::PENDING_MAX`], drops the held bytes,
/// and the key acts as it does while the model waits for late answers.
fn on_cut_reply_key(model: &mut Model, mut bytes: Vec<u8>, key: KeyEvent) -> Vec<Effect> {
    let byte = text(key)
        .and_then(|c| u8::try_from(c).ok())
        .filter(|&b| probe::is_csi_body(b) || probe::is_csi_final(b));
    match byte {
        Some(b'_') if bytes.len() == 1 => {
            model.late = Late::Graphics { keys: 0 };
            return Vec::new();
        }
        Some(byte) if bytes.len() < probe::PENDING_MAX && (bytes.len() > 1 || byte == b'[') => {
            bytes.push(byte);
            if bytes.len() > 2 && probe::is_csi_final(byte) {
                model.late = Late::Waiting;
                return probe::decode_csi(&bytes)
                    .into_iter()
                    .flat_map(|event| update(model, Msg::Terminal(event)))
                    .collect();
            }
            model.late = Late::Csi(bytes);
            return Vec::new();
        }
        _ => {}
    }
    model.late = Late::Waiting;
    on_key(model, key)
}

fn on_input_key(model: &mut Model, key: KeyEvent) -> Vec<Effect> {
    let action = model.keymap.get(&key);
    if action != Some(Action::Quit) {
        model.quit_armed = false;
    }
    if let Some(action) = action {
        act(model, action)
    } else {
        model.pose().textarea().input(input_key(key));
        Vec::new()
    }
}

/// The wheel scrolls the pane in focus. A drag of the left button selects in the pane in focus,
/// and its release copies the selection. A press on no position clears the selection.
fn on_mouse(model: &mut Model, mouse: MouseEvent) -> Vec<Effect> {
    match mouse.kind {
        MouseEventKind::ScrollUp => wheel(model, Wheel::Up),
        MouseEventKind::ScrollDown => wheel(model, Wheel::Down),
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

fn wheel(model: &mut Model, step: Wheel) -> Vec<Effect> {
    match model.keymap.wheel(step) {
        Some(action) => act(model, action),
        None => Vec::new(),
    }
}

fn act(model: &mut Model, action: Action) -> Vec<Effect> {
    match action {
        Action::Quit => {
            let waiting: usize = model.sends.values().map(VecDeque::len).sum();
            if waiting == 0 || model.quit_armed {
                return vec![Effect::Quit];
            }
            model.quit_armed = true;
            model.hint(&format!(
                "{} not sent yet, quit again to leave and drop them",
                line_count(waiting)
            ));
            Vec::new()
        }
        Action::FocusNext => {
            move_focus(model, true);
            Vec::new()
        }
        Action::FocusPrev => {
            move_focus(model, false);
            Vec::new()
        }
        Action::Submit | Action::SubmitLines => {
            let effects = if action == Action::Submit {
                submit(model)
            } else {
                submit_lines(model)
            };
            if let Some(state) = model.panes.get_mut(&model.focused_pane()) {
                state.to_bottom();
            }
            effects.into_iter().collect()
        }
        Action::Newline => {
            model.pose().textarea().insert_newline();
            Vec::new()
        }
        Action::JoinLines => {
            join_lines(model);
            Vec::new()
        }
        Action::DeleteToLineStart => {
            let textarea = model.pose().textarea();
            // At the start of a line, the delete of the library takes the line break before it.
            if textarea.cursor().1 > 0 {
                textarea.delete_line_by_head();
            }
            Vec::new()
        }
        Action::Undo => {
            model.pose().textarea().undo();
            Vec::new()
        }
        Action::Redo => {
            model.pose().textarea().redo();
            Vec::new()
        }
        Action::Ignore => Vec::new(),
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

/// The world in focus, or a notice that no world has the focus.
fn world_to_send_to(model: &mut Model) -> Option<String> {
    let world = model.focused().map(str::to_owned);
    if world.is_none() {
        model.push(
            PaneId::NOTICES,
            "-- no world has the focus, so the line goes nowhere",
        );
    }
    world
}

/// Adds `texts` to the sends of `world`, in order. If no send of the world waits, the first
/// one goes to the server now. If one waits, each of `texts` goes after the reply of the send
/// before it.
fn enqueue(
    model: &mut Model,
    world: String,
    texts: impl IntoIterator<Item = String>,
) -> Option<Effect> {
    let queue = model.sends.entry(world.clone()).or_default();
    let idle = queue.is_empty();
    queue.extend(texts);
    let text = queue.front().filter(|_| idle)?.clone();
    Some(Effect::Send { world, text })
}

/// Sends the text of the pose editor as one line to the world in focus.
fn submit(model: &mut Model) -> Option<Effect> {
    let world = world_to_send_to(model)?;
    let text = model.input();
    model.set_input("");
    enqueue(model, world, [text])
}

/// Sends each line of the pose editor as its own line. Each line goes to the world in focus
/// now, also if the focus moves before the last line goes.
fn submit_lines(model: &mut Model) -> Option<Effect> {
    let world = world_to_send_to(model)?;
    let text = model.input();
    model.set_input("");
    enqueue(model, world, text.split('\n').map(str::to_owned))
}

/// Replaces each line break of the pose editor with the line break of the world in focus. The
/// edit goes through the text area, so two undos bring the lines back: one for the joined text,
/// and one for the lines.
fn join_lines(model: &mut Model) {
    let line_break = model
        .focused_world()
        .and_then(|world| world.line_break.clone());
    let Some(line_break) = line_break else {
        model.hint("the world in focus sets no line break, so the lines stay as they are");
        return;
    };
    let text = model.input();
    if !text.contains('\n') {
        return;
    }
    let textarea = model.pose().textarea();
    textarea.select_all();
    textarea.insert_str(text.replace('\n', &line_break));
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
            // The Reset does not clear the notices pane, so its notices are still there.
            let pane = model.focused_pane();
            if pane != PaneId::NOTICES {
                for line in model.notices.clone() {
                    model.push(pane, &line);
                }
            }
            return_waiting(model, "the server started again");
        }
        ClientEvent::Resumed => model.lost = false,
        ClientEvent::Lost => {
            model.lost = true;
            return_waiting(model, "the client lost the server");
        }
        ClientEvent::Event(event) => {
            model.lost = false;
            let index = model.world(&event.world);
            let unseen = model.focus != Some(index);
            let world = &mut model.worlds[index];
            world.unseen |= unseen;
            let pane = model
                .panes
                .entry(world.pane)
                .or_insert_with(|| PaneState::with_caps(Arc::clone(&model.caps)));
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

/// The reply of the server to the first send of a world. After `Sent`, the next send of the
/// world goes. After `Refused`, no send of the world goes: the refused text and each send
/// after it go back to the editor, before the text that the player typed since.
fn on_reply(model: &mut Model, reply: Reply) -> Vec<Effect> {
    let (world, text, reason) = match reply {
        Reply::Sent { world, text } => return next_send(model, world, &text).into_iter().collect(),
        Reply::Refused {
            world,
            text,
            reason,
        } => (world, text, reason),
    };
    let notice = format!("-- {}", clean(&reason));
    let pane = model
        .worlds
        .iter()
        .find(|each| each.id == world)
        .map_or(PaneId::NOTICES, |world| world.pane);
    model.push(pane, &notice);
    let waits = model
        .sends
        .get(&world)
        .is_some_and(|queue| queue.front() == Some(&text));
    let lines: Vec<String> = if waits {
        model.sends.remove(&world).into_iter().flatten().collect()
    } else {
        vec![text]
    };
    put_back(model, lines);
    Vec::new()
}

/// `1 line` or `<count> lines`.
fn line_count(count: usize) -> String {
    if count == 1 {
        "1 line".to_owned()
    } else {
        format!("{count} lines")
    }
}

/// Puts `lines` back in the pose editor, before the text that the player typed since.
fn put_back(model: &mut Model, mut lines: Vec<String>) {
    let typed = model.input();
    if !typed.is_empty() {
        lines.push(typed);
    }
    model.set_input(&lines.join("\n"));
}

/// Puts the sends that wait behind the send with the server back in the pose editor, in the
/// order of the worlds, with a notice that names `why`. The send with the server stays, so its
/// reply still moves on or puts back only that send.
fn return_waiting(model: &mut Model, why: &str) {
    let mut lines = Vec::new();
    for world in &model.worlds {
        if let Some(queue) = model.sends.get_mut(&world.id) {
            lines.extend(queue.drain(1..));
        }
    }
    if lines.is_empty() {
        return;
    }
    model.notice(&format!(
        "{why}, so {} that waited went back to the editor",
        line_count(lines.len())
    ));
    put_back(model, lines);
}

/// The next send of `world`, after the server took `text`. Only the reply of the first send
/// of the world moves its sends on.
fn next_send(model: &mut Model, world: String, text: &str) -> Option<Effect> {
    let queue = model.sends.get_mut(&world)?;
    if queue.front().map(String::as_str) != Some(text) {
        return None;
    }
    queue.pop_front();
    let Some(next) = queue.front().cloned() else {
        model.sends.remove(&world);
        return None;
    };
    Some(Effect::Send { world, text: next })
}

/// `key` as the pose editor reads it. A key that the editor does not know becomes
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
