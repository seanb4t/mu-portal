//! The resolver of the configuration, with layers that the tests build.

use std::collections::BTreeSet;

use mu_core::world::{
    BUILT_IN_TYPES, ConfigError, FIELD_PATHS, Layer, LayerFields, LayerKey, LayerOrigin,
    LayerSource, LoginFields, Trust, built_in_layers, parse_quirks, resolve,
};

/// A source that gives fixed layers.
struct Layers(Vec<Layer>);

impl LayerSource for Layers {
    fn layers(&self) -> Result<Vec<Layer>, ConfigError> {
        Ok(self.0.clone())
    }
}

fn owner(key: LayerKey, path: &str, extends: Option<&str>, fields: LayerFields) -> Layer {
    Layer {
        key,
        origin: LayerOrigin::File(path.to_owned()),
        trust: Trust::Owner,
        extends: extends.map(str::to_owned),
        fields,
    }
}

/// The base of the spec: the game `alpha` on `tinymux` with a host and a port, and the
/// character `aria` with `character`.
fn base(character: LayerFields) -> Vec<Layer> {
    vec![
        owner(
            LayerKey::Game("alpha".to_owned()),
            "games/alpha.toml",
            Some("tinymux"),
            LayerFields {
                host: Some("mux.example.org".to_owned()),
                port: Some(4201),
                ..LayerFields::default()
            },
        ),
        owner(
            LayerKey::Character("alpha/aria".parse().unwrap()),
            "characters/alpha/aria.toml",
            None,
            character,
        ),
    ]
}

/// A layer of the shared plugin `colors` with `fields`.
fn colors(fields: LayerFields) -> Layer {
    Layer {
        key: LayerKey::Plugin("colors".to_owned()),
        origin: LayerOrigin::Plugin("colors".to_owned()),
        trust: Trust::SharedPlugin,
        extends: None,
        fields,
    }
}

#[test]
fn each_field_has_a_source() {
    let character = LayerFields {
        login: Some(LoginFields {
            name: Some("Aria".to_owned()),
            secret: Some("env:ARIA_PW".to_owned()),
            ..LoginFields::default()
        }),
        ..LayerFields::default()
    };
    let worlds = resolve(&[&Layers(base(character))]).unwrap();
    let paths: BTreeSet<&str> = worlds[0].sources.keys().map(String::as_str).collect();
    assert_eq!(paths, FIELD_PATHS.iter().copied().collect());
}

#[test]
fn plugin_cannot_set_the_connection() {
    let login = |login: LoginFields| LayerFields {
        login: Some(login),
        ..LayerFields::default()
    };
    let cases = [
        (
            "host",
            LayerFields {
                host: Some("evil.example.org".to_owned()),
                ..LayerFields::default()
            },
        ),
        (
            "port",
            LayerFields {
                port: Some(4201),
                ..LayerFields::default()
            },
        ),
        (
            "tls",
            LayerFields {
                tls: Some(true),
                ..LayerFields::default()
            },
        ),
        (
            "login.name",
            login(LoginFields {
                name: Some("Eve".to_owned()),
                ..LoginFields::default()
            }),
        ),
        (
            "login.secret",
            login(LoginFields {
                secret: Some("env:EVE_PW".to_owned()),
                ..LoginFields::default()
            }),
        ),
        (
            "login.template",
            login(LoginFields {
                template: Some("page eve={password}".to_owned()),
                ..LoginFields::default()
            }),
        ),
    ];
    for (field, fields) in cases {
        let mut layers = base(LayerFields::default());
        layers.push(colors(fields));
        let error = resolve(&[&Layers(layers)]).unwrap_err().to_string();
        assert!(
            error.contains("colors") && error.contains(field),
            "{field}: {error}"
        );
    }
}

#[test]
fn plugin_can_set_a_preference() {
    let mut layers = base(LayerFields::default());
    layers.push(colors(LayerFields {
        columns: Some(72),
        ..LayerFields::default()
    }));
    let worlds = resolve(&[&Layers(layers)]).unwrap();
    assert_eq!(worlds[0].columns, 72);
}

#[test]
fn built_in_types_hold_only_quirks() {
    let names: Vec<String> = built_in_layers()
        .unwrap()
        .into_iter()
        .map(|layer| layer.origin.to_string())
        .collect();
    assert_eq!(
        names,
        ["built-in aresmush", "built-in evennia", "built-in tinymux"]
    );
    assert_eq!(BUILT_IN_TYPES.len(), names.len());
}

#[test]
fn quirks_refuse_other_fields() {
    for (field, text) in [
        ("columns", "columns = 80\n"),
        ("login.name", "[login]\nname = \"Aria\"\n"),
        ("host", "host = \"mux.example.org\"\n"),
        ("login", "login = [\"connect {login} {password}\"]\n"),
        ("keepalive", "keepalive = [\"IDLE\", 60]\n"),
    ] {
        let error = parse_quirks("tinymux", text).unwrap_err().to_string();
        assert!(error.contains(field), "{field}: {error}");
    }
}

#[test]
fn keepalive_of_a_built_in_type_resolves() {
    let worlds = resolve(&[&Layers(base(LayerFields::default()))]).unwrap();
    let keepalive = worlds[0].keepalive.clone().unwrap();
    assert_eq!(
        (keepalive.command.as_str(), keepalive.period.as_secs()),
        ("IDLE", 60)
    );
}
