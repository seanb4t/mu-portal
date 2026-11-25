//! The catalog of blocks, the plan of a layout, its tree, the reader and the writer.

use std::collections::BTreeSet;

use mu_config::layout::{
    BLOCKS, BUILT_IN, BlockId, BlockKind, Dir, Edge, Layout, LineRef, Node, PartSize, Place, Side,
    Size, SizeRule, Step, WORDS,
};
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

/// The tree as text, such as `rows[tabs, cols[output, notices:30%], editor]`.
fn shape(node: &Node) -> String {
    match node {
        Node::Pane(blocks) => blocks
            .iter()
            .map(|block| match block.max {
                Some(rows) => format!("{}:max{rows}", block.id),
                None => block.id.to_string(),
            })
            .collect::<Vec<_>>()
            .join("+"),
        Node::Split { dir, parts } => {
            let name = match dir {
                Dir::Rows => "rows",
                Dir::Columns => "cols",
            };
            let parts: Vec<String> = parts
                .iter()
                .map(|part| match part.size {
                    Some(PartSize::Percent(percent)) => format!("{}:{percent}%", shape(&part.node)),
                    Some(PartSize::Cells(cells)) => format!("{}:{cells}", shape(&part.node)),
                    None => shape(&part.node),
                })
                .collect();
            format!("{name}[{}]", parts.join(", "))
        }
    }
}

fn shape_of(lines: &[&str]) -> String {
    shape(layout(lines).tree())
}

fn id_of(text: &str) -> BlockId {
    BlockId::new(text).expect("an id")
}

