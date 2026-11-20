//! The scenarios of the output-pane capability on the pane widget, drawn into a `Buffer`.

use std::num::NonZeroU16;
use std::sync::Arc;

use mu_tui::store::LineId;
use mu_tui::width::Profile;
use mu_tui::{OutputPane, PaneState, Position};
use ratatui::buffer::{Buffer, CellDiffOption};
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;
use unicode_width::UnicodeWidthStr;

/// The symbols of the scroll bar: the thumb and the track.
const BAR: [&str; 2] = ["█", "║"];

fn render(pane: &mut PaneState, width: u16, height: u16) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
    OutputPane.render(buf.area, &mut buf, pane);
    buf
}

/// The rows of `buf` with no trailing spaces, and with no scroll bar column if it shows one. A
/// cell after a wide grapheme belongs to that grapheme.
fn rows(buf: &Buffer) -> Vec<String> {
    let width = if has_bar(buf) {
        buf.area.width - 1
    } else {
        buf.area.width
    };
    (0..buf.area.height)
        .map(|y| {
            let mut row = String::new();
            let mut x = 0;
            while x < width {
                let symbol = buf[(x, y)].symbol();
                row.push_str(symbol);
                x += u16::try_from(symbol.width().max(1)).unwrap();
            }
            row.trim_end().to_owned()
        })
        .collect()
}

fn last_column(buf: &Buffer) -> Vec<&str> {
    let x = buf.area.width - 1;
    (0..buf.area.height).map(|y| buf[(x, y)].symbol()).collect()
}

fn has_bar(buf: &Buffer) -> bool {
    last_column(buf).iter().any(|symbol| BAR.contains(symbol))
}

fn pane_with(lines: impl IntoIterator<Item = String>) -> PaneState {
    let mut pane = PaneState::default();
    for line in lines {
        pane.push(&line);
    }
    pane
}

fn numbers(range: std::ops::RangeInclusive<u32>) -> impl Iterator<Item = String> {
    range.map(|n| n.to_string())
}

fn id_of(pane: &PaneState, text: &str) -> LineId {
    pane.store()
        .lines()
        .find(|(_, line)| line.text == text)
        .map(|(id, _)| id)
        .unwrap()
}

#[test]
fn follows_at_the_bottom() {
    let mut pane = pane_with(numbers(1..=3));
    assert_eq!(rows(&render(&mut pane, 20, 3)), ["1", "2", "3"]);
    pane.push("4");
    assert_eq!(rows(&render(&mut pane, 20, 3)), ["2", "3", "4"]);
}

#[test]
fn holds_when_scrolled_back() {
    let mut pane = pane_with(numbers(1..=10));
    render(&mut pane, 20, 3);
    pane.scroll_rows(-4);
    assert_eq!(rows(&render(&mut pane, 20, 3)), ["4", "5", "6"]);
    pane.push("11");
    assert_eq!(rows(&render(&mut pane, 20, 3)), ["4", "5", "6"]);
}

#[test]
fn holds_when_scrolled_back_one_row() {
    let mut pane = pane_with(numbers(1..=10));
    render(&mut pane, 20, 3);
    pane.scroll_rows(-1);
    assert_eq!(rows(&render(&mut pane, 20, 3)), ["7", "8", "9"]);
    pane.push("11");
    assert_eq!(rows(&render(&mut pane, 20, 3)), ["7", "8", "9"]);
}

#[test]
fn follows_again_at_the_bottom() {
    let mut pane = pane_with(numbers(1..=10));
    render(&mut pane, 20, 3);
    pane.scroll_rows(-3);
    render(&mut pane, 20, 3);
    pane.scroll_rows(3);
    assert_eq!(rows(&render(&mut pane, 20, 3)), ["8", "9", "10"]);
    pane.push("11");
    assert_eq!(rows(&render(&mut pane, 20, 3)), ["9", "10", "11"]);
}

#[test]
fn narrower_pane_keeps_the_top_line() {
    let long = "x".repeat(25);
    let mut pane = pane_with((1..=100).map(|n| format!("line {n} {long}")));
    render(&mut pane, 40, 5);
    pane.scroll_rows(-46);
    assert_eq!(
        rows(&render(&mut pane, 40, 5))[0],
        format!("line 50 {long}")
    );
    assert_eq!(rows(&render(&mut pane, 20, 5))[0], "line 50");
}

