//! Word wrap of one logical line to a width in cells.
//!
//! A row is a byte range of the line text. A row breaks after the last space
//! that fits; a word wider than the row breaks at the grapheme that overflows.
//! A space at a break is dropped. A wide grapheme never straddles two rows.
//! An image line gives `rows` empty rows at any width.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::store::Line;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub start: u32,
    pub end: u32,
}

struct Wrapper<'o> {
    width: u32,
    row_start: u32,
    col: u32,
    /// The last break: start of its space, start of the next row, column after the space.
    brk: Option<(u32, u32, u32)>,
    out: &'o mut Vec<Row>,
}

impl Wrapper<'_> {
    fn feed(&mut self, at: u32, w: u32, space: bool, len: u32) {
        while w > 0 && self.col + w > self.width {
            if space {
                self.out.push(Row {
                    start: self.row_start,
                    end: at,
                });
                self.row_start = at + len;
                self.col = 0;
                self.brk = None;
                return;
            }
            match self.brk.take() {
                Some((end, next, bc)) if end > self.row_start => {
                    self.out.push(Row {
                        start: self.row_start,
                        end,
                    });
                    self.row_start = next;
                    self.col -= bc;
                }
                _ => {
                    self.out.push(Row {
                        start: self.row_start,
                        end: at,
                    });
                    self.row_start = at;
                    self.col = 0;
                }
            }
        }
        self.col += w;
        if space {
            self.brk = Some((at, at + len, self.col));
        }
    }
}

pub fn wrap(line: &Line<'_>, width: u16, out: &mut Vec<Row>) {
    out.clear();
    if let Some(img) = line.image {
        out.extend((0..img.rows).map(|_| Row { start: 0, end: 0 }));
        return;
    }
    let width = u32::from(width.max(2));
    let len = line.text.len() as u32;
    if line.ascii && line.width <= width {
        out.push(Row { start: 0, end: len });
        return;
    }
    let mut w = Wrapper {
        width,
        row_start: 0,
        col: 0,
        brk: None,
        out,
    };
    if line.ascii {
        for (i, &b) in line.text.as_bytes().iter().enumerate() {
            w.feed(i as u32, 1, b == b' ', 1);
        }
    } else {
        for (i, g) in line.text.grapheme_indices(true) {
            w.feed(i as u32, g.width() as u32, g == " ", g.len() as u32);
        }
    }
    let start = w.row_start;
    out.push(Row { start, end: len });
}

/// The number of rows a line takes at `width`.
pub fn row_count(line: &Line<'_>, width: u16, scratch: &mut Vec<Row>) -> usize {
    wrap(line, width, scratch);
    scratch.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{ImageRef, LineStore};

    fn rows(text: &str, width: u16) -> Vec<String> {
        let mut s = LineStore::new();
        s.push_text(text, &[]);
        let l = s.line(0);
        let mut out = Vec::new();
        wrap(&l, width, &mut out);
        out.iter()
            .map(|r| text[r.start as usize..r.end as usize].to_owned())
            .collect()
    }

    #[test]
    fn breaks_after_the_last_space_that_fits() {
        assert_eq!(rows("the quick brown fox", 10), ["the quick", "brown fox"]);
    }

    #[test]
    fn breaks_a_word_wider_than_the_row() {
        assert_eq!(rows("abcdefghij kl", 4), ["abcd", "efgh", "ij", "kl"]);
    }

    #[test]
    fn short_line_is_one_row() {
        assert_eq!(rows("hello", 80), ["hello"]);
        assert_eq!(rows("", 80), [""]);
    }

    #[test]
    fn wide_grapheme_never_straddles_rows() {
        let r = rows("ab漢字cd", 5);
        assert_eq!(r, ["ab漢", "字cd"]);
        for row in &r {
            assert!(row.width() <= 5);
        }
    }

    #[test]
    fn every_row_fits_and_no_text_is_lost() {
        let text = "a dragon 🐉 lands, 漢字 and words wrap around the edges of the pane again";
        for width in 2..40 {
            let r = rows(text, width);
            assert!(
                r.iter().all(|row| row.width() <= usize::from(width)),
                "{width}"
            );
            let kept: String = r.concat().split_whitespace().collect();
            let want: String = text.split_whitespace().collect();
            assert_eq!(kept, want, "{width}");
        }
    }

    #[test]
    fn image_line_takes_its_rows_at_any_width() {
        let mut s = LineStore::new();
        s.push_image(ImageRef {
            id: 1,
            cols: 30,
            rows: 6,
        });
        let mut out = Vec::new();
        assert_eq!(row_count(&s.line(0), 10, &mut out), 6);
        assert_eq!(row_count(&s.line(0), 200, &mut out), 6);
    }
}
