//! The task of one world: its transport, framer, hooks and handlers.

use std::collections::{BTreeMap, VecDeque};
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
use tokio::time::Instant;
use tracing::{Instrument as _, Span};

use crate::command::{Command, Envelope, Origin};
use crate::connection::{Cause, LoginMoment};
use crate::error::PortalError;
use crate::event::{Event, EventKind, NewEvent, Seq, Source};
use crate::framer::{Framer, Inbound, NewFramer, has_line_break};
use crate::id::WorldId;
use crate::journal::Journal;
use crate::script::{DEPTH_LIMIT, Handler, Hook, RATE_PER_SECOND, run_hooks};
use crate::secret::{MASK, Secret, SecretSource};
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

impl WorldState {
    /// Every state.
    pub const ALL: [Self; 3] = [Self::Connecting, Self::Connected, Self::Disconnected];

    /// The lower-case name of the state, for the telemetry.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Connecting => "connecting",
            Self::Connected => "connected",
            Self::Disconnected => "disconnected",
        }
    }
}

/// Records `value` in the gauge `mu.world.state` of `world` for `state`.
fn state_gauge(world: &WorldId, state: WorldState, value: u64) {
    tracing::event!(
        target: crate::METRICS_TARGET,
        tracing::Level::INFO,
        mu.world = %world,
        "gauge.mu.world.state" = value,
        mu.world.state.name = state.name(),
    );
}

/// The reply to a command: the sequence number of its event, or none.
pub(crate) type Reply = Result<Option<Seq>, PortalError>;

/// What a world queue holds.
pub(crate) enum Request {
    /// A command, with the reply for its caller if it has one, and the span of its caller.
    Command {
        envelope: Envelope,
        reply: Option<oneshot::Sender<Reply>>,
        span: Span,
    },
    /// A control frame. The session answers it after each request before it and, if the
    /// world is up, after the game answers a mark. It is not a command and no event records it.
    Frame(oneshot::Sender<Reply>),
}

impl Request {
    /// A command with the current span as the span of its caller.
    pub(crate) fn command(envelope: Envelope, reply: Option<oneshot::Sender<Reply>>) -> Self {
        Self::Command {
            envelope,
            reply,
            span: Span::current(),
        }
    }
}

/// Answers `frame` with no result. The caller can give up on the answer.
fn answer(frame: oneshot::Sender<Reply>) {
    let _ = frame.send(Ok(None));
}

/// The frames that wait for a mark, in the order of their requests.
#[derive(Default)]
struct WaitingFrames(VecDeque<oneshot::Sender<Reply>>);

