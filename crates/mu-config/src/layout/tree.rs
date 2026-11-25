//! The tree of a layout, and how a plan builds it.

use super::block::{BlockKind, SizeRule};
use super::step::{BlockId, Dir, EDITOR_MAX, PERCENT_MAX, PERCENT_MIN, Place, Side, Size, Step};
use super::{LEVELS_MAX, LayoutError};

/// One block in a pane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// The kind of the block.
    pub kind: BlockKind,
    /// Its id.
    pub id: BlockId,
    /// The `max` of an editor whose line gives one.
    pub max: Option<u16>,
}

/// The size of one part of a split.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartSize {
    /// A percent of the split.
    Percent(u8),
    /// A count of cells.
    Cells(u16),
}

/// One part of a split.
#[derive(Clone, Debug)]
pub struct Part {
    /// The size of the part. A part with none shares the rest of the split.
    pub size: Option<PartSize>,
    /// The block whose line made the part, and so carries its size. The part of an anchor has
    /// none.
    pub owner: Option<BlockId>,
    /// What the part holds.
    pub node: Node,
}

/// Two parts are the same part of the screen if they have the same size and hold the same
/// node. The owner says which line carries the size, and is no part of the screen.
impl PartialEq for Part {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.node == other.node
    }
}

impl Eq for Part {}

/// A node of the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// One or more blocks in one rectangle. More than one is a stack.
    Pane(Vec<Block>),
    /// Two or more parts in one direction.
    Split {
        /// The direction.
        dir: Dir,
        /// The parts, from the top or from the left.
        parts: Vec<Part>,
    },
}

impl Node {
    /// Each block of the tree, from the top left.
    pub fn blocks(&self) -> Box<dyn Iterator<Item = &Block> + '_> {
        match self {
            Self::Pane(blocks) => Box::new(blocks.iter()),
            Self::Split { parts, .. } => Box::new(parts.iter().flat_map(|part| part.node.blocks())),
        }
    }

    /// The count of the levels of splits: 0 for a pane, and 1 for a split of panes.
    #[must_use]
    pub fn levels(&self) -> usize {
        match self {
            Self::Pane(_) => 0,
            Self::Split { parts, .. } => {
                1 + parts
                    .iter()
                    .map(|part| part.node.levels())
                    .max()
                    .unwrap_or(0)
            }
        }
    }

    /// The path to the pane that holds `id`: the index of a part at each split.
    pub(super) fn path_of(&self, id: &BlockId) -> Option<Vec<usize>> {
        match self {
            Self::Pane(blocks) => blocks.iter().any(|block| &block.id == id).then(Vec::new),
            Self::Split { parts, .. } => parts.iter().enumerate().find_map(|(index, part)| {
                let mut path = part.node.path_of(id)?;
                path.insert(0, index);
                Some(path)
            }),
        }
    }

    pub(super) fn at_mut(&mut self, path: &[usize]) -> &mut Self {
        match (self, path) {
            (Self::Split { parts, .. }, [first, rest @ ..]) => parts[*first].node.at_mut(rest),
            (node, _) => node,
        }
    }

    fn pane(block: Block) -> Self {
        Self::Pane(vec![block])
    }

    /// Puts `new` on `side` of the node at `path`.
    fn put_beside(&mut self, path: &[usize], side: Side, new: Part) {
        if let [parent @ .., index] = path
            && let Self::Split { dir, parts } = self.at_mut(parent)
            && *dir == side.dir()
        {
            let at = if side.before() { *index } else { index + 1 };
            parts.insert(at, new);
            return;
        }
        let target = self.at_mut(path);
        let old = Part {
            size: None,
            owner: None,
            node: std::mem::replace(target, Self::Pane(Vec::new())),
        };
        let parts = if side.before() {
            vec![new, old]
        } else {
            vec![old, new]
        };
        *target = Self::Split {
            dir: side.dir(),
            parts,
        };
    }
}

