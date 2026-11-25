//! Command grammar of the `mu` binary.

pub mod bootstrap;
pub mod client;
pub mod config;
pub mod config_file;
pub mod config_verb;
pub mod env_path;
pub mod layout_verb;
pub mod remote;
pub mod server;
pub mod server_config;
pub mod socket;
pub mod telemetry;
pub mod world;

use std::io::Write;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};

pub use client::ClientArgs;
pub use server::ServerArgs;

/// Writes `line` and a newline to stderr in one write, so servers that share a log append whole
/// lines. Each control character but a line break and a tab becomes U+FFFD, so a value from a
/// file cannot send an escape sequence to the terminal.
fn report(line: &std::fmt::Arguments<'_>) {
    // A closed stderr leaves nothing to report to.
    let _ = report_to(&mut std::io::stderr(), line);
}

fn report_to(out: &mut impl Write, line: &std::fmt::Arguments<'_>) -> std::io::Result<()> {
    let text = mu_config::filter(&line.to_string(), &mu_config::REPORT);
    out.write_all(format!("{text}\n").as_bytes())
}

/// Writes `mu <verb>: <error>` to stderr and returns the status 1.
pub(crate) fn fail(verb: &str, error: &dyn std::fmt::Display) -> ExitCode {
    report(&format_args!("mu {verb}: {error}"));
    ExitCode::from(1)
}

/// Writes `mu <verb>: warning: <message>` to stderr.
pub(crate) fn warn(verb: &str, message: &dyn std::fmt::Display) {
    report(&format_args!("mu {verb}: warning: {message}"));
}

/// Writes `text` to stdout. A closed stdout is not an error of the verb, as it is not for
/// `--version`, but any other failed write fails the verb.
pub(crate) fn print(verb: &str, text: &str) -> ExitCode {
    print_to(&mut std::io::stdout(), verb, text)
}

fn print_to(out: &mut impl Write, verb: &str, text: &str) -> ExitCode {
    match out.write_all(text.as_bytes()) {
        Err(error) if error.kind() != std::io::ErrorKind::BrokenPipe => fail(verb, &error),
        _ => ExitCode::SUCCESS,
    }
}

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
    /// Show the configuration of one client, or reload the directory, on a running server
    Config(config_verb::ConfigVerbArgs),
    /// List, show, choose and change the layouts of a running server
    Layout(layout_verb::LayoutArgs),
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
        Some(Verb::Config(args)) => config_verb::run(&args),
        Some(Verb::Layout(args)) => layout_verb::run(&args),
        Some(Verb::Server(args)) => server::run(&args),
        Some(Verb::World(args)) => world::run(&args),
        Some(Verb::Version) => print("version", &Cli::command().render_long_version()),
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, ErrorKind};

    use super::*;

    /// A writer whose every write fails with `kind`.
    struct Failing(ErrorKind);

    impl Write for Failing {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(self.0.into())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// A writer that keeps each buffer that one call of `write` gets.
    #[derive(Default)]
    struct Writes(Vec<Vec<u8>>);

    impl Write for Writes {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.push(buf.to_vec());
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn report_writes_the_line_in_one_write() {
        let mut out = Writes::default();
        let (verb, error) = ("server", "no such file");
        report_to(&mut out, &format_args!("mu {verb}: {error}")).unwrap();
        assert_eq!(out.0, [b"mu server: no such file\n".to_vec()]);
    }

    #[test]
    fn report_replaces_control_characters() {
        let mut out = Writes::default();
        let key = "a\x1b]0;title\x07b\rc\u{9b}d";
        report_to(&mut out, &format_args!("mu client: {key}\n\tnext")).unwrap();
        assert_eq!(
            String::from_utf8(out.0.concat()).unwrap(),
            "mu client: a\u{fffd}]0;title\u{fffd}b\u{fffd}c\u{fffd}d\n\tnext\n"
        );
    }

    #[test]
    fn print_ignores_only_a_broken_pipe() {
        for (kind, status) in [
            (ErrorKind::BrokenPipe, ExitCode::SUCCESS),
            (ErrorKind::StorageFull, ExitCode::from(1)),
            (ErrorKind::PermissionDenied, ExitCode::from(1)),
        ] {
            assert_eq!(
                print_to(&mut Failing(kind), "test", "text"),
                status,
                "{kind}"
            );
        }
        assert_eq!(print_to(&mut Vec::new(), "test", "text"), ExitCode::SUCCESS);
    }
}
