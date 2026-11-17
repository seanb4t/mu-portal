//! The resolver: it builds the chain of each world, merges the fields with `figment2`,
//! accumulates the rules and the scripts, and checks the result.

use std::collections::BTreeMap;
use std::time::Duration;

use figment2::Figment;
use figment2::providers::{Named, Serialized};

use crate::framer::Charset;
use crate::id::WorldId;
use crate::secret::SecretRef;

use super::builtin::built_in_layers;
use super::check::check_layer;
use super::error::ConfigError;
use super::layer::{
    KeepaliveFields, Layer, LayerFields, LayerKey, LayerOrigin, LayerSource, LoginFields,
};
use super::{Entry, EntryKind, Keepalive, Login, ResolvedWorld, WorldDefaults};

/// The paths of the fields that a resolved world can have, in the order of `mu world show`.
pub const FIELD_PATHS: &[&str] = &[
    "host",
    "port",
    "tls",
    "charset",
    "columns",
    "rows",
    "login.template",
    "login.name",
    "login.secret",
    "keepalive.command",
    "keepalive.seconds",
];

/// Resolves each world of the layers of `sources`, in the order of their ids.
///
/// The resolver reads no secret.
///
/// # Errors
///
/// The first error of a layer, a chain, the merge or a world. It names the file or the world,
/// and the field.
pub fn resolve(sources: &[&dyn LayerSource]) -> Result<Vec<ResolvedWorld>, ConfigError> {
    let mut layers = Vec::new();
    for source in sources {
        layers.extend(source.layers()?);
    }
    for layer in &layers {
        check_layer(layer)?;
    }
    let built_in = built_in_layers()?;
    for layer in &layers {
        if let LayerKey::Type(name) = &layer.key
            && built_in.iter().any(|built| built.key == layer.key)
        {
            return Err(ConfigError::new(
                &layer.origin,
                format!("{name} is the name of a built-in type"),
            ));
        }
    }
    let index = Index::new(&layers, &built_in);
    for layer in &layers {
        if matches!(layer.key, LayerKey::Game(_) | LayerKey::Type(_)) {
            index.types_of(layer)?;
        }
    }
    let mut worlds = Vec::new();
    for layer in &layers {
        if let LayerKey::Character(id) = &layer.key {
            let chain = index.chain(id, layer)?;
            worlds.push(world(id.clone(), &chain)?);
        }
    }
    worlds.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(worlds)
}

/// The layers by what they configure.
struct Index<'a> {
    global: Option<&'a Layer>,
    types: BTreeMap<&'a str, &'a Layer>,
    games: BTreeMap<&'a str, &'a Layer>,
    plugins: Vec<&'a Layer>,
}

impl<'a> Index<'a> {
    fn new(layers: &'a [Layer], built_in: &'a [Layer]) -> Self {
        let mut index = Self {
            global: None,
            types: BTreeMap::new(),
            games: BTreeMap::new(),
            plugins: Vec::new(),
        };
        for layer in built_in.iter().chain(layers) {
            match &layer.key {
                LayerKey::Global => index.global = Some(layer),
                LayerKey::Type(name) => {
                    index.types.insert(name, layer);
                }
                LayerKey::Game(name) => {
                    index.games.insert(name, layer);
                }
                LayerKey::Plugin(_) => index.plugins.push(layer),
                LayerKey::Character(_) => {}
            }
        }
        index
    }

