//! The verb `mu layout`: the layouts of a running server, and the changes of them. It never
//! starts a server.

use std::fmt::Write as _;
use std::process::ExitCode;

use clap::{Args, Subcommand};
use mu_config::layout::{Layout, REWRITTEN, numbered, preview};
use mu_config::{STORE, filter};
use mu_proto as proto;
use proto::__buffa::oneof::edit_layout_request::Change as EditChange;

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
    /// Set the layout of this device, or of one world or one game on this device
    Use {
        /// The name of the layout
        name: String,
        #[command(flatten)]
        target: Target,
        #[command(flatten)]
        bootstrap: BootstrapArgs,
    },
    /// Remove the layout of one world or one game on this device
    Forget {
        #[command(flatten)]
        target: Target,
        #[command(flatten)]
        bootstrap: BootstrapArgs,
    },
    /// Put a block on the screen of a layout, with a sentence such as "tabs at top"
    Place {
        /// The name of the layout
        layout: String,
        /// The sentence
        sentence: String,
        #[command(flatten)]
        edit: EditArgs,
    },
    /// Take a block off the screen of a layout
    Hide {
        /// The name of the layout
        layout: String,
        /// The id of the block
        id: String,
        #[command(flatten)]
        edit: EditArgs,
    },
    /// Set the size of a block of a layout, such as 30%, 12 or "max 3"
    Size {
        /// The name of the layout
        layout: String,
        /// The id of the block
        id: String,
        /// The size
        size: String,
        #[command(flatten)]
        edit: EditArgs,
    },
    /// Put a line into the plan of a layout
    Insert {
        /// The name of the layout
        layout: String,
        /// The number of the new line, from 1
        number: u32,
        /// The sentence
        sentence: String,
        #[command(flatten)]
        edit: EditArgs,
    },
    /// Replace one line of the plan of a layout
    Edit {
        /// The name of the layout
        layout: String,
        /// The number of the line, or the id of its block
        line: String,
        /// The new sentence
        sentence: String,
        #[command(flatten)]
        edit: EditArgs,
    },
    /// Remove one line of the plan of a layout
    Delete {
        /// The name of the layout
        layout: String,
        /// The number of the line, or the id of its block
        line: String,
        #[command(flatten)]
        edit: EditArgs,
    },
    /// Print each block of a layout with its rectangle and what it shows
    Describe {
        /// The name of the layout
        name: String,
        /// The columns of the screen
        #[arg(long, default_value_t = 80)]
        width: u32,
        /// The rows of the screen
        #[arg(long, default_value_t = 24)]
        height: u32,
        /// Print one JSON document
        #[arg(long)]
        json: bool,
        #[command(flatten)]
        bootstrap: BootstrapArgs,
    },
}

/// The world or the game of a layout choice.
#[derive(Clone, Debug, Args)]
struct Target {
    /// A world id, for the layout of that world
    #[arg(long, conflicts_with = "game")]
    world: Option<String>,
    /// The name of a game, for the layout of each world of that game
    #[arg(long)]
    game: Option<String>,
}

impl Target {
    fn key(&self) -> Option<&str> {
        self.world.as_deref().or(self.game.as_deref())
    }
}

