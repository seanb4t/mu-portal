//! The output pane: a widget, and its state that the model keeps by pane id.

use std::collections::VecDeque;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget};

/// The lines that a pane keeps.
const KEPT_LINES: usize = 2000;

/// The lines of one output pane.
#[derive(Debug, Default)]
pub struct PaneState {
    lines: VecDeque<String>,
}

impl PaneState {
    /// Adds `line` at the bottom. The oldest line goes when the pane is full.
    pub fn push(&mut self, line: String) {
        if self.lines.len() == KEPT_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    /// Removes every line.
    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

/// Draws the newest lines of a [`PaneState`] that fit, one row each.
#[derive(Debug, Default, Clone, Copy)]
pub struct OutputPane;

impl StatefulWidget for OutputPane {
    type State = PaneState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut PaneState) {
        let skip = state.lines.len().saturating_sub(usize::from(area.height));
        let shown: Vec<Line> = state
            .lines
            .iter()
            .skip(skip)
            .map(|line| Line::raw(line.as_str()))
            .collect();
        Paragraph::new(shown).render(area, buf);
    }
}
