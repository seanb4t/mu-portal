//! The resolved configuration of a world, which the sessions read.

use crate::id::WorldId;
use crate::secret::SecretRef;

/// The login template of every world in this change.
pub const DEFAULT_LOGIN_TEMPLATE: &str = "connect {login} {password}";

/// How a world logs in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Login {
    /// The line to send, with `{login}` and `{password}` in it.
    pub template: String,
    /// The login name.
    pub name: String,
    /// Where the password comes from.
    pub secret: SecretRef,
}

impl Login {
    /// The login line for `password`.
    #[must_use]
    pub fn line(&self, password: &str) -> String {
        self.template
            .replace("{login}", &self.name)
            .replace("{password}", password)
    }
}

/// One world after the merge of its configuration layers. The `--fake-world` flags build it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedWorld {
    /// The id of the world.
    pub id: WorldId,
    /// The login, or none for a world that logs in by hand.
    pub login: Option<Login>,
}

impl ResolvedWorld {
    /// A world that logs in with the default template and its character part as the name.
    #[must_use]
    pub fn with_default_login(id: WorldId, secret: SecretRef) -> Self {
        let login = Login {
            template: DEFAULT_LOGIN_TEMPLATE.to_owned(),
            name: id.character().to_owned(),
            secret,
        };
        Self {
            id,
            login: Some(login),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_login_uses_the_character_part() {
        let world = ResolvedWorld::with_default_login(
            "alpha/aria".parse().unwrap(),
            "env:ALPHA_PW".parse().unwrap(),
        );
        assert_eq!(world.login.unwrap().line("pw"), "connect aria pw");
    }
}
