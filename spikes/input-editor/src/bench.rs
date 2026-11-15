use std::time::{Duration, Instant};

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui_textarea::{CursorMove, Input, Key};
use unicode_segmentation::UnicodeSegmentation;

use crate::editor::{Editor, normalize_paste};
use crate::sample;

const RUNS: usize = 50;

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

fn us(d: Duration) -> String {
    format!("{:.1}", d.as_secs_f64() * 1e6)
}

fn row(name: &str, value: impl std::fmt::Display, unit: &str) {
    println!("{name}\t{value}\t{unit}");
}

fn terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(width, height)).expect("test backend")
}

fn draw(term: &mut Terminal<TestBackend>, ed: &mut Editor) {
    term.draw(|f| {
        let area = f.area();
        ed.draw(f, area, true);
    })
    .expect("draw");
}

/// Whether the cell under the terminal cursor is the cell the widget drew
/// as its cursor.
fn cursor_agrees(term: &mut Terminal<TestBackend>) -> bool {
    let pos = term.get_cursor_position().expect("cursor");
    let buf = term.backend().buffer();
    buf.cell(pos)
        .is_some_and(|c| c.modifier.contains(Modifier::REVERSED))
}

pub fn run() {
    println!("measure\tvalue\tunit");
    let pose = sample::pose(4096);
    row("pose_bytes", pose.len(), "bytes");
    row("pose_lines", pose.lines().count(), "lines");

    paste(&pose);
    for width in [40u16, 80, 120] {
        frame(&pose, width);
    }
    keystrokes(&pose);
    for width in [20u16, 40, 80] {
        cursor_walk(&pose, width);
    }
    graphemes();
}

fn paste(pose: &str) {
    let times = (0..RUNS)
        .map(|_| {
            let mut ed = Editor::new();
            let t = Instant::now();
            ed.paste(pose);
            t.elapsed()
        })
        .collect();
    row("paste_insert_median", us(median(times)), "us");

    let mut ed = Editor::new();
    ed.paste(pose);
    let kept = ed.area.lines().join("\n") == normalize_paste(pose);
    row("paste_text_kept", kept, "bool");

    let mut steps = 0;
    while !ed.area.is_empty() && steps < 10_000 {
        ed.area.undo();
        steps += 1;
    }
    row("paste_undo_steps", steps, "undo calls to empty");
}

fn frame(pose: &str, width: u16) {
    let mut term = terminal(width, 8);
    let mut ed = Editor::new();
    ed.paste(pose);
    draw(&mut term, &mut ed);
    let rect = Rect::new(0, 0, width, 8);
    let pos = ed.cursor_position(rect);
    row(
        &format!("paste_cursor_on_screen_w{width}"),
        rect.contains(pos) && cursor_agrees(&mut term),
        "bool",
    );

    let times = (0..RUNS)
        .map(|_| {
            // A full redraw each time, as after a resize.
            term.clear().expect("clear");
            let t = Instant::now();
            draw(&mut term, &mut ed);
            t.elapsed()
        })
        .collect();
    row(
        &format!("frame_4k_median_w{width}"),
        us(median(times)),
        "us",
    );
}

/// A paste without bracketed paste mode arrives as one key event for each
/// character, and the client draws after each event.
fn keystrokes(pose: &str) {
    let mut term = terminal(80, 8);
    let mut ed = Editor::new();
    let t = Instant::now();
    for c in pose.chars() {
        let key = match c {
            '\n' => KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            c => KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE),
        };
        ed.area.input(Input::from(key));
        draw(&mut term, &mut ed);
    }
    row(
        "keystroke_paste_total",
        format!("{:.1}", t.elapsed().as_secs_f64() * 1e3),
        "ms",
    );
    let mut steps = 0;
    while !ed.area.is_empty() && steps < 10_000 {
        ed.area.undo();
        steps += 1;
    }
    row(
        "keystroke_paste_undo_steps",
        steps,
        "undo calls to empty (history limit)",
    );
}

/// Moves the cursor over every character of the pose and checks that the
/// terminal cursor sits on the cell the widget marks as its cursor.
fn cursor_walk(pose: &str, width: u16) {
    let height = 6;
    let mut term = terminal(width, height);
    let mut ed = Editor::new();
    ed.paste(pose);
    ed.area.move_cursor(CursorMove::Top);
    ed.area.move_cursor(CursorMove::Head);
    let total: usize = ed.area.lines().iter().map(|l| l.chars().count() + 1).sum();
    let mut disagree = 0;
    let mut inside = 0;
    for _ in 0..total {
        draw(&mut term, &mut ed);
        if !cursor_agrees(&mut term) {
            disagree += 1;
        }
        let ratatui_textarea::DataCursor(r, c) = ed.area.cursor();
        let line = &ed.area.lines()[r];
        let byte = line.char_indices().nth(c).map_or(line.len(), |(b, _)| b);
        if !line.is_char_boundary(byte)
            || !(byte == line.len() || line.grapheme_indices(true).any(|(b, _)| b == byte))
        {
            inside += 1;
        }
        ed.area.move_cursor(CursorMove::Forward);
    }
    row(&format!("cursor_positions_w{width}"), total, "positions");
    row(&format!("cursor_disagree_w{width}"), disagree, "positions");
    row(
        &format!("cursor_inside_grapheme_w{width}"),
        inside,
        "positions",
    );
}

/// The column that `screen_cursor` reports for the cursor after `g`, and
/// the column where the widget draws that cursor.
fn cursor_after(g: &str) -> (usize, String) {
    let mut term = terminal(20, 1);
    let mut ed = Editor::new();
    ed.area.insert_str(format!("{g}x"));
    ed.area.move_cursor(CursorMove::Head);
    for _ in 0..g.chars().count() {
        ed.area.move_cursor(CursorMove::Forward);
    }
    draw(&mut term, &mut ed);
    let buf = term.backend().buffer();
    let drawn = (0..20u16)
        .find(|&x| {
            buf.cell((x, 0))
                .is_some_and(|c| c.modifier.contains(Modifier::REVERSED))
        })
        .map_or("none".into(), |x| x.to_string());
    (ed.area.screen_cursor().col, drawn)
}

/// How many presses of Right and Backspace one grapheme takes.
fn graphemes() {
    for (name, g) in [
        ("family_zwj", "👨‍👩‍👧"),
        ("e_combining", "e\u{301}"),
        ("flag", "🇯🇵"),
        ("candle_vs16", "🕯️"),
        ("han", "港"),
    ] {
        let mut ed = Editor::new();
        ed.area.insert_str(format!("a{g}b"));
        ed.area.move_cursor(CursorMove::Head);
        ed.area.move_cursor(CursorMove::Forward);
        let mut rights = 0;
        while ed.area.cursor().1 < 1 + g.chars().count() {
            ed.area.input(Input {
                key: Key::Right,
                ..Input::default()
            });
            rights += 1;
        }
        let mut backs = 0;
        while ed.area.lines()[0] != "ab" {
            ed.area.input(Input {
                key: Key::Backspace,
                ..Input::default()
            });
            backs += 1;
            if backs > 20 {
                break;
            }
        }
        row(&format!("grapheme_right_{name}"), rights, "presses");
        let (reported, drawn) = cursor_after(g);
        row(
            &format!("cursor_col_after_{name}"),
            format!("{reported}/{drawn}"),
            "screen_cursor col / drawn col",
        );
        row(&format!("grapheme_backspace_{name}"), backs, "presses");
    }
}
