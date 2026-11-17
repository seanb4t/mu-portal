//! The configuration of the worlds: the layers, their merge, and the resolved world that the
//! sessions read.

use std::collections::BTreeMap;
use std::time::Duration;

use crate::framer::{Charset, TelnetSettings};
use crate::id::WorldId;
use crate::secret::SecretRef;

mod builtin;
mod check;
mod dir;
mod error;
mod layer;
mod parse;
mod resolve;

pub use builtin::{BUILT_IN_TYPES, built_in_layers, parse_quirks};
pub use dir::TomlDirSource;
pub use error::ConfigError;
pub use layer::{
    KeepaliveFields, Layer, LayerFields, LayerKey, LayerOrigin, LayerSource, LoginFields, Trust,
};
pub use resolve::{FIELD_PATHS, resolve};

/// The login template of a world whose layers set none.
pub const DEFAULT_LOGIN_TEMPLATE: &str = "connect {login} {password}";

/// The defaults of a world, which the default layer of the merge holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldDefaults {
    /// The character set of the text.
    pub charset: Charset,
    /// The width of the window.
    pub columns: u16,
    /// The height of the window.
    pub rows: u16,
    /// Whether the connection uses TLS.
    pub tls: bool,
    /// The login template.
    pub login_template: &'static str,
}

impl Default for WorldDefaults {
    /// The settings of [`TelnetSettings::default`], no TLS and [`DEFAULT_LOGIN_TEMPLATE`].
    fn default() -> Self {
        let telnet = TelnetSettings::default();
        Self {
            charset: telnet.charset,
            columns: telnet.columns,
            rows: telnet.rows,
            tls: false,
            login_template: DEFAULT_LOGIN_TEMPLATE,
        }
    }
}

/// How a world logs in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Login {
    /// The line to send, with `{login}` and `{password}` in it.
    pub template: String,
    /// The login name.
    pub name: String,
    /// Where the password comes from.
    pub secret: SecretRef,
}

impl Login {
    /// The login line for `password`.
    #[must_use]
    pub fn line(&self, password: &str) -> String {
        self.template
            .replace("{login}", &self.name)
            .replace("{password}", password)
    }
}

/// A command that the session sends at a fixed period, so the connection stays open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keepalive {
    /// The line to send.
    pub command: String,
    /// The time between two sends.
    pub period: Duration,
}

/// Whether an entry is a rule or a script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    /// An entry of `rules`.
    Rule,
    /// An entry of `scripts`.
    Script,
}

/// One rule or script: its id, the rest of its table, and the layer that added it.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// The id, unique among the rules and the scripts of a world.
    pub id: String,
    /// The whole table of the entry, with its `id`.
    pub table: toml::Table,
    /// The layer that added the entry.
    pub origin: LayerOrigin,
}

/// One world after the merge of its configuration layers.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedWorld {
    /// The id of the world.
    pub id: WorldId,
    /// The host of the game. A fake world has an empty host.
    pub host: String,
    /// The port of the game. A fake world has port 0.
    pub port: u16,
    /// Whether the connection uses TLS.
    pub tls: bool,
    /// The character set of the text.
    pub charset: Charset,
    /// The width of the window.
    pub columns: u16,
    /// The height of the window.
    pub rows: u16,
    /// The login, or none for a world that logs in by hand.
    pub login: Option<Login>,
    /// The keepalive, or none.
    pub keepalive: Option<Keepalive>,
    /// The rules, in the order of their first layer.
    pub rules: Vec<Entry>,
    /// The scripts, in the order of their first layer.
    pub scripts: Vec<Entry>,
    /// The source of each field that the world has, by its path such as `login.name`.
    pub sources: BTreeMap<String, LayerOrigin>,
}

impl ResolvedWorld {
    /// A world with the values of [`WorldDefaults`], no host, no login and no source.
    #[must_use]
    pub fn new(id: WorldId) -> Self {
        let defaults = WorldDefaults::default();
        Self {
            id,
            host: String::new(),
            port: 0,
            tls: defaults.tls,
            charset: defaults.charset,
            columns: defaults.columns,
            rows: defaults.rows,
            login: None,
            keepalive: None,
            rules: Vec::new(),
            scripts: Vec::new(),
            sources: BTreeMap::new(),
        }
    }

    /// A world that logs in with the default template and its character part as the name.
    #[must_use]
    pub fn with_default_login(id: WorldId, secret: SecretRef) -> Self {
        let login = Login {
            template: WorldDefaults::default().login_template.to_owned(),
            name: id.character().to_owned(),
            secret,
        };
        Self {
            login: Some(login),
            ..Self::new(id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_login_uses_the_character_part() {
        let world = ResolvedWorld::with_default_login(
            "alpha/aria".parse().unwrap(),
            "env:ALPHA_PW".parse().unwrap(),
        );
        assert_eq!(world.login.unwrap().line("pw"), "connect aria pw");
    }
}
