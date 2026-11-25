//! The catalog of blocks, the plan of a layout, its tree, the reader and the writer.

use mu_config::layout::{BlockId, BlockKind, Edge, Layout, Place, Side, Size, Step, WORDS};
use proptest::prelude::*;

fn layout(lines: &[&str]) -> Layout {
    Layout::parse(lines).unwrap_or_else(|error| panic!("{lines:?}: {error}"))
}

fn error(lines: &[&str]) -> String {
    Layout::parse(lines)
        .expect_err("the plan is bad")
        .to_string()
}

fn kind() -> impl Strategy<Value = BlockKind> {
    prop_oneof![
        Just(BlockKind::Tabs),
        Just(BlockKind::Output),
        Just(BlockKind::Editor),
        Just(BlockKind::Statusline),
    ]
}

fn id() -> impl Strategy<Value = BlockId> {
    "[a-z][a-z0-9-]{0,31}"
        .prop_filter("a word of the sentences", |text| {
            !WORDS.contains(&text.as_str())
        })
        .prop_map(|text| BlockId::new(&text).expect("the pattern gives an id"))
}

fn place() -> impl Strategy<Value = Place> {
    prop_oneof![
        Just(Place::First),
        (
            prop::sample::select(Side::ALL.to_vec()),
            prop::collection::vec(id(), 1..4)
        )
            .prop_map(|(side, anchors)| Place::Beside { side, anchors }),
        prop::sample::select(Edge::ALL.to_vec()).prop_map(Place::At),
        id().prop_map(Place::With),
    ]
}

fn size() -> impl Strategy<Value = Option<Size>> {
    prop_oneof![
        Just(None),
        (10_u8..=90).prop_map(|percent| Some(Size::Percent(percent))),
        (1_u16..=u16::MAX).prop_map(|cells| Some(Size::Cells(cells))),
        (1_u16..=100).prop_map(|rows| Some(Size::Max(rows))),
    ]
}

fn step() -> impl Strategy<Value = Step> {
    (kind(), prop::option::of(id()), place(), size()).prop_map(|(kind, id, place, size)| Step {
        kind,
        id: id.unwrap_or_else(|| BlockId::of(kind)),
        place,
        size,
    })
}

/// One try to add a line to a plan. Each number picks by its rest after a division.
#[derive(Clone, Debug)]
struct Move {
    kind: BlockKind,
    form: u8,
    side: Side,
    edge: Edge,
    anchors: Vec<usize>,
    size: Option<Size>,
}

fn a_move() -> impl Strategy<Value = Move> {
    (
        kind(),
        0_u8..8,
        prop::sample::select(Side::ALL.to_vec()),
        prop::sample::select(Edge::ALL.to_vec()),
        prop::collection::vec(0_usize..64, 1..4),
        size(),
    )
        .prop_map(|(kind, form, side, edge, anchors, size)| Move {
            kind,
            form,
            side,
            edge,
            anchors,
            size,
        })
}

/// A good layout from `moves`: each move that gives a bad plan is left out.
fn grown(moves: &[Move]) -> Layout {
    let mut steps = vec![Step {
        kind: BlockKind::Editor,
        id: BlockId::of(BlockKind::Editor),
        place: Place::First,
        size: None,
    }];
    for (number, each) in moves.iter().enumerate() {
        let pick = |index: usize| steps[index % steps.len()].id.clone();
        let place = match each.form {
            0 => Place::At(each.edge),
            1 => Place::With(pick(each.anchors[0])),
            2 | 3 => Place::Beside {
                side: each.side,
                anchors: each.anchors.iter().map(|index| pick(*index)).collect(),
            },
            _ => Place::Beside {
                side: each.side,
                anchors: vec![pick(each.anchors[0])],
            },
        };
        let mut next = steps.clone();
        next.push(Step {
            kind: each.kind,
            id: BlockId::new(&format!("b{number}")).expect("an id"),
            place,
            size: each.size,
        });
        if Layout::from_steps(next.clone()).is_ok() {
            steps = next;
        }
    }
    Layout::from_steps(steps).expect("each kept plan is good")
}

proptest! {
    #[test]
    fn a_line_reads_back(step in step()) {
        let line = step.to_string();
        prop_assert_eq!(Step::parse(&line), Ok(step), "{}", line);
    }

    #[test]
    fn any_layout_reads_back(moves in prop::collection::vec(a_move(), 0..24)) {
        let layout = grown(&moves);
        let written = layout.written();
        prop_assert!(written.is_ok(), "{:?}: {:?}", layout.lines(), written);
        let written = written.expect("checked");
        prop_assert_eq!(written.tree(), layout.tree(), "{:?} -> {:?}", layout.lines(), written.lines());
    }
}

#[test]
fn the_classic_plan_builds() {
    let classic = layout(&["output", "tabs above output", "editor below output"]);
    assert_eq!(classic.lines().len(), 3);
    assert_eq!(classic.tree().levels(), 1);
}

#[test]
fn an_unknown_block() {
    let text = error(&["editor", "spawns above editor"]);
    assert!(text.contains("line 2") && text.contains("spawns"), "{text}");
}
