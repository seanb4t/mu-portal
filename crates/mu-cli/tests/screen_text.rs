//! The text that the line rules match is the text that the screen shows.

use proptest::prelude::*;

/// A character of game text: escape and control bytes, the parts of escape sequences, and
/// printable text.
fn game_char() -> impl Strategy<Value = char> {
    prop_oneof![
        Just('\u{1b}'),
        Just('\t'),
        Just('\r'),
        Just('\u{7}'),
        Just('\u{9b}'),
        Just('\u{fffd}'),
        Just('['),
        Just(']'),
        Just(';'),
        Just('m'),
        Just('\\'),
        prop::char::range('0', '9'),
        prop::char::range('\u{0}', '\u{1f}'),
        prop::char::range('a', 'z'),
        prop::char::range('\u{e9}', '\u{ff}'),
        Just(' '),
    ]
}

#[expect(
    clippy::default_trait_access,
    reason = "the boundary keeps ratatui, the type of the style, out of mu-cli"
)]
fn screen_text(line: &str) -> mu_tui::convert::Converted {
    mu_tui::convert::convert(line, &mut Default::default())
}

proptest! {
    #[test]
    fn the_rules_see_the_text_of_the_screen(chars in prop::collection::vec(game_char(), 0..64)) {
        let line: String = chars.into_iter().collect();
        let screen = screen_text(&line);
        let plain = mu_core::rules::plain(&line);
        prop_assert_eq!(plain.as_ref(), screen.text.as_str());
    }
}
