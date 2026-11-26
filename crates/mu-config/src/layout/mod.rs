//! The layout of the screen: a plan of lines, and the tree that the plan builds.

mod block;
mod builtin;
mod change;
mod parse;
mod solve;
mod step;
mod tree;
mod write;

use std::fmt;

use crate::stream::MAIN;

pub use block::{BLOCKS, BlockKind, BlockRow, SizeRule};
pub use builtin::{BUILT_IN, BuiltIn, DEFAULT_LAYOUT, LayoutFile};
pub use change::{
    Change, Changed, FILL, LineRef, PANE_COLUMNS_MIN, PANE_ROWS_MIN, RESIZE_STEP, REWRITTEN,
};
pub use parse::{LINE_MAX, LineError};
pub use solve::{Area, Border, Placed, Solved, preview, solve};
pub use solve::{Described, describe};
pub use step::{
    BlockId, Dir, EDITOR_MAX, EDITOR_MAX_DEFAULT, Edge, ID_MAX, PERCENT_MAX, PERCENT_MIN, Place,
    Side, Size, Step, WORDS,
};
pub use tree::{Block, Node, Part, PartSize};

/// The lines `lines` of a plan with their numbers from 1, as each listing of a plan shows them.
#[must_use]
pub fn numbered<S: AsRef<str>>(lines: &[S]) -> Vec<String> {
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| format!("{}. {}", index + 1, line.as_ref()))
        .collect()
}

/// The most lines of a plan.
pub const LINES_MAX: usize = 32;
/// The most levels of splits of a tree.
pub const LEVELS_MAX: usize = 8;

/// What is wrong with a plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutError {
    /// The number of the line, from 1. An error about the whole plan has none.
    pub line: Option<usize>,
    /// The word or the id that the error is about.
    pub word: String,
    /// The column of the word, from 1, if the reader stopped at it.
    pub column: Option<usize>,
    /// What is wrong.
    pub reason: String,
}

impl LayoutError {
    pub(crate) fn at(line: usize, word: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            line: Some(line),
            word: word.into(),
            column: None,
            reason: reason.into(),
        }
    }

    pub(crate) fn whole(word: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            line: None,
            word: word.into(),
            column: None,
            reason: reason.into(),
        }
    }
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(f, "line {line}, ")?;
        }
        if let Some(column) = self.column {
            write!(f, "column {column}, ")?;
        }
        write!(f, "`{}`: {}", parse::shown(&self.word), self.reason)
    }
}

impl std::error::Error for LayoutError {}

/// A checked layout: its plan, and the tree that the plan builds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    steps: Vec<Step>,
    tree: Node,
}

impl Layout {
    /// Reads and checks the lines of a plan.
    ///
    /// # Errors
    ///
    /// A [`LayoutError`] that names the line and the word. An error about the plan as a whole,
    /// such as too many lines, names no line.
    pub fn parse<S: AsRef<str>>(lines: &[S]) -> Result<Self, LayoutError> {
        count_lines(lines.len())?;
        let steps = lines
            .iter()
            .enumerate()
            .map(|(index, line)| {
                Step::parse(line.as_ref()).map_err(|error| LayoutError {
                    line: Some(index + 1),
                    word: error.word,
                    column: Some(error.column),
                    reason: error.reason,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::from_steps(steps)
    }

    /// Checks a plan of steps, and builds its tree.
    ///
    /// # Errors
    ///
    /// A [`LayoutError`] that names the line and the id. An error about the plan as a whole
    /// names no line.
    pub fn from_steps(steps: Vec<Step>) -> Result<Self, LayoutError> {
        count_lines(steps.len())?;
        let mut root = None;
        for (index, step) in steps.iter().enumerate() {
            let line = index + 1;
            let held = steps[..index]
                .iter()
                .filter(|earlier| earlier.kind == step.kind)
                .count();
            let row = step.kind.row();
            if held >= row.max {
                return Err(LayoutError::at(
                    line,
                    row.name,
                    format!("a layout holds `{}` at most {} times", row.name, row.max),
                ));
            }
            // The limit is on the line as mu writes it, so each plan reads back.
            let written = step.to_string();
            if written.chars().count() > LINE_MAX {
                return Err(LayoutError::at(
                    line,
                    step.id.as_str(),
                    format!(
                        "a line has at most {LINE_MAX} characters, and mu writes this one with more"
                    ),
                ));
            }
            tree::apply(&mut root, step, line)?;
        }
        let tree = root.ok_or_else(|| LayoutError::whole("", "a layout has at least 1 line"))?;
        for row in BLOCKS {
            let held = steps.iter().filter(|step| step.kind == row.kind).count();
            if held < row.min {
                return Err(LayoutError::whole(
                    row.name,
                    format!("a layout holds `{}` at least {} times", row.name, row.min),
                ));
            }
        }
        Ok(Self { steps, tree })
    }

    /// The steps of the plan, in order.
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// The lines of the plan, in order.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        self.steps.iter().map(ToString::to_string).collect()
    }

    /// The tree that the plan builds.
    #[must_use]
    pub fn tree(&self) -> &Node {
        &self.tree
    }

    /// The id of each block of `kind`, in the order of the plan.
    pub fn ids_of(&self, kind: BlockKind) -> impl Iterator<Item = &BlockId> {
        self.steps
            .iter()
            .filter(move |step| step.kind == kind)
            .map(|step| &step.id)
    }

    /// Whether the layout holds a block `id` of `kind`.
    #[must_use]
    pub fn holds(&self, kind: BlockKind, id: &BlockId) -> bool {
        self.ids_of(kind).any(|each| each == id)
    }

    /// The lines of the plan with their numbers, as each listing of a plan shows them.
    #[must_use]
    pub fn numbered(&self) -> Vec<String> {
        numbered(&self.lines())
    }

    /// The main `output`: the one with the id [`MAIN`], or the first `output` of the plan if
    /// none has that id. It shows the lines of the world, and each stream that no other
    /// `output` shows. None for a plan with no `output`.
    #[must_use]
    pub fn main_output(&self) -> Option<&BlockId> {
        self.route(MAIN)
    }

    /// The `output` that shows the stream `stream`: the one with that id, or the main one.
    /// None for a plan with no `output`.
    #[must_use]
    pub fn route(&self, stream: &str) -> Option<&BlockId> {
        let found = |wanted: &str| {
            self.ids_of(BlockKind::Output)
                .find(|id| id.as_str() == wanted)
        };
        found(stream)
            .or_else(|| found(MAIN))
            .or_else(|| self.ids_of(BlockKind::Output).next())
    }

    /// The same screen as a plan that mu writes from the tree. Each block keeps its id.
    ///
    /// # Errors
    ///
    /// A [`LayoutError`] if the written plan does not build, which is a defect of the writer.
    pub fn written(&self) -> Result<Self, LayoutError> {
        Self::from_steps(write::plan(&self.tree))
    }
}

fn count_lines(count: usize) -> Result<(), LayoutError> {
    if count > LINES_MAX {
        return Err(LayoutError::whole(
            count.to_string(),
            format!("a layout has at most {LINES_MAX} lines"),
        ));
    }
    Ok(())
}
