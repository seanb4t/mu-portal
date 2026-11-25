//! The changes of a plan: the edits of one line, and the changes of the screen.

use super::step::{BlockId, Dir, PERCENT_MAX, PERCENT_MIN, Place, Size, Step};
use super::tree::{self, Node};
use super::{Layout, LayoutError, write};
use crate::names::listed;

/// The fewest columns of a pane of text.
pub const PANE_COLUMNS_MIN: u16 = 10;
/// The fewest rows of a pane of text.
pub const PANE_ROWS_MIN: u16 = 2;
/// The percent of one step of [`Layout::resize_by`].
pub const RESIZE_STEP: i32 = 5;

/// One line of a plan, by its number or by the id of its block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineRef {
    /// The number of the line, from 1.
    Number(usize),
    /// The id of the block of the line.
    Id(BlockId),
}

impl LineRef {
    /// Reads a number or an id.
    ///
    /// # Errors
    ///
    /// The reason that `text` is neither.
    pub fn parse(text: &str) -> Result<Self, &'static str> {
        if !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()) {
            return text
                .parse()
                .map(Self::Number)
                .map_err(|_| "the number of a line is too large");
        }
        BlockId::new(text).map(Self::Id)
    }
}

/// The result of a change of the screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Changed {
    /// The new layout.
    pub layout: Layout,
    /// Whether mu wrote the whole plan again, because no edit of one line gave the screen.
    pub rewritten: bool,
}

/// The range of the percent that keeps a block and the rest of its split at the smallest size
/// of a pane, in a split of `total` cells. `None` if no percent does.
fn percent_range(total: u16, dir: Dir) -> Option<(u32, u32)> {
    let total = u32::from(total);
    let (least, divider) = match dir {
        Dir::Columns => (u32::from(PANE_COLUMNS_MIN), 1),
        Dir::Rows => (u32::from(PANE_ROWS_MIN), 0),
    };
    if total == 0 {
        return None;
    }
    let low = u32::from(PERCENT_MIN).max((least * 100).div_ceil(total));
    let high = u32::from(PERCENT_MAX).min(total.checked_sub(least + divider)? * 100 / total);
    (low <= high).then_some((low, high))
}

impl Layout {
    fn index_of(&self, id: &BlockId) -> Result<usize, LayoutError> {
        self.steps
            .iter()
            .position(|step| &step.id == id)
            .ok_or_else(|| LayoutError::whole(id.as_str(), "no block of the layout has this id"))
    }

    /// The index of `line`, which is a number from 1 to the count of the lines, or an id.
    fn index(&self, line: &LineRef) -> Result<usize, LayoutError> {
        match line {
            LineRef::Id(id) => self.index_of(id),
            LineRef::Number(number) if (1..=self.steps.len()).contains(number) => Ok(number - 1),
            LineRef::Number(number) => Err(LayoutError::whole(
                number.to_string(),
                format!("the plan has the lines 1 to {}", self.steps.len()),
            )),
        }
    }

    /// Puts `step` at line `number`, from 1 to one more than the count of the lines.
    ///
    /// # Errors
    ///
    /// A [`LayoutError`] for a number outside that range, and for a plan that does not build.
    pub fn insert(&self, number: usize, step: Step) -> Result<Self, LayoutError> {
        if !(1..=self.steps.len() + 1).contains(&number) {
            return Err(LayoutError::whole(
                number.to_string(),
                format!(
                    "a new line takes a number from 1 to {}",
                    self.steps.len() + 1
                ),
            ));
        }
        let mut steps = self.steps.clone();
        steps.insert(number - 1, step);
        Self::from_steps(steps)
    }

    /// Replaces `line` with `step`.
    ///
    /// # Errors
    ///
    /// A [`LayoutError`] for a line that the plan does not have, and for a plan that does not
    /// build.
    pub fn replace(&self, line: &LineRef, step: Step) -> Result<Self, LayoutError> {
        let index = self.index(line)?;
        let mut steps = self.steps.clone();
        steps[index] = step;
        Self::from_steps(steps)
    }

