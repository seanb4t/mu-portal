//! `server.toml`, the configuration of the server in the configuration directory.

use std::path::{Path, PathBuf};

use mu_core::world::Hidden;
pub use mu_core::world::SERVER_FILE;
use serde::Deserialize;

pub use crate::config_file::FileError as ServerFileError;

/// The content of `server.toml`. A missing file is the default. An error of the file quotes no
/// value, because a value such as an endpoint URL can hold a password.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerFile {
    /// The table `[telemetry]`. A missing table is the default, the same as an empty one.
    #[serde(default)]
    pub telemetry: TelemetryTable,
}

/// The table `[telemetry]`. Its fields and their types are checked at each start, with the
/// export on or off. Its values are checked only when the export is on. `endpoint` and
/// `protocol` are checked even when a variable wins over them, but `certificate` only when
/// `OTEL_EXPORTER_OTLP_CERTIFICATE` is unset. The gRPC rule of no path and no query applies
/// only to the endpoint that a signal uses.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryTable {
    /// The base URL of the collector.
    pub endpoint: Option<String>,
    /// `http/protobuf` or `grpc`.
    pub protocol: Option<String>,
    /// A PEM file of the authorities to trust beside the roots of the system. In the file, a
    /// relative path starts at the configuration directory: [`read`] joins it to that directory.
    /// A path that is still relative after the join starts at the working directory, as the
    /// configuration directory does.
    pub certificate: Option<PathBuf>,
}

/// Reads `server.toml` from `dir`. With no directory, or no file, it is the default. A relative
/// `certificate` becomes a path under `dir`.
///
/// # Errors
///
/// A [`ServerFileError`] that names `server.toml`, and the field when there is one. An error of
/// the read names the path of the file, which holds the directory.
pub fn read(dir: Option<&Path>) -> Result<ServerFile, ServerFileError> {
    let mut file: ServerFile = crate::config_file::read(dir, SERVER_FILE, Hidden::All)?;
    if let Some(dir) = dir
        && let Some(path) = file
            .telemetry
            .certificate
            .as_mut()
            .filter(|path| path.is_relative())
    {
        *path = dir.join(&*path);
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use std::fs;

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
    fn read_error_names_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("not-a-directory");
        fs::write(&file, "").unwrap();
        let error = read(Some(&file)).unwrap_err().to_string();
        assert!(
            error.contains(&file.join(SERVER_FILE).display().to_string()),
            "{error}"
        );
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
