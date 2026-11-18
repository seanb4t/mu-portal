//! Commands: the only way to act on a world.

use crate::id::WorldId;

/// Who made a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Origin {
    /// A client, through the API.
    Player,
    /// A hook or a handler.
    Script,
    /// The server itself, for example at its start.
    System,
}

/// An action on one world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Sends one line to the game.
    Send {
        /// The world.
        world: WorldId,
        /// The line, with no line end.
        text: String,
    },
    /// Connects the world.
    Connect {
        /// The world.
        world: WorldId,
    },
    /// Disconnects the world.
    Disconnect {
        /// The world.
        world: WorldId,
    },
    /// Sends the login line of the world.
    Login {
        /// The world.
        world: WorldId,
    },
    /// Records a line for the player, and sends nothing to the game.
    Echo {
        /// The world.
        world: WorldId,
        /// The line.
        text: String,
    },
    /// Records a custom event.
    Emit {
        /// The world.
        world: WorldId,
        /// The text of the event.
        text: String,
    },
}

impl Command {
    /// The world that the command acts on.
    #[must_use]
    pub fn world(&self) -> &WorldId {
        match self {
            Self::Send { world, .. }
            | Self::Connect { world }
            | Self::Disconnect { world }
            | Self::Login { world }
            | Self::Echo { world, .. }
            | Self::Emit { world, .. } => world,
        }
    }

    /// The lower-case name of the variant, for the telemetry. It never holds the text.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Send { .. } => "send",
            Self::Connect { .. } => "connect",
            Self::Disconnect { .. } => "disconnect",
            Self::Login { .. } => "login",
            Self::Echo { .. } => "echo",
            Self::Emit { .. } => "emit",
        }
    }
}

impl Origin {
    /// Every origin.
    pub const ALL: [Self; 3] = [Self::Player, Self::Script, Self::System];

    /// The lower-case name of the origin, for the telemetry.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Player => "player",
            Self::Script => "script",
            Self::System => "system",
        }
    }
}

/// A command with its origin and its depth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    /// The command.
    pub command: Command,
    /// Who made it.
    pub origin: Origin,
    /// The length of the chain of handlers that made it. The API and the server start give 0.
    pub depth: u8,
}

impl Envelope {
    /// A command from a client, at the depth 0.
    #[must_use]
    pub fn player(command: Command) -> Self {
        Self {
            command,
            origin: Origin::Player,
            depth: 0,
        }
    }

    /// A command from the server itself, at the depth 0.
    #[must_use]
    pub fn system(command: Command) -> Self {
        Self {
            command,
            origin: Origin::System,
            depth: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_command_has_a_name() {
        let world: WorldId = "alpha/aria".parse().unwrap();
        let text = String::new();
        let commands = [
            Command::Send {
                world: world.clone(),
                text: text.clone(),
            },
            Command::Connect {
                world: world.clone(),
            },
            Command::Disconnect {
                world: world.clone(),
            },
            Command::Login {
                world: world.clone(),
            },
            Command::Echo {
                world: world.clone(),
                text: text.clone(),
            },
            Command::Emit { world, text },
        ];
        // A new variant of `Command` belongs in `commands` too.
        let names: Vec<_> = commands.iter().map(Command::name).collect();
        assert_eq!(
            names,
            ["send", "connect", "disconnect", "login", "echo", "emit"]
        );
    }

    #[test]
    fn each_origin_has_a_name() {
        let names: Vec<_> = Origin::ALL.into_iter().map(Origin::name).collect();
        assert_eq!(names, ["player", "script", "system"]);
    }
}
