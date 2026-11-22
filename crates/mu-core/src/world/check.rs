//! The checks of one layer: its values, then its trust. Each layer passes them before the merge.
//! [`Login::new`](super::Login::new), [`Keepalive::new`](super::Keepalive::new) and
//! [`LineBreak::new`](super::LineBreak::new) run the same checks of their values.

use std::collections::BTreeSet;

use crate::framer::{Charset, has_line_break};
use crate::secret::SecretRef;

use super::error::{ConfigError, FieldError};
use super::layer::{Layer, LayerFields, Trust};

/// The highest value of `keepalive.seconds`: one day.
pub(crate) const MAX_KEEPALIVE_SECONDS: u32 = 86_400;

/// The highest value of `reconnect.initial`: one hour.
const MAX_FIRST_WAIT: u32 = 3_600;

/// The highest value of `reconnect.max`: one day.
const MAX_WAIT: u32 = 86_400;

/// The highest value of `reconnect.attempts`.
const MAX_ATTEMPTS: u32 = 10_000;

/// The highest count of characters of `line_break`.
const MAX_LINE_BREAK_CHARS: usize = 16;

/// Checks the values of `layer`, then its trust.
pub(crate) fn check_layer(layer: &Layer) -> Result<(), ConfigError> {
    check_fields(&layer.fields).map_err(|error| ConfigError::of_field(&layer.origin, &error))?;
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
        ("autoconnect", fields.autoconnect.is_some()),
        ("login.name", login.name.is_some()),
        ("login.secret", login.secret.is_some()),
        ("login.template", login.template.is_some()),
    ]
    .into_iter()
    .find_map(|(field, set)| set.then_some(field))
}

/// Checks the name and the template of a login.
pub(crate) fn check_login(name: Option<&str>, template: Option<&str>) -> Result<(), FieldError> {
    if name.is_some_and(has_line_break) {
        return Err(FieldError::new("login.name", "must hold no CR and no LF"));
    }
    if let Some(template) = template {
        if has_line_break(template) {
            return Err(FieldError::new(
                "login.template",
                "must hold no CR and no LF",
            ));
        }
        if !template.contains("{password}") {
            return Err(FieldError::new("login.template", "must hold {password}"));
        }
    }
    Ok(())
}

/// Checks the command and the seconds of a keepalive.
pub(crate) fn check_keepalive(
    command: Option<&str>,
    seconds: Option<u32>,
) -> Result<(), FieldError> {
    if command.is_some_and(has_line_break) {
        return Err(FieldError::new(
            "keepalive.command",
            "must hold no CR and no LF",
        ));
    }
    if seconds.is_some_and(|seconds| !(1..=MAX_KEEPALIVE_SECONDS).contains(&seconds)) {
        return Err(FieldError::new(
            "keepalive.seconds",
            "must be from 1 to 86400",
        ));
    }
    Ok(())
}

/// Checks the text of a line break: 1 to [`MAX_LINE_BREAK_CHARS`] characters, and no control
/// character.
pub(crate) fn check_line_break(line_break: &str) -> Result<(), FieldError> {
    let count = line_break.chars().count();
    if !(1..=MAX_LINE_BREAK_CHARS).contains(&count) {
        return Err(FieldError::new(
            "line_break",
            format!("must hold from 1 to {MAX_LINE_BREAK_CHARS} characters"),
        ));
    }
    if line_break.chars().any(char::is_control) {
        return Err(FieldError::new(
            "line_break",
            "must hold no control character",
        ));
    }
    Ok(())
}

/// Checks the ranges of the fields of a reconnect.
pub(crate) fn check_reconnect(
    initial: Option<u32>,
    max: Option<u32>,
    attempts: Option<u32>,
) -> Result<(), FieldError> {
    for (field, value, range, message) in [
        (
            "reconnect.initial",
            initial,
            1..=MAX_FIRST_WAIT,
            "must be from 1 to 3600",
        ),
        (
            "reconnect.max",
            max,
            1..=MAX_WAIT,
            "must be from 1 to 86400",
        ),
        (
            "reconnect.attempts",
            attempts,
            0..=MAX_ATTEMPTS,
            "must be from 0 to 10000",
        ),
    ] {
        if value.is_some_and(|value| !range.contains(&value)) {
            return Err(FieldError::new(field, message));
        }
    }
    Ok(())
}

/// Checks that the first wait of a reconnect is not greater than its cap.
pub(crate) fn check_first_wait(initial: u32, max: u32) -> Result<(), FieldError> {
    if initial > max {
        return Err(FieldError::new(
            "reconnect.initial",
            format!("is {initial}, which is greater than reconnect.max, {max}"),
        ));
    }
    Ok(())
}

/// Checks each value of `fields`, and returns the error of the first one that fails.
fn check_fields(fields: &LayerFields) -> Result<(), FieldError> {
    let fail = |field: &str, message: &str| Err(FieldError::new(field, message));
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
    if let Some(line_break) = &fields.line_break {
        check_line_break(line_break)?;
    }
    if let Some(login) = &fields.login {
        check_login(login.name.as_deref(), login.template.as_deref())?;
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
        check_keepalive(keepalive.command.as_deref(), keepalive.seconds)?;
    }
    if let Some(reconnect) = &fields.reconnect {
        check_reconnect(reconnect.initial, reconnect.max, reconnect.attempts)?;
    }
    check_ids(fields)
}

/// Checks that each entry has a string id, and that no id repeats among the rules and the
/// scripts of the layer.
fn check_ids(fields: &LayerFields) -> Result<(), FieldError> {
    let mut seen = BTreeSet::new();
    for (array, entries) in [("rules", &fields.rules), ("scripts", &fields.scripts)] {
        for (index, entry) in entries.iter().enumerate() {
            let field = format!("{array}[{index}].id");
            let Some(id) = entry.get("id") else {
                return Err(FieldError::new(field, "an entry must have an id"));
            };
            let Some(id) = id.as_str() else {
                return Err(FieldError::new(field, "must be a string"));
            };
            if !seen.insert(id) {
                return Err(FieldError::new(
                    field,
                    format!("the id {id} repeats in this file"),
                ));
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
                ..KeepaliveFields::default()
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
