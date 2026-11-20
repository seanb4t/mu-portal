//! Shows an output pane with text lines and one image from a PNG file, for the visual check of
//! the kitty placeholders in a real terminal.
//!
//! `cargo run -p mu-tui --example image-pane -- <file.png>`
//!
//! The up and down arrows scroll one row, the page keys one page, and q, Esc or Ctrl+C leave. A
//! terminal with no kitty graphics shows the path of the file in place of the image.

use std::io::{self, Write};
use std::{env, fs};

use mu_tui::store::Image;
use mu_tui::{Caps, OutputPane, PaneState, kitty, probe};
use ratatui::Terminal;
use ratatui::backend::TerminaBackend;
use termina::escape::csi::{Csi, DecPrivateMode, DecPrivateModeCode, Mode};
use termina::event::{KeyCode, KeyEventKind, Modifiers};
use termina::{Event, EventReader, PlatformTerminal, Terminal as _, WindowSize};

/// The id of the one image.
const ID: u32 = 1;

/// The cell size in pixels when the terminal does not report its pixel size.
const CELL: (u32, u32) = (10, 20);

/// The width and the height in pixels from the IHDR chunk of a PNG file.
fn png_size(png: &[u8]) -> Option<(u32, u32)> {
    let header = png.get(..24)?;
    if !header.starts_with(b"\x89PNG\r\n\x1a\n") || &header[12..16] != b"IHDR" {
        return None;
    }
    let number = |at: usize| Some(u32::from_be_bytes(header.get(at..at + 4)?.try_into().ok()?));
    Some((number(16)?, number(20)?))
}

/// The pixels of one cell, from the window size.
fn cell_size(size: &WindowSize) -> (u32, u32) {
    match (size.pixel_width, size.pixel_height) {
        (Some(width), Some(height)) if size.cols > 0 && size.rows > 0 && width > 0 => (
            u32::from(width) / u32::from(size.cols),
            u32::from(height) / u32::from(size.rows),
        ),
        _ => CELL,
    }
}

fn cells(pixels: u32, cell: u32) -> u16 {
    u16::try_from(pixels.div_ceil(cell.max(1))).unwrap_or(u16::MAX)
}

fn private_mode(code: DecPrivateModeCode, on: bool) -> Csi {
    let mode = DecPrivateMode::Code(code);
    Csi::Mode(if on {
        Mode::SetDecPrivateMode(mode)
    } else {
        Mode::ResetDecPrivateMode(mode)
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("usage: image-pane <file.png>")?;
    let png = fs::read(&path)?;
    let (width, height) = png_size(&png).ok_or("not a PNG file")?;

    let mut platform = PlatformTerminal::new()?;
    platform.enter_raw_mode()?;
    platform.set_panic_hook(|out| {
        // The panic hook has no caller to report a failed write to.
        let _ = write!(
            out,
            "{}",
            private_mode(DecPrivateModeCode::ClearAndEnableAlternateScreen, false)
        );
    });
    write!(
        platform,
        "{}",
        private_mode(DecPrivateModeCode::ClearAndEnableAlternateScreen, true)
    )?;
    platform.flush()?;
    let (cell_width, cell_height) = cell_size(&platform.get_dimensions()?);
    let (outcome, _input) = probe::run(&mut platform, probe::WAIT)?;
    let graphics = outcome.graphics;
    let image = Image::new(ID, cells(height, cell_height), cells(width, cell_width));

    let mut pane = PaneState::with_caps(
        Caps {
            profile: outcome.profile,
            graphics,
        }
        .into(),
    );
    for n in 1..=40 {
        pane.push(&format!("line {n} before the image"));
    }
    pane.push_image(&path, image);
    for n in 1..=5 {
        pane.push(&format!("line {n} after the image"));
    }
    if graphics {
        platform.write_all(&kitty::transmit(image, &png))?;
        platform.flush()?;
    }

    let reader = platform.event_reader();
    let mut terminal = Terminal::new(TerminaBackend::new(platform))?;
    let result = show(&mut terminal, &reader, &mut pane);
    let out = terminal.backend_mut();
    if graphics {
        write!(out, "\x1b_Ga=d,d=I,i={ID},q=2\x1b\\")?;
    }
    write!(
        out,
        "{}{}",
        private_mode(DecPrivateModeCode::ClearAndEnableAlternateScreen, false),
        private_mode(DecPrivateModeCode::ShowCursor, true),
    )?;
    out.flush()?;
    Ok(result?)
}

fn show(
    terminal: &mut Terminal<TerminaBackend<PlatformTerminal>>,
    reader: &EventReader,
    pane: &mut PaneState,
) -> io::Result<()> {
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
