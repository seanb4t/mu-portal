//! The keymap: a table from a key with its modifiers, or a step of the mouse wheel, to an entry
//! of the [registry](crate::registry).

use mu_config::{Binding, Key, KeyName};
use termina::event::{KeyCode, KeyEvent, Modifiers};

use crate::registry::EntryId;

/// A key with its modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chord {
    /// The key.
    pub code: KeyCode,
    /// The modifiers held with it.
    pub modifiers: Modifiers,
}

impl Chord {
    /// `code` with no modifier.
    #[must_use]
    pub const fn plain(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: Modifiers::NONE,
        }
    }

    /// Ctrl and the character `c`.
    #[must_use]
    pub const fn ctrl(c: char) -> Self {
        Self::with(KeyCode::Char(c), Modifiers::CONTROL)
    }

    /// `code` with `modifiers`.
    #[must_use]
    pub const fn with(code: KeyCode, modifiers: Modifiers) -> Self {
        Self { code, modifiers }
    }

    /// The chord of a neutral key name. Tab with Shift is [`KeyCode::BackTab`] with Shift, as
    /// the terminal reports it.
    #[must_use]
    pub fn of_name(name: KeyName) -> Self {
        let held = name.modifiers;
        let mut modifiers = Modifiers::NONE;
        for (on, modifier) in [
            (held.ctrl, Modifiers::CONTROL),
            (held.alt, Modifiers::ALT),
            (held.shift, Modifiers::SHIFT),
            (held.super_key, Modifiers::SUPER),
        ] {
            if on {
                modifiers |= modifier;
            }
        }
        let code = match name.key {
            Key::Char(c) => KeyCode::Char(c),
            Key::Enter => KeyCode::Enter,
            Key::Tab if held.shift => KeyCode::BackTab,
            Key::Tab => KeyCode::Tab,
            Key::Backspace => KeyCode::Backspace,
            Key::Esc => KeyCode::Escape,
            Key::Up => KeyCode::Up,
            Key::Down => KeyCode::Down,
            Key::Left => KeyCode::Left,
            Key::Right => KeyCode::Right,
            Key::Home => KeyCode::Home,
            Key::End => KeyCode::End,
            Key::PageUp => KeyCode::PageUp,
            Key::PageDown => KeyCode::PageDown,
            Key::Insert => KeyCode::Insert,
            Key::Delete => KeyCode::Delete,
            Key::F(n) => KeyCode::Function(n),
        };
        Self { code, modifiers }
    }

    /// The chord that a binding of `key` has. An uppercase letter is its lowercase letter with
    /// Shift, and Tab with Shift is [`KeyCode::BackTab`].
    #[must_use]
    pub fn of_key(key: &KeyEvent) -> Self {
        let mut modifiers = key.modifiers;
        let code = match key.code {
            KeyCode::Char(c) if c.is_uppercase() => {
                let mut lower = c.to_lowercase();
                match (lower.next(), lower.next()) {
                    (Some(lower), None) => {
                        modifiers |= Modifiers::SHIFT;
                        KeyCode::Char(lower)
                    }
                    _ => key.code,
                }
            }
            KeyCode::Tab if modifiers.contains(Modifiers::SHIFT) => KeyCode::BackTab,
            code => code,
        };
        Self { code, modifiers }
    }
}

/// The entries that a key with modifiers never takes from the same key with no modifier. So
/// Shift+Up and Ctrl+Up reach the editor, and only plain Up and Down walk the history.
const NO_FALLBACK: [&str; 2] = ["history-prev", "history-next"];

/// One step of the mouse wheel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wheel {
    /// A step up, away from the user.
    Up,
    /// A step down, toward the user.
    Down,
}

/// The table from a [`Chord`] or a [`Wheel`] step to an [`EntryId`]. A key for which
/// [`Keymap::get`] gives no entry goes to the pose editor.
#[derive(Clone, Debug)]
pub struct Keymap {
    entries: Vec<(Chord, EntryId)>,
    wheel: Vec<(Wheel, EntryId)>,
}

