//! The layout tree: the screen as data, which [`view`](crate::view) walks.

use ratatui::layout::{Constraint, Direction};

use crate::model::PaneId;

/// One node of the layout tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// Areas side by side or one above the other. Each child takes the area of its constraint.
    Split {
        /// The direction of the split.
        direction: Direction,
        /// One constraint for each child.
        constraints: Vec<Constraint>,
        /// The children, in order.
        children: Vec<Node>,
    },
    /// One tab for each world. It shows the output pane of the world in focus, or the pane of
    /// notices when no world has the focus.
    Tabs,
    /// One view.
    Leaf(Leaf),
}

/// A view that a leaf of the layout tree names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leaf {
    /// The row of world tabs.
    WorldTabs,
    /// The output pane with this id.
    Pane(PaneId),
    /// The input line.
    Input,
}

impl Default for Node {
    /// The world tabs on the top row, the pane of the world in focus, and the input line on
    /// the bottom row.
    fn default() -> Self {
        Self::Split {
            direction: Direction::Vertical,
            constraints: vec![
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ],
            children: vec![
                Self::Leaf(Leaf::WorldTabs),
                Self::Tabs,
                Self::Leaf(Leaf::Input),
            ],
        }
    }
}
