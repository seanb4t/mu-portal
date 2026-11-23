//! The input history of one world, and a walk through it with Up and Down.

use std::collections::VecDeque;

/// The most entries of one history. A push past it drops the oldest entry.
pub const HISTORY_LIMIT: usize = 1000;

/// The texts that the player sent to one world, the oldest first, and the walk through them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct History {
    entries: VecDeque<String>,
    walk: Option<Walk>,
}

impl History {
    /// Adds `text` as the newest entry, unless it is empty or equal to the newest entry. Past
    /// [`HISTORY_LIMIT`] entries, it drops the oldest. An open walk ends first, and its draft is
    /// dropped: [`Self::end_walk`] gives the draft to a caller that keeps it. True if the history
    /// kept `text`.
    pub fn push(&mut self, text: &str) -> bool {
        self.walk = None;
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

    /// The entry before the one of the open walk, or the newest entry if no walk is open. The
    /// first step opens a walk with the text of `draft`. With no such entry, nothing changes.
    pub fn prev(&mut self, draft: impl FnOnce() -> String) -> Option<&str> {
        let place = match &self.walk {
            Some(walk) => walk.place.checked_sub(1)?,
            None => self.entries.len().checked_sub(1)?,
        };
        match &mut self.walk {
            Some(walk) => walk.place = place,
            None => {
                self.walk = Some(Walk {
                    place,
                    draft: draft(),
                });
            }
        }
        self.get(place)
    }

    /// The entry after the one of the open walk, or after the newest entry the draft, which ends
    /// the walk. `None` if no walk is open.
    pub fn next(&mut self) -> Option<String> {
        let walk = self.walk.as_mut()?;
        if walk.place + 1 < self.entries.len() {
            walk.place += 1;
            let place = walk.place;
            self.get(place).map(str::to_owned)
        } else {
            self.end_walk()
        }
    }

    /// Ends the open walk, and gives its draft.
    pub fn end_walk(&mut self) -> Option<String> {
        self.walk.take().map(|walk| walk.draft)
    }

    /// The place of the entry of the open walk, from 0 for the oldest.
    #[must_use]
    pub fn walk_place(&self) -> Option<usize> {
        self.walk.as_ref().map(|walk| walk.place)
    }
}

/// A walk through a history: the place of the entry in the editor, and the text that the editor
/// held before the first Up. Its place is always the place of an entry, because each change of
/// the entries ends the walk.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Walk {
    place: usize,
    draft: String,
}
