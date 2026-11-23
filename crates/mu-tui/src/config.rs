//! `client.toml`, the configuration of the terminal client. `mu-cli` reads and parses the
//! file, and this module turns it into the [`Settings`] of a run.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::keymap::{Chord, Keymap};
use crate::registry::EntryId;

/// How the client shows the echo line of a send.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
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

/// The table `[server]` of `client.toml`: how `mu client` waits for a server.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerTable {
    /// The longest time, in seconds, that `mu client` waits for a server to answer. `mu-cli`
    /// refuses a value that is not finite or not above 0.
    #[serde(default = "ServerTable::default_wait")]
    pub connect_wait_seconds: f64,
}

impl ServerTable {
    const fn default_wait() -> f64 {
        5.0
    }
}

impl Default for ServerTable {
    fn default() -> Self {
        Self {
            connect_wait_seconds: Self::default_wait(),
        }
    }
}

/// The content of `client.toml`. A missing file is the default.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientConfig {
    /// How the client shows the echo line of a send.
    #[serde(default)]
    pub echo: Echo,
    /// Each key name and the name of the entry that it runs, or `none`.
    #[serde(default)]
    pub keys: BTreeMap<String, String>,
    /// How `mu client` waits for a server.
    #[serde(default)]
    pub server: ServerTable,
}

/// What a run of the client takes from its configuration.
#[derive(Clone, Debug, Default)]
pub struct Settings {
    /// The default keymap with the bindings of `[keys]`.
    pub keymap: Keymap,
    /// How the client shows the echo line of a send.
    pub echo: Echo,
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

impl std::fmt::Display for KeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.reason)
    }
}

impl std::error::Error for KeyError {}

/// The value of a binding that removes the key.
const NONE: &str = "none";

impl ClientConfig {
    /// The default keymap with each binding of `[keys]` in place of the default of its key.
    /// `none` binds the key to `ignore`.
    ///
    /// # Errors
    ///
    /// A [`KeyError`] for a bad key name, an unknown entry, an entry that needs an argument, or
    /// two names of one key.
    pub fn keymap(&self) -> Result<Keymap, KeyError> {
        let mut keymap = Keymap::default();
        let mut named: Vec<(Chord, &str)> = Vec::new();
        for (key, value) in &self.keys {
            let error = |reason: String| KeyError {
                key: key.clone(),
                reason,
            };
            let chord = Chord::parse(key).map_err(|parse| error(parse.reason))?;
            if let Some((_, other)) = named.iter().find(|(each, _)| *each == chord) {
                return Err(error(format!("{other} and {key} name the same key")));
            }
            named.push((chord, key));
            let entry = if value == NONE {
                EntryId::of("ignore")
            } else {
                EntryId::named(value).ok_or_else(|| error(format!("no entry is named {value}")))?
            };
            if entry.entry().arg.is_some_and(|arg| arg.required) {
                return Err(error(format!(
                    "{value} needs a world, so a key cannot run it"
                )));
            }
            keymap.bind(chord, entry);
        }
        Ok(keymap)
    }

    /// The settings of a run: the keymap of [`Self::keymap`] and the echo.
    ///
    /// # Errors
    ///
    /// The [`KeyError`] of [`Self::keymap`].
    pub fn settings(&self) -> Result<Settings, KeyError> {
        Ok(Settings {
            keymap: self.keymap()?,
            echo: self.echo,
        })
    }
}
