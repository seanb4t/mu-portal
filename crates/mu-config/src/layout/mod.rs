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

pub use block::{BLOCKS, BlockKind, BlockRow, SizeRule};
pub use builtin::{BUILT_IN, BuiltIn, DEFAULT_LAYOUT, LayoutFile};
pub use change::{Changed, LineRef, PANE_COLUMNS_MIN, PANE_ROWS_MIN, RESIZE_STEP};
pub use parse::LineError;
pub use solve::{Area, Border, Placed, Solved, preview, solve};
pub use step::{
    BlockId, Dir, EDITOR_MAX, EDITOR_MAX_DEFAULT, Edge, ID_MAX, PERCENT_MAX, PERCENT_MIN, Place,
    Side, Size, Step, WORDS,
};
pub use tree::{Block, Node, Part, PartSize};

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

    fn whole(word: impl Into<String>, reason: impl Into<String>) -> Self {
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
        write!(f, "`{}`: {}", self.word, self.reason)
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
    /// A [`LayoutError`] that names the line and the word.
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
    /// A [`LayoutError`] that names the line and the id.
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
