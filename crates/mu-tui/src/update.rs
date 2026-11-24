//! [`update`]: the function that changes the [`Model`] for each message. It does no I/O. Only
//! the render of [`view`](crate::view) also changes the model, as the module doc of
//! [`model`](crate::model) says.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use mu_client::ClientEvent;
use mu_proto::{EventKind, Source, World as ProtoWorld};
use ratatui::style::{Modifier, Style};
use ratatui_textarea::{CursorMove, Input, Key};
use termina::Event as TermEvent;
use termina::escape::csi::{Csi, DecPrivateMode, DecPrivateModeCode, Device, Mode};
use termina::event::{
    KeyCode, KeyEvent, KeyEventKind, Modifiers, MouseButton, MouseEvent, MouseEventKind,
};

use crate::command::{self, Parse};
use crate::config::Echo;
use crate::convert::convert;
use crate::history::HISTORY_LIMIT;
use crate::keymap::Wheel;
use crate::model::{ConfigRetry, Late, Listing, Model, PaneId, QuitArm};
use crate::pane::{Caps, PaneState};
use crate::probe::{self, Outcome};
use crate::registry::{ArgKind, EntryId, candidates};
use crate::store::Image;
use crate::text::{clean, clean_paste, echo_rows, line};

/// One input of the model.
#[derive(Clone, Debug)]
pub enum Msg {
    /// An event of the terminal.
    Terminal(TermEvent),
    /// Run an entry of the registry with its argument.
    Run {
        /// The entry to run.
        entry: EntryId,
        /// Its argument, or `None` for no argument.
        arg: Option<String>,
    },
    /// An item of the event stream of the server.
    Client(ClientEvent),
    /// The result of the startup probe.
    Probe(Outcome),
    /// The bytes of an answer that the end of the probe cut, after the early input. The model
    /// holds them if it waits for late answers, and drops them if it does not.
    CutReply(Vec<u8>),
    /// The reply of a call.
    Reply(Reply),
    /// The list of the worlds after an [`Effect::ListWorlds`], or the reason that it failed.
    Listed(Result<Vec<ProtoWorld>, String>),
    /// The client configuration after an [`Effect::FetchConfig`], or the reason that the call
    /// failed.
    Config(Result<mu_proto::GetClientConfigResponse, String>),
    /// The wait of an [`Effect::RetryConfig`] ended.
    RetryConfig,
    /// The result of an [`Effect::Connect`] or an [`Effect::Disconnect`].
    Control {
        /// The world of the call.
        world: String,
        /// `Ok` if the server took the call, else the reason that it failed: a refusal, an error
        /// of the connection, or no answer in time.
        result: Result<(), String>,
    },
}

