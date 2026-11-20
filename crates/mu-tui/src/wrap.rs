//! The wrap of one line to a width in cells.
//!
//! The rows of a line are byte ranges that hold every byte of the line, in order. A row breaks
//! after a space where it can, and inside a word only where the word is wider than the row. A
//! space that does not fit at the end of a row hangs there: it belongs to the row but takes no
//! cell. A grapheme wider than the width takes a row of its own. Every line gives at least one
//! row, and a width of 0 gives no rows.

use unicode_segmentation::UnicodeSegmentation;

use crate::width::Profile;

/// One row of a wrapped line: the bytes `start..end` of the line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    /// The first byte of the row.
    pub start: usize,
    /// One past the last byte of the row.
    pub end: usize,
}

struct Wrapper<'a> {
    width: usize,
    row_start: usize,
    /// The cells that the row takes so far.
    col: usize,
    /// The last place to break: the byte after a space, and the cells before it.
    space_break: Option<(usize, usize)>,
    rows: &'a mut Vec<Row>,
}

impl Wrapper<'_> {
    fn end_row(&mut self, end: usize) {
        self.rows.push(Row {
            start: self.row_start,
            end,
        });
        self.row_start = end;
        self.space_break = None;
    }

    fn space(&mut self, at: usize, len: usize) {
        if self.col < self.width {
            self.col += 1;
        }
        self.space_break = Some((at + len, self.col));
    }

    fn grapheme(&mut self, at: usize, len: usize, cells: usize) {
        if cells > self.width {
            if at > self.row_start {
                self.end_row(at);
            }
            self.end_row(at + len);
            self.col = 0;
            return;
        }
        if self.col + cells > self.width {
            if let Some((after_space, cells_before)) = self
                .space_break
                .filter(|(after, _)| *after > self.row_start)
            {
                self.end_row(after_space);
                self.col -= cells_before;
            }
            if self.col + cells > self.width {
                self.end_row(at);
                self.col = 0;
            }
        }
        self.col += cells;
    }
}

/// Writes the rows of `text` at `width` cells to `rows`, in place of what `rows` held, with the
/// widths of `profile`.
pub fn wrap(text: &str, width: u16, profile: &Profile, rows: &mut Vec<Row>) {
    rows.clear();
    if width == 0 {
        return;
    }
    let width = usize::from(width);
    if text.is_ascii() && text.len() <= width {
        rows.push(Row {
            start: 0,
            end: text.len(),
        });
        return;
    }
    let mut wrapper = Wrapper {
        width,
        row_start: 0,
        col: 0,
        space_break: None,
        rows,
    };
    for (at, grapheme) in text.grapheme_indices(true) {
        if grapheme == " " {
            wrapper.space(at, grapheme.len());
        } else {
            wrapper.grapheme(at, grapheme.len(), profile.width(grapheme));
        }
    }
    if wrapper.row_start < text.len() || wrapper.rows.is_empty() {
        wrapper.end_row(text.len());
    }
}
