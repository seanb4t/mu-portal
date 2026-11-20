//! The keymap holds the wheel steps beside the keys.

use mu_tui::{Action, Keymap, Wheel};

#[test]
fn default_wheel_scrolls_three_rows() {
    let keymap = Keymap::default();
    assert_eq!(
        [keymap.wheel(Wheel::Up), keymap.wheel(Wheel::Down)],
        [Some(Action::ScrollRows(-3)), Some(Action::ScrollRows(3))]
    );
}

#[test]
fn wheel_binding_replaces_the_default() {
    let mut keymap = Keymap::default();
    keymap.bind_wheel(Wheel::Up, Action::ScrollPage(-1));
    assert_eq!(keymap.wheel(Wheel::Up), Some(Action::ScrollPage(-1)));
}

#[test]
fn empty_keymap_has_no_wheel() {
    assert_eq!(Keymap::empty().wheel(Wheel::Down), None);
}
