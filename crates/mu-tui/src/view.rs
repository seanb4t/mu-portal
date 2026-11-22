//! [`view`]: draws the model by walking its layout tree.

use std::collections::{HashMap, VecDeque};

use crate::editor::{Editor, EditorId, EditorState};
use crate::layout::{Leaf, Node, Size};
use crate::model::{Model, PaneId, World};
use crate::pane::{OutputPane, PaneState};
use crate::text::clean;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Paragraph;

/// The parts of the model that the walk reads, apart from the tree, so the walk can borrow
/// the pane states mutably while it reads the tree.
struct Parts<'a> {
    worlds: &'a [World],
    focus: Option<usize>,
    lost: bool,
    sends: &'a HashMap<String, VecDeque<String>>,
    editors: &'a mut HashMap<EditorId, EditorState>,
    panes: &'a mut HashMap<PaneId, PaneState>,
    focused_pane: PaneId,
}

/// Draws `model` on `frame`. A pane render can store what it drew in its state. The terminal
/// cursor goes to the insertion point of the pose editor.
pub fn view(model: &mut Model, frame: &mut Frame) {
    let focused_pane = model.focused_pane();
    let Model {
        worlds,
        focus,
        panes,
        editors,
        lost,
        layout,
        sends,
        ..
    } = model;
    let mut parts = Parts {
        worlds,
        focus: *focus,
        lost: *lost,
        sends,
        editors,
        panes,
        focused_pane,
    };
    render(layout, frame.area(), frame, &mut parts);
}

fn render(node: &Node, area: Rect, frame: &mut Frame, parts: &mut Parts) {
    match node {
        Node::Split {
            direction,
            children,
        } => {
            let constraints = children
                .iter()
                .map(|(size, _)| constraint(*size, *direction, area, parts));
            let areas = Layout::new(*direction, constraints).split(area);
            for ((_, child), child_area) in children.iter().zip(areas.iter()) {
                render(child, *child_area, frame, parts);
            }
        }
        Node::Tabs => render_pane(parts.focused_pane, area, frame, parts),
        Node::Leaf(Leaf::WorldTabs) => frame.render_widget(Paragraph::new(top_row(parts)), area),
        Node::Leaf(Leaf::Pane(pane)) => render_pane(*pane, area, frame, parts),
        Node::Leaf(Leaf::Editor(id)) => {
            if let Some(state) = parts.editors.get_mut(id) {
                frame.render_stateful_widget(Editor, area, state);
                // The pose editor has the focus for keys, so the terminal cursor is its cursor.
                if *id == EditorId::POSE
                    && let Some(cursor) = state.cursor()
                {
                    frame.set_cursor_position(cursor);
                }
            }
        }
    }
}

/// The constraint of a child of a split of `area` in `direction`.
fn constraint(size: Size, direction: Direction, area: Rect, parts: &Parts) -> Constraint {
    match size {
        Size::Fixed(constraint) => constraint,
        Size::Editor { id, max_rows } => {
            // The rows of the text give a height, so only a split of rows can use them. In a
            // split of columns, the editor takes the width that the other children leave.
            if direction == Direction::Horizontal {
                return Constraint::Min(0);
            }
            let rows = parts
                .editors
                .get(&id)
                .map_or(1, |state| state.rows(area.width));
            let limit = max_rows.min(area.height / 2).max(1);
            let rows = u16::try_from(rows).unwrap_or(u16::MAX).clamp(1, limit);
            Constraint::Length(rows)
        }
    }
}

fn render_pane(pane: PaneId, area: Rect, frame: &mut Frame, parts: &mut Parts) {
    if let Some(state) = parts.panes.get_mut(&pane) {
        frame.render_stateful_widget(OutputPane, area, state);
    }
}

fn top_row(parts: &Parts) -> String {
    let mut row = if parts.worlds.is_empty() {
        "no worlds".to_owned()
    } else {
        parts
            .worlds
            .iter()
            .enumerate()
            .map(|(index, world)| {
                let mark = if world.unseen { "*" } else { "" };
                let id = clean(&world.id);
                // The first send is with the server. The ones after it wait for its reply.
                let waiting = parts
                    .sends
                    .get(&world.id)
                    .map_or(0, |queue| queue.len().saturating_sub(1));
                let waiting = if waiting == 0 {
                    String::new()
                } else {
                    format!(" ({waiting} waiting)")
                };
                if parts.focus == Some(index) {
                    format!("[{id}]{mark}{waiting}")
                } else {
                    format!("{id}{mark}{waiting}")
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    if parts.lost {
        row.push_str("  disconnected");
    }
    row
}
