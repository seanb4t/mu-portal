//! The verb `mu layout`: the layouts of a running server. It never starts a server.

use std::fmt::Write as _;
use std::process::ExitCode;

use clap::{Args, Subcommand};
use mu_config::layout::{Layout, preview};
use mu_config::{STORE, filter};
use mu_proto as proto;

use crate::bootstrap::BootstrapArgs;
use crate::remote::call;

/// Arguments of `mu layout`.
#[derive(Clone, Debug, Args)]
pub struct LayoutArgs {
    #[command(subcommand)]
    command: LayoutCommand,
}

/// The commands of `mu layout`.
#[derive(Clone, Debug, Subcommand)]
enum LayoutCommand {
    /// Print each layout with a picture of its panes
    List {
        #[command(flatten)]
        bootstrap: BootstrapArgs,
    },
    /// Print the numbered lines of one layout and a picture of its panes
    Show {
        /// The name of the layout
        name: String,
        #[command(flatten)]
        bootstrap: BootstrapArgs,
    },
}

const VERB: &str = "layout";

/// The columns and the rows of the picture of a layout.
const PICTURE: (u16, u16) = (40, 12);

fn fail(error: &dyn std::fmt::Display) -> ExitCode {
    crate::fail(VERB, error)
}

/// Runs `mu layout`.
pub fn run(args: &LayoutArgs) -> ExitCode {
    let result = match &args.command {
        LayoutCommand::List { bootstrap } => list(bootstrap),
        LayoutCommand::Show { name, bootstrap } => show(bootstrap, name),
    };
    result.unwrap_or_else(|error| fail(&error))
}

fn layouts(bootstrap: &BootstrapArgs) -> Result<proto::ListLayoutsResponse, String> {
    let server = bootstrap.server()?;
    call(&server, |client| async move { client.list_layouts().await })
}

/// The numbered lines of `layout`, then its picture. A layout that this binary cannot read
/// gives its lines and no picture.
#[must_use]
pub fn described(layout: &proto::NamedLayout) -> String {
    let mut text = String::new();
    for (index, line) in layout.lines.iter().enumerate() {
        let _ = writeln!(text, "{}. {}", index + 1, filter(line, &STORE));
    }
    if let Ok(checked) = Layout::parse(&layout.lines) {
        for row in preview(&checked, PICTURE.0, PICTURE.1) {
            let _ = writeln!(text, "{row}");
        }
    }
    text
}

fn list(bootstrap: &BootstrapArgs) -> Result<ExitCode, String> {
    let reply = layouts(bootstrap)?;
    for notice in &reply.notices {
        crate::warn(VERB, &notice.text);
    }
    let mut text = String::new();
    for layout in &reply.layouts {
        let mark = if layout.built_in { " (built-in)" } else { "" };
        let _ = writeln!(text, "{}{mark}", filter(&layout.name, &STORE));
        text.push_str(&described(layout));
        text.push('\n');
    }
    Ok(crate::print(VERB, &text))
}

fn show(bootstrap: &BootstrapArgs, name: &str) -> Result<ExitCode, String> {
    let reply = layouts(bootstrap)?;
    if let Some(layout) = reply.layouts.iter().find(|layout| layout.name == name) {
        return Ok(crate::print(VERB, &described(layout)));
    }
    let file = format!("clients/layouts/{name}.toml");
    match reply.notices.iter().find(|notice| notice.path == file) {
        Some(notice) => {
            crate::warn(VERB, &notice.text);
            Ok(ExitCode::from(1))
        }
        None => Err(format!("no layout is named {}", filter(name, &STORE))),
    }
}
