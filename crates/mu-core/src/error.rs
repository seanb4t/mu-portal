//! Errors that the engine returns to the API.

use thiserror::Error;

use crate::event::Seq;
use crate::id::WorldId;

/// The error of a call on the `Portal` or the journal.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum PortalError {
    /// No world has this id.
    #[error("unknown world {0}")]
    UnknownWorld(WorldId),
    /// The world is connecting or disconnected.
    #[error("{0} is not connected")]
    NotConnected(WorldId),
    /// The world has no transport yet, so it cannot connect.
    #[error("{0} has no transport, so it cannot connect yet")]
    NoTransport(WorldId),
    /// The text holds a CR or an LF.
    #[error("the text holds a line break")]
    LineBreak,
    /// A subscribe starts after the next sequence number.
    #[error("sequence {from} is past the end of the journal at {last}")]
    PastEnd {
        /// The first number that the subscriber asked for.
        from: Seq,
        /// The last number in the journal.
        last: Seq,
    },
    /// The engine stopped.
    #[error("the server is stopping")]
    Stopped,
}
