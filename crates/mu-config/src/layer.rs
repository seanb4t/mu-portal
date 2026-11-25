//! The fields of a client layer, and the check of its `[keys]` table.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entries::EntryName;
use crate::key::KeyName;
use crate::label::{FieldError, LABEL_DEFAULT, Label, Segment};
use crate::layout::DEFAULT_LAYOUT;
use crate::world_id::WorldKey;

/// How the client shows the echo line of a send.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "&'static str", try_from = "String")]
pub enum Echo {
    /// With no dim attribute.
    Show,
    /// With the dim attribute.
    #[default]
    Dim,
    /// Not at all.
    Hide,
}

impl Echo {
    /// Every value.
    pub const ALL: [Self; 3] = [Self::Show, Self::Dim, Self::Hide];

    /// The name of the value, as a layer writes it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Show => "show",
            Self::Dim => "dim",
            Self::Hide => "hide",
        }
    }

    /// The value named `name`.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        crate::names::by_name(&Self::ALL, Self::name, name)
    }
}

impl From<Echo> for &'static str {
    fn from(echo: Echo) -> Self {
        echo.name()
    }
}

impl TryFrom<String> for Echo {
    type Error = String;

    fn try_from(name: String) -> Result<Self, String> {
        Self::named(&name).ok_or_else(|| {
            format!(
                "{name:?} is no echo value: the values are {}",
                crate::listed(&Self::ALL.map(Self::name))
            )
        })
    }
}

/// The fields of one client layer. Each is optional, and an unset field serializes as nothing,
/// so a later layer without it keeps the value of an earlier layer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientLayer {
    /// How the client shows the echo line of a send.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub echo: Option<Echo>,
    /// The name of the layout of the screen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    /// The table `[tabs]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tabs: Option<Tabs>,
    /// The table `[statusline]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statusline: Option<Statusline>,
    /// Each key name and the name of the entry that it runs, or `none`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<BTreeMap<String, String>>,
    /// Each world id or game name, and the name of the layout of those worlds.
    #[serde(
        default,
        rename = "world-layouts",
        skip_serializing_if = "Option::is_none"
    )]
    pub world_layouts: Option<BTreeMap<String, String>>,
}

/// The table `[tabs]` of a client layer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tabs {
    /// The label of a world tab.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// The table `[statusline]` of a client layer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Statusline {
    /// The names of the segments from the left edge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<Vec<String>>,
    /// The names of the segments to the right edge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<Vec<String>>,
}

impl ClientLayer {
    /// The built-in values: each field of [`FIELD_PATHS`] set, and no binding.
    #[must_use]
    pub fn defaults() -> Self {
        Self {
            echo: Some(Echo::default()),
            layout: Some(DEFAULT_LAYOUT.to_owned()),
            tabs: Some(Tabs {
                label: Some(LABEL_DEFAULT.to_owned()),
            }),
            statusline: Some(Statusline {
                left: Some(names(&Segment::LEFT_DEFAULT)),
                right: Some(names(&Segment::RIGHT_DEFAULT)),
            }),
            keys: None,
            world_layouts: None,
        }
    }
}

fn names(segments: &[Segment]) -> Vec<String> {
    segments.iter().map(|each| each.name().to_owned()).collect()
}

/// The path of each field of a client layer outside its keyed tables, in the order of
/// `mu config show`. A binding of `[keys]` has the path `keys.<key>`, and an entry of
/// `[world-layouts]` has the path `world-layouts.<key>`.
pub const FIELD_PATHS: &[&str] = &[
    "echo",
    "layout",
    "tabs.label",
    "statusline.left",
    "statusline.right",
];

/// The name of the table of the bindings.
pub const KEYS_TABLE: &str = "keys";
/// The name of the table of the layout of each world.
pub const WORLD_LAYOUTS_TABLE: &str = "world-layouts";

/// The path of the entry `key` of the keyed table `table`.
#[must_use]
pub fn keyed_path(table: &str, key: &str) -> String {
    format!("{table}.{key}")
}

/// Checks the fields of a layer other than `[keys]`: the label, the two lists of segments, and
/// each key of `[world-layouts]`.
///
/// # Errors
///
/// The path of the first bad field, and what is wrong with it.
pub fn check_fields(layer: &ClientLayer) -> Result<(), (String, FieldError)> {
    if let Some(label) = layer.tabs.as_ref().and_then(|tabs| tabs.label.as_ref()) {
        Label::parse(label).map_err(|error| ("tabs.label".to_owned(), error))?;
    }
    if let Some(statusline) = &layer.statusline {
        for (path, list) in [
            ("statusline.left", &statusline.left),
            ("statusline.right", &statusline.right),
        ] {
            if let Some(list) = list {
                Segment::check(list).map_err(|error| (path.to_owned(), error))?;
            }
        }
    }
    for key in layer.world_layouts.iter().flat_map(BTreeMap::keys) {
        key.parse::<WorldKey>().map_err(|error| {
            (
                keyed_path(WORLD_LAYOUTS_TABLE, key),
                FieldError {
                    reason: error.to_string(),
                },
            )
        })?;
    }
    Ok(())
}

/// The value of a binding that makes the key do nothing.
pub const NONE: &str = "none";

/// What a key runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binding {
    /// The entry of this name.
    Entry(&'static str),
    /// Nothing. The key does not fall back to the same key with no modifier.
    None,
}

/// A bad binding of `[keys]`. Its text is the reason alone, so the reader of the file puts the
/// file and the key before it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyError {
    /// The key name of the binding, as the file writes it.
    pub key: String,
    /// What is wrong with it.
    pub reason: String,
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.reason)
    }
}

impl std::error::Error for KeyError {}

/// Checks each binding of a `[keys]` table, and gives each key with what it runs.
///
/// # Errors
///
/// A [`KeyError`] for the first bad key name, unknown entry, entry that needs an argument, or
/// two names of one key.
pub fn check_keys(keys: &BTreeMap<String, String>) -> Result<Vec<(KeyName, Binding)>, KeyError> {
    let mut bindings: Vec<(KeyName, Binding)> = Vec::new();
    let mut names: Vec<&str> = Vec::new();
    for (key, value) in keys {
        let error = |reason: String| KeyError {
            key: key.clone(),
            reason,
        };
        let name = KeyName::parse(key).map_err(|parse| error(parse.reason))?;
        if let Some(place) = bindings.iter().position(|(each, _)| *each == name) {
            return Err(error(format!(
                "{} and {key} name the same key",
                names[place]
            )));
        }
        let binding = if value == NONE {
            Binding::None
        } else {
            let entry = EntryName::named(value)
                .ok_or_else(|| error(format!("no entry is named {value}")))?;
            if entry.needs_arg {
                return Err(error(format!(
                    "{value} needs an argument, so a key cannot run it"
                )));
            }
            Binding::Entry(entry.name)
        };
        bindings.push((name, binding));
        names.push(key);
    }
    Ok(bindings)
}
