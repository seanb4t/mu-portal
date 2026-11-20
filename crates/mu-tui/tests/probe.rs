//! The parser of the probe replies and what the replies give.

use mu_tui::probe::{GRAPHEMES, Outcome, Reply, parse};
use mu_tui::width::{Class, Rule};

#[test]
fn replies_parse() {
    let parsed = parse(b"\x1b[?2027;2$y\x1b[1;3R\x1b_Gi=31;OK\x1b\\\x1b[?62;22c");
    assert_eq!(
        parsed.replies,
        [
            Reply::Mode {
                mode: 2027,
                value: 2
            },
            Reply::Cursor { row: 1, col: 3 },
            Reply::Graphics(true),
            Reply::Da1,
        ]
    );
}

#[test]
fn graphics_error_is_no_graphics() {
    let parsed = parse(b"\x1b_Gi=31;ENOTSUPPORTED:no\x1b\\");
    assert_eq!(parsed.replies, [Reply::Graphics(false)]);
}

#[test]
fn graphics_reply_of_another_id_is_other_input() {
    let input = b"\x1b_Gi=7;OK\x1b\\";
    let parsed = parse(input);
    assert_eq!(parsed.other, input);
}

#[test]
fn keys_between_replies_are_other_input() {
    let parsed = parse(b"a\x1b[1;2Rb\x1b[Ac");
    assert_eq!(parsed.other, b"ab\x1b[Ac");
}

#[test]
fn incomplete_reply_stays() {
    let input = b"x\x1b[1;";
    let parsed = parse(input);
    assert_eq!(parsed.used, 1);
}

#[test]
fn incomplete_graphics_reply_stays() {
    let parsed = parse(b"\x1b_Gi=31;O");
    assert_eq!(parsed.used, 0);
}

/// The replies of a terminal that gives every probe grapheme `width` cells, with no DA1 if
/// `complete` is false.
fn replies(width: impl Fn(&str) -> u16, complete: bool) -> Vec<Reply> {
    let mut replies: Vec<Reply> = GRAPHEMES
        .iter()
        .map(|g| Reply::Cursor {
            row: 1,
            col: width(g) + 1,
        })
        .collect();
    if complete {
        replies.push(Reply::Da1);
    }
    replies
}

fn eight_wide_zwj(g: &str) -> u16 {
    if Class::of(g) == Class::Zwj { 8 } else { 2 }
}

#[test]
fn widths_come_from_the_cursor_positions() {
    let outcome = Outcome::learn(&replies(eight_wide_zwj, true));
    assert_eq!(outcome.profile.width("👩\u{200D}👩\u{200D}👦"), 8);
}

#[test]
fn replies_with_no_da1_keep_the_default_profile() {
    let outcome = Outcome::learn(&replies(eight_wide_zwj, false));
    assert_eq!(outcome.profile.rule(Class::Zwj), Rule::Unicode);
}

#[test]
fn mode_reset_means_the_probe_turned_it_on() {
    let outcome = Outcome::learn(&[
        Reply::Mode {
            mode: 2027,
            value: 2,
        },
        Reply::Da1,
    ]);
    assert!(outcome.resets_grapheme_mode());
}

#[test]
fn mode_set_means_the_probe_left_it() {
    let outcome = Outcome::learn(&[
        Reply::Mode {
            mode: 2027,
            value: 1,
        },
        Reply::Da1,
    ]);
    assert!(!outcome.resets_grapheme_mode());
}

#[test]
fn no_mode_reply_means_the_mode_goes_off() {
    assert!(Outcome::learn(&[]).resets_grapheme_mode());
}

#[test]
fn permanently_set_mode_stays() {
    let outcome = Outcome::learn(&[Reply::Mode {
        mode: 2027,
        value: 3,
    }]);
    assert!(!outcome.resets_grapheme_mode());
}
