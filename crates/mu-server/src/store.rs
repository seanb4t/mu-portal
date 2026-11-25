//! The configuration store: the one owner of the files of the configuration directory. It
//! resolves the worlds at the start, checks each client layer, resolves the configuration of
//! one client, and reads the directory again at a reload.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mu_config::layout::{BUILT_IN, BuiltIn, Change, DEFAULT_LAYOUT, Layout, LayoutFile};
use mu_config::{
    Choice, ClientLayer, DeviceName, Echo, FIELD_PATHS, KEYS_TABLE, Kind, LABEL_DEFAULT,
    LayoutName, NameError, Os, STORE, SaveTarget, Segment, WORLD_LAYOUTS_TABLE, WorldId, WorldKey,
    check_fields, check_keys, filter, keyed_path, layout_for, save_target,
};
use mu_core::ResolvedWorld;
use mu_core::world::{
    CLIENTS_DIR, ConfigError, DirSnapshot, Hidden, SnapEntry, is_under_clients, merge_layers,
    parse_toml, resolve, world_errors,
};
use sha2::{Digest as _, Sha256};
use tokio::sync::{Mutex, watch};

use crate::write;

/// The check of `server.toml` that the start of `mu-cli` also runs. It takes the entry of the
/// file in the snapshot, or none when the directory holds no such file, and gives the text of
/// the error.
pub type ServerCheck = Box<dyn Fn(Option<&SnapEntry>) -> Result<(), String> + Send + Sync>;

/// Writes one notice of the store where the server reports its warnings.
pub type NoticeSink = Box<dyn Fn(&Notice) + Send + Sync>;

/// One bad file or place: its path, relative to the configuration directory when it is under
/// it, and the whole text of the notice. Neither holds a control character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    /// The path of the file.
    pub path: String,
    /// The text of the notice.
    pub text: String,
}

impl Notice {
    /// A notice of `path` with `reason`, each control character written as U+FFFD.
    fn new(path: &str, reason: &str) -> Self {
        Self::whole(path, &format!("{path}: {reason}"))
    }

    /// A notice of `path` with the whole text `text`.
    fn whole(path: &str, text: &str) -> Self {
        Self {
            path: filter(path, &STORE),
            text: filter(text, &STORE),
        }
    }
}

impl fmt::Display for Notice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// What the store knows of one path under `clients/`.
#[derive(Clone, Debug, PartialEq, Eq)]
enum State {
    /// A good layer, its keys under their canonical names.
    Good(ClientLayer),
    /// A good layout file.
    Layout(Layout),
    /// A bad entry, with its notice.
    Bad(Notice),
}

/// The text of an entry that is not a regular file.
const NOT_REGULAR: &str = "not a regular file, so the server does not read it";

/// The client part of the store: each entry under `clients/` and what it holds, with the store
/// hash of them all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientPart {
    store_hash: String,
    entries: BTreeMap<String, SnapEntry>,
    states: BTreeMap<String, State>,
}

/// The configuration of one client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientConfig {
    /// The store hash of the client part that it comes from.
    pub store_hash: String,
    /// The revision of the layers of the client.
    pub revision: String,
    /// How the client shows the echo line of a send.
    pub echo: Echo,
    /// Each key, by its canonical name, and the entry that it runs, or `none`.
    pub bindings: Vec<(String, String)>,
    /// The name of the layout of the screen.
    pub layout_name: String,
    /// The lines of that layout.
    pub layout: Vec<String>,
    /// The layout of a world or of a game, for each entry of `[world-layouts]` with a good
    /// layout.
    pub world_layouts: Vec<NamedLayout>,
    /// The label of a world tab.
    pub tab_label: String,
    /// The names of the segments of the statusline from the left edge.
    pub statusline_left: Vec<String>,
    /// The names of the segments of the statusline to the right edge.
    pub statusline_right: Vec<String>,
    /// The path of each field, such as `echo` or `keys.alt+n`, and the layer that set it, or
    /// `default`.
    pub sources: BTreeMap<String, String>,
    /// One notice for each bad layer that applies to the client, and for each bad entry at the
    /// path of a layer directory.
    pub notices: Vec<Notice>,
}

/// The layouts that the layers of one client choose.
struct Chosen {
    /// The name of the layout of the screen.
    name: String,
    /// Its lines.
    lines: Vec<String>,
    /// The layout of each world entry with a good layout.
    worlds: Vec<NamedLayout>,
    /// Each layout name that the layers give, good or not.
    files: Vec<String>,
}

