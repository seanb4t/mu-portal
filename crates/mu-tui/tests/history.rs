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

#[test]
fn a_push_during_a_walk_ends_it() {
    let mut history = pushed(2);
    assert_eq!(history.older(|| "draft".to_owned()), Some("1"));
    assert_eq!(history.walk_place(), Some(1));
    history.push("2");
    assert_eq!(history.walk_place(), None);
    assert_eq!(history.newer(), None);
}

#[test]
fn a_walk_at_the_limit_keeps_its_entry() {
    let mut history = pushed(HISTORY_LIMIT);
    assert_eq!(history.older(String::new), Some("999"));
    assert_eq!(history.older(String::new), Some("998"));
    // A push at the limit drops the oldest entry, so it ends the walk instead of moving it.
    history.push("new");
    assert_eq!(history.older(|| "draft".to_owned()), Some("new"));
    assert_eq!(history.newer().as_deref(), Some("draft"));
}