/// The tree with no block `id`. An empty pane and a split of one part go away, and a split never
/// holds a split of its own direction. The result is `None` if `id` was the only block.
pub(super) fn without(node: Node, id: &BlockId) -> Option<Node> {
    match node {
        Node::Pane(mut blocks) => {
            blocks.retain(|block| &block.id != id);
            (!blocks.is_empty()).then_some(Node::Pane(blocks))
        }
        Node::Split { dir, parts } => {
            let mut kept: Vec<Part> = Vec::new();
            for part in parts {
                match without(part.node, id) {
                    None => {}
                    Some(Node::Split {
                        dir: inner_dir,
                        parts: inner,
                    }) if inner_dir == dir => kept.extend(inner),
                    Some(node) => kept.push(Part {
                        size: part.size,
                        owner: part.owner,
                        node,
                    }),
                }
            }
            if kept.len() < 2 {
                return kept.pop().map(|part| part.node);
            }
            if kept.iter().all(|part| part.size.is_some()) {
                kept[0].size = None;
            }
            Some(Node::Split { dir, parts: kept })
        }
    }
}

/// The part and the block that one step makes.
fn made(step: &Step, line: usize) -> Result<(Block, Option<PartSize>), LayoutError> {
    let bad = |reason: &str| {
        Err(LayoutError::at(
            line,
            step.size
                .map_or_else(|| step.kind.name().to_owned(), |size| size.to_string()),
            reason,
        ))
    };
    let mut block = Block {
        kind: step.kind,
        id: step.id.clone(),
        max: None,
    };
    let in_range = match step.size {
        None => true,
        Some(Size::Percent(percent)) => (PERCENT_MIN..=PERCENT_MAX).contains(&percent),
        Some(Size::Cells(cells)) => cells >= 1,
        Some(Size::Max(rows)) => (1..=EDITOR_MAX).contains(&rows),
    };
    if !in_range {
        return bad(
            "bad size: a percent is from 10% to 90%, a count is from 1, and `max` is from 1 to 100",
        );
    }
    let rule = step.kind.row().size;
    let size = match (step.size, rule) {
        (None, _) => None,
        (Some(Size::Max(rows)), SizeRule::Editor) => {
            block.max = Some(rows);
            None
        }
        (Some(_), SizeRule::Editor) => {
            return bad("bad size: an editor takes only `max` and a count of rows");
        }
        (Some(_), SizeRule::OneRow) => {
            return bad(&format!(
                "bad size: `{}` always takes 1 row",
                step.kind.name()
            ));
        }
        (Some(Size::Max(_)), SizeRule::Free) => {
            return bad("bad size: only an editor takes `max`");
        }
        (Some(Size::Percent(percent)), SizeRule::Free) => Some(PartSize::Percent(percent)),
        (Some(Size::Cells(cells)), SizeRule::Free) => Some(PartSize::Cells(cells)),
    };
    if size.is_some() && matches!(step.place, Place::First | Place::With(_)) {
        return bad("bad size: only a block beside another or at an edge takes a size");
    }
    Ok((block, size))
}

fn path_to(root: &Node, anchor: &BlockId, line: usize) -> Result<Vec<usize>, LayoutError> {
    root.path_of(anchor)
        .ok_or_else(|| LayoutError::at(line, anchor.as_str(), "no earlier line places this anchor"))
}

