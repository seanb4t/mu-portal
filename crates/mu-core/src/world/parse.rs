//! The parse of one TOML file into a typed struct.
//!
//! An error names the file, the line and the column, and the path of the field. It never
//! holds the `Display` of a `toml` error, because that text quotes the line of the file, and
//! the line can hold a password. Under `login` it also drops the message, because a message of
//! `serde` can quote the value.

use std::fmt;

use serde::de::DeserializeOwned;

use super::error::ConfigError;

/// Parses `text`, the file at `place`, into `T`.
pub(crate) fn parse<T: DeserializeOwned>(
    place: &impl fmt::Display,
    text: &str,
) -> Result<T, ConfigError> {
    let deserializer = toml::Deserializer::parse(text)
        .map_err(|error| ConfigError::new(at(place, text, error.span()), error.message()))?;
    serde_path_to_error::deserialize(deserializer).map_err(|error| {
        let path = error.path().to_string();
        let place = at(place, text, error.inner().span());
        if path.is_empty() || path == "." {
            ConfigError::new(place, error.inner().message())
        } else if path == "login" || path.starts_with("login.") {
            ConfigError::field(place, path, "is not a valid field or value")
        } else {
            ConfigError::field(place, path, error.inner().message())
        }
    })
}

/// `place` with the line and the column of the start of `span` in `text`.
fn at(place: &impl fmt::Display, text: &str, span: Option<std::ops::Range<usize>>) -> String {
    let Some(start) = span.map(|span| span.start.min(text.len())) else {
        return place.to_string();
    };
    let before = text.get(..start).unwrap_or(text);
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
    format!("{place}:{line}:{column}")
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[expect(dead_code, reason = "the tests read only the errors")]
    struct File {
        port: Option<u16>,
        login: Option<Login>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[expect(dead_code, reason = "the tests read only the errors")]
    struct Login {
        secret: Option<String>,
    }

    fn error(text: &str) -> String {
        parse::<File>(&"games/alpha.toml", text)
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn error_names_the_line_and_the_field() {
        assert!(error("\nport = 70000\n").starts_with("games/alpha.toml:2:8: port: "));
    }

    #[test]
    fn error_under_login_drops_the_message() {
        assert_eq!(
            error("[login]\nsecret = 5\n"),
            "games/alpha.toml:2:10: login.secret: is not a valid field or value"
        );
    }

    #[test]
    fn syntax_error_does_not_quote_the_line() {
        assert!(!error("[login]\nsecret = hunter2\n").contains("hunter2"));
    }
}
