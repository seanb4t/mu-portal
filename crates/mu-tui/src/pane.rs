//! The output pane: a widget, and its state that the model keeps by pane id.
//!
//! The state holds the line store, the scroll anchor and the selection. The scroll position is
//! an anchor, a line and a byte: the top row is the row that holds the anchor byte. A pane with
//! no anchor is at the bottom and follows new lines. Each render stores the map from the screen
//! rows to the lines, so a mouse position turns into a line and a byte.

use std::num::NonZeroU16;
use std::sync::Arc;

use ratatui::buffer::{Buffer, CellDiffOption};
use ratatui::layout::{Rect, Size};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::convert::Converted;
use crate::diacritics::DIACRITICS;
use crate::store::{CAPACITY, Image, LineId, LineStore};
use crate::text::clean;
use crate::width::{Class, Profile};
use crate::wrap::{Row, wrap};

/// A place in the text of a pane: a line and a byte in that line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    /// The line.
    pub line: LineId,
    /// The byte offset in the text of the line.
    pub byte: usize,
}

/// What one row of the screen showed at the last render.
#[derive(Clone, Copy, Debug)]
enum Drawn {
    /// The bytes of `row` in `line`.
    Text { line: LineId, row: Row },
    /// The unfinished line.
    Unfinished,
}

#[derive(Clone, Copy, Debug)]
struct DrawnRow {
    y: u16,
    drawn: Drawn,
}

/// The text area at the last render, with no scroll bar column and no unfinished row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Geometry {
    x: u16,
    width: u16,
    height: u16,
}

/// What the terminal can draw, as the startup probe learned it. Every pane shares it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Caps {
    /// The widths of the graphemes.
    pub profile: Profile,
    /// True if the terminal draws kitty images.
    pub graphics: bool,
    /// The size of one cell in pixels, if the terminal gives it.
    pub cell: Option<Size>,
}

/// The kitty unicode placeholder.
const PLACEHOLDER: char = '\u{10EEEE}';

/// The state of one output pane.
#[derive(Debug)]
pub struct PaneState {
    store: LineStore,
    /// The top row holds this byte. None at the bottom.
    anchor: Option<Position>,
    /// The two ends of the selection, in the order of the drag.
    selection: Option<(Position, Position)>,
    /// The cell of the press of a drag that has not ended.
    pressed: Option<Position>,
    unfinished: Option<Converted>,
    caps: Arc<Caps>,
    drawn: Vec<DrawnRow>,
    geometry: Option<Geometry>,
}

impl Default for PaneState {
    fn default() -> Self {
        Self::new(CAPACITY)
    }
}

