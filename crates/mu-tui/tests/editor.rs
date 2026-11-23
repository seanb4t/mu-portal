//! The scenarios of the input-editor capability on `update` and `view`, drawn on a
//! `TestBackend`.

use mu_tui::probe::Outcome;
use mu_tui::width::Profile;
use mu_tui::wrap::{self, Row};
use mu_tui::{EditorId, Effect, Model, Msg, update, view};
use proptest::prelude::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use termina::Event as TermEvent;
use termina::event::{KeyCode, Modifiers};

use support::{press, type_lines, type_text};

mod support;

const FAMILY: &str = "👨\u{200d}👩\u{200d}👧";

/// A sample pose: 4096 bytes in 51 lines.
const POSE: &str = include_str!("data/pose-4096.txt");

fn app() -> Model {
    Model::new(["alpha/aria".to_owned()])
}

/// The rows of a screen of `width` by `height`, with no trailing spaces.
fn screen(app: &mut Model, width: u16, height: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| view(app, frame)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| {
            let row: String = (0..width).map(|x| buffer[(x, y)].symbol()).collect();
            row.trim_end().to_owned()
        })
        .collect()
}

/// The last `count` rows of a screen of `width` by `height`.
fn bottom(app: &mut Model, width: u16, height: u16, count: usize) -> Vec<String> {
    let rows = screen(app, width, height);
    rows[rows.len() - count..].to_vec()
}

fn key(app: &mut Model, code: KeyCode) {
    assert_eq!(press(app, code, Modifiers::NONE), []);
}

fn ctrl(app: &mut Model, c: char) {
    assert_eq!(press(app, KeyCode::Char(c), Modifiers::CONTROL), []);
}

fn paste(app: &mut Model, text: &str) {
    assert_eq!(
        update(app, Msg::Terminal(TermEvent::Paste(text.to_owned()))),
        []
    );
}

/// The lines `1` to `12`.
fn twelve() -> Vec<String> {
    (1..=12).map(|n| n.to_string()).collect()
}

fn type_twelve(app: &mut Model) {
    let lines = twelve();
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    type_lines(app, &lines);
}

// The editor wraps a long line.

