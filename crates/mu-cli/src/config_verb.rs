//! The verb `mu config`: the configuration of one client, and the reload of the directory, from
//! a running server. It never starts a server.

use std::fmt::Write as _;
use std::fs;
use std::process::ExitCode;

use clap::{Args, Subcommand};
use mu_client::PortalClient;
use mu_config::{FIELD_PATHS, Kind, Os, REPORT, STORE, filter};
use mu_proto as proto;

use crate::bootstrap::{BootstrapArgs, Server};
use crate::client::answered;
use crate::socket::{checked_dir, current_uid, socket_dir};

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
    let dir = socket_dir(&server.socket);
    if fs::symlink_metadata(dir).is_ok() {
        checked_dir(dir, current_uid()).map_err(|error| error.to_string())?;
    }
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
    let printed = crate::print(VERB, &restart_lines(&reply.restart));
    if reply.notices.is_empty() {
        Ok(printed)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// One line for each path of `restart`, with each control character written as U+FFFD.
#[must_use]
pub fn restart_lines(restart: &[String]) -> String {
    restart.iter().fold(String::new(), |mut text, path| {
        let _ = writeln!(
            text,
            "{}: the change applies at the restart of the server",
            filter(path, &STORE)
        );
        text
    })
}

/// The value of the field at `path` of [`FIELD_PATHS`] in `reply`, as TOML, or none for a path
/// that the reply does not carry.
#[must_use]
pub fn field_value(reply: &proto::GetClientConfigResponse, path: &str) -> Option<toml::Value> {
    match path {
        "echo" => Some(toml::Value::String(reply.echo.clone())),
        _ => None,
    }
}

/// The configuration as TOML: the revision on the first line, then each field and each binding
/// with a comment that names its source. Each control character other than the line break and
/// the tab is written as U+FFFD.
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
    for path in FIELD_PATHS {
        if let Some(value) = field_value(reply, path) {
            let _ = writeln!(text, "{path} = {value} # {}", source(path));
        }
    }
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
    filter(&text, &REPORT)
}