/// One layout with what names it: the key of a `[world-layouts]` entry, or nothing for a row of
/// the list of every layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedLayout {
    /// The key of the entry: a world id or the name of a game. None in a list of layouts.
    pub key: Option<String>,
    /// The name of the layout.
    pub name: String,
    /// The lines of the layout.
    pub lines: Vec<String>,
    /// Whether mu ships the layout.
    pub built_in: bool,
}

/// The content of one entry of the input of a hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Content<'a> {
    /// The bytes of a file that the server read.
    Read(&'a [u8]),
    /// The kind of the error of an entry that the server cannot read.
    Unreadable(&'a str),
}

impl<'a> Content<'a> {
    fn of(entry: &'a SnapEntry) -> Self {
        match entry {
            SnapEntry::File(bytes) => Self::Read(bytes),
            SnapEntry::NotRegular => Self::Unreadable(NOT_REGULAR),
            SnapEntry::Error { kind, .. } => Self::Unreadable(kind),
        }
    }
}

/// The framed input of a hash of `entries`: for each entry, the length of the path as 8 bytes
/// big-endian and the path, then the tag 0 and the length and the bytes of a file, or the tag 1
/// and the length and the text of the kind of its error. No two lists give the same input.
#[must_use]
pub fn framed<'a>(entries: impl IntoIterator<Item = (&'a str, Content<'a>)>) -> Vec<u8> {
    fn frame(input: &mut Vec<u8>, bytes: &[u8]) {
        input.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        input.extend_from_slice(bytes);
    }
    let mut input = Vec::new();
    for (path, content) in entries {
        frame(&mut input, path.as_bytes());
        match content {
            Content::Read(bytes) => {
                input.push(0);
                frame(&mut input, bytes);
            }
            Content::Unreadable(kind) => {
                input.push(1);
                frame(&mut input, kind.as_bytes());
            }
        }
    }
    input
}

/// The first 16 hex digits of the SHA-256 hash of `input`.
fn digest(input: &[u8]) -> String {
    format!("{:.16x}", Sha256::digest(input))
}

/// The layer that applies to every client.
const GLOBAL: &str = "clients/global.toml";

/// A directory of layer files under `clients/`: its path, the name of a layer in it as the
/// notice writes it, and the check of that name.
struct Place {
    dir: &'static str,
    shown: &'static str,
    check: fn(&str) -> Result<(), NameError>,
}

/// The directories of the kind, the OS and the device layers, in the order of the merge.
const PLACES: [Place; 3] = [
    Place {
        dir: "clients/kinds",
        shown: "<kind>",
        check: |name| Kind::parse(name).map(drop),
    },
    Place {
        dir: "clients/os",
        shown: "<os>",
        check: |name| Os::parse(name).map(drop),
    },
    Place {
        dir: DEVICES_DIR,
        shown: "<device>",
        check: |name| DeviceName::new(name).map(drop),
    },
];

/// The directory of the layout files.
pub const LAYOUTS_DIR: &str = "clients/layouts";
/// The directory of the device layers.
pub const DEVICES_DIR: &str = "clients/devices";

/// The path of the layout file of `name`.
#[must_use]
pub fn layout_path(name: &str) -> String {
    path_in(LAYOUTS_DIR, name)
}

/// The path of the TOML file `name` right under the directory `dir`. [`name_in`] reads it back.
fn path_in(dir: &str, name: &str) -> String {
    format!("{dir}/{name}.toml")
}

/// The name of the layout file at `path`, or none for a path that is not in the directory of
/// the layout files.
fn layout_file(path: &str) -> Option<&str> {
    name_in(path, LAYOUTS_DIR)
}

/// The choice that `key` names: the entry of a world id or of a game, or the field `layout`
/// for no key.
fn choice_of(key: Option<&str>) -> Result<Choice, WriteError> {
    match key {
        Some(key) => key
            .parse::<WorldKey>()
            .map(Choice::Entry)
            .map_err(|error| refused(&error)),
        None => Ok(Choice::Default),
    }
}

/// The name of the file at `path`, if it is a TOML file right under the directory `dir`.
fn name_in<'a>(path: &'a str, dir: &str) -> Option<&'a str> {
    path.strip_prefix(dir)?
        .strip_prefix('/')?
        .strip_suffix(".toml")
        .filter(|name| !name.contains('/'))
}

