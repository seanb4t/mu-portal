//! The solver: from a layout and an area of the screen to the rectangle of each pane.

use super::Layout;
use super::block::{BlockKind, SizeRule};
use super::change::{PANE_COLUMNS_MIN, PANE_ROWS_MIN};
use super::step::{BlockId, Dir, EDITOR_MAX_DEFAULT};
use super::tree::{Block, Node, Part, PartSize};

/// A rectangle of cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Area {
    /// The first column, from 0.
    pub x: u16,
    /// The first row, from 0.
    pub y: u16,
    /// The count of columns.
    pub width: u16,
    /// The count of rows.
    pub height: u16,
}

impl Area {
    /// An area at the top left corner of the screen.
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    /// Whether the cell at column `x` and row `y` is in the area.
    #[must_use]
    pub const fn holds(&self, x: u16, y: u16) -> bool {
        x >= self.x && y >= self.y && x - self.x < self.width && y - self.y < self.height
    }

    const fn start(self, dir: Dir) -> u16 {
        match dir {
            Dir::Rows => self.y,
            Dir::Columns => self.x,
        }
    }

    const fn extent(self, dir: Dir) -> u16 {
        match dir {
            Dir::Rows => self.height,
            Dir::Columns => self.width,
        }
    }

    /// The part of the area from `start` with `extent` cells in `dir`.
    const fn slice(self, dir: Dir, start: u16, extent: u16) -> Self {
        match dir {
            Dir::Rows => Self {
                x: self.x,
                y: start,
                width: self.width,
                height: extent,
            },
            Dir::Columns => Self {
                x: start,
                y: self.y,
                width: extent,
                height: self.height,
            },
        }
    }
}

/// One pane on the screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placed {
    /// The rectangle of the pane, its title row included.
    pub area: Area,
    /// The blocks of the pane. More than one is a stack.
    pub blocks: Vec<Block>,
    /// Whether the first row of the pane is a title row.
    pub title: bool,
}

/// A border between two parts of a split that a drag can move.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Border {
    /// The direction of the split.
    pub dir: Dir,
    /// The cells that a press hits: the divider of two columns, or the first row of the lower
    /// part of two rows.
    pub line: Area,
    /// The block whose line carries the size of the part beside the border.
    pub owner: BlockId,
    /// The first cell of the part of the owner, in the direction of the split.
    pub owner_start: u16,
    /// The cell after the last cell of the part of the owner.
    pub owner_end: u16,
    /// Whether the part of the owner is before the border.
    pub owner_first: bool,
    /// The count of the cells of the split, in its direction.
    pub total: u16,
}

impl Border {
    /// The cells of the part of the owner if the border moves to the cell `at`.
    #[must_use]
    pub const fn cells_at(&self, at: u16) -> u16 {
        if self.owner_first {
            at.saturating_sub(self.owner_start)
        } else {
            match self.dir {
                // The divider takes the cell `at`, and the part starts after it.
                Dir::Columns => self.owner_end.saturating_sub(at.saturating_add(1)),
                Dir::Rows => self.owner_end.saturating_sub(at),
            }
        }
    }
}

/// What the solver gives.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Solved {
    /// Each pane with room on the screen.
    pub panes: Vec<Placed>,
    /// The cells between two parts: one column between two columns, and one row between two
    /// rows if the caller asks for a gap between rows.
    pub dividers: Vec<Area>,
    /// The borders that a drag can move.
    pub borders: Vec<Border>,
}

impl Solved {
    /// The pane that holds the block `id`.
    #[must_use]
    pub fn pane_of(&self, id: &BlockId) -> Option<&Placed> {
        self.panes
            .iter()
            .find(|pane| pane.blocks.iter().any(|block| &block.id == id))
    }

    /// The pane at column `x` and row `y`.
    #[must_use]
    pub fn pane_at(&self, x: u16, y: u16) -> Option<&Placed> {
        self.panes.iter().find(|pane| pane.area.holds(x, y))
    }
}

struct Solver<'a> {
    rows_of: &'a dyn Fn(&BlockId, u16) -> u16,
    /// Whether the layout holds more than one `output`, so each pane of outputs has a title.
    titles: bool,
    /// The count of rows between two parts of a split of rows.
    row_gap: u16,
    layout: &'a Layout,
    solved: Solved,
}

fn is_text(node: &Node) -> bool {
    matches!(node, Node::Pane(blocks) if blocks.iter().any(|block| block.kind == BlockKind::Output))
}