impl PaneState {
    /// An empty pane that keeps at most `capacity` lines.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            store: LineStore::new(capacity),
            anchor: None,
            selection: None,
            pressed: None,
            unfinished: None,
            caps: Arc::default(),
            drawn: Vec::new(),
            geometry: None,
        }
    }

    /// The lines of the pane.
    #[must_use]
    pub const fn store(&self) -> &LineStore {
        &self.store
    }

    /// Removes the unfinished line, then adds a line with no style at the bottom.
    pub fn push(&mut self, text: &str) {
        self.push_styled(text, &[]);
    }

    /// Removes the unfinished line, then adds a line at the bottom. `spans` are pairs of a byte
    /// offset and a style, in order. If the store drops its oldest line, an anchor, a press or a
    /// selection end on that line moves to the start of the oldest live line, and a selection
    /// with both ends on dropped lines goes.
    pub fn push_styled(&mut self, text: &str, spans: &[(usize, Style)]) {
        self.unfinished = None;
        self.store.push(text, spans);
        self.keep_live();
    }

    /// Removes the unfinished line, then adds an image line at the bottom, as
    /// [`Self::push_styled`] does. The pane shows each control character of `url` as U+FFFD.
    pub fn push_image(&mut self, url: &str, image: Image) {
        self.unfinished = None;
        self.store.push_image(&clean(url), image);
        self.keep_live();
    }

    /// Drops the `count` oldest lines, or every line if the pane holds fewer, as the store
    /// does at its capacity.
    pub fn trim(&mut self, count: usize) {
        self.store.trim(count);
        self.keep_live();
    }

    /// Moves each position on a line that the store dropped to the oldest live line.
    fn keep_live(&mut self) {
        let first = self.store.first_id();
        let keep = |position: Position| {
            if position.line < first {
                Position {
                    line: first,
                    byte: 0,
                }
            } else {
                position
            }
        };
        self.anchor = self.anchor.map(keep);
        self.pressed = self.pressed.map(keep);
        self.selection = match self.selection {
            Some((from, to)) if from.line < first && to.line < first => None,
            Some((from, to)) => Some((keep(from), keep(to))),
            None => None,
        };
    }

    /// Removes every line, the unfinished line, the selection and the scroll position.
    pub fn clear(&mut self) {
        let caps = Arc::clone(&self.caps);
        *self = Self::new(self.store.capacity());
        self.caps = caps;
    }

    /// An empty pane that wraps and draws for a terminal with `caps`.
    #[must_use]
    pub fn with_caps(caps: Arc<Caps>) -> Self {
        Self {
            caps,
            ..Self::default()
        }
    }

    /// Wraps and draws for a terminal with `caps`.
    pub fn set_caps(&mut self, caps: Arc<Caps>) {
        self.caps = caps;
    }

    /// Shows `text` with its styles as the unfinished line after the newest line, or no
    /// unfinished line.
    pub fn set_unfinished(&mut self, text: Option<Converted>) {
        self.unfinished = text;
    }

    /// The scroll anchor: the position that the top row holds. None at the bottom.
    #[must_use]
    pub const fn anchor(&self) -> Option<Position> {
        self.anchor
    }

    /// The two ends of the selection, in the order of the drag.
    #[must_use]
    pub const fn selection(&self) -> Option<(Position, Position)> {
        self.selection
    }

    /// Selects from `from` to `to`.
    pub fn select(&mut self, from: Position, to: Position) {
        self.selection = Some((from, to));
    }

    /// Starts a drag at `position`, the cell under the press, with an empty selection there.
    pub fn press(&mut self, position: Position) {
        self.pressed = Some(position);
        self.selection = Some((position, position));
    }

    /// Moves the end of the drag to `position`, the cell under the pointer. The selection holds
    /// the graphemes of both cells and every grapheme between them. A drag with no press does
    /// nothing.
    pub fn drag(&mut self, position: Position) {
        let Some(pressed) = self.pressed else {
            return;
        };
        self.selection = Some(if position >= pressed {
            (pressed, self.after(position))
        } else {
            (self.after(pressed), position)
        });
    }

    /// Ends the drag. Gives the copy text of a selection that is not empty. A press with no drag
    /// clears the selection.
    pub fn release(&mut self) -> Option<String> {
        self.pressed = None;
        match self.selection {
            Some((from, to)) if from != to => self.copy_text(),
            _ => {
                self.selection = None;
                None
            }
        }
    }

    /// The position after the grapheme at `position`, or `position` at the end of its line.
    fn after(&self, position: Position) -> Position {
        let line = self.store.line(position.line);
        if let Some(line) = line.filter(|line| line.image.is_some()) {
            return Position {
                byte: line.text.len(),
                ..position
            };
        }
        let len = line
            .and_then(|line| line.text.get(position.byte..))
            .and_then(|rest| rest.graphemes(true).next())
            .map_or(0, str::len);
        Position {
            byte: position.byte + len,
            ..position
        }
    }

    /// The selected text of each line, with one LF between lines and no style. The text of an
    /// image line is its whole URL.
    #[must_use]
    pub fn copy_text(&self) -> Option<String> {
        let (a, b) = self.selection?;
        let (from, to) = if a <= b { (a, b) } else { (b, a) };
        let mut copy = String::new();
        for id in from.line.0..=to.line.0 {
            let line = self.store.line(LineId(id))?;
            let text = line.text;
            let whole = line.image.is_some();
            let start = if id == from.line.0 && !whole {
                from.byte
            } else {
                0
            };
            let end = if id == to.line.0 && !whole {
                to.byte
            } else {
                text.len()
            };
            if id > from.line.0 {
                copy.push('\n');
            }
            copy.push_str(text.get(start.min(end)..end)?);
        }
        Some(copy)
    }

    /// Removes the selection and ends a drag.
    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.pressed = None;
    }

    /// Returns to the bottom, so the pane follows new lines again.
    pub fn to_bottom(&mut self) {
        self.anchor = None;
    }

    /// The rows of `line` at `width`. With kitty graphics, the rows of an image line are its
    /// row indexes, `n..n + 1` for the row `n`. Without, an image line is one row of its URL.
    fn rows_of(&self, line: LineId, width: u16) -> Vec<Row> {
        let mut rows = Vec::new();
        let Some(text) = self.store.line(line).filter(|_| width > 0) else {
            return rows;
        };
        match text.image {
            Some(image) if self.caps.graphics => {
                rows.extend((0..usize::from(image.rows())).map(|n| Row {
                    start: n,
                    end: n + 1,
                }));
            }
            Some(_) => rows.push(Row {
                start: 0,
                end: text.text.len(),
            }),
            None => wrap(text.text, width, &self.caps.profile, &mut rows),
        }
        rows
    }

    /// The top row as a line and the index of a row in that line, at `geometry`.
    fn top_row(&self, geometry: Geometry) -> Option<(LineId, usize)> {
        if self.store.is_empty() || geometry.width == 0 {
            return None;
        }
        if let Some(anchor) = self.anchor {
            let rows = self.rows_of(anchor.line, geometry.width);
            return Some((anchor.line, row_index(&rows, anchor.byte)));
        }
        let mut need = usize::from(geometry.height).max(1);
        let mut line = self.store.end_id().0;
        while line > self.store.first_id().0 {
            line -= 1;
            let count = self.rows_of(LineId(line), geometry.width).len();
            if count >= need {
                return Some((LineId(line), count - need));
            }
            need -= count;
        }
        Some((self.store.first_id(), 0))
    }

    /// True if, with `top` as the top row, the last row of the newest line is on the screen.
    fn bottom_shows(&self, top: (LineId, usize), geometry: Geometry) -> bool {
        let mut left = usize::from(geometry.height);
        let (mut line, mut index) = top;
        while line < self.store.end_id() {
            let count = self.rows_of(line, geometry.width).len();
            let take = count.saturating_sub(index);
            if take > left {
                return false;
            }
            left -= take;
            line = LineId(line.0 + 1);
            index = 0;
        }
        true
    }

    /// Scrolls by `delta` rows at the size of the last render: back for a negative `delta`,
    /// forward for a positive one. A scroll that brings the last row of the newest line onto the
    /// screen returns the pane to the bottom.
    pub fn scroll_rows(&mut self, delta: i32) {
        let Some(geometry) = self.geometry.filter(|g| g.height > 0) else {
            return;
        };
        let Some((mut line, mut index)) = self.top_row(geometry) else {
            return;
        };
        let mut rows = self.rows_of(line, geometry.width);
        for _ in 0..delta.unsigned_abs() {
            if delta < 0 {
                if index > 0 {
                    index -= 1;
                } else if line > self.store.first_id() {
                    line = LineId(line.0 - 1);
                    rows = self.rows_of(line, geometry.width);
                    index = rows.len().saturating_sub(1);
                } else {
                    break;
                }
            } else if index + 1 < rows.len() {
                index += 1;
            } else if line.0 + 1 < self.store.end_id().0 {
                line = LineId(line.0 + 1);
                rows = self.rows_of(line, geometry.width);
                index = 0;
            } else {
                break;
            }
        }
        self.anchor = if self.bottom_shows((line, index), geometry) {
            None
        } else {
            Some(Position {
                line,
                byte: rows.get(index).map_or(0, |row| row.start),
            })
        };
    }

    /// Scrolls by `pages` pages at the size of the last render, as [`Self::scroll_rows`] does. A
    /// page is the height of the text area less one row, and at least one row.
    pub fn scroll_pages(&mut self, pages: i32) {
        let Some(geometry) = self.geometry else {
            return;
        };
        let step = i32::from(geometry.height.saturating_sub(1).max(1));
        self.scroll_rows(pages.saturating_mul(step));
    }

    /// The position of the screen cell `x`, `y` at the last render. A cell of a wide grapheme
    /// gives the start of that grapheme, and a cell past the end of the text gives the end of
    /// the row. A cell of the scroll bar or of the unfinished line gives none. A row of a line
    /// that the pane dropped after the render gives the start of the oldest live line.
    #[must_use]
    pub fn position(&self, x: u16, y: u16) -> Option<Position> {
        let geometry = self.geometry?;
        if x < geometry.x || x - geometry.x >= geometry.width {
            return None;
        }
        let drawn = self.drawn.iter().find(|row| row.y == y)?;
        let Drawn::Text { line, row } = drawn.drawn else {
            return None;
        };
        if line < self.store.first_id() {
            return Some(Position {
                line: self.store.first_id(),
                byte: 0,
            });
        }
        let text = self.store.line(line)?;
        if text.image.is_some() && self.caps.graphics {
            return Some(Position { line, byte: 0 });
        }
        let text = text.text;
        let column = usize::from(x - geometry.x);
        let mut cells = 0;
        for (at, grapheme) in text.get(row.start..row.end)?.grapheme_indices(true) {
            let width = cell_width(&self.caps.profile, grapheme, geometry.width);
            if column < cells + width {
                return Some(Position {
                    line,
                    byte: row.start + at,
                });
            }
            cells += width;
        }
        Some(Position {
            line,
            byte: row.end,
        })
    }
}

