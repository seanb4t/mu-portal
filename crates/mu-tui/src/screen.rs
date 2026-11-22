//! The terminal of the client: raw mode, the alternate screen, mouse capture and bracketed paste,
//! the sequences that undo them, and [`finish`], which keeps the first error when a restore also fails.

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use ratatui::Terminal;
use ratatui::backend::{Backend, TerminaBackend, WindowSize};
use ratatui::layout::Size;
use termina::escape::csi::{
    Csi, DecPrivateMode, DecPrivateModeCode, Keyboard, KittyKeyboardFlags, Mode,
};
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

/// The modes that the client set after the alternate screen, which the exit must undo. The loop
/// and the panic hook share one value.
#[derive(Debug, Default)]
pub struct Restore {
    grapheme_mode: AtomicBool,
    keyboard_flag: AtomicBool,
}

impl Restore {
    /// Sets whether the exit turns off mode 2027.
    pub fn set_grapheme_mode(&self, on: bool) {
        self.grapheme_mode.store(on, Ordering::Relaxed);
    }

    /// Sets whether the exit pops one entry of the kitty keyboard flags.
    pub fn set_keyboard_flag(&self, on: bool) {
        self.keyboard_flag.store(on, Ordering::Relaxed);
    }
}

/// Pushes the kitty keyboard flag that disambiguates escape codes, and records it in `restore`
/// so the exit pops it. With the flag, the terminal reports a key such as Shift+Enter with its
/// modifiers.
///
/// # Errors
///
/// An I/O error of the terminal.
pub fn push_keyboard_flag(out: &mut impl Write, restore: &Restore) -> io::Result<()> {
    // The record comes first, so a failed write still gets its pop.
    restore.set_keyboard_flag(true);
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES;
    write!(out, "{}", Csi::Keyboard(Keyboard::PushFlags(flags)))?;
    out.flush()
}

/// Turns off mouse capture and bracketed paste, and each mode of `restore`, then leaves the
/// alternate screen and shows the cursor.
///
/// # Errors
///
/// An I/O error of the terminal.
pub fn leave(out: &mut impl Write, restore: &Restore) -> io::Result<()> {
    for mode in MOUSE.into_iter().rev() {
        write!(out, "{}", private_mode(mode, false))?;
    }
    write!(
        out,
        "{}",
        private_mode(DecPrivateModeCode::BracketedPaste, false)
    )?;
    if restore.grapheme_mode.load(Ordering::Relaxed) {
        write!(
            out,
            "{}",
            private_mode(DecPrivateModeCode::GraphemeClustering, false)
        )?;
    }
    // The terminal keeps the flags of the alternate screen apart, so the pop comes before the
    // client leaves that screen.
    // The pop is not idempotent, so one push gets one pop, also if this runs twice.
    if restore.keyboard_flag.swap(false, Ordering::Relaxed) {
        write!(out, "{}", Csi::Keyboard(Keyboard::PopFlags(1)))?;
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

/// Enters raw mode and the alternate screen and turns on mouse capture and bracketed paste. Sets
/// a panic hook that undoes each, and each mode of `restore`. On an error, the terminal leaves
/// each mode that this turned on.
///
/// # Errors
///
/// An I/O error of the terminal, or a terminal that gives no size.
pub fn open(restore: &Arc<Restore>) -> io::Result<(Screen, EventReader)> {
    let mut platform = PlatformTerminal::new()?;
    platform.enter_raw_mode()?;
    let hook_restore = Arc::clone(restore);
    platform.set_panic_hook(move |out| {
        // The panic hook has no caller to report a failed write to.
        let _ = leave(out, &hook_restore);
    });
    let reader = platform.event_reader();
    // The ratatui terminal reads the size, so it comes before the first mode.
    let mut screen = Terminal::new(TerminaBackend::new(platform))?;
    let out = screen.backend_mut();
    let entered = enter(out);
    let left = if entered.is_ok() {
        Ok(())
    } else {
        leave(out, &Restore::default())
    };
    finish(entered, left)?;
    Ok((screen, reader))
}

/// Enters the alternate screen and turns on mouse capture and bracketed paste.
///
/// # Errors
///
/// An I/O error of the terminal.
pub fn enter(out: &mut impl Write) -> io::Result<()> {
    write!(
        out,
        "{}",
        private_mode(DecPrivateModeCode::ClearAndEnableAlternateScreen, true)
    )?;
    for mode in MOUSE {
        write!(out, "{}", private_mode(mode, true))?;
    }
    write!(
        out,
        "{}",
        private_mode(DecPrivateModeCode::BracketedPaste, true)
    )?;
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
