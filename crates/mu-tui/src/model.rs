//! The model of the terminal client: one value that the loop owns. [`update`](crate::update)
//! changes it for each message. [`view`](crate::view) records what each pane drew, and returns
//! a scrolled pane to the bottom when the newest row shows.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;

use mu_config::layout::{
    BUILT_IN, Block, BlockId, BlockKind, DEFAULT_LAYOUT, Layout, NOTICES_STREAM,
};
use mu_config::{Choice, Label, Segment, WorldId, layout_for};
use mu_proto::World as ProtoWorld;
use ratatui::style::Style;

use crate::config::Echo;
use crate::editor::EditorState;
use crate::history::History;
use crate::keymap::Keymap;
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
    /// worlds. None for a world that has none, or that no list held yet, and for a text that is
    /// empty or holds a control character.
    pub(crate) line_break: Option<String>,
    /// True once a list of the worlds held this world, so `line_break` is known.
    pub(crate) listed: bool,
    /// The texts that the player sent to this world, and the walk of Up and Down through them.
    pub(crate) history: History,
    /// Whether the world is connected, from the last list of the worlds and the events since.
    pub(crate) state: WorldStatus,
    /// The time of the event of the last connect, in milliseconds since the Unix epoch, while
    /// the world is connected.
    pub(crate) connected_ms: Option<i64>,
    /// The time of the last send to the world from any client.
    pub(crate) sent_ms: Option<i64>,
}

/// Whether a world is connected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum WorldStatus {
    /// No list of the worlds and no event gave the state yet.
    #[default]
    Unknown,
    /// The server opens the connection.
    Connecting,
    /// The world is connected.
    Connected,
    /// The world is not connected.
    Disconnected,
    /// The server waits for its next attempt.
    Reconnecting,
}

impl WorldStatus {
    /// The state as the statusline shows it, or none for a state that is not known.
    pub(crate) const fn name(self) -> Option<&'static str> {
        match self {
            Self::Unknown => None,
            Self::Connecting => Some("connecting"),
            Self::Connected => Some("connected"),
            Self::Disconnected => Some("disconnected"),
            Self::Reconnecting => Some("reconnecting"),
        }
    }
}

/// Whether the wait of a tick of the clock runs, so a second one does not start.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TickWait {
    /// No tick waits.
    #[default]
    Idle,
    /// A tick waits, and its [`Msg::Tick`](crate::update::Msg::Tick) did not come yet.
    Waiting,
}

/// The order of the worlds: the order of [`WorldId`], by the game part and then by the
/// character part. A text that is no world id comes first, in the order of its text.
fn order(id: &str) -> (Option<WorldId>, &str) {
    match id.parse::<WorldId>() {
        Ok(world) => (Some(world), ""),
        Err(_) => (None, id),
    }
}

