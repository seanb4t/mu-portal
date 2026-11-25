//! [`view`]: draws the model. The solver of `mu-config` gives the rectangle of each pane of the
//! layout, and one function draws each kind of block.

use mu_config::WorldId;
use mu_config::layout::{Area, BLOCKS, Block, BlockId, BlockKind, Placed, Solved, solve};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Paragraph;

use crate::editor::Editor;
use crate::model::Model;
use crate::pane::OutputPane;
use crate::text::clean;

/// The function that draws the pane of one kind of block.
type Draw = fn(&mut Model, &Placed, &mut Frame);

/// The draw function of each block of the catalog.
const DRAWS: &[(BlockKind, Draw)] = &[
    (BlockKind::Tabs, draw_tabs),
    (BlockKind::Output, draw_output),
    (BlockKind::Editor, draw_editor),
    (BlockKind::Statusline, draw_statusline),
];

/// The name of each block that has a draw function, in the order of the table.
#[must_use]
pub fn drawn_blocks() -> Vec<&'static str> {
    DRAWS.iter().map(|(kind, _)| kind.name()).collect()
}

/// The name of each block of the catalog, in its order.
#[must_use]
pub fn catalog_blocks() -> Vec<&'static str> {
    BLOCKS.iter().map(|row| row.name).collect()
}

const fn rect(area: Area) -> Rect {
    Rect::new(area.x, area.y, area.width, area.height)
}

/// The rectangle of each pane of the layout on the screen, in an area of `width` columns and
/// `height` rows.
#[must_use]
pub fn solved(model: &Model, width: u16, height: u16) -> Solved {
    let world = model.focused().map(str::to_owned);
    let rows_of = |id: &BlockId, columns: u16| {
        model
            .editors
            .get(&(world.clone(), id.clone()))
            .map_or(1, |state| {
                u16::try_from(state.rows(columns)).unwrap_or(u16::MAX)
            })
    };
    solve(model.layout(), Area::new(width, height), &rows_of)
}

/// Draws `model` on `frame`. A pane render can store what it drew in its state. The terminal
/// cursor goes to the insertion point of the editor with the input focus.
pub fn view(model: &mut Model, frame: &mut Frame) {
    let area = frame.area();
    model.screen = (area.width, area.height);
    let solved = solved_now(model);
    for divider in &solved.dividers {
        let column = vec!["│"; usize::from(divider.height)].join("\n");
        frame.render_widget(Paragraph::new(column), rect(*divider));
    }
    for pane in &solved.panes {
        let Some(kind) = pane.blocks.first().map(|block| block.kind) else {
            continue;
        };
        if let Some((_, draw)) = DRAWS.iter().find(|(each, _)| *each == kind) {
            draw(model, pane, frame);
        }
    }
    if model.arrange.is_some() {
        draw_arrange(model, &solved, frame);
    }
}

/// The rectangle of each pane of the layout on the screen, in the area of the last frame. In
/// arrange mode, the last row is for the keys of the mode.
#[must_use]
pub fn solved_now(model: &Model) -> Solved {
    let (width, height) = model.screen;
    let rows = if model.arrange.is_some() {
        height.saturating_sub(1)
    } else {
        height
    };
    solved(model, width, rows)
}

/// The marks of arrange mode: the letter of each block on the first cells of its pane, the part
/// where a dragged block lands, and the keys of the mode on the last row.
fn draw_arrange(model: &Model, solved: &Solved, frame: &mut Frame) {
    let marked = Style::default().add_modifier(Modifier::REVERSED);
    for pane in &solved.panes {
        let letters = crate::arrange::marks(model.layout(), pane);
        let row = Area {
            height: 1,
            width: pane
                .area
                .width
                .min(u16::try_from(letters.len()).unwrap_or(u16::MAX)),
            ..pane.area
        };
        frame.render_widget(Paragraph::new(letters).style(marked), rect(row));
    }
    let drag = model.arrange.as_ref().and_then(|mode| mode.drag.as_ref());
    if let Some((from, column, row)) = drag
        && let Some((pane, drop)) = crate::arrange::drop_target(solved, from, *column, *row)
    {
        let part = rect(crate::arrange::drop_area(pane.area, drop));
        frame.buffer_mut().set_style(part, marked);
    }
    let (width, height) = model.screen;
    if height > 0 {
        let last = Rect::new(0, height - 1, width, 1);
        frame.render_widget(Paragraph::new(model.arrange_row()), last);
    }
}

