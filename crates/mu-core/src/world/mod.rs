//! The configuration of the worlds: the layers, their merge, and the resolved world that the
//! sessions read.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

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
    KeepaliveFields, Layer, LayerFields, LayerKey, LayerOrigin, LayerSource, LoginFields,
    ReconnectFields, Trust,
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
    /// Whether the world connects at the start of the server.
    pub autoconnect: bool,
    /// The mode of the keepalive.
    pub keepalive_mode: KeepaliveMode,
    /// The waits of the reconnect, whose mode is [`ReconnectMode::Backoff`].
    pub reconnect: Backoff,
}

impl Default for WorldDefaults {
    /// The settings of [`TelnetSettings::default`], no TLS, [`DEFAULT_LOGIN_TEMPLATE`], no
    /// autoconnect, the keepalive mode `idle`, and a reconnect from 1 to 60 seconds that stops
    /// after 30 attempts.
    fn default() -> Self {
        let telnet = TelnetSettings::default();
        Self {
            charset: telnet.charset,
            columns: telnet.columns,
            rows: telnet.rows,
            tls: false,
            login_template: DEFAULT_LOGIN_TEMPLATE,
            autoconnect: false,
            keepalive_mode: KeepaliveMode::Idle,
            reconnect: Backoff {
                initial: 1,
                max: 60,
                attempts: 30,
            },
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

/// When a keepalive sends its line: the value of `keepalive.mode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeepaliveMode {
    /// After the set seconds with no write to the game.
    #[default]
    Idle,
    /// Every set seconds.
    Interval,
    /// Never.
    Off,
}

impl KeepaliveMode {
    /// The name of the mode in the configuration.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Interval => "interval",
            Self::Off => "off",
        }
    }
}

/// The keepalive of a world: a mode, a line to send, and the seconds of the mode, from 1 to
/// 86400. The line holds no CR and no LF.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keepalive {
    mode: KeepaliveMode,
    command: String,
    seconds: u32,
}

impl Keepalive {
    /// A keepalive that sends `command` after `seconds` with no write: the mode
    /// [`KeepaliveMode::Idle`].
    ///
    /// # Errors
    ///
    /// A [`FieldError`] when `command` holds a CR or an LF, or `seconds` is not from 1 to 86400.
    pub fn new(command: impl Into<String>, seconds: u32) -> Result<Self, FieldError> {
        let command = command.into();
        check::check_keepalive(Some(&command), Some(seconds))?;
        Ok(Self {
            mode: KeepaliveMode::Idle,
            command,
            seconds,
        })
    }

    /// The same keepalive with `mode`.
    #[must_use]
    pub fn with_mode(self, mode: KeepaliveMode) -> Self {
        Self { mode, ..self }
    }

    /// The mode.
    #[must_use]
    pub fn mode(&self) -> KeepaliveMode {
        self.mode
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

/// The value of `reconnect.mode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReconnectMode {
    /// Waits longer before each attempt.
    #[default]
    Backoff,
    /// Never reconnects.
    Off,
}

impl ReconnectMode {
    /// The name of the mode in the configuration.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Backoff => "backoff",
            Self::Off => "off",
        }
    }
}

/// The waits of a reconnect: the first wait and the cap, in seconds, and the number of failed
/// attempts in a row before the world stops. The first wait is from 1 to 3600 and not greater
/// than the cap, the cap is from 1 to 86400, and the attempts are from 0 to 10000. With 0
/// attempts, the world never stops.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Backoff {
    initial: u32,
    max: u32,
    attempts: u32,
}

impl Backoff {
    /// A backoff from `initial` seconds up to `max` seconds, which stops after `attempts`.
    ///
    /// # Errors
    ///
    /// A [`FieldError`] when a value is out of its range, or `initial` is greater than `max`.
    pub fn new(initial: u32, max: u32, attempts: u32) -> Result<Self, FieldError> {
        check::check_reconnect(Some(initial), Some(max), Some(attempts))?;
        check::check_first_wait(initial, max)?;
        Ok(Self {
            initial,
            max,
            attempts,
        })
    }

    /// The first wait, in seconds.
    #[must_use]
    pub fn initial(&self) -> u32 {
        self.initial
    }

    /// The cap of each wait, in seconds.
    #[must_use]
    pub fn max(&self) -> u32 {
        self.max
    }

    /// The number of failed attempts in a row before the world stops, or 0 for no limit.
    #[must_use]
    pub fn attempts(&self) -> u32 {
        self.attempts
    }
}

/// Whether a world reconnects after a drop that the player did not ask for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reconnect {
    /// It reconnects with these waits.
    Backoff(Backoff),
    /// It does not reconnect.
    Off,
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
    /// Whether the world connects at the start of the server.
    pub autoconnect: bool,
    /// The character set of the text.
    pub charset: Charset,
    /// The width of the window.
    pub columns: u16,
    /// The height of the window.
    pub rows: u16,
    /// The login, or none for a world that logs in by hand.
    pub login: Option<Login>,
    /// The mode of `keepalive.mode`, also for a world with no keepalive.
    pub keepalive_mode: KeepaliveMode,
    /// The keepalive, or none for the mode `off` or a world with no command and no seconds.
    pub keepalive: Option<Keepalive>,
    /// Whether the world reconnects after a drop.
    pub reconnect: Reconnect,
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
            autoconnect: defaults.autoconnect,
            charset: defaults.charset,
            columns: defaults.columns,
            rows: defaults.rows,
            login: None,
            keepalive_mode: defaults.keepalive_mode,
            keepalive: None,
            reconnect: Reconnect::Backoff(defaults.reconnect),
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
