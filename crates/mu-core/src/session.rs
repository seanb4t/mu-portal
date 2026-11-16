//! The task of one world: its transport, framer, hooks and handlers.

use std::collections::BTreeMap;
use std::io;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures::future::BoxFuture;
use governor::clock::Clock;
use governor::{Quota, RateLimiter};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _, ReadHalf, WriteHalf};
use tokio::sync::{mpsc, oneshot, watch};

use crate::command::{Command, Envelope, Origin};
use crate::error::PortalError;
use crate::event::{Event, EventKind, NewEvent, Seq, Source};
use crate::framer::{Framer, Inbound, NewFramer, has_line_break};
use crate::id::WorldId;
use crate::journal::Journal;
use crate::script::{DEPTH_LIMIT, Handler, Hook, RATE_PER_SECOND, run_hooks};
use crate::secret::{MASK, Secret};
use crate::transport::{ByteStream, Transport};
use crate::world::Login;

/// The state of a world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WorldState {
    /// The transport is opening.
    Connecting,
    /// The world is connected.
    Connected,
    /// The world has no connection.
    Disconnected,
}

/// The reply to a command: the sequence number of its event, or none.
pub(crate) type Reply = Result<Option<Seq>, PortalError>;

pub(crate) struct Request {
    pub(crate) envelope: Envelope,
    pub(crate) reply: Option<oneshot::Sender<Reply>>,
}

/// The command channel of each world.
pub(crate) type Router = Arc<BTreeMap<WorldId, mpsc::UnboundedSender<Request>>>;

/// Routes handler commands of the event at `depth` to their worlds.
pub(crate) fn route(router: &Router, commands: Vec<Command>, depth: u8) {
    for command in commands {
        let Some(sender) = router.get(command.world()) else {
            tracing::warn!(world = %command.world(), "a handler named an unknown world");
            continue;
        };
        let envelope = Envelope {
            command,
            origin: Origin::Script,
            depth: depth.saturating_add(1),
        };
        // A closed channel means the server stops.
        let _ = sender.send(Request {
            envelope,
            reply: None,
        });
    }
}

/// Hands `event` to each handler and routes their commands. A script error goes to no
/// handler, so a handler that fails cannot start a loop of errors.
pub(crate) fn run_handlers(handlers: &[Arc<dyn Handler>], router: &Router, event: &Event) {
    if event.kind == EventKind::ScriptError {
        return;
    }
    for handler in handlers {
        route(router, handler.on_event(event), event.depth);
    }
}

/// The clock of the rate limit. It reads tokio time, so a paused clock drives it.
#[derive(Clone, Copy, Debug, Default)]
struct TokioClock;

impl Clock for TokioClock {
    type Instant = std::time::Instant;

    fn now(&self) -> Self::Instant {
        tokio::time::Instant::now().into_std()
    }
}

type Limiter = RateLimiter<
    governor::state::NotKeyed,
    governor::state::InMemoryState,
    TokioClock,
    governor::middleware::NoOpMiddleware<std::time::Instant>,
>;

const RATE: NonZeroU32 = NonZeroU32::new(RATE_PER_SECOND).expect("the rate is not 0");

/// Everything a session needs, given by the `Portal`.
pub(crate) struct SessionParts {
    pub(crate) world: WorldId,
    pub(crate) login: Option<Login>,
    pub(crate) password: Option<Secret>,
    pub(crate) transport: Arc<dyn Transport>,
    pub(crate) framer: NewFramer,
    pub(crate) hooks: Vec<Arc<dyn Hook>>,
    pub(crate) handlers: Vec<Arc<dyn Handler>>,
    pub(crate) journal: Arc<dyn Journal>,
    pub(crate) router: Router,
    pub(crate) state: watch::Sender<WorldState>,
    pub(crate) requests: mpsc::UnboundedReceiver<Request>,
    /// Set by [`crate::Portal::stop`]. After it, the session refuses each connect.
    pub(crate) stopped: Arc<AtomicBool>,
}

enum Conn {
    Down,
    Connecting {
        connect: BoxFuture<'static, io::Result<Box<dyn ByteStream>>>,
        depth: u8,
    },
    Up {
        read: ReadHalf<Box<dyn ByteStream>>,
        write: WriteHalf<Box<dyn ByteStream>>,
        framer: Box<dyn Framer>,
    },
}

