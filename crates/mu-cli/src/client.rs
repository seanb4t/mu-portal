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

use crate::config::ConfigArgs;
use crate::socket::{SocketArgs, socket_dir};

/// How long the client waits for a server that it started.
const START_LIMIT: Duration = Duration::from_secs(5);

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

/// The settings of `client.toml` in `dir`, or the error to print: the file, then the key of a
/// bad binding.
fn settings(dir: Option<&Path>) -> Result<Settings, String> {
    let file: ClientConfig = crate::config_file::read(dir, CLIENT_FILE, Hidden::Tables(&[]))
        .map_err(|error| error.to_string())?;
    file.settings()
        .map_err(|error| format!("{CLIENT_FILE}: keys.{}: {error}", error.key))
}

/// Runs the terminal client. Nothing runs before the check of the terminal, so a client
/// with no terminal creates no directory and no file. It reads `client.toml` before the screen
/// opens, and a bad file stops it there. With no server on the socket, the client starts one in
/// the background with the same configuration directory and attaches to it.
pub fn run(args: &ClientArgs) -> ExitCode {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return fail(&"stdin and stdout must be a terminal");
    }
    args.config.warn_missing("client");
    let config = args.config.dir_from_env();
    let settings = match settings(config.as_deref()) {
        Ok(settings) => settings,
        Err(error) => return fail(&error),
    };
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
        match tokio::time::timeout(START_LIMIT, client.list_worlds()).await {
            Ok(Ok(_)) => {}
            Ok(Err(_)) => {
                if let Some(error) = start_and_wait(&client, &socket, dir, config.as_deref()).await
                {
                    return error;
                }
            }
            Err(_) => {
                return fail(&format!(
                    "the server on {} did not answer within 5 seconds",
                    socket.display()
                ));
            }
        }
        match mu_tui::run(client, settings).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => fail(&error),
        }
    })
}

/// Starts a server for `socket` with the configuration directory `config`, and waits until it
/// answers. The failure to return, if it does not start or answer.
async fn start_and_wait(
    client: &PortalClient,
    socket: &Path,
    dir: &Path,
    config: Option<&Path>,
) -> Option<ExitCode> {
    let log = dir.join(SERVER_LOG);
    if let Err(error) = start_server(socket, &log, config) {
        return Some(fail(&format!("{}: {error}", log.display())));
    }
    if !answers(client).await {
        return Some(fail(&format!(
            "no server answered on {} within 5 seconds; read {}",
            socket.display(),
            log.display()
        )));
    }
    None
}

/// Starts `mu server` for `socket` in its own process group, with stdout and stderr
/// appended to `log`, and `--config <config>` when there is a directory.
fn start_server(socket: &Path, log: &Path, config: Option<&Path>) -> io::Result<()> {
    let log: File = OpenOptions::new().create(true).append(true).open(log)?;
    let mut command = Command::new(std::env::current_exe()?);
    command.arg("server").arg("--socket").arg(socket);
    if let Some(config) = config {
        command.arg("--config").arg(config);
    }
    command
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;
    Ok(())
}

/// Tries `ListWorlds` with the backoff of the resume, for up to 5 seconds in all.
async fn answers(client: &PortalClient) -> bool {
    let tries = async {
        let mut backoff = mu_client::backoff();
        while client.list_worlds().await.is_err() {
            let wait = backoff.next().unwrap_or(START_LIMIT);
            tokio::time::sleep(wait).await;
        }
    };
    tokio::time::timeout(START_LIMIT, tries).await.is_ok()
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
            let error = settings(Some(dir.path())).unwrap_err();
            assert!(
                error.starts_with(&format!("client.toml: keys.{key}: ")),
                "{error}"
            );
        }
    }
}