impl Keymap {
    /// A keymap with no entry.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries: Vec::new(),
            wheel: Vec::new(),
        }
    }

    /// Maps the wheel step `step` to `entry`, in place of an earlier entry of the same step.
    pub fn bind_wheel(&mut self, step: Wheel, entry: EntryId) {
        match self.wheel.iter_mut().find(|(each, _)| *each == step) {
            Some(bound) => bound.1 = entry,
            None => self.wheel.push((step, entry)),
        }
    }

    /// The entry of the wheel step `step`.
    #[must_use]
    pub fn wheel(&self, step: Wheel) -> Option<EntryId> {
        self.wheel
            .iter()
            .find(|(each, _)| *each == step)
            .map(|(_, entry)| *entry)
    }

    /// The entry of each chord and each wheel step, the chords first.
    pub fn bindings(&self) -> impl Iterator<Item = EntryId> + '_ {
        self.entries
            .iter()
            .map(|(_, entry)| *entry)
            .chain(self.wheel.iter().map(|(_, entry)| *entry))
    }

    /// Maps `chord` to `entry`, in place of an earlier entry of the same chord.
    pub fn bind(&mut self, chord: Chord, entry: EntryId) {
        match self.entries.iter_mut().find(|(each, _)| *each == chord) {
            Some(bound) => bound.1 = entry,
            None => self.entries.push((chord, entry)),
        }
    }

    /// The default keymap with each binding in place of the default of its key. A key bound to
    /// [`Binding::None`] runs `ignore`, so it does not fall back to the key with no modifier.
    #[must_use]
    pub fn with_bindings(bindings: &[(KeyName, Binding)]) -> Self {
        let mut keymap = Self::default();
        for (name, binding) in bindings {
            let entry = match binding {
                Binding::None => EntryId::of("ignore"),
                Binding::Entry(entry) => EntryId::of(entry),
            };
            keymap.bind(Chord::of_name(*name), entry);
        }
        keymap
    }

    /// The entry of `key`. An uppercase letter is looked up as its lowercase letter with Shift,
    /// and Tab with Shift as [`KeyCode::BackTab`]. A key with modifiers and no entry of its own
    /// takes the entry of the same key with no modifier.
    #[must_use]
    pub fn get(&self, key: &KeyEvent) -> Option<EntryId> {
        self.find(Chord::of_key(key)).or_else(|| {
            self.find(Chord::plain(key.code))
                .filter(|entry| !NO_FALLBACK.contains(&entry.entry().name))
        })
    }

    fn find(&self, chord: Chord) -> Option<EntryId> {
        self.entries
            .iter()
            .find(|(each, _)| *each == chord)
            .map(|(_, entry)| *entry)
    }
}

impl Default for Keymap {
    /// Ctrl+C quits, Ctrl+N and Ctrl+P move the focus, and Alt+1 to Alt+9 give it to the world
    /// at that place. Alt+E gives the input focus to the next editor. Alt+Home and Alt+End scroll
    /// the pane to its ends. The page keys scroll one
    /// page, and each step of the mouse wheel scrolls three rows.
    ///
    /// Enter and Ctrl+M send the text of the pose editor as one command. Alt+Enter and
    /// Ctrl+Enter send each line as its own command. Shift+Enter and Ctrl+J insert a line
    /// break. Alt+R joins the lines with the line break of the world.
    ///
    /// Up on the first line and Down on the last line walk the input history.
    ///
    /// The editing keys follow readline: Ctrl+U deletes to the start of the line. Ctrl+R does
    /// nothing, so the redo of the text area does not take it. Ctrl+Z is undo, and Ctrl+Shift+Z
    /// is redo.
    fn default() -> Self {
        let mut keymap = Self::empty();
        keymap.bind(Chord::ctrl('c'), EntryId::of("quit"));
        keymap.bind(Chord::ctrl('n'), EntryId::of("focus-next"));
        keymap.bind(Chord::ctrl('p'), EntryId::of("focus-prev"));
        keymap.bind(
            Chord::with(KeyCode::Char('e'), Modifiers::ALT),
            EntryId::of("editor-next"),
        );
        keymap.bind(Chord::plain(KeyCode::Enter), EntryId::of("send"));
        keymap.bind(Chord::ctrl('m'), EntryId::of("send"));
        keymap.bind(
            Chord::with(KeyCode::Enter, Modifiers::ALT),
            EntryId::of("send-lines"),
        );
        keymap.bind(
            Chord::with(KeyCode::Enter, Modifiers::CONTROL),
            EntryId::of("send-lines"),
        );
        keymap.bind(
            Chord::with(KeyCode::Enter, Modifiers::SHIFT),
            EntryId::of("newline"),
        );
        keymap.bind(Chord::ctrl('j'), EntryId::of("newline"));
        keymap.bind(
            Chord::with(KeyCode::Char('r'), Modifiers::ALT),
            EntryId::of("join-lines"),
        );
        keymap.bind(Chord::ctrl('u'), EntryId::of("delete-to-line-start"));
        keymap.bind(Chord::ctrl('r'), EntryId::of("ignore"));
        keymap.bind(Chord::ctrl('z'), EntryId::of("undo"));
        keymap.bind(
            Chord::with(
                KeyCode::Char('z'),
                Modifiers::CONTROL.union(Modifiers::SHIFT),
            ),
            EntryId::of("redo"),
        );
        keymap.bind(Chord::plain(KeyCode::Up), EntryId::of("history-prev"));
        keymap.bind(Chord::plain(KeyCode::Down), EntryId::of("history-next"));
        keymap.bind(Chord::plain(KeyCode::PageUp), EntryId::of("scroll-page-up"));
        keymap.bind(
            Chord::plain(KeyCode::PageDown),
            EntryId::of("scroll-page-down"),
        );
        for digit in '1'..='9' {
            keymap.bind(
                Chord::with(KeyCode::Char(digit), Modifiers::ALT),
                EntryId::of(&format!("focus-{digit}")),
            );
        }
        keymap.bind(
            Chord::with(KeyCode::Home, Modifiers::ALT),
            EntryId::of("scroll-top"),
        );
        keymap.bind(
            Chord::with(KeyCode::End, Modifiers::ALT),
            EntryId::of("scroll-bottom"),
        );
        keymap.bind_wheel(Wheel::Up, EntryId::of("scroll-up"));
        keymap.bind_wheel(Wheel::Down, EntryId::of("scroll-down"));
        keymap
    }
}
