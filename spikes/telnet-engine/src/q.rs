//! Option negotiation with the Q method of RFC 1143.
//!
//! Each option has one state for each side. The local side is this client (WILL and WONT
//! come from it), and the remote side is the server.

/// The negotiation state of one option on one side.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Q {
    #[default]
    No,
    Yes,
    /// A request to disable is open. `opposite` is a queued request to enable again.
    WantNo {
        opposite: bool,
    },
    /// A request to enable is open. `opposite` is a queued request to disable again.
    WantYes {
        opposite: bool,
    },
}

/// What the engine does after an option message from the peer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Step {
    pub next: Q,
    /// `Some(true)` sends the positive answer (DO or WILL), `Some(false)` the negative.
    pub reply: Option<bool>,
    /// `Some(true)` reports that the option is now on, `Some(false)` that it is now off.
    pub report: Option<bool>,
    /// The peer broke the protocol, for example an answer to a request that was never sent.
    pub violation: bool,
}

impl Step {
    const fn new(next: Q, reply: Option<bool>, report: Option<bool>) -> Self {
        Self {
            next,
            reply,
            report,
            violation: false,
        }
    }
}

/// The peer asks to enable the option (WILL for the remote side, DO for the local side).
pub fn on_enable(state: Q, accept: bool) -> Step {
    match state {
        Q::No if accept => Step::new(Q::Yes, Some(true), Some(true)),
        Q::No => Step::new(Q::No, Some(false), None),
        Q::Yes => Step::new(Q::Yes, None, None),
        Q::WantNo { opposite: false } => Step {
            violation: true,
            ..Step::new(Q::No, None, None)
        },
        Q::WantNo { opposite: true } => Step {
            violation: true,
            ..Step::new(Q::Yes, None, None)
        },
        Q::WantYes { opposite: false } => Step::new(Q::Yes, None, Some(true)),
        Q::WantYes { opposite: true } => {
            Step::new(Q::WantNo { opposite: false }, Some(false), Some(true))
        }
    }
}

/// The peer asks to disable the option (WONT for the remote side, DONT for the local side).
pub fn on_disable(state: Q) -> Step {
    match state {
        Q::No => Step::new(Q::No, None, None),
        Q::Yes => Step::new(Q::No, Some(false), Some(false)),
        Q::WantNo { opposite: false } => Step::new(Q::No, None, Some(false)),
        Q::WantNo { opposite: true } => {
            Step::new(Q::WantYes { opposite: false }, Some(true), Some(false))
        }
        Q::WantYes { .. } => Step::new(Q::No, None, None),
    }
}

/// This side asks to enable or disable the option. Returns the next state and whether to send
/// the request now (`Some(true)` for DO or WILL, `Some(false)` for DONT or WONT).
pub fn request(state: Q, enable: bool) -> (Q, Option<bool>) {
    match (state, enable) {
        (Q::No, true) => (Q::WantYes { opposite: false }, Some(true)),
        (Q::Yes, false) => (Q::WantNo { opposite: false }, Some(false)),
        (Q::WantNo { .. }, true) => (Q::WantNo { opposite: true }, None),
        (Q::WantNo { .. }, false) => (Q::WantNo { opposite: false }, None),
        (Q::WantYes { .. }, false) => (Q::WantYes { opposite: true }, None),
        (Q::WantYes { .. }, true) => (Q::WantYes { opposite: false }, None),
        (s @ (Q::No | Q::Yes), _) => (s, None),
    }
}
