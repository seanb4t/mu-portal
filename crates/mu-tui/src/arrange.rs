//! Arrange mode: the player moves, stacks, adds, hides and resizes the blocks of the screen
//! with keys or with a drag, and sees each step. Enter saves, and Esc restores.

use mu_config::layout::{
    Area, BLOCKS, Block, BlockId, BlockKind, Change, Layout, Place, Placed, Side, Solved, Step,
};
use termina::event::{KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::model::Model;
use crate::registry::EntryId;
use crate::update::Effect;

/// The keys of the mode, which the last row shows.
pub const KEYS: &str =
    "Enter save  Esc cancel  a-z pick  arrow place  = stack  <> size  - hide  + add";

/// What the player picked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Pick {
    /// A block of the screen.
    Shown(BlockId),
    /// A new block of this kind, with this id.
    New(BlockKind, BlockId),
}

impl Pick {
    fn id(&self) -> &BlockId {
        match self {
            Self::Shown(id) | Self::New(_, id) => id,
        }
    }
}

/// The part of a block where a dragged block lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Drop {
    /// On this side of the block.
    Beside(Side),
    /// In the pane of the block, as a stack.
    Stack,
}

/// The state of arrange mode.
#[derive(Clone, Debug)]
pub(crate) struct Arrange {
    /// The layout from before the mode, which Esc restores.
    before: Layout,
    /// The layouts before each change of the mode, for the undo.
    undo: Vec<Layout>,
    /// The picked block.
    pub(crate) picked: Option<Pick>,
    /// The second block, which the picked block goes beside.
    pub(crate) second: Option<BlockId>,
    /// Whether the last row lists the blocks that the layout can take.
    pub(crate) adding: bool,
    /// The reason of the last move that the layout rules refused.
    pub(crate) message: Option<String>,
    /// The block of a drag, and the cell of the mouse.
    pub(crate) drag: Option<(BlockId, u16, u16)>,
}

/// The letter of the block at `index`, from `a`.
fn letter(index: usize) -> Option<char> {
    u8::try_from(index)
        .ok()
        .filter(|index| *index < 26)
        .map(|index| char::from(b'a' + index))
}

/// The blocks of the screen in the order of their letters.
pub(crate) fn shown(layout: &Layout) -> Vec<Block> {
    layout.tree().blocks().cloned().collect()
}

/// Each kind of block that the layout can still take, with the id of a new block of it.
pub(crate) fn addable(layout: &Layout) -> Vec<(BlockKind, BlockId)> {
    BLOCKS
        .iter()
        .filter(|row| layout.ids_of(row.kind).count() < row.max)
        .filter_map(|row| {
            let taken = |id: &str| layout.steps().iter().any(|step| step.id.as_str() == id);
            // A layout holds a block at most `max` times, so one of these ids is free.
            let id = std::iter::once(row.name.to_owned())
                .chain((2..=row.max + 1).map(|number| format!("{}-{number}", row.name)))
                .find(|id| !taken(id))?;
            BlockId::new(&id).ok().map(|id| (row.kind, id))
        })
        .collect()
}

/// The upper letter of each block of `pane`, as the screen marks it.
pub(crate) fn marks(layout: &Layout, pane: &Placed) -> String {
    let all = shown(layout);
    pane.blocks
        .iter()
        .filter_map(|block| {
            let index = all.iter().position(|each| each.id == block.id)?;
            letter(index).map(|c| c.to_ascii_uppercase())
        })
        .collect()
}

/// The cells of the part at one edge of a pane of `extent` cells, where a drop goes beside
/// the pane: a third, and one cell at least.
fn third(extent: u16) -> u16 {
    (extent / 3).max(1)
}

/// The part of `area` at the cell: the outer third of a side, or the middle.
pub(crate) fn drop_at(area: Area, column: u16, row: u16) -> Drop {
    let (x, y) = (column.saturating_sub(area.x), row.saturating_sub(area.y));
    if x < third(area.width) && area.width >= 3 {
        Drop::Beside(Side::Left)
    } else if x >= area.width.saturating_sub(third(area.width)) && area.width >= 3 {
        Drop::Beside(Side::Right)
    } else if y < third(area.height) && area.height >= 3 {
        Drop::Beside(Side::Above)
    } else if y >= area.height.saturating_sub(third(area.height)) && area.height >= 3 {
        Drop::Beside(Side::Below)
    } else {
        Drop::Stack
    }
}

/// The cells of `area` that the mark of `drop` covers.
pub(crate) fn drop_area(area: Area, drop: Drop) -> Area {
    match drop {
        Drop::Stack => area,
        Drop::Beside(Side::Left) => Area {
            width: third(area.width),
            ..area
        },
        Drop::Beside(Side::Right) => Area {
            x: area.x + area.width.saturating_sub(third(area.width)),
            width: third(area.width),
            ..area
        },
        Drop::Beside(Side::Above) => Area {
            height: third(area.height),
            ..area
        },
        Drop::Beside(Side::Below) => Area {
            y: area.y + area.height.saturating_sub(third(area.height)),
            height: third(area.height),
            ..area
        },
    }
}