/// What set the arm of a quit, which decides what cancels it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum QuitArm {
    /// A key. The next key or action that is not a quit cancels it.
    Key,
    /// The command `/quit`. A `/quit` or a quit key leaves. Any other submitted text or command
    /// cancels it, and another key does not.
    Command,
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
    /// The state of each editor, by the id of its world and the id of its block. The key with
    /// no world is the editor while no world has the focus.
    pub(crate) editors: HashMap<(Option<String>, BlockId), EditorState>,
    /// The editor with the input focus. [`Model::input_id`] gives the editor of the layout on
    /// the screen that it stands for.
    pub(crate) input: BlockId,
    /// The pane of each `output` that is not the main output of its plan, by the id of its
    /// world and the id of its block.
    pub(crate) extra_panes: HashMap<(Option<String>, BlockId), PaneId>,
    /// The block in front of each stack, by the id of the first block of its pane.
    pub(crate) fronts: HashMap<BlockId, BlockId>,
    /// The sends of each world that the server did not answer yet, in the order of the keys.
    /// The first one is with the server. The client sends the next one after that reply. So
    /// the lines of a world reach the game in order. A world with no such send has no entry.
    pub(crate) sends: HashMap<String, VecDeque<String>>,
    pub(crate) lost: bool,
    pub(crate) keymap: Keymap,
    /// The layout of the field `layout`, and its name.
    pub(crate) layout: Layout,
    pub(crate) layout_name: String,
    /// The layout and its name for each entry of `[world-layouts]`, by the key of the entry.
    pub(crate) world_layouts: BTreeMap<String, (String, Layout)>,
    /// The label of a world tab.
    pub(crate) label: Label,
    /// The segments of the statusline from the left edge, and to the right edge.
    pub(crate) segments: (Vec<Segment>, Vec<Segment>),
    pub(crate) late: Late,
    /// The notices of the client, which a reset of the panes keeps.
    pub(crate) notices: Vec<String>,
    /// The notices and the hints that no pane showed, because the layout held no `output`.
    pub(crate) unshown: Vec<String>,
    /// True if a late answer gave mode 2027 as set before the probe.
    pub(crate) keeps_grapheme_mode: bool,
    /// Set by a Quit while a send has no reply yet, the send with the server included. A second
    /// Quit then leaves.
    pub(crate) quit_arm: Option<QuitArm>,
    pub(crate) listing: Listing,
    /// True until the first list of the worlds ends. While it is set, no world gets the focus.
    pub(crate) first_list: bool,
    /// How the echo line of a send shows.
    pub(crate) echo: Echo,
    /// The reason of the last list of the worlds, if it failed.
    pub(crate) list_failure: Option<String>,
    /// The client configuration that the model applied last.
    pub(crate) config: Option<mu_proto::GetClientConfigResponse>,
    /// The latest store hash of the watch of the configuration.
    pub(crate) latest_hash: Option<String>,
    /// The count of the fetches of the configuration that failed since the last apply.
    pub(crate) config_failures: usize,
    pub(crate) config_retry: ConfigRetry,
    /// The time of the last tick of the clock, in milliseconds since the Unix epoch.
    pub(crate) now_ms: i64,
    pub(crate) tick: TickWait,
    /// The columns and the rows of the last frame.
    pub(crate) screen: (u16, u16),
    /// The name of this device, which a save of the layout needs.
    pub(crate) device: String,
    /// The names of the layouts of the last list of the layouts.
    pub(crate) layout_names: Vec<String>,
    /// The drag of a border, from its press to its release.
    pub(crate) drag: Option<Drag>,
    /// The pane that took the last press of the mouse, for its drag and its release.
    pub(crate) pressed_pane: Option<PaneId>,
    /// The state of arrange mode, while the mode is on.
    pub(crate) arrange: Option<crate::arrange::Arrange>,
}

/// The drag of a border between two panes.
#[derive(Clone, Debug)]
pub(crate) struct Drag {
    /// The layout at the press. Each report of the mouse resizes this layout, so the result
    /// does not drift.
    pub(crate) from: Layout,
    /// The border. A press on a title row with no border above it has none, and can only be a
    /// click.
    pub(crate) border: Option<mu_config::layout::Border>,
    /// The column and the row of the press.
    pub(crate) pressed: (u16, u16),
    /// Whether the mouse moved the border since the press.
    pub(crate) moved: bool,
    /// The reason that the layout rules refused the last report of the drag.
    pub(crate) refused: Option<String>,
    /// Whether the last report of the drag made mu write the whole plan again.
    pub(crate) rewritten: bool,
}

/// Whether an [`Effect::Wait`](crate::update::Effect::Wait) for
/// [`Timer::RetryConfig`](crate::update::Timer::RetryConfig) waits, so a second
/// one does not start.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ConfigRetry {
    /// No retry waits.
    #[default]
    Idle,
    /// A retry waits, and its [`Msg::RetryConfig`](crate::update::Msg::RetryConfig) did not
    /// come yet.
    Waiting,
}

