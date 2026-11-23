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
