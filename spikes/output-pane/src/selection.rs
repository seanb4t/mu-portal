//! Selection in logical positions, so a selection survives a re-wrap and a
//! copy gives the game's lines, not the wrapped rows.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::store::LineStore;
use crate::view::{Pos, VisibleRow};

#[derive(Clone, Copy, Debug)]
pub struct Selection {
    pub anchor: Pos,
    pub head: Pos,
}

impl Selection {
    /// The first and last selected grapheme, both included.
    pub fn range(&self) -> (Pos, Pos) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }
}

/// The position under screen cell (`x`, `y`) of the pane.
pub fn hit(store: &LineStore, rows: &[VisibleRow], x: u16, y: u16) -> Option<Pos> {
    let row = rows.get(usize::from(y))?;
    let line = store.line(row.line);
    if line.image.is_some() {
        return Some(Pos {
            line: row.line,
            byte: 0,
        });
    }
    let text = &line.text[row.start as usize..row.end as usize];
    let mut col = 0u16;
    let mut last = row.start;
    for (i, g) in text.grapheme_indices(true) {
        last = row.start + i as u32;
        col += g.width() as u16;
        if col > x {
            break;
        }
    }
    Some(Pos {
        line: row.line,
        byte: last,
    })
}

fn grapheme_end(text: &str, byte: u32) -> usize {
    let b = (byte as usize).min(text.len());
    text[b..].graphemes(true).next().map_or(b, |g| b + g.len())
}

/// The selected text: logical lines joined with `\n`. An image line copies as `[image]`.
pub fn extract(store: &LineStore, sel: &Selection) -> String {
    let (a, b) = sel.range();
    let mut out = String::new();
    for id in a.line.max(store.first_id())..=b.line.min(store.end_id().saturating_sub(1)) {
        if id != a.line {
            out.push('\n');
        }
        let line = store.line(id);
        if line.image.is_some() {
            out.push_str("[image]");
            continue;
        }
        let from = if id == a.line {
            (a.byte as usize).min(line.text.len())
        } else {
            0
        };
        let to = if id == b.line {
            grapheme_end(line.text, b.byte)
        } else {
            line.text.len()
        };
        out.push_str(&line.text[from..to.max(from)]);
    }
    out
}

/// The OSC 52 sequence that puts `text` on the system clipboard.
pub fn osc52(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", STANDARD.encode(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::Viewport;

    #[test]
    fn copy_of_wrapped_rows_gives_logical_lines() {
        let mut s = LineStore::new();
        s.push_text("first line that wraps over several rows", &[]);
        s.push_text("second", &[]);
        let mut vp = Viewport::default();
        let (mut rows, mut scratch) = (Vec::new(), Vec::new());
        vp.layout(&s, 10, 20, &mut rows, &mut scratch);
        let start = hit(&s, &rows, 0, 0).unwrap();
        let last = rows.len() as u16 - 1;
        let end = hit(&s, &rows, 9, last).unwrap();
        let sel = Selection {
            anchor: end,
            head: start,
        };
        assert_eq!(
            extract(&s, &sel),
            "first line that wraps over several rows\nsecond"
        );
    }

    #[test]
    fn hit_maps_a_column_to_the_grapheme_under_it() {
        let mut s = LineStore::new();
        s.push_text("ab漢cd", &[]);
        let mut vp = Viewport::default();
        let (mut rows, mut scratch) = (Vec::new(), Vec::new());
        vp.layout(&s, 20, 1, &mut rows, &mut scratch);
        assert_eq!(hit(&s, &rows, 2, 0).unwrap().byte, 2);
        assert_eq!(hit(&s, &rows, 3, 0).unwrap().byte, 2);
        assert_eq!(hit(&s, &rows, 4, 0).unwrap().byte, 5);
    }

    #[test]
    fn osc52_carries_base64_text() {
        assert_eq!(osc52("hi"), "\x1b]52;c;aGk=\x07");
    }
}