/// What the layout file `name` at `path` holds.
fn layout_state(path: &str, name: &str, entry: &SnapEntry) -> State {
    let bad = |reason: &str| State::Bad(Notice::new(path, reason));
    if let Err(error) = LayoutName::new(name) {
        return bad(&error.to_string());
    }
    let text = match entry.text() {
        Ok(text) => text,
        Err(reason) => return bad(&reason),
    };
    let file: LayoutFile = match parse_toml(&path, text, Hidden::All) {
        Ok(file) => file,
        Err(error) => return State::Bad(Notice::whole(path, &error.to_string())),
    };
    match Layout::parse(&file.place) {
        Ok(layout) => State::Layout(layout),
        Err(error) => bad(&error.to_string()),
    }
}

/// The directories on the path of each layer of a client.
fn layer_dirs<'a>() -> impl Iterator<Item = &'a str> {
    std::iter::once(CLIENTS_DIR).chain(PLACES.iter().map(|place| place.dir))
}

/// The text of a path under `clients/` that is not a layer file.
fn not_a_layer() -> String {
    let places: Vec<String> = PLACES
        .iter()
        .map(|place| path_in(place.dir, place.shown))
        .collect();
    format!(
        "not a layer file: the layer files are {GLOBAL}, {}. A layout file is {LAYOUTS_DIR}/<name>.toml",
        mu_config::listed(&places)
    )
}

/// Whether `path` is a layer file, or the reason that it is not.
fn layer_rule(path: &str) -> Result<(), String> {
    if path == GLOBAL {
        return Ok(());
    }
    for place in &PLACES {
        if let Some(name) = name_in(path, place.dir) {
            return (place.check)(name).map_err(|error| error.to_string());
        }
    }
    Err(not_a_layer())
}

/// What the entry at `path` under `clients/` holds.
fn state(path: &str, entry: &SnapEntry) -> State {
    let bad = |reason: &str| State::Bad(Notice::new(path, reason));
    if let Some(name) = layout_file(path) {
        return layout_state(path, name, entry);
    }
    if let SnapEntry::File(_) = entry
        && let Err(reason) = layer_rule(path)
    {
        return bad(&reason);
    }
    let text = match entry.text() {
        Ok(text) => text,
        Err(reason) => return bad(&reason),
    };
    let mut layer: ClientLayer = match parse_toml(&path, text, Hidden::All) {
        Ok(layer) => layer,
        Err(error) => return State::Bad(Notice::whole(path, &error.to_string())),
    };
    if let Some(keys) = &layer.keys {
        match check_keys(keys) {
            Ok(bindings) => {
                layer.keys = Some(
                    bindings
                        .iter()
                        .zip(keys.values())
                        .map(|((name, _), value)| (name.to_string(), value.clone()))
                        .collect(),
                );
            }
            Err(error) => {
                return bad(&format!(
                    "{}: {}",
                    keyed_path(KEYS_TABLE, &error.key),
                    error.reason
                ));
            }
        }
    }
    if let Err((path, error)) = check_fields(&layer) {
        return bad(&format!("{path}: {error}"));
    }
    State::Good(layer)
}

impl ClientPart {
    /// The client part of `snapshot`.
    #[must_use]
    pub fn new(snapshot: &DirSnapshot) -> Self {
        let entries: BTreeMap<String, SnapEntry> = snapshot
            .clients()
            .map(|(path, entry)| (path.to_owned(), entry.clone()))
            .collect();
        let states = entries
            .iter()
            .map(|(path, entry)| (path.clone(), state(path, entry)))
            .collect();
        let store_hash = digest(&framed(
            entries
                .iter()
                .map(|(path, entry)| (path.as_str(), Content::of(entry))),
        ));
        Self {
            store_hash,
            entries,
            states,
        }
    }

    /// The hash of every entry under `clients/`.
    #[must_use]
    pub fn store_hash(&self) -> &str {
        &self.store_hash
    }

    /// One notice for each bad entry under `clients/`.
    #[must_use]
    pub fn notices(&self) -> Vec<Notice> {
        self.states
            .values()
            .filter_map(|state| match state {
                State::Bad(notice) => Some(notice.clone()),
                State::Good(_) | State::Layout(_) => None,
            })
            .collect()
    }

    /// The lines of the good layout `name`, and whether mu ships it.
    fn layout_named(&self, name: &str) -> Option<(Vec<String>, bool)> {
        if let Some(built_in) = BuiltIn::named(name) {
            let lines = built_in.lines.iter().map(ToString::to_string).collect();
            return Some((lines, true));
        }
        match self.states.get(&layout_path(name)) {
            Some(State::Layout(layout)) => Some((layout.lines(), false)),
            _ => None,
        }
    }