fn sum(values: impl Iterator<Item = u16>) -> u16 {
    values.fold(0, u16::saturating_add)
}

impl Solver<'_> {
    const fn gap(&self, dir: Dir) -> u16 {
        match dir {
            Dir::Columns => 1,
            Dir::Rows => self.row_gap,
        }
    }

    /// The fewest cells of `node` in `dir`.
    fn least(&self, node: &Node, dir: Dir) -> u16 {
        match node {
            Node::Pane(_) if is_text(node) => match dir {
                Dir::Columns => PANE_COLUMNS_MIN,
                Dir::Rows => PANE_ROWS_MIN,
            },
            Node::Pane(_) => 1,
            Node::Split { dir: own, parts } => {
                let each = parts.iter().map(|part| self.least(&part.node, dir));
                if *own == dir {
                    let gaps = self
                        .gap(dir)
                        .saturating_mul(Self::count(parts).saturating_sub(1));
                    sum(each).saturating_add(gaps)
                } else {
                    each.max().unwrap_or(0)
                }
            }
        }
    }

    fn count(parts: &[Part]) -> u16 {
        u16::try_from(parts.len()).unwrap_or(u16::MAX)
    }

    /// The rows that `node` wants in a split of rows of `total` rows, at `width` columns. A
    /// node that shares the rest wants none.
    fn wanted_rows(&self, node: &Node, width: u16, total: u16) -> Option<u16> {
        match node {
            Node::Pane(blocks) => {
                let block = blocks.first()?;
                match block.kind.row().size {
                    SizeRule::OneRow => Some(1),
                    SizeRule::Free => None,
                    SizeRule::Editor => {
                        let limit = block
                            .max
                            .unwrap_or(EDITOR_MAX_DEFAULT)
                            .min(total / 2)
                            .max(1);
                        Some((self.rows_of)(&block.id, width).clamp(1, limit))
                    }
                }
            }
            Node::Split {
                dir: Dir::Columns,
                parts,
            } => {
                let widths = self.sizes(parts, Dir::Columns, width, 0);
                parts
                    .iter()
                    .zip(widths)
                    .map(|(part, width)| self.wanted_rows(&part.node, width, total))
                    .try_fold(0, |most, rows| rows.map(|rows| most.max(rows)))
            }
            Node::Split { dir: Dir::Rows, .. } => None,
        }
    }

    /// The cells of each part of a split of `total` cells in `dir`. `cross` is the count of
    /// the cells in the other direction.
    fn sizes(&self, parts: &[Part], dir: Dir, total: u16, cross: u16) -> Vec<u16> {
        let gaps = self
            .gap(dir)
            .saturating_mul(Self::count(parts).saturating_sub(1));
        let room = total.saturating_sub(gaps);
        // The size that each part asks for. A part with none shares the rest.
        let asked: Vec<Option<u16>> = parts
            .iter()
            .map(|part| match part.size {
                Some(PartSize::Percent(percent)) => {
                    let cells = u32::from(total) * u32::from(percent) / 100;
                    Some(u16::try_from(cells).unwrap_or(u16::MAX))
                }
                Some(PartSize::Cells(cells)) => Some(cells),
                None if dir == Dir::Rows => self.wanted_rows(&part.node, cross, total),
                None => None,
            })
            .collect();
        let least: Vec<u16> = parts
            .iter()
            .map(|part| self.least(&part.node, dir))
            .collect();
        let sharing = asked.iter().filter(|size| size.is_none()).count();
        // Each part with a size takes it in order, raised to its smallest size. It takes no
        // cell that another part needs for its own smallest size.
        let mut left = room.saturating_sub(sum(least.iter().copied()));
        let mut sizes: Vec<u16> = asked
            .iter()
            .zip(&least)
            .map(|(size, least)| {
                size.map_or(0, |cells| {
                    let cells = cells.max(*least).min(left.saturating_add(*least));
                    left = left.saturating_sub(cells.saturating_sub(*least));
                    cells
                })
            })
            .collect();
        let used = sum(sizes.iter().copied());
        let mut rest = room.saturating_sub(used);
        if sharing == 0 {
            // Every part has a size. The last part takes the cells that are left over.
            if let Some(last) = sizes.last_mut() {
                *last = last.saturating_add(rest);
            }
            return sizes;
        }
        let sharing = u16::try_from(sharing).unwrap_or(u16::MAX);
        let (each, mut extra) = (rest / sharing, rest % sharing);
        for (size, asked) in sizes.iter_mut().zip(&asked) {
            if asked.is_none() {
                *size = each + u16::from(extra > 0);
                extra = extra.saturating_sub(1);
                rest = rest.saturating_sub(*size);
            }
        }
        sizes
    }

    fn node(&mut self, node: &Node, area: Area) {
        match node {
            Node::Pane(blocks) => {
                if area.width == 0 || area.height == 0 {
                    return;
                }
                let title = is_text(node) && (blocks.len() > 1 || self.titles);
                self.solved.panes.push(Placed {
                    area,
                    blocks: blocks.clone(),
                    title,
                });
            }
            Node::Split { dir, parts } => self.split(*dir, parts, area),
        }
    }

    /// The block that owns the border between the parts `index` and `index + 1`, and whether
    /// its part is the first of the two.
    fn border_owner(&self, parts: &[Part], index: usize) -> Option<(BlockId, bool)> {
        let free = |id: &BlockId| {
            self.layout
                .steps()
                .iter()
                .any(|step| &step.id == id && step.kind.row().size == SizeRule::Free)
        };
        let (first, second) = (&parts[index], &parts[index + 1]);
        [
            (second, false, true),
            (first, true, true),
            (second, false, false),
            (first, true, false),
        ]
        .into_iter()
        .filter(|(part, _, sized)| !sized || part.size.is_some())
        .find_map(|(part, is_first, _)| {
            let owner = part.owner.as_ref().filter(|id| free(id))?;
            Some((owner.clone(), is_first))
        })
    }

    fn split(&mut self, dir: Dir, parts: &[Part], area: Area) {
        let cross = match dir {
            Dir::Rows => area.width,
            Dir::Columns => area.height,
        };
        let sizes = self.sizes(parts, dir, area.extent(dir), cross);
        let gap = self.gap(dir);
        let end = area.start(dir).saturating_add(area.extent(dir));
        let mut at = area.start(dir);
        let mut areas = Vec::with_capacity(parts.len());
        for (index, size) in sizes.iter().enumerate() {
            // A terminal with too little room cuts the last parts.
            let size = (*size).min(end - at);
            areas.push(area.slice(dir, at, size));
            at += size;
            let gap = gap.min(end - at);
            if index + 1 < parts.len() && gap > 0 {
                self.solved.dividers.push(area.slice(dir, at, gap));
                at += gap;
            }
        }
        for index in 0..parts.len().saturating_sub(1) {
            let Some((owner, owner_first)) = self.border_owner(parts, index) else {
                continue;
            };
            let after = areas[index + 1];
            let line = match dir {
                Dir::Columns => area.slice(dir, after.x.saturating_sub(1), 1),
                Dir::Rows => area.slice(dir, after.y, 1),
            };
            let part_area = if owner_first { areas[index] } else { after };
            self.solved.borders.push(Border {
                dir,
                line,
                owner,
                owner_start: part_area.start(dir),
                owner_end: part_area.start(dir).saturating_add(part_area.extent(dir)),
                owner_first,
                total: area.extent(dir),
            });
        }
        for (part, part_area) in parts.iter().zip(areas) {
            self.node(&part.node, part_area);
        }
    }
}