/// 10 words of 5 letters, with a space between each two: 59 characters.
fn ten_words() -> String {
    ('a'..='j')
        .map(|c| c.to_string().repeat(5))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn line_wider_than_the_editor() {
    let mut app = app();
    let text = ten_words();
    type_text(&mut app, &text);
    let rows = bottom(&mut app, 40, 24, 2);
    assert_eq!(
        rows,
        [
            "aaaaa bbbbb ccccc ddddd eeeee fffff",
            "ggggg hhhhh iiiii jjjjj"
        ]
    );
    let effects = press(&mut app, KeyCode::Enter, Modifiers::NONE);
    assert_eq!(
        effects,
        [Effect::Send {
            world: "alpha/aria".to_owned(),
            text
        }]
    );
}

#[test]
fn word_wider_than_the_editor() {
    let mut app = app();
    type_text(&mut app, &"x".repeat(30));
    assert_eq!(
        bottom(&mut app, 20, 24, 2),
        ["x".repeat(20), "x".repeat(10)]
    );
}

#[test]
fn editor_and_pane_wrap_alike() {
    for (text, width) in [(ten_words(), 40), ("x".repeat(30), 20)] {
        let mut app = app();
        type_text(&mut app, &text);
        let mut pane_rows: Vec<Row> = Vec::new();
        wrap::wrap(&text, width, &Profile::default(), &mut pane_rows);
        let pane_rows: Vec<&str> = pane_rows
            .iter()
            .map(|row| text[row.start..row.end].trim_end())
            .collect();
        let editor_rows = bottom(&mut app, width, 24, pane_rows.len());
        assert_eq!(editor_rows, pane_rows, "{text}");
    }
}

// The editor grows with its text.

#[test]
fn three_lines() {
    let mut app = app();
    type_lines(&mut app, &["one", "two", "three"]);
    let rows = screen(&mut app, 80, 24);
    assert_eq!(rows[20..], ["", "one", "two", "three"]);
}

#[test]
fn more_rows_than_the_limit() {
    let mut app = app();
    type_twelve(&mut app);
    let rows = screen(&mut app, 80, 24);
    assert_eq!(rows[15..], [&[String::new()][..], &twelve()[4..]].concat());
}

#[test]
fn short_terminal() {
    let mut app = app();
    type_twelve(&mut app);
    let rows = screen(&mut app, 80, 10);
    assert_eq!(rows[4..], [&[String::new()][..], &twelve()[7..]].concat());
}

#[test]
fn terminal_of_17_rows() {
    let mut app = app();
    type_twelve(&mut app);
    let rows = screen(&mut app, 80, 17);
    assert_eq!(rows[8..], [&[String::new()][..], &twelve()[4..]].concat());
}

#[test]
fn terminal_of_3_rows() {
    let mut app = app();
    type_lines(&mut app, &["one", "two", "three"]);
    let rows = screen(&mut app, 80, 3);
    assert_eq!(rows, ["[alpha/aria]", "", "three"]);
}

#[test]
fn terminal_of_1_row() {
    let mut app = app();
    type_lines(&mut app, &["one", "two", "three"]);
    screen(&mut app, 80, 1);
    let editor = app.editor(EditorId::POSE).unwrap();
    assert!(editor.area().height <= 1, "{:?}", editor.area());
}

#[test]
fn back_to_one_row() {
    let mut app = app();
    type_lines(&mut app, &["one", "two", "three"]);
    assert_eq!(screen(&mut app, 80, 24)[21..], ["one", "two", "three"]);
    press(&mut app, KeyCode::Enter, Modifiers::NONE);
    assert_eq!(screen(&mut app, 80, 24)[20..], ["", "", "", ""]);
    type_text(&mut app, "x");
    assert_eq!(screen(&mut app, 80, 24)[20..], ["", "", "", "x"]);
}

// The cursor moves by grapheme.

#[test]
fn backspace_after_an_emoji_with_joiners() {
    let mut app = app();
    type_text(&mut app, &format!("a{FAMILY}"));
    key(&mut app, KeyCode::Backspace);
    assert_eq!(app.input(), "a");
}

#[test]
fn left_over_a_flag() {
    let mut app = app();
    type_text(&mut app, "a🇺🇸");
    key(&mut app, KeyCode::Left);
    type_text(&mut app, "x");
    assert_eq!(app.input(), "ax🇺🇸");
}

#[test]
fn right_over_a_combining_accent() {
    let mut app = app();
    type_text(&mut app, "e\u{301}z");
    key(&mut app, KeyCode::Home);
    key(&mut app, KeyCode::Right);
    type_text(&mut app, "x");
    assert_eq!(app.input(), "e\u{301}xz");
}

#[test]
fn delete_before_a_skin_tone() {
    let mut app = app();
    type_text(&mut app, "👍🏽b");
    key(&mut app, KeyCode::Home);
    key(&mut app, KeyCode::Delete);
    assert_eq!(app.input(), "b");
}

#[test]
fn up_into_an_emoji_with_joiners() {
    let mut app = app();
    type_lines(&mut app, &[&format!("a{FAMILY}"), "xxx"]);
    screen(&mut app, 80, 24);
    key(&mut app, KeyCode::Up);
    type_text(&mut app, "z");
    assert_eq!(app.input(), format!("a{FAMILY}z\nxxx"));
}

// The editor uses the widths of the terminal.

/// The index of the first cell of the last row that holds `symbol`.
fn column_of(app: &mut Model, symbol: &str) -> Option<usize> {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| view(app, frame)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..80).position(|x| buffer[(x, 23)].symbol() == symbol)
}