/// Whether a list of the worlds is with the server, so a second one does not go.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Listing {
    /// No list is with the server.
    #[default]
    Idle,
    /// A list is with the server, and its reply did not come yet.
    Asked,
    /// A list is with the server, and the server reset after the client asked for it. Its
    /// reply can be of the server before the reset, so the client asks again after it.
    AskedBeforeReset,
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
            editors: HashMap::new(),
            input: BlockId::of(BlockKind::Editor),
            extra_panes: HashMap::new(),
            fronts: HashMap::new(),
            sends: HashMap::new(),
            caps,
            next_pane: 1,
            lost: false,
            keymap: Keymap::default(),
            layout: default_layout(),
            layout_name: DEFAULT_LAYOUT.to_owned(),
            world_layouts: BTreeMap::new(),
            label: Label::default(),
            segments: (
                Segment::LEFT_DEFAULT.to_vec(),
                Segment::RIGHT_DEFAULT.to_vec(),
            ),
            late: Late::None,
            notices: Vec::new(),
            unshown: Vec::new(),
            keeps_grapheme_mode: false,
            quit_arm: None,
            listing: Listing::Idle,
            list_failure: None,
            first_list: false,
            echo: Echo::default(),
            config: None,
            latest_hash: None,
            config_failures: 0,
            config_retry: ConfigRetry::Idle,
            now_ms: 0,
            tick: TickWait::Idle,
            screen: (0, 0),
            device: String::new(),
            layout_names: BUILT_IN.iter().map(|each| each.name.to_owned()).collect(),
            drag: None,
            pressed_pane: None,
            arrange: None,
        };
        let input = model.input.clone();
        model.editor_of(None, &input);
        for id in worlds {
            model.world(&id);
        }
        if !model.worlds.is_empty() {
            model.set_focus(0);
        }
        model
    }

    /// A model with no world that waits for its first list of the worlds. The caller asks for
    /// that list. Until the first list ends, the top row shows `listing worlds` and no world gets
    /// the focus.
    #[must_use]
    pub fn starting() -> Self {
        let mut model = Self::new([]);
        model.listing = Listing::Asked;
        model.first_list = true;
        model
    }

    /// Ends the first list of the worlds: the first world, if there is one, gets the focus.
    pub(crate) fn end_first_list(&mut self) {
        self.first_list = false;
        if !self.worlds.is_empty() {
            self.set_focus(0);
        }
    }

    /// The client configuration that the model applied last.
    #[must_use]
    pub const fn config(&self) -> Option<&mu_proto::GetClientConfigResponse> {
        self.config.as_ref()
    }

    /// The notice lines of the client, which a reset of the panes keeps.
    #[must_use]
    pub fn notices(&self) -> &[String] {
        &self.notices
    }

    /// The model with `device` as the name of this device.
    #[must_use]
    pub fn with_device(mut self, device: &str) -> Self {
        device.clone_into(&mut self.device);
        self
    }

    /// The names of the layouts of the last list of the layouts. Before the first list, they
    /// are the names of the built-in layouts.
    #[must_use]
    pub fn layout_names(&self) -> &[String] {
        &self.layout_names
    }

    /// The model with `echo` as the style of the echo line of a send.
    #[must_use]
    pub const fn with_echo(mut self, echo: Echo) -> Self {
        self.echo = echo;
        self
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

    /// This model with `keymap` in place of its keymap.
    #[must_use]
    pub fn with_keymap(mut self, keymap: Keymap) -> Self {
        self.keymap = keymap;
        self
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

    /// The layout of `world`: the layout of its entry of `[world-layouts]`, then of the entry
    /// of its game, then of the field `layout`.
    pub(crate) fn layout_of(&self, world: Option<&str>) -> &Layout {
        let world = world.and_then(|id| id.parse::<WorldId>().ok());
        match layout_for(world.as_ref(), &self.world_layouts) {
            Choice::Entry(key) => self
                .world_layouts
                .get(key.as_str())
                .map_or(&self.layout, |(_, layout)| layout),
            Choice::Default => &self.layout,
        }
    }

    /// The entry or the field that gives the layout on the screen.
    #[must_use]
    pub fn choice(&self) -> Choice {
        let world = self.focused().and_then(|id| id.parse::<WorldId>().ok());
        layout_for(world.as_ref(), &self.world_layouts)
    }

    /// Whether the layout on the screen shows a time, so that the screen changes with the
    /// clock.
    pub(crate) fn shows_a_time(&self) -> bool {
        self.layout().ids_of(BlockKind::Statusline).next().is_some()
            && self
                .segments
                .0
                .iter()
                .chain(&self.segments.1)
                .any(|segment| segment.is_time())
    }

    /// The layout on the screen: the layout of the world in focus.
    #[must_use]
    pub fn layout(&self) -> &Layout {
        self.layout_of(self.focused())
    }

    /// The name of the layout on the screen.
    #[must_use]
    pub fn layout_name(&self) -> &str {
        match self.choice() {
            Choice::Entry(key) => self
                .world_layouts
                .get(key.as_str())
                .map_or(self.layout_name.as_str(), |(name, _)| name),
            Choice::Default => &self.layout_name,
        }
    }

    /// The editor of the layout of `world` that has the input focus: the stored one if that
    /// layout holds it, and the first editor of the plan if not.
    pub(crate) fn input_id_of(&self, world: Option<&str>) -> BlockId {
        let layout = self.layout_of(world);
        if layout.holds(BlockKind::Editor, &self.input) {
            return self.input.clone();
        }
        layout
            .ids_of(BlockKind::Editor)
            .next()
            .cloned()
            .unwrap_or_else(|| self.input.clone())
    }

    /// The id of the editor with the input focus.
    #[must_use]
    pub fn input_id(&self) -> BlockId {
        self.input_id_of(self.focused())
    }

    /// The text of the editor with the input focus, with an LF between each two lines.
    #[must_use]
    pub fn input(&self) -> String {
        self.editor(&self.input_id())
            .map_or_else(String::new, EditorState::text)
    }

    /// The state of the editor `id` of the world in focus, or of no world when no world has
    /// the focus.
    #[must_use]
    pub fn editor(&self, id: &BlockId) -> Option<&EditorState> {
        self.editors
            .get(&(self.focused().map(str::to_owned), id.clone()))
    }

    /// The editor `id` of `world`, made if it does not exist yet.
    pub(crate) fn editor_of(&mut self, world: Option<&str>, id: &BlockId) -> &mut EditorState {
        let caps = &self.caps;
        self.editors
            .entry((world.map(str::to_owned), id.clone()))
            .or_insert_with(|| EditorState::with_caps(Arc::clone(caps)))
    }

    /// The editor with the input focus: of the world in focus, or of no world when no world
    /// has the focus.
    pub(crate) fn pose(&mut self) -> &mut EditorState {
        let world = self.focused().map(str::to_owned);
        let id = self.input_id();
        self.editor_of(world.as_deref(), &id)
    }

    /// The editor of `world` that has the input focus in the layout of that world, made if the
    /// world has none yet.
    pub(crate) fn pose_of(&mut self, world: &str) -> &mut EditorState {
        let id = self.input_id_of(Some(world));
        self.editor_of(Some(world), &id)
    }

    /// The text of the editor of `world` that has the input focus in the layout of that world,
    /// or `None` for a world that the model does not hold.
    #[must_use]
    pub fn input_of(&self, world: &str) -> Option<String> {
        self.find(world)?;
        let key = (Some(world.to_owned()), self.input_id_of(Some(world)));
        self.editors.get(&key).map(EditorState::text)
    }

    /// The text of the editor `id` of `world`, or `None` for an editor with no state yet.
    #[must_use]
    pub fn text_of(&self, world: Option<&str>, id: &BlockId) -> Option<String> {
        self.editors
            .get(&(world.map(str::to_owned), id.clone()))
            .map(EditorState::text)
    }

    /// The input history of `world`, the oldest entry first. `None` for a world that the model
    /// does not hold.
    #[must_use]
    pub fn history(&self, world: &str) -> Option<Vec<&str>> {
        self.find(world)
            .map(|index| self.worlds[index].history.iter().collect())
    }

    /// The place of the open walk of `world` in its history, and the length of that history.
    #[must_use]
    pub fn walk(&self, world: &str) -> Option<(usize, usize)> {
        let world = &self.worlds[self.find(world)?];
        world
            .history
            .walk_place()
            .map(|place| (place, world.history.len()))
    }

    /// Gives the focus to the world at `index` and clears its unseen mark. If no world had the
    /// focus, the text of the loose editor moves into the editor of that world, and the kept
    /// notices go into its pane. Until the first list ends, it does nothing.
    pub(crate) fn set_focus(&mut self, index: usize) {
        if self.first_list {
            return;
        }
        let had_focus = self.focus.is_some();
        self.focus = Some(index);
        let world = &mut self.worlds[index];
        world.unseen = false;
        if !had_focus {
            let id = world.id.clone();
            // Each text of no world moves into the editor with the same id of this world.
            let loose: Vec<BlockId> = self
                .editors
                .keys()
                .filter(|(world, _)| world.is_none())
                .map(|(_, editor)| editor.clone())
                .collect();
            for editor in loose {
                let state = self.editor_of(None, &editor);
                let text = state.text();
                state.set_text("");
                if !text.is_empty() {
                    self.editor_of(Some(&id), &editor).set_text(&text);
                }
            }
            self.replay_notices();
        }
    }

    /// Adds the kept notices at the bottom of the pane that shows the notices of the world in
    /// focus.
    pub(crate) fn replay_notices(&mut self) {
        let Some(pane) = self.pane_for(NOTICES_STREAM) else {
            return;
        };
        for line in self.notices.clone() {
            self.push(pane, &line);
        }
    }

    /// The block of `blocks`, the blocks of one pane, that shows: the one in front of the stack.
    pub(crate) fn front_of<'a>(&self, blocks: &'a [Block]) -> Option<&'a Block> {
        let first = blocks.first()?;
        let wanted = self.fronts.get(&first.id);
        blocks
            .iter()
            .find(|block| Some(&block.id) == wanted)
            .or(Some(first))
    }

    /// The pane of the world `world`, or the pane of notices for no world.
    fn world_pane(&self, world: Option<&str>) -> PaneId {
        world
            .and_then(|id| self.find(id))
            .map_or(PaneId::NOTICES, |index| self.worlds[index].pane)
    }

    /// The pane of the `output` `id` of `world`: the pane of the world for the main `output`
    /// of its plan, and a pane of its own for each other one. None for a block that has no
    /// pane yet. This is the one rule of which pane an `output` shows.
    pub(crate) fn pane_of(&self, world: Option<&str>, id: &BlockId) -> Option<PaneId> {
        let first = self.layout_of(world).main_output()?;
        if first == id {
            return Some(self.world_pane(world));
        }
        self.extra_panes
            .get(&(world.map(str::to_owned), id.clone()))
            .copied()
    }

    /// The pane of the `output` `id` of the world in focus.
    #[must_use]
    pub fn pane_of_block(&self, id: &BlockId) -> Option<PaneId> {
        self.pane_of(self.focused(), id)
    }

    /// The pane that takes a line of the stream `stream` for `world`, made if it does not
    /// exist yet. None if the layout of the world holds no `output`.
    pub(crate) fn pane_in(&mut self, world: Option<&str>, stream: &str) -> Option<PaneId> {
        let target = self.layout_of(world).route(stream)?.clone();
        if let Some(pane) = self.pane_of(world, &target) {
            return Some(pane);
        }
        let pane = PaneId(self.next_pane);
        self.next_pane += 1;
        self.panes
            .insert(pane, PaneState::with_caps(Arc::clone(&self.caps)));
        self.extra_panes
            .insert((world.map(str::to_owned), target), pane);
        Some(pane)
    }

    /// The pane that takes a line of the stream `stream` for the world in focus.
    pub(crate) fn pane_for(&mut self, stream: &str) -> Option<PaneId> {
        let world = self.focused().map(str::to_owned);
        self.pane_in(world.as_deref(), stream)
    }

    /// Takes the line break of each world of a list of the worlds. A listed world that the
    /// model does not hold yet comes in.
    pub(crate) fn learn(&mut self, worlds: Vec<ProtoWorld>) {
        for listed in worlds {
            let index = self.world(&listed.id);
            let world = &mut self.worlds[index];
            // The editor holds no control character, and an empty text joins nothing.
            world.line_break = listed
                .line_break
                .filter(|text| !text.is_empty() && !text.chars().any(char::is_control));
            world.listed = true;
            world.state = match listed.state.as_known() {
                Some(mu_proto::WorldState::WORLD_STATE_CONNECTING) => WorldStatus::Connecting,
                Some(mu_proto::WorldState::WORLD_STATE_CONNECTED) => WorldStatus::Connected,
                Some(mu_proto::WorldState::WORLD_STATE_DISCONNECTED) => WorldStatus::Disconnected,
                Some(mu_proto::WorldState::WORLD_STATE_RECONNECTING) => WorldStatus::Reconnecting,
                Some(mu_proto::WorldState::WORLD_STATE_UNSPECIFIED) | None => WorldStatus::Unknown,
            };
            // The time of the connect is a fact of a connected world only.
            if world.state != WorldStatus::Connected {
                world.connected_ms = None;
            }
        }
    }

    /// The index of `id`, or `None` if the model does not hold that world.
    pub(crate) fn find(&self, id: &str) -> Option<usize> {
        self.search(id)
            .ok()
            .filter(|&index| self.worlds[index].id == id)
    }

    /// The index of `id` in the sorted worlds, or the index where it goes.
    fn search(&self, id: &str) -> Result<usize, usize> {
        self.worlds
            .binary_search_by(|world| order(&world.id).cmp(&order(id)))
    }

    /// The index of `id`, which this adds in its order, with a new pane, if it is new. A new
    /// world takes the focus when no world has it, with the notices in its pane, and the focus
    /// stays on its world.
    pub(crate) fn world(&mut self, id: &str) -> usize {
        match self.search(id) {
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
                        listed: false,
                        history: History::default(),
                        state: WorldStatus::Unknown,
                        connected_ms: None,
                        sent_ms: None,
                    },
                );
                self.pose_of(id);
                match self.focus {
                    Some(focus) if focus >= index => self.focus = Some(focus + 1),
                    None => self.set_focus(index),
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

    /// Adds the system line `-- <text>` at the bottom of the pane of the notices. After a Reset
    /// clears the panes of the worlds, the line comes back in the pane of the world in focus.
    pub(crate) fn notice(&mut self, text: &str) {
        let line = format!("-- {text}");
        self.show_notice(&line);
        self.notices.push(line);
    }

    /// Adds the line `-- <text>` at the bottom of the pane of the notices, for a key that did
    /// nothing.
    /// Unlike [`Self::notice`], the line does not come back after a Reset, because it is about
    /// one moment.
    pub(crate) fn hint(&mut self, text: &str) {
        self.show_notice(&format!("-- {text}"));
    }

    /// Adds `line` at the bottom of the pane that shows the notices of the world in focus. A
    /// layout with no `output` has no such pane, so the line waits for one.
    fn show_notice(&mut self, line: &str) {
        match self.pane_for(NOTICES_STREAM) {
            Some(pane) => self.push(pane, line),
            None => self.unshown.push(line.to_owned()),
        }
    }

    /// Adds each line that waited for a pane of the notices, if the layout has one now.
    pub(crate) fn show_unshown(&mut self) {
        if self.unshown.is_empty() {
            return;
        }
        if let Some(pane) = self.pane_for(NOTICES_STREAM) {
            for line in std::mem::take(&mut self.unshown) {
                self.push(pane, &line);
            }
        }
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

/// The layout of a client whose configuration names none.
pub(crate) fn default_layout() -> Layout {
    let lines = BUILT_IN
        .iter()
        .find(|each| each.name == DEFAULT_LAYOUT)
        .map_or(&["editor"][..], |each| each.lines);
    Layout::parse(lines)
        .or_else(|_| Layout::parse(&["editor"]))
        .unwrap_or_else(|error| unreachable!("the plan of one editor is good: {error}"))
}