    /// Removes `line`.
    ///
    /// # Errors
    ///
    /// A [`LayoutError`] for a line that the plan does not have, for a block that a later line
    /// names as an anchor, and for a plan that does not build.
    pub fn delete(&self, line: &LineRef) -> Result<Self, LayoutError> {
        let index = self.index(line)?;
        let id = &self.steps[index].id;
        let users: Vec<String> = self
            .steps
            .iter()
            .enumerate()
            .filter(|(_, step)| match &step.place {
                Place::Beside { anchors, .. } => anchors.contains(id),
                Place::With(anchor) => anchor == id,
                Place::First | Place::At(_) => false,
            })
            .map(|(user, _)| (user + 1).to_string())
            .collect();
        if !users.is_empty() {
            return Err(LayoutError::whole(
                id.as_str(),
                format!("line {} names this block as an anchor", listed(&users)),
            ));
        }
        let mut steps = self.steps.clone();
        steps.remove(index);
        Self::from_steps(steps)
    }

    /// The first of `plans` that builds `wanted`, or the plan that the writer gives for it.
    fn settle(wanted: &Node, plans: Vec<Vec<Step>>) -> Result<Changed, LayoutError> {
        for steps in plans {
            if let Ok(layout) = Self::from_steps(steps)
                && &layout.tree == wanted
            {
                return Ok(Changed {
                    layout,
                    rewritten: false,
                });
            }
        }
        Ok(Changed {
            layout: Self::from_steps(write::plan(wanted))?,
            rewritten: true,
        })
    }

    /// Puts the block of `step` on the screen. A block that is on the screen moves.
    ///
    /// # Errors
    ///
    /// A [`LayoutError`] for a step that the layout rules refuse.
    pub fn place(&self, step: Step) -> Result<Changed, LayoutError> {
        let Some(index) = self.steps.iter().position(|each| each.id == step.id) else {
            let mut steps = self.steps.clone();
            steps.push(step);
            return Ok(Changed {
                layout: Self::from_steps(steps)?,
                rewritten: false,
            });
        };
        let mut wanted = tree::without(self.tree.clone(), &step.id);
        tree::apply(&mut wanted, &step, index + 1)?;
        let Some(wanted) = wanted else {
            return Err(LayoutError::whole(
                step.id.as_str(),
                "the layout has no block",
            ));
        };
        let mut in_place = self.steps.clone();
        in_place[index] = step.clone();
        let mut at_end = self.steps.clone();
        at_end.remove(index);
        at_end.push(step);
        Self::settle(&wanted, vec![in_place, at_end])
    }

    /// Takes the block `id` off the screen.
    ///
    /// # Errors
    ///
    /// A [`LayoutError`] for an id that the layout does not have, and for a layout that needs
    /// the block.
    pub fn hide(&self, id: &BlockId) -> Result<Changed, LayoutError> {
        let index = self.index_of(id)?;
        let kind = self.steps[index].kind;
        let held = self.steps.iter().filter(|step| step.kind == kind).count();
        if held <= kind.row().min {
            return Err(LayoutError::whole(
                id.as_str(),
                format!(
                    "a layout holds `{}` at least {} times",
                    kind.name(),
                    kind.row().min
                ),
            ));
        }
        let Some(wanted) = tree::without(self.tree.clone(), id) else {
            return Err(LayoutError::whole(
                id.as_str(),
                "the layout has no other block",
            ));
        };
        let mut steps = self.steps.clone();
        steps.remove(index);
        Self::settle(&wanted, vec![steps])
    }

