//! The catalog of the blocks of a layout.

use std::fmt;

use crate::names::by_name;

/// One part of the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockKind {
    /// The row of world tabs.
    Tabs,
    /// Lines of text.
    Output,
    /// Typed text.
    Editor,
    /// One row of segments about the world in focus.
    Statusline,
}

/// How a block gets its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeRule {
    /// Always one row. A line gives it no size.
    OneRow,
    /// The rows of its text, up to a `max`. A line gives it only `max`.
    Editor,
    /// A percent, a count of cells, or a share of the rest.
    Free,
}

/// One row of the catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockRow {
    /// The block.
    pub kind: BlockKind,
    /// The name of the block in a sentence.
    pub name: &'static str,
    /// The fewest times a layout holds the block.
    pub min: usize,
    /// The most times a layout holds the block.
    pub max: usize,
    /// How the block gets its size.
    pub size: SizeRule,
    /// Whether the block shares a pane with other blocks as a stack, and shows a title row.
    pub stacks: bool,
}

/// Every block, in the order of the catalog.
pub const BLOCKS: &[BlockRow] = &[
    BlockRow {
        kind: BlockKind::Tabs,
        name: "tabs",
        min: 0,
        max: 1,
        size: SizeRule::OneRow,
        stacks: false,
    },
    BlockRow {
        kind: BlockKind::Output,
        name: "output",
        min: 0,
        max: 8,
        size: SizeRule::Free,
        stacks: true,
    },
    BlockRow {
        kind: BlockKind::Editor,
        name: "editor",
        min: 1,
        max: 4,
        size: SizeRule::Editor,
        stacks: false,
    },
    BlockRow {
        kind: BlockKind::Statusline,
        name: "statusline",
        min: 0,
        max: 1,
        size: SizeRule::OneRow,
        stacks: false,
    },
];

impl BlockKind {
    /// The row of the block in [`BLOCKS`].
    #[must_use]
    pub fn row(self) -> &'static BlockRow {
        let index = match self {
            Self::Tabs => 0,
            Self::Output => 1,
            Self::Editor => 2,
            Self::Statusline => 3,
        };
        &BLOCKS[index]
    }

    /// The name of the block in a sentence.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.row().name
    }

    /// The block named `name`.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        by_name(BLOCKS, |row| row.name, name).map(|row| row.kind)
    }
}

impl fmt::Display for BlockKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
