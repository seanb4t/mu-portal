//! The text of an event on the screen.

use mu_proto::{Event, Source};

/// `text` with no control character, so remote text cannot send a sequence to the terminal.
pub(crate) fn clean(text: &str) -> String {
    text.chars().map(printable).collect()
}

/// `c` as it shows on the screen: a space for a tab, U+FFFD for each other control character,
/// and every other char as it is.
pub(crate) fn printable(c: char) -> char {
    match c {
        '\t' => ' ',
        c if c.is_control() => char::REPLACEMENT_CHARACTER,
        c => c,
    }
}

/// The text of an event on the screen: a game line as it is, a player line after `> `, and
/// a line of the system or a credential after `-- `.
pub(crate) fn line(event: &Event) -> String {
    match event.source.as_known() {
        Some(Source::SOURCE_PLAYER) => format!("> {}", clean(&event.text)),
        Some(Source::SOURCE_SYSTEM | Source::SOURCE_CREDENTIAL) => {
            format!("-- {}", clean(&event.text))
        }
        _ => clean(&event.text),
    }
}
