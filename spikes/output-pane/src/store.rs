//! The line store: logical lines in fixed-size chunks.
//!
//! A chunk holds the text of its lines in one `String`, the style spans in one
//! `Vec`, and one 16-byte record per line. Line ids are absolute and never
//! reused, so a position stays valid after the oldest chunks are dropped.
//! Text must not hold control characters: ingest expands tabs and strips the rest.

use std::collections::{HashMap, VecDeque};

use unicode_width::UnicodeWidthStr;

use crate::style::Style;

pub const CHUNK_LINES: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageRef {
    pub id: u32,
    pub cols: u16,
    pub rows: u16,
}

/// A style change at `start`, a byte offset in the line.
#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub start: u32,
    pub style: u16,
}

#[derive(Clone, Copy)]
struct Record {
    text_end: u32,
    span_end: u32,
    width: u32,
    flags: u8,
}

const ASCII: u8 = 1;
const IMAGE: u8 = 2;

#[derive(Default)]
struct Chunk {
    text: String,
    spans: Vec<Span>,
    records: Vec<Record>,
    images: Vec<(u32, ImageRef)>,
}

impl Chunk {
    fn bytes(&self) -> usize {
        self.text.capacity()
            + self.spans.capacity() * size_of::<Span>()
            + self.records.capacity() * size_of::<Record>()
            + self.images.capacity() * size_of::<(u32, ImageRef)>()
    }
}

#[derive(Clone, Copy)]
pub struct Line<'a> {
    pub text: &'a str,
    pub spans: &'a [Span],
    pub width: u32,
    pub ascii: bool,
    pub image: Option<ImageRef>,
}

#[derive(Default)]
pub struct LineStore {
    chunks: VecDeque<Chunk>,
    first: usize,
    len: usize,
    styles: Vec<Style>,
    style_ids: HashMap<Style, u16>,
}

