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
