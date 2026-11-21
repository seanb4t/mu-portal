//! The built-in types, which ship inside the binary. Each one sets only quirks, and can extend
//! another built-in type.

use super::check::check_layer;
use super::error::ConfigError;
use super::layer::{Layer, LayerKey, LayerOrigin, Quirks, Trust};
use super::parse::{LAYER_HIDDEN, parse};

/// The name and the TOML text of each built-in type.
pub const BUILT_IN_TYPES: &[(&str, &str)] = &[
    ("aresmush", include_str!("types/aresmush.toml")),
    ("evennia", include_str!("types/evennia.toml")),
    ("pennmush", include_str!("types/pennmush.toml")),
    ("rhostmush", include_str!("types/rhostmush.toml")),
    ("tinymux", include_str!("types/tinymux.toml")),
];

/// Reads `text` as the built-in type `name`, through the schema of the quirk fields and the
/// checks of the values of each layer.
///
/// # Errors
///
/// An error that names the type, for a TOML syntax error, or with the field, for a field that is
/// not a quirk or a value that fails a check.
pub fn parse_quirks(name: &str, text: &str) -> Result<Layer, ConfigError> {
    let origin = LayerOrigin::BuiltIn(name.to_owned());
    let quirks: Quirks = parse(&origin, text, LAYER_HIDDEN)?;
    let (extends, fields) = quirks.split();
    let layer = Layer {
        key: LayerKey::Type(name.to_owned()),
        origin,
        trust: Trust::Owner,
        extends,
        fields,
    };
    check_layer(&layer)?;
    Ok(layer)
}

/// The layer of each built-in type.
///
/// # Errors
///
/// An error of [`parse_quirks`] for a built-in type.
pub fn built_in_layers() -> Result<Vec<Layer>, ConfigError> {
    BUILT_IN_TYPES
        .iter()
        .map(|(name, text)| parse_quirks(name, text))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quirks_refuse_connection_fields() {
        for (text, field) in [
            ("[keepalive]\nmode = \"off\"\n", "keepalive.mode"),
            ("autoconnect = true\n", "autoconnect"),
            ("[reconnect]\nmode = \"off\"\n", "reconnect"),
        ] {
            let error = parse_quirks("bad", text).unwrap_err().to_string();
            assert!(error.contains(field), "{text:?}: {error}");
        }
    }

    #[test]
    fn bad_quirk_fails_the_value_checks() {
        let error = parse_quirks("bad", "[login]\ntemplate = \"connect {login}\"\n").unwrap_err();
        assert!(error.to_string().contains("login.template"), "{error}");
    }
}
