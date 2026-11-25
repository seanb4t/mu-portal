//! The writer: from a tree to a plan that builds the same tree.

use super::block::SizeRule;
use super::step::{BlockId, Dir, Place, Side, Size, Step};
use super::tree::{Block, Node, Part, PartSize};

/// Whether a block of `node` can stand for it. A node in a part with a size needs a block
/// that takes a size, because the line of that block carries the size of the part.
fn stands(node: &Node, sized: bool) -> bool {
    match node {
        Node::Pane(blocks) => blocks
            .first()
            .is_some_and(|block| !sized || block.kind.row().size == SizeRule::Free),
        Node::Split { parts, .. } => parts
            .iter()
            .any(|part| part.size.is_none() && stands(&part.node, sized)),
    }
}

/// The part of a split that the other parts are placed beside: the first part with no size
/// whose block can stand for the split.
fn anchor_index(parts: &[Part], sized: bool) -> usize {
    parts
        .iter()
        .position(|part| part.size.is_none() && stands(&part.node, sized))
        .unwrap_or(0)
}

/// The block that stands for a node: the first block of the anchor part, down to a pane.
fn rep(node: &Node, sized: bool) -> Option<&Block> {
    match node {
        Node::Pane(blocks) => blocks.first(),
        Node::Split { parts, .. } => rep(&parts[anchor_index(parts, sized)].node, sized),
    }
}

fn step(block: &Block, place: Place, size: Option<PartSize>) -> Step {
    let size = match (block.max, size) {
        (Some(rows), _) => Some(Size::Max(rows)),
        (None, Some(PartSize::Percent(percent))) => Some(Size::Percent(percent)),
        (None, Some(PartSize::Cells(cells))) => Some(Size::Cells(cells)),
        (None, None) => None,
    };
    Step {
        kind: block.kind,
        id: block.id.clone(),
        place,
        size,
    }
}

fn beside(side: Side, anchor: &BlockId) -> Place {
    Place::Beside {
        side,
        anchors: vec![anchor.clone()],
    }
}

/// Writes the inside of `node`, whose block `rep(node)` is already placed as one pane. `sized`
/// says whether the part of `node` has a size.
fn expand(node: &Node, sized: bool, plan: &mut Vec<Step>) {
    match node {
        Node::Pane(blocks) => {
            if let [first, rest @ ..] = blocks.as_slice() {
                for block in rest {
                    plan.push(step(block, Place::With(first.id.clone()), None));
                }
            }
        }
        Node::Split { dir, parts } => {
            let anchor = anchor_index(parts, sized);
            let Some(anchor_block) = rep(&parts[anchor].node, sized) else {
                return;
            };
            let (before, after) = match dir {
                Dir::Rows => (Side::Above, Side::Below),
                Dir::Columns => (Side::Left, Side::Right),
            };
            // Each line puts its block next to the anchor. So the parts before the anchor go
            // in order, and the parts after it go from the last one.
            let order = (0..anchor)
                .map(|index| (index, before))
                .chain((anchor + 1..parts.len()).rev().map(|index| (index, after)));
            for (index, side) in order {
                let part_sized = parts[index].size.is_some();
                if let Some(block) = rep(&parts[index].node, part_sized) {
                    plan.push(step(
                        block,
                        beside(side, &anchor_block.id),
                        parts[index].size,
                    ));
                }
            }
            for (index, part) in parts.iter().enumerate() {
                let part_sized = if index == anchor {
                    sized
                } else {
                    part.size.is_some()
                };
                expand(&part.node, part_sized, plan);
            }
        }
    }
}

/// The plan of `tree`.
pub(super) fn plan(tree: &Node) -> Vec<Step> {
    let mut plan = Vec::new();
    if let Some(first) = rep(tree, false) {
        plan.push(step(first, Place::First, None));
        expand(tree, false, &mut plan);
    }
    plan
}
