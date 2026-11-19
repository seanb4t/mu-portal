//! Secrets: the password of each world, and where it comes from.

use std::ffi::OsString;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use secrecy::{ExposeSecret as _, SecretString};
use thiserror::Error;

/// The text that replaces a password in each event.
pub const MASK: &str = "****";

/// Where a secret comes from. The only form is `env:<NAME>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SecretRef {
    /// The environment variable of this name.
    Env(String),
}

/// A text that is not a secret reference.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("{0:?} is not a secret reference of the form env:<NAME>")]
pub struct BadSecretRef(String);

impl FromStr for SecretRef {
    type Err = BadSecretRef;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.strip_prefix("env:") {
            Some(name) if !name.is_empty() && !name.contains(['=', '\0']) => {
                Ok(Self::Env(name.to_owned()))
            }
            _ => Err(BadSecretRef(text.to_owned())),
        }
    }
}

impl fmt::Display for SecretRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Env(name) => write!(f, "env:{name}"),
        }
    }
}

/// A secret value. `Debug` prints no value, and the memory is zeroed on drop.
#[derive(Clone)]
pub struct Secret(SecretString);

impl Secret {
    /// Wraps a value.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(SecretString::from(value))
    }

    /// The value. Do not log it or write it into the journal.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }

    /// `text` with each occurrence of this secret replaced by [`MASK`].
    #[must_use]
    pub fn mask(&self, text: &str) -> String {
        mask(text, self.expose())
    }
}

/// Replaces each occurrence of `password` in `text` with [`MASK`]. If the result still holds
/// the password, the whole text becomes [`MASK`].
fn mask(text: &str, password: &str) -> String {
    if password.is_empty() {
        return text.to_owned();
    }
    let masked = text.replace(password, MASK);
    if masked.contains(password) {
        MASK.to_owned()
    } else {
        masked
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(****)")
    }
}

/// A secret that cannot be read or used. Each error names the reference, never the value.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SecretError {
    /// The variable is not set.
    #[error("{0} is not set")]
    Missing(SecretRef),
    /// The variable is empty.
    #[error("{0} is empty")]
    Empty(SecretRef),
    /// The variable is not Unicode.
    #[error("{0} is not Unicode")]
    NotUnicode(SecretRef),
    /// The mask `****` holds the password, so a masked text can still hold it.
    #[error("the password in {0} is one to four stars, which the mask cannot hide")]
    Unmaskable(SecretRef),
    /// The password holds a CR or an LF, which splits the login line.
    #[error("the password in {0} holds a line break")]
    LineBreak(SecretRef),
}

/// A source of secrets.
pub trait SecretStore {
    /// The secret of `reference`.
    ///
    /// # Errors
    ///
    /// A [`SecretError`] that names the reference.
    fn get(&self, reference: &SecretRef) -> Result<Secret, SecretError>;
}

/// Reads `env:<NAME>` from the environment of the process.
pub struct EnvSecretStore<F = fn(&str) -> Option<OsString>> {
    lookup: F,
}

impl EnvSecretStore {
    /// A store that reads the environment of the process.
    #[must_use]
    pub fn new() -> Self {
        Self {
            lookup: |name| std::env::var_os(name),
        }
    }
}

impl Default for EnvSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Fn(&str) -> Option<OsString>> EnvSecretStore<F> {
    /// A store that reads each variable through `lookup`.
    pub fn with_lookup(lookup: F) -> Self {
        Self { lookup }
    }
}

impl<F: Fn(&str) -> Option<OsString>> SecretStore for EnvSecretStore<F> {
    fn get(&self, reference: &SecretRef) -> Result<Secret, SecretError> {
        let SecretRef::Env(name) = reference;
        let value = (self.lookup)(name).ok_or_else(|| SecretError::Missing(reference.clone()))?;
        let value = value
            .into_string()
            .map_err(|_| SecretError::NotUnicode(reference.clone()))?;
        if value.is_empty() {
            return Err(SecretError::Empty(reference.clone()));
        }
        Ok(Secret::new(value))
    }
}

/// The password of `reference`, refused if the mask cannot hide it or it splits the login line.
///
/// # Errors
///
/// A [`SecretError`] that names the reference.
pub fn load_password(
    store: &dyn SecretStore,
    reference: &SecretRef,
) -> Result<Secret, SecretError> {
    let secret = store.get(reference)?;
    check_password(secret.expose(), reference)?;
    Ok(secret)
}

/// Where a session reads the password of its login.
#[derive(Clone)]
pub enum SecretSource {
    /// A password read once, at the start of the server: a fake world.
    Fixed(Secret),
    /// A password read through [`load_password`] at each connect: a configured world.
    Store(SecretRef, Arc<dyn SecretStore + Send + Sync>),
}

impl SecretSource {
    /// The password.
    ///
    /// # Errors
    ///
    /// A [`SecretError`] of [`load_password`] for [`SecretSource::Store`].
    pub fn read(&self) -> Result<Secret, SecretError> {
        match self {
            Self::Fixed(secret) => Ok(secret.clone()),
            Self::Store(reference, store) => load_password(store.as_ref(), reference),
        }
    }
}

