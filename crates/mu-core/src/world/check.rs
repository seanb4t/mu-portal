//! The checks of the values of one layer, which the resolver runs on each layer before the
//! trust check and the merge.

use std::collections::BTreeSet;

use crate::framer::{Charset, has_line_break};
use crate::secret::SecretRef;

use super::error::ConfigError;
use super::layer::{Layer, LayerFields, Trust};

/// The highest value of `keepalive.seconds`: one day.
pub(crate) const MAX_KEEPALIVE_SECONDS: u32 = 86_400;

/// Checks the values of `layer`, then its trust.
pub(crate) fn check_layer(layer: &Layer) -> Result<(), ConfigError> {
    check_fields(&layer.fields)
        .map_err(|(field, message)| ConfigError::field(&layer.origin, field, message))?;
    if layer.trust == Trust::SharedPlugin
        && let Some(field) = connection_field(&layer.fields)
    {
        return Err(ConfigError::field(
            &layer.origin,
            field,
            "a shared plugin cannot set the connection or the login",
        ));
    }
    Ok(())
}

/// The first field of the connection or the login that `fields` sets.
fn connection_field(fields: &LayerFields) -> Option<&'static str> {
    let login = fields.login.clone().unwrap_or_default();
    [
        ("host", fields.host.is_some()),
        ("port", fields.port.is_some()),
        ("tls", fields.tls.is_some()),
        ("login.name", login.name.is_some()),
        ("login.secret", login.secret.is_some()),
        ("login.template", login.template.is_some()),
    ]
    .into_iter()
    .find_map(|(field, set)| set.then_some(field))
}

/// Checks each value of `fields`, and returns the field and the message of the first error.
fn check_fields(fields: &LayerFields) -> Result<(), (String, String)> {
    let fail = |field: &str, message: &str| Err((field.to_owned(), message.to_owned()));
    for (field, value) in [
        ("port", fields.port),
        ("columns", fields.columns),
        ("rows", fields.rows),
    ] {
        if value == Some(0) {
            return fail(field, "must be from 1 to 65535");
        }
    }
    if let Some(name) = &fields.charset
        && Charset::from_name(name).is_none()
    {
        return fail("charset", "must be UTF-8 or ISO-8859-1");
    }
    if let Some(login) = &fields.login {
        if login.name.as_deref().is_some_and(has_line_break) {
            return fail("login.name", "must hold no CR and no LF");
        }
        if let Some(template) = &login.template {
            if has_line_break(template) {
                return fail("login.template", "must hold no CR and no LF");
            }
            if !template.contains("{password}") {
                return fail("login.template", "must hold {password}");
            }
        }
        if let Some(secret) = &login.secret
            && secret.parse::<SecretRef>().is_err()
        {
            return fail(
                "login.secret",
                "must be a secret reference of the form env:<NAME>",
            );
        }
    }
    if let Some(keepalive) = &fields.keepalive {
        if keepalive.command.as_deref().is_some_and(has_line_break) {
            return fail("keepalive.command", "must hold no CR and no LF");
        }
        if keepalive
            .seconds
            .is_some_and(|seconds| !(1..=MAX_KEEPALIVE_SECONDS).contains(&seconds))
        {
            return fail("keepalive.seconds", "must be from 1 to 86400");
        }
    }
    check_ids(fields)
}

/// Checks that each entry has a string id, and that no id repeats among the rules and the
/// scripts of the layer.
fn check_ids(fields: &LayerFields) -> Result<(), (String, String)> {
    let mut seen = BTreeSet::new();
    for (array, entries) in [("rules", &fields.rules), ("scripts", &fields.scripts)] {
        for (index, entry) in entries.iter().enumerate() {
            let field = format!("{array}[{index}].id");
            let Some(id) = entry.get("id") else {
                return Err((field, "an entry must have an id".to_owned()));
            };
            let Some(id) = id.as_str() else {
                return Err((field, "must be a string".to_owned()));
            };
            if !seen.insert(id) {
                return Err((field, format!("the id {id} repeats in this file")));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::layer::{KeepaliveFields, LayerKey, LayerOrigin, LoginFields};

    fn layer(fields: LayerFields) -> Layer {
        Layer {
            key: LayerKey::Global,
            origin: LayerOrigin::File("global.toml".to_owned()),
            trust: Trust::Owner,
            extends: None,
            fields,
        }
    }

    #[test]
    fn keepalive_of_one_day_passes() {
        let fields = LayerFields {
            keepalive: Some(KeepaliveFields {
                command: Some("idle".to_owned()),
                seconds: Some(MAX_KEEPALIVE_SECONDS),
            }),
            ..LayerFields::default()
        };
        assert_eq!(check_layer(&layer(fields)), Ok(()));
    }

    #[test]
    fn secret_error_does_not_hold_the_value() {
        let fields = LayerFields {
            login: Some(LoginFields {
                secret: Some("hunter2".to_owned()),
                ..LoginFields::default()
            }),
            ..LayerFields::default()
        };
        let error = check_layer(&layer(fields)).unwrap_err().to_string();
        assert!(!error.contains("hunter2"), "{error}");
    }
}
