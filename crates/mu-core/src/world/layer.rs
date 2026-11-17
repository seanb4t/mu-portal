//! The layers of the configuration, and the seam that gives them to the resolver.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::id::WorldId;

use super::error::ConfigError;

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
    /// The line to send.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// The seconds between two sends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<u32>,
}

/// One file of the configuration directory, as the parse reads it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LayerFile {
    extends: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    tls: Option<bool>,
    charset: Option<String>,
    columns: Option<u16>,
    rows: Option<u16>,
    login: Option<LoginFields>,
    keepalive: Option<KeepaliveFields>,
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
            charset: self.charset,
            columns: self.columns,
            rows: self.rows,
            login: self.login,
            keepalive: self.keepalive,
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
    login: Option<QuirkLogin>,
    keepalive: Option<KeepaliveFields>,
}

/// The table `login` of a built-in type.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct QuirkLogin {
    template: Option<String>,
}

impl From<Quirks> for LayerFields {
    fn from(quirks: Quirks) -> Self {
        Self {
            charset: quirks.charset,
            login: quirks.login.map(|login| LoginFields {
                template: login.template,
                ..LoginFields::default()
            }),
            keepalive: quirks.keepalive,
            ..Self::default()
        }
    }
}
