//! The line store: its capacity and its trim.

use mu_tui::store::LineStore;
use proptest::prelude::*;

fn texts(store: &LineStore) -> Vec<String> {
    store
        .lines()
        .map(|(_, line)| line.text.to_owned())
        .collect()
}

#[test]
fn oldest_line_goes() {
    let mut store = LineStore::new(1000);
    for n in 1..=1000 {
        store.push(&n.to_string(), &[]);
    }
    store.push("1001", &[]);
    let want: Vec<String> = (2..=1001).map(|n: u32| n.to_string()).collect();
    assert_eq!(texts(&store), want);
}

#[test]
fn empty_pane_trims() {
    let mut store = LineStore::new(1000);
    store.trim(1);
    store.trim(0);
    assert_eq!(
        (store.first_id(), store.end_id(), texts(&store)),
        (
            store.end_id(),
            mu_tui::store::LineId(0),
            Vec::<String>::new()
        )
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn store_keeps_its_capacity_and_the_newest_line(
        capacity in 1..=10_000usize,
        pushes in 1..=20_000usize,
    ) {
        let mut store = LineStore::new(capacity);
        for n in 0..pushes {
            store.push(&n.to_string(), &[]);
        }
        prop_assert!(store.len() <= capacity);
        let newest = store.lines().next_back().map(|(_, line)| line.text.to_owned());
        prop_assert_eq!(newest, Some((pushes - 1).to_string()));
    }
}
