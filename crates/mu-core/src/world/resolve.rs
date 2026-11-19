//! The resolver: it builds the chain of each world, merges the fields with `figment2`,
//! accumulates the rules and the scripts, and checks the result.

use std::collections::BTreeMap;

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
    ReconnectFields,
};
use super::{
    Backoff, Entry, EntryKind, Keepalive, Login, Reconnect, ReconnectMode, ResolvedWorld,
    WorldDefaults,
};

/// The paths of the fields that a resolved world can have, in the order of `mu world show`. A
/// field of [`LayerFields`] that is not here gets no source.
pub const FIELD_PATHS: &[&str] = &[
    "host",
    "port",
    "tls",
    "autoconnect",
    "charset",
    "columns",
    "rows",
    "login.template",
    "login.name",
    "login.secret",
    "keepalive.mode",
    "keepalive.command",
    "keepalive.seconds",
    "reconnect.mode",
    "reconnect.initial",
    "reconnect.max",
    "reconnect.attempts",
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
        autoconnect: Some(defaults.autoconnect),
        keepalive: Some(KeepaliveFields {
            mode: Some(defaults.keepalive_mode),
            ..KeepaliveFields::default()
        }),
        reconnect: Some(ReconnectFields {
            mode: Some(ReconnectMode::Backoff),
            initial: Some(defaults.reconnect.initial()),
            max: Some(defaults.reconnect.max()),
            attempts: Some(defaults.reconnect.attempts()),
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
        (Some(name), Some(secret)) => {
            let template = login.template.ok_or_else(|| missing("login.template"))?;
            let secret = secret.parse::<SecretRef>().map_err(|_| {
                ConfigError::field(&id, "login.secret", "is not a secret reference")
            })?;
            Some(
                Login::new(template, name, secret)
                    .map_err(|error| ConfigError::of_field(&id, &error))?,
            )
        }
        (Some(_), None) => return Err(missing("login.secret")),
        (None, Some(_)) => return Err(missing("login.name")),
        (None, None) => None,
    };
    let keepalive_fields = merged.keepalive.unwrap_or_default();
    let keepalive_mode = keepalive_fields.mode.unwrap_or_default();
    let keepalive = keepalive(&id, keepalive_fields)?;
    let reconnect = reconnect(&id, &merged.reconnect.unwrap_or_default())?;
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
        autoconnect: merged.autoconnect.unwrap_or(defaults.autoconnect),
        charset,
        columns: merged.columns.unwrap_or(defaults.columns),
        rows: merged.rows.unwrap_or(defaults.rows),
        login,
        keepalive_mode,
        keepalive,
        reconnect,
        rules,
        scripts,
        sources,
    })
}

/// The keepalive of the world `id` from its merged table.
fn keepalive(id: &WorldId, fields: KeepaliveFields) -> Result<Option<Keepalive>, ConfigError> {
    let missing = |field: &str| ConfigError::field(id, field, "no layer sets it");
    match (fields.command, fields.seconds) {
        (Some(command), Some(seconds)) => Keepalive::new(command, seconds)
            .map(|keepalive| Some(keepalive.with_mode(fields.mode.unwrap_or_default())))
            .map_err(|error| ConfigError::of_field(id, &error)),
        (Some(_), None) => Err(missing("keepalive.seconds")),
        (None, Some(_)) => Err(missing("keepalive.command")),
        (None, None) => Ok(None),
    }
}

/// The reconnect of the world `id` from its merged table.
fn reconnect(id: &WorldId, fields: &ReconnectFields) -> Result<Reconnect, ConfigError> {
    if fields.mode == Some(ReconnectMode::Off) {
        return Ok(Reconnect::Off);
    }
    let defaults = WorldDefaults::default().reconnect;
    let initial = fields.initial.unwrap_or(defaults.initial());
    let max = fields.max.unwrap_or(defaults.max());
    if initial > max {
        return Err(ConfigError::field(
            id,
            "reconnect.initial",
            format!("is {initial}, which is greater than reconnect.max, {max}"),
        ));
    }
    Backoff::new(initial, max, fields.attempts.unwrap_or(defaults.attempts()))
        .map(Reconnect::Backoff)
        .map_err(|error| ConfigError::of_field(id, &error))
}

#[cfg(test)]
mod tests {
    use figment2::Figment;
    use figment2::providers::Serialized;
    use proptest::prelude::*;

