//! The input history of one world, and a walk through it with Up and Down.

use std::collections::VecDeque;

/// The most entries of one history. A push past it drops the oldest entry.
pub const HISTORY_LIMIT: usize = 1000;

/// The texts that the player sent to one world, the oldest first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct History {
    entries: VecDeque<String>,
}

impl History {
    /// Adds `text` as the newest entry, unless it is empty or equal to the newest entry. Past
    /// [`HISTORY_LIMIT`] entries, it drops the oldest. True if the history kept `text`.
    pub fn push(&mut self, text: &str) -> bool {
        if text.is_empty() || self.entries.back().is_some_and(|newest| newest == text) {
            return false;
        }
        if self.entries.len() == HISTORY_LIMIT {
            self.entries.pop_front();
        }
        self.entries.push_back(text.to_owned());
        true
    }

    /// The number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if the history has no entry.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entry at `place`, from 0 for the oldest.
    #[must_use]
    pub fn get(&self, place: usize) -> Option<&str> {
        self.entries.get(place).map(String::as_str)
    }

    /// The entries, the oldest first.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(String::as_str)
    }
}

/// A walk through a history: the place of the entry in the editor, and the text that the editor
/// held before the first Up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Walk {
    /// The place of the entry in the editor, from 0 for the oldest.
    pub place: usize,
    /// The text of the editor before the walk.
    pub draft: String,
}
