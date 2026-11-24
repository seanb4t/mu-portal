//! `server.toml`, the configuration of the server in the configuration directory.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub use mu_core::world::SERVER_FILE;
use mu_core::world::{Hidden, SnapEntry, parse_toml};
use serde::Deserialize;

pub use crate::config_file::FileError as ServerFileError;
use crate::telemetry::{self, Export, SettingError};

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

/// An error of `server.toml`: of the file, or of a value of `[telemetry]`.
#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    /// The file cannot be read, or holds a bad table, field or value.
    #[error(transparent)]
    File(#[from] ServerFileError),
    /// A bad value of the telemetry.
    #[error(transparent)]
    Setting(#[from] SettingError),
}

/// Parses `entry`, the entry of `server.toml` in the snapshot of `dir`. With no entry, it is the
/// default. A relative `certificate` becomes a path under `dir`.
///
/// # Errors
///
/// A [`ServerFileError`] that names `server.toml`, and the field when there is one. An error of
/// the read names the path of the file, which holds the directory.
pub fn read(dir: Option<&Path>, entry: Option<&SnapEntry>) -> Result<ServerFile, ServerFileError> {
    let (Some(dir), Some(entry)) = (dir, entry) else {
        return Ok(ServerFile::default());
    };
    let unreadable = |text: &str| ServerFileError::Unreadable {
        path: dir.join(SERVER_FILE),
        text: text.to_owned(),
    };
    let text = match entry {
        SnapEntry::File(bytes) => std::str::from_utf8(bytes)
            .map_err(|_| unreadable("stream did not contain valid UTF-8"))?,
        SnapEntry::NotRegular => return Err(unreadable("not a regular file")),
        SnapEntry::Error { text, .. } => return Err(unreadable(text)),
    };
    let mut file: ServerFile = parse_toml(&SERVER_FILE, text, Hidden::All)?;
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

/// Checks `server.toml` as the start and each reload do: [`read`], then the values of the
/// export with the variables that `var` reads.
///
/// # Errors
///
/// A [`CheckError`] that names the file or the variable.
pub fn check(
    dir: Option<&Path>,
    entry: Option<&SnapEntry>,
    var: impl Fn(&str) -> Option<OsString>,
) -> Result<Option<Export>, CheckError> {
    let file = read(dir, entry)?;
    Ok(telemetry::export(var, &file.telemetry)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(text: &str) -> SnapEntry {
        SnapEntry::File(text.as_bytes().to_vec())
    }

    fn read_text(text: &str) -> (PathBuf, ServerFile) {
        let dir = PathBuf::from("/etc/mu");
        let file = read(Some(&dir), Some(&entry(text))).unwrap();
        (dir, file)
    }

    fn certificate(file: &ServerFile) -> &Path {
        file.telemetry.certificate.as_deref().unwrap()
    }

    #[test]
    fn error_under_telemetry_holds_no_value() {
        let text = entry("telemetry = \"https://u:hunter2@h\"\n");
        let error = read(Some(Path::new("/etc/mu")), Some(&text))
            .unwrap_err()
            .to_string();
        assert!(!error.contains("hunter2"), "{error}");
        assert!(error.contains("telemetry"), "{error}");
    }

    #[test]
    fn read_error_names_the_path() {
        let unreadable = SnapEntry::Error {
            kind: "permission denied".to_owned(),
            text: "Permission denied (os error 13)".to_owned(),
        };
        let error = read(Some(Path::new("/etc/mu")), Some(&unreadable))
            .unwrap_err()
            .to_string();
        assert_eq!(
            error,
            "/etc/mu/server.toml: Permission denied (os error 13)"
        );
    }

    #[test]
    fn relative_certificate_starts_at_the_directory() {
        let (dir, file) = read_text("[telemetry]\ncertificate = \"certs/ca.pem\"\n");
        assert_eq!(certificate(&file), dir.join("certs/ca.pem"));
    }

    #[test]
    fn absolute_certificate_is_kept() {
        let (_dir, file) = read_text("[telemetry]\ncertificate = \"/etc/mu/ca.pem\"\n");
        assert_eq!(certificate(&file), Path::new("/etc/mu/ca.pem"));
    }
}
