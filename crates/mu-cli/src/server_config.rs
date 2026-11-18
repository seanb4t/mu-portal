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
    /// The table `[telemetry]`. A missing table is the default, the same as an empty one.
    #[serde(default)]
    pub telemetry: TelemetryTable,
}

/// The table `[telemetry]`. Its values are checked when the export is on. `endpoint` and
/// `protocol` are checked even when a variable wins over them, but `certificate` only when
/// `OTEL_EXPORTER_OTLP_CERTIFICATE` is unset.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryTable {
    /// The base URL of the collector.
    pub endpoint: Option<String>,
    /// `http/protobuf` or `grpc`.
    pub protocol: Option<String>,
    /// A PEM file of the authorities to trust beside the roots of the system. In the file, a
    /// relative path starts at the configuration directory: [`read`] joins it to that directory,
    /// so a table that `read` returns holds no relative path.
    pub certificate: Option<PathBuf>,
}

/// The tables of `server.toml` whose values can hold a password: an endpoint URL can.
const HIDDEN: &[&str] = &["telemetry"];

/// An error of `server.toml`.
#[derive(Debug, thiserror::Error)]
pub enum ServerFileError {
    /// The file holds a bad table, field or value.
    #[error(transparent)]
    Parse(#[from] ConfigError),
    /// The file cannot be read.
    #[error("{SERVER_FILE}: {0}")]
    Read(io::Error),
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
            let mut file: ServerFile = parse_toml(&SERVER_FILE, &text, HIDDEN)?;
            if let Some(path) = file
                .telemetry
                .certificate
                .as_mut()
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
        file.telemetry.certificate.as_deref().unwrap()
    }

    #[test]
    fn error_under_telemetry_holds_no_value() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(SERVER_FILE),
            "telemetry = \"https://u:hunter2@h\"\n",
        )
        .unwrap();
        let error = read(Some(dir.path())).unwrap_err().to_string();
        assert!(!error.contains("hunter2"), "{error}");
        assert!(error.contains("telemetry"), "{error}");
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
