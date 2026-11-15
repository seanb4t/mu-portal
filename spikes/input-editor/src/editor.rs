use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui_textarea::{TextArea, WrapMode};

/// One input of the client: a soft-wrapped `TextArea` that places the
/// terminal cursor at the insertion point, so an input method draws its
/// preedit text there.
pub struct Editor {
    pub area: TextArea<'static>,
}

impl Editor {
    pub fn new() -> Self {
        let mut area = TextArea::default();
        area.set_wrap_mode(WrapMode::WordOrGlyph);
        area.set_cursor_line_style(Style::default());
        Self { area }
    }

    /// Inserts a paste as one edit. Terminal.app sends each line break of a
    /// bracketed paste as CR.
    pub fn paste(&mut self, text: &str) {
        self.area.insert_str(normalize_paste(text));
    }

    /// Takes the text out of the editor, as Enter does.
    pub fn take(&mut self) -> String {
        let text = self.area.lines().join("\n");
        self.area = {
            let mut fresh = Editor::new();
            fresh
                .area
                .set_block(self.area.block().cloned().unwrap_or_default());
            fresh.area
        };
        text
    }

    /// Draws the editor. A focused editor also places the terminal cursor.
    pub fn draw(&mut self, frame: &mut Frame, rect: Rect, focused: bool) {
        let cursor = if focused {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        self.area.set_cursor_style(cursor);
        frame.render_widget(&self.area, rect);
        if focused {
            frame.set_cursor_position(self.cursor_position(rect));
        }
    }

    /// The screen cell of the insertion point. Valid after a render.
    pub fn cursor_position(&self, rect: Rect) -> Position {
        let inner = self.area.block().map_or(rect, |b| b.inner(rect));
        let screen = self.area.screen_cursor();
        let (top, left) = self.area.scroll_offset();
        let row = screen.row.saturating_sub(top as usize) as u16;
        let col = screen.col.saturating_sub(left as usize) as u16;
        Position::new(inner.x + col, inner.y + row)
    }
}

pub fn normalize_paste(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_paste_keeps_each_line_break_once() {
        assert_eq!(normalize_paste("a\r\nb\rc\nd"), "a\nb\nc\nd");
    }
}
