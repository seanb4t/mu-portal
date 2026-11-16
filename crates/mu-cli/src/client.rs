//! The verb `mu client`.

use std::fs::{File, OpenOptions};
use std::io::{self, IsTerminal as _, Write as _};
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

use clap::Args;
use mu_client::PortalClient;

use crate::socket::{SocketArgs, current_uid, private_dir};

/// How long the client waits for a server that it started.
const START_LIMIT: Duration = Duration::from_secs(5);

/// The log of a server that a client started, in the socket directory.
pub const SERVER_LOG: &str = "server.log";

/// Arguments of the client. `mu` and `mu client` both accept them.
#[derive(Clone, Debug, Default, Args)]
pub struct ClientArgs {
    #[command(flatten)]
    socket: SocketArgs,
}

fn fail(error: &dyn std::fmt::Display) -> ExitCode {
    // A closed stderr leaves nothing to report to.
    let _ = writeln!(io::stderr(), "mu client: {error}");
    ExitCode::from(1)
}

/// Runs the terminal client. Nothing runs before the check of the terminal, so a client
/// with no terminal creates no directory and no file. With no server on the socket, the
/// client starts one in the background and attaches to it.
pub fn run(args: &ClientArgs) -> ExitCode {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return fail(&"stdin and stdout must be a terminal");
    }
    let socket = args.socket.resolve();
    let dir = socket.parent().unwrap_or_else(|| Path::new("/"));
    if let Err(error) = private_dir(dir, current_uid()) {
        return fail(&error);
    }
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => return fail(&error),
    };
    runtime.block_on(async {
        let client = PortalClient::unix(&socket);
        if client.list_worlds().await.is_err() {
            let log = dir.join(SERVER_LOG);
            if let Err(error) = start_server(&socket, &log) {
                return fail(&format!("{}: {error}", log.display()));
            }
            if !answers(&client).await {
                return fail(&format!(
                    "no server answered on {} within 5 seconds; read {}",
                    socket.display(),
                    log.display()
                ));
            }
        }
        match mu_tui::run(client).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => fail(&error),
        }
    })
}

/// Starts `mu server` for `socket` in its own process group, with stdout and stderr
/// appended to `log`.
fn start_server(socket: &Path, log: &Path) -> io::Result<()> {
    let log: File = OpenOptions::new().create(true).append(true).open(log)?;
    Command::new(std::env::current_exe()?)
        .arg("server")
        .arg("--socket")
        .arg(socket)
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
