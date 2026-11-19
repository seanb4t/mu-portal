//! The layers of the configuration, and the seam that gives them to the resolver.

use std::fmt;
use std::marker::PhantomData;

use serde::de::{MapAccess, Visitor, value::MapAccessDeserializer};
use serde::{Deserialize, Deserializer, Serialize};

use crate::id::WorldId;

use super::error::ConfigError;
use super::{KeepaliveMode, ReconnectMode};

/// What a layer configures.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayerKey {
    /// `global.toml`, which reaches each world.
    Global,
    /// A type, built in or from `types/<name>.toml`.
    Type(String),
    /// A game, from `games/<game>.toml`.
    Game(String),
    /// A character, which makes one world.
    Character(WorldId),
    /// A layer of a plugin, which sits between the game and the character.
    Plugin(String),
}

/// How far the resolver trusts a layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trust {
    /// A layer of the owner, which sets any field.
    Owner,
    /// A layer of a shared plugin, which cannot set the connection or the login.
    SharedPlugin,
}

/// Where a layer, or a field of a resolved world, comes from.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayerOrigin {
    /// A file, by its path relative to the configuration directory, with `/` between parts.
    File(String),
    /// A built-in type, by its name.
    BuiltIn(String),
    /// A plugin, by its name.
    Plugin(String),
    /// The defaults of a world.
    Default,
}

impl fmt::Display for LayerOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(path) => f.write_str(path),
            Self::BuiltIn(name) => write!(f, "built-in {name}"),
            Self::Plugin(name) => write!(f, "plugin {name}"),
            Self::Default => f.write_str("default"),
        }
    }
}

/// One layer of the configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    /// What the layer configures.
    pub key: LayerKey,
    /// Where the layer comes from.
    pub origin: LayerOrigin,
    /// How far the resolver trusts the layer.
    pub trust: Trust,
    /// The type that a game or a type extends.
    pub extends: Option<String>,
    /// The fields that the layer sets.
    pub fields: LayerFields,
}

/// A source of layers, such as a directory of TOML files.
pub trait LayerSource {
    /// The layers of the source.
    ///
    /// # Errors
    ///
    /// An error that names the file and the field that the source cannot read.
    fn layers(&self) -> Result<Vec<Layer>, ConfigError>;
}

/// The fields of one layer. A field that the layer does not set is `None`.
///
/// The merge serializes the fields, and a `None` field is left out, so it keeps the value of
/// an earlier layer. The arrays stay out of the merge.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerFields {
    /// The host of the game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    /// The port of the game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// Whether the connection uses TLS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<bool>,
    /// Whether the world connects at the start of the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autoconnect: Option<bool>,
    /// The name of the character set, such as `iso-8859-1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charset: Option<String>,
    /// The width of the window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u16>,
    /// The height of the window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<u16>,
    /// The table `login`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub login: Option<LoginFields>,
    /// The table `keepalive`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keepalive: Option<KeepaliveFields>,
    /// The table `reconnect`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reconnect: Option<ReconnectFields>,
    /// The entries of `rules`.
    #[serde(skip)]
    pub rules: Vec<toml::Table>,
    /// The entries of `scripts`.
    #[serde(skip)]
    pub scripts: Vec<toml::Table>,
    /// The ids of the inherited rules and scripts that the layer removes.
    #[serde(skip)]
    pub disable: Vec<String>,
}

/// The fields of the table `login`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginFields {
    /// The line to send, with `{login}` and `{password}` in it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// The login name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The secret reference of the password, such as `env:ARIA_PW`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

/// The fields of the table `keepalive`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeepaliveFields {
    /// When the keepalive sends its line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<KeepaliveMode>,
    /// The line to send.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// The seconds of the mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<u32>,
}

/// The fields of the table `reconnect`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconnectFields {
    /// Whether the world reconnects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<ReconnectMode>,
    /// The first wait, in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial: Option<u32>,
    /// The cap of each wait, in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
    /// The number of failed attempts in a row before the world stops, or 0 for no limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempts: Option<u32>,
}

/// One file of the configuration directory, as the parse reads it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LayerFile {
    extends: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    tls: Option<bool>,
    autoconnect: Option<bool>,
    charset: Option<String>,
    columns: Option<u16>,
    rows: Option<u16>,
    #[serde(default, deserialize_with = "table")]
    login: Option<LoginFields>,
    #[serde(default, deserialize_with = "table")]
    keepalive: Option<KeepaliveFields>,
    #[serde(default, deserialize_with = "table")]
    reconnect: Option<ReconnectFields>,
    #[serde(default)]
    rules: Vec<toml::Table>,
    #[serde(default)]
    scripts: Vec<toml::Table>,
    #[serde(default)]
    disable: Vec<String>,
}

impl LayerFile {
    /// The `extends` of the file, and its fields.
    pub(crate) fn split(self) -> (Option<String>, LayerFields) {
        let fields = LayerFields {
            host: self.host,
            port: self.port,
            tls: self.tls,
            autoconnect: self.autoconnect,
            charset: self.charset,
            columns: self.columns,
            rows: self.rows,
            login: self.login,
            keepalive: self.keepalive,
            reconnect: self.reconnect,
            rules: self.rules,
            scripts: self.scripts,
            disable: self.disable,
        };
        (self.extends, fields)
    }
}

/// A built-in type, which sets only quirks.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Quirks {
    charset: Option<String>,
    #[serde(default, deserialize_with = "table")]
    login: Option<QuirkLogin>,
    #[serde(default, deserialize_with = "table")]
    keepalive: Option<QuirkKeepalive>,
}

/// The table `login` of a built-in type.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct QuirkLogin {
    template: Option<String>,
}

/// The table `keepalive` of a built-in type: its command and seconds, and no mode.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct QuirkKeepalive {
    command: Option<String>,
    seconds: Option<u32>,
}

impl From<Quirks> for LayerFields {
    fn from(quirks: Quirks) -> Self {
        Self {
            charset: quirks.charset,
            login: quirks.login.map(|login| LoginFields {
                template: login.template,
                ..LoginFields::default()
            }),
            keepalive: quirks.keepalive.map(|keepalive| KeepaliveFields {
                command: keepalive.command,
                seconds: keepalive.seconds,
                ..KeepaliveFields::default()
            }),
            ..Self::default()
        }
    }
}

/// Reads a table field as `T`, and refuses any other value. The derived `Deserialize` of a
/// struct also reads an array, by the order of the fields, and drops extra elements.
fn table<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Table<T>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>> Visitor<'de> for Table<T> {
        type Value = T;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a table")
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
            T::deserialize(MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_map(Table(PhantomData)).map(Some)
}