/// The pane under the cell that a dragged block can land on, and the part of it.
pub(crate) fn drop_target(
    solved: &Solved,
    from: &BlockId,
    column: u16,
    row: u16,
) -> Option<(Placed, Drop)> {
    let pane = solved.pane_at(column, row)?;
    if pane.blocks.iter().any(|block| &block.id == from) {
        return None;
    }
    Some((pane.clone(), drop_at(pane.area, column, row)))
}

impl Model {
    /// Starts arrange mode.
    pub(crate) fn start_arrange(&mut self) {
        self.arrange = Some(Arrange {
            before: self.layout().clone(),
            undo: Vec::new(),
            picked: None,
            second: None,
            adding: false,
            message: None,
            drag: None,
        });
    }

    /// Whether arrange mode is on.
    #[must_use]
    pub fn arranges(&self) -> bool {
        self.arrange.is_some()
    }

    /// The last row of the screen in arrange mode: the reason of a refused move, the blocks
    /// that the layout can take, the pick, or the keys of the mode.
    pub(crate) fn arrange_row(&self) -> String {
        let Some(mode) = &self.arrange else {
            return String::new();
        };
        if let Some(message) = &mode.message {
            return message.clone();
        }
        let count = shown(self.layout()).len();
        if mode.adding {
            let list: Vec<String> = addable(self.layout())
                .iter()
                .enumerate()
                .filter_map(|(index, (kind, _))| {
                    Some(format!("{} {}", letter(count + index)?, kind.name()))
                })
                .collect();
            return format!("add: {}", list.join("  "));
        }
        match (&mode.picked, &mode.second) {
            (Some(pick), Some(second)) => {
                format!("{} to {second}: an arrow places it, = stacks it", pick.id())
            }
            (Some(pick), None) => format!(
                "{}: a letter and an arrow place it, < > size it, - hides it, Esc drops it",
                pick.id()
            ),
            _ => KEYS.to_owned(),
        }
    }

    /// Applies `change` in arrange mode. The screen shows the result, and nothing is saved. A
    /// change that the layout rules refuse changes nothing, and the last row shows the reason.
    fn arrange_change(&mut self, change: Change) {
        let before = self.layout().clone();
        let changed = before.apply(change);
        if self.arrange_show(before, changed)
            && let Some(mode) = &mut self.arrange
        {
            mode.picked = None;
            mode.second = None;
            mode.adding = false;
        }
    }

    /// Shows the result of a change of the mode to the layout `before`, and returns whether
    /// the change is good. A good change goes on the screen, and `before` goes on the undo
    /// list. A refused change changes nothing and shows its reason.
    fn arrange_show(
        &mut self,
        before: Layout,
        changed: Result<mu_config::layout::Changed, mu_config::layout::LayoutError>,
    ) -> bool {
        let (shown, message) = match changed {
            Ok(changed) => (Some(changed.layout), None),
            Err(error) => (None, Some(error.to_string())),
        };
        let good = shown.is_some();
        if let Some(layout) = shown {
            self.show_layout(layout);
        }
        if let Some(mode) = &mut self.arrange {
            if good {
                mode.undo.push(before);
            }
            mode.message = message;
        }
        good
    }

    /// The step that places the pick with `place`.
    fn arrange_step(&self, pick: &Pick, place: Place) -> Option<Step> {
        let kind = match pick {
            Pick::New(kind, _) => *kind,
            Pick::Shown(id) => {
                self.layout()
                    .steps()
                    .iter()
                    .find(|step| &step.id == id)?
                    .kind
            }
        };
        Some(Step {
            kind,
            id: pick.id().clone(),
            place,
            size: None,
        })
    }

    /// A plain letter: it picks a block, or names the second block.
    fn arrange_letter(&mut self, typed: char) {
        let blocks = shown(self.layout());
        let new_blocks = addable(self.layout());
        let Some(mode) = &mut self.arrange else {
            return;
        };
        mode.message = None;
        let index = usize::from(u8::try_from(typed).unwrap_or(b'a').saturating_sub(b'a'));
        if mode.adding {
            mode.adding = false;
            match index
                .checked_sub(blocks.len())
                .and_then(|at| new_blocks.get(at))
            {
                Some((kind, id)) => mode.picked = Some(Pick::New(*kind, id.clone())),
                None => mode.message = Some(format!("{typed} is no block of the list")),
            }
            return;
        }
        let Some(block) = blocks.get(index) else {
            mode.message = Some(format!("{typed} is no block of the screen"));
            return;
        };
        if mode.picked.is_none() {
            mode.picked = Some(Pick::Shown(block.id.clone()));
        } else {
            mode.second = Some(block.id.clone());
        }
    }

