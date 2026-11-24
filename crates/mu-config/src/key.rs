//! The grammar of a key name: modifiers from `ctrl`, `alt`, `shift` and `super`, then one key,
//! joined with `+`.

use std::fmt;

/// The modifiers held with a key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each modifier is one flag that a key name sets on its own"
)]
pub struct Modifiers {
    /// Ctrl.
    pub ctrl: bool,
    /// Alt, or Option on macOS.
    pub alt: bool,
    /// Shift.
    pub shift: bool,
    /// Super, or Command on macOS.
    pub super_key: bool,
}

impl Modifiers {
    /// The flag of the modifier named `name`.
    fn flag(&mut self, name: &str) -> Option<&mut bool> {
        match name {
            "ctrl" => Some(&mut self.ctrl),
            "alt" => Some(&mut self.alt),
            "shift" => Some(&mut self.shift),
            "super" => Some(&mut self.super_key),
            _ => None,
        }
    }

    /// The names of the modifiers that are held, in the order `ctrl`, `alt`, `shift`, `super`.
    fn names(self) -> impl Iterator<Item = &'static str> {
        [
            (self.ctrl, "ctrl"),
            (self.alt, "alt"),
            (self.shift, "shift"),
            (self.super_key, "super"),
        ]
        .into_iter()
        .filter(|(held, _)| *held)
        .map(|(_, name)| name)
    }
}

/// One key, with no modifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    /// A character that is not an uppercase letter and not a control character.
    Char(char),
    /// Enter.
    Enter,
    /// Tab. With Shift, the terminal reports Back Tab.
    Tab,
    /// Backspace.
    Backspace,
    /// Escape.
    Esc,
    /// Up.
    Up,
    /// Down.
    Down,
    /// Left.
    Left,
    /// Right.
    Right,
    /// Home.
    Home,
    /// End.
    End,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
    /// Insert.
    Insert,
    /// Delete.
    Delete,
    /// A function key from F1 to F12.
    F(u8),
}

/// The names of the special keys, and of the characters that have a name. The canonical name of
/// a key is the first name here that gives it.
const NAMED: [(&str, Key); 17] = [
    ("enter", Key::Enter),
    ("tab", Key::Tab),
    ("backspace", Key::Backspace),
    ("esc", Key::Esc),
    ("space", Key::Char(' ')),
    ("plus", Key::Char('+')),
    ("period", Key::Char('.')),
    ("up", Key::Up),
    ("down", Key::Down),
    ("left", Key::Left),
    ("right", Key::Right),
    ("home", Key::Home),
    ("end", Key::End),
    ("pageup", Key::PageUp),
    ("pagedown", Key::PageDown),
    ("insert", Key::Insert),
    ("delete", Key::Delete),
];

/// A key with its modifiers, as a key name writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyName {
    /// The modifiers.
    pub modifiers: Modifiers,
    /// The key.
    pub key: Key,
}

/// A key name that [`KeyName::parse`] refuses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyNameError {
    /// The key name.
    pub name: String,
    /// What is wrong with it.
    pub reason: String,
}

impl fmt::Display for KeyNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.reason)
    }
}

impl std::error::Error for KeyNameError {}

impl KeyName {
    /// Reads a key name. A key is one character that is not an uppercase letter, a control
    /// character or `+`, the name `plus`, the name `period`, or the name of a special key. `.`
    /// and `period` are one key. `shift` goes only with a lowercase character or a special key,
    /// because the terminal reports Shift and another character as the shifted character.
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
        let mut modifiers = Modifiers::default();
        for part in parts {
            let flag = modifiers.flag(part).ok_or_else(|| {
                error(if part.is_empty() {
                    "the key + is named plus".to_owned()
                } else {
                    format!("{part:?} is no modifier: use ctrl, alt, shift or super")
                })
            })?;
            if *flag {
                return Err(error(format!("{part} is there twice")));
            }
            *flag = true;
        }
        let key = key_of(key).map_err(error)?;
        if modifiers.shift
            && let Key::Char(c) = key
            && !c.is_lowercase()
        {
            return Err(error(format!(
                "shift with {} is not a key, because the terminal sends the shifted character",
                Self::key_name(key)
            )));
        }
        Ok(Self { modifiers, key })
    }

    /// The name of `key` alone.
    fn key_name(key: Key) -> String {
        match key {
            Key::F(n) => format!("f{n}"),
            Key::Char(c) => NAMED
                .iter()
                .find(|(_, each)| *each == key)
                .map_or_else(|| c.to_string(), |(name, _)| (*name).to_owned()),
            key => NAMED
                .iter()
                .find(|(_, each)| *each == key)
                .map_or_else(String::new, |(name, _)| (*name).to_owned()),
        }
    }
}

/// The canonical name: the modifiers in the order `ctrl`, `alt`, `shift`, `super`, then the key.
/// It holds no `.`, and [`KeyName::parse`] reads it back.
impl fmt::Display for KeyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for name in self.modifiers.names() {
            write!(f, "{name}+")?;
        }
        f.write_str(&Self::key_name(self.key))
    }
}

/// The key of the last part of a key name.
fn key_of(key: &str) -> Result<Key, String> {
    if let Some((_, named)) = NAMED.iter().find(|(name, _)| *name == key) {
        return Ok(*named);
    }
    if let Some(digits) = key.strip_prefix('f')
        && !digits.is_empty()
        && digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return match digits.parse::<u8>() {
            Ok(n @ 1..=12) if !digits.starts_with('0') => Ok(Key::F(n)),
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
        (Some(c), None) => Ok(Key::Char(c)),
        _ => Err(format!("{key:?} is no key")),
    }
}