impl LineStore {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.intern(Style::default());
        s
    }

    /// The id of the oldest stored line.
    pub fn first_id(&self) -> usize {
        self.first
    }

    /// One past the id of the newest line.
    pub fn end_id(&self) -> usize {
        self.first + self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn style(&self, id: u16) -> Style {
        self.styles[usize::from(id)]
    }

    fn intern(&mut self, style: Style) -> u16 {
        if let Some(&id) = self.style_ids.get(&style) {
            return id;
        }
        let id = u16::try_from(self.styles.len()).expect("style table full");
        self.styles.push(style);
        self.style_ids.insert(style, id);
        id
    }

    fn tail(&mut self) -> &mut Chunk {
        if self
            .chunks
            .back()
            .is_none_or(|c| c.records.len() == CHUNK_LINES)
        {
            if let Some(full) = self.chunks.back_mut() {
                full.text.shrink_to_fit();
                full.spans.shrink_to_fit();
                full.images.shrink_to_fit();
            }
            let mut c = Chunk::default();
            c.records.reserve_exact(CHUNK_LINES);
            self.chunks.push_back(c);
        }
        self.chunks.back_mut().expect("tail chunk")
    }

    /// Appends a text line. `spans` are (byte offset, style) pairs in order.
    pub fn push_text(&mut self, text: &str, spans: &[(usize, Style)]) {
        debug_assert!(!text.chars().any(char::is_control));
        let ids: Vec<(u32, u16)> = spans
            .iter()
            .map(|&(at, st)| (u32::try_from(at).expect("line too long"), self.intern(st)))
            .collect();
        let ascii = text.is_ascii();
        let width = if ascii { text.len() } else { text.width() };
        let chunk = self.tail();
        chunk.text.push_str(text);
        chunk
            .spans
            .extend(ids.into_iter().map(|(start, style)| Span { start, style }));
        chunk.records.push(Record {
            text_end: u32::try_from(chunk.text.len()).expect("chunk text over 4 GiB"),
            span_end: u32::try_from(chunk.spans.len()).expect("too many spans"),
            width: u32::try_from(width).expect("line too wide"),
            flags: if ascii { ASCII } else { 0 },
        });
        self.len += 1;
    }

    /// Appends a line that shows an image. It takes `image.rows` rows at any width.
    pub fn push_image(&mut self, image: ImageRef) {
        let chunk = self.tail();
        let index = u32::try_from(chunk.records.len()).expect("chunk index");
        chunk.images.push((index, image));
        chunk.records.push(Record {
            text_end: u32::try_from(chunk.text.len()).expect("chunk text over 4 GiB"),
            span_end: u32::try_from(chunk.spans.len()).expect("too many spans"),
            width: u32::from(image.cols),
            flags: IMAGE,
        });
        self.len += 1;
    }

    pub fn line(&self, id: usize) -> Line<'_> {
        let rel = id - self.first;
        let chunk = &self.chunks[rel / CHUNK_LINES];
        let i = rel % CHUNK_LINES;
        let rec = chunk.records[i];
        let (text_start, span_start) = match i {
            0 => (0, 0),
            _ => {
                let p = chunk.records[i - 1];
                (p.text_end as usize, p.span_end as usize)
            }
        };
        let image = if rec.flags & IMAGE == 0 {
            None
        } else {
            let at = chunk
                .images
                .binary_search_by_key(&(i as u32), |&(k, _)| k)
                .expect("image record");
            Some(chunk.images[at].1)
        };
        Line {
            text: &chunk.text[text_start..rec.text_end as usize],
            spans: &chunk.spans[span_start..rec.span_end as usize],
            width: rec.width,
            ascii: rec.flags & ASCII != 0,
            image,
        }
    }

    /// Drops whole chunks from the front while more than `max_lines` remain.
    pub fn trim(&mut self, max_lines: usize) {
        while self.len - self.chunks[0].records.len() >= max_lines {
            let c = self.chunks.pop_front().expect("front chunk");
            self.first += c.records.len();
            self.len -= c.records.len();
        }
    }

    /// The heap bytes the store holds, by capacity.
    pub fn bytes(&self) -> usize {
        self.chunks.iter().map(Chunk::bytes).sum::<usize>()
            + self.chunks.capacity() * size_of::<Chunk>()
            + self.styles.capacity() * size_of::<Style>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_across_chunk_boundaries_keep_text_and_spans() {
        let mut s = LineStore::new();
        let red = Style::fg(0xff0000);
        for i in 0..CHUNK_LINES + 3 {
            s.push_text(&format!("line {i}"), &[(5, red)]);
        }
        let l = s.line(CHUNK_LINES + 1);
        assert_eq!(l.text, format!("line {}", CHUNK_LINES + 1));
        assert_eq!(l.spans.len(), 1);
        assert_eq!(s.style(l.spans[0].style), red);
        assert_eq!(
            s.line(CHUNK_LINES - 1).text,
            format!("line {}", CHUNK_LINES - 1)
        );
    }

    #[test]
    fn image_line_carries_its_reference() {
        let mut s = LineStore::new();
        s.push_text("before", &[]);
        let img = ImageRef {
            id: 7,
            cols: 10,
            rows: 4,
        };
        s.push_image(img);
        s.push_text("after", &[]);
        assert_eq!(s.line(1).image, Some(img));
        assert_eq!(s.line(2).text, "after");
        assert_eq!(s.line(2).image, None);
    }

    #[test]
    fn trim_keeps_absolute_ids() {
        let mut s = LineStore::new();
        for i in 0..CHUNK_LINES * 3 {
            s.push_text(&i.to_string(), &[]);
        }
        s.trim(CHUNK_LINES + 10);
        assert_eq!(s.first_id(), CHUNK_LINES);
        assert_eq!(s.end_id(), CHUNK_LINES * 3);
        assert_eq!(s.line(CHUNK_LINES * 2).text, (CHUNK_LINES * 2).to_string());
    }
}
