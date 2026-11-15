//! Command grammar of the `mu` binary.

use std::io::Write;
use std::process::ExitCode;

use clap::{Args, CommandFactory, Parser, Subcommand};

/// One server and many clients for MU* text games.
#[derive(Debug, Parser)]
#[command(
    name = "mu",
    version,
    disable_help_subcommand = true,
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub verb: Option<Verb>,

    #[command(flatten)]
    pub client: ClientArgs,
}

/// Arguments of the client. `mu` and `mu client` both accept them.
#[derive(Debug, Default, Args)]
pub struct ClientArgs {}

#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Start the terminal client (the default verb)
    Client(ClientArgs),
    /// Run the server
    Server,
    /// Print the name and the version
    Version,
}

/// Names of the verbs, read from the command definition.
pub fn verb_names() -> Vec<String> {
    Cli::command()
        .get_subcommands()
        .map(|verb| verb.get_name().to_owned())
        .collect()
}

/// Runs the parsed command and returns the exit status of the process.
pub fn run(cli: Cli) -> ExitCode {
    match cli.verb {
        None => client(cli.client),
        Some(Verb::Client(args)) => client(args),
        Some(Verb::Server) => not_implemented("server"),
        Some(Verb::Version) => {
            print!("{}", Cli::command().render_long_version());
            ExitCode::SUCCESS
        }
    }
}

fn client(_args: ClientArgs) -> ExitCode {
    not_implemented("client")
}

/// Reports a verb that has no implementation. Opens no file.
fn not_implemented(verb: &str) -> ExitCode {
    // A closed stderr leaves nothing to report to, so the write error is dropped.
    let _ = writeln!(std::io::stderr(), "{verb}: not implemented");
    ExitCode::from(1)
}
