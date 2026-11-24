//! The layers of a configuration directory of TOML files.

use std::path::PathBuf;

use crate::id::{PART_RULE, WorldId, is_part};

use super::error::ConfigError;
use super::layer::{Layer, LayerFile, LayerKey, LayerOrigin, LayerSource, Trust};
use super::parse::{LAYER_HIDDEN, parse};
use super::snapshot::{DirSnapshot, SnapEntry};

/// The layers of a configuration directory.
///
/// The directory holds `global.toml`, `types/<name>.toml`, `games/<game>.toml` and
/// `characters/<game>/<character>.toml`, and [`SERVER_FILE`] and the directory
/// [`CLIENTS_DIR`](super::CLIENTS_DIR) at the top, which hold no layer of a world.
/// Any other file whose name ends in `.toml`, in any case, is an error. So is an entry that
/// cannot be read, such as a link to nothing.
/// A missing directory holds no layers.
#[derive(Clone, Debug)]
pub struct TomlDirSource {
    root: PathBuf,
}

impl TomlDirSource {
    /// The source of the directory `root`.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl LayerSource for TomlDirSource {
    fn layers(&self) -> Result<Vec<Layer>, ConfigError> {
        DirSnapshot::take(&self.root)?.layers()
    }
}

impl LayerSource for DirSnapshot {
    fn layers(&self) -> Result<Vec<Layer>, ConfigError> {
        let entries: Vec<(&str, &SnapEntry)> = self.world_entries().collect();
        if let Some((path, text)) = entries.iter().find_map(|(path, entry)| match entry {
            SnapEntry::Error { text, .. } => Some((*path, text)),
            _ => None,
        }) {
            return Err(ConfigError::new(path, text.clone()));
        }
        let keys = entries
            .iter()
            .map(|(path, _)| key_of(path))
            .collect::<Result<Vec<_>, _>>()?;
        entries
            .iter()
            .zip(keys)
            .map(|((path, entry), key)| layer(path, entry, key))
            .collect()
    }
}

/// Reads the file at `path` as the layer `key`.
fn layer(path: &str, entry: &SnapEntry, key: LayerKey) -> Result<Layer, ConfigError> {
    let origin = LayerOrigin::File(path.to_owned());
    let text = match entry {
        SnapEntry::File(bytes) => std::str::from_utf8(bytes)
            .map_err(|_| ConfigError::new(&origin, "stream did not contain valid UTF-8"))?,
        SnapEntry::NotRegular => {
            return Err(ConfigError::new(&origin, "not a regular file"));
        }
        SnapEntry::Error { text, .. } => return Err(ConfigError::new(&origin, text.clone())),
    };
    let (extends, fields) = parse::<LayerFile>(&origin, text, LAYER_HIDDEN)?.split();
    match (&key, &extends) {
        (LayerKey::Game(_), None) => {
            return Err(ConfigError::field(
                &origin,
                "extends",
                "a game must name its type",
            ));
        }
        (LayerKey::Global | LayerKey::Character(_), Some(_)) => {
            return Err(ConfigError::field(
                &origin,
                "extends",
                "only a game or a type names a parent",
            ));
        }
        _ => {}
    }
    Ok(Layer {
        key,
        origin,
        trust: Trust::Owner,
        extends,
        fields,
    })
}

/// The file of the server at the top of the directory. It holds no layer.
pub const SERVER_FILE: &str = "server.toml";

/// The file of the terminal client that this version no longer reads.
const OLD_CLIENT_FILE: &str = "client.toml";

/// The layer that the file at `path` holds.
fn key_of(path: &str) -> Result<LayerKey, ConfigError> {
    let parts: Vec<&str> = path.split('/').collect();
    let stem = |file: &str| -> Result<String, ConfigError> {
        let stem = file.strip_suffix(".toml").unwrap_or(file);
        if is_part(stem) {
            Ok(stem.to_owned())
        } else {
            Err(ConfigError::new(path, format!("the name {PART_RULE}")))
        }
    };
    match parts.as_slice() {
        [file] if *file == "global.toml" => Ok(LayerKey::Global),
        [file] if file.eq_ignore_ascii_case(OLD_CLIENT_FILE) => Err(ConfigError::new(
            path,
            "the client configuration moved: move its content to clients/global.toml, \
             without the table [server]",
        )),
        [directory, file] if *directory == "types" => Ok(LayerKey::Type(stem(file)?)),
        [directory, file] if *directory == "games" => Ok(LayerKey::Game(stem(file)?)),
        [directory, game, file] if *directory == "characters" => {
            let character = stem(file)?;
            let id: WorldId = format!("{game}/{character}")
                .parse()
                .map_err(|_| ConfigError::new(path, format!("the game {PART_RULE}")))?;
            Ok(LayerKey::Character(id))
        }
        _ => Err(ConfigError::new(
            path,
            "not a file of the configuration: global.toml, types/<name>.toml, \
             games/<game>.toml or characters/<game>/<character>.toml",
        )),
    }
}