#[test]
fn resize_at_the_bottom() {
    let mut pane = pane_with(["the quick brown fox jumps over the lazy dog".to_owned()]);
    render(&mut pane, 40, 3);
    let shown = rows(&render(&mut pane, 20, 3));
    assert_eq!(shown.last().unwrap(), "dog");
}

#[test]
fn anchor_on_a_dropped_line() {
    let mut pane = PaneState::new(1000);
    for line in numbers(1..=1000) {
        pane.push(&line);
    }
    render(&mut pane, 20, 3);
    pane.scroll_rows(-2000);
    assert_eq!(rows(&render(&mut pane, 20, 3))[0], "1");
    for line in numbers(1001..=1005) {
        pane.push(&line);
    }
    assert_eq!(rows(&render(&mut pane, 20, 3))[0], "6");
}

fn selected_one_and_two() -> PaneState {
    let mut pane = PaneState::new(1000);
    for line in numbers(1..=1000) {
        pane.push(&line);
    }
    let one = id_of(&pane, "1");
    let two = id_of(&pane, "2");
    pane.select(
        Position { line: one, byte: 0 },
        Position { line: two, byte: 1 },
    );
    pane
}

#[test]
fn selection_on_a_dropped_line() {
    let mut pane = selected_one_and_two();
    pane.push("1001");
    assert_eq!(pane.copy_text().as_deref(), Some("2"));
}

#[test]
fn selection_cleared_when_both_ends_drop() {
    let mut pane = selected_one_and_two();
    pane.push("1001");
    pane.push("1002");
    assert_eq!(pane.selection(), None);
}

#[test]
fn scroll_bar_halfway() {
    let mut pane = pane_with(numbers(1..=100));
    render(&mut pane, 10, 10);
    pane.scroll_rows(-41);
    let buf = render(&mut pane, 10, 10);
    assert_eq!(rows(&buf)[0], "50");
    let thumb: Vec<usize> = last_column(&buf)
        .iter()
        .enumerate()
        .filter(|(_, symbol)| **symbol == "█")
        .map(|(y, _)| y)
        .collect();
    assert!(
        !thumb.is_empty() && thumb.iter().all(|y| (4..=5).contains(y)),
        "{thumb:?}"
    );
}

#[test]
fn no_scroll_bar_when_rows_fit() {
    let mut pane = pane_with(numbers(1..=3));
    assert!(!has_bar(&render(&mut pane, 10, 10)));
}

#[test]
fn no_scroll_bar_when_rows_equal_the_height() {
    let mut pane = pane_with(numbers(1..=10));
    assert!(!has_bar(&render(&mut pane, 10, 10)));
}

#[test]
fn scroll_bar_takes_a_column() {
    let mut pane = pane_with(["aaaaaaaaa".to_owned(), "bbbbbbbbbbb".to_owned()]);
    let buf = render(&mut pane, 10, 2);
    assert!(has_bar(&buf));
    assert_eq!(rows(&buf), ["bbbbbbbbb", "bb"]);
}

#[test]
fn no_scroll_bar_when_rows_fit_only_at_full_width() {
    let mut pane = pane_with(["aaaaaaaaaa".to_owned(), "b".to_owned()]);
    let first = render(&mut pane, 10, 2);
    let second = render(&mut pane, 10, 2);
    assert_eq!(
        (has_bar(&first), has_bar(&second), rows(&second)),
        (false, false, vec!["aaaaaaaaaa".to_owned(), "b".to_owned()])
    );
}

#[test]
fn second_cell_of_a_wide_grapheme() {
    let mut pane = pane_with(["a日b".to_owned()]);
    render(&mut pane, 20, 3);
    let line = id_of(&pane, "a日b");
    assert_eq!(pane.position(2, 0), Some(Position { line, byte: 1 }));
}

#[test]
fn past_the_end_of_a_line() {
    let mut pane = pane_with(["abc".to_owned()]);
    render(&mut pane, 20, 3);
    let line = id_of(&pane, "abc");
    assert_eq!(pane.position(9, 0), Some(Position { line, byte: 3 }));
}

#[test]
fn no_position() {
    let mut pane = pane_with(numbers(1..=10));
    pane.set_unfinished(Some("Name: ".into()));
    let buf = render(&mut pane, 10, 3);
    assert!(has_bar(&buf));
    assert_eq!((pane.position(9, 0), pane.position(0, 2)), (None, None));
}