    /// The types that `layer` extends, from the most specific to the most general.
    fn types_of(&self, layer: &'a Layer) -> Result<Vec<&'a Layer>, ConfigError> {
        let mut seen: Vec<&Layer> = Vec::new();
        if matches!(layer.key, LayerKey::Type(_)) {
            seen.push(layer);
        }
        let mut current = layer;
        while let Some(name) = &current.extends {
            let Some(&parent) = self.types.get(name.as_str()) else {
                return Err(ConfigError::field(
                    &current.origin,
                    "extends",
                    format!("{name} is neither a built-in type nor a file in types/"),
                ));
            };
            if let Some(start) = seen.iter().position(|type_| type_.key == parent.key) {
                let files: Vec<String> = seen[start..]
                    .iter()
                    .map(|type_| type_.origin.to_string())
                    .collect();
                return Err(ConfigError::field(
                    &parent.origin,
                    "extends",
                    format!("the types make a loop: {}", files.join(", ")),
                ));
            }
            seen.push(parent);
            current = parent;
        }
        if matches!(layer.key, LayerKey::Type(_)) {
            seen.remove(0);
        }
        Ok(seen)
    }

    /// The layers of the world `id`, from the most general to the character.
    fn chain(&self, id: &WorldId, character: &'a Layer) -> Result<Vec<&'a Layer>, ConfigError> {
        let game = self.games.get(id.game()).ok_or_else(|| {
            ConfigError::new(
                &character.origin,
                format!("the game has no file games/{}.toml", id.game()),
            )
        })?;
        let mut chain: Vec<&Layer> = self.global.into_iter().collect();
        chain.extend(self.types_of(game)?.into_iter().rev());
        chain.push(game);
        chain.extend(&self.plugins);
        chain.push(character);
        Ok(chain)
    }
}

/// The layer of the defaults.
fn defaults() -> LayerFields {
    let defaults = WorldDefaults::default();
    LayerFields {
        tls: Some(defaults.tls),
        charset: Some(defaults.charset.name().to_owned()),
        columns: Some(defaults.columns),
        rows: Some(defaults.rows),
        login: Some(LoginFields {
            template: Some(defaults.login_template.to_owned()),
            ..LoginFields::default()
        }),
        ..LayerFields::default()
    }
}

/// Merges `chain` into the world `id`.
fn world(id: WorldId, chain: &[&Layer]) -> Result<ResolvedWorld, ConfigError> {
    let mut origins = vec![LayerOrigin::Default];
    let mut figment = Figment::from(Named::new("0", Serialized::defaults(defaults())));
    for layer in chain {
        let name = origins.len().to_string();
        origins.push(layer.origin.clone());
        figment = figment.merge(Named::new(name, Serialized::defaults(&layer.fields)));
    }
    let merged: LayerFields = figment
        .extract()
        .map_err(|error| ConfigError::new(&id, error.to_string()))?;
    let mut sources = BTreeMap::new();
    for path in FIELD_PATHS {
        if let Some(metadata) = figment.find_metadata(path)
            && figment.find_value(path).is_ok()
            && let Some(origin) = metadata
                .name
                .parse::<usize>()
                .ok()
                .and_then(|index| origins.get(index))
        {
            sources.insert((*path).to_owned(), origin.clone());
        }
    }
    let (rules, scripts) = accumulate(chain)?;
    finish(id, merged, sources, rules, scripts)
}

/// The rules and the scripts of `chain`.
fn accumulate(chain: &[&Layer]) -> Result<(Vec<Entry>, Vec<Entry>), ConfigError> {
    let mut entries: Vec<(EntryKind, Entry)> = Vec::new();
    for layer in chain {
        for id in &layer.fields.disable {
            let Some(position) = entries.iter().position(|(_, entry)| &entry.id == id) else {
                return Err(ConfigError::field(
                    &layer.origin,
                    "disable",
                    format!("{id} names no inherited rule or script"),
                ));
            };
            entries.remove(position);
        }
        let new = [
            (EntryKind::Rule, &layer.fields.rules),
            (EntryKind::Script, &layer.fields.scripts),
        ];
        for (kind, tables) in new {
            for table in tables {
                let id = table
                    .get("id")
                    .and_then(toml::Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                let entry = Entry {
                    id,
                    table: table.clone(),
                    origin: layer.origin.clone(),
                };
                match entries.iter_mut().find(|(_, old)| old.id == entry.id) {
                    Some((old_kind, _)) if *old_kind != kind => {
                        return Err(ConfigError::new(
                            &layer.origin,
                            format!("{} is the id of both a rule and a script", entry.id),
                        ));
                    }
                    Some((_, old)) => *old = entry,
                    None => entries.push((kind, entry)),
                }
            }
        }
    }
    let (rules, scripts): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .partition(|(kind, _)| *kind == EntryKind::Rule);
    Ok((
        rules.into_iter().map(|(_, entry)| entry).collect(),
        scripts.into_iter().map(|(_, entry)| entry).collect(),
    ))
}

/// Checks the merged fields of the world `id`, and builds the world.
fn finish(
    id: WorldId,
    merged: LayerFields,
    sources: BTreeMap<String, LayerOrigin>,
    rules: Vec<Entry>,
    scripts: Vec<Entry>,
) -> Result<ResolvedWorld, ConfigError> {
    let missing = |field: &str| ConfigError::field(&id, field, "no layer sets it");
    let host = merged
        .host
        .filter(|host| !host.is_empty())
        .ok_or_else(|| missing("host"))?;
    let port = merged.port.ok_or_else(|| missing("port"))?;
    let charset = merged
        .charset
        .as_deref()
        .and_then(Charset::from_name)
        .ok_or_else(|| missing("charset"))?;
    let login = merged.login.unwrap_or_default();
    let login = match (login.name, login.secret) {
        (Some(name), Some(secret)) => Some(Login {
            template: login.template.ok_or_else(|| missing("login.template"))?,
            name,
            secret: secret.parse::<SecretRef>().map_err(|_| {
                ConfigError::field(&id, "login.secret", "is not a secret reference")
            })?,
        }),
        (Some(_), None) => return Err(missing("login.secret")),
        (None, Some(_)) => return Err(missing("login.name")),
        (None, None) => None,
    };
    let keepalive = merged.keepalive.unwrap_or_default();
    let keepalive = match keepalive {
        KeepaliveFields {
            command: Some(command),
            seconds: Some(seconds),
        } => Some(Keepalive {
            command,
            period: Duration::from_secs(u64::from(seconds)),
        }),
        KeepaliveFields {
            command: Some(_),
            seconds: None,
        } => return Err(missing("keepalive.seconds")),
        KeepaliveFields {
            command: None,
            seconds: Some(_),
        } => return Err(missing("keepalive.command")),
        KeepaliveFields {
            command: None,
            seconds: None,
        } => None,
    };
    let mut sources = sources;
    if login.is_none() {
        sources.retain(|path, _| !path.starts_with("login."));
    }
    let defaults = WorldDefaults::default();
    Ok(ResolvedWorld {
        id,
        host,
        port,
        tls: merged.tls.unwrap_or(defaults.tls),
        charset,
        columns: merged.columns.unwrap_or(defaults.columns),
        rows: merged.rows.unwrap_or(defaults.rows),
        login,
        keepalive,
        rules,
        scripts,
        sources,
    })
}
