//! One line of a plan: the block, its id, its place and its size.

use std::fmt;

use super::block::BlockKind;

/// The longest id of a block, in characters.
pub const ID_MAX: usize = 32;

/// The words of a sentence. No id is one of them.
pub const WORDS: &[&str] = &[
    "as", "above", "below", "left", "right", "of", "at", "top", "bottom", "with", "and", "max",
];

/// The id of a block: 1 to [`ID_MAX`] characters from `a` to `z`, `0` to `9` and `-`. It
/// starts with a letter, and is no word of [`WORDS`]. It is always lower case.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(String);

impl BlockId {
    /// Checks `id`, after it turns each upper-case letter to lower case.
    ///
    /// # Errors
    ///
    /// The reason that `id` is no id.
    pub fn new(id: &str) -> Result<Self, &'static str> {
        let id = id.to_ascii_lowercase();
        let mut bytes = id.bytes();
        if !bytes.next().is_some_and(|first| first.is_ascii_lowercase()) {
            return Err("an id starts with a letter from a to z");
        }
        if !bytes.all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'-')) {
            return Err("an id holds only a to z, 0 to 9 and -");
        }
        if id.len() > ID_MAX {
            return Err("an id has at most 32 characters");
        }
        if WORDS.contains(&id.as_str()) {
            return Err("an id is not a word of the sentences");
        }
        Ok(Self(id))
    }

    /// The id of a block of `kind` whose line has no `as`.
    #[must_use]
    pub fn of(kind: BlockKind) -> Self {
        Self(kind.default_id().to_owned())
    }

    /// The id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for BlockId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The side of an anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Above the anchor.
    Above,
    /// Below the anchor.
    Below,
    /// Left of the anchor.
    Left,
    /// Right of the anchor.
    Right,
}

impl Side {
    /// Every side.
    pub const ALL: [Self; 4] = [Self::Above, Self::Below, Self::Left, Self::Right];

    /// The words of the side in a sentence.
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self {
            Self::Above => "above",
            Self::Below => "below",
            Self::Left => "left of",
            Self::Right => "right of",
        }
    }

    /// The direction of a split on this side.
    #[must_use]
    pub const fn dir(self) -> Dir {
        match self {
            Self::Above | Self::Below => Dir::Rows,
            Self::Left | Self::Right => Dir::Columns,
        }
    }

    /// Whether the new block comes before the anchor in the split.
    #[must_use]
    pub const fn before(self) -> bool {
        matches!(self, Self::Above | Self::Left)
    }
}

/// An edge of the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
    /// The right edge.
    Right,
}

impl Edge {
    /// Every edge.
    pub const ALL: [Self; 4] = [Self::Top, Self::Bottom, Self::Left, Self::Right];

    /// The word of the edge in a sentence.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// The side of the old screen that the new block takes.
    #[must_use]
    pub const fn side(self) -> Side {
        match self {
            Self::Top => Side::Above,
            Self::Bottom => Side::Below,
            Self::Left => Side::Left,
            Self::Right => Side::Right,
        }
    }
}

/// The direction of a split.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    /// The parts are one above the other.
    Rows,
    /// The parts are side by side.
    Columns,
}

/// Where a line puts its block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    /// The first line: the whole screen.
    First,
    /// On one side of one or more anchors.
    Beside {
        /// The side.
        side: Side,
        /// The anchors, one or more.
        anchors: Vec<BlockId>,
    },
    /// Along a whole edge of the screen.
    At(Edge),
    /// In the pane of an anchor, as a stack.
    With(BlockId),
}

/// The size on a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    /// A percent of the split, from [`PERCENT_MIN`] to [`PERCENT_MAX`].
    Percent(u8),
    /// A count of cells, from 1.
    Cells(u16),
    /// The most rows of an editor, from 1 to [`EDITOR_MAX`].
    Max(u16),
}

/// The smallest percent of a size.
pub const PERCENT_MIN: u8 = 10;
/// The largest percent of a size.
pub const PERCENT_MAX: u8 = 90;
/// The largest `max` of an editor.
pub const EDITOR_MAX: u16 = 100;
/// The `max` of an editor whose line gives none.
pub const EDITOR_MAX_DEFAULT: u16 = 8;

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Percent(percent) => write!(f, "{percent}%"),
            Self::Cells(cells) => write!(f, "{cells}"),
            Self::Max(rows) => write!(f, "max {rows}"),
        }
    }
}

/// One line of a plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// The block.
    pub kind: BlockKind,
    /// The id of the block: the default id of its kind, or the word after `as`.
    pub id: BlockId,
    /// Where the block goes.
    pub place: Place,
    /// The size after the comma.
    pub size: Option<Size>,
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.kind.name())?;
        if self.id.as_str() != self.kind.default_id() {
            write!(f, " as {}", self.id)?;
        }
        match &self.place {
            Place::First => {}
            Place::Beside { side, anchors } => {
                write!(f, " {}", side.words())?;
                for (index, anchor) in anchors.iter().enumerate() {
                    let join = if index == 0 { " " } else { " and " };
                    write!(f, "{join}{anchor}")?;
                }
            }
            Place::At(edge) => write!(f, " at {}", edge.word())?,
            Place::With(anchor) => write!(f, " with {anchor}")?,
        }
        if let Some(size) = self.size {
            write!(f, ", {size}")?;
        }
        Ok(())
    }
}
