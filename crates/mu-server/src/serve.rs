//! The steps of the server process after the secrets and the socket directory: the lock,
//! the socket, the `Portal` and the stop.

use std::fs::{self, File, TryLockError};
use std::future::Future;
use std::io::{self, Write as _};
use std::os::unix::fs::{FileTypeExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mu_core::journal::DEFAULT_CAPACITY;
use mu_core::{MemoryJournal, Portal, WorldSetup};
use thiserror::Error;
use tokio::net::UnixListener;
use tokio::sync::watch;

use crate::service::PortalApi;

/// The start of the line that the server writes to stderr when it serves. The line goes on
/// with the socket path and `(process <id>)`.
pub const SERVING: &str = "mu server: serving the API on";

/// A start of the server that failed. Each error names the path.
#[derive(Debug, Error)]
pub enum ServeError {
    /// Another server holds the lock of the socket.
    #[error("a server already serves {}", .0.display())]
    Already(PathBuf),
    /// A file that is not a socket is at the socket path.
    #[error("{} exists and is not a socket, so the server keeps it and does not start", .0.display())]
    NotASocket(PathBuf),
    /// An I/O error on a path.
    #[error("{}: {source}", path.display())]
    Io {
        /// The path of the failed step.
        path: PathBuf,
        /// The error.
        source: io::Error,
    },
}

fn at(path: &Path) -> impl FnOnce(io::Error) -> ServeError + '_ {
    move |source| ServeError::Io {
        path: path.to_owned(),
        source,
    }
}

/// The path of the lock of `socket`: `<socket>.lock`.
#[must_use]
pub fn lock_path(socket: &Path) -> PathBuf {
    let mut name = socket.as_os_str().to_owned();
    name.push(".lock");
    PathBuf::from(name)
}

/// Takes the lock of `socket`. The lock lasts as long as the file stays open.
///
/// # Errors
///
/// [`ServeError::Already`] if another process holds it.
pub fn lock(socket: &Path) -> Result<File, ServeError> {
    let path = lock_path(socket);
    let file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(at(&path))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(ServeError::Already(socket.to_owned())),
        Err(TryLockError::Error(source)) => Err(ServeError::Io { path, source }),
    }
}

/// Removes a stale socket file at `socket`, and refuses any other file.
fn clear(socket: &Path) -> Result<(), ServeError> {
    match fs::symlink_metadata(socket) {
        Ok(metadata) if metadata.file_type().is_socket() => {
            // The lock makes sure that no other server serves this socket.
            fs::remove_file(socket).map_err(at(socket))
        }
        Ok(_) => Err(ServeError::NotASocket(socket.to_owned())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(at(socket)(error)),
    }
}

/// Runs steps 3 to 7 of the server: lock, clear, bind with mode 0600, write the line on
/// stderr, build the `Portal`, connect each world, and serve until `shutdown` resolves.
/// Then it ends each stream, disconnects each world and removes the socket file.
///
/// The parent of `socket` must be a private directory: owned by this user, with mode 0700.
/// Only that directory keeps other users from the socket before its mode changes to 0600.
/// `mu-cli` makes sure of it with `private_dir` before it calls this function.
///
/// # Errors
///
/// A [`ServeError`] that names the path.
pub async fn serve(
    socket: &Path,
    worlds: Vec<WorldSetup>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    let _lock = lock(socket)?;
    clear(socket)?;
    let listener = UnixListener::bind(socket).map_err(at(socket))?;
    fs::set_permissions(socket, fs::Permissions::from_mode(0o600)).map_err(at(socket))?;
    // A closed stderr leaves nothing to report to.
    // One write for the line: servers that share a log append whole lines.
    let line = format!(
        "{SERVING} {} (process {})\n",
        socket.display(),
        std::process::id()
    );
    let _ = io::stderr().write_all(line.as_bytes());

    let journal = MemoryJournal::new(DEFAULT_CAPACITY)
        .unwrap_or_else(|_| unreachable!("the default capacity is valid"));
    let portal = Portal::start(Arc::new(journal), worlds, Vec::new());
    portal.connect_all().await;

    let (stopping, stop_receiver) = watch::channel(false);
    let api = PortalApi::new(
        portal.clone(),
        uuid::Uuid::new_v4().to_string(),
        stop_receiver,
    );
    let served = axum::serve(listener, api.router())
        .with_graceful_shutdown(async move {
            shutdown.await;
            // Each open stream ends, so the graceful shutdown does not wait for it.
            stopping.send_replace(true);
        })
        .await;

    for (world, _) in portal.worlds() {
        // A world that is down already is not an error at the stop.
        let _ = portal.disconnect(&world).await;
    }
    let removed = fs::remove_file(socket).map_err(at(socket));
    served.map_err(at(socket))?;
    removed
}

/// Resolves on the first SIGTERM or SIGINT.
///
/// # Errors
///
/// An I/O error if the process cannot listen for the signals.
pub fn signals() -> io::Result<impl Future<Output = ()> + Send + 'static> {
    use tokio::signal::unix::{SignalKind, signal};
    let mut term = signal(SignalKind::terminate())?;
    let mut int = signal(SignalKind::interrupt())?;
    Ok(async move {
        tokio::select! {
            _ = term.recv() => {}
            _ = int.recv() => {}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_held_lock_is_already() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("s.sock");
        let _first = lock(&socket).unwrap();
        let error = lock(&socket).unwrap_err();
        assert!(matches!(error, ServeError::Already(_)));
        assert!(error.to_string().contains(&socket.display().to_string()));
    }

    #[test]
    fn a_regular_file_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("s.sock");
        fs::write(&socket, "keep").unwrap();
        assert!(matches!(clear(&socket), Err(ServeError::NotASocket(_))));
        assert_eq!(fs::read_to_string(&socket).unwrap(), "keep");
    }
}
