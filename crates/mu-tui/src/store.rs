//! The line store of an output pane: the lines of a world in chunks of a fixed number of lines.
//!
//! A line id is absolute and never used again, so a position stays valid after the store drops
//! its oldest lines. A chunk holds the text of its lines in one string, the style spans in one
//! list, and one record for each line.

use std::collections::{HashMap, VecDeque};

use ratatui::style::Style;

use crate::diacritics::DIACRITICS;
use crate::text::clean_styled;

/// The number of lines in one chunk.
pub const CHUNK_LINES: usize = 4096;

/// The number of lines that a pane keeps.
pub const CAPACITY: usize = 100_000;

/// The absolute id of a line. The first line that a store gets has the id 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineId(pub u64);

/// The id of a style in the style table of a store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StyleId(u16);

impl StyleId {
    /// The default style. Every store has it.
    pub const DEFAULT: Self = Self(0);
}

/// A style that starts at `start`, a byte offset in the text of the line, and holds to the next
/// span or to the end of the line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    /// The byte offset where the style starts.
    pub start: usize,
    /// The style.
    pub style: StyleId,
}

/// The most rows and the most columns of an image: the number of kitty diacritics.
pub const IMAGE_CELLS: u16 = 297;

const _: () = assert!(IMAGE_CELLS as usize == DIACRITICS.len());

/// The ids of an image: the foreground color of a placeholder holds the id in 24 bits, and 0
/// means no image.
pub const IMAGE_IDS: std::ops::RangeInclusive<u32> = 1..=0x00FF_FFFF;

/// The kitty image of an image line: its id and its size in cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Image {
    id: u32,
    rows: u16,
    columns: u16,
}

impl Image {
    /// An image with the id `id` of `rows` and `columns` cells, or none for an id outside
    /// [`IMAGE_IDS`]. A size of 0 becomes 1, and a size above [`IMAGE_CELLS`] becomes
    /// [`IMAGE_CELLS`].
    #[must_use]
    pub fn new(id: u32, rows: u16, columns: u16) -> Option<Self> {
        IMAGE_IDS.contains(&id).then(|| Self {
            id,
            rows: rows.clamp(1, IMAGE_CELLS),
            columns: columns.clamp(1, IMAGE_CELLS),
        })
    }

    /// The id, in [`IMAGE_IDS`].
    #[must_use]
    pub const fn id(self) -> u32 {
        self.id
    }

    /// The rows that the image takes.
    #[must_use]
    pub const fn rows(self) -> u16 {
        self.rows
    }

    /// The columns that the image takes.
    #[must_use]
    pub const fn columns(self) -> u16 {
        self.columns
    }
}

#[derive(Clone, Copy, Debug)]
struct Record {
    text_end: usize,
    span_end: usize,
    image: Option<Image>,
}

#[derive(Debug, Default)]
struct Chunk {
    text: String,
    spans: Vec<Span>,
    records: Vec<Record>,
}

/// One line of a store.
#[derive(Clone, Copy, Debug)]
pub struct Line<'a> {
    /// The text, with no control character.
    pub text: &'a str,
    /// The style spans, in the order of their start.
    pub spans: &'a [Span],
    /// The image of an image line, whose text is the URL of the image.
    pub image: Option<Image>,
}

/// The lines of one pane.
#[derive(Debug)]
pub struct LineStore {
    chunks: VecDeque<Chunk>,
    /// The id of the first record of the first chunk.
    base: u64,
    /// The id of the oldest line that the store keeps.
    first: u64,
    /// One past the id of the newest line.
    end: u64,
    capacity: usize,
    styles: Vec<Style>,
    style_ids: HashMap<Style, StyleId>,
}

impl Default for LineStore {
    fn default() -> Self {
        Self::new(CAPACITY)
    }
}

