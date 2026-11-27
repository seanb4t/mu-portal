//! The one walk of game text: the printable characters and the SGR sequences, with every other
//! escape sequence dropped. The client draws from it, and the rules match on it.

use vte::{Params, Parser, Perform};

use crate::ControlMap;

/// The char of the kitty image placeholder. Only the pane of the client draws it, for an image
/// line.
pub const PLACEHOLDER: char = '\u{10EEEE}';

/// The characters as the screen shows them: a space for a tab, and U+FFFD for each other control
/// character and for [`PLACEHOLDER`].
pub const SCREEN: ControlMap = ControlMap {
    keep: &[('\t', ' ')],
    also: &[PLACEHOLDER],
};

/// What [`walk`] gives.
pub trait Sink {
    /// One character of the text, after the map of the walk.
    fn char(&mut self, c: char);

    /// The parameter groups of one SGR sequence. A group holds the values that colons join.
    fn sgr(&mut self, groups: &[&[u16]]);
}

struct Walker<'a, S> {
    map: &'a ControlMap,
    sink: &'a mut S,
}

impl<S: Sink> Perform for Walker<'_, S> {
    fn print(&mut self, c: char) {
        self.sink.char(self.map.map(c));
    }

    fn execute(&mut self, byte: u8) {
        self.sink.char(self.map.map(char::from(byte)));
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: char) {
        if action == 'm' && intermediates.is_empty() && !ignore {
            let groups: Vec<&[u16]> = params.iter().collect();
            self.sink.sgr(&groups);
        }
    }
}

/// Walks `text` from a new parser in its ground state, and gives `sink` each character after
/// `map` and each SGR sequence.
pub fn walk(text: &str, map: &ControlMap, sink: &mut impl Sink) {
    let mut walker = Walker { map, sink };
    Parser::new().advance(&mut walker, text.as_bytes());
}
