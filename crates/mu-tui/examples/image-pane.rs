//! Shows an output pane with text lines and one image from a PNG file, for the visual check of
//! the kitty placeholders in a real terminal.
//!
//! `cargo run -p mu-tui --example image-pane -- <file.png>`
//!
//! The up and down arrows scroll one row, the page keys one page, and q, Esc or Ctrl+C leave. A
//! terminal with no kitty graphics shows the path of the file in place of the image.

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, fs};

use mu_tui::screen::{self, Screen};
use mu_tui::store::Image;
use mu_tui::{Caps, OutputPane, PaneState, kitty, probe};
use ratatui::layout::Size;
use termina::event::{KeyCode, KeyEventKind, Modifiers};
use termina::{Event, EventReader};

/// The id of the one image.
const ID: u32 = 1;

/// The cell size in pixels when the terminal does not report its pixel size.
const CELL: Size = Size::new(10, 20);

/// The width and the height in pixels from the IHDR chunk of a PNG file.
fn png_size(png: &[u8]) -> Option<(u32, u32)> {
    let header = png.get(..24)?;
    if !header.starts_with(b"\x89PNG\r\n\x1a\n") || &header[12..16] != b"IHDR" {
        return None;
    }
    let number = |at: usize| Some(u32::from_be_bytes(header.get(at..at + 4)?.try_into().ok()?));
    Some((number(16)?, number(20)?))
}

fn cells(pixels: u32, cell: u16) -> u16 {
    u16::try_from(pixels.div_ceil(u32::from(cell.max(1)))).unwrap_or(u16::MAX)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("usage: image-pane <file.png>")?;
    let png = fs::read(&path)?;
    let size = png_size(&png).ok_or("not a PNG file")?;
    let grapheme_mode = Arc::new(AtomicBool::new(true));
    let (mut terminal, reader) = screen::open(&grapheme_mode)?;
    let result = session(&mut terminal, &reader, &grapheme_mode, &path, &png, size);
    screen::leave(
        terminal.backend_mut(),
        grapheme_mode.load(Ordering::Relaxed),
    )?;
    result
}

/// Probes the terminal, shows the pane with the image until the user leaves, and deletes the
/// image. After the pane shows, the error is the one of `show`, the one of the delete, or both.
/// The caller restores the terminal after it, also after an error.
fn session(
    terminal: &mut Screen,
    reader: &EventReader,
    grapheme_mode: &AtomicBool,
    path: &str,
    png: &[u8],
    (width, height): (u32, u32),
) -> Result<(), Box<dyn std::error::Error>> {
    let cell = screen::cell(terminal)?;
    let (outcome, _input) = probe::run(terminal.backend_mut(), probe::WAIT)?;
    grapheme_mode.store(outcome.resets_grapheme_mode(), Ordering::Relaxed);
    let graphics = outcome.graphics;
    let size = cell.unwrap_or(CELL);
    let image = Image::new(ID, cells(height, size.height), cells(width, size.width))
        .ok_or("the image id is out of range")?;

    let mut pane = PaneState::with_caps(
        Caps {
            profile: outcome.profile,
            graphics,
            cell,
        }
        .into(),
    );
    for n in 1..=200 {
        pane.push(&format!("line {n} before the image"));
    }
    pane.push_image(path, image);
    for n in 1..=5 {
        pane.push(&format!("line {n} after the image"));
    }
    let out = terminal.backend_mut();
    if graphics {
        out.write_all(&kitty::transmit(image, png))?;
        out.flush()?;
    }
    let shown = show(terminal, reader, &mut pane);
    let deleted = if graphics { delete(terminal) } else { Ok(()) };
    match (shown, deleted) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error.into()),
        (Err(shown), Err(deleted)) => {
            Err(format!("{shown}; the delete of the image also failed: {deleted}").into())
        }
    }
}

fn delete(terminal: &mut Screen) -> io::Result<()> {
    let out = terminal.backend_mut();
    write!(out, "\x1b_Ga=d,d=I,i={ID},q=2\x1b\\")?;
    out.flush()
}

fn show(terminal: &mut Screen, reader: &EventReader, pane: &mut PaneState) -> io::Result<()> {
    loop {
        terminal.draw(|frame| frame.render_stateful_widget(OutputPane, frame.area(), pane))?;
        match reader.read(|_| true)? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') | KeyCode::Escape => return Ok(()),
                KeyCode::Char('c') if key.modifiers.contains(Modifiers::CONTROL) => return Ok(()),
                KeyCode::Up => pane.scroll_rows(-1),
                KeyCode::Down => pane.scroll_rows(1),
                KeyCode::PageUp => pane.scroll_pages(-1),
                KeyCode::PageDown => pane.scroll_pages(1),
                _ => {}
            },
            _ => {}
        }
    }
}
