//! The configuration store: the one owner of the files of the configuration directory. It
//! resolves the worlds at the start, checks each client layer, resolves the configuration of
//! one client, and reads the directory again at a reload.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mu_config::{ClientLayer, DeviceName, Echo, Kind, NameError, Os, STORE, check_keys, filter};
use mu_core::ResolvedWorld;
use mu_core::world::{
    CLIENTS_DIR, ConfigError, DirSnapshot, Hidden, SnapEntry, is_under_clients, merge_layers,
    parse_toml, resolve, world_errors,
};
use sha2::{Digest as _, Sha256};
use tokio::sync::{Mutex, watch};

/// The check of `server.toml` that the start of `mu-cli` also runs. It takes the entry of the
/// file in the snapshot, or none when the directory holds no such file, and gives the text of
/// the error.
pub type ServerCheck = Box<dyn Fn(Option<&SnapEntry>) -> Result<(), String> + Send + Sync>;

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
    /// The path of each field, such as `echo` or `keys.alt+n`, and the layer that set it, or
    /// `default`.
    pub sources: BTreeMap<String, String>,
    /// One notice for each bad layer that applies to the client, and for each bad entry at the
    /// path of a layer directory.
    pub notices: Vec<Notice>,
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
        dir: "clients/devices",
        shown: "<device>",
        check: |name| DeviceName::new(name).map(drop),
    },
];

/// The directories on the path of each layer of a client.
fn layer_dirs<'a>() -> impl Iterator<Item = &'a str> {
    std::iter::once(CLIENTS_DIR).chain(PLACES.iter().map(|place| place.dir))
}

/// The text of a path under `clients/` that is not a layer file.
fn not_a_layer() -> String {
    let places: Vec<String> = PLACES
        .iter()
        .map(|place| format!("{}/{}.toml", place.dir, place.shown))
        .collect();
    format!(
        "not a layer file: the layer files are {GLOBAL}, {}",
        mu_config::listed(&places)
    )
}

/// Whether `path` is a layer file, or the reason that it is not.
fn layer_rule(path: &str) -> Result<(), String> {
    if path == GLOBAL {
        return Ok(());
    }
    for place in &PLACES {
        if let Some(name) = path
            .strip_prefix(place.dir)
            .and_then(|rest| rest.strip_prefix('/'))
            .and_then(|file| file.strip_suffix(".toml"))
            .filter(|name| !name.contains('/'))
        {
            return (place.check)(name).map_err(|error| error.to_string());
        }
    }
    Err(not_a_layer())
}

/// What the entry at `path` under `clients/` holds.
fn state(path: &str, entry: &SnapEntry) -> State {
    let bad = |reason: &str| State::Bad(Notice::new(path, reason));
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
            Err(error) => return bad(&format!("keys.{}: {}", error.key, error.reason)),
        }
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
                State::Good(_) => None,
            })
            .collect()
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
                    .map(|(place, name)| format!("{}/{name}.toml", place.dir)),
            )
            .collect();
        let mut notices = Vec::new();
        let mut good: Vec<(&str, &ClientLayer)> = Vec::new();
        for path in layer_dirs().chain(files.iter().map(String::as_str)) {
            match self.states.get(path) {
                Some(State::Bad(notice)) => notices.push(notice.clone()),
                Some(State::Good(layer)) => good.push((path, layer)),
                None => {}
            }
        }
        let revision = digest(&framed(
            layer_dirs()
                .chain(files.iter().map(String::as_str))
                .collect::<BTreeSet<&str>>()
                .into_iter()
                .filter_map(|path| Some((path, Content::of(self.entries.get(path)?)))),
        ));
        let defaults = ClientLayer {
            echo: Some(Echo::default()),
            keys: None,
        };
        let mut layers = vec![&defaults];
        layers.extend(good.iter().map(|(_, layer)| *layer));
        let mut paths = vec!["echo".to_owned()];
        let keys: BTreeSet<&String> = good
            .iter()
            .filter_map(|(_, layer)| layer.keys.as_ref())
            .flat_map(BTreeMap::keys)
            .collect();
        paths.extend(keys.iter().map(|key| format!("keys.{key}")));
        let (merged, places) = merge_layers(&layers, &paths).unwrap_or_else(|error| {
            notices.push(Notice::new(
                CLIENTS_DIR,
                &format!("the merge of the layers failed, so the defaults apply: {error}"),
            ));
            (defaults.clone(), BTreeMap::new())
        });
        let sources = places
            .into_iter()
            .map(|(path, place)| {
                let source = place
                    .checked_sub(1)
                    .and_then(|place| good.get(place))
                    .map_or("default", |(path, _)| path);
                (path, source.to_owned())
            })
            .collect();
        ClientConfig {
            store_hash: self.store_hash.clone(),
            revision,
            echo: merged.echo.unwrap_or_default(),
            bindings: merged.keys.unwrap_or_default().into_iter().collect(),
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
}

impl Reloaded {
    /// A reload that read no snapshot.
    fn failed(notice: Notice) -> Self {
        Self {
            notices: vec![notice],
            restart: Vec::new(),
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
}

impl Store {
    /// The store of the directory `root`, from `snapshot`, its snapshot at the start.
    ///
    /// # Errors
    ///
    /// The first [`ConfigError`] of the worlds: a bad world stops the start.
    pub fn open(
        root: Option<&Path>,
        snapshot: DirSnapshot,
        server_check: ServerCheck,
    ) -> Result<Opened, ConfigError> {
        let worlds = resolve(&[&snapshot])?;
        let part = ClientPart::new(&snapshot);
        let notices = part.notices();
        let store = Self {
            root: root.map(Path::to_path_buf),
            start: snapshot,
            client: watch::Sender::new(Arc::new(part)),
            reload: Mutex::new(()),
            server_check,
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
            tracing::warn!("{notice}");
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
        }
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