fn step_of(text: &str) -> Step {
    Step::parse(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn ids(layout: &Layout) -> BTreeSet<String> {
    layout
        .tree()
        .blocks()
        .map(|block| block.id.to_string())
        .collect()
}

const SIDEBAR: &[&str] = &[
    "output",
    "tabs above output",
    "editor below output",
    "statusline below editor",
    "output as notices right of output, 30%",
];

// The catalog.

#[test]
fn the_editor_is_required() {
    let text = error(&["output", "tabs above output"]);
    assert!(text.contains("editor"), "{text}");
}

#[test]
fn a_second_editor_with_no_id() {
    let text = error(&["editor", "editor above editor"]);
    assert!(text.contains("line 2") && text.contains("editor"), "{text}");
}

fn editors(count: usize) -> Vec<String> {
    let mut lines = vec!["editor".to_owned()];
    lines.extend((1..count).map(|number| format!("editor as e{number} above editor")));
    lines
}

fn outputs(count: usize) -> Vec<String> {
    let mut lines = vec!["editor".to_owned()];
    lines.extend((0..count).map(|number| format!("output as o{number} above editor")));
    lines
}

#[test]
fn four_editors() {
    assert!(Layout::parse(&editors(4)).is_ok());
}

#[test]
fn five_editors() {
    let text = Layout::parse(&editors(5)).expect_err("five").to_string();
    assert!(text.contains("line 5") && text.contains("editor"), "{text}");
}

#[test]
fn eight_outputs() {
    assert!(Layout::parse(&outputs(8)).is_ok());
}

#[test]
fn nine_outputs() {
    let text = Layout::parse(&outputs(9)).expect_err("nine").to_string();
    assert!(
        text.contains("line 10") && text.contains("output"),
        "{text}"
    );
}

#[test]
fn only_outputs_stack() {
    let stacking: Vec<&str> = BLOCKS
        .iter()
        .filter(|row| row.stacks)
        .map(|row| row.name)
        .collect();
    assert_eq!(stacking, ["output"]);
}

#[test]
fn tabs_and_statusline_are_one_row() {
    let one_row: Vec<&str> = BLOCKS
        .iter()
        .filter(|row| row.size == SizeRule::OneRow)
        .map(|row| row.name)
        .collect();
    assert_eq!(one_row, ["tabs", "statusline"]);
}

// The reader and the build.

#[test]
fn an_unknown_block() {
    let text = error(&["editor", "spawns above editor"]);
    assert!(text.contains("line 2") && text.contains("spawns"), "{text}");
}

#[test]
fn a_block_twice() {
    let text = error(&["editor", "tabs above editor", "tabs below editor"]);
    assert!(text.contains("line 3") && text.contains("tabs"), "{text}");
}

#[test]
fn a_block_that_is_not_placed_yet() {
    let text = error(&["editor", "tabs above output", "output above editor"]);
    assert!(text.contains("line 2") && text.contains("output"), "{text}");
}

#[test]
fn an_id_twice() {
    let text = error(&[
        "editor",
        "output as main above editor",
        "output as main at right",
    ]);
    assert!(text.contains("line 3") && text.contains("main"), "{text}");
}

#[test]
fn an_id_that_is_a_word_of_the_sentences() {
    let text = error(&["editor", "output as above above editor"]);
    assert!(text.contains("line 2") && text.contains("above"), "{text}");
}

#[test]
fn an_id_of_32_characters() {
    let line = format!("output as {} above editor", "a".repeat(32));
    assert!(Layout::parse(&["editor", line.as_str()]).is_ok());
}

#[test]
fn an_id_of_33_characters() {
    let id = "a".repeat(33);
    let line = format!("output as {id} above editor");
    let text = error(&["editor", line.as_str()]);
    assert!(text.contains("line 2") && text.contains(&id), "{text}");
}

#[test]
fn anchors_that_are_not_neighbors() {
    let text = error(&[
        "output",
        "tabs above output",
        "editor below output",
        "output as notices right of tabs and editor",
    ]);
    assert!(
        text.contains("line 4") && text.contains("tabs") && text.contains("editor"),
        "{text}"
    );
}

#[test]
fn a_size_on_the_first_line() {
    let text = error(&["output, 50%", "editor below output"]);
    assert!(text.contains("line 1") && text.contains("50%"), "{text}");
}

#[test]
fn the_editor_after_the_notices() {
    assert_eq!(
        shape_of(&[
            "output",
            "output as notices right of output",
            "editor below output"
        ]),
        "cols[rows[output, editor], notices]"
    );
}

#[test]
fn the_editor_before_the_notices() {
    assert_eq!(
        shape_of(&[
            "output",
            "editor below output",
            "output as notices right of output"
        ]),
        "rows[cols[output, notices], editor]"
    );
}

#[test]
fn an_id_in_a_later_line() {
    assert_eq!(
        shape_of(&[
            "output",
            "editor below output",
            "output as side right of output",
            "tabs above side"
        ]),
        "rows[cols[output, rows[tabs, side]], editor]"
    );
}

#[test]
fn a_block_beside_two_rows() {
    assert_eq!(
        shape_of(&[
            "output",
            "tabs above output",
            "editor below output",
            "statusline below editor",
            "output as notices right of output and editor, 33%",
        ]),
        "rows[tabs, cols[rows[output, editor], notices:33%], statusline]"
    );
}

#[test]
fn no_editor() {
    let text = error(&["output"]);
    assert!(text.contains("editor"), "{text}");
}

#[test]
fn a_percent_out_of_range() {
    for size in ["9%", "91%", "95%"] {
        let line = format!("output above editor, {size}");
        let text = error(&["editor", line.as_str()]);
        assert!(text.contains("line 2") && text.contains(size), "{text}");
    }
}

#[test]
fn a_percent_of_90() {
    assert_eq!(
        shape_of(&["editor", "output above editor, 90%"]),
        "rows[output:90%, editor]"
    );
}

#[test]
fn a_count_of_zero_cells() {
    let text = error(&["editor", "output above editor, 0"]);
    assert!(text.contains("line 2") && text.contains('0'), "{text}");
}

#[test]
fn a_count_of_65535() {
    assert!(Layout::parse(&["editor", "output above editor, 65535"]).is_ok());
}

#[test]
fn a_count_of_65536() {
    let text = error(&["editor", "output above editor, 65536"]);
    assert!(text.contains("line 2") && text.contains("65536"), "{text}");
}

#[test]
fn too_many_lines() {
    let lines = vec!["editor"; 33];
    let text = Layout::parse(&lines).expect_err("33 lines").to_string();
    assert!(text.contains("32") && !text.contains("line 2"), "{text}");
}

#[test]
fn the_largest_plan() {
    let mut lines = editors(4);
    lines.extend((0..8).map(|number| format!("output as o{number} above editor")));
    lines.push("tabs at top".to_owned());
    lines.push("statusline at bottom".to_owned());
    assert_eq!(lines.len(), 14);
    assert!(Layout::parse(&lines).is_ok());
}

/// A plan whose tree has one more level for each output.
fn nested(outputs: usize) -> Vec<String> {
    let mut lines = vec![
        "editor".to_owned(),
        "output as o0 right of editor".to_owned(),
    ];
    for number in 1..outputs {
        let side = if number % 2 == 1 { "below" } else { "right of" };
        lines.push(format!("output as o{number} {side} o{}", number - 1));
    }
    lines
}

#[test]
fn a_tree_of_8_levels() {
    let layout = Layout::parse(&nested(8)).expect("8 levels");
    assert_eq!(layout.tree().levels(), 8);
}

#[test]
fn a_tree_too_deep() {
    let mut lines = nested(8);
    lines.push("tabs right of o7".to_owned());
    let text = Layout::parse(&lines).expect_err("9 levels").to_string();
    assert!(text.contains("line 10") && text.contains('8'), "{text}");
}

#[test]
fn the_editor_does_not_stack() {
    let text = error(&["output", "editor with output"]);
    assert!(text.contains("line 2") && text.contains("editor"), "{text}");
}

#[test]
fn a_size_on_the_tabs() {
    let text = error(&["editor", "tabs above editor, 3"]);
    assert!(text.contains("line 2") && text.contains('3'), "{text}");
}

#[test]
fn a_bad_editor_size() {
    for size in ["max 0", "max 101", "30%", "4"] {
        let line = format!("editor below output, {size}");
        let text = error(&["output", line.as_str()]);
        assert!(text.contains("line 2"), "{size}: {text}");
    }
}

#[test]
fn an_editor_max_of_1_and_100() {
    assert_eq!(
        shape_of(&["output", "editor below output, max 1"]),
        "rows[output, editor:max1]"
    );
    assert_eq!(
        shape_of(&["output", "editor below output, max 100"]),
        "rows[output, editor:max100]"
    );
}

// The built-in layouts.

#[test]
fn each_built_in_layout_is_valid() {
    for each in BUILT_IN {
        assert!(each.layout().is_ok(), "{}", each.name);
    }
    let names: Vec<&str> = BUILT_IN.iter().map(|each| each.name).collect();
    assert_eq!(names, ["classic", "focus", "sidebar", "status"]);
}

#[test]
fn a_written_plan_keeps_each_id() {
    for each in BUILT_IN {
        let layout = each.layout().expect("valid");
        let written = layout.written().expect("the writer gives a good plan");
        assert_eq!(ids(&written), ids(&layout), "{}", each.name);
        assert_eq!(written.tree(), layout.tree(), "{}", each.name);
    }
}

// The line edits.

#[test]
fn an_insert_in_the_middle() {
    let plan = layout(&["output", "editor below output"]);
    let next = plan
        .insert(2, step_of("output as notices right of output"))
        .expect("good");
    assert_eq!(
        next.lines(),
        [
            "output",
            "output as notices right of output",
            "editor below output"
        ]
    );
}

#[test]
fn an_insert_at_zero() {
    let plan = layout(&["output", "editor below output"]);
    let text = plan
        .insert(0, step_of("tabs above output"))
        .expect_err("0")
        .to_string();
    assert!(text.contains('0') && text.contains("1 to 3"), "{text}");
}

#[test]
fn an_insert_at_the_end() {
    let plan = layout(&["output", "editor below output"]);
    let next = plan.insert(3, step_of("tabs above output")).expect("good");
    assert_eq!(next.lines().len(), 3);
}

#[test]
fn an_insert_past_the_end() {
    let plan = layout(&["output", "editor below output"]);
    let text = plan
        .insert(4, step_of("tabs above output"))
        .expect_err("4")
        .to_string();
    assert!(text.contains('4'), "{text}");
}

#[test]
fn a_line_by_its_number_or_its_id() {
    let plan = layout(SIDEBAR);
    let by_number = plan
        .replace(
            &LineRef::Number(5),
            step_of("output as notices right of output, 40%"),
        )
        .expect("good");
    let by_id = plan
        .replace(
            &LineRef::Id(id_of("notices")),
            step_of("output as notices right of output, 40%"),
        )
        .expect("good");
    assert_eq!(by_number, by_id);
    assert_eq!(LineRef::parse("5"), Ok(LineRef::Number(5)));
    assert_eq!(LineRef::parse("notices"), Ok(LineRef::Id(id_of("notices"))));
}

#[test]
fn a_replace_of_line_zero() {
    let text = layout(SIDEBAR)
        .replace(&LineRef::Number(0), step_of("output"))
        .expect_err("0")
        .to_string();
    assert!(text.contains('0') && text.contains("1 to 5"), "{text}");
}

#[test]
fn a_replace_of_a_line_past_the_end() {
    let text = layout(SIDEBAR)
        .replace(&LineRef::Number(6), step_of("output"))
        .expect_err("6")
        .to_string();
    assert!(text.contains('6'), "{text}");
}

#[test]
fn a_delete_of_line_zero() {
    let text = layout(SIDEBAR)
        .delete(&LineRef::Number(0))
        .expect_err("0")
        .to_string();
    assert!(text.contains('0'), "{text}");
}

#[test]
fn a_delete_of_a_line_past_the_end() {
    let text = layout(SIDEBAR)
        .delete(&LineRef::Number(6))
        .expect_err("6")
        .to_string();
    assert!(text.contains('6'), "{text}");
}

#[test]
fn a_delete_of_an_anchor_names_its_lines() {
    let text = layout(SIDEBAR)
        .delete(&LineRef::Id(id_of("editor")))
        .expect_err("an anchor")
        .to_string();
    assert!(text.contains("editor") && text.contains("line 4"), "{text}");
}

// The screen changes.

#[test]
fn hide_refuses_the_editor() {
    let text = layout(SIDEBAR)
        .hide(&id_of("editor"))
        .expect_err("the editor")
        .to_string();
    assert!(text.contains("editor"), "{text}");
}

#[test]
fn a_resize_changes_one_line() {
    let changed = layout(SIDEBAR)
        .resize(&id_of("notices"), Size::Percent(40))
        .expect("good");
    assert!(!changed.rewritten);
    let mut wanted: Vec<String> = SIDEBAR.iter().map(ToString::to_string).collect();
    wanted[4] = "output as notices right of output, 40%".to_owned();
    assert_eq!(changed.layout.lines(), wanted);
}

#[test]
fn a_resize_out_of_range_is_refused() {
    let text = layout(SIDEBAR)
        .resize(&id_of("notices"), Size::Percent(95))
        .expect_err("95%")
        .to_string();
    assert!(text.contains("95%"), "{text}");
}

#[test]
fn a_size_of_the_first_block_is_refused() {
    assert!(
        layout(SIDEBAR)
            .resize(&id_of("output"), Size::Percent(60))
            .is_err()
    );
}

#[test]
fn a_size_of_the_first_block_names_the_sized_blocks() {
    let text = layout(SIDEBAR)
        .resize(&id_of("output"), Size::Percent(60))
        .expect_err("the first block")
        .to_string();
    assert!(text.contains("notices") && !text.contains("tabs"), "{text}");
}

#[test]
fn a_move_to_a_later_anchor_goes_to_the_end() {
    let changed = layout(SIDEBAR)
        .place(step_of("tabs below notices"))
        .expect("good");
    assert!(!changed.rewritten);
    assert_eq!(
        changed.layout.lines(),
        [
            "output",
            "editor below output",
            "statusline below editor",
            "output as notices right of output, 30%",
            "tabs below notices",
        ]
    );
}

#[test]
fn a_hide_of_an_anchor_writes_the_plan_again() {
    let plan = layout(&[
        "output",
        "editor below output",
        "output as notices right of output, 30%",
        "output as side below notices",
    ]);
    let changed = plan.hide(&id_of("notices")).expect("good");
    assert!(changed.rewritten);
    assert_eq!(
        ids(&changed.layout),
        BTreeSet::from(["output".to_owned(), "editor".to_owned(), "side".to_owned()])
    );
}

#[test]
fn a_border_gives_a_whole_percent() {
    let changed = layout(SIDEBAR)
        .resize_border(&id_of("notices"), 40, 80)
        .expect("good");
    assert_eq!(
        changed.layout.lines()[4],
        "output as notices right of output, 50%"
    );
}

#[test]
fn a_border_takes_the_smallest_percent_that_reaches_the_drag() {
    let plan = layout(&[
        "output",
        "editor below output",
        "output as notices below output, 50%",
    ]);
    let changed = plan.resize_border(&id_of("notices"), 8, 24).expect("good");
    assert_eq!(
        changed.layout.lines()[2],
        "output as notices below output, 34%"
    );
}

#[test]
fn a_border_stops_at_the_smallest_pane() {
    let changed = layout(SIDEBAR)
        .resize_border(&id_of("notices"), 2, 80)
        .expect("good");
    assert_eq!(
        changed.layout.lines()[4],
        "output as notices right of output, 13%"
    );
}

#[test]
fn a_border_stops_at_ten_percent() {
    let changed = layout(SIDEBAR)
        .resize_border(&id_of("notices"), 5, 300)
        .expect("good");
    assert_eq!(
        changed.layout.lines()[4],
        "output as notices right of output, 10%"
    );
}

#[test]
fn a_border_in_a_split_of_20_columns() {
    let plan = layout(SIDEBAR);
    let changed = plan.resize_border(&id_of("notices"), 12, 20).expect("good");
    assert_eq!(changed.layout, plan);
}

#[test]
fn a_border_in_a_split_of_3_rows() {
    let plan = layout(&[
        "output",
        "editor below output",
        "output as notices below output, 50%",
    ]);
    let changed = plan.resize_border(&id_of("notices"), 1, 3).expect("good");
    assert_eq!(changed.layout, plan);
}

/// One change of the screen for the property test.
#[derive(Clone, Debug)]
enum Change {
    Hide(usize),
    Resize(usize, Option<Size>),
    Move(usize, Move),
}

fn change() -> impl Strategy<Value = Change> {
    prop_oneof![
        (0_usize..64).prop_map(Change::Hide),
        (0_usize..64, size()).prop_map(|(index, size)| Change::Resize(index, size)),
        (0_usize..64, a_move()).prop_map(|(index, each)| Change::Move(index, each)),
    ]
}

proptest! {
    #[test]
    fn a_screen_change_gives_the_wanted_screen(
        moves in prop::collection::vec(a_move(), 0..16),
        change in change(),
    ) {
        let before = grown(&moves);
        let steps = before.steps();
        let pick = |index: usize| steps[index % steps.len()].clone();
        let (result, gone, moved) = match change {
            Change::Hide(index) => {
                let id = pick(index).id;
                (before.hide(&id), Some(id), None)
            }
            Change::Resize(index, size) => {
                let id = pick(index).id;
                (before.resize(&id, size.unwrap_or(Size::Percent(50))), None, None)
            }
            Change::Move(index, each) => {
                let old = pick(index);
                let anchor = pick(each.anchors[0]).id;
                let place = match each.form {
                    0 => Place::At(each.edge),
                    1 => Place::With(anchor),
                    _ => Place::Beside { side: each.side, anchors: vec![anchor] },
                };
                let step = Step { kind: old.kind, id: old.id, place, size: each.size };
                (before.place(step.clone()), None, Some(step))
            }
        };
        let Ok(changed) = result else { return Ok(()); };
        let after = &changed.layout;
        // The plan is good, and it builds the tree of the result.
        let again = Layout::parse(&after.lines());
        prop_assert_eq!(again.as_ref().map(Layout::tree), Ok(after.tree()));
        // Each id stays, but for a hidden block.
        let mut wanted = ids(&before);
        if let Some(id) = &gone {
            wanted.remove(id.as_str());
        }
        prop_assert_eq!(ids(after), wanted);
        // A change that mu did not write again keeps each other line.
        if !changed.rewritten {
            let touched = gone.as_ref().or(moved.as_ref().map(|step| &step.id));
            let other = |layout: &Layout| -> BTreeSet<String> {
                layout
                    .steps()
                    .iter()
                    .filter(|step| Some(&step.id) != touched)
                    .map(ToString::to_string)
                    .collect()
            };
            if touched.is_some() {
                prop_assert_eq!(other(after), other(&before));
            }
        }
        // A moved block has the place of its step in the plan that builds the screen.
        if let (Some(step), false) = (&moved, changed.rewritten) {
            prop_assert!(after.steps().contains(step), "{:?}", after.lines());
        }
    }
}

// The solver.

use mu_config::layout::{Area, preview, solve};

fn solved(lines: &[&str], width: u16, height: u16) -> mu_config::layout::Solved {
    solve(&layout(lines), Area::new(width, height), &|_, _| 1)
}

/// The rectangle of the pane of `id` as `(first column, last column, first row, last row)`,
/// each from 1.
fn rect(solved: &mu_config::layout::Solved, id: &str) -> (u16, u16, u16, u16) {
    let area = solved
        .pane_of(&id_of(id))
        .unwrap_or_else(|| panic!("no pane of {id}"))
        .area;
    (
        area.x + 1,
        area.x + area.width,
        area.y + 1,
        area.y + area.height,
    )
}

#[test]
fn the_sidebar_divider_is_in_column_56() {
    let solved = solved(SIDEBAR, 80, 24);
    assert_eq!(rect(&solved, "notices"), (57, 80, 2, 22));
    assert_eq!(rect(&solved, "output"), (1, 55, 2, 22));
    assert_eq!(
        solved.dividers,
        [Area {
            x: 55,
            y: 1,
            width: 1,
            height: 21
        }]
    );
    assert_eq!(rect(&solved, "tabs"), (1, 80, 1, 1));
    assert_eq!(rect(&solved, "statusline"), (1, 80, 24, 24));
}

#[test]
fn two_percents_past_the_split() {
    let solved = solved(
        &[
            "editor",
            "output as a right of editor, 60%",
            "output as b right of editor, 60%",
        ],
        100,
        10,
    );
    let (editor, a, b) = (
        rect(&solved, "editor"),
        rect(&solved, "a"),
        rect(&solved, "b"),
    );
    assert!(editor.1 >= editor.0, "{editor:?}");
    assert_eq!(a.1, 100);
    assert!(b.0 > editor.1 && a.0 > b.1, "{editor:?} {b:?} {a:?}");
}

#[test]
fn a_count_past_the_screen() {
    let solved = solved(&["editor", "output above editor, 500"], 80, 24);
    assert_eq!(rect(&solved, "output"), (1, 80, 1, 23));
    assert_eq!(rect(&solved, "editor"), (1, 80, 24, 24));
}

#[test]
fn two_large_counts_in_one_split() {
    let solved = solved(
        &[
            "editor",
            "output as a above editor, 65535",
            "output as b above editor, 65535",
        ],
        80,
        24,
    );
    assert_eq!(rect(&solved, "editor"), (1, 80, 24, 24));
}

proptest! {
    #[test]
    fn solve_stays_inside_the_area(
        moves in prop::collection::vec(a_move(), 0..24),
        width in 0_u16..=300,
        height in 0_u16..=100,
        rows in 0_u16..=u16::MAX,
    ) {
        let layout = grown(&moves);
        let area = Area::new(width, height);
        let solved = solve(&layout, area, &|_, _| rows);
        let cells = solved
            .panes
            .iter()
            .map(|pane| pane.area)
            .chain(solved.dividers.iter().copied());
        let mut taken = vec![false; usize::from(width) * usize::from(height)];
        for each in cells {
            prop_assert!(
                u32::from(each.x) + u32::from(each.width) <= u32::from(width)
                    && u32::from(each.y) + u32::from(each.height) <= u32::from(height),
                "{:?} in {:?}: {:?}", each, area, layout.lines()
            );
            for row in each.y..each.y + each.height {
                for column in each.x..each.x + each.width {
                    let index = usize::from(row) * usize::from(width) + usize::from(column);
                    prop_assert!(!taken[index], "{:?} twice: {:?}", each, layout.lines());
                    taken[index] = true;
                }
            }
        }
    }
}

#[test]
fn the_picture_of_the_sidebar() {
    let picture = preview(&layout(SIDEBAR), 40, 12).join("\n");
    println!("{picture}");
    assert!(
        picture.contains("notices") && picture.contains('┬'),
        "{picture}"
    );
}

// The streams.

#[test]
fn a_stream_goes_to_the_output_with_its_id() {
    assert_eq!(layout(SIDEBAR).route("notices"), Some(&id_of("notices")));
}

#[test]
fn a_stream_with_no_output_goes_to_the_first() {
    assert_eq!(layout(SIDEBAR).route("channels"), Some(&id_of("output")));
}

#[test]
fn a_plan_with_no_output() {
    assert_eq!(layout(&["editor"]).route("notices"), None);
}
