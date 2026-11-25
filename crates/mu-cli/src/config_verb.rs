//! The verb `mu config`: the configuration of one client, and the reload of the directory, from
//! a running server. It never starts a server.

use std::fmt::Write as _;
use std::process::ExitCode;

use clap::{Args, Subcommand};
use mu_config::{
    FIELD_PATHS, KEYS_TABLE, Kind, Os, REPORT, STORE, WORLD_LAYOUTS_TABLE, filter, keyed_path,
};
use mu_proto as proto;

use crate::bootstrap::BootstrapArgs;
use crate::remote::call;

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
    let list = |names: &[String]| {
        toml::Value::Array(names.iter().cloned().map(toml::Value::String).collect())
    };
    match path {
        "echo" => Some(toml::Value::String(reply.echo.clone())),
        "layout" => Some(toml::Value::String(reply.layout_name.clone())),
        "tabs.label" => Some(toml::Value::String(reply.tab_label.clone())),
        "statusline.left" => Some(list(&reply.statusline_left)),
        "statusline.right" => Some(list(&reply.statusline_right)),
        _ => None,
    }
}

/// The key `key` as a part of a dotted TOML key: bare if TOML allows it, and quoted if not.
fn key_part(key: &str) -> String {
    toml_edit::Key::new(key).display_repr().into_owned()
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
    // A dotted key is TOML only before the first table header, so these lines come before
    // `[keys]`.
    for entry in &reply.world_layouts {
        let name = toml::Value::String(entry.name.clone());
        let _ = writeln!(
            text,
            "{WORLD_LAYOUTS_TABLE}.{} = {name} # {}",
            key_part(&entry.key),
            source(&keyed_path(WORLD_LAYOUTS_TABLE, &entry.key))
        );
    }
    let _ = writeln!(text, "# the lines of the layout {}:", reply.layout_name);
    for line in mu_config::layout::numbered(&reply.layout) {
        let _ = writeln!(text, "#   {line}");
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
            source(&keyed_path(KEYS_TABLE, &binding.key))
        );
    }
    filter(&text, &REPORT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_layouts_print_before_the_keys_header() {
        let reply = proto::GetClientConfigResponse {
            world_layouts: vec![
                proto::NamedLayout {
                    key: "alpha/aria".to_owned(),
                    name: "focus".to_owned(),
                    ..Default::default()
                },
                proto::NamedLayout {
                    key: "alpha".to_owned(),
                    name: "status".to_owned(),
                    ..Default::default()
                },
            ],
            bindings: vec![proto::Binding {
                key: "alt+n".to_owned(),
                entry: "focus-next".to_owned(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let text = render(&reply);
        let world = text
            .find("world-layouts.\"alpha/aria\" = \"focus\" # default")
            .expect("the world entry");
        let game = text
            .find("world-layouts.alpha = \"status\" # default")
            .expect("the game entry");
        let keys = text.find("[keys]").expect("the header");
        assert!(world < keys && game < keys, "{text}");
        assert!(text.parse::<toml::Table>().is_ok(), "{text}");
    }
}
