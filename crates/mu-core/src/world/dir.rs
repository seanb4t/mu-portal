//! The layers of a configuration directory of TOML files.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::id::{PART_RULE, WorldId, is_part};

use super::error::ConfigError;
use super::layer::{Layer, LayerFile, LayerKey, LayerOrigin, LayerSource, Trust};
use super::parse::parse;

/// The layers of a configuration directory.
///
/// The directory holds `global.toml`, `types/<name>.toml`, `games/<game>.toml` and
/// `characters/<game>/<character>.toml`. Any other file whose name ends in `.toml`, in any
/// case, is an error.
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
        let mut files = Vec::new();
        match fs::metadata(&self.root) {
            Ok(_) => walk(&self.root, &mut Vec::new(), &mut files)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(ConfigError::new(self.root.display(), error.to_string())),
        }
        files.sort();
        let keys = files
            .iter()
            .map(|parts| key_of(parts))
            .collect::<Result<Vec<_>, _>>()?;
        files
            .iter()
            .zip(keys)
            .map(|(parts, key)| self.layer(parts, key))
            .collect()
    }
}

impl TomlDirSource {
    /// Reads the file at `parts` as the layer `key`.
    fn layer(&self, parts: &[String], key: LayerKey) -> Result<Layer, ConfigError> {
        let origin = LayerOrigin::File(parts.join("/"));
        let path: PathBuf = parts
            .iter()
            .fold(self.root.clone(), |path, part| path.join(part));
        let text = fs::read_to_string(&path)
            .map_err(|error| ConfigError::new(&origin, error.to_string()))?;
        let (extends, fields) = parse::<LayerFile>(&origin, &text)?.split();
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
}

/// Adds to `files` the path of each file under `directory` whose name ends in `.toml`, in any
/// case, as its parts relative to the root. `prefix` holds the parts of `directory`.
fn walk(
    directory: &Path,
    prefix: &mut Vec<String>,
    files: &mut Vec<Vec<String>>,
) -> Result<(), ConfigError> {
    let place = if prefix.is_empty() {
        directory.display().to_string()
    } else {
        prefix.join("/")
    };
    let entries =
        fs::read_dir(directory).map_err(|error| ConfigError::new(&place, error.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|error| ConfigError::new(&place, error.to_string()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        let toml = name.to_ascii_lowercase().ends_with(".toml");
        prefix.push(name);
        if path.is_dir() {
            walk(&path, prefix, files)?;
        } else if toml {
            files.push(prefix.clone());
        }
        prefix.pop();
    }
    Ok(())
}

/// The layer that the file at `parts` holds.
fn key_of(parts: &[String]) -> Result<LayerKey, ConfigError> {
    let path = parts.join("/");
    let stem = |file: &str| -> Result<String, ConfigError> {
        let stem = file.strip_suffix(".toml").unwrap_or(file);
        if is_part(stem) {
            Ok(stem.to_owned())
        } else {
            Err(ConfigError::new(&path, format!("the name {PART_RULE}")))
        }
    };
    match parts {
        [file] if file == "global.toml" => Ok(LayerKey::Global),
        [directory, file] if directory == "types" => Ok(LayerKey::Type(stem(file)?)),
        [directory, file] if directory == "games" => Ok(LayerKey::Game(stem(file)?)),
        [directory, game, file] if directory == "characters" => {
            let character = stem(file)?;
            let id: WorldId = format!("{game}/{character}")
                .parse()
                .map_err(|_| ConfigError::new(&path, format!("the game {PART_RULE}")))?;
            Ok(LayerKey::Character(id))
        }
        _ => Err(ConfigError::new(
            &path,
            "not a file of the configuration: global.toml, types/<name>.toml, \
             games/<game>.toml or characters/<game>/<character>.toml",
        )),
    }
}
