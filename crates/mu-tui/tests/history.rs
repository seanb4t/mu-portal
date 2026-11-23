//! The input history of one world.

use mu_tui::history::{HISTORY_LIMIT, History};

/// A history with the pushes `0` to `count - 1`.
fn pushed(count: usize) -> History {
    let mut history = History::default();
    for each in 0..count {
        assert!(history.push(&each.to_string()));
    }
    history
}

#[test]
fn history_drops_the_oldest_past_1000() {
    let full = pushed(HISTORY_LIMIT);
    assert_eq!((full.len(), full.get(0)), (1000, Some("0")));
    let past = pushed(HISTORY_LIMIT + 1);
    assert_eq!((past.len(), past.get(0)), (1000, Some("1")));
}
