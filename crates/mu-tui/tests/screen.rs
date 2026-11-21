//! The cell size in pixels from the window size, and the result of a step with its restore.

use std::io::{self, ErrorKind};

use mu_tui::screen::{cell_pixels, finish};
use ratatui::backend::WindowSize;
use ratatui::layout::Size;

fn window(cells: (u16, u16), pixels: (u16, u16)) -> WindowSize {
    WindowSize {
        columns_rows: Size::new(cells.0, cells.1),
        pixels: Size::new(pixels.0, pixels.1),
    }
}

#[test]
fn cell_is_the_window_over_its_cells() {
    assert_eq!(
        cell_pixels(&window((80, 24), (800, 480))),
        Some(Size::new(10, 20))
    );
}

#[test]
fn no_pixel_size_gives_no_cell() {
    let sizes = [
        window((80, 24), (0, 0)),
        window((80, 24), (800, 0)),
        window((0, 0), (800, 480)),
    ];
    assert_eq!(sizes.map(|size| cell_pixels(&size)), [None; 3]);
}

fn error(kind: ErrorKind, text: &str) -> io::Error {
    io::Error::new(kind, text.to_owned())
}

#[test]
fn both_errors_keep_the_first() {
    let result = finish::<()>(
        Err(error(ErrorKind::BrokenPipe, "read failed")),
        Err(error(ErrorKind::PermissionDenied, "restore failed")),
    );
    let error = result.unwrap_err();
    assert_eq!(
        (error.kind(), error.to_string()),
        (
            ErrorKind::BrokenPipe,
            "read failed (the restore also failed: restore failed)".to_owned()
        )
    );
}

#[test]
fn read_error_alone() {
    let result = finish::<()>(Err(error(ErrorKind::BrokenPipe, "read failed")), Ok(()));
    let error = result.unwrap_err();
    assert_eq!(
        (error.kind(), error.to_string()),
        (ErrorKind::BrokenPipe, "read failed".to_owned())
    );
}

#[test]
fn restore_error_alone() {
    let result = finish(
        Ok(7),
        Err(error(ErrorKind::PermissionDenied, "restore failed")),
    );
    let error = result.unwrap_err();
    assert_eq!(
        (error.kind(), error.to_string()),
        (ErrorKind::PermissionDenied, "restore failed".to_owned())
    );
}

#[test]
fn both_ok() {
    assert_eq!(finish(Ok(7), Ok(())).unwrap(), 7);
}

/// The bytes that `leave` writes for `restore`.
fn left(restore: &mu_tui::screen::Restore) -> String {
    let mut out = Vec::new();
    mu_tui::screen::leave(&mut out, restore).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn enter_turns_on_bracketed_paste() {
    let mut out = Vec::new();
    mu_tui::screen::enter(&mut out).unwrap();
    assert!(String::from_utf8(out).unwrap().contains("\x1b[?2004h"));
}

#[test]
fn leave_turns_off_bracketed_paste() {
    assert!(left(&mu_tui::screen::Restore::default()).contains("\x1b[?2004l"));
}

#[test]
fn leave_pops_the_keyboard_flag_only_after_a_push() {
    let restore = mu_tui::screen::Restore::default();
    assert!(!left(&restore).contains("\x1b[<1u"));
    let mut out = Vec::new();
    mu_tui::screen::push_keyboard_flag(&mut out, &restore).unwrap();
    assert_eq!(out, b"\x1b[>1u");
    let left = left(&restore);
    let (pop, screen) = (left.find("\x1b[<1u"), left.find("\x1b[?1049l"));
    assert!(pop.is_some() && pop < screen, "{left:?}");
}

#[test]
fn leave_turns_off_mode_2027_only_when_asked() {
    let restore = mu_tui::screen::Restore::default();
    assert!(!left(&restore).contains("\x1b[?2027l"));
    restore.set_grapheme_mode(true);
    assert!(left(&restore).contains("\x1b[?2027l"));
}