/// What an entry of the registry does. [`Entry::build`](crate::registry::Entry::build) of each
/// entry makes the action of that entry.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Give the focus to the world at this place of the top row, from 1. With no world at that
    /// place, do nothing.
    FocusIndex(u8),
    /// Scroll the pane in focus to its oldest line.
    ScrollTop,
    /// Return the pane in focus to the bottom.
    ScrollBottom,
    /// Ask the server to connect this world, or the world in focus for `None`.
    Connect(Option<String>),
    /// Ask the server to disconnect this world, or the world in focus for `None`.
    Disconnect(Option<String>),
    /// Give the focus to this world. For a world that the model does not hold, add a hint.
    Focus(String),
    /// Show one line for each entry of the registry.
    Help,
    /// On the first line of the pose editor, put the previous entry of the history of the world
    /// in focus in the editor. On another line, move the cursor up.
    HistoryPrev,
    /// On the last line of the pose editor, put the next entry of the history, or the draft
    /// after the newest entry, in the editor. On another line, move the cursor down.
    HistoryNext,
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
    /// Ask the server for the list of the worlds, and give it back as a [`Msg::Listed`].
    ListWorlds,
    /// Ask the server to connect `world`, and give the result back as a [`Msg::Control`].
    Connect {
        /// The world to connect.
        world: String,
    },
    /// Ask the server to disconnect `world`, and give the result back as a [`Msg::Control`].
    Disconnect {
        /// The world to disconnect.
        world: String,
    },
    /// Ask the server for the client configuration, and give it back as a [`Msg::Config`].
    FetchConfig,
    /// Wait `after`, then give back a [`Msg::RetryConfig`].
    RetryConfig {
        /// The wait.
        after: Duration,
    },
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
        Msg::Run { entry, arg } => run_entry(model, entry, arg),
        Msg::Client(item) => on_client(model, item),
        Msg::Config(Ok(reply)) => {
            if model.latest_hash.as_ref() == Some(&reply.store_hash) {
                model.apply_config(reply);
                model.config_failures = 0;
            }
            Vec::new()
        }
        Msg::Config(Err(reason)) => config_failed(model, &reason),
        Msg::RetryConfig => {
            model.config_retry = ConfigRetry::Idle;
            if config_stale(model) {
                vec![Effect::FetchConfig]
            } else {
                Vec::new()
            }
        }
        Msg::Listed(listed) => {
            let again = model.listing == Listing::AskedBeforeReset;
            model.listing = Listing::Idle;
            // A failed list keeps its reason for Alt+R. A later new world, a reset, or Alt+R on
            // a world that no list held asks again.
            match listed {
                Ok(worlds) => {
                    model.list_failure = None;
                    model.learn(worlds);
                }
                Err(reason) => {
                    if model.first_list && !again {
                        model.notice(&format!("could not list the worlds: {reason}"));
                    }
                    model.list_failure = Some(reason);
                }
            }
            if again {
                // The reply can be of the server before the reset, so it does not end the first
                // list.
                list_worlds(model).into_iter().collect()
            } else {
                if model.first_list {
                    model.end_first_list();
                }
                Vec::new()
            }
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
        Msg::Control { world, result } => {
            if let Err(reason) = result {
                model.hint(&format!("{world}: {reason}"));
            }
            Vec::new()
        }
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
    if let Some(entry) = model.keymap.get(&key) {
        run_entry(model, entry, None)
    } else {
        if model.quit_arm == Some(QuitArm::Key) {
            model.quit_arm = None;
        }
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
        Some(entry) => run_entry(model, entry, None),
        None => Vec::new(),
    }
}

/// Builds the action of `entry` with `arg` and runs it. An entry that needs an argument and has
/// none, or that takes none and has one, runs nothing and gives a hint. Every action but a quit
/// cancels a quit that a key armed.
fn run_entry(model: &mut Model, entry: EntryId, arg: Option<String>) -> Vec<Effect> {
    let name = entry.entry().name;
    let arg = match arity(entry, arg, name) {
        Ok(arg) => arg,
        Err(reason) => {
            model.hint(&reason);
            return Vec::new();
        }
    };
    let action = (entry.entry().build)(arg);
    if action != Action::Quit && model.quit_arm == Some(QuitArm::Key) {
        model.quit_arm = None;
    }
    act(model, &action)
}

/// The text ` (<kept> in history)`, or nothing if the history kept no entry.
fn in_history(kept: usize) -> String {
    if kept > 0 {
        format!(" ({kept} in history)")
    } else {
        String::new()
    }
}

/// The text `no world has the focus, so <what>`.
fn no_world(what: &str) -> String {
    format!("no world has the focus, so {what}")
}

/// The world that an argument names, or the world in focus for `None`. `name` is the entry,
/// for the text of a refusal.
///
/// # Errors
///
/// The text of a hint if no world has the focus, or if the client holds no world `world`.
fn world_arg(model: &Model, arg: Option<String>, name: &str) -> Result<String, String> {
    match arg {
        None => model
            .focused()
            .map(str::to_owned)
            .ok_or_else(|| no_world(&format!("{name} needs a {}", ArgKind::World.word()))),
        Some(world) if candidates(model, ArgKind::World).contains(&world) => Ok(world),
        Some(world) => Err(format!("no world is named {world}")),
    }
}

