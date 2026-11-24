//! The read of a TOML file at the top of the configuration directory, such as `server.toml` and
//! `client.toml`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use mu_core::world::{ConfigError, Hidden, parse_toml};
use serde::de::DeserializeOwned;

/// An error of a file of the configuration directory.
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    /// The file holds a bad table, field or value.
    #[error(transparent)]
    Parse(#[from] ConfigError),
    /// The entry at `path` in the snapshot of the directory cannot be read.
    #[error("{}: {text}", path.display())]
    Unreadable {
        /// The path of the file in the configuration directory.
        path: PathBuf,
        /// The error of the read.
        text: String,
    },
    /// The file at `path` cannot be read.
    #[error("{}: {source}", path.display())]
    Read {
        /// The path of the file in the configuration directory.
        path: PathBuf,
        /// The error of the read.
        source: io::Error,
    },
}

/// Reads the file `name` from `dir` and parses it with the `hidden` rule of its values. With no
/// directory, or no file, it is the default.
///
/// # Errors
///
/// A [`FileError`] that names the file, and the field or the line when there is one. An error of
/// the read names the path of the file, which holds the directory.
pub fn read<T: DeserializeOwned + Default>(
    dir: Option<&Path>,
    name: &str,
    hidden: Hidden<'_>,
) -> Result<T, FileError> {
    let Some(dir) = dir else {
        return Ok(T::default());
    };
    let path = dir.join(name);
    match fs::read_to_string(&path) {
        Ok(text) => Ok(parse_toml(&name, &text, hidden)?),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(T::default()),
        Err(source) => Err(FileError::Read { path, source }),
    }
}
