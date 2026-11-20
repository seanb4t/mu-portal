//! The line store: its capacity, its trim and its style table.

use mu_tui::store::{LineId, LineStore};
use proptest::prelude::*;
use ratatui::style::{Color, Style};

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
        (store.end_id(), LineId(0), Vec::<String>::new())
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
        let want: Vec<String> = (pushes.saturating_sub(capacity)..pushes)
            .map(|n| n.to_string())
            .collect();
        prop_assert_eq!(texts(&store), want);
    }
}

#[test]
fn lines_after_a_freed_chunk() {
    let ends: Vec<(String, String, usize)> = [4097, 8192, 8193]
        .into_iter()
        .map(|pushes| {
            let mut store = LineStore::new(4096);
            for n in 1..=pushes {
                store.push(&n.to_string(), &[]);
            }
            let kept = texts(&store);
            (kept[0].clone(), kept[kept.len() - 1].clone(), kept.len())
        })
        .collect();
    let want = |first: &str, last: &str| (first.to_owned(), last.to_owned(), 4096);
    assert_eq!(
        ends,
        [
            want("2", "4097"),
            want("4097", "8192"),
            want("4098", "8193")
        ]
    );
}

#[test]
fn style_past_the_table_limit_is_the_default() {
    let style = |n: u32| {
        let [_, _, high, low] = n.to_be_bytes();
        Style::new().fg(Color::Rgb(1, high, low))
    };
    let mut store = LineStore::new(70_000);
    // The default style takes the first entry, so 65535 new styles fill the table.
    let ids: Vec<_> = (0..=65_535)
        .map(|n| store.push("x", &[(0, style(n))]))
        .collect();
    let style_of = |n: usize| store.style(store.line(ids[n]).unwrap().spans[0].style);
    assert_eq!(
        (style_of(65_534), style_of(65_535)),
        (style(65_534), Style::default())
    );
}

fn spans_of(store: &LineStore, id: LineId) -> Vec<(usize, Style)> {
    store
        .line(id)
        .unwrap()
        .spans
        .iter()
        .map(|span| (span.start, store.style(span.style)))
        .collect()
}

#[test]
fn spans_out_of_order_are_stored_in_order() {
    let (red, blue) = (Style::new().fg(Color::Red), Style::new().fg(Color::Blue));
    let mut store = LineStore::new(10);
    let id = store.push("ab", &[(1, red), (0, blue)]);
    assert_eq!(spans_of(&store, id), [(0, blue), (1, red)]);
}

#[test]
fn spans_out_of_order_are_stored_in_order_after_a_control_character() {
    let (red, blue) = (Style::new().fg(Color::Red), Style::new().fg(Color::Blue));
    let mut store = LineStore::new(10);
    let id = store.push("a\u{1}b", &[(2, red), (0, blue)]);
    assert_eq!(spans_of(&store, id), [(0, blue), (4, red)]);
}
