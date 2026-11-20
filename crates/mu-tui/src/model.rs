//! The model of the terminal client: one value that the loop owns. [`update`](crate::update)
//! changes it for each message. [`view`](crate::view) records what each pane drew, and returns
//! a scrolled pane to the bottom when the newest row shows.

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use mu_client::ClientError;
use mu_proto::World as ProtoWorld;
use ratatui::style::Style;
use ratatui_textarea::{CursorMove, TextArea};

use crate::keymap::Keymap;
use crate::layout::Node;
use crate::pane::{Caps, PaneState};

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
    /// The SGR style at the end of the last game line, which the next game line starts with.
    pub(crate) style: Style,
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
    /// What the terminal can draw, which every pane shares.
    pub(crate) caps: Arc<Caps>,
    next_pane: u32,
    pub(crate) input: TextArea<'static>,
    pub(crate) lost: bool,
    pub(crate) keymap: Keymap,
    pub(crate) layout: Node,
    pub(crate) late: Late,
    /// The notices of the client, which a reset of the panes keeps.
    pub(crate) notices: Vec<String>,
    /// True if a late reply gave mode 2027 as set before the probe.
    pub(crate) keeps_grapheme_mode: bool,
}

/// The wait for the probe replies that came after the probe stopped reading. Termina gives the
/// kitty graphics reply as keys: Alt+`_`, the keys of its text, then Alt+`\`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Late {
    /// Every reply came, or the probe did not stop early.
    #[default]
    None,
    /// The replies after the probe end with DA1.
    Waiting,
    /// A graphics reply started with Alt+`_`, and `keys` of its text came.
    Graphics { keys: usize },
}

impl Model {
    /// A model with these worlds, the default keymap and the default layout. The worlds are in
    /// order by game and then by character, and the first world in that order has the focus.
    #[must_use]
    pub fn new(worlds: impl IntoIterator<Item = String>) -> Self {
        let mut model = Self {
            worlds: Vec::new(),
            focus: None,
            panes: HashMap::from([(PaneId::NOTICES, PaneState::default())]),
            caps: Arc::default(),
            next_pane: 1,
            input: input_line(String::new()),
            lost: false,
            keymap: Keymap::default(),
            layout: Node::default(),
            late: Late::None,
            notices: Vec::new(),
            keeps_grapheme_mode: false,
        };
        for id in worlds {
            model.world(&id);
        }
        model.focus = (!model.worlds.is_empty()).then_some(0);
        model
    }

    /// A model with the worlds of a list of worlds. A failed list gives no worlds and a notice
    /// with its error. The events of each world add the world later.
    #[must_use]
    pub fn from_listing(listed: Result<Vec<ProtoWorld>, ClientError>) -> Self {
        match listed {
            Ok(worlds) => Self::new(worlds.into_iter().map(|world| world.id)),
            Err(error) => {
                let mut model = Self::new([]);
                model.notice(&format!("could not list the worlds: {error}"));
                model
            }
        }
    }

    /// Adds a notice that the query for the window size failed with `error`.
    pub fn window_size_failed(&mut self, error: &io::Error) {
        self.notice(&format!("the terminal gave no window size: {error}"));
    }

    /// True if a late reply gave mode 2027 as set before the probe, so the client leaves the
    /// mode on at exit.
    #[must_use]
    pub const fn keeps_grapheme_mode(&self) -> bool {
        self.keeps_grapheme_mode
    }

    /// Drops the keys of a kitty graphics reply until a DA1 reply comes. The update of the probe
    /// result calls this when the probe stopped before DA1, so the rest of its replies come as terminal events.
    pub(crate) fn await_late_replies(&mut self) {
        self.late = Late::Waiting;
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

    /// The index of `id`, which this adds in its order, with a new pane, if it is new. A new
    /// world takes the focus when no world has it, with the notices in its pane, and the focus
    /// stays on its world.
    pub(crate) fn world(&mut self, id: &str) -> usize {
        match self
            .worlds
            .binary_search_by(|world| order(&world.id).cmp(&order(id)))
        {
            Ok(index) => index,
            Err(index) => {
                let pane = PaneId(self.next_pane);
                self.next_pane += 1;
                self.panes
                    .insert(pane, PaneState::with_caps(Arc::clone(&self.caps)));
                self.worlds.insert(
                    index,
                    World {
                        id: id.to_owned(),
                        pane,
                        unseen: false,
                        style: Style::default(),
                    },
                );
                match self.focus {
                    Some(focus) if focus >= index => self.focus = Some(focus + 1),
                    None => {
                        self.focus = Some(index);
                        for line in self.notices.clone() {
                            self.push(pane, &line);
                        }
                    }
                    Some(_) => {}
                }
                index
            }
        }
    }

    /// Wraps and draws every pane for a terminal with `caps`.
    pub(crate) fn set_caps(&mut self, caps: Caps) {
        self.caps = Arc::new(caps);
        for pane in self.panes.values_mut() {
            pane.set_caps(Arc::clone(&self.caps));
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

    /// Adds the system line `-- <text>` at the bottom of the pane in focus. After a Reset
    /// clears the panes of the worlds, the line comes back in the pane of the world in focus.
    pub(crate) fn notice(&mut self, text: &str) {
        let line = format!("-- {text}");
        self.push(self.focused_pane(), &line);
        self.notices.push(line);
    }

    /// Adds `line` at the bottom of `pane`.
    pub(crate) fn push(&mut self, pane: PaneId, line: &str) {
        self.panes
            .entry(pane)
            .or_insert_with(|| PaneState::with_caps(Arc::clone(&self.caps)))
            .push(line);
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
