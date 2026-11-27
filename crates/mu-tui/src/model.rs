//! The model of the terminal client: one value that the loop owns. [`update`](crate::update)
//! changes it for each message. [`view`](crate::view) records what each pane drew, and returns
//! a scrolled pane to the bottom when the newest row shows.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::Arc;

use mu_config::layout::{BUILT_IN, Block, BlockId, BlockKind, DEFAULT_LAYOUT, Layout};
use mu_config::stream::{Scope, StreamName};
use mu_config::{Choice, Label, Segment, WorldId, layout_for};
use mu_proto::World as ProtoWorld;
use ratatui::style::{Modifier, Style};

use crate::config::Echo;
use crate::editor::EditorState;
use crate::history::History;
use crate::keymap::Keymap;
use crate::pane::{Caps, PaneState};

/// The handle of an output pane. A [`PaneKey`] names the pane that it stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PaneId(usize);

/// The handle of the pane of `main` of no world, which the model makes at its start.
const CLIENT_MAIN: PaneId = PaneId(0);

/// Who a pane, an editor or a line belongs to.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Owner {
    /// One world, by its id.
    World(String),
    /// No world.
    Client,
    /// Every world: the pane of an output of all worlds.
    All,
}

impl Owner {
    /// The owner `World` of `world`, or `Client` for no world.
    #[must_use]
    pub fn of(world: Option<&str>) -> Self {
        world.map_or(Self::Client, |id| Self::World(id.to_owned()))
    }

    /// The owner of the pane of an output of `scope` in the layout of `world`: `All` for an
    /// output of all worlds, and the owner of `world` for each other output. This is the one
    /// rule of the owner of an output pane.
    #[must_use]
    pub fn of_output(scope: Scope, world: Option<&str>) -> Self {
        match scope {
            Scope::AllWorlds => Self::All,
            Scope::World => Self::of(world),
        }
    }

    /// The world of the owner `World`, and none for each other owner.
    #[must_use]
    pub fn world(&self) -> Option<&str> {
        match self {
            Self::World(id) => Some(id),
            Self::Client | Self::All => None,
        }
    }
}

/// The name of an output pane or an editor: its owner and the id of its block.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PaneKey {
    /// The owner.
    pub owner: Owner,
    /// The id of the block.
    pub id: BlockId,
}

impl PaneKey {
    /// The key of the block `id` of `owner`.
    #[must_use]
    pub const fn new(owner: Owner, id: BlockId) -> Self {
        Self { owner, id }
    }

    /// The key of the output `main` of `world`, or of no world.
    #[must_use]
    pub fn main(world: Option<&str>) -> Self {
        Self::new(Owner::of(world), BlockId::of(BlockKind::Output))
    }
}

/// A notice of the client, which a reset of the panes keeps.
#[derive(Clone, Debug)]
pub(crate) struct Kept {
    /// The line, with its `-- `.
    pub(crate) line: String,
    /// The world that the notice belongs to, or `Client` for none.
    pub(crate) owner: Owner,
    /// The panes that hold the line.
    pub(crate) panes: HashSet<PaneKey>,
}

/// The output panes: the state of each pane, and the handle of each pane by its key. A handle
/// comes only from [`Panes::at`], and no pane goes away, so each handle names a state.
#[derive(Debug)]
pub(crate) struct Panes {
    /// The state of each pane, at the index of its handle.
    states: Vec<PaneState>,
    keys: HashMap<PaneKey, PaneId>,
}

impl Panes {
    /// The pane of `main` of no world alone.
    fn new() -> Self {
        Self {
            states: vec![PaneState::default()],
            keys: HashMap::from([(PaneKey::main(None), CLIENT_MAIN)]),
        }
    }

    /// The handle of the pane of `key`, or none for a pane that holds no line yet.
    pub(crate) fn id(&self, key: &PaneKey) -> Option<PaneId> {
        self.keys.get(key).copied()
    }

    /// The handle of the pane of `key`, made with `caps` if it does not exist yet.
    pub(crate) fn make(&mut self, key: &PaneKey, caps: &Arc<Caps>) -> PaneId {
        if let Some(pane) = self.keys.get(key) {
            return *pane;
        }
        let pane = PaneId(self.states.len());
        self.states.push(PaneState::with_caps(Arc::clone(caps)));
        self.keys.insert(key.clone(), pane);
        pane
    }

    /// The state of the pane of `key`, made with `caps` if it does not exist yet.
    pub(crate) fn at(&mut self, key: &PaneKey, caps: &Arc<Caps>) -> &mut PaneState {
        let pane = self.make(key, caps);
        &mut self.states[pane.0]
    }