    /// Each built-in layout and each good layout file, in the order of the names, and the
    /// notice of each bad layout file.
    #[must_use]
    pub fn layouts(&self) -> (Vec<NamedLayout>, Vec<Notice>) {
        let mut names: BTreeSet<&str> = BUILT_IN.iter().map(|each| each.name).collect();
        let mut notices = Vec::new();
        for (path, state) in &self.states {
            let Some(name) = layout_file(path) else {
                continue;
            };
            match state {
                State::Layout(_) => {
                    names.insert(name);
                }
                State::Bad(notice) => notices.push(notice.clone()),
                State::Good(_) => {}
            }
        }
        let layouts = names
            .into_iter()
            .filter_map(|name| {
                let (lines, built_in) = self.layout_named(name)?;
                Some(NamedLayout {
                    key: None,
                    name: name.to_owned(),
                    lines,
                    built_in,
                })
            })
            .collect();
        (layouts, notices)
    }

    /// The notice of the layout file `name`, if the file is bad.
    #[must_use]
    pub fn layout_notice(&self, name: &str) -> Option<Notice> {
        match self.states.get(&layout_path(name)) {
            Some(State::Bad(notice)) => Some(notice.clone()),
            _ => None,
        }
    }

    /// Whether the entry at `path` is a bad layer, with its notice.
    #[must_use]
    pub fn bad_at(&self, path: &str) -> Option<Notice> {
        match self.states.get(path) {
            Some(State::Bad(notice)) => Some(notice.clone()),
            _ => None,
        }
    }

    /// The layout of the screen and the layout of each world entry. A name of no good layout
    /// gives a notice, and the next choice applies.
    fn chosen(
        &self,
        layout: Option<String>,
        entries: BTreeMap<String, String>,
        sources: &mut BTreeMap<String, String>,
        notices: &mut Vec<Notice>,
    ) -> Chosen {
        let wanted = layout.unwrap_or_else(|| DEFAULT_LAYOUT.to_owned());
        let mut files = vec![wanted.clone()];
        let (name, lines) = if let Some((lines, _)) = self.layout_named(&wanted) {
            (wanted, lines)
        } else {
            let source = sources.insert("layout".to_owned(), "default".to_owned());
            notices.push(Notice::new(
                source.as_deref().unwrap_or(CLIENTS_DIR),
                &format!(
                    "layout: no good layout is named {wanted}, so the layout is {DEFAULT_LAYOUT}"
                ),
            ));
            let lines = self
                .layout_named(DEFAULT_LAYOUT)
                .map_or_else(Vec::new, |(lines, _)| lines);
            (DEFAULT_LAYOUT.to_owned(), lines)
        };
        let mut worlds = Vec::new();
        for (key, entry) in entries {
            files.push(entry.clone());
            let path = keyed_path(WORLD_LAYOUTS_TABLE, &key);
            if let Some((lines, built_in)) = self.layout_named(&entry) {
                worlds.push(NamedLayout {
                    key: Some(key),
                    name: entry,
                    lines,
                    built_in,
                });
            } else {
                let source = sources.remove(&path);
                notices.push(Notice::new(
                    source.as_deref().unwrap_or(CLIENTS_DIR),
                    &format!(
                        "{path}: no good layout is named {entry}, so the entry does not apply"
                    ),
                ));
            }
        }
        Chosen {
            name,
            lines,
            worlds,
            files,
        }
    }

