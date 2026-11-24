//! The names of a client: its kind, its OS and its device.

use std::fmt;

/// A name that a check refuses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameError {
    /// What the name is for: `kind`, `os` or `device`.
    pub what: &'static str,
    /// The name.
    pub name: String,
    /// What is wrong with it.
    pub reason: String,
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}: {}", self.what, self.name, self.reason)
    }
}

impl std::error::Error for NameError {}

/// `names` as a list in text, such as `a`, `a and b` or `a, b and c`.
#[must_use]
pub fn listed<S: AsRef<str>>(names: &[S]) -> String {
    match names {
        [] => String::new(),
        [one] => one.as_ref().to_owned(),
        [rest @ .., last] => {
            let rest: Vec<&str> = rest.iter().map(AsRef::as_ref).collect();
            format!("{} and {}", rest.join(", "), last.as_ref())
        }
    }
}

/// The kind of a client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// The terminal client.
    Tui,
}

impl Kind {
    /// Every kind.
    pub const ALL: [Self; 1] = [Self::Tui];

    /// The name of the kind, which is also the name of its layer file.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Tui => "tui",
        }
    }

    /// The kind named `name`.
    ///
    /// # Errors
    ///
    /// A [`NameError`] that lists the kinds.
    pub fn parse(name: &str) -> Result<Self, NameError> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.name() == name)
            .ok_or_else(|| NameError {
                what: "kind",
                name: name.to_owned(),
                reason: format!(
                    "is no kind: the kinds are {}",
                    listed(&Self::ALL.map(Self::name))
                ),
            })
    }
}

/// The OS of a client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Os {
    /// macOS.
    Macos,
    /// Linux.
    Linux,
}

impl Os {
    /// Every OS.
    pub const ALL: [Self; 2] = [Self::Macos, Self::Linux];

    /// The name of the OS, which is also the name of its layer file.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Macos => "macos",
            Self::Linux => "linux",
        }
    }

    /// The OS named `name`.
    ///
    /// # Errors
    ///
    /// A [`NameError`] that lists the OS names.
    pub fn parse(name: &str) -> Result<Self, NameError> {
        Self::ALL
            .into_iter()
            .find(|os| os.name() == name)
            .ok_or_else(|| NameError {
                what: "os",
                name: name.to_owned(),
                reason: format!(
                    "is no OS: the OS names are {}",
                    listed(&Self::ALL.map(Self::name))
                ),
            })
    }

    /// The OS of this machine.
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Linux
        }
    }
}

/// The longest device name, in characters.
pub const DEVICE_MAX: usize = 64;

/// The name of a device: 1 to [`DEVICE_MAX`] characters from `a` to `z`, `0` to `9`, `-`, `_`
/// and `.`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DeviceName(String);

impl DeviceName {
    /// Checks `name`.
    ///
    /// # Errors
    ///
    /// A [`NameError`] that names the device name and the rule.
    pub fn new(name: &str) -> Result<Self, NameError> {
        let good = (1..=DEVICE_MAX).contains(&name.len())
            && name
                .bytes()
                .all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.'));
        if good {
            Ok(Self(name.to_owned()))
        } else {
            Err(NameError {
                what: "device",
                name: name.to_owned(),
                reason: format!(
                    "a device name has 1 to {DEVICE_MAX} characters from a to z, 0 to 9, -, _ and ."
                ),
            })
        }
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DeviceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
