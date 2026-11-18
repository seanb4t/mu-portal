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
pub use dir::{SERVER_FILE, TomlDirSource};
pub use error::{ConfigError, FieldError};
pub use layer::{
    KeepaliveFields, Layer, LayerFields, LayerKey, LayerOrigin, LayerSource, LoginFields, Trust,
};
pub use parse::{Hidden, parse as parse_toml};
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

/// How a world logs in. Its template holds `{password}`, and neither the template nor the name
/// holds a CR or an LF, so the login is one line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Login {
    template: String,
    name: String,
    secret: SecretRef,
}

impl Login {
    /// A login that sends `template`, with `{login}` and `{password}` in it, as `name`, with
    /// the password from `secret`.
    ///
    /// # Errors
    ///
    /// A [`FieldError`] when `template` holds no `{password}`, or `template` or `name` holds a
    /// CR or an LF.
    pub fn new(
        template: impl Into<String>,
        name: impl Into<String>,
        secret: SecretRef,
    ) -> Result<Self, FieldError> {
        let (template, name) = (template.into(), name.into());
        check::check_login(Some(&name), Some(&template))?;
        Ok(Self {
            template,
            name,
            secret,
        })
    }

    /// The line to send, with `{login}` and `{password}` in it.
    #[must_use]
    pub fn template(&self) -> &str {
        &self.template
    }

    /// The login name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Where the password comes from.
    #[must_use]
    pub fn secret(&self) -> &SecretRef {
        &self.secret
    }

    /// The login line for `password`.
    #[must_use]
    pub fn line(&self, password: &str) -> String {
        self.template
            .replace("{login}", &self.name)
            .replace("{password}", password)
    }
}

/// The keepalive of a world: a line to send, and the seconds between two sends, from 1 to
/// 86400. The line holds no CR and no LF.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keepalive {
    command: String,
    seconds: u32,
}

impl Keepalive {
    /// A keepalive that sends `command` every `seconds`.
    ///
    /// # Errors
    ///
    /// A [`FieldError`] when `command` holds a CR or an LF, or `seconds` is not from 1 to 86400.
    pub fn new(command: impl Into<String>, seconds: u32) -> Result<Self, FieldError> {
        let command = command.into();
        check::check_keepalive(Some(&command), Some(seconds))?;
        Ok(Self { command, seconds })
    }

    /// The line to send.
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// The seconds between two sends.
    #[must_use]
    pub fn seconds(&self) -> u32 {
        self.seconds
    }

    /// The time between two sends.
    #[must_use]
    pub fn period(&self) -> Duration {
        Duration::from_secs(u64::from(self.seconds))
    }
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
    /// The host of the game. The resolver never gives an empty host, but
    /// [`ResolvedWorld::new`] does.
    pub host: String,
    /// The port of the game. The resolver never gives port 0, but [`ResolvedWorld::new`] does.
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
    /// The rules, in the order that their ids were added. An override keeps its place, and an
    /// id that is disabled and added again moves to the end.
    pub rules: Vec<Entry>,
    /// The scripts, in the same order as the rules.
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
        // The default template and the id rule pass the checks of `Login::new`.
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

    #[test]
    fn login_refuses_a_bad_template_or_name() {
        let secret = || "env:PW".parse::<SecretRef>().unwrap();
        for (template, name, field) in [
            ("connect {login}", "aria", "login.template"),
            ("connect {login} {password}\nlook", "aria", "login.template"),
            ("connect {login} {password}", "aria\r", "login.name"),
        ] {
            let error = Login::new(template, name, secret()).unwrap_err();
            assert_eq!(error.field(), field, "{template:?} {name:?}");
        }
        assert!(Login::new("connect {login} {password}", "aria", secret()).is_ok());
    }

    #[test]
    fn keepalive_refuses_a_bad_command_or_period() {
        for (command, seconds) in [("idle", 0), ("idle", 86_401), ("idle\n", 60)] {
            assert!(
                Keepalive::new(command, seconds).is_err(),
                "{command:?} {seconds}"
            );
        }
        let keepalive = Keepalive::new("idle", 86_400).unwrap();
        assert_eq!(keepalive.period().as_secs(), 86_400);
    }
}
