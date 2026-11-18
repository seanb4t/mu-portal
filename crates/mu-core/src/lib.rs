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
pub mod portal;
pub mod script;
pub mod secret;
mod session;
pub mod telnet;
pub mod transport;
pub mod world;

/// The target of the metric events of the engine and the API, which the layer of the metrics
/// reads.
pub const METRICS_TARGET: &str = "mu::metrics";

pub use command::{Command, Envelope, Origin};
pub use error::PortalError;
pub use event::{Event, EventKind, NewEvent, Seq, Source};
pub use fake::{ConnectGate, FakeGame, FakeTransport};
pub use framer::{Charset, Framer, Inbound, LineFramer, NewFramer, TelnetFramer, TelnetSettings};
pub use id::{BadWorldId, WorldId};
pub use journal::{Journal, Lagged, MemoryJournal, Subscription};
pub use portal::{Portal, WorldSetup};
pub use script::{DEPTH_LIMIT, Handler, Hook, HookAction, RATE_PER_SECOND};
pub use secret::{EnvSecretStore, Secret, SecretError, SecretRef, SecretStore, load_password};
pub use session::WorldState;
pub use transport::{ByteStream, Transport};
pub use world::{Keepalive, Login, ResolvedWorld};
