//! The error of a TOML file at the top of the configuration directory, such as `server.toml`.

use std::path::PathBuf;

use mu_core::world::ConfigError;

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
}
