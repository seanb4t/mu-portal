//! The `Portal`: the handle of the engine that the API holds.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::{mpsc, oneshot, watch};

use crate::command::{Command, Envelope};
use crate::connection::{Immediate, LoginMoment};
use crate::error::PortalError;
use crate::event::Seq;
use crate::framer::{LineFramer, NewFramer};
use crate::id::WorldId;
use crate::journal::{Journal, Subscription};
use crate::script::{Handler, Hook};
use crate::secret::{Secret, SecretSource};
use crate::session::{self, Reply, Request, Router, SessionParts, WorldState};
use crate::transport::Transport;
use crate::world::ResolvedWorld;

/// One world with the source of its password, its transport and its scripts.
pub struct WorldSetup {
    world: ResolvedWorld,
    secret: Option<SecretSource>,
    transport: Option<Arc<dyn Transport>>,
    framer: NewFramer,
    login_moment: Arc<dyn LoginMoment>,
    hooks: Vec<Arc<dyn Hook>>,
    handlers: Vec<Arc<dyn Handler>>,
}

impl WorldSetup {
    /// A world with the [`LineFramer`], the login at the connect, no hook and no handler. The
    /// `password` is read once, at the start of the server.
    #[must_use]
    pub fn new(
        world: ResolvedWorld,
        password: Option<Secret>,
        transport: Arc<dyn Transport>,
    ) -> Self {
        Self::with(world, password.map(SecretSource::Fixed), Some(transport))
    }

    fn with(
        world: ResolvedWorld,
        secret: Option<SecretSource>,
        transport: Option<Arc<dyn Transport>>,
    ) -> Self {
        Self {
            world,
            secret,
            transport,
            framer: Arc::new(|| Box::new(LineFramer::new())),
            login_moment: Arc::new(Immediate),
            hooks: Vec::new(),
            handlers: Vec::new(),
        }
    }

    /// Sends the login at the moment that `moment` gives, instead of at the connect.
    #[must_use]
    pub fn with_login_moment(mut self, moment: Arc<dyn LoginMoment>) -> Self {
        self.login_moment = moment;
        self
    }

    /// A world with no transport and no password: a TLS world. It stays disconnected, and a
    /// connect of it fails with [`PortalError::TlsNotSupported`].
    #[must_use]
    pub fn without_transport(world: ResolvedWorld) -> Self {
        Self::with(world, None, None)
    }

    /// Uses `framer` for each new connection instead of the [`LineFramer`].
    #[must_use]
    pub fn with_framer(mut self, framer: NewFramer) -> Self {
        self.framer = framer;
        self
    }

    /// Adds a hook after the hooks already there.
    #[must_use]
    pub fn with_hook(mut self, hook: Arc<dyn Hook>) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Adds a handler of the events of this world.
    #[must_use]
    pub fn with_handler(mut self, handler: Arc<dyn Handler>) -> Self {
        self.handlers.push(handler);
        self
    }
}

/// The engine: one session task for each world, one journal and the server handlers.
#[derive(Clone)]
pub struct Portal {
    router: Router,
    states: Arc<BTreeMap<WorldId, watch::Receiver<WorldState>>>,
    journal: Arc<dyn Journal>,
    stopped: Arc<AtomicBool>,
    /// The last event that the server handlers handled. None with no server handlers.
    handled: Option<watch::Receiver<Seq>>,
}

impl Portal {
    /// Starts a task for each world and one for the server handlers. No world connects yet.
    ///
    /// Must run inside a tokio runtime.
    #[must_use]
    pub fn start(
        journal: Arc<dyn Journal>,
        worlds: Vec<WorldSetup>,
        server_handlers: Vec<Arc<dyn Handler>>,
    ) -> Self {
        let mut senders = BTreeMap::new();
        let mut receivers = Vec::new();
        let mut states = BTreeMap::new();
        for setup in worlds {
            let (sender, requests) = mpsc::unbounded_channel();
            let (state, state_receiver) = watch::channel(WorldState::Disconnected);
            senders.insert(setup.world.id.clone(), sender);
            states.insert(setup.world.id.clone(), state_receiver);
            receivers.push((setup, requests, state));
        }
        let router: Router = Arc::new(senders);
        let stopped = Arc::new(AtomicBool::new(false));
        for (setup, requests, state) in receivers {
            tokio::spawn(session::run(SessionParts {
                world: setup.world.id,
                login: setup.world.login,
                secret: setup.secret,
                transport: setup.transport,
                framer: setup.framer,
                login_moment: setup.login_moment,
                keepalive: setup.world.keepalive,
                hooks: setup.hooks,
                handlers: setup.handlers,
                journal: Arc::clone(&journal),
                router: Arc::clone(&router),
                state,
                requests,
                stopped: Arc::clone(&stopped),
            }));
        }
        let handled = if server_handlers.is_empty() {
            None
        } else {
            // The subscription starts here, so the handlers see each event after the start.
            let last = journal.last();
            let next = last.next();
            journal.subscribe(next).ok().map(|subscription| {
                let (progress, handled) = watch::channel(last);
                tokio::spawn(run_server_handlers(
                    Arc::clone(&journal),
                    subscription,
                    next,
                    Arc::clone(&router),
                    server_handlers,
                    progress,
                ));
                handled
            })
        };
        Self {
            router,
            states: Arc::new(states),
            journal,
            stopped,
            handled,
        }
    }

    /// The journal of the server.
    #[must_use]
    pub fn journal(&self) -> &Arc<dyn Journal> {
        &self.journal
    }

    /// Each world with its state, in the order of the ids.
    #[must_use]
    pub fn worlds(&self) -> Vec<(WorldId, WorldState)> {
        self.states
            .iter()
            .map(|(id, state)| (id.clone(), *state.borrow()))
            .collect()
    }

