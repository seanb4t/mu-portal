//! [`view`]: draws the model by walking its layout tree.

use std::collections::HashMap;

use ratatui::Frame;
use ratatui::layout::{Layout, Rect};
use ratatui::widgets::Paragraph;
use ratatui_textarea::TextArea;

use crate::layout::{Leaf, Node};
use crate::model::{Model, PaneId, World};
use crate::pane::{OutputPane, PaneState};
use crate::text::clean;

/// The parts of the model that the walk reads, apart from the tree, so the walk can borrow
/// the pane states mutably while it reads the tree.
struct Parts<'a> {
    worlds: &'a [World],
    focus: Option<usize>,
    lost: bool,
    input: &'a TextArea<'static>,
    panes: &'a mut HashMap<PaneId, PaneState>,
    focused_pane: PaneId,
}

/// Draws `model` on `frame`. A pane render can store what it drew in its state.
pub fn view(model: &mut Model, frame: &mut Frame) {
    let focused_pane = model.focused_pane();
    let Model {
        worlds,
        focus,
        panes,
        input,
        lost,
        layout,
        ..
    } = model;
    let mut parts = Parts {
        worlds,
        focus: *focus,
        lost: *lost,
        input,
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
            let constraints = children.iter().map(|(constraint, _)| *constraint);
            let areas = Layout::new(*direction, constraints).split(area);
            for ((_, child), child_area) in children.iter().zip(areas.iter()) {
                render(child, *child_area, frame, parts);
            }
        }
        Node::Tabs => render_pane(parts.focused_pane, area, frame, parts),
        Node::Leaf(Leaf::WorldTabs) => frame.render_widget(Paragraph::new(top_row(parts)), area),
        Node::Leaf(Leaf::Pane(pane)) => render_pane(*pane, area, frame, parts),
        Node::Leaf(Leaf::Input) => frame.render_widget(parts.input, area),
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
                if parts.focus == Some(index) {
                    format!("[{id}]{mark}")
                } else {
                    format!("{id}{mark}")
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
