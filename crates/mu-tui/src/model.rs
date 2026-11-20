//! The model of the terminal client: one value that the loop owns and only
//! [`update`](crate::update) changes.

use std::collections::HashMap;

use ratatui::style::Style;
use ratatui_textarea::{CursorMove, TextArea};

use crate::keymap::Keymap;
use crate::layout::Node;
use crate::pane::PaneState;

/// The id of an output pane. Each world has one pane, and one more pane holds the lines of no
/// world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PaneId(u32);

impl PaneId {
    /// The pane of notices for no world.
    pub const NOTICES: Self = Self(0);
}

#[derive(Debug)]
pub(crate) struct World {
    pub(crate) id: String,
    pub(crate) pane: PaneId,
    pub(crate) unseen: bool,
}

/// The order of the worlds: by the game part, then by the character part.
fn order(id: &str) -> (&str, &str) {
    id.split_once('/').unwrap_or((id, ""))
}

/// The state of the terminal client.
#[derive(Debug)]
pub struct Model {
    pub(crate) worlds: Vec<World>,
    /// An index into `worlds`. None if and only if there is no world.
    pub(crate) focus: Option<usize>,
    pub(crate) panes: HashMap<PaneId, PaneState>,
    next_pane: u32,
    pub(crate) input: TextArea<'static>,
    pub(crate) lost: bool,
    pub(crate) keymap: Keymap,
    pub(crate) layout: Node,
}

impl Model {
    /// A model with these worlds, in their order, the default keymap and the default layout.
    /// The first world has the focus.
    #[must_use]
    pub fn new(worlds: impl IntoIterator<Item = String>) -> Self {
        let mut model = Self {
            worlds: Vec::new(),
            focus: None,
            panes: HashMap::from([(PaneId::NOTICES, PaneState::default())]),
            next_pane: 1,
            input: input_line(String::new()),
            lost: false,
            keymap: Keymap::default(),
            layout: Node::default(),
        };
        for id in worlds {
            model.world(&id);
        }
        model.focus = (!model.worlds.is_empty()).then_some(0);
        model
    }

    /// The id of the world in focus.
    #[must_use]
    pub fn focused(&self) -> Option<&str> {
        self.focus
            .and_then(|index| self.worlds.get(index))
            .map(|world| world.id.as_str())
    }

    /// The ids of the worlds, in their order.
    #[must_use]
    pub fn world_ids(&self) -> Vec<&str> {
        self.worlds.iter().map(|world| world.id.as_str()).collect()
    }

    /// The text of the input line.
    #[must_use]
    pub fn input(&self) -> &str {
        self.input.lines().first().map_or("", String::as_str)
    }

    /// The index of `id`, which this adds in its order, with a new pane, if it is new.
    pub(crate) fn world(&mut self, id: &str) -> usize {
        match self
            .worlds
            .binary_search_by(|world| order(&world.id).cmp(&order(id)))
        {
            Ok(index) => index,
            Err(index) => {
                let pane = PaneId(self.next_pane);
                self.next_pane += 1;
                self.panes.insert(pane, PaneState::default());
                self.worlds.insert(
                    index,
                    World {
                        id: id.to_owned(),
                        pane,
                        unseen: false,
                    },
                );
                match self.focus {
                    Some(focus) if focus >= index => self.focus = Some(focus + 1),
                    None => self.focus = Some(index),
                    Some(_) => {}
                }
                index
            }
        }
    }

    /// The state of the pane `id`.
    #[must_use]
    pub fn pane(&self, id: PaneId) -> Option<&PaneState> {
        self.panes.get(&id)
    }

    /// The pane of the world in focus, or the pane of notices.
    #[must_use]
    pub fn focused_pane(&self) -> PaneId {
        self.focus
            .and_then(|index| self.worlds.get(index))
            .map_or(PaneId::NOTICES, |world| world.pane)
    }

    /// Adds `line` at the bottom of `pane`.
    pub(crate) fn push(&mut self, pane: PaneId, line: &str) {
        self.panes.entry(pane).or_default().push(line);
    }

    /// Puts `text` in the input line, with the cursor at its end.
    pub(crate) fn set_input(&mut self, text: String) {
        self.input = input_line(text);
    }
}

fn input_line(text: String) -> TextArea<'static> {
    let mut input = TextArea::new(vec![text]);
    input.set_cursor_line_style(Style::default());
    input.move_cursor(CursorMove::End);
    input
}
