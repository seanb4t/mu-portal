//! The editor: a text of several lines that the player writes, wraps, pastes and sends. The pose
//! editor is one instance of it.

use std::sync::Arc;

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui_textarea::{CursorMove, TextArea, WrapMode};

use crate::pane::Caps;

/// The id of an editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EditorId(u32);

impl EditorId {
    /// The editor of the text that Enter sends to the world in focus.
    pub const POSE: Self = Self(0);
}

/// The state of one editor.
#[derive(Debug)]
pub struct EditorState {
    textarea: TextArea<'static>,
    /// The area that the last render drew in.
    area: Rect,
}

impl EditorState {
    /// An empty editor for a terminal with `caps`.
    #[must_use]
    pub fn with_caps(caps: &Arc<Caps>) -> Self {
        Self {
            textarea: textarea(&[String::new()], caps),
            area: Rect::default(),
        }
    }

    /// Wraps and draws the editor for a terminal with `caps`.
    pub(crate) fn set_caps(&mut self, caps: &Arc<Caps>) {
        set_widths(&mut self.textarea, caps);
    }

    /// The text of the editor, with an LF between each two lines.
    #[must_use]
    pub fn text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    /// Puts `text` in the editor, with one line for each part between two LF and the cursor at
    /// the end. The history of the edits starts again.
    pub(crate) fn set_text(&mut self, text: &str, caps: &Arc<Caps>) {
        let lines: Vec<String> = text.split('\n').map(str::to_owned).collect();
        self.textarea = textarea(&lines, caps);
    }

    /// The rows that the text takes in an area of `width` columns.
    #[must_use]
    pub fn rows(&self, width: u16) -> usize {
        self.textarea.wrapped_row_count(width)
    }

    /// The cell of the insertion point in the area of the last render. After a row that the text
    /// fills, it is the last column of that row. None if the area has no cell, or the row of the
    /// cursor does not show.
    #[must_use]
    pub fn cursor(&self) -> Option<Position> {
        let last_column = self.area.width.checked_sub(1)?;
        let cursor = self.textarea.screen_cursor();
        let (top_row, _) = self.textarea.scroll_offset();
        let row = u16::try_from(cursor.row).ok()?.checked_sub(top_row)?;
        if row >= self.area.height {
            return None;
        }
        let column = u16::try_from(cursor.col)
            .unwrap_or(u16::MAX)
            .min(last_column);
        Some(Position::new(self.area.x + column, self.area.y + row))
    }

    /// The text area, for an edit.
    pub(crate) fn textarea(&mut self) -> &mut TextArea<'static> {
        &mut self.textarea
    }
}

/// A text area with `lines`, the cursor at the end, the word wrap, and the widths of `caps`.
/// The terminal cursor is the only cursor, so the text area draws none.
fn textarea(lines: &[String], caps: &Arc<Caps>) -> TextArea<'static> {
    let mut textarea = TextArea::new(lines.to_vec());
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default());
    textarea.set_wrap_mode(WrapMode::WordOrGlyph);
    set_widths(&mut textarea, caps);
    textarea.move_cursor(CursorMove::Bottom);
    textarea.move_cursor(CursorMove::End);
    textarea
}

fn set_widths(textarea: &mut TextArea<'static>, caps: &Arc<Caps>) {
    let caps = Arc::clone(caps);
    textarea.set_grapheme_width(move |grapheme| caps.profile.width(grapheme));
}

/// The widget of an editor. Its state is an [`EditorState`].
#[derive(Clone, Copy, Debug, Default)]
pub struct Editor;

impl StatefulWidget for Editor {
    type State = EditorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut EditorState) {
        state.area = area;
        (&state.textarea).render(area, buf);
    }
}