/// The block of `pane` that shows: the one in front of its stack.
pub(crate) fn front<'a>(model: &Model, pane: &'a Placed) -> Option<&'a Block> {
    model.front_of(&pane.blocks)
}

/// The text between two names of a title row.
const TITLE_GAP: &str = " ";

/// Each block of the title row of `pane`, with its text and its columns on the screen. The
/// block that shows has its id in square brackets.
pub(crate) fn title_cells<'a>(
    model: &Model,
    pane: &'a Placed,
) -> Vec<(&'a Block, String, std::ops::Range<u16>)> {
    let shown = front(model, pane).map(|block| &block.id);
    let gap = u16::try_from(TITLE_GAP.len()).unwrap_or(u16::MAX);
    let mut start = pane.area.x;
    pane.blocks
        .iter()
        .map(|block| {
            let text = if Some(&block.id) == shown {
                format!("[{}]", block.id)
            } else {
                block.id.to_string()
            };
            let end = start.saturating_add(u16::try_from(text.len()).unwrap_or(u16::MAX));
            let cell = (block, text, start..end);
            start = end.saturating_add(gap);
            cell
        })
        .collect()
}

/// The title row of `pane`: the id of each block, with the one that shows in square brackets.
pub(crate) fn title_row(model: &Model, pane: &Placed) -> String {
    title_cells(model, pane)
        .into_iter()
        .map(|(_, text, _)| text)
        .collect::<Vec<_>>()
        .join(TITLE_GAP)
}

/// The area of the lines of `pane`, below its title row.
pub(crate) const fn content(pane: &Placed) -> Area {
    if pane.title {
        Area {
            x: pane.area.x,
            y: pane.area.y.saturating_add(1),
            width: pane.area.width,
            height: pane.area.height.saturating_sub(1),
        }
    } else {
        pane.area
    }
}

fn draw_output(model: &mut Model, pane: &Placed, frame: &mut Frame) {
    if pane.title {
        let row = Area {
            height: 1,
            ..pane.area
        };
        frame.render_widget(Paragraph::new(title_row(model, pane)), rect(row));
    }
    let Some(id) = front(model, pane).map(|block| block.id.clone()) else {
        return;
    };
    if let Some(state) = model
        .pane_of_block(&id)
        .and_then(|pane| model.panes.get_mut(&pane))
    {
        frame.render_stateful_widget(OutputPane, rect(content(pane)), state);
    }
}

fn draw_editor(model: &mut Model, pane: &Placed, frame: &mut Frame) {
    let Some(id) = pane.blocks.first().map(|block| block.id.clone()) else {
        return;
    };
    let has_input = model.input_id() == id;
    let world = model.focused().map(str::to_owned);
    let state = model.editor_of(world.as_deref(), &id);
    frame.render_stateful_widget(Editor, rect(pane.area), state);
    // The editor with the input focus takes the keys, so the terminal cursor is its cursor.
    if has_input && let Some(cursor) = state.cursor() {
        frame.set_cursor_position(cursor);
    }
}

fn draw_tabs(model: &mut Model, pane: &Placed, frame: &mut Frame) {
    let row = Area {
        height: 1,
        ..pane.area
    };
    frame.render_widget(Paragraph::new(top_row(model)), rect(row));
}

fn draw_statusline(model: &mut Model, pane: &Placed, frame: &mut Frame) {
    let row = Area {
        height: 1,
        ..pane.area
    };
    let text = crate::statusline::row(model, row.width);
    frame.render_widget(Paragraph::new(text), rect(row));
}

fn top_row(model: &Model) -> String {
    let mut row = if model.first_list {
        "listing worlds".to_owned()
    } else if model.worlds.is_empty() {
        "no worlds".to_owned()
    } else {
        model
            .worlds
            .iter()
            .enumerate()
            .map(|(index, world)| {
                let mark = if world.unseen { "*" } else { "" };
                // An id that is no world id has no game and no character, so it shows as it is.
                let label = world.id.parse::<WorldId>().map_or_else(
                    |_| clean(&world.id),
                    |id| clean(&model.label.render(&id, index + 1)),
                );
                // The first send is with the server. The ones after it wait for its reply.
                let waiting = model
                    .sends
                    .get(&world.id)
                    .map_or(0, |queue| queue.len().saturating_sub(1));
                let waiting = if waiting == 0 {
                    String::new()
                } else {
                    format!(" ({waiting} waiting)")
                };
                if model.focus == Some(index) {
                    format!("[{label}]{mark}{waiting}")
                } else {
                    format!("{label}{mark}{waiting}")
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    if model.lost {
        row.push_str("  disconnected");
    }
    row
}
