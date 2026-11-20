//! The keymap: a table from a key with its modifiers to an [`Action`].

use termina::event::{KeyCode, KeyEvent, Modifiers};

use crate::update::Action;

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
        Self {
            code: KeyCode::Char(c),
            modifiers: Modifiers::CONTROL,
        }
    }
}

/// The table from a [`Chord`] to an [`Action`]. A key that is in no entry goes to the input
/// line.
#[derive(Clone, Debug)]
pub struct Keymap {
    entries: Vec<(Chord, Action)>,
}

impl Keymap {
    /// A keymap with no entry.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Maps `chord` to `action`, in place of an earlier entry of the same chord.
    pub fn bind(&mut self, chord: Chord, action: Action) {
        match self.entries.iter_mut().find(|(each, _)| *each == chord) {
            Some(entry) => entry.1 = action,
            None => self.entries.push((chord, action)),
        }
    }

    /// The action of `key`. A key with modifiers and no entry of its own takes the entry of
    /// the same key with no modifier.
    #[must_use]
    pub fn get(&self, key: &KeyEvent) -> Option<Action> {
        let exact = Chord {
            code: key.code,
            modifiers: key.modifiers,
        };
        self.find(exact)
            .or_else(|| self.find(Chord::plain(key.code)))
    }

    fn find(&self, chord: Chord) -> Option<Action> {
        self.entries
            .iter()
            .find(|(each, _)| *each == chord)
            .map(|(_, action)| *action)
    }
}

impl Default for Keymap {
    /// Ctrl+C quits, Ctrl+N and Ctrl+P move the focus, and Enter sends the input line.
    fn default() -> Self {
        let mut keymap = Self::empty();
        keymap.bind(Chord::ctrl('c'), Action::Quit);
        keymap.bind(Chord::ctrl('n'), Action::FocusNext);
        keymap.bind(Chord::ctrl('p'), Action::FocusPrev);
        keymap.bind(Chord::plain(KeyCode::Enter), Action::Submit);
        keymap
    }
}
