//! The verb `mu config`: the configuration of one client, and the reload of the directory, from
//! a running server. It never starts a server.

use std::fmt::Write as _;
use std::process::ExitCode;

use clap::{Args, Subcommand};
use mu_client::PortalClient;
use mu_config::{Kind, Os};
use mu_proto as proto;

use crate::bootstrap::{BootstrapArgs, Server};
use crate::client::answered;

/// Arguments of `mu config`.
#[derive(Clone, Debug, Args)]
pub struct ConfigVerbArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

/// The commands of `mu config`.
#[derive(Clone, Debug, Subcommand)]
enum ConfigCommand {
    /// Print the configuration of one client as TOML, with the source of each field
    Show {
        #[command(flatten)]
        bootstrap: BootstrapArgs,
        /// The kind of the client
        #[arg(long, default_value = "tui")]
        kind: String,
        /// The OS of the client: macos or linux. The default is the OS of this machine
        #[arg(long)]
        os: Option<String>,
    },
    /// Ask the server to read its configuration directory again
    Reload {
        #[command(flatten)]
        bootstrap: BootstrapArgs,
    },
}

const VERB: &str = "config";

fn fail(error: &dyn std::fmt::Display) -> ExitCode {
    crate::fail(VERB, error)
}

/// Runs `mu config`.
pub fn run(args: &ConfigVerbArgs) -> ExitCode {
    let result = match &args.command {
        ConfigCommand::Show {
            bootstrap,
            kind,
            os,
        } => show(bootstrap, kind, os.as_deref()),
        ConfigCommand::Reload { bootstrap } => reload(bootstrap),
    };
    result.unwrap_or_else(|error| fail(&error))
}

/// Runs `call` with the client of `server` on a new runtime.
fn call<T, F: Future<Output = Result<T, mu_client::ClientError>>>(
    server: &Server,
    call: impl FnOnce(PortalClient) -> F,
) -> Result<T, String> {
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    runtime.block_on(async {
        let client = PortalClient::unix(&server.socket);
        answered(&server.url, server.wait, call(client)).await
    })
}

fn show(bootstrap: &BootstrapArgs, kind: &str, os: Option<&str>) -> Result<ExitCode, String> {
    let server = bootstrap.server()?;
    let device = bootstrap.device()?;
    let kind = Kind::parse(kind).map_err(|error| error.to_string())?;
    let os = os
        .map_or(Ok(Os::current()), Os::parse)
        .map_err(|error| error.to_string())?;
    let reply = call(&server, |client| async move {
        client
            .client_config(kind.name(), os.name(), device.as_str())
            .await
    })?;
    for notice in &reply.notices {
        crate::warn(VERB, &notice.text);
    }
    Ok(crate::print(VERB, &render(&reply)))
}

fn reload(bootstrap: &BootstrapArgs) -> Result<ExitCode, String> {
    let server = bootstrap.server()?;
    let reply = call(
        &server,
        |client| async move { client.reload_config().await },
    )?;
    for notice in &reply.notices {
        crate::warn(VERB, &notice.text);
    }
    let text = reply.restart.iter().fold(String::new(), |mut text, path| {
        let _ = writeln!(
            text,
            "{path}: the change applies at the restart of the server"
        );
        text
    });
    let printed = crate::print(VERB, &text);
    if reply.notices.is_empty() {
        Ok(printed)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// The configuration as TOML: the revision on the first line, then each field and each binding
/// with a comment that names its source.
#[must_use]
pub fn render(reply: &proto::GetClientConfigResponse) -> String {
    let source = |path: &str| {
        reply
            .sources
            .get(path)
            .map_or("default", String::as_str)
            .to_owned()
    };
    let mut text = format!("# revision {}\n", reply.revision);
    let echo = toml::Value::String(reply.echo.clone());
    let _ = writeln!(text, "echo = {echo} # {}", source("echo"));
    if !reply.bindings.is_empty() {
        text.push_str("\n[keys]\n");
    }
    for binding in &reply.bindings {
        let key = toml::Value::String(binding.key.clone());
        let entry = toml::Value::String(binding.entry.clone());
        let _ = writeln!(
            text,
            "{key} = {entry} # {}",
            source(&format!("keys.{}", binding.key))
        );
    }
    text
}
