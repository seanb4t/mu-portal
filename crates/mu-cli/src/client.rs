//! The verb `mu client`.

use std::fs::{File, OpenOptions};
use std::io::{self, IsTerminal as _};
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

use clap::Args;
use mu_client::PortalClient;
use mu_core::world::{CLIENT_FILE, Hidden};
use mu_tui::config::{ClientConfig, Settings};
use tokio::net::UnixStream;

use crate::config::ConfigArgs;
use crate::socket::{SocketArgs, socket_dir};

/// The log of a server that a client started, in the socket directory.
pub const SERVER_LOG: &str = "server.log";

/// Arguments of the client. `mu` and `mu client` both accept them.
#[derive(Clone, Debug, Default, Args)]
pub struct ClientArgs {
    #[command(flatten)]
    socket: SocketArgs,
    #[command(flatten)]
    config: ConfigArgs,
}

fn fail(error: &dyn std::fmt::Display) -> ExitCode {
    crate::fail("client", error)
}

/// What `client.toml` gives a run: the settings of the screen, and the longest wait for a
/// server to answer.
struct FileSettings {
    settings: Settings,
    wait: Duration,
}

/// The settings of `client.toml` in `dir`, or the error to print: the file, then the field or
/// the key of a bad binding.
fn settings(dir: Option<&Path>) -> Result<FileSettings, String> {
    let file: ClientConfig = crate::config_file::read(dir, CLIENT_FILE, Hidden::Tables(&[]))
        .map_err(|error| error.to_string())?;
    let wait = connect_wait(file.server.connect_wait_seconds)
        .map_err(|reason| format!("{CLIENT_FILE}: server.connect_wait_seconds: {reason}"))?;
    let settings = file
        .settings()
        .map_err(|error| format!("{CLIENT_FILE}: keys.{}: {error}", error.key))?;
    Ok(FileSettings { settings, wait })
}

/// The wait of `seconds`, or the reason that it is no wait. A value under 1 ns is no wait.
fn connect_wait(seconds: f64) -> Result<Duration, String> {
    Some(seconds)
        .filter(|seconds| *seconds >= 1e-9)
        .and_then(|seconds| Duration::try_from_secs_f64(seconds).ok())
        .ok_or_else(|| format!("{seconds} is not a finite number of seconds of 1 ns or more"))
}

/// The wait as the messages print it, such as `5 s` or `0.2 s`.
fn shown(wait: Duration) -> String {
    format!("{} s", wait.as_secs_f64())
}

/// What answered on the socket before the first list.
enum Found {
    /// Nothing listens: the socket path has no file, or the connection is refused.
    Nothing,
    /// A connection opened, so a server listens.
    Listener,
}

/// Whether anything listens on `socket`, or the error of a connection that neither opened nor
/// found that nothing listens.
async fn listener(socket: &Path) -> io::Result<Found> {
    match UnixStream::connect(socket).await {
        Ok(_) => Ok(Found::Listener),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
            ) =>
        {
            Ok(Found::Nothing)
        }
        Err(error) => Err(error),
    }
}

/// Runs the terminal client. Nothing runs before the check of the terminal, so a client
/// with no terminal creates no directory and no file. It reads `client.toml` before the screen
/// opens, and a bad file stops it there. If nothing listens on the socket, the client starts a
/// server in the background and attaches to it. Any other failure of the first list stops it.
/// The first list, and the wait for a started server, each wait at most the time of
/// `[server] connect_wait_seconds`.
pub fn run(args: &ClientArgs) -> ExitCode {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return fail(&"stdin and stdout must be a terminal");
    }
    args.config.warn_missing("client");
    let config = args.config.dir_from_env();
    let FileSettings { settings, wait } = match settings(config.as_deref()) {
        Ok(file) => file,
        Err(error) => return fail(&error),
    };
    let named = args.config.named();
    let socket = match args.socket.private_socket() {
        Ok(socket) => socket,
        Err(error) => return fail(&error),
    };
    let dir = socket_dir(&socket);
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => return fail(&error),
    };
    runtime.block_on(async {
        let client = PortalClient::unix(&socket);
        let first = async {
            match listener(&socket).await {
                Ok(Found::Nothing) => start_and_wait(&client, &socket, dir, named, wait).await,
                Ok(Found::Listener) => match tokio::time::timeout(wait, client.list_worlds()).await
                {
                    Ok(Ok(_)) => Ok(()),
                    Ok(Err(error)) => Err(format!("{}: {error}", socket.display())),
                    Err(_) => Err(format!(
                        "the server on {} did not answer within {}",
                        socket.display(),
                        shown(wait)
                    )),
                },
                Err(error) => Err(format!("{}: {error}", socket.display())),
            }
        };
        if let Err(error) = first.await {
            return fail(&error);
        }
        match mu_tui::run(client, settings).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => fail(&error),
        }
    })
}

