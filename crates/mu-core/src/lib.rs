//! Server engine for `mu`.
//!
//! The engine knows no wire format. `mu-server` adapts it to the API schema.

pub mod command;
pub mod error;
pub mod event;
pub mod fake;
pub mod framer;
pub mod id;
pub mod journal;
pub mod secret;
pub mod transport;
pub mod world;

pub use command::{Command, Envelope, Origin};
pub use error::PortalError;
pub use event::{Event, EventKind, NewEvent, Seq, Source};
pub use fake::{ConnectGate, FakeGame, FakeTransport};
pub use framer::{Framer, Inbound, LineFramer};
pub use id::{BadWorldId, WorldId};
pub use journal::{Journal, Lagged, MemoryJournal, Subscription};
pub use secret::{EnvSecretStore, Secret, SecretError, SecretRef, SecretStore, load_password};
pub use transport::{ByteStream, Transport};
pub use world::{Login, ResolvedWorld};
