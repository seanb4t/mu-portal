//! The viewport: which rows of which lines are on screen.
//!
//! The scroll position is an anchor: a line id and a byte in it (the row index
//! for an image line). The top row of the screen is the row that holds the
//! anchor byte. A resize re-wraps only the lines on screen, and the same text
//! stays at the top. No global row index exists.

use crate::store::LineStore;
use crate::wrap::{Row, wrap};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos {
    pub line: usize,
    pub byte: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibleRow {
    pub line: usize,
    pub row: u16,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug)]
pub struct Viewport {
    pub top: Pos,
    /// The newest line stays at the bottom while this is set.
    pub follow: bool,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            top: Pos::default(),
            follow: true,
        }
    }
}

fn anchor_of(rows: &[Row], index: usize, image: bool) -> u32 {
    if image {
        index as u32
    } else {
        rows[index].start
    }
}

fn row_of(rows: &[Row], byte: u32, image: bool) -> usize {
    if image {
        (byte as usize).min(rows.len() - 1)
    } else {
        rows.partition_point(|r| r.start <= byte).saturating_sub(1)
    }
}

impl Viewport {
    /// Fills `out` with the rows on screen, at most `height` of them.
    pub fn layout(
        &mut self,
        store: &LineStore,
        width: u16,
        height: u16,
        out: &mut Vec<VisibleRow>,
        scratch: &mut Vec<Row>,
    ) {
        out.clear();
        let h = usize::from(height);
        if store.is_empty() || h == 0 {
            return;
        }
        if !self.follow {
            self.top.line = self.top.line.max(store.first_id());
            if self.top.line < store.end_id() {
                let mut id = self.top.line;
                let l = store.line(id);
                wrap(&l, width, scratch);
                let mut k = row_of(scratch, self.top.byte, l.image.is_some());
                loop {
                    for (i, r) in scratch.iter().enumerate().skip(k) {
                        if out.len() == h {
                            return;
                        }
                        out.push(VisibleRow {
                            line: id,
                            row: i as u16,
                            start: r.start,
                            end: r.end,
                        });
                    }
                    id += 1;
                    if id == store.end_id() {
                        break;
                    }
                    wrap(&store.line(id), width, scratch);
                    k = 0;
                }
                if out.len() == h {
                    return;
                }
            }
            out.clear();
            self.follow = true;
        }
        let mut id = store.end_id();
        while out.len() < h && id > store.first_id() {
            id -= 1;
            wrap(&store.line(id), width, scratch);
            for (i, r) in scratch.iter().enumerate().rev() {
                out.push(VisibleRow {
                    line: id,
                    row: i as u16,
                    start: r.start,
                    end: r.end,
                });
                if out.len() == h {
                    break;
                }
            }
        }
        out.reverse();
        let t = out[0];
        let image = store.line(t.line).image.is_some();
        self.top = Pos {
            line: t.line,
            byte: if image { u32::from(t.row) } else { t.start },
        };
    }

    /// Moves the top of the screen by `delta` rows. Down past the end turns on follow.
    pub fn scroll(&mut self, store: &LineStore, width: u16, delta: isize, scratch: &mut Vec<Row>) {
        if store.is_empty() || delta == 0 {
            return;
        }
        if delta > 0 && self.follow {
            return;
        }
        self.follow = false;
        let mut id = self.top.line.clamp(store.first_id(), store.end_id() - 1);
        let l = store.line(id);
        wrap(&l, width, scratch);
        let mut image = l.image.is_some();
        let mut k = row_of(scratch, self.top.byte, image);
        if delta < 0 {
            for _ in 0..delta.unsigned_abs() {
                if k > 0 {
                    k -= 1;
                } else if id > store.first_id() {
                    id -= 1;
                    let l = store.line(id);
                    image = l.image.is_some();
                    wrap(&l, width, scratch);
                    k = scratch.len() - 1;
                } else {
                    break;
                }
            }
        } else {
            for _ in 0..delta {
                if k + 1 < scratch.len() {
                    k += 1;
                } else if id + 1 < store.end_id() {
                    id += 1;
                    let l = store.line(id);
                    image = l.image.is_some();
                    wrap(&l, width, scratch);
                    k = 0;
                } else {
                    break;
                }
            }
        }
        self.top = Pos {
            line: id,
            byte: anchor_of(scratch, k, image),
        };
    }

    pub fn home(&mut self, store: &LineStore) {
        self.follow = false;
        self.top = Pos {
            line: store.first_id(),
            byte: 0,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(n: usize) -> LineStore {
        let mut s = LineStore::new();
        for i in 0..n {
            s.push_text(
                &format!("line {i} has a few words that wrap at narrow widths"),
                &[],
            );
        }
        s
    }

    #[test]
    fn follow_shows_the_newest_line_at_the_bottom() {
        let s = store(100);
        let mut vp = Viewport::default();
        let (mut out, mut scratch) = (Vec::new(), Vec::new());
        vp.layout(&s, 20, 10, &mut out, &mut scratch);
        assert_eq!(out.len(), 10);
        assert_eq!(out.last().map(|r| r.line), Some(99));
    }

    #[test]
    fn the_top_text_survives_a_resize() {
        let s = store(100);
        let mut vp = Viewport::default();
        let (mut out, mut scratch) = (Vec::new(), Vec::new());
        vp.layout(&s, 20, 10, &mut out, &mut scratch);
        vp.scroll(&s, 20, -50, &mut scratch);
        vp.layout(&s, 20, 10, &mut out, &mut scratch);
        let before = out[0];
        let byte = before.start;
        for width in [13, 40, 7, 120, 20] {
            vp.layout(&s, width, 10, &mut out, &mut scratch);
            let top = out[0];
            assert_eq!(top.line, before.line, "{width}");
            assert!(
                top.start <= byte && byte <= top.end.max(top.start),
                "{width}"
            );
        }
    }

    #[test]
    fn scrolling_down_past_the_end_turns_on_follow() {
        let s = store(30);
        let mut vp = Viewport::default();
        let (mut out, mut scratch) = (Vec::new(), Vec::new());
        vp.layout(&s, 20, 10, &mut out, &mut scratch);
        vp.scroll(&s, 20, -5, &mut scratch);
        assert!(!vp.follow);
        vp.scroll(&s, 20, 100, &mut scratch);
        vp.layout(&s, 20, 10, &mut out, &mut scratch);
        assert!(vp.follow);
    }

    #[test]
    fn scrolled_view_stays_put_while_lines_arrive() {
        let mut s = store(50);
        let mut vp = Viewport::default();
        let (mut out, mut scratch) = (Vec::new(), Vec::new());
        vp.layout(&s, 20, 10, &mut out, &mut scratch);
        vp.scroll(&s, 20, -40, &mut scratch);
        vp.layout(&s, 20, 10, &mut out, &mut scratch);
        let before = out.clone();
        for _ in 0..20 {
            s.push_text("new line", &[]);
        }
        vp.layout(&s, 20, 10, &mut out, &mut scratch);
        assert_eq!(out, before);
    }
}
