//! The text of an event on the screen.

use std::borrow::Cow;

use mu_proto::{Event, Source};
use ratatui::style::Style;

/// `text` with no control character, so remote text cannot send a sequence to the terminal.
pub(crate) fn clean(text: &str) -> String {
    text.chars().map(printable).collect()
}

/// The char of the kitty image placeholder. Only the pane draws it, for an image line.
pub(crate) const PLACEHOLDER: char = '\u{10EEEE}';

/// `c` as it shows on the screen: a space for a tab, U+FFFD for each other control character
/// and for the image placeholder, and every other char as it is.
pub(crate) fn printable(c: char) -> char {
    match c {
        '\t' => ' ',
        PLACEHOLDER => char::REPLACEMENT_CHARACTER,
        c if c.is_control() => char::REPLACEMENT_CHARACTER,
        c => c,
    }
}

/// `text` as [`clean`] gives it, and `spans` with each byte offset moved to the same char in
/// that text. An offset inside a char moves to the start of the next char.
pub(crate) fn clean_styled<'a>(
    text: &'a str,
    spans: &'a [(usize, Style)],
) -> (Cow<'a, str>, Cow<'a, [(usize, Style)]>) {
    if text.chars().all(|c| printable(c) == c) {
        return (Cow::Borrowed(text), Cow::Borrowed(spans));
    }
    let mut out = String::with_capacity(text.len());
    let mut moved = Vec::with_capacity(spans.len());
    let mut rest = spans.iter().peekable();
    for (at, c) in text.char_indices() {
        while let Some(&&(offset, style)) = rest.peek()
            && offset <= at
        {
            moved.push((out.len(), style));
            rest.next();
        }
        out.push(printable(c));
    }
    moved.extend(rest.map(|&(_, style)| (out.len(), style)));
    (Cow::Owned(out), Cow::Owned(moved))
}

/// The text of an event on the screen with its control characters replaced: a player line
/// after `> `, a line of the system or a credential after `-- `, and any other line alone.
pub(crate) fn line(event: &Event) -> String {
    match event.source.as_known() {
        Some(Source::SOURCE_PLAYER) => format!("> {}", clean(&event.text)),
        Some(Source::SOURCE_SYSTEM | Source::SOURCE_CREDENTIAL) => {
            format!("-- {}", clean(&event.text))
        }
        _ => clean(&event.text),
    }
}
