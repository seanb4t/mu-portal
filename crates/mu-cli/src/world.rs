//! The verb `mu world`: the configured worlds, read from the files with no server.

use std::fmt::Write as _;
use std::process::ExitCode;

use clap::{Args, Subcommand};
use mu_core::world::{Entry, FIELD_PATHS};
use mu_core::{Reconnect, ReconnectMode, ResolvedWorld};

use crate::config::ConfigArgs;

/// Arguments of `mu world`.
#[derive(Clone, Debug, Args)]
pub struct WorldArgs {
    #[command(subcommand)]
    command: WorldCommand,
}

/// The commands of `mu world`.
#[derive(Clone, Debug, Subcommand)]
enum WorldCommand {
    /// Print the id of each configured world
    List(ConfigArgs),
    /// Print a resolved world as TOML, with the source of each field
    Show {
        /// The id of the world, `<game>/<character>`
        id: String,
        #[command(flatten)]
        config: ConfigArgs,
    },
}

fn fail(error: &dyn std::fmt::Display) -> ExitCode {
    crate::fail("world", error)
}

/// Runs `mu world`.
pub fn run(args: &WorldArgs) -> ExitCode {
    let (config, id) = match &args.command {
        WorldCommand::List(config) => (config, None),
        WorldCommand::Show { id, config } => (config, Some(id)),
    };
    let worlds = match config.worlds("world") {
        Ok(worlds) => worlds,
        Err(error) => return fail(&error),
    };
    let text = match id {
        None => worlds.iter().fold(String::new(), |mut text, world| {
            let _ = writeln!(text, "{}", world.id);
            text
        }),
        Some(id) => match worlds.iter().find(|world| world.id.as_str() == id) {
            Some(world) => match render(world) {
                Ok(text) => text,
                Err(error) => return fail(&error),
            },
            None => return fail(&format!("{id}: no configured world has this id")),
        },
    };
    crate::print("world", &text)
}

/// The world as TOML. Each field ends with a comment that names its source, and each rule
/// and each script has a comment line with its source above it. The fields come in the order
/// of [`FIELD_PATHS`]. A field with no recorded source has the comment `unknown`.
///
/// # Errors
///
/// The id of a rule or a script that does not serialize as TOML, and why.
pub fn render(world: &ResolvedWorld) -> Result<String, String> {
    let mut text = String::new();
    let mut table = "";
    for path in FIELD_PATHS {
        let Some(value) = value_of(world, path) else {
            continue;
        };
        let (head, key) = path.split_once('.').unwrap_or(("", path));
        if head != table {
            let _ = write!(text, "\n[{head}]\n");
            table = head;
        }
        let source = world
            .sources
            .get(*path)
            .map_or_else(|| "unknown".to_owned(), ToString::to_string);
        let _ = writeln!(text, "{key} = {value} # {source}");
    }
    for (array, entries) in [("rules", &world.rules), ("scripts", &world.scripts)] {
        for entry in entries {
            text.push_str(&render_entry(array, entry)?);
        }
    }
    Ok(text)
}

/// The value of the field at `path`, or `None` when the world does not have its table.
fn value_of(world: &ResolvedWorld, path: &str) -> Option<toml::Value> {
    let login = world.login.as_ref();
    let keepalive = world.keepalive.as_ref();
    let backoff = match &world.reconnect {
        Reconnect::Backoff(backoff) => Some(backoff),
        Reconnect::Off => None,
    };
    Some(match path {
        "host" => world.host.clone().into(),
        "port" => i64::from(world.port).into(),
        "tls" => world.tls.into(),
        "autoconnect" => world.autoconnect.into(),
        "charset" => world.charset.name().to_ascii_lowercase().into(),
        "columns" => i64::from(world.columns).into(),
        "rows" => i64::from(world.rows).into(),
        "login.template" => login?.template().into(),
        "login.name" => login?.name().into(),
        "login.secret" => login?.secret().to_string().into(),
        "keepalive.mode" => toml::Value::try_from(world.keepalive_mode).ok()?,
        "keepalive.command" => keepalive?.command().into(),
        "keepalive.seconds" => i64::from(keepalive?.seconds()).into(),
        "reconnect.mode" => toml::Value::try_from(match world.reconnect {
            Reconnect::Backoff(_) => ReconnectMode::Backoff,
            Reconnect::Off => ReconnectMode::Off,
        })
        .ok()?,
        "reconnect.initial" => i64::from(backoff?.initial()).into(),
        "reconnect.max" => i64::from(backoff?.max()).into(),
        "reconnect.attempts" => i64::from(backoff?.attempts()).into(),
        _ => return None,
    })
}

/// One entry as its own document, `{ <array> = [entry] }`, under a comment with its source.
fn render_entry(array: &str, entry: &Entry) -> Result<String, String> {
    let mut document = toml::Table::new();
    document.insert(
        array.to_owned(),
        toml::Value::Array(vec![toml::Value::Table(entry.table.clone())]),
    );
    let body =
        toml::to_string(&document).map_err(|error| format!("{array} {}: {error}", entry.id))?;
    Ok(format!("\n# {}\n{body}", entry.origin))
}

#[cfg(test)]
mod tests {
    use mu_core::{Keepalive, Login};

    use super::*;

    #[test]
    fn each_field_path_has_a_value() {
        let world = ResolvedWorld {
            login: Some(
                Login::new(
                    "connect {login} {password}",
                    "Aria",
                    "env:ARIA_PW".parse().unwrap(),
                )
                .unwrap(),
            ),
            keepalive: Some(Keepalive::new("idle", 60).unwrap()),
            ..ResolvedWorld::new("alpha/aria".parse().unwrap())
        };
        for path in FIELD_PATHS {
            assert!(value_of(&world, path).is_some(), "{path}");
        }
    }
}