#[test]
fn style_follows_its_text_across_a_wrap() {
    use ratatui::style::{Color, Style};
    let mut pane = PaneState::default();
    pane.push_styled("aaaaaaaa redword", &[(9, Style::new().fg(Color::Red))]);
    let buf = render(&mut pane, 10, 3);
    assert_eq!(rows(&buf)[..2], ["aaaaaaaa", "redword"]);
    let red: Vec<Color> = (0..7).map(|x| buf[(x, 1)].fg).collect();
    let plain: Vec<Color> = (0..8).map(|x| buf[(x, 0)].fg).collect();
    assert_eq!((red, plain), (vec![Color::Red; 7], vec![Color::Reset; 8]));
}

#[test]
fn wide_grapheme_at_the_end_of_a_row() {
    let mut pane = pane_with(["abcd日本".to_owned()]);
    let buf = render(&mut pane, 5, 3);
    assert_eq!(rows(&buf)[..2], ["abcd", "日本"]);
    assert_eq!(buf[(4, 0)].symbol(), " ");
}

const FAMILY: &str = "👨\u{200D}👩\u{200D}👧";

#[test]
fn cells_take_the_learned_width() {
    let mut pane = PaneState::with_profile(Arc::new(Profile::learn(&[(FAMILY, 8)])));
    pane.push(&format!("{FAMILY}x"));
    let buf = render(&mut pane, 20, 3);
    let family = &buf[(0, 0)];
    assert_eq!(
        (family.diff_option, buf[(8, 0)].symbol()),
        (
            CellDiffOption::ForcedWidth(NonZeroU16::new(8).unwrap()),
            "x"
        )
    );
}

#[test]
fn cells_take_the_default_width() {
    let mut pane = pane_with([format!("{FAMILY}x")]);
    let buf = render(&mut pane, 20, 3);
    let family = &buf[(0, 0)];
    assert_eq!(
        (family.diff_option, buf[(2, 0)].symbol()),
        (
            CellDiffOption::ForcedWidth(NonZeroU16::new(2).unwrap()),
            "x"
        )
    );
}

#[test]
fn unfinished_line_replaced_by_line() {
    let mut pane = pane_with(numbers(1..=2));
    pane.set_unfinished(Some("hel".into()));
    assert_eq!(rows(&render(&mut pane, 20, 4))[3], "hel");
    pane.push("hello");
    let shown = rows(&render(&mut pane, 20, 4));
    assert_eq!(shown.iter().rfind(|row| !row.is_empty()).unwrap(), "hello");
    assert!(!shown.iter().any(|row| row == "hel"), "{shown:?}");
}

#[test]
fn unfinished_line_replaced_by_unfinished_line() {
    let mut pane = pane_with(numbers(1..=2));
    pane.set_unfinished(Some("Name: ".into()));
    render(&mut pane, 20, 4);
    pane.set_unfinished(Some("Password: ".into()));
    let shown = rows(&render(&mut pane, 20, 4));
    assert_eq!(shown[3], "Password:");
    assert!(
        !shown.iter().any(|row| row.starts_with("Name:")),
        "{shown:?}"
    );
}

#[test]
fn selection_copies_game_lines() {
    let mut pane = pane_with(["first line here".to_owned(), "second".to_owned()]);
    render(&mut pane, 10, 5);
    let first = id_of(&pane, "first line here");
    let second = id_of(&pane, "second");
    pane.select(
        Position {
            line: first,
            byte: 6,
        },
        Position {
            line: second,
            byte: 6,
        },
    );
    assert_eq!(pane.copy_text().as_deref(), Some("line here\nsecond"));
}

#[test]
fn selection_shows_in_reverse() {
    use ratatui::style::Modifier;
    let mut pane = pane_with(["first line here".to_owned()]);
    let line = id_of(&pane, "first line here");
    pane.select(Position { line, byte: 11 }, Position { line, byte: 15 });
    let buf = render(&mut pane, 20, 3);
    let reversed: Vec<u16> = (0..20)
        .filter(|x| buf[(*x, 0)].modifier.contains(Modifier::REVERSED))
        .collect();
    assert_eq!(reversed, [11, 12, 13, 14]);
}