/// Starts a server for `socket`, with `--config` only for a directory that the user named, and
/// waits up to `wait` until it answers. The error to print, if it does not start or answer.
async fn start_and_wait(
    client: &PortalClient,
    socket: &Path,
    dir: &Path,
    named: Option<&Path>,
    wait: Duration,
) -> Result<(), String> {
    let log = dir.join(SERVER_LOG);
    start_server(socket, &log, named).map_err(|error| format!("{}: {error}", log.display()))?;
    if answers(client, wait).await {
        Ok(())
    } else {
        Err(format!(
            "no server answered on {} within {}; read {}",
            socket.display(),
            shown(wait),
            log.display()
        ))
    }
}

/// Starts `mu server` for `socket` in its own process group, with stdout and stderr
/// appended to `log`, and `--config <config>` when there is a directory. With no directory, the
/// server resolves the default from the environment that it inherits.
fn start_server(socket: &Path, log: &Path, config: Option<&Path>) -> io::Result<()> {
    let log: File = OpenOptions::new().create(true).append(true).open(log)?;
    let mut command = Command::new(std::env::current_exe()?);
    command.arg("server").arg("--socket").arg(socket);
    if let Some(config) = config {
        // One argument, so a path that starts with `-` stays a value.
        let mut flag = std::ffi::OsString::from("--config=");
        flag.push(config);
        command.arg(flag);
    }
    command
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;
    Ok(())
}

/// Tries `ListWorlds` with the backoff of the resume, for up to `wait` in all.
async fn answers(client: &PortalClient, wait: Duration) -> bool {
    let tries = async {
        let mut backoff = mu_client::backoff();
        while client.list_worlds().await.is_err() {
            let step = backoff.next().unwrap_or(wait);
            tokio::time::sleep(step).await;
        }
    };
    tokio::time::timeout(wait, tries).await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_binding_error_starts_with_the_file_and_the_key() {
        let dir = tempfile::tempdir().unwrap();
        for (key, entry) in [("ctrl+N", "quit"), ("alt+n", "fly"), ("alt+f", "focus")] {
            std::fs::write(
                dir.path().join(CLIENT_FILE),
                format!("[keys]\n\"{key}\" = \"{entry}\"\n"),
            )
            .unwrap();
            let Err(error) = settings(Some(dir.path())) else {
                panic!("{key} is a good binding");
            };
            assert!(
                error.starts_with(&format!("client.toml: keys.{key}: ")),
                "{error}"
            );
        }
    }

    #[test]
    fn a_wait_that_is_no_time_is_refused() {
        for seconds in [0.0, -1.0, 1e-10, 0.9e-9, f64::NAN, f64::INFINITY, f64::MAX] {
            assert!(connect_wait(seconds).is_err(), "{seconds}");
        }
        assert_eq!(connect_wait(0.2), Ok(Duration::from_millis(200)));
        assert_eq!(connect_wait(1e-9), Ok(Duration::from_nanos(1)));
    }

    #[test]
    fn the_wait_is_five_seconds_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let Ok(file) = settings(Some(dir.path())) else {
            panic!("no file gives the defaults");
        };
        assert_eq!(file.wait, Duration::from_secs(5));
    }
}