/// Runs the command `parse`. A bad command leaves the editor as it is and adds a hint. A good
/// command clears the editor before it runs. Any command but `/quit` cancels an armed quit.
fn run_command(model: &mut Model, text: &str, parse: Parse<'_>) -> Vec<Effect> {
    let checked = check_command(model, parse);
    let entry = match checked {
        Ok((entry, _)) => Some(entry),
        Err(_) => None,
    };
    let quit = EntryId::of("quit");
    if entry != Some(quit) {
        model.quit_arm = None;
    }
    match checked {
        Err(reason) => {
            model.hint(&reason);
            Vec::new()
        }
        Ok((entry, arg)) => {
            record(model, text);
            model.set_input("");
            let effects = run_entry(model, entry, arg);
            if entry == quit && model.quit_arm.is_some() {
                model.quit_arm = Some(QuitArm::Command);
            }
            effects
        }
    }
}

/// The entry and the argument of the command `parse`, after the checks of a command.
///
/// # Errors
///
/// The text of a hint that says why the command is bad.
fn check_command(model: &Model, parse: Parse<'_>) -> Result<(EntryId, Option<String>), String> {
    let name = parse.name;
    let Some(id) = EntryId::named(name) else {
        return Err(format!("unknown command /{name}"));
    };
    let entry = id.entry();
    if entry.edits {
        return Err(format!(
            "/{name} is not a command, because it edits the text"
        ));
    }
    let shown = format!("/{name}");
    let arg = match (entry.arg, arity(id, parse.arg.map(str::to_owned), &shown)?) {
        (Some(arg), given) => match arg.kind {
            ArgKind::World => Some(world_arg(model, given, &shown)?),
        },
        (None, given) => given,
    };
    Ok((id, arg))
}

/// `given`, if it fits the argument of `entry`. `shown` names the entry in the text of a hint.
///
/// # Errors
///
/// The text of a hint if `entry` takes no argument and has one, or needs one and has none.
fn arity(entry: EntryId, given: Option<String>, shown: &str) -> Result<Option<String>, String> {
    match (entry.entry().arg, given) {
        (None, Some(_)) => Err(format!("{shown} takes no argument")),
        (Some(arg), None) if arg.required => Err(format!("{shown} needs a {}", arg.kind.word())),
        (_, given) => Ok(given),
    }
}

fn act(model: &mut Model, action: &Action) -> Vec<Effect> {
    match *action {
        Action::Quit => {
            let waiting: usize = model.sends.values().map(VecDeque::len).sum();
            if waiting == 0 || model.quit_arm.is_some() {
                return vec![Effect::Quit];
            }
            model.quit_arm = Some(QuitArm::Key);
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
            let effects = submit(model, *action == Action::SubmitLines);
            if let Some(state) = model.panes.get_mut(&model.focused_pane()) {
                state.to_bottom();
            }
            effects
        }
        Action::Newline => {
            model.pose().textarea().insert_newline();
            Vec::new()
        }
        Action::JoinLines => join_lines(model).into_iter().collect(),
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
        Action::ScrollPage(pages) => in_pane(model, |state| state.scroll_pages(pages)),
        Action::ScrollRows(rows) => in_pane(model, |state| state.scroll_rows(rows)),
        Action::FocusIndex(place) => {
            if let Some(index) = usize::from(place).checked_sub(1)
                && index < model.worlds.len()
            {
                model.set_focus(index);
            }
            Vec::new()
        }
        Action::ScrollTop => in_pane(model, PaneState::to_top),
        Action::ScrollBottom => in_pane(model, PaneState::to_bottom),
        Action::Connect(ref world) => control(model, world.clone(), true),
        Action::Disconnect(ref world) => control(model, world.clone(), false),
        Action::Focus(ref world) => {
            if model.first_list {
                model.hint("the worlds are not listed yet, so the focus does not move");
                return Vec::new();
            }
            match world_arg(model, Some(world.clone()), "/focus") {
                Ok(world) => {
                    if let Some(index) = model.find(&world) {
                        model.set_focus(index);
                    }
                }
                Err(reason) => model.hint(&reason),
            }
            Vec::new()
        }
        Action::Help => {
            help(model);
            Vec::new()
        }
        Action::HistoryPrev => {
            history_prev(model);
            Vec::new()
        }
        Action::HistoryNext => {
            history_next(model);
            Vec::new()
        }
    }
}