    /// The configuration of a client of `kind` on `os` with the name `device`.
    #[must_use]
    pub fn resolve(&self, kind: Kind, os: Os, device: &DeviceName) -> ClientConfig {
        let names = [kind.name(), os.name(), device.as_str()];
        let files: Vec<String> = std::iter::once(GLOBAL.to_owned())
            .chain(
                PLACES
                    .iter()
                    .zip(names)
                    .map(|(place, name)| path_in(place.dir, name)),
            )
            .collect();
        let mut notices = Vec::new();
        let mut good: Vec<(&str, &ClientLayer)> = Vec::new();
        for path in layer_dirs().chain(files.iter().map(String::as_str)) {
            match self.states.get(path) {
                Some(State::Bad(notice)) => notices.push(notice.clone()),
                Some(State::Good(layer)) => good.push((path, layer)),
                Some(State::Layout(_)) | None => {}
            }
        }
        let defaults = ClientLayer::defaults();
        let mut layers = vec![&defaults];
        layers.extend(good.iter().map(|(_, layer)| *layer));
        let mut paths: Vec<String> = FIELD_PATHS.iter().map(|path| (*path).to_owned()).collect();
        let keys: BTreeSet<&String> = good
            .iter()
            .filter_map(|(_, layer)| layer.keys.as_ref())
            .flat_map(BTreeMap::keys)
            .collect();
        paths.extend(keys.iter().map(|key| keyed_path(KEYS_TABLE, key)));
        let world_keys: BTreeSet<&String> = good
            .iter()
            .filter_map(|(_, layer)| layer.world_layouts.as_ref())
            .flat_map(BTreeMap::keys)
            .collect();
        paths.extend(
            world_keys
                .iter()
                .map(|key| keyed_path(WORLD_LAYOUTS_TABLE, key)),
        );
        let (mut merged, places) = merge_layers(&layers, &paths).unwrap_or_else(|error| {
            notices.push(Notice::new(
                CLIENTS_DIR,
                &format!("the merge of the layers failed, so the defaults apply: {error}"),
            ));
            (defaults.clone(), BTreeMap::new())
        });
        let mut sources: BTreeMap<String, String> = places
            .into_iter()
            .map(|(path, place)| {
                let source = place
                    .checked_sub(1)
                    .and_then(|place| good.get(place))
                    .map_or("default", |(path, _)| path);
                (path, source.to_owned())
            })
            .collect();
        let chosen = self.chosen(
            merged.layout.take(),
            merged.world_layouts.take().unwrap_or_default(),
            &mut sources,
            &mut notices,
        );
        let layout_files: Vec<String> = chosen.files.iter().map(|name| layout_path(name)).collect();
        let revision = digest(&framed(
            layer_dirs()
                .chain(files.iter().map(String::as_str))
                .chain(layout_files.iter().map(String::as_str))
                .collect::<BTreeSet<&str>>()
                .into_iter()
                .filter_map(|path| Some((path, Content::of(self.entries.get(path)?)))),
        ));
        let tabs = merged.tabs.unwrap_or_default();
        let statusline = merged.statusline.unwrap_or_default();
        let segments = |list: Option<Vec<String>>, default: &[Segment]| {
            list.unwrap_or_else(|| default.iter().map(|each| each.name().to_owned()).collect())
        };
        ClientConfig {
            store_hash: self.store_hash.clone(),
            revision,
            echo: merged.echo.unwrap_or_default(),
            bindings: merged.keys.unwrap_or_default().into_iter().collect(),
            layout_name: chosen.name,
            layout: chosen.lines,
            world_layouts: chosen.worlds,
            tab_label: tabs.label.unwrap_or_else(|| LABEL_DEFAULT.to_owned()),
            statusline_left: segments(statusline.left, &Segment::LEFT_DEFAULT),
            statusline_right: segments(statusline.right, &Segment::RIGHT_DEFAULT),
            sources,
            notices,
        }
    }
}

/// What [`Store::open`] gives.
pub struct Opened {
    /// The store.
    pub store: Store,
    /// The resolved worlds.
    pub worlds: Vec<ResolvedWorld>,
    /// One notice for each bad entry under `clients/`.
    pub notices: Vec<Notice>,
}

/// What a reload gives.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reloaded {
    /// One notice for each bad file.
    pub notices: Vec<Notice>,
    /// The path of each world file and of `server.toml` that differs from the snapshot of the
    /// start. The change applies at the next start. No path holds a control character.
    pub restart: Vec<String>,
    /// Whether the reload read the directory. If not, the store serves what it served before.
    pub read: bool,
}

impl Reloaded {
    /// A reload that read no snapshot.
    fn failed(notice: Notice) -> Self {
        Self {
            notices: vec![notice],
            restart: Vec::new(),
            read: false,
        }
    }
}

/// The store of one server.
pub struct Store {
    root: Option<PathBuf>,
    start: DirSnapshot,
    client: watch::Sender<Arc<ClientPart>>,
    reload: Mutex<()>,
    server_check: ServerCheck,
    report: NoticeSink,
}

impl Store {
    /// The store of the directory `root`, from `snapshot`, its snapshot at the start. It gives
    /// each notice of the start and of each reload to `report`.
    ///
    /// # Errors
    ///
    /// The first [`ConfigError`] of the worlds: a bad world stops the start.
    pub fn open(
        root: Option<&Path>,
        snapshot: DirSnapshot,
        server_check: ServerCheck,
        report: NoticeSink,
    ) -> Result<Opened, ConfigError> {
        let worlds = resolve(&[&snapshot])?;
        let part = ClientPart::new(&snapshot);
        let notices = part.notices();
        for notice in &notices {
            report(notice);
        }
        let store = Self {
            root: root.map(Path::to_path_buf),
            start: snapshot,
            client: watch::Sender::new(Arc::new(part)),
            reload: Mutex::new(()),
            server_check,
            report,
        };
        Ok(Opened {
            store,
            worlds,
            notices,
        })
    }