    /// A receiver of the state of `world`.
    ///
    /// # Errors
    ///
    /// [`PortalError::UnknownWorld`] for a world that does not exist.
    pub fn state(&self, world: &WorldId) -> Result<watch::Receiver<WorldState>, PortalError> {
        self.states
            .get(world)
            .cloned()
            .ok_or_else(|| PortalError::UnknownWorld(world.clone()))
    }

    /// The events from `from`, then each new event. See [`Journal::subscribe`].
    ///
    /// # Errors
    ///
    /// [`PortalError::PastEnd`] for a `from` past the next number.
    pub fn subscribe(&self, from: Seq) -> Result<Subscription, PortalError> {
        self.journal.subscribe(from)
    }

    /// Runs one command and returns the sequence number of its event, if it made one.
    ///
    /// # Errors
    ///
    /// [`PortalError::UnknownWorld`], [`PortalError::NotConnected`],
    /// [`PortalError::TlsNotSupported`], [`PortalError::NoSecret`], [`PortalError::LineBreak`], or
    /// [`PortalError::Stopped`].
    pub async fn submit(&self, envelope: Envelope) -> Result<Option<Seq>, PortalError> {
        let world = envelope.command.world().clone();
        self.ask(&world, |reply| Request::command(envelope, Some(reply)))
            .await
    }

    /// Sends the request that `make` builds from the answer sender to `world`, and waits for the
    /// answer. A frame answers with no number.
    async fn ask(
        &self,
        world: &WorldId,
        make: impl FnOnce(oneshot::Sender<Reply>) -> Request,
    ) -> Reply {
        let sender = self
            .router
            .get(world)
            .ok_or_else(|| PortalError::UnknownWorld(world.clone()))?;
        let (reply, answer) = oneshot::channel();
        sender.send(make(reply)).map_err(|_| PortalError::Stopped)?;
        answer.await.map_err(|_| PortalError::Stopped)?
    }

    /// Sends one line of the player to `world`. No number means a hook gagged the line.
    ///
    /// # Errors
    ///
    /// See [`Portal::submit`].
    pub async fn send(&self, world: &WorldId, text: &str) -> Result<Option<Seq>, PortalError> {
        self.submit(Envelope::player(Command::Send {
            world: world.clone(),
            text: text.to_owned(),
        }))
        .await
    }

    /// Connects `world`, as the player.
    ///
    /// # Errors
    ///
    /// See [`Portal::submit`].
    pub async fn connect(&self, world: &WorldId) -> Result<(), PortalError> {
        self.submit(Envelope::player(Command::Connect {
            world: world.clone(),
        }))
        .await
        .map(|_| ())
    }

    /// Disconnects `world`, as the player.
    ///
    /// # Errors
    ///
    /// See [`Portal::submit`].
    pub async fn disconnect(&self, world: &WorldId) -> Result<(), PortalError> {
        self.submit(Envelope::player(Command::Disconnect {
            world: world.clone(),
        }))
        .await
        .map(|_| ())
    }

    /// Stops the engine, as the server: disconnects every world, and from now on each
    /// connect fails with [`PortalError::Stopped`], from the API and from a handler alike.
    /// When it returns, no world is up.
    pub async fn stop(&self) {
        // The flag goes up first, so a connect that a session reads after it fails.
        self.stopped.store(true, Ordering::Release);
        self.to_every_world(|world, reply| {
            Request::command(Envelope::system(Command::Disconnect { world }), Some(reply))
        })
        .await;
    }

    /// Connects every world, as the server. Each world logs in when it connects.
    pub async fn connect_all(&self) {
        self.to_every_world(|world, reply| {
            Request::command(Envelope::system(Command::Connect { world }), Some(reply))
        })
        .await;
    }

    /// Waits until the engine has no work left: the server handlers handled each event, each
    /// world answered a frame, and the journal got no event meanwhile.
    ///
    /// Only tests call it. A game that ignores `IAC DO TIMING-MARK` makes it hang.
    ///
    /// # Errors
    ///
    /// [`PortalError::Stopped`] when a frame gets no answer or the server handlers end.
    pub async fn wait_idle(&self) -> Result<(), PortalError> {
        loop {
            let mark = self.journal.last();
            if let Some(handled) = &self.handled {
                handled
                    .clone()
                    .wait_for(|seq| *seq >= mark)
                    .await
                    .map_err(|_| PortalError::Stopped)?;
            }
            for result in self.to_every_world(|_, reply| Request::Frame(reply)).await {
                result?;
            }
            if self.journal.last() == mark {
                return Ok(());
            }
        }
    }

    /// Sends one request to each world, in the order of the ids, and returns each result.
    async fn to_every_world(
        &self,
        make: fn(WorldId, oneshot::Sender<Reply>) -> Request,
    ) -> Vec<Reply> {
        let mut results = Vec::with_capacity(self.router.len());
        for world in self.router.keys() {
            results.push(self.ask(world, |reply| make(world.clone(), reply)).await);
        }
        results
    }
}

/// Runs the server handlers on each event of every world, from the next event on.
async fn run_server_handlers(
    journal: Arc<dyn Journal>,
    mut subscription: Subscription,
    mut next: Seq,
    router: Router,
    handlers: Vec<Arc<dyn Handler>>,
    progress: watch::Sender<Seq>,
) {
    while let Some(item) = subscription.next().await {
        let Ok(event) = item else {
            // The journal keeps every event, so a subscribe from the next number loses none.
            let Ok(again) = journal.subscribe(next) else {
                return;
            };
            subscription = again;
            continue;
        };
        next = event.seq.next();
        session::run_handlers(&handlers, &router, &event);
        progress.send_replace(event.seq);
    }
}