    /// Makes the picked block larger or smaller by one step.
    fn arrange_size(&mut self, steps: i32) {
        let Some(Pick::Shown(id)) = self.arrange.as_ref().and_then(|mode| mode.picked.clone())
        else {
            return;
        };
        let solved = crate::view::solved_now(self);
        let Some(border) = solved.borders.iter().find(|border| border.owner == id) else {
            if let Some(mode) = &mut self.arrange {
                mode.message = Some(format!("{id} takes no size"));
            }
            return;
        };
        let cells = border.owner_end.saturating_sub(border.owner_start);
        let before = self.layout().clone();
        let changed = before.resize_by(&id, steps, cells, border.total);
        self.arrange_show(before, changed);
    }

    /// Leaves the mode and saves the layout, if the mode changed it.
    fn arrange_save(&mut self) -> Vec<Effect> {
        let Some(mode) = self.arrange.take() else {
            return Vec::new();
        };
        if &mode.before == self.layout() {
            return Vec::new();
        }
        self.save_shown()
    }

    /// A key in arrange mode. No key reaches the editor or the keymap.
    pub(crate) fn arrange_key(&mut self, key: KeyEvent) -> Vec<Effect> {
        let plain = key.modifiers.is_empty() || key.modifiers == Modifiers::SHIFT;
        let (picked, second) = self.arrange.as_ref().map_or((None, None), |mode| {
            (mode.picked.clone(), mode.second.clone())
        });
        match key.code {
            // A plain letter always picks a block, even if a layer binds it to an entry.
            KeyCode::Char(typed) if key.modifiers.is_empty() && typed.is_ascii_lowercase() => {
                self.arrange_letter(typed);
            }
            KeyCode::Enter if plain => return self.arrange_save(),
            KeyCode::Escape if plain => {
                let Some(mode) = &mut self.arrange else {
                    return Vec::new();
                };
                if mode.picked.is_some() || mode.adding {
                    mode.picked = None;
                    mode.second = None;
                    mode.adding = false;
                    mode.message = None;
                } else {
                    let before = mode.before.clone();
                    self.arrange = None;
                    self.show_layout(before);
                }
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right if plain => {
                let side = match key.code {
                    KeyCode::Up => Side::Above,
                    KeyCode::Down => Side::Below,
                    KeyCode::Left => Side::Left,
                    _ => Side::Right,
                };
                if let (Some(pick), Some(second)) = (picked, second) {
                    let place = Place::Beside {
                        side,
                        anchors: vec![second],
                    };
                    if let Some(step) = self.arrange_step(&pick, place) {
                        self.arrange_change(Change::Place(step));
                    }
                }
            }
            KeyCode::Char('=') => {
                if let (Some(pick), Some(second)) = (picked, second)
                    && let Some(step) = self.arrange_step(&pick, Place::With(second))
                {
                    self.arrange_change(Change::Place(step));
                }
            }
            KeyCode::Char('>') => self.arrange_size(1),
            KeyCode::Char('<') => self.arrange_size(-1),
            KeyCode::Char('-') => {
                if let Some(Pick::Shown(id)) = picked {
                    self.arrange_change(Change::Hide(id));
                }
            }
            KeyCode::Char('+') => {
                if let Some(mode) = &mut self.arrange
                    && mode.picked.is_none()
                {
                    mode.adding = true;
                    mode.message = None;
                }
            }
            _ => {
                if self.keymap.get(&key) == Some(EntryId::of("undo")) {
                    let last = self.arrange.as_mut().and_then(|mode| mode.undo.pop());
                    if let Some(last) = last {
                        self.show_layout(last);
                    }
                }
            }
        }
        Vec::new()
    }

    /// A mouse event in arrange mode: a drag from one block onto another moves the first.
    pub(crate) fn arrange_mouse(&mut self, mouse: MouseEvent) {
        let solved = crate::view::solved_now(self);
        let (column, row) = (mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let from = solved
                    .pane_at(column, row)
                    .and_then(|pane| crate::view::front(self, pane))
                    .map(|block| block.id.clone());
                if let (Some(mode), Some(from)) = (&mut self.arrange, from) {
                    mode.drag = Some((from, column, row));
                    mode.message = None;
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some((_, at_column, at_row)) =
                    self.arrange.as_mut().and_then(|mode| mode.drag.as_mut())
                {
                    (*at_column, *at_row) = (column, row);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                let Some((from, _, _)) = self.arrange.as_mut().and_then(|mode| mode.drag.take())
                else {
                    return;
                };
                let Some((pane, drop)) = drop_target(&solved, &from, column, row) else {
                    return;
                };
                let Some(target) = crate::view::front(self, &pane).map(|block| block.id.clone())
                else {
                    return;
                };
                let place = match drop {
                    Drop::Stack => Place::With(target),
                    Drop::Beside(side) => Place::Beside {
                        side,
                        anchors: vec![target],
                    },
                };
                if let Some(step) = self.arrange_step(&Pick::Shown(from), place) {
                    self.arrange_change(Change::Place(step));
                }
            }
            _ => {}
        }
    }
}
