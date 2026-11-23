//! The keymap: a table from a key with its modifiers, or a step of the mouse wheel, to an entry
//! of the [registry](crate::registry).

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

    /// The chord of a key name: modifiers from `ctrl`, `alt`, `shift` and `super`, then one key,
    /// joined with `+`. A key is one character that is not an uppercase letter, a control
    /// character or `+`, the name `plus`, or the name of a special key. `shift+tab` is
    /// [`KeyCode::BackTab`] with Shift, and `shift` and a letter is the lowercase letter with
    /// Shift. `shift` with any other character is an error, because the terminal sends the
    /// shifted character.
    ///
    /// # Errors
    ///
    /// A [`KeyNameError`] that names the key name and the reason.
    pub fn parse(name: &str) -> Result<Self, KeyNameError> {
        let error = |reason: String| KeyNameError {
            name: name.to_owned(),
            reason,
        };
        let mut parts: Vec<&str> = name.split('+').collect();
        let key = parts.pop().unwrap_or_default();
        let mut modifiers = Modifiers::NONE;
        for part in parts {
            let (_, modifier) = MODIFIERS
                .iter()
                .find(|(each, _)| *each == part)
                .ok_or_else(|| {
                    error(if part.is_empty() {
                        "the key + is named plus".to_owned()
                    } else {
                        format!("{part:?} is no modifier: use ctrl, alt, shift or super")
                    })
                })?;
            if modifiers.contains(*modifier) {
                return Err(error(format!("{part} is there twice")));
            }
            modifiers |= *modifier;
        }
        let code = key_code(key).map_err(error)?;
        if modifiers.contains(Modifiers::SHIFT)
            && let KeyCode::Char(c) = code
            && !c.is_lowercase()
        {
            return Err(error(format!(
                "shift with {key} is not a key, because the terminal sends the shifted character"
            )));
        }
        let code = if code == KeyCode::Tab && modifiers.contains(Modifiers::SHIFT) {
            KeyCode::BackTab
        } else {
            code
        };
        Ok(Self { code, modifiers })
    }

    /// The key name of the chord, which [`Self::parse`] reads back for a chord that it made: the
    /// modifiers in the order
    /// `ctrl`, `alt`, `shift`, `super`, then the key. None for a key that has no name.
    #[must_use]
    pub fn name(self) -> Option<String> {
        let key = match self.code {
            KeyCode::BackTab => "tab".to_owned(),
            KeyCode::Char(c) => NAMED
                .iter()
                .find(|(_, code)| *code == KeyCode::Char(c))
                .map_or_else(|| c.to_string(), |(name, _)| (*name).to_owned()),
            KeyCode::Function(n) => format!("f{n}"),
            code => (*NAMED.iter().find(|(_, each)| *each == code)?.0).to_owned(),
        };
        let mut parts: Vec<String> = MODIFIERS
            .iter()
            .filter(|(_, modifier)| self.modifiers.contains(*modifier))
            .map(|(name, _)| (*name).to_owned())
            .collect();
        parts.push(key);
        Some(parts.join("+"))
    }

    /// The chord that a binding of `key` has. An uppercase letter is its lowercase letter with
    /// Shift, and Tab with Shift is [`KeyCode::BackTab`].
    fn of_key(key: &KeyEvent) -> Self {
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

/// A key name that [`Chord::parse`] refuses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyNameError {
    /// The key name.
    pub name: String,
    /// What is wrong with it.
    pub reason: String,
}

impl std::fmt::Display for KeyNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.reason)
    }
}

impl std::error::Error for KeyNameError {}

/// The modifiers of a key name, in the order that [`Chord::name`] writes them.
const MODIFIERS: [(&str, Modifiers); 4] = [
    ("ctrl", Modifiers::CONTROL),
    ("alt", Modifiers::ALT),
    ("shift", Modifiers::SHIFT),
    ("super", Modifiers::SUPER),
];

/// The names of the special keys, and of the characters ` ` and `+`.
const NAMED: [(&str, KeyCode); 16] = [
    ("enter", KeyCode::Enter),
    ("tab", KeyCode::Tab),
    ("backspace", KeyCode::Backspace),
    ("esc", KeyCode::Escape),
    ("space", KeyCode::Char(' ')),
    ("plus", KeyCode::Char('+')),
    ("up", KeyCode::Up),
    ("down", KeyCode::Down),
    ("left", KeyCode::Left),
    ("right", KeyCode::Right),
    ("home", KeyCode::Home),
    ("end", KeyCode::End),
    ("pageup", KeyCode::PageUp),
    ("pagedown", KeyCode::PageDown),
    ("insert", KeyCode::Insert),
    ("delete", KeyCode::Delete),
];

/// The key code of the last part of a key name.
fn key_code(key: &str) -> Result<KeyCode, String> {
    if let Some((_, code)) = NAMED.iter().find(|(name, _)| *name == key) {
        return Ok(*code);
    }
    if let Some(digits) = key.strip_prefix('f')
        && !digits.is_empty()
        && digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return match digits.parse::<u8>() {
            Ok(n @ 1..=12) if !digits.starts_with('0') => Ok(KeyCode::Function(n)),
            _ => Err(format!("{key} is no function key: they are f1 to f12")),
        };
    }
    let mut chars = key.chars();
    match (chars.next(), chars.next()) {
        (None, _) => Err("the name has no key".to_owned()),
        (Some(c), None) if c.is_control() => Err(format!(
            "the control character {c:?} is no key: name the key, such as tab or enter"
        )),
        (Some(c), None) if c.is_uppercase() => Err(format!(
            "{c} is uppercase: name the key with shift and the lowercase letter"
        )),
        (Some(c), None) => Ok(KeyCode::Char(c)),
        _ => Err(format!("{key:?} is no key")),
    }
}

/// The entries that Alt+1 to Alt+9 run, in the order of the digits.
const FOCUS_PLACES: [&str; 9] = [
    "focus-1", "focus-2", "focus-3", "focus-4", "focus-5", "focus-6", "focus-7", "focus-8",
    "focus-9",
];

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
    /// at that place. Alt+Home and Alt+End scroll the pane to its ends. The page keys scroll one
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
        for (digit, name) in ('1'..='9').zip(FOCUS_PLACES) {
            keymap.bind(
                Chord::with(KeyCode::Char(digit), Modifiers::ALT),
                EntryId::of(name),
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