/// The index of the row of `rows` that holds `byte`, or the last row.
fn row_index(rows: &[Row], byte: usize) -> usize {
    rows.iter()
        .position(|row| byte >= row.start && byte < row.end)
        .unwrap_or_else(|| rows.len().saturating_sub(1))
}

/// The cells that `grapheme` takes in a row `width` cells wide: one for a grapheme wider than
/// the row, which shows as U+FFFD.
fn cell_width(profile: &Profile, grapheme: &str, width: u16) -> usize {
    match profile.width(grapheme) {
        cells if cells > usize::from(width) => 1,
        cells => cells,
    }
}

/// The span that holds `byte`: the last span of `spans` that starts at or before it. `spans` are
/// in the order of their start, and `start` gives the start of a span.
fn span_at<T>(spans: &[T], byte: usize, start: impl Fn(&T) -> usize) -> Option<&T> {
    spans[..spans.partition_point(|span| start(span) <= byte)].last()
}

/// True if `position` is inside `selection`, from its first end up to its last end.
fn selected(selection: Option<(Position, Position)>, position: Position) -> bool {
    selection.is_some_and(|(a, b)| {
        let (from, to) = if a <= b { (a, b) } else { (b, a) };
        position >= from && position < to
    })
}

/// Draws a [`PaneState`]: its rows from the anchor, or the newest rows at the bottom, a scroll
/// bar in the last column when the rows do not fit, and the unfinished line on the last row.
#[derive(Debug, Default, Clone, Copy)]
pub struct OutputPane;