    /// The current client part.
    #[must_use]
    pub fn client(&self) -> Arc<ClientPart> {
        Arc::clone(&self.client.borrow())
    }

    /// A receiver that sees each new client part.
    #[must_use]
    pub fn watch(&self) -> watch::Receiver<Arc<ClientPart>> {
        self.client.subscribe()
    }

    /// Reads the directory again. A new client part replaces the current one if its store hash
    /// differs. A change to a world file or to `server.toml` is listed, and applies at the next
    /// start.
    pub async fn reload(&self) -> Reloaded {
        let _one = self.reload.lock().await;
        self.reload_locked().await
    }

    /// The reload, for a caller that holds the lock of the reloads.
    async fn reload_locked(&self) -> Reloaded {
        let root = self.root.clone();
        let taken = tokio::task::spawn_blocking(move || match root {
            Some(root) if matches!(root.try_exists(), Ok(false)) => {
                let shown = root.display().to_string();
                Err(Notice::new(
                    &shown,
                    "the configuration directory does not exist",
                ))
            }
            Some(root) => DirSnapshot::take(&root)
                .map_err(|error| Notice::whole(error.place(), &error.to_string())),
            None => Ok(DirSnapshot::default()),
        })
        .await;
        let reloaded = match taken {
            Ok(Ok(snapshot)) => self.reload_from(&snapshot),
            Ok(Err(notice)) => Reloaded::failed(notice),
            Err(error) => {
                tracing::error!("the reload failed: {error}");
                Reloaded::failed(Notice::new("reload", &error.to_string()))
            }
        };
        for notice in &reloaded.notices {
            (self.report)(notice);
        }
        reloaded
    }

    /// The reload of `snapshot`.
    fn reload_from(&self, snapshot: &DirSnapshot) -> Reloaded {
        let mut notices: Vec<Notice> = world_errors(snapshot)
            .iter()
            .map(|error| Notice::whole(error.path(), &error.to_string()))
            .collect();
        if let Err(error) = (self.server_check)(snapshot.server_file()) {
            notices.push(Notice::whole(mu_core::world::SERVER_FILE, &error));
        }
        let part = ClientPart::new(snapshot);
        notices.extend(part.notices());
        self.client.send_if_modified(|current| {
            if current.store_hash == part.store_hash {
                false
            } else {
                *current = Arc::new(part);
                true
            }
        });
        Reloaded {
            notices,
            restart: changed(&self.start, snapshot),
            read: true,
        }
    }
}

/// Why a write of the store did not happen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteError {
    /// The request is one that the rules refuse. The store checks a request before its first
    /// write, so nothing was written.
    Refused(String),
    /// The server could not write.
    Failed(String),
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(reason) | Self::Failed(reason) => f.write_str(reason),
        }
    }
}

impl std::error::Error for WriteError {}

fn refused(error: &dyn fmt::Display) -> WriteError {
    WriteError::Refused(error.to_string())
}

/// The result of an edit of a layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edited {
    /// The name of the layout that the edit wrote.
    pub name: String,
    /// The lines of that layout.
    pub lines: Vec<String>,
    /// Whether mu wrote the whole plan again.
    pub rewritten: bool,
}

/// Makes the directories of the client configuration under `root`.
fn ensure_dirs(root: &Path) -> Result<(), WriteError> {
    for dir in [CLIENTS_DIR, LAYOUTS_DIR, DEVICES_DIR] {
        write::ensure_dir(&root.join(dir), dir).map_err(|error| match error {
            write::DirError::NotADirectory(reason) => WriteError::Refused(reason),
            write::DirError::Failed(reason) => WriteError::Failed(reason),
        })?;
    }
    Ok(())
}

/// Writes the layout file `name` with `lines` under `root`.
fn put_layout(root: &Path, name: &LayoutName, lines: &[String]) -> Result<(), WriteError> {
    ensure_dirs(root)?;
    let path = layout_path(name.as_str());
    write::write_whole(&root.join(&path), &path, &write::layout_text(lines))
        .map_err(WriteError::Failed)
}

/// A checked change of a device layer that is not written yet.
struct ChoiceEdit {
    /// The file of the layer.
    target: PathBuf,
    /// The path of the file under the configuration directory.
    path: String,
    /// The text of the layer after the change.
    text: String,
}

impl ChoiceEdit {
    /// Writes the layer.
    fn write(&self) -> Result<(), WriteError> {
        write::write_whole(&self.target, &self.path, &self.text).map_err(WriteError::Failed)
    }
}

