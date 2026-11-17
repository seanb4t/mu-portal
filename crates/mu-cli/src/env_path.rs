//! The parse of a path from a flag or a variable, and the lookup of an XDG base directory.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// A path from a flag or a variable. An empty value is none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathValue(Option<PathBuf>);

impl PathValue {
    /// The path, or none for an empty value.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.0.as_deref()
    }
}

/// Turns an empty value into none, so that `clap` does not refuse it.
#[expect(
    clippy::unnecessary_wraps,
    reason = "clap takes a value parser that returns a Result"
)]
pub(crate) fn parse_path(value: &str) -> Result<PathValue, std::convert::Infallible> {
    Ok(PathValue((!value.is_empty()).then(|| PathBuf::from(value))))
}

/// The base directory that the variable value `value` names. An unset, empty or relative value
/// counts as none, as the XDG Base Directory specification asks.
#[must_use]
pub fn xdg_base(value: Option<OsString>) -> Option<PathBuf> {
    value.map(PathBuf::from).filter(|dir| dir.is_absolute())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_value_is_none() {
        assert_eq!(parse_path("").unwrap().path(), None);
    }

    #[test]
    fn relative_base_is_none() {
        assert_eq!(xdg_base(Some("xdg".into())), None);
    }
}
