//! The scenarios of the output-pane capability on the pane widget, drawn into a `Buffer`.

use std::num::NonZeroU16;
use std::sync::Arc;

use mu_tui::store::{Image, LineId};
use mu_tui::width::Profile;
use mu_tui::{Caps, OutputPane, PaneState, Position};
use ratatui::buffer::{Buffer, CellDiffOption};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
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
    let mut pane = PaneState::with_caps(Arc::new(Caps {
        profile: Profile::learn(&[(FAMILY, 8)]),
        graphics: false,
        cell: None,
    }));
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
fn learned_width_of_a_class_that_is_not_risky_is_forced() {
    let mut pane = PaneState::with_caps(Arc::new(Caps {
        profile: Profile::learn(&[("①", 2)]),
        graphics: false,
        cell: None,
    }));
    pane.push("①x");
    let buf = render(&mut pane, 20, 3);
    assert_eq!(
        (buf[(0, 0)].diff_option, buf[(2, 0)].symbol()),
        (
            CellDiffOption::ForcedWidth(NonZeroU16::new(2).unwrap()),
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

const MAP: &str = "https://example.com/map.png";

/// An empty pane in a terminal that draws kitty images if `graphics`.
fn image_pane(graphics: bool) -> PaneState {
    PaneState::with_caps(Arc::new(Caps {
        profile: Profile::default(),
        graphics,
        cell: None,
    }))
}

fn is_placeholder(buf: &Buffer, x: u16, y: u16) -> bool {
    buf[(x, y)].symbol().starts_with('\u{10EEEE}')
}

/// The number of rows of `buf` that start with a placeholder, and the number of placeholders
/// in the first row.
fn placeholder_size(buf: &Buffer) -> (usize, usize) {
    let rows = (0..buf.area.height)
        .filter(|&y| is_placeholder(buf, 0, y))
        .count();
    let columns = (0..buf.area.width)
        .filter(|&x| is_placeholder(buf, x, 0))
        .count();
    (rows, columns)
}

#[test]
fn image_rows_do_not_wrap() {
    let mut pane = image_pane(true);
    pane.push_image(MAP, Image::new(7, 4, 10).unwrap());
    let rows: Vec<usize> = [40, 20]
        .map(|width| placeholder_size(&render(&mut pane, width, 10)).0)
        .to_vec();
    assert_eq!(rows, [4, 4]);
}

#[test]
fn image_placeholder_cells() {
    let mut pane = image_pane(true);
    pane.push_image(MAP, Image::new(7, 4, 10).unwrap());
    let buf = render(&mut pane, 40, 10);
    let cell = &buf[(2, 1)];
    assert_eq!(
        (cell.symbol(), cell.fg),
        ("\u{10EEEE}\u{030D}\u{030E}", Color::Rgb(0, 0, 7))
    );
}

#[test]
fn image_rows_capped_at_297() {
    let sizes: Vec<(usize, usize)> = [(297, 10), (298, 10), (400, 10), (1, 298), (0, 0)]
        .into_iter()
        .map(|(rows, columns)| {
            let mut pane = image_pane(true);
            pane.push_image(MAP, Image::new(7, rows, columns).unwrap());
            placeholder_size(&render(&mut pane, 300, 400))
        })
        .collect();
    assert_eq!(sizes, [(297, 10), (297, 10), (297, 10), (1, 297), (1, 1)]);
}

#[test]
fn image_id_above_255() {
    let mut pane = image_pane(true);
    pane.push_image(MAP, Image::new(256, 2, 3).unwrap());
    let buf = render(&mut pane, 10, 4);
    let colors: Vec<Color> = (0..2)
        .flat_map(|y| (0..3).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].fg)
        .collect();
    assert_eq!(colors, [Color::Rgb(0, 1, 0); 6]);
}

#[test]
fn image_wider_than_the_pane() {
    let mut pane = image_pane(true);
    pane.push_image(MAP, Image::new(7, 2, 10).unwrap());
    let buf = render(&mut pane, 5, 4);
    let cells: Vec<String> = (0..2)
        .flat_map(|y| (0..5).map(move |x| (x, y)))
        .map(|(x, y)| buf[(x, y)].symbol().to_owned())
        .collect();
    let marks = ['\u{0305}', '\u{030D}', '\u{030E}', '\u{0310}', '\u{0312}'];
    let want: Vec<String> = (0..2)
        .flat_map(|row| (0..5).map(move |column| (row, column)))
        .map(|(row, column)| ['\u{10EEEE}', marks[row], marks[column]].iter().collect())
        .collect();
    assert_eq!(cells, want);
}

#[test]
fn image_url_fallback() {
    let mut pane = image_pane(false);
    pane.push_image(MAP, Image::new(7, 4, 10).unwrap());
    assert_eq!(rows(&render(&mut pane, 40, 3)), [MAP, "", ""]);
}

#[test]
fn url_with_control_characters() {
    let mut pane = image_pane(false);
    pane.push_image("https://x/\x1b[2J\x07", Image::new(7, 4, 10).unwrap());
    assert_eq!(
        rows(&render(&mut pane, 40, 1)),
        ["https://x/\u{FFFD}[2J\u{FFFD}"]
    );
}

#[test]
fn image_copies_its_url() {
    let mut pane = image_pane(true);
    pane.push("before");
    pane.push_image(MAP, Image::new(7, 2, 10).unwrap());
    pane.push("after");
    render(&mut pane, 40, 6);
    let press = pane.position(1, 1).unwrap();
    let end = pane.position(4, 2).unwrap();
    pane.press(press);
    pane.drag(end);
    assert_eq!(pane.release().as_deref(), Some(MAP));
}

#[test]
fn resize_that_shows_the_bottom_follows_new_lines() {
    let mut pane = pane_with(numbers(1..=10));
    render(&mut pane, 20, 3);
    pane.scroll_rows(-1);
    render(&mut pane, 20, 5);
    pane.push("11");
    pane.push("12");
    assert_eq!(rows(&render(&mut pane, 20, 5)).last().unwrap(), "12");
}

#[test]
fn resize_that_fits_every_row_hides_the_bar() {
    let mut pane = pane_with(numbers(1..=4));
    render(&mut pane, 20, 2);
    pane.scroll_rows(-1);
    let buf = render(&mut pane, 20, 6);
    assert_eq!((has_bar(&buf), pane.anchor()), (false, None));
}

#[test]
fn image_ids_outside_24_bits_are_refused() {
    let ids: Vec<Option<u32>> = [0, 1, 0x00FF_FFFF, 0x0100_0000]
        .into_iter()
        .map(|id| Image::new(id, 1, 1).map(Image::id))
        .collect();
    assert_eq!(ids, [None, Some(1), Some(0x00FF_FFFF), None]);
}

#[test]
fn empty_line_renders_an_empty_row() {
    let mut pane = pane_with(["a", "", "b"].map(str::to_owned));
    assert_eq!(rows(&render(&mut pane, 20, 3)), ["a", "", "b"]);
}

#[test]
fn grapheme_wider_than_the_pane_renders_as_one_replacement() {
    let mut pane = pane_with(["x日y".to_owned()]);
    assert_eq!(rows(&render(&mut pane, 1, 3)), ["x", "\u{FFFD}", "y"]);
}

#[test]
fn empty_pane_after_trims_renders_an_empty_area() {
    let mut pane = PaneState::default();
    pane.trim(1);
    pane.trim(0);
    let buf = render(&mut pane, 20, 3);
    assert_eq!(buf, Buffer::empty(buf.area));
}

#[test]
fn trim_moves_the_anchor_to_the_oldest_live_line() {
    let mut pane = pane_with(numbers(1..=10));
    render(&mut pane, 20, 3);
    pane.scroll_pages(-3);
    pane.trim(5);
    assert_eq!(pane.anchor().map(|at| at.line), Some(id_of(&pane, "6")));
}

#[test]
fn scroll_back_holds_when_lines_wrap_beside_the_bar() {
    let mut pane = pane_with(std::iter::repeat_n("a".repeat(10), 10));
    let bottom = rows(&render(&mut pane, 10, 3));
    pane.scroll_rows(-1);
    assert_ne!(rows(&render(&mut pane, 10, 3)), bottom);
}

#[test]
fn render_at_zero_width_keeps_the_scroll_position() {
    let mut pane = pane_with(numbers(1..=10));
    render(&mut pane, 20, 3);
    pane.scroll_rows(-2);
    let anchor = pane.anchor();
    render(&mut pane, 0, 3);
    assert_eq!((anchor.is_some(), pane.anchor()), (true, anchor));
}

#[test]
fn zero_width_grapheme_after_a_full_row_draws_nothing() {
    let mut pane = pane_with(["a".repeat(20) + "\u{200B}"]);
    assert_eq!(
        rows(&render(&mut pane, 20, 3)),
        ["a".repeat(20), String::new(), String::new()]
    );
}

#[test]
fn zero_width_grapheme_after_a_full_unfinished_line_draws_nothing() {
    let mut pane = PaneState::default();
    let prompt = "a".repeat(20) + "\u{200B}";
    pane.set_unfinished(Some(prompt.as_str().into()));
    assert_eq!(rows(&render(&mut pane, 20, 1)), ["a".repeat(20)]);
}

#[test]
fn zero_width_grapheme_at_the_end_of_a_row_takes_no_cell() {
    let mut pane = pane_with(["ab\u{200B}".to_owned()]);
    assert_eq!(render(&mut pane, 20, 1)[(2, 0)].symbol(), " ");
}

/// The foreground color of each cell of the first row.
fn colors(buf: &Buffer, width: u16) -> Vec<Color> {
    (0..width).map(|x| buf[(x, 0)].fg).collect()
}

#[test]
fn pushed_escape_reaches_no_cell() {
    let mut pane = PaneState::default();
    pane.push("a\x1b[2Jb");
    assert_eq!(rows(&render(&mut pane, 10, 1)), ["a\u{FFFD}[2Jb"]);
}

#[test]
fn unfinished_escape_reaches_no_cell() {
    let mut pane = PaneState::default();
    pane.set_unfinished(Some("a\x07b".into()));
    assert_eq!(rows(&render(&mut pane, 10, 1)), ["a\u{FFFD}b"]);
}

#[test]
fn style_after_a_wider_replacement_starts_at_its_char() {
    let mut pane = PaneState::default();
    let (red, blue) = (Style::new().fg(Color::Red), Style::new().fg(Color::Blue));
    pane.push_styled("\u{85}ab", &[(0, red), (3, blue)]);
    assert_eq!(
        colors(&render(&mut pane, 3, 1), 3),
        [Color::Red, Color::Red, Color::Blue]
    );
}

#[test]
fn placeholder_char_in_text_is_a_replacement() {
    let mut pane = PaneState::default();
    pane.push("\u{10EEEE}b");
    assert_eq!(rows(&render(&mut pane, 3, 1)), ["\u{FFFD}b"]);
}

fn small_pane_without_its_first_line() -> PaneState {
    let mut pane = PaneState::new(3);
    for line in ["a", "b", "c", "d"] {
        pane.push(line);
    }
    pane
}

#[test]
fn select_on_a_dropped_line_starts_at_the_oldest_live_line() {
    let mut pane = small_pane_without_its_first_line();
    let b = id_of(&pane, "b");
    let c = id_of(&pane, "c");
    pane.select(
        Position {
            line: LineId(0),
            byte: 0,
        },
        Position { line: c, byte: 1 },
    );
    assert_eq!(
        (pane.selection(), pane.copy_text().as_deref()),
        (
            Some((Position { line: b, byte: 0 }, Position { line: c, byte: 1 })),
            Some("b\nc")
        )
    );
}

#[test]
fn press_on_a_dropped_line_starts_at_the_oldest_live_line() {
    let mut pane = small_pane_without_its_first_line();
    let b = id_of(&pane, "b");
    pane.press(Position {
        line: LineId(0),
        byte: 0,
    });
    assert_eq!(
        pane.selection(),
        Some((Position { line: b, byte: 0 }, Position { line: b, byte: 0 }))
    );
}

#[test]
fn select_inside_a_char_starts_at_the_char() {
    let mut pane = pane_with(["a日b".to_owned()]);
    let line = id_of(&pane, "a日b");
    pane.select(Position { line, byte: 2 }, Position { line, byte: 4 });
    assert_eq!(
        (pane.selection(), pane.copy_text().as_deref()),
        (
            Some((Position { line, byte: 1 }, Position { line, byte: 4 })),
            Some("日")
        )
    );
}

#[test]
fn press_inside_a_char_starts_at_the_char() {
    let mut pane = pane_with(["a日b".to_owned()]);
    let line = id_of(&pane, "a日b");
    pane.press(Position { line, byte: 2 });
    pane.drag(Position { line, byte: 4 });
    assert_eq!(pane.release().as_deref(), Some("日b"));
}

#[test]
fn select_past_the_end_of_a_line_ends_at_the_line() {
    let mut pane = pane_with(["abc".to_owned()]);
    let line = id_of(&pane, "abc");
    pane.select(Position { line, byte: 1 }, Position { line, byte: 9 });
    assert_eq!(pane.copy_text().as_deref(), Some("bc"));
}

#[test]
fn select_past_the_newest_line_ends_at_the_newest_line() {
    let mut pane = pane_with(["abc".to_owned()]);
    let line = id_of(&pane, "abc");
    pane.select(
        Position { line, byte: 1 },
        Position {
            line: LineId(line.0 + 5),
            byte: 0,
        },
    );
    assert_eq!(pane.copy_text().as_deref(), Some("bc"));
}

#[test]
fn select_in_an_empty_pane_selects_nothing() {
    let mut pane = PaneState::default();
    pane.select(
        Position {
            line: LineId(0),
            byte: 0,
        },
        Position {
            line: LineId(0),
            byte: 0,
        },
    );
    assert_eq!(pane.selection(), None);
}
