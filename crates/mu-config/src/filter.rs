//! The filter of control characters in a text that goes to a terminal.

/// How [`filter`] writes each character: the characters that it keeps or writes as another
/// character, and the characters that it writes as U+FFFD beside the control characters.
#[derive(Clone, Copy, Debug)]
pub struct ControlMap {
    /// Each character and its output. It wins over the rest of the map.
    pub keep: &'static [(char, char)],
    /// The characters that are not control characters but are written as U+FFFD too.
    pub also: &'static [char],
}

impl ControlMap {
    /// The output of `c`.
    #[must_use]
    pub fn map(&self, c: char) -> char {
        if let Some((_, out)) = self.keep.iter().find(|(each, _)| *each == c) {
            *out
        } else if c.is_control() || self.also.contains(&c) {
            char::REPLACEMENT_CHARACTER
        } else {
            c
        }
    }
}

/// U+FFFD for every control character, the line break and the tab included. The text of a
/// notice of the store is one line.
pub const STORE: ControlMap = ControlMap {
    keep: &[],
    also: &[],
};

/// The line break and the tab as they are, and U+FFFD for each other control character. A
/// report on stderr keeps its lines.
pub const REPORT: ControlMap = ControlMap {
    keep: &[('\n', '\n'), ('\t', '\t')],
    also: &[],
};

/// `text` with each character written as `map` says.
#[must_use]
pub fn filter(text: &str, map: &ControlMap) -> String {
    text.chars().map(|c| map.map(c)).collect()
}
