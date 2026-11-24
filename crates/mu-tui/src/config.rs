//! `client.toml`, the configuration of the terminal client. `mu-cli` reads and parses the
//! file, and this module turns it into the [`Settings`] of a run.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::keymap::Keymap;

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

pub use mu_config::KeyError;

impl ClientConfig {
    /// The default keymap with each binding of `[keys]` in place of the default of its key.
    /// `none` binds the key to `ignore`.
    ///
    /// # Errors
    ///
    /// The [`KeyError`] of [`mu_config::check_keys`].
    pub fn keymap(&self) -> Result<Keymap, KeyError> {
        Ok(Keymap::with_bindings(&mu_config::check_keys(&self.keys)?))
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
