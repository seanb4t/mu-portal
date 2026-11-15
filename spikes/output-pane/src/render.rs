//! The two renderers under test: a ratatui widget, and direct writes of a
//! full frame of escape sequences.

use std::io::Write as _;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style as RStyle};
use ratatui::widgets::Widget;

use crate::cells::for_each_cell;
use crate::store::LineStore;
use crate::style::{Style, rgb};
use crate::view::{Pos, VisibleRow};

fn color(c: Option<u32>) -> Color {
    c.map_or(Color::Reset, |c| {
        let (r, g, b) = rgb(c);
        Color::Rgb(r, g, b)
    })
}

fn to_ratatui(s: Style, selected: bool) -> RStyle {
    let mut m = Modifier::empty();
    m.set(Modifier::BOLD, s.bold);
    m.set(Modifier::ITALIC, s.italic);
    m.set(Modifier::UNDERLINED, s.underline);
    m.set(Modifier::REVERSED, selected);
    RStyle::new()
        .fg(color(s.fg))
        .bg(color(s.bg))
        .add_modifier(m)
}

pub struct Pane<'a> {
    pub store: &'a LineStore,
    pub rows: &'a [VisibleRow],
    pub sel: Option<(Pos, Pos)>,
}

impl Widget for Pane<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (y, row) in self.rows.iter().enumerate().take(usize::from(area.height)) {
            let y = area.y + y as u16;
            for_each_cell(self.store, row, area.width, self.sel, |c| {
                buf[(area.x + c.x, y)]
                    .set_symbol(c.symbol)
                    .set_style(to_ratatui(c.style, c.selected));
            });
        }
    }
}

/// Direct writes: every frame repaints every row inside a synchronized update.
#[derive(Default)]
pub struct Direct {
    pub out: Vec<u8>,
}

fn sgr(out: &mut Vec<u8>, s: Style, selected: bool) {
    out.extend_from_slice(b"\x1b[0");
    if s.bold {
        out.extend_from_slice(b";1");
    }
    if s.italic {
        out.extend_from_slice(b";3");
    }
    if s.underline {
        out.extend_from_slice(b";4");
    }
    if selected {
        out.extend_from_slice(b";7");
    }
    if let Some(c) = s.fg {
        let (r, g, b) = rgb(c);
        let _ = write!(out, ";38;2;{r};{g};{b}");
    }
    if let Some(c) = s.bg {
        let (r, g, b) = rgb(c);
        let _ = write!(out, ";48;2;{r};{g};{b}");
    }
    out.push(b'm');
}

impl Direct {
    pub fn frame(
        &mut self,
        store: &LineStore,
        rows: &[VisibleRow],
        width: u16,
        height: u16,
        sel: Option<(Pos, Pos)>,
        status: &str,
    ) -> &[u8] {
        let out = &mut self.out;
        out.clear();
        out.extend_from_slice(b"\x1b[?2026h");
        for y in 0..height {
            let _ = write!(out, "\x1b[{};1H\x1b[0m", y + 1);
            if let Some(row) = rows.get(usize::from(y)) {
                let mut cur = (Style::default(), false);
                for_each_cell(store, row, width, sel, |c| {
                    if (c.style, c.selected) != cur {
                        cur = (c.style, c.selected);
                        sgr(out, c.style, c.selected);
                    }
                    out.extend_from_slice(c.symbol.as_bytes());
                });
                out.extend_from_slice(b"\x1b[0m");
            }
            out.extend_from_slice(b"\x1b[K");
        }
        let status: String = status.chars().take(usize::from(width)).collect();
        let _ = write!(out, "\x1b[{};1H\x1b[0;7m{status}\x1b[K\x1b[0m", height + 1);
        out.extend_from_slice(b"\x1b[?2026l");
        &self.out
    }
}
