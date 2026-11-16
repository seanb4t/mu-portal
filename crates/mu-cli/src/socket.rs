//! The socket path of the server and the private directory that holds it.

use std::ffi::OsString;
use std::fs::{self, DirBuilder};
use std::io;
use std::os::unix::fs::{DirBuilderExt as _, MetadataExt as _};
use std::path::{Path, PathBuf};

use clap::Args;
use thiserror::Error;

/// The variable that names the socket.
pub const SOCKET_ENV: &str = "MU_SOCKET";

/// The path of the socket: `--socket`, then `MU_SOCKET`, then the default.
#[derive(Clone, Debug, Default, Args)]
pub struct SocketArgs {
    /// The Unix socket of the server
    #[arg(long, env = SOCKET_ENV, value_name = "PATH", value_parser = parse_socket)]
    socket: Option<SocketValue>,
}

/// A value of `--socket` or `MU_SOCKET`. An empty value is none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SocketValue(Option<PathBuf>);

/// Turns an empty value into none, so that `clap` does not refuse it.
#[expect(
    clippy::unnecessary_wraps,
    reason = "clap takes a value parser that returns a Result"
)]
fn parse_socket(value: &str) -> Result<SocketValue, std::convert::Infallible> {
    Ok(SocketValue(
        (!value.is_empty()).then(|| PathBuf::from(value)),
    ))
}

impl SocketArgs {
    /// The socket path, with `runtime_dir` as the value of `XDG_RUNTIME_DIR`.
    #[must_use]
    pub fn path(&self, runtime_dir: Option<OsString>, uid: u32) -> PathBuf {
        self.socket
            .as_ref()
            .and_then(|value| value.0.clone())
            .unwrap_or_else(|| default_socket(runtime_dir, uid))
    }

    /// The socket path from the environment of the process.
    #[must_use]
    pub fn resolve(&self) -> PathBuf {
        self.path(std::env::var_os("XDG_RUNTIME_DIR"), current_uid())
    }

    /// The absolute socket path, after [`private_dir`] accepts its directory. A relative path
    /// is relative to the working directory.
    ///
    /// # Errors
    ///
    /// A [`PrivateDirError`] that names the directory.
    pub fn private_socket(&self) -> Result<PathBuf, PrivateDirError> {
        let socket = self.resolve();
        let socket = std::path::absolute(&socket).map_err(|source| PrivateDirError::Io {
            path: socket.clone(),
            source,
        })?;
        private_dir(socket_dir(&socket), current_uid())?;
        Ok(socket)
    }
}

/// The directory of an absolute socket path.
#[must_use]
pub fn socket_dir(socket: &Path) -> &Path {
    socket.parent().unwrap_or_else(|| Path::new("/"))
}

/// The user id of the process.
#[must_use]
pub fn current_uid() -> u32 {
    rustix::process::getuid().as_raw()
}

/// `server.sock` in `mu` under the runtime directory. An empty or relative runtime directory
/// counts as unset, and then the directory is `mu-<uid>` under the temporary directory.
#[must_use]
pub fn default_socket(runtime_dir: Option<OsString>, uid: u32) -> PathBuf {
    let runtime = runtime_dir
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute());
    let dir = match runtime {
        Some(runtime) => runtime.join("mu"),
        None => std::env::temp_dir().join(format!("mu-{uid}")),
    };
    dir.join("server.sock")
}

/// A socket directory that is not private.
#[derive(Debug, Error)]
pub enum PrivateDirError {
    /// The path is not a directory.
    #[error("{} is not a directory", .0.display())]
    NotADirectory(PathBuf),
    /// Another user owns the directory.
    #[error("{} belongs to the user id {owner}, not to {expected}", path.display())]
    Owner {
        /// The directory.
        path: PathBuf,
        /// Its owner.
        owner: u32,
        /// The user id of the process.
        expected: u32,
    },
    /// The group or others have access.
    #[error("{} has the mode {mode:04o}; the group and others must have no access", path.display())]
    Open {
        /// The directory.
        path: PathBuf,
        /// Its permission bits.
        mode: u32,
    },
    /// An I/O error on the directory.
    #[error("{}: {source}", path.display())]
    Io {
        /// The directory.
        path: PathBuf,
        /// The error.
        source: io::Error,
    },
}

/// Creates `dir` with mode 0700 if it is missing. Refuses a directory that `owner` does not
/// own, or whose mode gives access to the group or to others.
///
/// # Errors
///
/// A [`PrivateDirError`] that names `dir`.
pub fn private_dir(dir: &Path, owner: u32) -> Result<(), PrivateDirError> {
    let io = |source| PrivateDirError::Io {
        path: dir.to_owned(),
        source,
    };
    match fs::symlink_metadata(dir) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(dir)
                .map_err(io)?;
        }
        Err(error) => return Err(io(error)),
        Ok(_) => {}
    }
    let metadata = fs::symlink_metadata(dir).map_err(io)?;
    if !metadata.is_dir() {
        return Err(PrivateDirError::NotADirectory(dir.to_owned()));
    }
    if metadata.uid() != owner {
        return Err(PrivateDirError::Owner {
            path: dir.to_owned(),
            owner: metadata.uid(),
            expected: owner,
        });
    }
    let mode = metadata.mode() & 0o7777;
    if mode & 0o077 != 0 {
        return Err(PrivateDirError::Open {
            path: dir.to_owned(),
            mode,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt as _;

    use super::*;

    const UID: u32 = 501;

    fn under_tmp() -> PathBuf {
        std::env::temp_dir().join("mu-501/server.sock")
    }

    #[test]
    fn default_socket_under_runtime_dir_path() {
        let path = default_socket(Some("/run/user/501".into()), UID);
        assert_eq!(path, Path::new("/run/user/501/mu/server.sock"));
    }

    #[test]
    fn default_socket_without_runtime_dir() {
        assert_eq!(default_socket(None, UID), under_tmp());
    }

    #[test]
    fn empty_runtime_dir_counts_as_unset() {
        assert_eq!(default_socket(Some(OsString::new()), UID), under_tmp());
    }

    #[test]
    fn relative_runtime_dir_counts_as_unset() {
        assert_eq!(default_socket(Some("run/user".into()), UID), under_tmp());
    }

    #[test]
    fn empty_socket_env_counts_as_unset() {
        let args = SocketArgs {
            socket: Some(parse_socket("").unwrap()),
        };
        let path = args.path(Some("/run/user/501".into()), UID);
        assert_eq!(path, Path::new("/run/user/501/mu/server.sock"));
    }

    #[test]
    fn private_dir_creates_mode_0700() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("run");
        private_dir(&dir, current_uid()).unwrap();
        assert_eq!(fs::metadata(&dir).unwrap().mode() & 0o777, 0o700);
    }

    #[test]
    fn private_dir_refuses_another_owner() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("run");
        DirBuilder::new().mode(0o700).create(&dir).unwrap();
        let owner = fs::metadata(&dir).unwrap().uid();
        let error = private_dir(&dir, owner + 1).unwrap_err();
        assert!(matches!(error, PrivateDirError::Owner { .. }));
        assert!(error.to_string().contains(&dir.display().to_string()));
    }

    #[test]
    fn private_dir_refuses_group_access() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("run");
        DirBuilder::new().mode(0o700).create(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o750)).unwrap();
        let error = private_dir(&dir, current_uid()).unwrap_err();
        assert!(matches!(error, PrivateDirError::Open { mode: 0o750, .. }));
    }
}