    use super::*;
    use crate::world::layer::{LayerKey, Trust};

    /// The paths of `fields` that a figment of it holds.
    fn keys(fields: &LayerFields) -> Vec<&'static str> {
        let figment = Figment::from(Serialized::defaults(fields));
        FIELD_PATHS
            .iter()
            .copied()
            .filter(|path| figment.find_value(path).is_ok())
            .collect()
    }

    #[test]
    fn empty_layer_gives_figment_no_keys() {
        assert_eq!(keys(&LayerFields::default()), Vec::<&str>::new());
    }

    #[test]
    fn each_nested_field_alone_gives_one_key() {
        let text = || Some("x".to_owned());
        let login = |login: LoginFields| LayerFields {
            login: Some(login),
            ..LayerFields::default()
        };
        let keepalive = |keepalive: KeepaliveFields| LayerFields {
            keepalive: Some(keepalive),
            ..LayerFields::default()
        };
        let cases = [
            (
                "login.template",
                login(LoginFields {
                    template: text(),
                    ..LoginFields::default()
                }),
            ),
            (
                "login.name",
                login(LoginFields {
                    name: text(),
                    ..LoginFields::default()
                }),
            ),
            (
                "login.secret",
                login(LoginFields {
                    secret: text(),
                    ..LoginFields::default()
                }),
            ),
            (
                "keepalive.command",
                keepalive(KeepaliveFields {
                    command: text(),
                    ..KeepaliveFields::default()
                }),
            ),
            (
                "keepalive.seconds",
                keepalive(KeepaliveFields {
                    seconds: Some(60),
                    ..KeepaliveFields::default()
                }),
            ),
        ];
        for (path, fields) in cases {
            assert_eq!(keys(&fields), [path], "{path}");
        }
    }

    /// One layer of the model: the ids that it disables, and the ids of the rules it adds.
    #[derive(Clone, Debug)]
    struct Step {
        disable: Vec<String>,
        rules: Vec<String>,
    }

    fn step() -> impl Strategy<Value = Step> {
        let id = prop::sample::select(vec!["a", "b", "c", "d"]).prop_map(str::to_owned);
        (
            prop::collection::btree_set(id.clone(), 0..3),
            prop::collection::btree_set(id, 0..3),
        )
            .prop_map(|(disable, rules)| Step {
                disable: disable.into_iter().collect(),
                rules: rules.into_iter().collect(),
            })
    }

    fn layer(index: usize, step: &Step) -> Layer {
        let rules = step
            .rules
            .iter()
            .map(|id| {
                let mut table = toml::Table::new();
                table.insert("id".to_owned(), id.clone().into());
                table
            })
            .collect();
        Layer {
            key: LayerKey::Global,
            origin: LayerOrigin::File(format!("{index}.toml")),
            trust: Trust::Owner,
            extends: None,
            fields: LayerFields {
                rules,
                disable: step.disable.clone(),
                ..LayerFields::default()
            },
        }
    }

    proptest! {
        #[test]
        fn accumulation_matches_a_model(steps in prop::collection::vec(step(), 1..6)) {
            // The model: a list of (id, layer) in the order of first appearance. A disable
            // of an id that is not in the list is an error.
            let mut model: Vec<(String, usize)> = Vec::new();
            let mut model_ok = true;
            for (index, step) in steps.iter().enumerate() {
                for id in &step.disable {
                    match model.iter().position(|(old, _)| old == id) {
                        Some(position) => { model.remove(position); }
                        None => model_ok = false,
                    }
                }
                for id in &step.rules {
                    match model.iter_mut().find(|(old, _)| old == id) {
                        Some(entry) => entry.1 = index,
                        None => model.push((id.clone(), index)),
                    }
                }
            }
            let layers: Vec<Layer> = steps.iter().enumerate().map(|(i, s)| layer(i, s)).collect();
            let chain: Vec<&Layer> = layers.iter().collect();
            let result = accumulate(&chain);
            if !model_ok {
                prop_assert!(result.is_err());
                return Ok(());
            }
            let (rules, scripts) = result.unwrap();
            prop_assert!(scripts.is_empty());
            let got: Vec<(String, String)> =
                rules.into_iter().map(|entry| (entry.id, entry.origin.to_string())).collect();
            let want: Vec<(String, String)> =
                model.into_iter().map(|(id, index)| (id, format!("{index}.toml"))).collect();
            prop_assert_eq!(got, want);
        }
    }
}
