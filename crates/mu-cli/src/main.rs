use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    mu_cli::run(mu_cli::Cli::parse())
}
