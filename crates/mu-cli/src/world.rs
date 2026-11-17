//! The verb `mu world`: the configured worlds, read from the files with no server.

use std::fmt::Write as _;
use std::io::Write as _;
use std::process::ExitCode;

use clap::{Args, Subcommand};
use mu_core::ResolvedWorld;
use mu_core::world::{Entry, LayerOrigin};

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
    let worlds = match config.worlds() {
        Ok(worlds) => worlds,
        Err(error) => return fail(&error),
    };
    let text = match id {
        None => worlds.iter().fold(String::new(), |mut text, world| {
            let _ = writeln!(text, "{}", world.id);
            text
        }),
        Some(id) => match worlds.iter().find(|world| world.id.as_str() == id) {
            Some(world) => render(world),
            None => return fail(&format!("{id}: no configured world has this id")),
        },
    };
    // A closed stdout is not an error of the verb.
    let _ = std::io::stdout().write_all(text.as_bytes());
    ExitCode::SUCCESS
}

/// The world as TOML. Each field ends with a comment that names its source, and each rule
/// and each script has a comment line with its source above it.
#[must_use]
pub fn render(world: &ResolvedWorld) -> String {
    let source = |path: &str| {
        world
            .sources
            .get(path)
            .map_or_else(|| LayerOrigin::Default.to_string(), ToString::to_string)
    };
    let mut text = String::new();
    let field = |text: &mut String, key: &str, path: &str, value: toml::Value| {
        let _ = writeln!(text, "{key} = {value} # {}", source(path));
    };
    field(&mut text, "host", "host", world.host.clone().into());
    field(&mut text, "port", "port", i64::from(world.port).into());
    field(&mut text, "tls", "tls", world.tls.into());
    let charset = world.charset.name().to_ascii_lowercase();
    field(&mut text, "charset", "charset", charset.into());
    field(
        &mut text,
        "columns",
        "columns",
        i64::from(world.columns).into(),
    );
    field(&mut text, "rows", "rows", i64::from(world.rows).into());
    if let Some(login) = &world.login {
        text.push_str("\n[login]\n");
        field(
            &mut text,
            "template",
            "login.template",
            login.template.clone().into(),
        );
        field(&mut text, "name", "login.name", login.name.clone().into());
        field(
            &mut text,
            "secret",
            "login.secret",
            login.secret.to_string().into(),
        );
    }
    if let Some(keepalive) = &world.keepalive {
        text.push_str("\n[keepalive]\n");
        field(
            &mut text,
            "command",
            "keepalive.command",
            keepalive.command.clone().into(),
        );
        let seconds = i64::try_from(keepalive.period.as_secs()).unwrap_or(i64::MAX);
        field(&mut text, "seconds", "keepalive.seconds", seconds.into());
    }
    for (array, entries) in [("rules", &world.rules), ("scripts", &world.scripts)] {
        for entry in entries {
            text.push_str(&render_entry(array, entry));
        }
    }
    text
}

/// One entry as its own document, `{ <array> = [entry] }`, under a comment with its source.
fn render_entry(array: &str, entry: &Entry) -> String {
    let mut document = toml::Table::new();
    document.insert(
        array.to_owned(),
        toml::Value::Array(vec![toml::Value::Table(entry.table.clone())]),
    );
    let body = toml::to_string(&document).unwrap_or_default();
    format!("\n# {}\n{body}", entry.origin)
}
