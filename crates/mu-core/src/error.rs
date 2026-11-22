//! Errors that the engine returns to the API.

use thiserror::Error;

use crate::event::Seq;
use crate::id::WorldId;

/// Why a text with a line break cannot go to its world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum LineBreakError {
    /// The text holds a CR, which no world joins.
    #[error("the text holds a line break")]
    CarriageReturn,
    /// The text holds an LF, and the world has no `line_break` to put in its place.
    #[error("the text holds a line break, and the world sets no line_break")]
    NotJoined,
}

/// The error of a call on the `Portal` or the journal.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum PortalError {
    /// No world has this id.
    #[error("unknown world {0}")]
    UnknownWorld(WorldId),
    /// The world is connecting, reconnecting or disconnected.
    #[error("{0} is not connected")]
    NotConnected(WorldId),
    /// The world uses TLS, which the server cannot connect yet.
    #[error("{0} uses TLS, which this version cannot connect")]
    TlsNotSupported(WorldId),
    /// The world has a login, and its secret cannot be read. The reason says why, and names the
    /// secret reference when there is one.
    #[error("{0} cannot read the secret of its login: {1}")]
    NoSecret(WorldId, String),
    /// The text holds a line break that the world cannot send.
    #[error("{0}")]
    LineBreak(LineBreakError),
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
