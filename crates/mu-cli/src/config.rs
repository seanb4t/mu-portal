//! The configuration directory of the worlds, and its resolution.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::Args;
use mu_core::ResolvedWorld;
use mu_core::world::{ConfigError, TomlDirSource, resolve};

use crate::env_path::{PathValue, parse_path, xdg_base};

/// The variable that names the configuration directory.
pub const CONFIG_ENV: &str = "MU_CONFIG";

/// The configuration directory: `--config`, then `MU_CONFIG`, then `mu` under
/// `XDG_CONFIG_HOME`, then `.config/mu` under `HOME`.
#[derive(Clone, Debug, Default, Args)]
pub struct ConfigArgs {
    /// The configuration directory of the worlds
    #[arg(long, env = CONFIG_ENV, value_name = "DIR", value_parser = parse_path)]
    config: Option<PathValue>,
}

impl ConfigArgs {
    /// The directory, with `config_home` as the value of `XDG_CONFIG_HOME` and `home` as the
    /// value of `HOME`. An empty or relative `XDG_CONFIG_HOME` or `HOME` counts as unset. With
    /// none of them, there is no directory.
    #[must_use]
    pub fn dir(&self, config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
        if let Some(dir) = self.config.as_ref().and_then(PathValue::path) {
            return Some(dir.to_path_buf());
        }
        xdg_base(config_home)
            .map(|base| base.join("mu"))
            .or_else(|| xdg_base(home).map(|home| home.join(".config").join("mu")))
    }

    /// The resolved worlds of the directory, read from the environment of the process. With no
    /// directory, or a missing one, there are no worlds. If `--config` or `MU_CONFIG` names a
    /// missing directory, a warning of `verb` on stderr names it.
    ///
    /// # Errors
    ///
    /// A [`ConfigError`] that names the file or the world, and the field.
    pub fn worlds(&self, verb: &str) -> Result<Vec<ResolvedWorld>, ConfigError> {
        let dir = self.dir(
            std::env::var_os("XDG_CONFIG_HOME"),
            std::env::var_os("HOME"),
        );
        if let Some(named) = self.config.as_ref().and_then(PathValue::path)
            && matches!(named.try_exists(), Ok(false))
        {
            crate::warn(
                verb,
                &format_args!(
                    "{}: the configuration directory does not exist, so no worlds are configured",
                    named.display()
                ),
            );
        }
        match dir {
            Some(dir) => resolve_dir(&dir),
            None => Ok(Vec::new()),
        }
    }
}

/// The resolved worlds of the directory `dir`.
///
/// # Errors
///
/// A [`ConfigError`] that names the file or the world, and the field.
pub fn resolve_dir(dir: &Path) -> Result<Vec<ResolvedWorld>, ConfigError> {
    resolve(&[&TomlDirSource::new(dir)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_gives_the_default() {
        assert_eq!(
            ConfigArgs::default().dir(None, Some("/home/aria".into())),
            Some(PathBuf::from("/home/aria/.config/mu"))
        );
    }

    #[test]
    fn relative_home_gives_none() {
        assert_eq!(ConfigArgs::default().dir(None, Some("home".into())), None);
    }
}
