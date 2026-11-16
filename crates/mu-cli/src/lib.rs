//! Command grammar of the `mu` binary.

pub mod client;
pub mod server;
pub mod socket;

use std::io::Write;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};

pub use client::ClientArgs;

/// Writes `mu <verb>: <error>` to stderr and returns the status 1. The line goes out in one
/// write, so servers that share a log append whole lines.
pub(crate) fn fail(verb: &str, error: &dyn std::fmt::Display) -> ExitCode {
    // A closed stderr leaves nothing to report to.
    let _ = std::io::stderr().write_all(format!("mu {verb}: {error}\n").as_bytes());
    ExitCode::from(1)
}
pub use server::ServerArgs;

/// One server and many clients for MU* text games.
#[derive(Debug, Parser)]
#[command(
    name = "mu",
    version,
    disable_help_subcommand = true,
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    /// The verb to run. `None` means the client.
    #[command(subcommand)]
    pub verb: Option<Verb>,

    /// Arguments of the client, for a command line with no verb.
    #[command(flatten)]
    pub client: ClientArgs,
}

/// The verbs of the `mu` binary.
#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Start the terminal client (the default verb)
    Client(ClientArgs),
    /// Run the server
    Server(ServerArgs),
    /// Print the name and the version
    Version,
}

/// Names of the verbs, read from the command definition.
#[must_use]
pub fn verb_names() -> Vec<String> {
    Cli::command()
        .get_subcommands()
        .map(|verb| verb.get_name().to_owned())
        .collect()
}

/// Runs the parsed command and returns the exit status of the process.
pub fn run(cli: Cli) -> ExitCode {
    match cli.verb {
        None => client::run(&cli.client),
        Some(Verb::Client(args)) => client::run(&args),
        Some(Verb::Server(args)) => server::run(&args),
        Some(Verb::Version) => {
            // A closed stdout is not an error of the verb, as it is not for `--version`.
            let _ = write!(
                std::io::stdout(),
                "{}",
                Cli::command().render_long_version()
            );
            ExitCode::SUCCESS
        }
    }
}
