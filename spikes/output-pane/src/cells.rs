//! The cells of one visible row, shared by both renderers.
//!
//! An image row gives kitty unicode placeholder cells: U+10EEEE with a row
//! diacritic and a column diacritic, and the image id as the 24-bit
//! foreground color. Every cell carries both diacritics, so a renderer that
//! writes only changed cells still places each one.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::diacritics::DIACRITICS;
use crate::store::LineStore;
use crate::style::Style;
use crate::view::{Pos, VisibleRow};

pub const PLACEHOLDER: char = '\u{10EEEE}';

pub struct Cell<'a> {
    pub x: u16,
    pub symbol: &'a str,
    pub style: Style,
    pub selected: bool,
}

/// Writes the placeholder for image cell (`row`, `col`) into `buf`.
pub fn placeholder(row: u16, col: u16, buf: &mut [u8; 16]) -> &str {
    let mut n = PLACEHOLDER.encode_utf8(&mut buf[..]).len();
    n += DIACRITICS[usize::from(row)]
        .encode_utf8(&mut buf[n..])
        .len();
    n += DIACRITICS[usize::from(col)]
        .encode_utf8(&mut buf[n..])
        .len();
    std::str::from_utf8(&buf[..n]).expect("utf-8 placeholder")
}

pub fn for_each_cell(
    store: &LineStore,
    row: &VisibleRow,
    width: u16,
    sel: Option<(Pos, Pos)>,
    mut f: impl FnMut(Cell<'_>),
) {
    let line = store.line(row.line);
    let selected = |byte: u32| {
        sel.is_some_and(|(a, b)| {
            let p = Pos {
                line: row.line,
                byte,
            };
            a <= p && p <= b
        })
    };
    if let Some(img) = line.image {
        let mut buf = [0u8; 16];
        let sel_line = sel.is_some_and(|(a, b)| a.line <= row.line && row.line <= b.line);
        for col in 0..img.cols.min(width).min(DIACRITICS.len() as u16) {
            let symbol = placeholder(row.row, col, &mut buf);
            f(Cell {
                x: col,
                symbol,
                style: Style::fg(img.id),
                selected: sel_line,
            });
        }
        return;
    }
    let spans = line.spans;
    let mut si = spans.partition_point(|s| s.start <= row.start);
    let mut style = if si == 0 {
        Style::default()
    } else {
        store.style(spans[si - 1].style)
    };
    let text = &line.text[row.start as usize..row.end as usize];
    let mut x = 0u16;
    let mut emit = |off: usize, g: &str, w: u16| {
        let byte = row.start + off as u32;
        while si < spans.len() && spans[si].start <= byte {
            style = store.style(spans[si].style);
            si += 1;
        }
        f(Cell {
            x,
            symbol: g,
            style,
            selected: selected(byte),
        });
        x += w;
    };
    if line.ascii {
        for (i, g) in text.char_indices() {
            emit(i, &text[i..i + g.len_utf8()], 1);
        }
    } else {
        for (i, g) in text.grapheme_indices(true) {
            emit(i, g, g.width() as u16);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ImageRef;
    use crate::view::Viewport;

    #[test]
    fn placeholder_encodes_row_and_column() {
        let mut buf = [0u8; 16];
        let s = placeholder(2, 3, &mut buf);
        let chars: Vec<char> = s.chars().collect();
        assert_eq!(chars, [PLACEHOLDER, DIACRITICS[2], DIACRITICS[3]]);
    }

    #[test]
    fn image_row_cells_carry_the_id_and_crop_to_the_width() {
        let mut s = LineStore::new();
        s.push_image(ImageRef {
            id: 0x00_0102,
            cols: 12,
            rows: 3,
        });
        let mut vp = Viewport::default();
        let (mut rows, mut scratch) = (Vec::new(), Vec::new());
        vp.layout(&s, 8, 3, &mut rows, &mut scratch);
        let mut seen = Vec::new();
        for_each_cell(&s, &rows[1], 8, None, |c| {
            seen.push((c.x, c.symbol.to_owned(), c.style.fg))
        });
        assert_eq!(seen.len(), 8);
        let mut buf = [0u8; 16];
        assert_eq!(
            seen[5],
            (5, placeholder(1, 5, &mut buf).to_owned(), Some(0x00_0102))
        );
    }

    #[test]
    fn spans_style_the_right_cells() {
        let mut s = LineStore::new();
        let red = Style::fg(0xff_0000);
        s.push_text("ab cd", &[(3, red)]);
        let mut vp = Viewport::default();
        let (mut rows, mut scratch) = (Vec::new(), Vec::new());
        vp.layout(&s, 20, 1, &mut rows, &mut scratch);
        let mut fg = Vec::new();
        for_each_cell(&s, &rows[0], 20, None, |c| fg.push(c.style.fg));
        assert_eq!(fg, [None, None, None, Some(0xff_0000), Some(0xff_0000)]);
    }
}
