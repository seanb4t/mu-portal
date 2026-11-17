//! Command grammar of the `mu` binary.

pub mod client;
pub mod config;
pub mod env_path;
pub mod server;
pub mod socket;
pub mod world;

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

/// Writes `mu <verb>: warning: <message>` to stderr, in one write, as [`fail`] does.
pub(crate) fn warn(verb: &str, message: &dyn std::fmt::Display) {
    // A closed stderr leaves nothing to report to.
    let _ = std::io::stderr().write_all(format!("mu {verb}: warning: {message}\n").as_bytes());
}

/// Writes `text` to stdout. A closed stdout is not an error of the verb, as it is not for
/// `--version`, but any other failed write fails the verb.
pub(crate) fn print(verb: &str, text: &str) -> ExitCode {
    match std::io::stdout().write_all(text.as_bytes()) {
        Err(error) if error.kind() != std::io::ErrorKind::BrokenPipe => fail(verb, &error),
        _ => ExitCode::SUCCESS,
    }
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
    /// List and show the configured worlds
    World(world::WorldArgs),
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
        Some(Verb::World(args)) => world::run(&args),
        Some(Verb::Version) => print("version", &Cli::command().render_long_version()),
    }
}