    /// Sets the size on the line of the block `id`.
    ///
    /// # Errors
    ///
    /// A [`LayoutError`] for an id that the layout does not have, for the block of the first
    /// line, and for a size that the block refuses.
    pub fn resize(&self, id: &BlockId, size: Size) -> Result<Changed, LayoutError> {
        let index = self.index_of(id)?;
        if index == 0 && !matches!(size, Size::Max(_)) {
            let beside: Vec<&Step> = self
                .steps
                .iter()
                .filter(|step| {
                    matches!(&step.place, Place::Beside { anchors, .. } if anchors.contains(id))
                })
                .collect();
            let sized: Vec<&BlockId> = beside
                .iter()
                .filter(|step| step.size.is_some())
                .map(|step| &step.id)
                .collect();
            let names: Vec<&BlockId> = if sized.is_empty() {
                beside.iter().map(|step| &step.id).collect()
            } else {
                sized
            };
            let reason = if names.is_empty() {
                "the block of the first line takes the rest of the screen, and has no size"
                    .to_owned()
            } else {
                format!(
                    "the block of the first line takes the rest of its split: set the size of {}",
                    listed(&names)
                )
            };
            return Err(LayoutError::at(1, id.as_str(), reason));
        }
        let mut steps = self.steps.clone();
        steps[index].size = Some(size);
        Ok(Changed {
            layout: Self::from_steps(steps)?,
            rewritten: false,
        })
    }

    /// The direction of the split that the line of `id` adds a part to.
    fn dir_of(&self, index: usize) -> Result<Dir, LayoutError> {
        match &self.steps[index].place {
            Place::Beside { side, .. } => Ok(side.dir()),
            Place::At(edge) => Ok(edge.side().dir()),
            Place::First | Place::With(_) => Err(LayoutError::at(
                index + 1,
                self.steps[index].id.as_str(),
                "only a block beside another or at an edge takes a size",
            )),
        }
    }

    /// Sets the size of `id` to `percent`, kept in the range that [`percent_range`] gives. With
    /// no such range, the plan stays as it is.
    fn resize_within(
        &self,
        id: &BlockId,
        percent: u32,
        total: u16,
    ) -> Result<Changed, LayoutError> {
        let index = self.index_of(id)?;
        let Some((low, high)) = percent_range(total, self.dir_of(index)?) else {
            return Ok(Changed {
                layout: self.clone(),
                rewritten: false,
            });
        };
        let percent = u8::try_from(percent.clamp(low, high)).unwrap_or(PERCENT_MAX);
        self.resize(id, Size::Percent(percent))
    }

    /// Makes the block `id` larger or smaller by `steps` of 5 percent. The block has `cells`
    /// of the `total` cells of its split now. The percent stays from 10 to 90, and both sides
    /// keep the smallest size of a pane.
    ///
    /// # Errors
    ///
    /// A [`LayoutError`] for an id that the layout does not have, and for a block that takes
    /// no size.
    pub fn resize_by(
        &self,
        id: &BlockId,
        steps: i32,
        cells: u16,
        total: u16,
    ) -> Result<Changed, LayoutError> {
        let index = self.index_of(id)?;
        let now = match self.steps[index].size {
            Some(Size::Percent(percent)) => i32::from(percent),
            _ if total == 0 => 0,
            _ => i32::from(cells) * 100 / i32::from(total),
        };
        let wanted = now.saturating_add(steps.saturating_mul(RESIZE_STEP)).max(0);
        self.resize_within(id, u32::try_from(wanted).unwrap_or(0), total)
    }

    /// Sets the size of the block `id` to the smallest whole percent that gives `cells` of the
    /// `total` cells of its split. The percent stays from 10 to 90, and both sides keep the
    /// smallest size of a pane.
    ///
    /// # Errors
    ///
    /// A [`LayoutError`] for an id that the layout does not have, and for a block that takes
    /// no size.
    pub fn resize_border(
        &self,
        id: &BlockId,
        cells: u16,
        total: u16,
    ) -> Result<Changed, LayoutError> {
        let percent = if total == 0 {
            0
        } else {
            (u32::from(cells) * 100).div_ceil(u32::from(total))
        };
        self.resize_within(id, percent, total)
    }
}
