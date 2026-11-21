//! The model of the terminal client: one value that the loop owns. [`update`](crate::update)
//! changes it for each message. [`view`](crate::view) records what each pane drew, and returns
//! a scrolled pane to the bottom when the newest row shows.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use mu_client::ClientError;
use mu_proto::World as ProtoWorld;
use ratatui::style::Style;

use crate::editor::{EditorId, EditorState};
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
    /// The text that the game reads as a line break inside one command, from the list of the
    /// worlds. None for a world that has none, or that the list did not hold.
    pub(crate) line_break: Option<String>,
}

/// The lines of a send of each line that did not go yet.
#[derive(Debug)]
pub(crate) struct LineQueue {
    /// The world that had the focus at the key. Each line goes to it.
    pub(crate) world: String,
    /// The line that the server did not answer yet.
    pub(crate) sent: String,
    /// The lines after it, in order. The queue stays until the server answers its last line.
    pub(crate) rest: VecDeque<String>,
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
    pub(crate) editors: HashMap<EditorId, EditorState>,
    /// The lines of a send of each line that waits for the server.
    pub(crate) queue: Option<LineQueue>,
    /// The world and the text of each plain send that the server did not answer yet.
    pub(crate) plain_sends: Vec<(String, String)>,
    pub(crate) lost: bool,
    pub(crate) keymap: Keymap,
    pub(crate) layout: Node,
    pub(crate) late: Late,
    /// The notices of the client, which a reset of the panes keeps.
    pub(crate) notices: Vec<String>,
    /// True if a late answer gave mode 2027 as set before the probe.
    pub(crate) keeps_grapheme_mode: bool,
}

/// The wait for the probe answers that came after the probe stopped reading. Termina gives the
/// kitty graphics answer as keys: Alt+`_`, the keys of its text, then Alt+`\`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Late {
    /// Every answer came, or the probe did not stop early.
    #[default]
    None,
    /// The answers after the probe end with DA1.
    Waiting,
    /// A graphics answer started with Alt+`_`, and `keys` of its text came.
    Graphics { keys: usize },
    /// A graphics answer started with Alt+`_`, and the key Escape came. A `\` key ends the answer.
    GraphicsEscape,
    /// The probe stopped inside a CSI answer, or after the ESC of one. The held bytes start with
    /// ESC, and the keys of the rest of the answer add to them until the final byte. At most
    /// [`PENDING_MAX`](crate::probe::PENDING_MAX) bytes.
    Csi(Vec<u8>),
}

impl Model {
    /// A model with these worlds, the default keymap and the default layout. The worlds are in
    /// order by game and then by character, and the first world in that order has the focus.
    #[must_use]
    pub fn new(worlds: impl IntoIterator<Item = String>) -> Self {
        let caps = Arc::default();
        let mut model = Self {
            worlds: Vec::new(),
            focus: None,
            panes: HashMap::from([(PaneId::NOTICES, PaneState::default())]),
            editors: HashMap::from([(EditorId::POSE, EditorState::with_caps(Arc::clone(&caps)))]),
            queue: None,
            plain_sends: Vec::new(),
            caps,
            next_pane: 1,
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
            Ok(worlds) => {
                let mut model = Self::new(worlds.iter().map(|world| world.id.clone()));
                for listed in worlds {
                    let index = model.world(&listed.id);
                    model.worlds[index].line_break = listed.line_break;
                }
                model
            }
            Err(error) => {
                let mut model = Self::new([]);
                model.notice(&format!("could not list the worlds: {error}"));
                model
            }
        }
    }

    /// True if a late answer gave mode 2027 as set before the probe, so the client leaves the
    /// mode on at exit.
    #[must_use]
    pub const fn keeps_grapheme_mode(&self) -> bool {
        self.keeps_grapheme_mode
    }

    /// Waits for the late answers of the probe until DA1 comes. It drops the keys of a kitty
    /// graphics answer.
    pub(crate) fn await_late_replies(&mut self) {
        self.late = Late::Waiting;
    }

    /// Holds `bytes`, the start of a CSI answer that the probe cut, and rebuilds the answer from
    /// the keys after it. `bytes` start with ESC, and have `[` second if they hold more than one
    /// byte. They hold at most [`PENDING_MAX`](crate::probe::PENDING_MAX) bytes.
    pub(crate) fn hold_cut_reply(&mut self, bytes: Vec<u8>) {
        debug_assert!(
            bytes.first() == Some(&0x1b)
                && bytes.len() <= crate::probe::PENDING_MAX
                && (bytes.len() == 1 || bytes[1] == b'['),
            "a cut answer starts with ESC, and a longer one with ESC [: {bytes:?}"
        );
        self.late = Late::Csi(bytes);
    }

    /// The world in focus.
    pub(crate) fn focused_world(&self) -> Option<&World> {
        self.focus.and_then(|index| self.worlds.get(index))
    }

    /// The id of the world in focus.
    #[must_use]
    pub fn focused(&self) -> Option<&str> {
        self.focused_world().map(|world| world.id.as_str())
    }

    /// The ids of the worlds, in their order.
    #[must_use]
    pub fn world_ids(&self) -> Vec<&str> {
        self.worlds.iter().map(|world| world.id.as_str()).collect()
    }

    /// The text of the pose editor, with an LF between each two lines.
    #[must_use]
    pub fn input(&self) -> String {
        self.editor(EditorId::POSE)
            .map_or_else(String::new, EditorState::text)
    }

    /// The state of the editor `id`.
    #[must_use]
    pub fn editor(&self, id: EditorId) -> Option<&EditorState> {
        self.editors.get(&id)
    }

    /// The pose editor.
    pub(crate) fn pose(&mut self) -> &mut EditorState {
        let caps = &self.caps;
        self.editors
            .entry(EditorId::POSE)
            .or_insert_with(|| EditorState::with_caps(Arc::clone(caps)))
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
                        line_break: None,
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
        for editor in self.editors.values_mut() {
            editor.set_caps(Arc::clone(&self.caps));
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
        self.focused_world()
            .map_or(PaneId::NOTICES, |world| world.pane)
    }

    /// Adds the system line `-- <text>` at the bottom of the pane in focus. After a Reset
    /// clears the panes of the worlds, the line comes back in the pane of the world in focus.
    pub(crate) fn notice(&mut self, text: &str) {
        let line = format!("-- {text}");
        self.push(self.focused_pane(), &line);
        self.notices.push(line);
    }

    /// Adds the line `-- <text>` at the bottom of the pane in focus, for a key that did nothing.
    /// Unlike [`Self::notice`], the line does not come back after a Reset, because it is about
    /// one moment.
    pub(crate) fn hint(&mut self, text: &str) {
        self.push(self.focused_pane(), &format!("-- {text}"));
    }

    /// Adds `line` at the bottom of `pane`.
    pub(crate) fn push(&mut self, pane: PaneId, line: &str) {
        self.panes
            .entry(pane)
            .or_insert_with(|| PaneState::with_caps(Arc::clone(&self.caps)))
            .push(line);
    }

    /// Puts `text` in the pose editor, one line for each part between two LF, with the cursor at
    /// its end.
    pub(crate) fn set_input(&mut self, text: &str) {
        self.pose().set_text(text);
    }
}