enum Step {
    Request(Option<Request>),
    Connected(io::Result<Box<dyn ByteStream>>, u8),
    Read(io::Result<usize>),
}

struct Session {
    parts: SessionParts,
    conn: Conn,
    limiter: Limiter,
    last_script_error: Option<tokio::time::Instant>,
}

/// Runs the session of one world until the `Portal` drops its channel.
pub(crate) async fn run(parts: SessionParts) {
    let mut session = Session {
        parts,
        conn: Conn::Down,
        limiter: RateLimiter::direct_with_clock(Quota::per_second(RATE), TokioClock),
        last_script_error: None,
    };
    let mut buffer = vec![0_u8; 4096];
    loop {
        let step = tokio::select! {
            request = session.parts.requests.recv() => Step::Request(request),
            step = progress(&mut session.conn, &mut buffer) => step,
        };
        match step {
            Step::Request(None) => return,
            Step::Request(Some(request)) => {
                let reply = session.handle(request.envelope).await;
                if let Some(sender) = request.reply {
                    // The caller can give up on the reply.
                    let _ = sender.send(reply);
                }
            }
            Step::Connected(result, depth) => session.on_connect(result, depth).await,
            Step::Read(Ok(0) | Err(_)) => {
                session.down("the game closed the connection", 0);
            }
            Step::Read(Ok(n)) => session.on_bytes(&buffer[..n]).await,
        }
    }
}

/// The next step of the connection: a finished connect or a read. Pending with no connection.
async fn progress(conn: &mut Conn, buffer: &mut [u8]) -> Step {
    match conn {
        Conn::Connecting { connect, depth } => Step::Connected(connect.await, *depth),
        Conn::Up { read, .. } => Step::Read(read.read(buffer).await),
        Conn::Down => std::future::pending().await,
    }
}

impl Session {
    fn world(&self) -> &WorldId {
        &self.parts.world
    }

    /// Masks, records and hands an event to the world handlers.
    fn record(&mut self, kind: EventKind, origin: Origin, text: &str, depth: u8) -> Event {
        let text = self
            .parts
            .password
            .as_ref()
            .map_or_else(|| text.to_owned(), |password| password.mask(text));
        let event = self.parts.journal.append(NewEvent {
            world: self.world().clone(),
            kind,
            source: Source::of(kind, origin),
            text,
            depth,
        });
        run_handlers(&self.parts.handlers, &self.parts.router, &event);
        event
    }

    fn script_error(&mut self, text: &str, depth: u8) {
        self.record(EventKind::ScriptError, Origin::Script, text, depth);
    }

    /// Records the drop of a command over the rate, at most once in each second.
    fn rate_drop(&mut self, depth: u8) {
        let now = tokio::time::Instant::now();
        let due = self
            .last_script_error
            .is_none_or(|last| now.duration_since(last) >= Duration::from_secs(1));
        if due {
            self.last_script_error = Some(now);
            let text = format!(
                "a handler sent more than {RATE_PER_SECOND} commands in one second, so the engine dropped a command"
            );
            self.script_error(&text, depth);
        }
    }

    fn set_state(&self, state: WorldState) {
        self.parts.state.send_replace(state);
    }

    fn down(&mut self, reason: &str, depth: u8) {
        self.conn = Conn::Down;
        self.set_state(WorldState::Disconnected);
        self.record(EventKind::Disconnected, Origin::System, reason, depth);
    }

    async fn write_line(&mut self, line: &str, depth: u8) -> Result<(), PortalError> {
        let Conn::Up { write, framer, .. } = &mut self.conn else {
            return Err(PortalError::NotConnected(self.world().clone()));
        };
        let bytes = framer.encode(line);
        if write.write_all(&bytes).await.is_err() || write.flush().await.is_err() {
            self.down("the connection failed", depth);
            return Err(PortalError::NotConnected(self.world().clone()));
        }
        Ok(())
    }