impl WaitingFrames {
    fn push(&mut self, frame: oneshot::Sender<Reply>) {
        self.0.push_back(frame);
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    /// Answers the oldest frame. With no frame, it does nothing.
    fn answer_first(&mut self) {
        if let Some(frame) = self.0.pop_front() {
            answer(frame);
        }
    }

    fn answer_all(&mut self) {
        while !self.0.is_empty() {
            self.answer_first();
        }
    }
}

/// The command channel of each world.
pub(crate) type Router = Arc<BTreeMap<WorldId, mpsc::UnboundedSender<Request>>>;

/// Routes handler commands of the event at `depth` to their worlds.
pub(crate) fn route(router: &Router, commands: Vec<Command>, depth: u8) {
    for command in commands {
        let Some(sender) = router.get(command.world()) else {
            tracing::warn!(mu.world = %command.world(), "a handler named an unknown world");
            continue;
        };
        let envelope = Envelope {
            command,
            origin: Origin::Script,
            depth: depth.saturating_add(1),
        };
        // A closed channel means the server stops.
        let _ = sender.send(Request::command(envelope, None));
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
    /// The source of the password, or none for a world with no login.
    pub(crate) secret: Option<SecretSource>,
    pub(crate) transport: Option<Arc<dyn Transport>>,
    pub(crate) framer: NewFramer,
    /// When the login line goes out.
    pub(crate) login_moment: Arc<dyn LoginMoment>,
    pub(crate) hooks: Vec<Arc<dyn Hook>>,
    pub(crate) handlers: Vec<Arc<dyn Handler>>,
    pub(crate) journal: Arc<dyn Journal>,
    pub(crate) router: Router,
    pub(crate) state: watch::Sender<WorldState>,
    pub(crate) requests: mpsc::UnboundedReceiver<Request>,
    /// Set by [`crate::Portal::stop`]. After it, the session refuses each connect.
    pub(crate) stopped: Arc<AtomicBool>,
}

/// The time that a connect of the transport can take.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Why a connect of the transport failed.
enum Failure {
    /// The transport gave an error.
    Connect(io::Error),
    /// The connect took longer than [`CONNECT_TIMEOUT`].
    Timeout,
}

enum Conn {
    Down,
    Connecting {
        connect: BoxFuture<'static, Result<Box<dyn ByteStream>, Failure>>,
        depth: u8,
        frames: WaitingFrames,
        /// The `world.connection` span. It closes when its last clone drops: `down` drops this
        /// one, a step that runs in the span holds a clone until the step ends, and the end of
        /// the session drops the rest.
        span: Span,
    },
    Up {
        read: ReadHalf<Box<dyn ByteStream>>,
        write: WriteHalf<Box<dyn ByteStream>>,
        framer: Box<dyn Framer>,
        /// The login that waits for its moment. The world stays in `connecting` until it goes.
        pending: Option<PendingLogin>,
        frames: WaitingFrames,
        /// The `world.connection` span. It closes when its last clone drops: `down` drops this
        /// one, a step that runs in the span holds a clone until the step ends, and the end of
        /// the session drops the rest.
        span: Span,
    },
}

/// A login that waits for the moment of its [`LoginMoment`].
struct PendingLogin {
    /// The open of the stream.
    connect: Instant,
    /// The last byte from the game, or none before the first.
    last_byte: Option<Instant>,
    /// The depth of the command that started the connect.
    depth: u8,
}

impl Conn {
    /// Whether the world is up and logged in, so a line can go to the game.
    fn is_connected(&self) -> bool {
        matches!(self, Self::Up { pending: None, .. })
    }

    /// The `world.connection` span, or a disabled span with no connection.
    fn span(&self) -> Span {
        match self {
            Self::Down => Span::none(),
            Self::Connecting { span, .. } | Self::Up { span, .. } => span.clone(),
        }
    }
}

enum Step {
    Request(Option<Request>),
    Connected(Result<Box<dyn ByteStream>, Failure>, u8),
    Read(io::Result<usize>),
    /// The moment of the pending login came.
    Login,
}

struct Session {
    parts: SessionParts,
    conn: Conn,
    limiter: Limiter,
    last_script_error: Option<tokio::time::Instant>,
    /// The password of the last connect, which masks each event. A fixed source gives it at
    /// the start.
    password: Option<Secret>,
}

/// Runs the session of one world until the `Portal` drops its channel.
pub(crate) async fn run(parts: SessionParts) {
    let mut session = Session {
        parts,
        conn: Conn::Down,
        limiter: RateLimiter::direct_with_clock(Quota::per_second(RATE), TokioClock),
        last_script_error: None,
        password: None,
    };
    if let Some(SecretSource::Fixed(password)) = &session.parts.secret {
        session.password = Some(password.clone());
    }
    let start = *session.parts.state.borrow();
    state_gauge(&session.parts.world, start, 1);
    let mut buffer = vec![0_u8; 4096];
    loop {
        let step = tokio::select! {
            request = session.parts.requests.recv() => Step::Request(request),
            step = progress(&mut session.conn, &*session.parts.login_moment, &mut buffer) => step,
        };
        match step {
            Step::Request(None) => return,
            Step::Request(Some(Request::Command {
                envelope,
                reply,
                span,
            })) => {
                let command = tracing::info_span!(
                    parent: &span,
                    "command",
                    mu.world = %session.parts.world,
                    mu.command = envelope.command.name(),
                    mu.origin = envelope.origin.name(),
                    mu.depth = envelope.depth,
                );
                let result = session.handle(envelope).instrument(command).await;
                if let Some(sender) = reply {
                    // The caller can give up on the reply.
                    let _ = sender.send(result);
                }
            }
            Step::Request(Some(Request::Frame(frame))) => session.on_frame(frame).await,
            Step::Connected(result, depth) => {
                let span = session.conn.span();
                session.on_connect(result, depth).instrument(span).await;
            }
            Step::Read(Ok(0)) => {
                session.down(Cause::Closed, "the game closed the connection", 0);
            }
            Step::Read(Err(_)) => session.down(Cause::Lost, "the connection failed", 0),
            Step::Read(Ok(n)) => {
                let span = session.conn.span();
                session.on_bytes(&buffer[..n]).instrument(span).await;
            }
            Step::Login => {
                let span = session.conn.span();
                session.on_login_moment().instrument(span).await;
            }
        }
    }
}

/// The next step of the connection: a finished connect, a read, or the moment of a pending
/// login. Pending with no connection.
async fn progress(conn: &mut Conn, moment: &dyn LoginMoment, buffer: &mut [u8]) -> Step {
    match conn {
        Conn::Connecting { connect, depth, .. } => Step::Connected(connect.await, *depth),
        Conn::Up {
            read,
            pending: Some(pending),
            ..
        } => {
            let deadline = moment.deadline(pending.connect, pending.last_byte);
            tokio::select! {
                result = read.read(buffer) => Step::Read(result),
                () = tokio::time::sleep_until(deadline) => Step::Login,
            }
        }
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
        tracing::event!(
            target: crate::METRICS_TARGET,
            tracing::Level::INFO,
            mu.world = %self.parts.world,
            "monotonic_counter.mu.journal.events" = 1_u64,
            mu.event.kind = kind.name(),
        );
        run_handlers(&self.parts.handlers, &self.parts.router, &event);
        event
    }

    fn script_error(&mut self, text: &str, depth: u8) {
        tracing::event!(
            target: crate::METRICS_TARGET,
            tracing::Level::INFO,
            mu.world = %self.parts.world,
            "monotonic_counter.mu.script.errors" = 1_u64,
        );
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

    /// Adds 1 to `mu.commands.dropped` of this world, with `reason` as `mu.drop.reason`.
    fn dropped(&self, reason: &'static str) {
        tracing::event!(
            target: crate::METRICS_TARGET,
            tracing::Level::INFO,
            mu.world = %self.parts.world,
            "monotonic_counter.mu.commands.dropped" = 1_u64,
            mu.drop.reason = reason,
        );
    }

    fn set_state(&self, state: WorldState) {
        let old = self.parts.state.send_replace(state);
        // The 1 comes last, so a state that does not change keeps its 1.
        state_gauge(self.world(), old, 0);
        state_gauge(self.world(), state, 1);
    }

    /// Ends the connection or the connect for `cause`, records it on the span of the connection
    /// and in the journal, and answers each waiting frame.
    fn down(&mut self, cause: Cause, reason: &str, depth: u8) {
        let mut frames = match std::mem::replace(&mut self.conn, Conn::Down) {
            Conn::Connecting { frames, span, .. } | Conn::Up { frames, span, .. } => {
                span.record("mu.connection.end", cause.end());
                if cause.is_error() {
                    span.record("otel.status_code", "ERROR");
                }
                frames
            }
            Conn::Down => WaitingFrames::default(),
        };
        self.set_state(WorldState::Disconnected);
        self.record(EventKind::Disconnected, Origin::System, reason, depth);
        frames.answer_all();
    }

    async fn write_bytes(&mut self, bytes: &[u8], depth: u8) -> Result<(), PortalError> {
        let Conn::Up { write, .. } = &mut self.conn else {
            return Err(PortalError::NotConnected(self.world().clone()));
        };
        if write.write_all(bytes).await.is_err() || write.flush().await.is_err() {
            self.down(Cause::Lost, "the connection failed", depth);
            return Err(PortalError::NotConnected(self.world().clone()));
        }
        Ok(())
    }

    async fn write_line(&mut self, line: &str, depth: u8) -> Result<(), PortalError> {
        let Conn::Up { framer, .. } = &self.conn else {
            return Err(PortalError::NotConnected(self.world().clone()));
        };
        let bytes = framer.encode(line);
        self.write_bytes(&bytes, depth).await
    }

    /// Writes the mark for the oldest waiting frame. A framer with no mark bytes answers
    /// that frame at once.
    async fn write_mark(&mut self, depth: u8) -> Result<(), PortalError> {
        let Conn::Up { framer, frames, .. } = &mut self.conn else {
            return Err(PortalError::NotConnected(self.world().clone()));
        };
        let bytes = framer.mark();
        if bytes.is_empty() {
            frames.answer_first();
            return Ok(());
        }
        self.write_bytes(&bytes, depth).await
    }

    async fn on_frame(&mut self, frame: oneshot::Sender<Reply>) {
        match &mut self.conn {
            Conn::Down => answer(frame),
            Conn::Connecting { frames, .. }
            | Conn::Up {
                frames,
                pending: Some(_),
                ..
            } => frames.push(frame),
            Conn::Up { frames, .. } => {
                frames.push(frame);
                // A failed write already answered the frame through `down`.
                let _ = self.write_mark(0).await;
            }
        }
    }

    /// Starts each connect of the world: reads the secret, opens the `world.connection` span, and
    /// starts the connect of the transport under [`CONNECT_TIMEOUT`]. A failed read starts
    /// nothing.
    fn connect(&mut self, depth: u8) -> Result<(), PortalError> {
        let Some(transport) = self.parts.transport.clone() else {
            return Err(PortalError::TlsNotSupported(self.world().clone()));
        };
        if let Some(password) = self.read_secret()? {
            self.password = Some(password);
        }
        let span = tracing::info_span!(
            parent: None,
            "world.connection",
            mu.world = %self.parts.world,
            mu.connection.end = tracing::field::Empty,
            otel.status_code = tracing::field::Empty,
        );
        span.follows_from(Span::current());
        let connect = transport.connect(self.world());
        let connect = Box::pin(async move {
            match tokio::time::timeout(CONNECT_TIMEOUT, connect).await {
                Ok(result) => result.map_err(Failure::Connect),
                Err(_) => Err(Failure::Timeout),
            }
        });
        self.conn = Conn::Connecting {
            connect,
            depth,
            frames: WaitingFrames::default(),
            span,
        };
        self.set_state(WorldState::Connecting);
        Ok(())
    }

    /// The password of a connect, read from its source. None for a world with no login.
    fn read_secret(&self) -> Result<Option<Secret>, PortalError> {
        if self.parts.login.is_none() {
            return Ok(None);
        }
        let no_secret = |reason: String| PortalError::NoSecret(self.world().clone(), reason);
        let Some(source) = &self.parts.secret else {
            return Err(no_secret("the world has no secret".to_owned()));
        };
        source
            .read()
            .map(Some)
            .map_err(|error| no_secret(error.to_string()))
    }

    async fn login(&mut self, depth: u8) -> Reply {
        let Some(login) = self.parts.login.clone() else {
            return Ok(None);
        };
        // A connect reads the password, so a world with no password never connected.
        let Some(password) = &self.password else {
            return Err(PortalError::NotConnected(self.world().clone()));
        };
        let password = password.expose().to_owned();
        self.write_line(&login.line(&password), depth).await?;
        // The event gets the line of the template for the mask, so it never holds the password.
        let event = self.record(EventKind::Login, Origin::System, &login.line(MASK), depth);
        Ok(Some(event.seq))
    }

    async fn on_connect(&mut self, result: Result<Box<dyn ByteStream>, Failure>, depth: u8) {
        match result {
            Ok(stream) => {
                let (read, write) = tokio::io::split(stream);
                let (frames, span) = match &mut self.conn {
                    Conn::Connecting { frames, span, .. } => (std::mem::take(frames), span.clone()),
                    Conn::Down | Conn::Up { .. } => (WaitingFrames::default(), Span::current()),
                };
                let connect = Instant::now();
                let waits = self.parts.login.is_some()
                    && self.parts.login_moment.deadline(connect, None) > connect;
                self.conn = Conn::Up {
                    read,
                    write,
                    framer: (self.parts.framer)(),
                    pending: waits.then_some(PendingLogin {
                        connect,
                        last_byte: None,
                        depth,
                    }),
                    frames,
                    span,
                };
                if !waits {
                    self.logged_in(depth).await;
                }
            }
            Err(Failure::Connect(error)) => self.down(
                Cause::ConnectFailed,
                &format!("the connect failed: {error}"),
                depth,
            ),
            Err(Failure::Timeout) => {
                let seconds = CONNECT_TIMEOUT.as_secs();
                let reason = format!("the connect failed: it timed out after {seconds} seconds");
                self.down(Cause::Timeout, &reason, depth);
            }
        }
    }

    /// Sends the pending login at its moment.
    async fn on_login_moment(&mut self) {
        let Conn::Up { pending, .. } = &mut self.conn else {
            return;
        };
        if let Some(pending) = pending.take() {
            self.logged_in(pending.depth).await;
        }
    }

    /// Moves the world to `connected`: records the connect, sends the login, and writes the
    /// mark of each frame that waits.
    async fn logged_in(&mut self, depth: u8) {
        let waiting = match &self.conn {
            Conn::Up { frames, .. } => frames.len(),
            Conn::Down | Conn::Connecting { .. } => return,
        };
        self.set_state(WorldState::Connected);
        self.record(EventKind::Connected, Origin::System, "connected", depth);
        // A failed write already recorded the disconnect and answered each frame.
        let _ = self.login(depth).await;
        for _ in 0..waiting {
            if self.write_mark(depth).await.is_err() {
                break;
            }
        }
    }

    async fn on_bytes(&mut self, bytes: &[u8]) {
        let Conn::Up {
            framer, pending, ..
        } = &mut self.conn
        else {
            return;
        };
        if let Some(pending) = pending {
            // Every byte counts, telnet negotiation included.
            pending.last_byte = Some(Instant::now());
        }
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
                    if self.write_bytes(&bytes, 0).await.is_err() {
                        return;
                    }
                }
                Inbound::Mark => {
                    if let Conn::Up { frames, .. } = &mut self.conn {
                        frames.answer_first();
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
            self.dropped("depth");
            self.script_error(&text, depth);
            return Ok(None);
        }
        if origin == Origin::Script && self.limiter.check().is_err() {
            self.dropped("rate");
            self.rate_drop(depth);
            return Ok(None);
        }
        match command {
            Command::Send { text, .. } => {
                if has_line_break(&text) {
                    return Err(PortalError::LineBreak);
                }
                if !self.conn.is_connected() {
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
                if self.parts.transport.is_none() {
                    return Err(PortalError::TlsNotSupported(self.world().clone()));
                }
                if matches!(self.conn, Conn::Down) {
                    self.connect(depth)?;
                }
                Ok(None)
            }
            Command::Disconnect { .. } => {
                if matches!(self.conn, Conn::Down) {
                    return Ok(None);
                }
                self.down(Cause::Requested(origin), "disconnected", depth);
                Ok(None)
            }
            Command::Login { .. } if !self.conn.is_connected() => {
                Err(PortalError::NotConnected(self.world().clone()))
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

#[cfg(test)]
mod tests {
    use tokio::sync::oneshot::error::TryRecvError;

    use super::*;

    #[test]
    fn each_state_has_a_name() {
        let names: Vec<_> = WorldState::ALL.into_iter().map(WorldState::name).collect();
        assert_eq!(names, ["connecting", "connected", "disconnected"]);
    }

    #[test]
    fn waiting_frames_answer_in_order() {
        let mut frames = WaitingFrames::default();
        let mut answers: Vec<_> = (0..3)
            .map(|_| {
                let (frame, answer) = oneshot::channel();
                frames.push(frame);
                answer
            })
            .collect();
        frames.answer_first();
        assert!(matches!(answers[0].try_recv(), Ok(Ok(None))));
        assert!(matches!(answers[1].try_recv(), Err(TryRecvError::Empty)));
        assert!(matches!(answers[2].try_recv(), Err(TryRecvError::Empty)));
        frames.answer_all();
        assert!(matches!(answers[1].try_recv(), Ok(Ok(None))));
        assert!(matches!(answers[2].try_recv(), Ok(Ok(None))));
        // With no frame left, an answer changes nothing.
        frames.answer_first();
    }
}
