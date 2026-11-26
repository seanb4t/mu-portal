//! The filter of control characters.

use mu_config::{REPORT, STORE, filter};
use proptest::prelude::*;

#[test]
fn the_report_map_keeps_a_line_break_and_a_tab() {
    assert_eq!(filter("a\n\tb\u{1b}", &REPORT), "a\n\tb\u{fffd}");
}

#[test]
fn the_store_map_replaces_a_line_break() {
    assert_eq!(filter("lap\n.toml", &STORE), "lap\u{fffd}.toml");
}

proptest! {
    #[test]
    fn the_filter_leaves_no_control_character(text in any::<String>()) {
        let out = filter(&text, &STORE);
        prop_assert!(!out.chars().any(char::is_control), "{:?}", out);
        prop_assert_eq!(out.chars().count(), text.chars().count());
    }
}

#[test]
fn a_map_keeps_a_text_it_writes_as_it_is() {
    assert_eq!(
        (
            REPORT.keeps("one\ttwo\n"),
            REPORT.keeps("one\u{7}"),
            STORE.keeps("one\ttwo"),
        ),
        (true, false, false)
    );
}
