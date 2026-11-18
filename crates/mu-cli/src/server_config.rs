//! `server.toml`, the configuration of the server in the configuration directory.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub use mu_core::world::SERVER_FILE;
use mu_core::world::{ConfigError, parse_toml};
use serde::Deserialize;

/// The content of `server.toml`. A missing file is the default.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerFile {
    /// The table `[telemetry]`.
    pub telemetry: Option<TelemetryTable>,
}

/// The table `[telemetry]`. Its values are checked when the export is on, but `certificate`
/// only when `OTEL_EXPORTER_OTLP_CERTIFICATE` is unset.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryTable {
    /// The base URL of the collector.
    pub endpoint: Option<String>,
    /// `http/protobuf` or `grpc`.
    pub protocol: Option<String>,
    /// A PEM file of the authorities to trust beside the roots of the system. A relative path
    /// starts at the configuration directory.
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

/// Reads `server.toml` from `dir`. With no directory, or no file, it is the default. A relative
/// `certificate` becomes a path under `dir`.
///
/// # Errors
///
/// A [`ServerFileError`] that names `server.toml`, and the field when there is one.
pub fn read(dir: Option<&Path>) -> Result<ServerFile, ServerFileError> {
    let Some(dir) = dir else {
        return Ok(ServerFile::default());
    };
    match fs::read_to_string(dir.join(SERVER_FILE)) {
        Ok(text) => {
            let mut file: ServerFile = parse_toml(&SERVER_FILE, &text)?;
            if let Some(path) = file
                .telemetry
                .as_mut()
                .and_then(|table| table.certificate.as_mut())
                .filter(|path| path.is_relative())
            {
                *path = dir.join(&*path);
            }
            Ok(file)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(ServerFile::default()),
        Err(error) => Err(ServerFileError::Read(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_text(text: &str) -> (tempfile::TempDir, ServerFile) {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(SERVER_FILE), text).unwrap();
        let file = read(Some(dir.path())).unwrap();
        (dir, file)
    }

    fn certificate(file: &ServerFile) -> &Path {
        file.telemetry
            .as_ref()
            .unwrap()
            .certificate
            .as_deref()
            .unwrap()
    }

    #[test]
    fn relative_certificate_starts_at_the_directory() {
        let (dir, file) = read_text("[telemetry]\ncertificate = \"certs/ca.pem\"\n");
        assert_eq!(certificate(&file), dir.path().join("certs/ca.pem"));
    }

    #[test]
    fn absolute_certificate_is_kept() {
        let (_dir, file) = read_text("[telemetry]\ncertificate = \"/etc/mu/ca.pem\"\n");
        assert_eq!(certificate(&file), Path::new("/etc/mu/ca.pem"));
    }
}
