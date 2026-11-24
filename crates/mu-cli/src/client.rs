//! The verb `mu client`.

use std::cell::Cell;
use std::fs::{File, OpenOptions};
use std::io::{self, IsTerminal as _};
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

use clap::Args;
use mu_client::{ClientError, PortalClient};
use mu_config::{Kind, Os};
use tokio::net::UnixStream;

use crate::bootstrap::BootstrapArgs;
use crate::socket::{current_uid, private_dir, socket_dir};

/// The log of a server that a client started, in the socket directory.
pub const SERVER_LOG: &str = "server.log";

/// Arguments of the client. `mu` and `mu client` both accept them.
#[derive(Clone, Debug, Default, Args)]
pub struct ClientArgs {
    #[command(flatten)]
    bootstrap: BootstrapArgs,
}

fn fail(error: &dyn std::fmt::Display) -> ExitCode {
    crate::fail("client", error)
}

/// The longest wait for a server to answer, when nothing sets one.
pub(crate) const DEFAULT_WAIT: Duration = Duration::from_secs(5);

/// The wait of `seconds`, or the reason that it is no wait. A value under 1 ns is no wait.
pub(crate) fn connect_wait(seconds: f64) -> Result<Duration, String> {
    Some(seconds)
        .filter(|seconds| *seconds >= 1e-9)
        .and_then(|seconds| Duration::try_from_secs_f64(seconds).ok())
        .ok_or_else(|| format!("{seconds} is not a finite number of seconds of 1 ns or more"))
}

/// The wait as the messages print it, such as `5 s` or `0.2 s`.
pub(crate) fn shown(wait: Duration) -> String {
    format!("{} s", wait.as_secs_f64())
}

/// The answer of `call` to the server at `url`, or the error to print: the error of the call,
/// or a server that did not answer within `wait`.
pub(crate) async fn answered<T>(
    url: &str,
    wait: Duration,
    call: impl Future<Output = Result<T, ClientError>>,
) -> Result<T, String> {
    match tokio::time::timeout(wait, call).await {
        Ok(Ok(answer)) => Ok(answer),
        Ok(Err(error)) => Err(format!("{url}: {error}")),
        Err(_) => Err(format!(
            "the server on {url} did not answer within {}",
            shown(wait)
        )),
    }
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
/// with no terminal creates no directory and no file. If nothing listens on the socket, the
/// client starts a server in the background. It makes the first list of the worlds, waits for
/// the first store hash of `WatchConfig` and gets its client configuration, all within one wait
/// of `connect_wait`, before the screen opens.
pub fn run(args: &ClientArgs) -> ExitCode {
    let (server, device) = match args.bootstrap.server().and_then(|server| {
        let device = args.bootstrap.device()?;
        Ok((server, device))
    }) {
        Ok(values) => values,
        Err(error) => return fail(&error),
    };
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return fail(&"stdin and stdout must be a terminal");
    }
    let socket = server.socket;
    let dir = socket_dir(&socket);
    if let Err(error) = private_dir(dir, current_uid()) {
        return fail(&error);
    }
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => return fail(&error),
    };
    runtime.block_on(async {
        let client = PortalClient::unix(&socket);
        let name = socket.display().to_string();
        let log = dir.join(SERVER_LOG);
        let started = Cell::new(false);
        let failed = |error: ClientError| format!("{name}: {error}");
        let list = async {
            match listener(&socket).await {
                Ok(Found::Nothing) => {
                    start_server(&socket, &log)
                        .map_err(|error| format!("{}: {error}", log.display()))?;
                    started.set(true);
                    answers(&client).await;
                    Ok(())
                }
                Ok(Found::Listener) => client.list_worlds().await.map(drop).map_err(failed),
                Err(error) => Err(format!("{name}: {error}")),
            }
        };
        let hash = async {
            let mut hashes = client.watch_config().await.map_err(failed)?;
            hashes
                .next()
                .await
                .map_err(failed)?
                .ok_or_else(|| format!("{name}: the watch of the configuration ended"))
        };
        let config = async {
            client
                .client_config(Kind::Tui.name(), Os::current().name(), device.as_str())
                .await
                .map_err(failed)
        };
        let late = || {
            if started.get() {
                format!(
                    "no server answered on {name} within {}; read {}",
                    shown(server.wait),
                    log.display()
                )
            } else {
                format!(
                    "the server on {name} did not answer within {}",
                    shown(server.wait)
                )
            }
        };
        let config = match Box::pin(in_one_wait(server.wait, list, hash, config)).await {
            Some(Ok(config)) => config,
            Some(Err(error)) => return fail(&error),
            None => return fail(&late()),
        };
        match mu_tui::run(client, config).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => fail(&error),
        }
    })
}

/// The answer of `config`, after `list` and then `hash` succeed, or the first error. All three
/// steps run within one wait of `wait`, and `None` is a wait that ended first.
async fn in_one_wait<A, B, C>(
    wait: Duration,
    list: impl Future<Output = Result<A, String>>,
    hash: impl Future<Output = Result<B, String>>,
    config: impl Future<Output = Result<C, String>>,
) -> Option<Result<C, String>> {
    let steps = async {
        list.await?;
        hash.await?;
        config.await
    };
    tokio::time::timeout(wait, steps).await.ok()
}

/// Starts `mu server` for `socket` in its own process group, with stdout and stderr appended to
/// `log`. The server finds its directory from the environment that it inherits.
fn start_server(socket: &Path, log: &Path) -> io::Result<()> {
    let log: File = OpenOptions::new().create(true).append(true).open(log)?;
    let mut command = Command::new(std::env::current_exe()?);
    command.arg("server").arg("--socket").arg(socket);
    command
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;
    Ok(())
}

/// Tries `ListWorlds` with the backoff of the resume until it succeeds.
async fn answers(client: &PortalClient) {
    let mut backoff = mu_client::backoff();
    while client.list_worlds().await.is_err() {
        let step = backoff.next().unwrap_or(DEFAULT_WAIT);
        tokio::time::sleep(step).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A step that takes `seconds` of the paused clock and succeeds.
    async fn step(seconds: f64) -> Result<(), String> {
        tokio::time::sleep(Duration::from_secs_f64(seconds)).await;
        Ok(())
    }

    #[tokio::test(start_paused = true)]
    async fn one_wait_covers_the_three_steps() {
        let wait = Duration::from_secs(1);
        let within = in_one_wait(wait, step(0.3), step(0.3), step(0.3)).await;
        let late = in_one_wait(wait, step(0.4), step(0.4), step(0.4)).await;
        assert!(
            within == Some(Ok(())) && late.is_none(),
            "{within:?} {late:?}"
        );
    }
}
