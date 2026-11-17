//! Conversion between the types of the engine and the types of the schema.

use std::time::UNIX_EPOCH;

use connectrpc::ConnectError;
use mu_core::{Event, EventKind, PortalError, Source, WorldState};
use mu_proto as proto;

/// The code of each engine error. One function holds the whole map.
#[must_use]
pub fn status(error: &PortalError) -> ConnectError {
    let message = error.to_string();
    match error {
        PortalError::UnknownWorld(_) => ConnectError::not_found(message),
        PortalError::NotConnected(_) | PortalError::NoTransport(_) | PortalError::NoPassword(_) => {
            ConnectError::failed_precondition(message)
        }
        PortalError::LineBreak => ConnectError::invalid_argument(message),
        PortalError::PastEnd { .. } => ConnectError::out_of_range(message),
        PortalError::Stopped => ConnectError::unavailable(message),
    }
}

fn kind(kind: EventKind) -> proto::EventKind {
    match kind {
        EventKind::Connected => proto::EventKind::EVENT_KIND_CONNECTED,
        EventKind::Disconnected => proto::EventKind::EVENT_KIND_DISCONNECTED,
        EventKind::Line => proto::EventKind::EVENT_KIND_LINE,
        EventKind::Prompt => proto::EventKind::EVENT_KIND_PROMPT,
        EventKind::Message => proto::EventKind::EVENT_KIND_MESSAGE,
        EventKind::Input => proto::EventKind::EVENT_KIND_INPUT,
        EventKind::Echo => proto::EventKind::EVENT_KIND_ECHO,
        EventKind::Login => proto::EventKind::EVENT_KIND_LOGIN,
        EventKind::Custom => proto::EventKind::EVENT_KIND_CUSTOM,
        EventKind::ScriptError => proto::EventKind::EVENT_KIND_SCRIPT_ERROR,
    }
}

fn source(source: Source) -> proto::Source {
    match source {
        Source::Game => proto::Source::SOURCE_GAME,
        Source::Player => proto::Source::SOURCE_PLAYER,
        Source::Script => proto::Source::SOURCE_SCRIPT,
        Source::System => proto::Source::SOURCE_SYSTEM,
        Source::Credential => proto::Source::SOURCE_CREDENTIAL,
    }
}

/// The schema value of a world state.
#[must_use]
pub fn state(state: WorldState) -> proto::WorldState {
    match state {
        WorldState::Connecting => proto::WorldState::WORLD_STATE_CONNECTING,
        WorldState::Connected => proto::WorldState::WORLD_STATE_CONNECTED,
        WorldState::Disconnected => proto::WorldState::WORLD_STATE_DISCONNECTED,
    }
}

/// The schema message of an event.
#[must_use]
pub fn event(event: &Event) -> proto::Event {
    let time_ms = event.time.duration_since(UNIX_EPOCH).map_or(0, |since| {
        i64::try_from(since.as_millis()).unwrap_or(i64::MAX)
    });
    proto::Event {
        seq: event.seq.0,
        world: event.world.to_string(),
        kind: kind(event.kind).into(),
        source: source(event.source).into(),
        text: event.text.clone(),
        time_ms,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use connectrpc::ErrorCode;
    use mu_core::{Seq, WorldId};

    use super::*;

    #[test]
    fn portal_errors_map_to_codes() {
        let world: WorldId = "alpha/aria".parse().unwrap();
        let cases = [
            (
                PortalError::UnknownWorld(world.clone()),
                ErrorCode::NotFound,
            ),
            (
                PortalError::NotConnected(world.clone()),
                ErrorCode::FailedPrecondition,
            ),
            (
                PortalError::NoTransport(world.clone()),
                ErrorCode::FailedPrecondition,
            ),
            (
                PortalError::NoPassword(world),
                ErrorCode::FailedPrecondition,
            ),
            (PortalError::LineBreak, ErrorCode::InvalidArgument),
            (
                PortalError::PastEnd {
                    from: Seq(8),
                    last: Seq(6),
                },
                ErrorCode::OutOfRange,
            ),
            (PortalError::Stopped, ErrorCode::Unavailable),
        ];
        for (error, code) in cases {
            assert_eq!(status(&error).code, code, "{error:?}");
        }
    }

    #[test]
    fn each_kind_and_source_has_a_schema_value() {
        let kinds = [
            (EventKind::Connected, proto::EventKind::EVENT_KIND_CONNECTED),
            (
                EventKind::Disconnected,
                proto::EventKind::EVENT_KIND_DISCONNECTED,
            ),
            (EventKind::Line, proto::EventKind::EVENT_KIND_LINE),
            (EventKind::Prompt, proto::EventKind::EVENT_KIND_PROMPT),
            (EventKind::Message, proto::EventKind::EVENT_KIND_MESSAGE),
            (EventKind::Input, proto::EventKind::EVENT_KIND_INPUT),
            (EventKind::Echo, proto::EventKind::EVENT_KIND_ECHO),
            (EventKind::Login, proto::EventKind::EVENT_KIND_LOGIN),
            (EventKind::Custom, proto::EventKind::EVENT_KIND_CUSTOM),
            (
                EventKind::ScriptError,
                proto::EventKind::EVENT_KIND_SCRIPT_ERROR,
            ),
        ];
        assert_eq!(kinds.len(), EventKind::ALL.len());
        for (each, value) in kinds {
            assert_eq!(kind(each), value, "{each:?}");
        }
        let sources = [
            (Source::Game, proto::Source::SOURCE_GAME),
            (Source::Player, proto::Source::SOURCE_PLAYER),
            (Source::Script, proto::Source::SOURCE_SCRIPT),
            (Source::System, proto::Source::SOURCE_SYSTEM),
            (Source::Credential, proto::Source::SOURCE_CREDENTIAL),
        ];
        for (each, value) in sources {
            assert_eq!(source(each), value, "{each:?}");
        }
    }
}