/// Up: the previous entry of the history of the world in focus. The first Up saves the text of
/// the editor as the draft of a new walk. With no older entry, nothing changes.
fn history_prev(model: &mut Model) {
    let textarea = model.pose().textarea();
    if textarea.cursor().0 > 0 {
        textarea.move_cursor(CursorMove::Up);
        return;
    }
    let Some(index) = model.focus else {
        return;
    };
    let draft = model.input();
    let Some(text) = model.worlds[index]
        .history
        .older(|| draft)
        .map(str::to_owned)
    else {
        return;
    };
    model.pose().set_text_at_start(&text);
}

/// Down: the next entry of the history of the world in focus, or after the newest entry the
/// draft, which ends the walk. With no walk, nothing changes.
fn history_next(model: &mut Model) {
    let textarea = model.pose().textarea();
    if textarea.cursor().0 + 1 < textarea.lines().len() {
        textarea.move_cursor(CursorMove::Down);
        return;
    }
    let Some(index) = model.focus else {
        return;
    };
    let Some(text) = model.worlds[index].history.newer() else {
        return;
    };
    model.pose().set_text(&text);
}

/// Adds `text` to the history of the world in focus. An open walk ends first, and its draft goes
/// into the history before `text`.
fn record(model: &mut Model, text: &str) {
    let Some(index) = model.focus else {
        return;
    };
    let history = &mut model.worlds[index].history;
    if let Some(draft) = history.end_walk() {
        history.push(&draft);
    }
    history.push(text);
}

/// Runs `change` on the state of the pane in focus.
fn in_pane(model: &mut Model, change: impl FnOnce(&mut PaneState)) -> Vec<Effect> {
    if let Some(state) = model.panes.get_mut(&model.focused_pane()) {
        change(state);
    }
    Vec::new()
}

/// The effect that connects `world`, or for `connect` false disconnects it. `None` is the world
/// in focus. With no such world, a hint and no effect.
fn control(model: &mut Model, world: Option<String>, connect: bool) -> Vec<Effect> {
    let name = if connect { "connect" } else { "disconnect" };
    match world_arg(model, world, name) {
        Ok(world) if connect => vec![Effect::Connect { world }],
        Ok(world) => vec![Effect::Disconnect { world }],
        Err(reason) => {
            model.hint(&reason);
            Vec::new()
        }
    }
}

/// Adds one hint for each entry of the registry: its name, its argument and its description.
fn help(model: &mut Model) {
    for entry in crate::registry::ENTRIES {
        let arg = match entry.arg {
            None => String::new(),
            Some(arg) => {
                let kind = arg.kind.word();
                if arg.required {
                    format!(" <{kind}>")
                } else {
                    format!(" [{kind}]")
                }
            }
        };
        model.hint(&format!("{}{arg}: {}", entry.name, entry.description));
    }
}

