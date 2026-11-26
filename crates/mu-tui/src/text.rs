//! The text of an event on the screen.

use std::borrow::Cow;

use mu_proto::{Event, Source};
use ratatui::style::Style;

/// `text` with no control character, so remote text cannot send a sequence to the terminal.
pub(crate) fn clean(text: &str) -> String {
    text.chars().map(printable).collect()
}

/// The text of a paste as the editor takes it. Each CR LF and each lone CR becomes an LF. Each
/// other control character goes, apart from the LF and the tab. So the text holds no ESC, and the editor
/// cannot write a sequence of the paste to the terminal.
pub(crate) fn clean_paste(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                chars.next_if_eq(&'\n');
                out.push('\n');
            }
            '\n' | '\t' => out.push(c),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}

pub(crate) use mu_config::ansi::PLACEHOLDER;

/// `c` as it shows on the screen: a space for a tab, U+FFFD for each other control character
/// and for the image placeholder, and every other char as it is.
pub(crate) fn printable(c: char) -> char {
    mu_config::ansi::SCREEN.map(c)
}

/// `text` as [`clean`] gives it, and `spans` in the order of their start. Spans with the same
/// start keep their order. If `text` has a char that [`clean`] replaces, each byte offset moves
/// to the same char in the new text, and an offset inside a char moves to the start of the next
/// char. Otherwise the offsets stay as they are.
pub(crate) fn clean_styled<'a>(
    text: &'a str,
    spans: &'a [(usize, Style)],
) -> (Cow<'a, str>, Cow<'a, [(usize, Style)]>) {
    let spans = if spans.is_sorted_by_key(|&(start, _)| start) {
        Cow::Borrowed(spans)
    } else {
        let mut sorted = spans.to_vec();
        sorted.sort_by_key(|&(start, _)| start);
        Cow::Owned(sorted)
    };
    if text.chars().all(|c| printable(c) == c) {
        return (Cow::Borrowed(text), spans);
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

/// The rows of the echo line of a player event, with the control characters of `text` replaced.
/// With a `line_break`, the text splits at each one: the first part goes after `> `, and each
/// other part after two spaces. With none, the text is one row after `> `.
#[must_use]
pub fn echo_rows(text: &str, line_break: Option<&str>) -> Vec<String> {
    let text = clean(text);
    let parts: Vec<&str> = match line_break {
        Some(line_break) if !line_break.is_empty() => text.split(line_break).collect(),
        _ => vec![text.as_str()],
    };
    parts
        .iter()
        .enumerate()
        .map(|(index, part)| {
            let prefix = if index == 0 { "> " } else { "  " };
            format!("{prefix}{part}")
        })
        .collect()
}

/// The text of an event on the screen with its control characters replaced: a line of the
/// system or a credential after `-- `, and any other line alone. [`echo_rows`] gives the rows of
/// a player line.
pub(crate) fn line(event: &Event) -> String {
    match event.source.as_known() {
        Some(Source::SOURCE_SYSTEM | Source::SOURCE_CREDENTIAL) => {
            format!("-- {}", clean(&event.text))
        }
        _ => clean(&event.text),
    }
}