/// The change that sets the choice `choice` of the device layer of `device` under `root` to
/// the layout `name`, or removes it for no name. It writes no file of the layer. The layer
/// must be good before the change and after it, and a removal must find its entry. A caller
/// makes this check before its first write, so a refused request writes nothing.
fn choice_edit(
    root: &Path,
    part: &ClientPart,
    device: &DeviceName,
    choice: &Choice,
    name: Option<&str>,
) -> Result<ChoiceEdit, WriteError> {
    let path = path_in(DEVICES_DIR, device.as_str());
    if let Some(notice) = part.bad_at(&path) {
        return Err(WriteError::Refused(notice.text));
    }
    ensure_dirs(root)?;
    let target = root.join(&path);
    let text = match std::fs::read_to_string(&target) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(WriteError::Failed(format!("{path}: {error}"))),
    };
    let edit = name.map_or(write::FieldEdit::Remove, |name| {
        write::FieldEdit::Set(name.to_owned())
    });
    let edited = match choice {
        Choice::Default => write::edit_field(&text, &["layout"], &edit),
        Choice::Entry(key) => write::edit_field(&text, &[WORLD_LAYOUTS_TABLE, key.as_str()], &edit),
    }
    .map_err(|error| WriteError::Refused(format!("{path}: {error}")))?;
    if let (None, Choice::Entry(key)) = (name, choice)
        && edited == text
    {
        return Err(WriteError::Refused(format!(
            "{path} holds no entry {:?} of [{WORLD_LAYOUTS_TABLE}]",
            key.as_str()
        )));
    }
    if let State::Bad(notice) = state(&path, &SnapEntry::File(edited.clone().into_bytes())) {
        return Err(WriteError::Refused(notice.text));
    }
    Ok(ChoiceEdit {
        target,
        path,
        text: edited,
    })
}

/// The error of a request that names the layout `name`, which is no good layout. It holds the
/// notice of a layout file with that name that is bad.
fn no_layout(part: &ClientPart, name: &str) -> WriteError {
    match part.layout_notice(name) {
        Some(notice) => WriteError::Refused(notice.text),
        None => WriteError::Refused(format!("no good layout is named {name:?}")),
    }
}