impl LineStore {
    /// A store with no line that keeps at most `capacity` lines. A capacity of 0 counts as 1.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            chunks: VecDeque::new(),
            base: 0,
            first: 0,
            end: 0,
            capacity: capacity.max(1),
            styles: vec![Style::default()],
            style_ids: HashMap::from([(Style::default(), StyleId::DEFAULT)]),
        }
    }

    /// The id of the oldest line that the store keeps. In an empty store, it is the id that the
    /// next line gets.
    #[must_use]
    pub const fn first_id(&self) -> LineId {
        LineId(self.first)
    }

    /// One past the id of the newest line.
    #[must_use]
    pub const fn end_id(&self) -> LineId {
        LineId(self.end)
    }

    /// The most lines that the store keeps.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// The number of lines that the store keeps.
    #[must_use]
    pub fn len(&self) -> usize {
        usize::try_from(self.end - self.first).unwrap_or(usize::MAX)
    }

    /// True if the store keeps no line.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.end == self.first
    }

    /// The style of `id`.
    #[must_use]
    pub fn style(&self, id: StyleId) -> Style {
        self.styles
            .get(usize::from(id.0))
            .copied()
            .unwrap_or_default()
    }

    /// The id of `style`, which this adds to the table if it is new. When the table is full, a
    /// new style gets the default style.
    fn intern(&mut self, style: Style) -> StyleId {
        if let Some(&id) = self.style_ids.get(&style) {
            return id;
        }
        let Ok(index) = u16::try_from(self.styles.len()) else {
            return StyleId::DEFAULT;
        };
        let id = StyleId(index);
        self.styles.push(style);
        self.style_ids.insert(style, id);
        id
    }

    /// Adds a line at the end, and drops the oldest line if the store is then over its
    /// capacity. `spans` are pairs of a byte offset and a style. The store keeps them in the order
    /// of their offset, and keeps `text` as [`crate::text::clean`] gives it.
    pub fn push(&mut self, text: &str, spans: &[(usize, Style)]) -> LineId {
        self.push_line(text, spans, None)
    }

    /// Adds an image line at the end, as [`Self::push`] does, with `url` as its text.
    pub fn push_image(&mut self, url: &str, image: Image) -> LineId {
        self.push_line(url, &[], Some(image))
    }

    fn push_line(&mut self, text: &str, spans: &[(usize, Style)], image: Option<Image>) -> LineId {
        let (text, spans) = clean_styled(text, spans);
        let spans: Vec<Span> = spans
            .iter()
            .map(|&(start, style)| Span {
                start,
                style: self.intern(style),
            })
            .collect();
        if self
            .chunks
            .back()
            .is_none_or(|chunk| chunk.records.len() == CHUNK_LINES)
        {
            self.chunks.push_back(Chunk {
                records: Vec::with_capacity(CHUNK_LINES),
                ..Chunk::default()
            });
        }
        if let Some(chunk) = self.chunks.back_mut() {
            chunk.text.push_str(&text);
            chunk.spans.extend(spans);
            chunk.records.push(Record {
                text_end: chunk.text.len(),
                span_end: chunk.spans.len(),
                image,
            });
        }
        let id = LineId(self.end);
        self.end += 1;
        self.trim(self.len().saturating_sub(self.capacity));
        id
    }

    /// Drops the `count` oldest lines, or every line if the store keeps fewer. Frees each chunk
    /// with no live line. A trim of zero lines or of an empty store changes nothing.
    pub fn trim(&mut self, count: usize) {
        let count = u64::try_from(count).unwrap_or(u64::MAX);
        self.first = self.first.saturating_add(count).min(self.end);
        while let Some(front) = self.chunks.front() {
            let front_end = self.base + front.records.len() as u64;
            if front_end > self.first {
                break;
            }
            self.base = front_end;
            self.chunks.pop_front();
        }
    }

    /// The line `id`, if the store keeps it.
    #[must_use]
    pub fn line(&self, id: LineId) -> Option<Line<'_>> {
        if id.0 < self.first || id.0 >= self.end {
            return None;
        }
        let offset = usize::try_from(id.0 - self.base).ok()?;
        let chunk = self.chunks.get(offset / CHUNK_LINES)?;
        let index = offset % CHUNK_LINES;
        let record = chunk.records.get(index)?;
        let (text_start, span_start) = match index.checked_sub(1) {
            Some(previous) => {
                let previous = chunk.records.get(previous)?;
                (previous.text_end, previous.span_end)
            }
            None => (0, 0),
        };
        Some(Line {
            text: chunk.text.get(text_start..record.text_end)?,
            spans: chunk.spans.get(span_start..record.span_end)?,
            image: record.image,
        })
    }

    /// The lines that the store keeps, oldest first, with their ids.
    pub fn lines(&self) -> impl DoubleEndedIterator<Item = (LineId, Line<'_>)> + '_ {
        (self.first..self.end).filter_map(|id| self.line(LineId(id)).map(|line| (LineId(id), line)))
    }
}
