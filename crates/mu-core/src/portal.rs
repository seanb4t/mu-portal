//! The `Portal`: the handle of the engine that the API holds.

use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch};

use crate::command::{Command, Envelope};
use crate::error::PortalError;
use crate::event::Seq;
use crate::framer::{LineFramer, NewFramer};
use crate::id::WorldId;
use crate::journal::{Journal, Subscription};
use crate::script::{Handler, Hook};
use crate::secret::Secret;
use crate::session::{self, Request, Router, SessionParts, WorldState};
use crate::transport::Transport;
use crate::world::ResolvedWorld;

/// One world with its password, its transport and its scripts.
pub struct WorldSetup {
    world: ResolvedWorld,
    password: Option<Secret>,
    transport: Arc<dyn Transport>,
    framer: NewFramer,
    hooks: Vec<Arc<dyn Hook>>,
    handlers: Vec<Arc<dyn Handler>>,
}

impl WorldSetup {
    /// A world with the [`LineFramer`], no hook and no handler.
    #[must_use]
    pub fn new(
        world: ResolvedWorld,
        password: Option<Secret>,
        transport: Arc<dyn Transport>,
    ) -> Self {
        Self {
            world,
            password,
            transport,
            framer: Arc::new(|| Box::new(LineFramer::new())),
            hooks: Vec::new(),
            handlers: Vec::new(),
        }
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
        for (setup, requests, state) in receivers {
            tokio::spawn(session::run(SessionParts {
                world: setup.world.id,
                login: setup.world.login,
                password: setup.password,
                transport: setup.transport,
                framer: setup.framer,
                hooks: setup.hooks,
                handlers: setup.handlers,
                journal: Arc::clone(&journal),
                router: Arc::clone(&router),
                state,
                requests,
            }));
        }
        if !server_handlers.is_empty() {
            // The subscription starts here, so the handlers see each event after the start.
            let next = journal.last().next();
            if let Ok(subscription) = journal.subscribe(next) {
                tokio::spawn(run_server_handlers(
                    Arc::clone(&journal),
                    subscription,
                    next,
                    Arc::clone(&router),
                    server_handlers,
                ));
            }
        }
        Self {
            router,
            states: Arc::new(states),
            journal,
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
    /// [`PortalError::LineBreak`], or [`PortalError::Stopped`].
    pub async fn submit(&self, envelope: Envelope) -> Result<Option<Seq>, PortalError> {
        let world = envelope.command.world();
        let sender = self
            .router
            .get(world)
            .ok_or_else(|| PortalError::UnknownWorld(world.clone()))?;
        let (reply, answer) = oneshot::channel();
        sender
            .send(Request {
                envelope,
                reply: Some(reply),
            })
            .map_err(|_| PortalError::Stopped)?;
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

    /// Connects every world, as the server. Each world logs in when it connects.
    pub async fn connect_all(&self) {
        for world in self.router.keys() {
            // Only a stopped engine fails here.
            let _ = self
                .submit(Envelope::system(Command::Connect {
                    world: world.clone(),
                }))
                .await;
        }
    }
}

/// Runs the server handlers on each event of every world, from the next event on.
async fn run_server_handlers(
    journal: Arc<dyn Journal>,
    mut subscription: Subscription,
    mut next: Seq,
    router: Router,
    handlers: Vec<Arc<dyn Handler>>,
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
    }
}
