//! The parse of one TOML file into a typed struct.
//!
//! An error names the file, the line and the column, and the path of the field. It never
//! holds the `Display` of a `toml` error, because that text quotes the line of the file, and
//! the line can hold a password. Under each table that the caller hides, or in the whole file,
//! it also drops the message, because a message of `serde` can quote the value.

use std::fmt;

use serde::de::DeserializeOwned;

use super::error::ConfigError;

/// The values whose errors hold no message of `serde`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hidden<'a> {
    /// The values under these tables.
    Tables(&'a [&'a str]),
    /// Every value of the file.
    All,
}

impl Hidden<'_> {
    /// Whether the field at `path` is hidden.
    fn covers(self, path: &str) -> bool {
        match self {
            Self::Tables(tables) => tables.iter().any(|table| under(path, table)),
            Self::All => true,
        }
    }
}

/// The tables of a world layer whose values can hold a password.
pub const LAYER_HIDDEN: Hidden<'static> = Hidden::Tables(&["login"]);

/// Parses `text`, the file at `place`, into `T`. For a field that `hidden` covers, an error
/// holds no message of `serde`.
///
/// # Errors
///
/// A [`ConfigError`] that names `place`, the line and the column. An error in a value also names
/// the path of the field.
pub fn parse<T: DeserializeOwned>(
    place: &impl fmt::Display,
    text: &str,
    hidden: Hidden<'_>,
) -> Result<T, ConfigError> {
    let deserializer = toml::Deserializer::parse(text)
        .map_err(|error| ConfigError::new(at(place, text, error.span()), error.message()))?;
    serde_path_to_error::deserialize(deserializer).map_err(|error| {
        let path = error.path().to_string();
        let place = at(place, text, error.inner().span());
        if path.is_empty() || path == "." {
            ConfigError::new(place, error.inner().message())
        } else if hidden.covers(&path) {
            ConfigError::field(place, path, "is not a valid field or value")
        } else {
            ConfigError::field(place, path, error.inner().message())
        }
    })
}

/// Whether `path` is `table` or a path inside it.
fn under(path: &str, table: &str) -> bool {
    path.strip_prefix(table)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('.') || rest.starts_with('['))
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
        parse::<File>(&"games/alpha.toml", text, LAYER_HIDDEN)
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
    fn all_hidden_drops_the_message_of_each_field() {
        let error = parse::<File>(&"server.toml", "port = \"hunter2\"\n", Hidden::All)
            .unwrap_err()
            .to_string();
        assert_eq!(
            error,
            "server.toml:1:8: port: is not a valid field or value"
        );
    }

    #[test]
    fn syntax_error_does_not_quote_the_line() {
        assert!(!error("[login]\nsecret = hunter2\n").contains("hunter2"));
    }
}