fn solve_with(
    layout: &Layout,
    area: Area,
    rows_of: &dyn Fn(&BlockId, u16) -> u16,
    row_gap: u16,
    titles: bool,
) -> Solved {
    let mut solver = Solver {
        rows_of,
        titles,
        row_gap,
        layout,
        solved: Solved::default(),
    };
    solver.node(layout.tree(), area);
    solver.solved
}

/// The rectangle of each pane of `layout` in `area`. `rows_of` gives the wrapped rows of the
/// text of an editor at a width.
#[must_use]
pub fn solve(layout: &Layout, area: Area, rows_of: &dyn Fn(&BlockId, u16) -> u16) -> Solved {
    let outputs = layout
        .steps()
        .iter()
        .filter(|step| step.kind == BlockKind::Output)
        .count();
    solve_with(layout, area, rows_of, 0, outputs > 1)
}

/// The mark of a line of the picture, from the lines above it, below it, left of it and right of
/// it.
fn mark(around: [bool; 4]) -> char {
    match (around[0], around[1], around[2], around[3]) {
        (true, true, true, true) => '┼',
        (true, true, true, false) => '┤',
        (true, true, false, true) => '├',
        (true, false, true, true) => '┴',
        (false, true, true, true) => '┬',
        (true, false, true, false) => '┘',
        (true, false, false, true) => '└',
        (false, true, true, false) => '┐',
        (false, true, false, true) => '┌',
        (true, _, false, false) | (false, true, false, false) => '│',
        _ => '─',
    }
}

