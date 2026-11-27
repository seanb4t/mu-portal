//! Events of the journal, and the rule that gives each event its source.

use mu_config::stream::Streams;
use std::fmt;
use std::time::SystemTime;
use ulid::Ulid;

use crate::command::Origin;
use crate::id::WorldId;

/// The sequence number of an event. The journal starts at 1 and has no gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Seq(pub u64);

impl Seq {
    /// The number after this one.
    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl fmt::Display for Seq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// What an event records.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventKind {
    /// The world connected.
    Connected,
    /// The world disconnected.
    Disconnected,
    /// A line from the game.
    Line,
    /// A prompt from the game, with no line end.
    Prompt,
    /// A message of a side protocol from the game.
    Message,
    /// A line sent to the game.
    Input,
    /// A line shown to the player and not sent to the game.
    Echo,
    /// The login line, with the password masked.
    Login,
    /// A custom event of a script.
    Custom,
    /// An error of a hook, a handler or a limit of the engine.
    ScriptError,
}

impl EventKind {
    /// Every kind, in the order of the schema.
    pub const ALL: [Self; 10] = [
        Self::Connected,
        Self::Disconnected,
        Self::Line,
        Self::Prompt,
        Self::Message,
        Self::Input,
        Self::Echo,
        Self::Login,
        Self::Custom,
        Self::ScriptError,
    ];

    /// The snake-case name of the kind, for the telemetry.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::Disconnected => "disconnected",
            Self::Line => "line",
            Self::Prompt => "prompt",
            Self::Message => "message",
            Self::Input => "input",
            Self::Echo => "echo",
            Self::Login => "login",
            Self::Custom => "custom",
            Self::ScriptError => "script_error",
        }
    }
}

/// Who an event comes from. Each event has exactly one source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    /// The game.
    Game,
    /// The player, through a client.
    Player,
    /// A hook or a handler.
    Script,
    /// The server itself.
    System,
    /// The login with a secret.
    Credential,
}

impl Source {
    /// The source of an event of `kind` that a command of `origin` caused.
    ///
    /// Only an input and an echo take their source from the origin.
    #[must_use]
    pub fn of(kind: EventKind, origin: Origin) -> Self {
        match kind {
            EventKind::Line | EventKind::Prompt | EventKind::Message => Self::Game,
            EventKind::Connected | EventKind::Disconnected => Self::System,
            EventKind::Login => Self::Credential,
            EventKind::Custom | EventKind::ScriptError => Self::Script,
            EventKind::Input | EventKind::Echo => match origin {
                Origin::Player => Self::Player,
                Origin::Script => Self::Script,
                Origin::System => Self::System,
            },
        }
    }
}

/// An event that the journal has not numbered yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewEvent {
    /// The world of the event.
    pub world: WorldId,
    /// What the event records.
    pub kind: EventKind,
    /// Who the event comes from.
    pub source: Source,
    /// The text, with the password of the world masked.
    pub text: String,
    /// The depth of the command chain that made the event. A line of the game has 0.
    pub depth: u8,
    /// The streams of the event: for a line from the game, the streams that a hook set, else the
    /// streams of the rules; [`Streams::main`] for each other event.
    pub streams: Streams,
}

/// An event in the journal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// The sequence number.
    pub seq: Seq,
    /// The id, which the journal makes when it numbers the event. It is unique also across a
    /// restart of the server.
    pub id: Ulid,
    /// The world of the event.
    pub world: WorldId,
    /// What the event records.
    pub kind: EventKind,
    /// Who the event comes from.
    pub source: Source,
    /// The text, with the password of the world masked.
    pub text: String,
    /// When the journal recorded the event.
    pub time: SystemTime,
    /// The depth of the command chain that made the event. The schema does not carry it.
    pub depth: u8,
    /// The streams of the event.
    pub streams: Streams,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_source_follows_kind() {
        let fixed = [
            (EventKind::Line, Source::Game),
            (EventKind::Prompt, Source::Game),
            (EventKind::Message, Source::Game),
            (EventKind::Connected, Source::System),
            (EventKind::Disconnected, Source::System),
            (EventKind::Login, Source::Credential),
            (EventKind::Custom, Source::Script),
            (EventKind::ScriptError, Source::Script),
        ];
        for origin in [Origin::Player, Origin::Script, Origin::System] {
            for (kind, source) in fixed {
                assert_eq!(Source::of(kind, origin), source, "{kind:?} from {origin:?}");
            }
        }
        for kind in [EventKind::Input, EventKind::Echo] {
            assert_eq!(Source::of(kind, Origin::Player), Source::Player);
            assert_eq!(Source::of(kind, Origin::Script), Source::Script);
            assert_eq!(Source::of(kind, Origin::System), Source::System);
        }
        assert_eq!(fixed.len() + 2, EventKind::ALL.len());
    }

    #[test]
    fn each_kind_has_a_name() {
        let names: Vec<_> = EventKind::ALL.into_iter().map(EventKind::name).collect();
        assert_eq!(
            names,
            [
                "connected",
                "disconnected",
                "line",
                "prompt",
                "message",
                "input",
                "echo",
                "login",
                "custom",
                "script_error",
            ]
        );
    }
}
