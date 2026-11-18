//! `server.toml`, the configuration of the server in the configuration directory.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use mu_core::world::{ConfigError, parse_toml};
use serde::Deserialize;

/// The name of the file in the configuration directory.
pub const SERVER_FILE: &str = "server.toml";

/// The content of `server.toml`. A missing file is the default.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerFile {
    /// The table `[telemetry]`.
    pub telemetry: Option<TelemetryTable>,
}

/// The table `[telemetry]`. Its values are checked when the export is on.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryTable {
    /// The base URL of the collector.
    pub endpoint: Option<String>,
    /// `http/protobuf` or `grpc`.
    pub protocol: Option<String>,
    /// A PEM file of the authorities to trust beside the roots of the system.
    pub certificate: Option<PathBuf>,
}

/// An error of `server.toml`.
#[derive(Debug, thiserror::Error)]
pub enum ServerFileError {
    /// The file holds a bad table, field or value.
    #[error(transparent)]
    Parse(#[from] ConfigError),
    /// The file cannot be read.
    #[error("{SERVER_FILE}: {0}")]
    Read(#[source] io::Error),
}

/// Reads `server.toml` from `dir`. With no directory, or no file, it is the default.
///
/// # Errors
///
/// A [`ServerFileError`] that names `server.toml`, and the field when there is one.
pub fn read(dir: Option<&Path>) -> Result<ServerFile, ServerFileError> {
    let Some(dir) = dir else {
        return Ok(ServerFile::default());
    };
    match fs::read_to_string(dir.join(SERVER_FILE)) {
        Ok(text) => Ok(parse_toml(&SERVER_FILE, &text)?),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(ServerFile::default()),
        Err(error) => Err(ServerFileError::Read(error)),
    }
}
