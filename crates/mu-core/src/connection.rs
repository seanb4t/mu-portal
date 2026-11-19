//! The rules of one connection of a world: when the login goes out.

use std::fmt::Debug;
use std::time::Duration;

use tokio::time::Instant;

/// When a world sends its login line. Each kind is pure: it gives a deadline from times that
/// the session passes, and holds no timer.
pub trait LoginMoment: Debug + Send + Sync {
    /// The time of the login line, from the time of the connect and the time of the last byte
    /// from the game. `last_byte` is `None` before the first byte.
    fn deadline(&self, connect: Instant, last_byte: Option<Instant>) -> Instant;
}

/// The login goes out at the connect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Immediate;

impl LoginMoment for Immediate {
    fn deadline(&self, connect: Instant, _last_byte: Option<Instant>) -> Instant {
        connect
    }
}

/// The login goes out after a quiet time with no byte from the game, and at the latest at a cap
/// after the connect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quiet {
    quiet: Duration,
    cap: Duration,
}

impl Quiet {
    /// A login after `quiet` with no byte, and at the latest `cap` after the connect.
    #[must_use]
    pub fn new(quiet: Duration, cap: Duration) -> Self {
        Self { quiet, cap }
    }
}

impl Default for Quiet {
    /// One quiet second, and at the latest 10 seconds after the connect.
    fn default() -> Self {
        Self::new(Duration::from_secs(1), Duration::from_secs(10))
    }
}

impl LoginMoment for Quiet {
    fn deadline(&self, connect: Instant, last_byte: Option<Instant>) -> Instant {
        let last = last_byte.map_or(connect, |last_byte| last_byte.max(connect));
        (last + self.quiet).min(connect + self.cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn immediate_logs_in_at_the_connect() {
        let connect = Instant::now();
        assert_eq!(
            Immediate.deadline(connect, Some(connect + ms(300))),
            connect
        );
    }

    #[test]
    fn quiet_after_a_banner() {
        let connect = Instant::now();
        let deadline = Quiet::default().deadline(connect, Some(connect + ms(300)));
        assert_eq!(deadline, connect + ms(1_300));
    }

    #[test]
    fn quiet_for_a_silent_game() {
        let connect = Instant::now();
        assert_eq!(
            Quiet::default().deadline(connect, None),
            connect + ms(1_000)
        );
    }

    #[test]
    fn quiet_for_a_game_that_is_never_quiet() {
        let connect = Instant::now();
        let deadline = Quiet::default().deadline(connect, Some(connect + ms(9_500)));
        assert_eq!(deadline, connect + ms(10_000));
    }

    #[test]
    fn quiet_never_passes_the_cap() {
        let connect = Instant::now();
        let deadline = Quiet::default().deadline(connect, Some(connect + ms(60_000)));
        assert_eq!(deadline, connect + ms(10_000));
    }
}
