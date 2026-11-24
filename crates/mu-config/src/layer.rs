//! The fields of a client layer, and the check of its `[keys]` table.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entries::EntryName;
use crate::key::KeyName;

/// How the client shows the echo line of a send.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
        [Self::Show, Self::Dim, Self::Hide]
            .into_iter()
            .find(|echo| echo.name() == name)
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
    /// Each key name and the name of the entry that it runs, or `none`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<BTreeMap<String, String>>,
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
