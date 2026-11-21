//! The terminal of the client: raw mode, the alternate screen and mouse capture, the sequences
//! that undo them, and [`finish`], which keeps the first error when a restore also fails.

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use ratatui::Terminal;
use ratatui::backend::{Backend, TerminaBackend, WindowSize};
use ratatui::layout::Size;
use termina::escape::csi::{Csi, DecPrivateMode, DecPrivateModeCode, Mode};
use termina::{EventReader, PlatformTerminal, Terminal as _};

/// The ratatui terminal on the Termina backend.
pub type Screen = Terminal<TerminaBackend<PlatformTerminal>>;

fn private_mode(code: DecPrivateModeCode, on: bool) -> Csi {
    let mode = DecPrivateMode::Code(code);
    Csi::Mode(if on {
        Mode::SetDecPrivateMode(mode)
    } else {
        Mode::ResetDecPrivateMode(mode)
    })
}

/// The modes of mouse capture: press and release, drag, and the SGR format of the reports.
const MOUSE: [DecPrivateModeCode; 3] = [
    DecPrivateModeCode::MouseTracking,
    DecPrivateModeCode::ButtonEventMouse,
    DecPrivateModeCode::SGRMouse,
];

/// Turns off mouse capture, and mode 2027 if `grapheme_mode`, then leaves the alternate screen
/// and shows the cursor.
///
/// # Errors
///
/// An I/O error of the terminal.
pub fn leave(out: &mut impl Write, grapheme_mode: bool) -> io::Result<()> {
    for mode in MOUSE.into_iter().rev() {
        write!(out, "{}", private_mode(mode, false))?;
    }
    if grapheme_mode {
        write!(
            out,
            "{}",
            private_mode(DecPrivateModeCode::GraphemeClustering, false)
        )?;
    }
    write!(
        out,
        "{}{}",
        private_mode(DecPrivateModeCode::ClearAndEnableAlternateScreen, false),
        private_mode(DecPrivateModeCode::ShowCursor, true),
    )?;
    out.flush()
}

/// The result of a step and the restore after it.
///
/// # Errors
///
/// The error of the step, with the error of the restore in its message if both failed. If only
/// the restore failed, its error.
pub fn finish<T>(result: io::Result<T>, restore: io::Result<()>) -> io::Result<T> {
    match (result, restore) {
        (Err(error), Err(restore)) => Err(io::Error::new(
            error.kind(),
            format!("{error} (the restore also failed: {restore})"),
        )),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(restore)) => Err(restore),
        (Ok(value), Ok(())) => Ok(value),
    }
}

/// Enters raw mode and the alternate screen and turns on mouse capture. Sets a panic hook that
/// undoes each, and turns off mode 2027 if `grapheme_mode` holds true. On an error, the terminal
/// leaves each mode that this turned on.
///
/// # Errors
///
/// An I/O error of the terminal, or a terminal that gives no size.
pub fn open(grapheme_mode: &Arc<AtomicBool>) -> io::Result<(Screen, EventReader)> {
    let mut platform = PlatformTerminal::new()?;
    platform.enter_raw_mode()?;
    let hook_mode = Arc::clone(grapheme_mode);
    platform.set_panic_hook(move |out| {
        // The panic hook has no caller to report a failed write to.
        let _ = leave(out, hook_mode.load(Ordering::Relaxed));
    });
    let reader = platform.event_reader();
    // The ratatui terminal reads the size, so it comes before the first mode.
    let mut screen = Terminal::new(TerminaBackend::new(platform))?;
    let out = screen.backend_mut();
    let entered = enter(out);
    let left = if entered.is_ok() {
        Ok(())
    } else {
        leave(out, false)
    };
    finish(entered, left)?;
    Ok((screen, reader))
}

/// Enters the alternate screen and turns on mouse capture.
fn enter(out: &mut impl Write) -> io::Result<()> {
    write!(
        out,
        "{}",
        private_mode(DecPrivateModeCode::ClearAndEnableAlternateScreen, true)
    )?;
    for mode in MOUSE {
        write!(out, "{}", private_mode(mode, true))?;
    }
    out.flush()
}

/// The size of one cell in pixels, from the size of the window. None if the terminal gives no
/// pixel size.
#[must_use]
pub fn cell_pixels(size: &WindowSize) -> Option<Size> {
    let (cells, pixels) = (size.columns_rows, size.pixels);
    if cells.width == 0 || cells.height == 0 {
        return None;
    }
    let cell = Size::new(pixels.width / cells.width, pixels.height / cells.height);
    (cell.width > 0 && cell.height > 0).then_some(cell)
}

/// The size of one cell of `screen` in pixels. None if the terminal gives no pixel size.
///
/// # Errors
///
/// An I/O error of the window size query.
pub fn cell(screen: &mut Screen) -> io::Result<Option<Size>> {
    Backend::window_size(screen.backend_mut()).map(|size| cell_pixels(&size))
}
