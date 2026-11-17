//! Option negotiation with the Q method of RFC 1143, section 7.
//!
//! Each option has one state on each side. A step is a pure function of the state and the
//! message, so a test can walk each row of the table.

/// The state of one option on one side.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Q {
    /// The option is off.
    #[default]
    No,
    /// The option is on.
    Yes,
    /// A request to turn the option off is open. `opposite` queues a request to turn it on.
    WantNo {
        /// A queued request for the other state.
        opposite: bool,
    },
    /// A request to turn the option on is open. `opposite` queues a request to turn it off.
    WantYes {
        /// A queued request for the other state.
        opposite: bool,
    },
}

/// What follows a message of the peer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Step {
    /// The next state.
    pub next: Q,
    /// `Some(true)` sends the positive reply (DO or WILL), `Some(false)` the negative one.
    pub reply: Option<bool>,
    /// `Some(true)` reports that the option is now on, `Some(false)` that it is now off.
    pub report: Option<bool>,
    /// The peer answered a request that this side never sent.
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

    const fn violation(next: Q) -> Self {
        Self {
            next,
            reply: None,
            report: None,
            violation: true,
        }
    }
}

/// The peer asks to turn the option on: WILL for the game side, DO for the client side.
/// `accept` says whether the policy accepts the option on this side.
#[must_use]
pub fn on_enable(state: Q, accept: bool) -> Step {
    match state {
        Q::No if accept => Step::new(Q::Yes, Some(true), Some(true)),
        Q::No => Step::new(Q::No, Some(false), None),
        Q::Yes => Step::new(Q::Yes, None, None),
        Q::WantNo { opposite: false } => Step::violation(Q::No),
        Q::WantNo { opposite: true } => Step::violation(Q::Yes),
        Q::WantYes { opposite: false } => Step::new(Q::Yes, None, Some(true)),
        Q::WantYes { opposite: true } => {
            Step::new(Q::WantNo { opposite: false }, Some(false), Some(true))
        }
    }
}

/// The peer asks to turn the option off: WONT for the game side, DONT for the client side.
#[must_use]
pub fn on_disable(state: Q) -> Step {
    match state {
        Q::No | Q::WantYes { .. } => Step::new(Q::No, None, None),
        Q::Yes => Step::new(Q::No, Some(false), Some(false)),
        Q::WantNo { opposite: false } => Step::new(Q::No, None, Some(false)),
        Q::WantNo { opposite: true } => {
            Step::new(Q::WantYes { opposite: false }, Some(true), Some(false))
        }
    }
}

/// This side asks to turn the option on or off. Gives the next state, and the request to send
/// now: `Some(true)` for DO or WILL, `Some(false)` for DONT or WONT.
#[must_use]
pub fn request(state: Q, enable: bool) -> (Q, Option<bool>) {
    match (state, enable) {
        (Q::No, true) => (Q::WantYes { opposite: false }, Some(true)),
        (Q::Yes, false) => (Q::WantNo { opposite: false }, Some(false)),
        (Q::WantNo { .. }, true) => (Q::WantNo { opposite: true }, None),
        (Q::WantNo { .. }, false) => (Q::WantNo { opposite: false }, None),
        (Q::WantYes { .. }, false) => (Q::WantYes { opposite: true }, None),
        (Q::WantYes { .. }, true) => (Q::WantYes { opposite: false }, None),
        (state @ (Q::No | Q::Yes), _) => (state, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: Q = Q::WantNo { opposite: false };
    const NO_OPPOSITE: Q = Q::WantNo { opposite: true };
    const YES_EMPTY: Q = Q::WantYes { opposite: false };
    const YES_OPPOSITE: Q = Q::WantYes { opposite: true };

    fn step(next: Q, reply: Option<bool>, report: Option<bool>) -> Step {
        Step::new(next, reply, report)
    }

    #[test]
    fn will_in_no_when_accepted_sends_do() {
        assert_eq!(on_enable(Q::No, true), step(Q::Yes, Some(true), Some(true)));
    }

    #[test]
    fn will_in_no_when_refused_sends_dont() {
        assert_eq!(on_enable(Q::No, false), step(Q::No, Some(false), None));
    }

    #[test]
    fn will_in_yes_is_ignored() {
        assert_eq!(on_enable(Q::Yes, true), step(Q::Yes, None, None));
    }

    #[test]
    fn will_in_wantno_empty_is_an_error() {
        assert_eq!(on_enable(EMPTY, true), Step::violation(Q::No));
    }

    #[test]
    fn will_in_wantno_opposite_is_an_error() {
        assert_eq!(on_enable(NO_OPPOSITE, true), Step::violation(Q::Yes));
    }

    #[test]
    fn will_in_wantyes_empty_turns_on() {
        assert_eq!(on_enable(YES_EMPTY, false), step(Q::Yes, None, Some(true)));
    }

    #[test]
    fn will_in_wantyes_opposite_asks_to_turn_off() {
        assert_eq!(
            on_enable(YES_OPPOSITE, true),
            step(EMPTY, Some(false), Some(true))
        );
    }

    #[test]
    fn wont_in_no_is_ignored() {
        assert_eq!(on_disable(Q::No), step(Q::No, None, None));
    }

    #[test]
    fn wont_in_yes_sends_dont() {
        assert_eq!(on_disable(Q::Yes), step(Q::No, Some(false), Some(false)));
    }

    #[test]
    fn wont_in_wantno_empty_turns_off() {
        assert_eq!(on_disable(EMPTY), step(Q::No, None, Some(false)));
    }

    #[test]
    fn wont_in_wantno_opposite_asks_to_turn_on() {
        assert_eq!(
            on_disable(NO_OPPOSITE),
            step(YES_EMPTY, Some(true), Some(false))
        );
    }

    #[test]
    fn wont_in_wantyes_stays_off() {
        assert_eq!(on_disable(YES_EMPTY), step(Q::No, None, None));
        assert_eq!(on_disable(YES_OPPOSITE), step(Q::No, None, None));
    }

    #[test]
    fn request_to_enable_follows_the_table() {
        assert_eq!(request(Q::No, true), (YES_EMPTY, Some(true)));
        assert_eq!(request(Q::Yes, true), (Q::Yes, None));
        assert_eq!(request(EMPTY, true), (NO_OPPOSITE, None));
        assert_eq!(request(NO_OPPOSITE, true), (NO_OPPOSITE, None));
        assert_eq!(request(YES_EMPTY, true), (YES_EMPTY, None));
        assert_eq!(request(YES_OPPOSITE, true), (YES_EMPTY, None));
    }

    #[test]
    fn request_to_disable_follows_the_table() {
        assert_eq!(request(Q::No, false), (Q::No, None));
        assert_eq!(request(Q::Yes, false), (EMPTY, Some(false)));
        assert_eq!(request(EMPTY, false), (EMPTY, None));
        assert_eq!(request(NO_OPPOSITE, false), (EMPTY, None));
        assert_eq!(request(YES_EMPTY, false), (YES_OPPOSITE, None));
        assert_eq!(request(YES_OPPOSITE, false), (YES_OPPOSITE, None));
    }
}
