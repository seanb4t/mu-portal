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

// Key names of the client layers, mapped to the chords of the terminal.

use mu_config::{ENTRY_NAMES, Key, KeyName, Modifiers as Held};
use mu_tui::{Chord, Model, Msg, update};
use proptest::prelude::*;
use termina::Event as TermEvent;
use termina::event::{KeyCode, KeyEvent, Modifiers};

fn parse(name: &str) -> Chord {
    Chord::of_name(KeyName::parse(name).unwrap_or_else(|error| panic!("{error}")))
}

#[test]
fn the_entry_table_matches_the_registry() {
    let registry: Vec<(&str, bool)> = ENTRIES
        .iter()
        .map(|entry| (entry.name, entry.arg.is_some_and(|arg| arg.required)))
        .collect();
    let table: Vec<(&str, bool)> = ENTRY_NAMES
        .iter()
        .map(|entry| (entry.name, entry.needs_arg))
        .collect();
    assert_eq!(table, registry);
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

/// A neutral key, and whether Shift can go with it.
fn neutral_key() -> impl Strategy<Value = (Key, bool)> {
    let special = vec![
        Key::Enter,
        Key::Tab,
        Key::Backspace,
        Key::Esc,
        Key::Up,
        Key::Down,
        Key::Left,
        Key::Right,
        Key::Home,
        Key::End,
        Key::PageUp,
        Key::PageDown,
        Key::Insert,
        Key::Delete,
    ];
    prop_oneof![
        proptest::sample::select(special).prop_map(|key| (key, true)),
        (1u8..=12).prop_map(|n| (Key::F(n), true)),
        proptest::char::range('a', 'z').prop_map(|c| (Key::Char(c), true)),
        proptest::sample::select(
            " +.0123456789!@#$%^&*()-=[]{};:'\",<>/?`~|\\_"
                .chars()
                .collect::<Vec<_>>()
        )
        .prop_map(|c| (Key::Char(c), false)),
    ]
}

/// The key code that the terminal reports for `key`.
fn code_of(key: Key, shift: bool) -> KeyCode {
    match key {
        Key::Char(c) => KeyCode::Char(c),
        Key::Enter => KeyCode::Enter,
        Key::Tab if shift => KeyCode::BackTab,
        Key::Tab => KeyCode::Tab,
        Key::Backspace => KeyCode::Backspace,
        Key::Esc => KeyCode::Escape,
        Key::Up => KeyCode::Up,
        Key::Down => KeyCode::Down,
        Key::Left => KeyCode::Left,
        Key::Right => KeyCode::Right,
        Key::Home => KeyCode::Home,
        Key::End => KeyCode::End,
        Key::PageUp => KeyCode::PageUp,
        Key::PageDown => KeyCode::PageDown,
        Key::Insert => KeyCode::Insert,
        Key::Delete => KeyCode::Delete,
        Key::F(n) => KeyCode::Function(n),
    }
}

proptest! {
    #[test]
    fn each_key_maps_to_the_chord_of_its_event(
        (key, shift_ok) in neutral_key(),
        ctrl in any::<bool>(),
        alt in any::<bool>(),
        shift in any::<bool>(),
        super_key in any::<bool>(),
    ) {
        let shift = shift && shift_ok;
        let name = KeyName { modifiers: Held { ctrl, alt, shift, super_key }, key };
        let mut modifiers = Modifiers::NONE;
        for (on, modifier) in [
            (ctrl, Modifiers::CONTROL),
            (alt, Modifiers::ALT),
            (shift, Modifiers::SHIFT),
            (super_key, Modifiers::SUPER),
        ] {
            if on {
                modifiers |= modifier;
            }
        }
        let event = KeyEvent::new(code_of(key, shift), modifiers);
        prop_assert_eq!(Chord::of_name(name), Chord::of_key(&event));
    }
}
