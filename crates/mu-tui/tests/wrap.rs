//! The wrap of one line to a width.

use mu_tui::width::Profile;
use mu_tui::wrap::{Row, wrap};
use proptest::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

fn rows(text: &str, width: u16) -> Vec<String> {
    let mut out = Vec::new();
    wrap(text, width, &Profile::default(), &mut out);
    out.iter()
        .map(|row| text[row.start..row.end].to_owned())
        .collect()
}

/// The rows as the pane shows them, with no space at the end.
fn shown(text: &str, width: u16) -> Vec<String> {
    rows(text, width)
        .iter()
        .map(|row| row.trim_end_matches(' ').to_owned())
        .collect()
}

#[test]
fn long_line_wraps_at_a_space() {
    assert_eq!(
        shown("the quick brown fox jumps over the lazy dog", 20),
        ["the quick brown fox", "jumps over the lazy", "dog"]
    );
}

#[test]
fn word_wider_than_the_pane() {
    assert_eq!(shown("abcdefghijklmnop", 10), ["abcdefghij", "klmnop"]);
}

#[test]
fn empty_line_takes_a_row() {
    let counts: Vec<usize> = ["a", "", "b"]
        .iter()
        .map(|line| rows(line, 20).len())
        .collect();
    assert_eq!(counts, [1, 1, 1]);
}

#[test]
fn learned_width_wider_than_the_pane() {
    let text = "ab👨\u{200D}👩\u{200D}👧c";
    let mut out = Vec::new();
    wrap(text, 5, &wide_zwj(), &mut out);
    let rows: Vec<&str> = out.iter().map(|row| &text[row.start..row.end]).collect();
    assert_eq!(rows, ["ab", "👨\u{200D}👩\u{200D}👧", "c"]);
}

#[test]
fn grapheme_wider_than_the_pane() {
    assert_eq!(rows("x日y", 1), ["x", "日", "y"]);
}

/// The cells that a row takes on the screen with the widths of `profile`: its text with no
/// space at the end, and one cell for a lone grapheme wider than `width`.
fn cells(profile: &Profile, row: &str, width: u16) -> usize {
    let text = row.trim_end_matches(' ');
    let mut graphemes = text.graphemes(true);
    match (graphemes.next(), graphemes.next()) {
        (Some(only), None) if profile.width(only) > usize::from(width) => 1,
        _ => text.graphemes(true).map(|g| profile.width(g)).sum(),
    }
}

/// A profile that gives a ZWJ sequence 8 cells.
fn wide_zwj() -> Profile {
    Profile::learn(&[("👨\u{200D}👩\u{200D}👧", 8)])
}

fn line() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            Just(" ".to_owned()),
            Just("日".to_owned()),
            Just("e\u{301}".to_owned()),
            Just("🐉".to_owned()),
            Just("👩\u{200d}👩\u{200d}👧".to_owned()),
            "[a-z]{1,12}",
        ],
        0..40,
    )
    .prop_map(|parts| parts.concat())
}

proptest! {
    #[test]
    fn wrap_keeps_every_byte_in_rows_that_fit(
        text in line(),
        width in prop_oneof![Just(0u16), Just(1u16), Just(5u16), 0..=300u16],
        profile in prop_oneof![Just(Profile::default()), Just(wide_zwj())],
    ) {
        let mut out: Vec<Row> = Vec::new();
        wrap(&text, width, &profile, &mut out);
        if width == 0 {
            prop_assert!(out.is_empty());
            return Ok(());
        }
        prop_assert!(!out.is_empty());
        let joined: String = out.iter().map(|row| &text[row.start..row.end]).collect();
        prop_assert_eq!(joined, text.clone());
        for row in &out {
            prop_assert!(row.start <= row.end);
            prop_assert!(cells(&profile, &text[row.start..row.end], width) <= usize::from(width));
        }
    }
}