    async fn login(&mut self, depth: u8) -> Reply {
        let Some(login) = self.parts.login.clone() else {
            return Ok(None);
        };
        let password = self
            .parts
            .password
            .as_ref()
            .map_or("", |password| password.expose())
            .to_owned();
        self.write_line(&login.line(&password), depth).await?;
        // The event gets the line of the template for the mask, so it never holds the password.
        let event = self.record(EventKind::Login, Origin::System, &login.line(MASK), depth);
        Ok(Some(event.seq))
    }

    async fn on_connect(&mut self, result: io::Result<Box<dyn ByteStream>>, depth: u8) {
        match result {
            Ok(stream) => {
                let (read, write) = tokio::io::split(stream);
                self.conn = Conn::Up {
                    read,
                    write,
                    framer: (self.parts.framer)(),
                };
                self.set_state(WorldState::Connected);
                self.record(EventKind::Connected, Origin::System, "connected", depth);
                // A failed login line already recorded the disconnect.
                let _ = self.login(depth).await;
            }
            Err(error) => self.down(&format!("the connect failed: {error}"), depth),
        }
    }

    async fn on_bytes(&mut self, bytes: &[u8]) {
        let Conn::Up { framer, .. } = &mut self.conn else {
            return;
        };
        for inbound in framer.feed(bytes) {
            match inbound {
                Inbound::Line(line) => {
                    if let Some(line) = run_hooks(&self.parts.hooks, &line, Hook::on_game_line) {
                        self.record(EventKind::Line, Origin::System, &line, 0);
                    }
                }
                Inbound::Prompt(prompt) => {
                    self.record(EventKind::Prompt, Origin::System, &prompt, 0);
                }
                Inbound::Reply(bytes) => {
                    if let Conn::Up { write, .. } = &mut self.conn
                        && write.write_all(&bytes).await.is_err()
                    {
                        self.down("the connection failed", 0);
                        return;
                    }
                }
                Inbound::Message { name, body } => {
                    let text = format!("{name} {}", String::from_utf8_lossy(&body));
                    self.record(EventKind::Message, Origin::System, &text, 0);
                }
            }
        }
    }

    async fn handle(&mut self, envelope: Envelope) -> Reply {
        let Envelope {
            command,
            origin,
            depth,
        } = envelope;
        if depth > DEPTH_LIMIT {
            let text = format!(
                "a chain of handlers passed the depth limit of {DEPTH_LIMIT}, so the engine dropped a command"
            );
            self.script_error(&text, depth);
            return Ok(None);
        }
        if origin == Origin::Script && self.limiter.check().is_err() {
            self.rate_drop(depth);
            return Ok(None);
        }
        match command {
            Command::Send { text, .. } => {
                if has_line_break(&text) {
                    return Err(PortalError::LineBreak);
                }
                if !matches!(self.conn, Conn::Up { .. }) {
                    return Err(PortalError::NotConnected(self.world().clone()));
                }
                let Some(text) = run_hooks(&self.parts.hooks, &text, Hook::on_player_line) else {
                    return Ok(None);
                };
                if has_line_break(&text) {
                    self.script_error(
                        "a hook replaced a line with text that holds a line break, so the engine dropped the line",
                        depth,
                    );
                    return Ok(None);
                }
                self.write_line(&text, depth).await?;
                Ok(Some(
                    self.record(EventKind::Input, origin, &text, depth).seq,
                ))
            }
            Command::Connect { .. } => {
                if self.parts.stopped.load(Ordering::Acquire) {
                    return Err(PortalError::Stopped);
                }
                if matches!(self.conn, Conn::Down) {
                    let connect = self.parts.transport.connect(self.world());
                    self.conn = Conn::Connecting { connect, depth };
                    self.set_state(WorldState::Connecting);
                }
                Ok(None)
            }
            Command::Disconnect { .. } => {
                if matches!(self.conn, Conn::Down) {
                    return Ok(None);
                }
                self.down("disconnected", depth);
                Ok(None)
            }
            Command::Login { .. } => self.login(depth).await,
            Command::Echo { text, .. } => {
                Ok(Some(self.record(EventKind::Echo, origin, &text, depth).seq))
            }
            Command::Emit { text, .. } => Ok(Some(
                self.record(EventKind::Custom, origin, &text, depth).seq,
            )),
        }
    }
}
