//! The layout tree: the screen as data, which [`view`](crate::view) walks.

use ratatui::layout::{Constraint, Direction};

use crate::editor::EditorId;
use crate::model::PaneId;

/// One node of the layout tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// Areas side by side or one above the other. Each child takes the area of its size.
    Split {
        /// The direction of the split.
        direction: Direction,
        /// The children, in order, each with its size.
        children: Vec<(Size, Node)>,
    },
    /// One tab for each world. It shows the output pane of the world in focus, or the pane of
    /// notices when no world has the focus.
    Tabs,
    /// One view.
    Leaf(Leaf),
}

/// The size of a child of a split.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    /// The size of a constraint.
    Fixed(Constraint),
    /// One row for each wrapped row of the text of an editor, and at least one row. It is at
    /// most `max_rows`, and at most half of the rows that the split divides. In a split of
    /// columns, it is the width that the other children leave.
    Editor {
        /// The editor whose text gives the rows.
        id: EditorId,
        /// The most rows.
        max_rows: u16,
    },
}

/// The most rows of the pose editor in the default layout.
pub const POSE_EDITOR_ROWS: u16 = 8;

/// A view that a leaf of the layout tree names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leaf {
    /// The row of world tabs.
    WorldTabs,
    /// The output pane with this id.
    Pane(PaneId),
    /// The editor with this id.
    Editor(EditorId),
}

impl Default for Node {
    /// The world tabs on the top row, the pane of the world in focus, and the pose editor on
    /// the bottom rows.
    fn default() -> Self {
        Self::Split {
            direction: Direction::Vertical,
            children: vec![
                (
                    Size::Fixed(Constraint::Length(1)),
                    Self::Leaf(Leaf::WorldTabs),
                ),
                (Size::Fixed(Constraint::Min(0)), Self::Tabs),
                (
                    Size::Editor {
                        id: EditorId::POSE,
                        max_rows: POSE_EDITOR_ROWS,
                    },
                    Self::Leaf(Leaf::Editor(EditorId::POSE)),
                ),
            ],
        }
    }
}