/// The arguments that each change of a layout takes.
#[derive(Clone, Debug, Args)]
struct EditArgs {
    /// A world id. A change of a built-in layout then sets the layout of that world
    #[arg(long)]
    world: Option<String>,
    #[command(flatten)]
    bootstrap: BootstrapArgs,
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
        LayoutCommand::Use {
            name,
            target,
            bootstrap,
        } => choose(bootstrap, name, target.key()),
        LayoutCommand::Forget { target, bootstrap } => match target.key() {
            Some(key) => choose(bootstrap, "", Some(key)),
            None => Err("give --world or --game".to_owned()),
        },
        LayoutCommand::Place {
            layout,
            sentence,
            edit,
        } => change(layout, edit, EditChange::Place(sentence.clone())),
        LayoutCommand::Hide { layout, id, edit } => {
            change(layout, edit, EditChange::Hide(id.clone()))
        }
        LayoutCommand::Size {
            layout,
            id,
            size,
            edit,
        } => change(
            layout,
            edit,
            EditChange::Size(Box::new(proto::SizeChange {
                id: id.clone(),
                size: size.clone(),
                ..Default::default()
            })),
        ),
        LayoutCommand::Insert {
            layout,
            number,
            sentence,
            edit,
        } => change(
            layout,
            edit,
            EditChange::Insert(Box::new(proto::InsertLine {
                number: *number,
                line: sentence.clone(),
                ..Default::default()
            })),
        ),
        LayoutCommand::Edit {
            layout,
            line,
            sentence,
            edit,
        } => change(
            layout,
            edit,
            EditChange::Replace(Box::new(proto::ReplaceLine {
                line: line.clone(),
                sentence: sentence.clone(),
                ..Default::default()
            })),
        ),
        LayoutCommand::Delete { layout, line, edit } => {
            change(layout, edit, EditChange::Delete(line.clone()))
        }
        LayoutCommand::Describe {
            name,
            width,
            height,
            json,
            bootstrap,
        } => describe(bootstrap, name, *width, *height, *json),
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
    let lines: Vec<_> = layout
        .lines
        .iter()
        .map(|line| filter(line, &STORE))
        .collect();
    for line in numbered(&lines) {
        let _ = writeln!(text, "{line}");
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
    let file = mu_server::layout_path(name);
    match reply.notices.iter().find(|notice| notice.path == file) {
        Some(notice) => {
            crate::warn(VERB, &notice.text);
            Ok(ExitCode::from(1))
        }
        None => Err(format!("no layout is named {}", filter(name, &STORE))),
    }
}

fn choose(bootstrap: &BootstrapArgs, name: &str, key: Option<&str>) -> Result<ExitCode, String> {
    let server = bootstrap.server()?;
    let device = bootstrap.device()?;
    let (name, key) = (name.to_owned(), key.map(str::to_owned));
    call(&server, |client| async move {
        client
            .use_layout(device.as_str(), &name, key.as_deref())
            .await
    })?;
    Ok(ExitCode::SUCCESS)
}

fn change(layout: &str, edit: &EditArgs, change: EditChange) -> Result<ExitCode, String> {
    let server = edit.bootstrap.server()?;
    let device = edit.bootstrap.device()?;
    let request = proto::EditLayoutRequest {
        name: layout.to_owned(),
        device: device.as_str().to_owned(),
        world: edit.world.clone().unwrap_or_default(),
        change: Some(change),
        ..Default::default()
    };
    let reply = call(&server, |client| async move {
        client.edit_layout(request).await
    })?;
    if reply.rewritten {
        crate::warn(VERB, &REWRITTEN);
    }
    let saved = proto::NamedLayout {
        name: reply.name.clone(),
        lines: reply.lines,
        ..Default::default()
    };
    let mut text = String::new();
    let _ = writeln!(text, "{}", filter(&reply.name, &STORE));
    text.push_str(&described(&saved));
    Ok(crate::print(VERB, &text))
}

fn describe(
    bootstrap: &BootstrapArgs,
    name: &str,
    width: u32,
    height: u32,
    json: bool,
) -> Result<ExitCode, String> {
    let server = bootstrap.server()?;
    let name = name.to_owned();
    let reply = call(&server, |client| async move {
        client.describe_layout(&name, width, height).await
    })?;
    let text = if json {
        let blocks: Vec<serde_json::Value> = reply
            .blocks
            .iter()
            .map(|block| {
                serde_json::json!({
                    "id": block.id,
                    "name": block.name,
                    "line": block.line,
                    "x": block.x,
                    "y": block.y,
                    "width": block.width,
                    "height": block.height,
                    "front": block.front,
                    "data_source": block.data_source,
                })
            })
            .collect();
        let mut text = serde_json::to_string_pretty(&blocks).map_err(|error| error.to_string())?;
        text.push('\n');
        text
    } else {
        reply.blocks.iter().fold(String::new(), |mut text, block| {
            let _ = writeln!(
                text,
                "{}. {} ({}): column {}, row {}, {} by {}{}: {}",
                block.line,
                filter(&block.id, &STORE),
                block.name,
                block.x + 1,
                block.y + 1,
                block.width,
                block.height,
                if block.front { "" } else { ", behind" },
                block.data_source,
            );
            text
        })
    };
    Ok(crate::print(VERB, &text))
}