impl OutputPane {
    /// The rows to show from the top row, at most `geometry.height`.
    fn rows(state: &PaneState, geometry: Geometry) -> Vec<(LineId, Row)> {
        let mut shown = Vec::new();
        let Some((mut line, mut index)) = state.top_row(geometry) else {
            return shown;
        };
        let height = usize::from(geometry.height);
        while shown.len() < height && line < state.store.end_id() {
            let rows = state.rows_of(line, geometry.width);
            shown.extend(rows.iter().skip(index).map(|row| (line, *row)));
            line = LineId(line.0 + 1);
            index = 0;
        }
        shown.truncate(height);
        shown
    }

    /// True if every row fits at `geometry` with no anchor.
    fn fits(state: &PaneState, geometry: Geometry) -> bool {
        state.anchor.is_none() && Self::all_fit(state, geometry)
    }

    /// True if the rows of every line fit at `geometry`.
    fn all_fit(state: &PaneState, geometry: Geometry) -> bool {
        let mut left = usize::from(geometry.height);
        for line in (state.store.first_id().0..state.store.end_id().0).rev() {
            let count = state.rows_of(LineId(line), geometry.width).len();
            if count > left {
                return false;
            }
            left -= count;
        }
        true
    }

    fn draw_row(
        state: &PaneState,
        line: LineId,
        row: Row,
        at: (u16, u16),
        width: u16,
        buf: &mut Buffer,
    ) {
        let Some(text) = state.store.line(line) else {
            return;
        };
        if let Some(image) = text.image.filter(|_| state.caps.graphics) {
            draw_placeholders(image, row.start, at, width, buf);
            return;
        }
        let Some(row_text) = text.text.get(row.start..row.end) else {
            return;
        };
        let style_at = |offset: usize| {
            let byte = row.start + offset;
            let style = span_at(text.spans, byte, |span| span.start)
                .map_or_else(Style::default, |span| state.store.style(span.style));
            if selected(state.selection, Position { line, byte }) {
                style.add_modifier(Modifier::REVERSED)
            } else {
                style
            }
        };
        draw_text(row_text, style_at, at, width, &state.caps.profile, buf);
    }
}

/// Draws the row `row` of `image` from `at` as kitty placeholders, in at most `width` cells.
/// Each cell holds the diacritic of its row and of its column, and its foreground color is the
/// image id.
fn draw_placeholders(image: Image, row: usize, at: (u16, u16), width: u16, buf: &mut Buffer) {
    let (x, y) = at;
    let [_, red, green, blue] = image.id().to_be_bytes();
    let row_mark = DIACRITICS[row.min(DIACRITICS.len() - 1)];
    let mut symbol = String::new();
    for (column, column_mark) in DIACRITICS
        .iter()
        .take(usize::from(image.columns().min(width)))
        .enumerate()
    {
        symbol.clear();
        symbol.extend([PLACEHOLDER, row_mark, *column_mark]);
        let cell_x = x + u16::try_from(column).unwrap_or(u16::MAX);
        buf[(cell_x, y)]
            .set_symbol(&symbol)
            .set_style(Style::default().fg(Color::Rgb(red, green, blue)));
    }
}