/// Applies the step of line `line` to the tree.
pub(super) fn apply(root: &mut Option<Node>, step: &Step, line: usize) -> Result<(), LayoutError> {
    let (block, part_size) = made(step, line)?;
    let Some(tree) = root else {
        if step.place != Place::First {
            return Err(LayoutError::at(
                line,
                step.kind.name(),
                "the first line is one block with no place",
            ));
        }
        *root = Some(Node::pane(block));
        return Ok(());
    };
    if tree.blocks().any(|each| each.id == step.id) {
        return Err(LayoutError::at(
            line,
            step.id.as_str(),
            "an earlier line places a block with this id",
        ));
    }
    let new = Part {
        size: part_size,
        owner: Some(block.id.clone()),
        node: Node::pane(block.clone()),
    };
    match &step.place {
        Place::First => {
            return Err(LayoutError::at(
                line,
                step.kind.name(),
                "unknown form: only the first line has no place",
            ));
        }
        Place::With(anchor) => {
            let path = path_to(tree, anchor, line)?;
            let Node::Pane(blocks) = tree.at_mut(&path) else {
                return Err(LayoutError::at(
                    line,
                    anchor.as_str(),
                    "the anchor has no pane",
                ));
            };
            let stacks = |kind: BlockKind| kind.row().stacks;
            if !stacks(step.kind) || !blocks.iter().all(|each| stacks(each.kind)) {
                return Err(LayoutError::at(
                    line,
                    step.kind.name(),
                    "only `output` blocks share a pane",
                ));
            }
            blocks.push(block);
        }
        Place::At(edge) => {
            let side = edge.side();
            match tree {
                Node::Split { dir, parts } if *dir == side.dir() => {
                    let at = if side.before() { 0 } else { parts.len() };
                    parts.insert(at, new);
                }
                _ => tree.put_beside(&[], side, new),
            }
        }
        Place::Beside { side, anchors } => beside(tree, *side, anchors, new, line)?,
    }
    if tree.levels() > LEVELS_MAX {
        return Err(LayoutError::at(
            line,
            step.id.as_str(),
            format!("a layout has at most {LEVELS_MAX} levels of splits"),
        ));
    }
    Ok(())
}

fn beside(
    tree: &mut Node,
    side: Side,
    anchors: &[BlockId],
    new: Part,
    line: usize,
) -> Result<(), LayoutError> {
    let paths = anchors
        .iter()
        .map(|anchor| path_to(tree, anchor, line))
        .collect::<Result<Vec<_>, _>>()?;
    let first = &paths[0];
    let common = (0..first.len())
        .find(|&depth| paths.iter().any(|path| path.get(depth) != first.get(depth)))
        .unwrap_or(first.len());
    let shared = &first[..common];
    if paths.iter().any(|path| path.len() == common) {
        // Every anchor is in one pane.
        tree.put_beside(shared, side, new);
        return Ok(());
    }
    let mut indexes: Vec<usize> = paths.iter().map(|path| path[common]).collect();
    indexes.sort_unstable();
    indexes.dedup();
    let (low, high) = (indexes[0], indexes[indexes.len() - 1]);
    let names = || crate::names::listed(anchors);
    if high - low + 1 != indexes.len() {
        return Err(LayoutError::at(
            line,
            names(),
            "these anchors are not neighbors in one split",
        ));
    }
    let Node::Split { dir, parts } = tree.at_mut(shared) else {
        return Err(LayoutError::at(
            line,
            names(),
            "these anchors share no split",
        ));
    };
    if *dir == side.dir() {
        let at = if side.before() { low } else { high + 1 };
        parts.insert(at, new);
        return Ok(());
    }
    if indexes.len() == parts.len() {
        tree.put_beside(shared, side, new);
        return Ok(());
    }
    if parts[low..=high].iter().all(|part| part.size.is_some()) {
        return Err(LayoutError::at(
            line,
            names(),
            "one of these anchors must have no size",
        ));
    }
    let wrapped: Vec<Part> = parts.drain(low..=high).collect();
    let old = Part {
        size: None,
        owner: None,
        node: Node::Split {
            dir: *dir,
            parts: wrapped,
        },
    };
    let inner = if side.before() {
        vec![new, old]
    } else {
        vec![old, new]
    };
    parts.insert(
        low,
        Part {
            size: None,
            owner: None,
            node: Node::Split {
                dir: side.dir(),
                parts: inner,
            },
        },
    );
    Ok(())
}