#[test]
fn editor_uses_the_probed_widths() {
    let mut app = app();
    let outcome = Outcome {
        profile: Profile::learn(&[(FAMILY, 8)]),
        complete: true,
        measured: true,
        ..Outcome::default()
    };
    assert_eq!(update(&mut app, Msg::Probe(outcome)), []);
    type_text(&mut app, &format!("{FAMILY}x"));
    // Column 9, counted from 1.
    assert_eq!(column_of(&mut app, "x"), Some(8));
}

#[test]
fn world_editors_use_the_probed_widths() {
    let mut app = Model::new([]);
    let outcome = Outcome {
        profile: Profile::learn(&[(FAMILY, 8)]),
        complete: true,
        measured: true,
        ..Outcome::default()
    };
    assert_eq!(update(&mut app, Msg::Probe(outcome)), []);
    let world = |id: &str| mu_proto::World {
        id: id.to_owned(),
        ..Default::default()
    };
    update(
        &mut app,
        Msg::Listed(Ok(vec![world("alpha/aria"), world("beta/bryn")])),
    );
    type_text(&mut app, &format!("{FAMILY}x"));
    assert_eq!(column_of(&mut app, "x"), Some(8));
    press(&mut app, KeyCode::Char('n'), Modifiers::CONTROL);
    type_text(&mut app, &format!("{FAMILY}y"));
    assert_eq!(column_of(&mut app, "y"), Some(8));
}

#[test]
fn editor_uses_unicode_width() {
    let mut app = app();
    type_text(&mut app, &format!("{FAMILY}x"));
    // Column 3, counted from 1.
    assert_eq!(column_of(&mut app, "x"), Some(2));
}

// Keys insert a line break.

#[test]
fn each_key_inserts_a_line_break() {
    for (code, modifiers) in [
        (KeyCode::Enter, Modifiers::SHIFT),
        (KeyCode::Char('j'), Modifiers::CONTROL),
    ] {
        let mut app = app();
        type_text(&mut app, "a");
        assert_eq!(press(&mut app, code, modifiers), [], "{code:?}");
        type_text(&mut app, "b");
        assert_eq!(app.input(), "a\nb", "{code:?}");
        assert_eq!(bottom(&mut app, 80, 24, 2), ["a", "b"], "{code:?}");
    }
}

// The editing keys follow readline.

#[test]
fn ctrl_u_deletes_to_the_start_of_the_line() {
    let mut app = app();
    type_text(&mut app, "one two");
    for _ in 0..3 {
        key(&mut app, KeyCode::Left);
    }
    ctrl(&mut app, 'u');
    assert_eq!(app.input(), "two");
}

#[test]
fn ctrl_u_at_the_start_of_a_line() {
    let mut app = app();
    type_lines(&mut app, &["one", "two"]);
    key(&mut app, KeyCode::Home);
    ctrl(&mut app, 'u');
    assert_eq!(app.input(), "one\ntwo");
}

#[test]
fn undo_and_redo() {
    let mut app = app();
    paste(&mut app, "one");
    ctrl(&mut app, 'z');
    assert_eq!(app.input(), "");
    let redo = Modifiers::CONTROL | Modifiers::SHIFT;
    assert_eq!(press(&mut app, KeyCode::Char('z'), redo), []);
    assert_eq!(app.input(), "one");
}

#[test]
fn ctrl_r_changes_nothing() {
    let mut app = app();
    paste(&mut app, "one");
    ctrl(&mut app, 'z');
    ctrl(&mut app, 'r');
    assert_eq!(app.input(), "");
}

// A paste is one edit.

#[test]
fn paste_of_several_lines() {
    let mut app = app();
    type_text(&mut app, "x");
    paste(&mut app, "one\r\ntwo\rthree");
    assert_eq!(app.input(), "xone\ntwo\nthree");
}

#[test]
fn undo_of_a_paste() {
    assert_eq!((POSE.len(), POSE.lines().count()), (4096, 51));
    let mut app = app();
    type_text(&mut app, "x");
    paste(&mut app, POSE);
    assert_eq!(app.input(), format!("x{POSE}"));
    ctrl(&mut app, 'z');
    assert_eq!(app.input(), "x");
}

