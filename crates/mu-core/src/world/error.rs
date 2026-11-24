//! The error of the configuration.

use std::fmt;

use thiserror::Error;

/// An error in the configuration: the place, such as a file or a world, and what is wrong.
///
/// The text never holds the value of a field under `login`, or a line of a file.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("{place}: {message}")]
pub struct ConfigError {
    place: String,
    message: String,
}

impl ConfigError {
    /// An error at `place`.
    pub(crate) fn new(place: impl fmt::Display, message: impl Into<String>) -> Self {
        Self {
            place: place.to_string(),
            message: message.into(),
        }
    }

    /// An error in the field `field` at `place`.
    pub(crate) fn field(
        place: impl fmt::Display,
        field: impl fmt::Display,
        message: impl fmt::Display,
    ) -> Self {
        Self::new(place, format!("{field}: {message}"))
    }

    /// An error in the field of `error` at `place`.
    pub(crate) fn of_field(place: impl fmt::Display, error: &FieldError) -> Self {
        Self::field(place, &error.field, &error.message)
    }

    /// The place of the error, such as `games/alpha.toml:3:1` or `alpha/aria`.
    #[must_use]
    pub fn place(&self) -> &str {
        &self.place
    }

    /// The file path of the place, without the line and the column.
    #[must_use]
    pub fn path(&self) -> &str {
        self.place.split(':').next().unwrap_or_default()
    }
}

/// A value that breaks a rule of its field: the path of the field, such as `login.template`,
/// and what is wrong. The text never holds a value under `login`.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("{field}: {message}")]
pub struct FieldError {
    field: String,
    message: String,
}

impl FieldError {
    /// An error in the field `field`.
    pub(crate) fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }

    /// The path of the field.
    #[must_use]
    pub fn field(&self) -> &str {
        &self.field
    }
}
