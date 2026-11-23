//! The keymap binds keys and wheel steps to the entries of the registry.

use mu_tui::{Action, ENTRIES, EntryId, Keymap, Wheel};

fn named(name: &str) -> EntryId {
    EntryId::named(name).expect("the entry exists")
}

#[test]
fn default_wheel_scrolls_three_rows() {
    let keymap = Keymap::default();
    assert_eq!(
        [keymap.wheel(Wheel::Up), keymap.wheel(Wheel::Down)],
        [Some(named("scroll-up")), Some(named("scroll-down"))]
    );
}

#[test]
fn wheel_binding_replaces_the_default() {
    let mut keymap = Keymap::default();
    keymap.bind_wheel(Wheel::Up, named("scroll-page-up"));
    assert_eq!(keymap.wheel(Wheel::Up), Some(named("scroll-page-up")));
}

#[test]
fn empty_keymap_has_no_wheel() {
    assert_eq!(Keymap::empty().wheel(Wheel::Down), None);
}

/// True if `name` is lowercase words of letters and digits joined with `-`, the first word of
/// letters only.
fn is_kebab(name: &str) -> bool {
    let mut words = name.split('-');
    let first = words.next().unwrap_or_default();
    !first.is_empty()
        && first.chars().all(|c| c.is_ascii_lowercase())
        && words.all(|word| {
            !word.is_empty()
                && word
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

#[test]
fn entry_names_are_unique_and_kebab() {
    let mut names: Vec<&str> = ENTRIES.iter().map(|entry| entry.name).collect();
    let bad: Vec<&str> = names
        .iter()
        .copied()
        .filter(|name| !is_kebab(name))
        .collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!((bad, names.len()), (Vec::<&str>::new(), ENTRIES.len()));
}

#[test]
fn default_keymap_binds_entries_with_no_required_argument() {
    let required: Vec<&str> = Keymap::default()
        .bindings()
        .map(EntryId::entry)
        .filter(|entry| entry.arg.is_some_and(|arg| arg.required))
        .map(|entry| entry.name)
        .collect();
    assert!(required.is_empty(), "{required:?}");
}

#[test]
fn focus_rows_match_their_index() {
    let rows: Vec<(String, Action)> = (1..=9u8)
        .map(|n| {
            let entry = named(&format!("focus-{n}")).entry();
            (entry.name.to_owned(), (entry.build)(None))
        })
        .collect();
    let expected: Vec<(String, Action)> = (1..=9u8)
        .map(|n| (format!("focus-{n}"), Action::FocusIndex(n)))
        .collect();
    assert_eq!(rows, expected);
}

#[test]
fn edits_flag_matches_the_editing_rows() {
    let editing: Vec<&str> = ENTRIES
        .iter()
        .filter(|entry| entry.edits)
        .map(|entry| entry.name)
        .collect();
    assert_eq!(
        editing,
        [
            "send",
            "send-lines",
            "newline",
            "join-lines",
            "delete-to-line-start",
            "undo",
            "redo",
            "ignore",
            "history-prev",
            "history-next"
        ]
    );
}

#[test]
fn default_keymap_leaves_tab_free() {
    use termina::event::{KeyCode, KeyEvent, Modifiers};
    let keymap = Keymap::default();
    let keys = [
        KeyEvent::new(KeyCode::Tab, Modifiers::NONE),
        KeyEvent::new(KeyCode::BackTab, Modifiers::SHIFT),
        KeyEvent::new(KeyCode::Char(' '), Modifiers::CONTROL),
    ];
    assert_eq!(keys.map(|key| keymap.get(&key)), [None, None, None]);
}

// Key names of `client.toml`.

use mu_tui::{Chord, Model, Msg, update};
use proptest::prelude::*;
use termina::Event as TermEvent;
use termina::event::{KeyCode, KeyEvent, Modifiers};

fn parse(name: &str) -> Chord {
    Chord::parse(name).unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn ctrl_plus_and_alt_1_are_read() {
    assert_eq!(
        [parse("ctrl+plus"), parse("alt+1")],
        [
            Chord::with(KeyCode::Char('+'), Modifiers::CONTROL),
            Chord::with(KeyCode::Char('1'), Modifiers::ALT),
        ]
    );
}

#[test]
fn f1_and_f12_are_read() {
    assert_eq!(
        [parse("f1"), parse("f12")],
        [
            Chord::plain(KeyCode::Function(1)),
            Chord::plain(KeyCode::Function(12))
        ]
    );
}

#[test]
fn a_bad_modifier_names_the_good_ones() {
    let error = Chord::parse("hyper+n").expect_err("hyper").to_string();
    assert!(error.ends_with("use ctrl, alt, shift or super"), "{error}");
    let error = Chord::parse("ctrl++").expect_err("ctrl++").to_string();
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
        "f0",
        "f01",
        "f13",
        "alt+\t",
        "alt+\r",
        "alt+\u{7f}",
        "hyper+n",
    ] {
        let error = Chord::parse(name).expect_err(name);
        assert!(
            error.to_string().starts_with(&format!("{name}: ")),
            "{error}"
        );
    }
}

#[test]
fn shift_tab_matches_back_tab() {
    let mut keymap = Keymap::empty();
    keymap.bind(parse("shift+tab"), named("focus-prev"));
    let keys = [
        KeyEvent::new(KeyCode::BackTab, Modifiers::SHIFT),
        KeyEvent::new(KeyCode::Tab, Modifiers::SHIFT),
    ];
    assert_eq!(
        keys.map(|key| keymap.get(&key)),
        [Some(named("focus-prev")); 2]
    );
}

#[test]
fn shift_and_a_letter_match_the_key() {
    let mut keymap = Keymap::empty();
    keymap.bind(parse("shift+z"), named("quit"));
    let keys = [
        KeyEvent::new(KeyCode::Char('z'), Modifiers::SHIFT),
        KeyEvent::new(KeyCode::Char('Z'), Modifiers::SHIFT),
        KeyEvent::new(KeyCode::Char('Z'), Modifiers::NONE),
    ];
    assert_eq!(keys.map(|key| keymap.get(&key)), [Some(named("quit")); 3]);
}

#[test]
fn an_unbound_shift_letter_types_it() {
    let mut model = Model::new(["alpha/aria".to_owned()]);
    for modifiers in [Modifiers::SHIFT, Modifiers::NONE] {
        let key = KeyEvent::new(KeyCode::Char('Z'), modifiers);
        assert_eq!(update(&mut model, Msg::Terminal(TermEvent::Key(key))), []);
    }
    assert_eq!(model.input(), "ZZ");
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
        let chord = Chord::parse(&name).map_err(|error| TestCaseError::fail(error.to_string()))?;
        let written = chord.name().expect("a parsed chord has a name");
        prop_assert_eq!(Chord::parse(&written).ok(), Some(chord));
        let other = turned.iter().copied().chain([key.as_str()]).collect::<Vec<_>>().join("+");
        prop_assert_eq!(Chord::parse(&other).ok(), Some(chord));
    }
}