#[test]
fn control_characters_in_a_paste() {
    let mut app = app();
    paste(&mut app, "a\x1b[31mb\x07\tc\u{9b}d");
    assert_eq!(app.input(), "a[31mb\tcd");
}

#[test]
fn paste_of_64_kib() {
    let line = "The quick brown fox jumps over the lazy dog. ";
    let text = line.repeat(64 * 1024 / line.len() + 1);
    let text = &text[..64 * 1024];
    let mut app = app();
    paste(&mut app, text);
    assert_eq!(app.input(), text);
    ctrl(&mut app, 'z');
    assert_eq!(app.input(), "");
}

#[test]
fn paste_over_a_selection() {
    let mut app = app();
    type_text(&mut app, "xy");
    assert_eq!(press(&mut app, KeyCode::Left, Modifiers::SHIFT), []);
    paste(&mut app, "z");
    assert_eq!(app.input(), "xz");
    ctrl(&mut app, 'z');
    assert_eq!(app.input(), "x");
    ctrl(&mut app, 'z');
    assert_eq!(app.input(), "xy");
}

proptest! {
    #[test]
    fn clean_paste_holds_no_control_character(text in any::<String>()) {
        let mut first = app();
        paste(&mut first, &text);
        let cleaned = first.input();
        prop_assert!(
            !cleaned.chars().any(|c| c.is_control() && c != '\t' && c != '\n'),
            "{cleaned:?}"
        );
        let mut second = app();
        paste(&mut second, &cleaned);
        prop_assert_eq!(second.input(), cleaned);
    }
}

/// A key that moves the cursor, deletes or undoes.
fn move_or_edit() -> impl Strategy<Value = (KeyCode, Modifiers)> {
    let plain = [
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Home,
        KeyCode::End,
        KeyCode::Backspace,
        KeyCode::Delete,
    ];
    let with_ctrl = [
        // The moves and the deletes by word.
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Char('w'),
        // Delete to the end and to the start of the line.
        KeyCode::Char('k'),
        KeyCode::Char('u'),
        // Undo.
        KeyCode::Char('z'),
    ];
    let keys: Vec<(KeyCode, Modifiers)> = plain
        .into_iter()
        .map(|code| (code, Modifiers::NONE))
        .chain(with_ctrl.into_iter().map(|code| (code, Modifiers::CONTROL)))
        .chain([(KeyCode::Char('z'), Modifiers::CONTROL | Modifiers::SHIFT)])
        .collect();
    prop::sample::select(keys)
}

proptest! {
    /// After each sequence of moves, deletes, undos and redos, a typed `z` lands between two
    /// graphemes. A `z` inside a grapheme leaves a part of it, such as an emoji with a joiner
    /// and no next emoji.
    #[test]
    fn cursor_stays_at_a_grapheme_boundary(
        lines in prop::collection::vec(
            prop::collection::vec(
                prop::sample::select(vec!["a", " ", FAMILY, "🇺🇸", "e\u{301}", "👍🏽"]),
                0..6,
            ),
            1..4,
        ),
        keys in prop::collection::vec(move_or_edit(), 0..40),
    ) {
        let mut app = app();
        let lines: Vec<String> = lines.iter().map(|line| line.concat()).collect();
        // A paste is one edit of whole graphemes. Typed text is one edit for each code point,
        // so an undo of it leaves a part of a grapheme with the cursor at its end.
        paste(&mut app, &lines.join("\n"));
        screen(&mut app, 80, 24);
        for (code, modifiers) in keys {
            prop_assert_eq!(press(&mut app, code, modifiers), []);
        }
        type_text(&mut app, "z");
        let text = app.input();
        let whole = ["a", " ", FAMILY, "🇺🇸", "e\u{301}", "👍🏽", "z", "\n"];
        for grapheme in unicode_segmentation::UnicodeSegmentation::graphemes(text.as_str(), true) {
            prop_assert!(whole.contains(&grapheme), "{grapheme:?} in {text:?}");
        }
    }
}
