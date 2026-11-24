//! The grammar of key names and the check of a `[keys]` table.

use std::collections::BTreeMap;

use mu_config::{Binding, Key, KeyName, Modifiers, check_keys};
use proptest::prelude::*;

fn parse(name: &str) -> KeyName {
    KeyName::parse(name).unwrap_or_else(|error| panic!("{error}"))
}

fn with(key: Key, modifiers: Modifiers) -> KeyName {
    KeyName { modifiers, key }
}

const CTRL: Modifiers = Modifiers {
    ctrl: true,
    alt: false,
    shift: false,
    super_key: false,
};
const ALT: Modifiers = Modifiers {
    ctrl: false,
    alt: true,
    shift: false,
    super_key: false,
};

/// The reason of the check of a table with the one binding `key` = `value`.
fn reason(key: &str, value: &str) -> String {
    let keys = BTreeMap::from([(key.to_owned(), value.to_owned())]);
    let error = check_keys(&keys).expect_err(key);
    assert_eq!(error.key, key);
    error.reason
}

#[test]
fn ctrl_plus_and_alt_1_are_read() {
    assert_eq!(
        [parse("ctrl+plus"), parse("alt+1")],
        [with(Key::Char('+'), CTRL), with(Key::Char('1'), ALT)]
    );
}

#[test]
fn f1_and_f12_are_read() {
    assert_eq!(
        [parse("f1"), parse("f12")],
        [
            with(Key::F(1), Modifiers::default()),
            with(Key::F(12), Modifiers::default())
        ]
    );
}

#[test]
fn a_bad_modifier_names_the_good_ones() {
    let error = KeyName::parse("hyper+n").expect_err("hyper").to_string();
    assert!(error.ends_with("use ctrl, alt, shift or super"), "{error}");
    let error = KeyName::parse("ctrl++").expect_err("ctrl++").to_string();
    assert!(error.ends_with("the key + is named plus"), "{error}");
}

#[test]
fn bad_key_names_are_refused_with_the_name() {
    for name in [
        "ctrl+N",
        "ctrl+ctrl+n",
        "ctrl++",
        "+",
        "ctrl+",
        "",
        "ctrl",
        "shift+1",
        "shift+space",
        "shift+plus",
        "shift+period",
        "f0",
        "f01",
        "f13",
        "alt+\t",
        "alt+\r",
        "alt+\u{7f}",
        "hyper+n",
    ] {
        let error = KeyName::parse(name).expect_err(name);
        assert!(
            error.to_string().starts_with(&format!("{name}: ")),
            "{error}"
        );
    }
}

#[test]
fn a_dot_key_has_the_canonical_name_period() {
    assert_eq!(
        [parse("alt+.").to_string(), parse("alt+period").to_string()],
        ["alt+period", "alt+period"]
    );
}

#[test]
fn two_names_of_one_key_in_one_layer() {
    let keys = BTreeMap::from([
        ("ctrl+shift+z".to_owned(), "redo".to_owned()),
        ("shift+ctrl+z".to_owned(), "undo".to_owned()),
    ]);
    let error = check_keys(&keys).expect_err("two names");
    assert_eq!(
        (error.key.as_str(), error.reason.as_str()),
        (
            "shift+ctrl+z",
            "ctrl+shift+z and shift+ctrl+z name the same key"
        )
    );
}

#[test]
fn an_uppercase_key() {
    assert!(reason("ctrl+N", "quit").contains("uppercase"));
}

#[test]
fn a_control_character() {
    assert!(reason("alt+\u{7f}", "quit").contains("control character"));
}

#[test]
fn shift_with_a_digit() {
    assert!(reason("shift+1", "quit").contains("shift with 1"));
}

#[test]
fn an_unknown_entry() {
    assert_eq!(reason("alt+n", "fly"), "no entry is named fly");
}

#[test]
fn an_unknown_key_name() {
    assert!(reason("hyper+n", "quit").contains("is no modifier"));
}

#[test]
fn an_entry_that_needs_an_argument() {
    assert!(reason("alt+f", "focus").starts_with("focus needs an argument"));
}

#[test]
fn none_and_an_entry_are_bindings() {
    let keys = BTreeMap::from([
        ("alt+n".to_owned(), "focus-next".to_owned()),
        ("ctrl+n".to_owned(), "none".to_owned()),
    ]);
    assert_eq!(
        check_keys(&keys).expect("good"),
        [
            (parse("alt+n"), Binding::Entry("focus-next")),
            (parse("ctrl+n"), Binding::None)
        ]
    );
}

/// A key of a key name, as the name writes it, and whether Shift can go with it.
fn key_part() -> impl Strategy<Value = (String, bool)> {
    let special = [
        "enter",
        "tab",
        "backspace",
        "esc",
        "up",
        "down",
        "left",
        "right",
        "home",
        "end",
        "pageup",
        "pagedown",
        "insert",
        "delete",
    ];
    prop_oneof![
        proptest::sample::select(special.to_vec()).prop_map(|name| (name.to_owned(), true)),
        Just(("space".to_owned(), false)),
        Just(("plus".to_owned(), false)),
        Just(("period".to_owned(), false)),
        (1u8..=12).prop_map(|n| (format!("f{n}"), true)),
        proptest::char::range('a', 'z').prop_map(|c| (c.to_string(), true)),
        proptest::sample::select(
            "0123456789!@#$%^&*()-=[]{};:'\",.<>/?`~|\\_"
                .chars()
                .collect::<Vec<_>>()
        )
        .prop_map(|c| (c.to_string(), false)),
    ]
}

proptest! {
    #[test]
    fn key_names_read_back(
        (modifiers, turned) in proptest::sample::subsequence(vec!["ctrl", "alt", "shift", "super"], 0..=4)
            .prop_flat_map(|modifiers| (Just(modifiers.clone()), Just(modifiers).prop_shuffle())),
        (key, shift_ok) in key_part(),
    ) {
        prop_assume!(shift_ok || !modifiers.contains(&"shift"));
        let name = modifiers.iter().copied().chain([key.as_str()]).collect::<Vec<_>>().join("+");
        let parsed = KeyName::parse(&name).map_err(|error| TestCaseError::fail(error.to_string()))?;
        let written = parsed.to_string();
        prop_assert!(!written.contains('.'), "{}", written);
        prop_assert_eq!(KeyName::parse(&written).ok(), Some(parsed));
        let other = turned.iter().copied().chain([key.as_str()]).collect::<Vec<_>>().join("+");
        prop_assert_eq!(KeyName::parse(&other).ok(), Some(parsed));
    }
}
