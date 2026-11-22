//! The keymap: a table from a key with its modifiers, or a step of the mouse wheel, to an
//! [`Action`].

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
        Self::with(KeyCode::Char(c), Modifiers::CONTROL)
    }

    /// `code` with `modifiers`.
    #[must_use]
    pub const fn with(code: KeyCode, modifiers: Modifiers) -> Self {
        Self { code, modifiers }
    }
}

/// One step of the mouse wheel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wheel {
    /// A step up, away from the user.
    Up,
    /// A step down, toward the user.
    Down,
}

/// The table from a [`Chord`] or a [`Wheel`] step to an [`Action`]. A key for which
/// [`Keymap::get`] gives no action goes to the pose editor.
#[derive(Clone, Debug)]
pub struct Keymap {
    entries: Vec<(Chord, Action)>,
    wheel: Vec<(Wheel, Action)>,
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

    /// Maps the wheel step `step` to `action`, in place of an earlier entry of the same step.
    pub fn bind_wheel(&mut self, step: Wheel, action: Action) {
        match self.wheel.iter_mut().find(|(each, _)| *each == step) {
            Some(entry) => entry.1 = action,
            None => self.wheel.push((step, action)),
        }
    }

    /// The action of the wheel step `step`.
    #[must_use]
    pub fn wheel(&self, step: Wheel) -> Option<Action> {
        self.wheel
            .iter()
            .find(|(each, _)| *each == step)
            .map(|(_, action)| *action)
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
    /// Ctrl+C quits, and Ctrl+N and Ctrl+P move the focus. The page keys scroll one page, and
    /// each step of the mouse wheel scrolls three rows.
    ///
    /// Enter and Ctrl+M send the text of the pose editor as one command. Alt+Enter and
    /// Ctrl+Enter send each line as its own command. Shift+Enter and Ctrl+J insert a line
    /// break. Alt+R joins the lines with the line break of the world.
    ///
    /// The editing keys follow readline: Ctrl+U deletes to the start of the line. Ctrl+R does
    /// nothing, so the redo of the text area does not take it. Ctrl+Z is undo, and Ctrl+Shift+Z
    /// is redo.
    fn default() -> Self {
        let mut keymap = Self::empty();
        keymap.bind(Chord::ctrl('c'), Action::Quit);
        keymap.bind(Chord::ctrl('n'), Action::FocusNext);
        keymap.bind(Chord::ctrl('p'), Action::FocusPrev);
        keymap.bind(Chord::plain(KeyCode::Enter), Action::Submit);
        keymap.bind(Chord::ctrl('m'), Action::Submit);
        keymap.bind(
            Chord::with(KeyCode::Enter, Modifiers::ALT),
            Action::SubmitLines,
        );
        keymap.bind(
            Chord::with(KeyCode::Enter, Modifiers::CONTROL),
            Action::SubmitLines,
        );
        keymap.bind(
            Chord::with(KeyCode::Enter, Modifiers::SHIFT),
            Action::Newline,
        );
        keymap.bind(Chord::ctrl('j'), Action::Newline);
        keymap.bind(
            Chord::with(KeyCode::Char('r'), Modifiers::ALT),
            Action::JoinLines,
        );
        keymap.bind(Chord::ctrl('u'), Action::DeleteToLineStart);
        keymap.bind(Chord::ctrl('r'), Action::Ignore);
        keymap.bind(Chord::ctrl('z'), Action::Undo);
        keymap.bind(
            Chord::with(
                KeyCode::Char('z'),
                Modifiers::CONTROL.union(Modifiers::SHIFT),
            ),
            Action::Redo,
        );
        keymap.bind(Chord::plain(KeyCode::PageUp), Action::ScrollPage(-1));
        keymap.bind(Chord::plain(KeyCode::PageDown), Action::ScrollPage(1));
        keymap.bind_wheel(Wheel::Up, Action::ScrollRows(-3));
        keymap.bind_wheel(Wheel::Down, Action::ScrollRows(3));
        keymap
    }
}