impl fmt::Debug for SecretSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed(secret) => f.debug_tuple("Fixed").field(secret).finish(),
            Self::Store(reference, _) => f.debug_tuple("Store").field(reference).finish(),
        }
    }
}

fn check_password(password: &str, reference: &SecretRef) -> Result<(), SecretError> {
    if MASK.contains(password) {
        return Err(SecretError::Unmaskable(reference.clone()));
    }
    if crate::framer::has_line_break(password) {
        return Err(SecretError::LineBreak(reference.clone()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn env(pairs: &[(&str, &str)]) -> EnvSecretStore<impl Fn(&str) -> Option<OsString>> {
        let map: HashMap<String, OsString> = pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), OsString::from(value)))
            .collect();
        EnvSecretStore::with_lookup(move |name| map.get(name).cloned())
    }

    fn pw() -> SecretRef {
        "env:ALPHA_PW".parse().unwrap()
    }

    #[test]
    fn env_secret_is_read() {
        let secret = env(&[("ALPHA_PW", "s3cret-pw")]).get(&pw()).unwrap();
        assert_eq!(secret.expose(), "s3cret-pw");
    }

    #[test]
    fn missing_env_secret_names_the_variable() {
        let error = env(&[]).get(&pw()).unwrap_err();
        assert_eq!(error, SecretError::Missing(pw()));
        assert!(error.to_string().contains("ALPHA_PW"));
    }

    #[test]
    fn empty_env_secret_is_refused() {
        assert_eq!(
            env(&[("ALPHA_PW", "")]).get(&pw()).unwrap_err(),
            SecretError::Empty(pw())
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_env_secret_is_refused() {
        use std::os::unix::ffi::OsStringExt as _;
        let store = EnvSecretStore::with_lookup(|_| Some(OsString::from_vec(vec![0x66, 0xff])));
        assert_eq!(store.get(&pw()).unwrap_err(), SecretError::NotUnicode(pw()));
    }

    #[test]
    fn store_source_reads_at_each_call() {
        let values = std::sync::Mutex::new(vec![None, Some(OsString::from("s3cret-pw"))]);
        let store = EnvSecretStore::with_lookup(move |_| values.lock().unwrap().pop().flatten());
        let source = SecretSource::Store(pw(), Arc::new(store));
        assert_eq!(source.read().unwrap().expose(), "s3cret-pw");
        assert_eq!(source.read().unwrap_err(), SecretError::Missing(pw()));
    }

    #[test]
    fn source_debug_hides_value() {
        let fixed = SecretSource::Fixed(Secret::new("s3cret-pw".to_owned()));
        let store = SecretSource::Store(pw(), Arc::new(env(&[("ALPHA_PW", "s3cret-pw")])));
        for source in [fixed, store] {
            assert!(!format!("{source:?}").contains("s3cret-pw"));
        }
    }

    #[test]
    fn secret_debug_hides_value() {
        let secret = Secret::new("s3cret-pw".to_owned());
        assert!(!format!("{secret:?}").contains("s3cret-pw"));
    }

    fn load(value: &str) -> Result<Secret, SecretError> {
        load_password(&env(&[("ALPHA_PW", value)]), &pw())
    }

    #[test]
    fn star_passwords_at_the_edge() {
        for refused in ["*", "**", "***", "****"] {
            assert_eq!(
                load(refused).unwrap_err(),
                SecretError::Unmaskable(pw()),
                "{refused}"
            );
        }
        assert!(load("*****").is_ok());
    }

    #[test]
    fn password_with_cr_is_refused() {
        assert_eq!(load("a\rb").unwrap_err(), SecretError::LineBreak(pw()));
    }

    #[test]
    fn password_with_lf_is_refused() {
        assert_eq!(load("a\nb").unwrap_err(), SecretError::LineBreak(pw()));
    }

    #[test]
    fn password_with_star_masks_whole_text() {
        assert_eq!(mask("aa*", "a*"), MASK);
        assert_eq!(mask("my pw is s3cret-pw", "s3cret-pw"), "my pw is ****");
    }

    proptest::proptest! {
        #[test]
        fn mask_never_leaves_the_password(
            text in ".{0,40}",
            password in "[a*\\-b]{1,6}|.{1,8}",
            noise in proptest::collection::vec(0_usize..41, 0..4),
        ) {
            proptest::prop_assume!(check_password(&password, &pw()).is_ok());
            let mut text: String = text;
            for at in noise {
                let at = text.char_indices().map(|(i, _)| i).chain([text.len()]).nth(at).unwrap_or(text.len());
                text.insert_str(at, &password);
            }
            proptest::prop_assert!(!mask(&text, &password).contains(&password));
        }
    }

    #[test]
    fn secret_refs_parse() {
        assert_eq!(
            "env:ALPHA_PW".parse::<SecretRef>().unwrap().to_string(),
            "env:ALPHA_PW"
        );
        for bad in ["ALPHA_PW", "env:", "file:x", "env:A=B"] {
            assert!(bad.parse::<SecretRef>().is_err(), "{bad:?}");
        }
    }
}