impl Store {
    /// Runs `work` with the root of the configuration directory and the current client part,
    /// then reads the directory again, also after a `work` that failed, because it can fail
    /// after a write. One write or reload runs at a time.
    ///
    /// A `work` that is done with a reload that failed is an error: the files hold the change,
    /// and the server does not serve it yet.
    async fn write<T: Send + 'static>(
        &self,
        work: impl FnOnce(&Path, &ClientPart) -> Result<T, WriteError> + Send + 'static,
    ) -> Result<T, WriteError> {
        let _one = self.reload.lock().await;
        let root = self.root.clone().ok_or_else(|| {
            WriteError::Failed("the server has no configuration directory".to_owned())
        })?;
        let part = self.client();
        let done = tokio::task::spawn_blocking(move || work(&root, &part))
            .await
            .map_err(|error| WriteError::Failed(error.to_string()));
        let reloaded = self.reload_locked().await;
        let done = done??;
        if reloaded.read {
            Ok(done)
        } else {
            let reasons: Vec<&str> = reloaded
                .notices
                .iter()
                .map(|notice| notice.text.as_str())
                .collect();
            Err(WriteError::Failed(format!(
                "the files hold the change, but the server could not read the configuration directory again: {}",
                reasons.join("; ")
            )))
        }
    }

    /// Writes the layout file `name` with `lines`. With `device`, the layer of that device then
    /// names it: in the field `layout`, or with `key` in the entry of that key of
    /// `[world-layouts]`.
    ///
    /// # Errors
    ///
    /// A [`WriteError`] for a bad name, a bad plan, a bad device, a bad key, a key with no
    /// device, a bad device layer, and for a failed write.
    pub async fn save_layout(
        &self,
        name: &str,
        lines: Vec<String>,
        device: Option<&str>,
        key: Option<&str>,
    ) -> Result<(), WriteError> {
        let name = LayoutName::new(name).map_err(|error| refused(&error))?;
        // The file holds the lines as mu writes them, not the text of the request.
        let lines = Layout::parse(&lines)
            .map_err(|error| refused(&format!("{name}: {error}")))?
            .lines();
        let device = device
            .map(DeviceName::new)
            .transpose()
            .map_err(|error| refused(&error))?;
        let choice = choice_of(key)?;
        if device.is_none() && choice != Choice::Default {
            return Err(WriteError::Refused(
                "a key needs a device: the entry of a key is in the layer of a device".to_owned(),
            ));
        }
        self.write(move |root, part| {
            // Each check runs before the first write.
            let chosen = device
                .map(|device| choice_edit(root, part, &device, &choice, Some(name.as_str())))
                .transpose()?;
            put_layout(root, &name, &lines)?;
            chosen.map_or(Ok(()), |edit| edit.write())
        })
        .await
    }

    /// Sets the layout of `device` to `name`. With `key`, a world id or the name of a game, it
    /// sets that entry of `[world-layouts]`, and an empty `name` removes the entry.
    ///
    /// # Errors
    ///
    /// A [`WriteError`] for an unknown layout, a bad device, a bad key, a bad device layer, and
    /// a failed write.
    pub async fn use_layout(
        &self,
        device: &str,
        name: &str,
        key: Option<&str>,
    ) -> Result<(), WriteError> {
        let device = DeviceName::new(device).map_err(|error| refused(&error))?;
        let choice = choice_of(key)?;
        let name = (!name.is_empty()).then(|| name.to_owned());
        if name.is_none() && choice == Choice::Default {
            return Err(WriteError::Refused(
                "a layout name is needed: only an entry of a world or of a game can be removed"
                    .to_owned(),
            ));
        }
        self.write(move |root, part| {
            if let Some(name) = &name
                && part.layout_named(name).is_none()
            {
                return Err(no_layout(part, name));
            }
            choice_edit(root, part, &device, &choice, name.as_deref())?.write()
        })
        .await
    }

    /// Applies `change` to the layout `name` and saves the result. A change of a built-in layout
    /// needs `device`: it saves a new layout, and the choice of that device then names it. With
    /// `world`, the choice is the one that gives the layout of that world.
    ///
    /// # Errors
    ///
    /// A [`WriteError`] for an unknown layout, a change that the layout rules refuse, a bad
    /// device, a bad world, and a failed write.
    pub async fn edit_layout(
        &self,
        name: &str,
        change: Change,
        device: Option<&str>,
        world: Option<&str>,
    ) -> Result<Edited, WriteError> {
        let device = device
            .map(DeviceName::new)
            .transpose()
            .map_err(|error| refused(&error))?;
        let world = world
            .map(str::parse::<WorldId>)
            .transpose()
            .map_err(|error| refused(&error))?;
        let name = name.to_owned();
        self.write(move |root, part| {
            let Some((lines, built_in)) = part.layout_named(&name) else {
                return Err(no_layout(part, &name));
            };
            let layout = Layout::parse(&lines).map_err(|error| refused(&error))?;
            let changed = layout.apply(change).map_err(|error| refused(&error))?;
            let lines = changed.layout.lines();
            let entries: BTreeMap<String, String> = match &device {
                Some(device) if built_in => part
                    .resolve(Kind::Tui, Os::current(), device)
                    .world_layouts
                    .into_iter()
                    .filter_map(|entry| Some((entry.key?, entry.name)))
                    .collect(),
                _ => BTreeMap::new(),
            };
            let choice = layout_for(world.as_ref(), &entries);
            if built_in && device.is_none() {
                return Err(WriteError::Refused(format!(
                    "{name} is a built-in layout: a change of it needs a device, and saves a new layout for that device"
                )));
            }
            let device_name = device.as_ref().map(DeviceName::as_str);
            let target =
                save_target(&choice, &name, device_name).map_err(|error| refused(&error))?;
            // Each check runs before the first write.
            let chosen = match (&target, &device) {
                (SaveTarget::New(new), Some(device)) => {
                    Some(choice_edit(root, part, device, &choice, Some(new.as_str()))?)
                }
                _ => None,
            };
            put_layout(root, target.name(), &lines)?;
            if let Some(edit) = chosen {
                edit.write()?;
            }
            let saved = target.name().as_str().to_owned();
            Ok(Edited {
                name: saved,
                lines,
                rewritten: changed.rewritten,
            })
        })
        .await
    }
}

/// The path of each entry outside `clients/` that differs between `before` and `after`.
fn changed(before: &DirSnapshot, after: &DirSnapshot) -> Vec<String> {
    let outside = |snapshot: &DirSnapshot| -> BTreeMap<String, SnapEntry> {
        snapshot
            .entries()
            .filter(|(path, _)| !is_under_clients(path))
            .map(|(path, entry)| (path.to_owned(), entry.clone()))
            .collect()
    };
    let (before, after) = (outside(before), outside(after));
    before
        .keys()
        .chain(after.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|path| before.get(*path) != after.get(*path))
        .map(|path| filter(path, &STORE))
        .collect()
}