/// The world in focus, or a notice that no world has the focus.
fn world_to_send_to(model: &mut Model) -> Option<String> {
    let world = model.focused().map(str::to_owned);
    if world.is_none() {
        model.push(
            PaneId::NOTICES,
            &format!("-- {}", no_world("the line goes nowhere")),
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

/// Runs the text of the pose editor as a command, or sends it to the world in focus: as one
/// line, or for `lines` as one line for each line of the text. A text of one line that starts
/// with `//` goes without its first `/`. Each line goes to the world that has the focus now,
/// also if the focus moves before the last line goes.
fn submit(model: &mut Model, lines: bool) -> Vec<Effect> {
    let text = model.input();
    if let Some(parse) = command::parse(&text, text.len()) {
        return run_command(model, &text, parse);
    }
    model.quit_arm = None;
    let Some(world) = world_to_send_to(model) else {
        return Vec::new();
    };
    record(model, &text);
    model.set_input("");
    let text = match text.strip_prefix('/') {
        Some(rest) if rest.starts_with('/') && !text.contains('\n') => rest.to_owned(),
        _ => text,
    };
    let effect = if lines {
        enqueue(model, world, text.split('\n').map(str::to_owned))
    } else {
        enqueue(model, world, [text])
    };
    effect.into_iter().collect()
}

/// Replaces each line break of the pose editor with the line break of the world in focus. The
/// edit goes through the text area, so two undos bring the lines back: one for the joined text,
/// and one for the lines. If no list held the world in focus yet, the editor stays as it is, and
/// the client asks for the list of the worlds.
fn join_lines(model: &mut Model) -> Option<Effect> {
    let Some(world) = model.focused_world() else {
        model.hint(&no_world("no line break joins the lines"));
        return None;
    };
    let (listed, line_break) = (world.listed, world.line_break.clone());
    if !listed {
        let hint = match &model.list_failure {
            Some(reason) => format!(
                "the client does not know the line break of the world in focus: the last list of \
                 the worlds failed ({reason}), so it asks again"
            ),
            None => "the client does not know the line break of the world in focus yet, try again"
                .to_owned(),
        };
        model.hint(&hint);
        return list_worlds(model);
    }
    let Some(line_break) = line_break else {
        model.hint("the world in focus sets no line break, so the lines stay as they are");
        return None;
    };
    let text = model.input();
    if !text.contains('\n') {
        return None;
    }
    let textarea = model.pose().textarea();
    textarea.select_all();
    textarea.insert_str(text.replace('\n', &line_break));
    None
}

/// True if the latest store hash is not the store hash of the applied configuration.
fn config_stale(model: &Model) -> bool {
    model.latest_hash.is_some()
        && model.latest_hash.as_ref() != model.config.as_ref().map(|config| &config.store_hash)
}

/// The effects of a failed fetch of the configuration. A failure while the configuration is not
/// stale changes nothing. Otherwise: a notice at the first failure since the last apply, and a
/// fetch again after a wait by [`mu_client::backoff`] while no other wait runs.
fn config_failed(model: &mut Model, reason: &str) -> Vec<Effect> {
    if !config_stale(model) {
        return Vec::new();
    }
    if model.config_failures == 0 {
        model.notice(&format!("could not get the client configuration: {reason}"));
    }
    let failures = model.config_failures;
    model.config_failures = failures.saturating_add(1);
    if model.config_retry == ConfigRetry::Waiting {
        return Vec::new();
    }
    model.config_retry = ConfigRetry::Waiting;
    let after = mu_client::backoff()
        .nth(failures)
        .unwrap_or(Duration::from_secs(5));
    vec![Effect::RetryConfig { after }]
}

/// A list of the worlds, unless one is with the server already.
fn list_worlds(model: &mut Model) -> Option<Effect> {
    (model.listing == Listing::Idle).then(|| {
        model.listing = Listing::Asked;
        Effect::ListWorlds
    })
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
    model.set_focus(next);
}

/// The effects of an item of the event stream: a list of the worlds after a Reset, or after
/// the first event of a world that the model did not hold, so the model learns its line break.
fn on_client(model: &mut Model, item: ClientEvent) -> Vec<Effect> {
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
                model.replay_notices(pane);
            }
            return_waiting(model, "the server started again");
            if model.listing == Listing::Asked {
                model.listing = Listing::AskedBeforeReset;
            }
            list_worlds(model).into_iter().collect()
        }
        ClientEvent::Resumed => {
            model.lost = false;
            Vec::new()
        }
        ClientEvent::StoreHash(hash) => {
            model.latest_hash = Some(hash);
            if config_stale(model) {
                vec![Effect::FetchConfig]
            } else {
                model.config_failures = 0;
                Vec::new()
            }
        }
        ClientEvent::Lost => {
            model.lost = true;
            return_waiting(model, "the client lost the server");
            Vec::new()
        }
        ClientEvent::Event(event) => {
            model.lost = false;
            let count = model.worlds.len();
            let index = model.world(&event.world);
            let effects = if model.worlds.len() > count {
                list_worlds(model).into_iter().collect()
            } else {
                Vec::new()
            };
            let unseen = model.focus != Some(index);
            let world = &mut model.worlds[index];
            world.unseen |= unseen;
            let pane = model
                .panes
                .entry(world.pane)
                .or_insert_with(|| PaneState::with_caps(Arc::clone(&model.caps)));
            let prompt = event.kind.as_known() == Some(EventKind::EVENT_KIND_PROMPT);
            match event.source.as_known() {
                Some(Source::SOURCE_PLAYER) => {
                    let spans = match model.echo {
                        Echo::Show => Vec::new(),
                        Echo::Dim => vec![(0, Style::default().add_modifier(Modifier::DIM))],
                        Echo::Hide => return effects,
                    };
                    for row in echo_rows(&event.text, world.line_break.as_deref()) {
                        pane.push_styled(&row, &spans);
                    }
                }
                Some(Source::SOURCE_SYSTEM | Source::SOURCE_CREDENTIAL) => {
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
            effects
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
    let waits = model
        .sends
        .get(&world)
        .is_some_and(|queue| queue.front() == Some(&text));
    let lines: Vec<String> = if waits {
        model.sends.remove(&world).into_iter().flatten().collect()
    } else {
        vec![text]
    };
    let kept = put_back(model, &world, lines);
    let notice = format!("-- {}{}", clean(&reason), in_history(kept));
    let pane = model
        .find(&world)
        .map_or(PaneId::NOTICES, |index| model.worlds[index].pane);
    model.push(pane, &notice);
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

/// Gives `texts` back to `world`. The first goes alone into the editor of that world, with the
/// cursor at its start. The others, and then the text that the editor held, go into the history
/// of that world, so each Up gives the next one. During a walk, the held text is the draft of
/// the walk. A world out of focus gets its unseen mark. The history keeps the held text and at
/// most [`HISTORY_LIMIT`] less one of the others, the first ones, so no push of this call drops
/// the held text. Returns the count of entries that the history kept.
fn put_back(model: &mut Model, world: &str, texts: Vec<String>) -> usize {
    let Some(index) = model.find(world) else {
        return 0;
    };
    let mut texts = texts.into_iter();
    let Some(first) = texts.next() else {
        return 0;
    };
    let held = match model.worlds[index].history.end_walk() {
        Some(draft) => draft,
        None => model.pose_of(world).text(),
    };
    model.pose_of(world).set_text_at_start(&first);
    let rest: Vec<String> = texts.take(HISTORY_LIMIT - 1).collect();
    let entry = &mut model.worlds[index];
    let mut kept = usize::from(entry.history.push(&held));
    for text in rest.iter().rev() {
        kept += usize::from(entry.history.push(text));
    }
    if model.focus != Some(index) {
        entry.unseen = true;
    }
    kept
}

/// Gives the sends that wait behind the send with the server back to their worlds, in the
/// order of the worlds, with a hint that names `why`, the count of each world, and the count of
/// the entries that the histories kept. The send with
/// the server stays, so its reply still moves on or puts back only that send.
fn return_waiting(model: &mut Model, why: &str) {
    let mut returned = Vec::new();
    for world in &model.worlds {
        if let Some(queue) = model.sends.get_mut(&world.id) {
            let texts: Vec<String> = queue.drain(1..).collect();
            if !texts.is_empty() {
                returned.push((world.id.clone(), texts));
            }
        }
    }
    if returned.is_empty() {
        return;
    }
    let total = returned.iter().map(|(_, texts)| texts.len()).sum();
    let counts: Vec<String> = returned
        .iter()
        .map(|(world, texts)| format!("{} of {world}", line_count(texts.len())))
        .collect();
    let kept: usize = returned
        .into_iter()
        .map(|(world, texts)| put_back(model, &world, texts))
        .sum();
    model.hint(&format!(
        "{why}, so {} that waited went back to the editor: {}{}",
        line_count(total),
        counts.join(", "),
        in_history(kept)
    ));
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