    /// The state of the pane `id`.
    pub(crate) fn get(&self, id: PaneId) -> Option<&PaneState> {
        self.states.get(id.0)
    }

    /// The state of the pane `id`, to change.
    pub(crate) fn get_mut(&mut self, id: PaneId) -> Option<&mut PaneState> {
        self.states.get_mut(id.0)
    }

    /// The state of each pane, to change.
    pub(crate) fn states_mut(&mut self) -> impl Iterator<Item = &mut PaneState> {
        self.states.iter_mut()
    }

    /// Clears each pane whose key `wanted` takes.
    pub(crate) fn clear_where(&mut self, wanted: impl Fn(&PaneKey) -> bool) {
        for (key, pane) in &self.keys {
            if wanted(key) {
                self.states[pane.0].clear();
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct World {
    pub(crate) id: String,
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
    /// Each pane that a line or a draw made.
    pub(crate) panes: Panes,
    /// What the terminal can draw, which every pane shares.
    pub(crate) caps: Arc<Caps>,
    /// The state of each editor, by its key. The owner `Client` holds the editors while no
    /// world has the focus.
    pub(crate) editors: HashMap<PaneKey, EditorState>,
    /// The editor with the input focus. [`Model::input_id`] gives the editor of the layout on
    /// the screen that it stands for.
    pub(crate) input: BlockId,
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
    /// The layouts that the last configuration gave, which a save that failed puts back.
    pub(crate) served: Option<Served>,
    /// The label of a world tab.
    pub(crate) label: Label,
    /// The segments of the statusline from the left edge, and to the right edge.
    pub(crate) segments: (Vec<Segment>, Vec<Segment>),
    pub(crate) late: Late,
    /// The notices of the client, which a reset of the panes keeps.
    pub(crate) notices: Vec<Kept>,
    /// The newest pairs of a stream and a world that got the hint of a stream with no output,
    /// at most [`NO_OUTPUT_MAX`], oldest first.
    pub(crate) no_output: VecDeque<(String, String)>,
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

/// The layouts of one configuration of the server.
#[derive(Clone, Debug)]
pub(crate) struct Served {
    /// The name of the layout of the field `layout`.
    pub(crate) name: String,
    /// The layout of the field `layout`.
    pub(crate) layout: Layout,
    /// The layout of each entry of `[world-layouts]`, by its key.
    pub(crate) worlds: BTreeMap<String, (String, Layout)>,
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
            panes: Panes::new(),
            editors: HashMap::new(),
            input: BlockId::of(BlockKind::Editor),
            fronts: HashMap::new(),
            sends: HashMap::new(),
            caps,
            lost: false,
            keymap: Keymap::default(),
            layout: default_layout(),
            layout_name: DEFAULT_LAYOUT.to_owned(),
            world_layouts: BTreeMap::new(),
            served: None,
            label: Label::default(),
            segments: (
                Segment::LEFT_DEFAULT.to_vec(),
                Segment::RIGHT_DEFAULT.to_vec(),
            ),
            late: Late::None,
            notices: Vec::new(),
            no_output: VecDeque::new(),
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
    pub fn notices(&self) -> Vec<&str> {
        self.notices.iter().map(|kept| kept.line.as_str()).collect()
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
            .get(&PaneKey::new(Owner::of(self.focused()), id.clone()))
    }

    /// The editor `id` of `world`, made if it does not exist yet.
    pub(crate) fn editor_of(&mut self, world: Option<&str>, id: &BlockId) -> &mut EditorState {
        let caps = &self.caps;
        self.editors
            .entry(PaneKey::new(Owner::of(world), id.clone()))
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
        let key = PaneKey::new(Owner::of(Some(world)), self.input_id_of(Some(world)));
        self.editors.get(&key).map(EditorState::text)
    }

    /// The text of the editor `id` of `world`, or `None` for an editor with no state yet.
    #[must_use]
    pub fn text_of(&self, world: Option<&str>, id: &BlockId) -> Option<String> {
        self.editors
            .get(&PaneKey::new(Owner::of(world), id.clone()))
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
    /// focus, the text of each loose editor moves into the editor of that world, and each kept
    /// notice of no world becomes a notice of that world and goes into its panes. Until the
    /// first list ends, it does nothing.
    pub(crate) fn set_focus(&mut self, index: usize) {
        if self.first_list {
            return;
        }
        // A drag and arrange mode belong to the layout of the world in focus. So a move of the
        // focus ends them, and puts their first layout back under that world.
        let ended = if self.focus == Some(index) {
            crate::config::Ended::Nothing
        } else {
            self.take_edits(true)
        };
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
                .filter(|key| key.owner == Owner::Client)
                .map(|key| key.id.clone())
                .collect();
            for editor in loose {
                let state = self.editor_of(None, &editor);
                let text = state.text();
                state.set_text("");
                if !text.is_empty() {
                    self.editor_of(Some(&id), &editor).set_text(&text);
                }
            }
            for kept in &mut self.notices {
                if kept.owner == Owner::Client {
                    kept.owner = Owner::World(id.clone());
                }
            }
            self.replay_notices();
        }
        // The player now reads the notices of this world, so the line goes there.
        self.say_ended("the focus moved", ended);
    }

    /// Routes each kept notice through the layout of its owner, and adds it at the bottom of
    /// each pane that shows it and does not hold it yet.
    pub(crate) fn replay_notices(&mut self) {
        let mut kept = std::mem::take(&mut self.notices);
        for notice in &mut kept {
            let (keys, _) = self.route(notice.owner.world(), &notices_stream());
            for key in keys {
                if notice.panes.insert(key.clone()) {
                    self.push_to(&key, &notice.owner, &notice.line, &[]);
                }
            }
        }
        self.notices = kept;
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

    /// The key of the `output` `id` in the layout of `world`, by [`Owner::of_output`].
    pub(crate) fn key_of(&self, world: Option<&str>, id: &BlockId) -> PaneKey {
        let scope = self
            .layout_of(world)
            .selection_of(id)
            .map_or(Scope::World, |selection| selection.scope);
        PaneKey::new(Owner::of_output(scope, world), id.clone())
    }

    /// The pane of the `output` `id` of `world`, or none for a block that holds no line yet.
    /// This is the one rule of which pane an `output` shows.
    pub(crate) fn pane_of(&self, world: Option<&str>, id: &BlockId) -> Option<PaneId> {
        self.panes.id(&self.key_of(world, id))
    }

    /// The pane of `key`, made if it does not exist yet.
    pub(crate) fn pane_at(&mut self, key: &PaneKey) -> PaneId {
        self.panes.make(key, &self.caps)
    }

    /// The keys of the panes that show a line of the streams `names` of `world`, each one time,
    /// in the order of `names` and in the order of the plan for each name, and the streams that
    /// no output of the layout of `world` shows. This is the one rule of where a line goes.
    pub(crate) fn route(
        &self,
        world: Option<&str>,
        names: &[StreamName],
    ) -> (Vec<PaneKey>, Vec<StreamName>) {
        let layout = self.layout_of(world);
        let mut keys: Vec<PaneKey> = Vec::new();
        let mut missing = Vec::new();
        for name in names {
            let mut found = false;
            for (id, selection) in layout.showing(name) {
                found = true;
                let key = PaneKey::new(Owner::of_output(selection.scope, world), id.clone());
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
            if !found {
                missing.push(name.clone());
            }
        }
        (keys, missing)
    }

    /// Adds the line `text` of `owner`, a world or `Client`, with `spans` at the bottom of the
    /// pane of `key`. The owner of `key` is `owner` or `All`. A pane of all worlds puts the dim
    /// name of the owner in front of the line.
    pub(crate) fn push_to(
        &mut self,
        key: &PaneKey,
        owner: &Owner,
        text: &str,
        spans: &[(usize, Style)],
    ) {
        debug_assert!(
            key.owner == Owner::All || key.owner == *owner,
            "a line of {owner:?} in a pane of {:?}",
            key.owner
        );
        let state = self.panes.at(key, &self.caps);
        if key.owner == Owner::All {
            let (text, spans) = prefixed(owner.world().unwrap_or("client"), text, spans);
            state.push_styled(&text, &spans);
        } else {
            state.push_styled(text, spans);
        }
    }

    /// The pane of the `output` `id` of the world in focus.
    #[must_use]
    pub fn pane_of_block(&self, id: &BlockId) -> Option<PaneId> {
        self.pane_of(self.focused(), id)
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

    /// The index of `id`, which this adds in its order, with the pane of its `main`, if it is
    /// new. A new world takes the focus when no world has it, with the notices in its panes,
    /// and the focus stays on its world.
    pub(crate) fn world(&mut self, id: &str) -> usize {
        match self.search(id) {
            Ok(index) => index,
            Err(index) => {
                self.pane_at(&PaneKey::main(Some(id)));
                self.worlds.insert(
                    index,
                    World {
                        id: id.to_owned(),
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
        for pane in self.panes.states_mut() {
            pane.set_caps(Arc::clone(&self.caps));
        }
        for editor in self.editors.values_mut() {
            editor.set_caps(Arc::clone(&self.caps));
        }
    }

    /// The state of the pane `id`.
    #[must_use]
    pub fn pane(&self, id: PaneId) -> Option<&PaneState> {
        self.panes.get(id)
    }

    /// The pane of `main` of the world in focus, or of no world when no world has the focus.
    #[must_use]
    pub fn focused_pane(&self) -> PaneId {
        self.panes
            .id(&PaneKey::main(self.focused()))
            .unwrap_or(CLIENT_MAIN)
    }

    /// Whether the stream `stream` of `world` gets the hint of a stream with no output: true the
    /// first time, and again after [`NO_OUTPUT_MAX`] newer pairs pushed the pair out. A game
    /// can make a new stream name for each line, so the memory of the pairs has a limit.
    pub(crate) fn first_no_output(&mut self, stream: &str, world: &str) -> bool {
        let pair = (stream.to_owned(), world.to_owned());
        if self.no_output.contains(&pair) {
            return false;
        }
        if self.no_output.len() == NO_OUTPUT_MAX {
            self.no_output.pop_front();
        }
        self.no_output.push_back(pair);
        true
    }

    /// Adds the system line `-- <text>` of the world in focus, or of no world, as a line of the
    /// stream `notices`. After a Reset clears the panes of the worlds, the line comes back in
    /// each pane that shows it then.
    pub(crate) fn notice(&mut self, text: &str) {
        let owner = Owner::of(self.focused());
        self.keep_notice(owner, crate::text::marked(text));
    }

    /// Adds `line` as a line of the stream `notices` of `owner`. After a Reset clears the panes
    /// of the worlds, the line comes back in each pane that shows it then.
    pub(crate) fn keep_notice(&mut self, owner: Owner, line: String) {
        let panes = self.show_notice(&owner, &line);
        self.notices.push(Kept { line, owner, panes });
    }

    /// Adds the line `-- <text>` of the world in focus, or of no world, as a line of the
    /// stream `notices`. Unlike [`Self::notice`], the line does not come back after a Reset,
    /// because it is about one moment.
    pub(crate) fn hint(&mut self, text: &str) {
        let owner = Owner::of(self.focused());
        self.show_notice(&owner, &crate::text::marked(text));
    }

    /// Adds `line` as a line of the stream `notices` of `owner`, and gives the keys of the
    /// panes that took it.
    pub(crate) fn show_notice(&mut self, owner: &Owner, line: &str) -> HashSet<PaneKey> {
        let (keys, _) = self.route(owner.world(), &notices_stream());
        for key in &keys {
            self.push_to(key, owner, line, &[]);
        }
        keys.into_iter().collect()
    }

    /// Puts `text` in the pose editor, one line for each part between two LF, with the cursor at
    /// its end.
    pub(crate) fn set_input(&mut self, text: &str) {
        self.pose().set_text(text);
    }
}

/// The most pairs of a stream and a world that the client remembers for the hint of a stream
/// with no output.
pub const NO_OUTPUT_MAX: usize = 256;

/// The stream `notices` alone.
fn notices_stream() -> Vec<StreamName> {
    vec![StreamName::notices()]
}

/// The line `text` with the dim prefix `<owner>: ` in front, and `spans` moved past it.
fn prefixed(owner: &str, text: &str, spans: &[(usize, Style)]) -> (String, Vec<(usize, Style)>) {
    let prefix = format!("{owner}: ");
    let start = prefix.len();
    let mut moved = vec![(0, Style::default().add_modifier(Modifier::DIM))];
    if spans.first().is_none_or(|(offset, _)| *offset != 0) {
        moved.push((start, Style::default()));
    }
    moved.extend(spans.iter().map(|(offset, style)| (offset + start, *style)));
    (prefix + text, moved)
}

/// The plan of the default layout if the built-in default does not build.
pub const FALLBACK_LAYOUT: &[&str] = &["output", "editor below main"];

/// The layout of a client whose configuration names none.
pub(crate) fn default_layout() -> Layout {
    let lines = BUILT_IN
        .iter()
        .find(|each| each.name == DEFAULT_LAYOUT)
        .map_or(FALLBACK_LAYOUT, |each| each.lines);
    Layout::parse(lines)
        .or_else(|_| Layout::parse(FALLBACK_LAYOUT))
        .unwrap_or_else(|error| unreachable!("the fallback plan is good: {error}"))
}
