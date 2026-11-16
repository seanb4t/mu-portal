//! The verb `mu client`.

use std::io::{IsTerminal as _, Write as _};
use std::process::ExitCode;

use clap::Args;

use crate::socket::SocketArgs;

/// Arguments of the client. `mu` and `mu client` both accept them.
#[derive(Clone, Debug, Default, Args)]
pub struct ClientArgs {
    #[command(flatten)]
    socket: SocketArgs,
}

impl ClientArgs {
    /// The socket arguments.
    #[must_use]
    pub fn socket(&self) -> &SocketArgs {
        &self.socket
    }
}

fn fail(error: &dyn std::fmt::Display) -> ExitCode {
    // A closed stderr leaves nothing to report to.
    let _ = writeln!(std::io::stderr(), "mu client: {error}");
    ExitCode::from(1)
}

/// Runs the terminal client. Nothing runs before the check of the terminal, so a client
/// with no terminal creates no directory and no file.
pub fn run(_args: &ClientArgs) -> ExitCode {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return fail(&"stdin and stdout must be a terminal");
    }
    fail(&"the terminal view is not built yet")
}