/// A picture of `layout` with box characters, `width` columns and `height` rows. Each pane
/// shows the ids of its blocks.
#[must_use]
pub fn preview(layout: &Layout, width: u16, height: u16) -> Vec<String> {
    let (columns, rows) = (usize::from(width), usize::from(height));
    if columns < 3 || rows < 3 {
        return Vec::new();
    }
    let inner = Area {
        x: 1,
        y: 1,
        width: width - 2,
        height: height - 2,
    };
    let solved = solve_with(layout, inner, &|_, _| 1, 1, false);
    let mut lines = vec![vec![false; columns]; rows];
    for (row, cells) in lines.iter_mut().enumerate() {
        for (column, cell) in cells.iter_mut().enumerate() {
            *cell = row == 0 || row == rows - 1 || column == 0 || column == columns - 1;
        }
    }
    for divider in &solved.dividers {
        for row in divider.y..divider.y.saturating_add(divider.height) {
            for column in divider.x..divider.x.saturating_add(divider.width) {
                if let Some(cell) = lines
                    .get_mut(usize::from(row))
                    .and_then(|cells| cells.get_mut(usize::from(column)))
                {
                    *cell = true;
                }
            }
        }
    }
    let is_line = |row: Option<usize>, column: Option<usize>| {
        row.zip(column)
            .and_then(|(row, column)| lines.get(row)?.get(column).copied())
            .unwrap_or(false)
    };
    let mut picture: Vec<Vec<char>> = (0..rows)
        .map(|row| {
            (0..columns)
                .map(|column| {
                    if !lines[row][column] {
                        return ' ';
                    }
                    mark([
                        is_line(row.checked_sub(1), Some(column)),
                        is_line(Some(row + 1), Some(column)),
                        is_line(Some(row), column.checked_sub(1)),
                        is_line(Some(row), Some(column + 1)),
                    ])
                })
                .collect()
        })
        .collect();
    for pane in &solved.panes {
        let names: Vec<&str> = pane.blocks.iter().map(|block| block.id.as_str()).collect();
        let text = names.join(" + ");
        let room = usize::from(pane.area.width.saturating_sub(1));
        for (offset, character) in text.chars().take(room).enumerate() {
            let column = usize::from(pane.area.x) + 1 + offset;
            if let Some(cell) = picture
                .get_mut(usize::from(pane.area.y))
                .and_then(|cells| cells.get_mut(column))
            {
                *cell = character;
            }
        }
    }
    picture
        .into_iter()
        .map(|cells| cells.into_iter().collect::<String>().trim_end().to_owned())
        .collect()
}

/// One block of a layout for a caller that cannot see the screen: where it is and what it
/// shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Described {
    /// The id of the block.
    pub id: BlockId,
    /// The name of the block in a sentence.
    pub name: &'static str,
    /// The number of its line, from 1.
    pub line: usize,
    /// The rectangle of its pane. A block with no room has an empty rectangle.
    pub area: Area,
    /// Whether the block is the first of its pane, which shows when the client starts.
    pub front: bool,
    /// What the block shows.
    pub source: String,
}

/// Each block of `layout` on a screen of `width` columns and `height` rows, in the order of
/// the plan.
#[must_use]
pub fn describe(layout: &Layout, width: u16, height: u16) -> Vec<Described> {
    let solved = solve(layout, Area::new(width, height), &|_, _| 1);
    let first_output = layout.ids_of(BlockKind::Output).next();
    layout
        .steps()
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let pane = solved.pane_of(&step.id);
            let source = match step.kind {
                BlockKind::Tabs => "one tab for each world of the server".to_owned(),
                BlockKind::Editor => {
                    "the text that the player types for the world in focus".to_owned()
                }
                BlockKind::Statusline => "the segments of the world in focus".to_owned(),
                BlockKind::Output if Some(&step.id) == first_output => {
                    "the lines of the world in focus, and each stream with no output of its own"
                        .to_owned()
                }
                BlockKind::Output => format!("the stream {} of the world in focus", step.id),
            };
            Described {
                id: step.id.clone(),
                name: step.kind.name(),
                line: index + 1,
                area: pane.map_or_else(Area::default, |pane| pane.area),
                front: pane.is_none_or(|pane| {
                    pane.blocks.first().is_some_and(|block| block.id == step.id)
                }),
                source,
            }
        })
        .collect()
}