/// Draws the graphemes of `text` from `at`, in at most `width` cells, with the widths of
/// `profile`. `style_at` gives the style of the grapheme at a byte offset of `text`. A grapheme
/// wider than `width` shows as U+FFFD. A grapheme of a risky class, or one whose learned width
/// differs from its `unicode-width` value, gets its width as a forced width. The cells after the
/// first cell of a wide grapheme are reset. A grapheme of no cell is not drawn.
fn draw_text(
    text: &str,
    style_at: impl Fn(usize) -> Style,
    at: (u16, u16),
    width: u16,
    profile: &Profile,
    buf: &mut Buffer,
) {
    let (x, y) = at;
    let mut cells = 0;
    for (offset, grapheme) in text.grapheme_indices(true) {
        let grapheme_cells = cell_width(profile, grapheme, width);
        if grapheme_cells == 0 {
            continue;
        }
        if cells + grapheme_cells > usize::from(width) {
            break;
        }
        let (symbol, forced) = if profile.width(grapheme) > usize::from(width) {
            ("\u{fffd}", None)
        } else if Class::of(grapheme).risky() || grapheme_cells != grapheme.width() {
            let forced = u16::try_from(grapheme_cells).ok().and_then(NonZeroU16::new);
            (grapheme, forced)
        } else {
            (grapheme, None)
        };
        let cell_x = x + u16::try_from(cells).unwrap_or(u16::MAX);
        let cell = &mut buf[(cell_x, y)];
        cell.set_symbol(symbol).set_style(style_at(offset));
        if let Some(forced) = forced {
            cell.set_diff_option(CellDiffOption::ForcedWidth(forced));
        }
        for extra in 1..grapheme_cells {
            buf[(cell_x + u16::try_from(extra).unwrap_or(u16::MAX), y)].reset();
        }
        cells += grapheme_cells;
    }
}

impl StatefulWidget for OutputPane {
    type State = PaneState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut PaneState) {
        state.drawn.clear();
        let unfinished_row = u16::from(state.unfinished.is_some() && area.height > 0);
        let full = Geometry {
            x: area.x,
            width: area.width,
            height: area.height - unfinished_row,
        };
        // A resize can bring the last row of the newest line onto the screen. The pane is then
        // at the bottom, so it follows new lines again. With an anchor, the rows are drawn beside
        // the scroll bar. An area with no column shows no row, so it keeps the anchor.
        if state.anchor.is_some() && full.width > 0 {
            let beside_bar = Geometry {
                width: full.width.saturating_sub(1),
                ..full
            };
            let bottom = state
                .top_row(beside_bar)
                .is_some_and(|top| state.bottom_shows(top, beside_bar));
            if bottom || Self::all_fit(state, full) {
                state.anchor = None;
            }
        }
        let bar = area.width > 0 && !Self::fits(state, full);
        let geometry = Geometry {
            width: full.width - u16::from(bar),
            ..full
        };
        state.geometry = Some(geometry);
        for (offset, (line, row)) in Self::rows(state, geometry).into_iter().enumerate() {
            let y = area.y + u16::try_from(offset).unwrap_or(u16::MAX);
            Self::draw_row(state, line, row, (area.x, y), geometry.width, buf);
            state.drawn.push(DrawnRow {
                y,
                drawn: Drawn::Text { line, row },
            });
        }
        if let Some(unfinished) = state.unfinished.as_ref().filter(|_| unfinished_row == 1) {
            let y = area.y + full.height;
            let style_at = |offset: usize| {
                span_at(&unfinished.spans, offset, |(start, _)| *start)
                    .map_or_else(Style::default, |(_, style)| *style)
            };
            draw_text(
                &unfinished.text,
                style_at,
                (area.x, y),
                area.width,
                &state.caps.profile,
                buf,
            );
            state.drawn.push(DrawnRow {
                y,
                drawn: Drawn::Unfinished,
            });
        }
        if bar {
            let top = state.top_row(geometry).map_or(0, |(line, _)| {
                usize::try_from(line.0 - state.store.first_id().0).unwrap_or(usize::MAX)
            });
            let mut bar_state = ScrollbarState::new(state.store.len()).position(top);
            let bar_area = Rect {
                height: full.height,
                ..area
            };
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .render(bar_area, buf, &mut bar_state);
        }
    }
}
