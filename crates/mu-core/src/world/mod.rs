//! The configuration of the worlds: the layers, their merge, and the resolved world that the
//! sessions read.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::framer::{Charset, TelnetSettings};
use crate::id::WorldId;
use crate::secret::SecretRef;

mod builtin;
mod check;
mod dir;
mod error;
mod layer;
mod parse;
mod resolve;

pub use builtin::{BUILT_IN_TYPES, built_in_layers, parse_quirks};
pub use dir::{CLIENT_FILE, SERVER_FILE, TomlDirSource};
pub use error::{ConfigError, FieldError};
pub use layer::{
    KeepaliveFields, Layer, LayerFields, LayerKey, LayerOrigin, LayerSource, LoginFields,
    ReconnectFields, Trust,
};
pub use parse::{Hidden, parse as parse_toml};
pub use resolve::{FIELD_PATHS, resolve};

/// The login template of a world whose layers set none.
pub const DEFAULT_LOGIN_TEMPLATE: &str = "connect {login} {password}";

/// The defaults of a world, which the default layer of the merge holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldDefaults {
    /// The character set of the text.
    pub charset: Charset,
    /// The width of the window.
    pub columns: u16,
    /// The height of the window.
    pub rows: u16,
    /// Whether the connection uses TLS.
    pub tls: bool,
    /// The login template.
    pub login_template: &'static str,
    /// Whether the world connects at the start of the server.
    pub autoconnect: bool,
    /// The mode of the keepalive.
    pub keepalive_mode: KeepaliveMode,
    /// The waits of the reconnect, whose mode is [`ReconnectMode::Backoff`].
    pub reconnect: Backoff,
}

impl Default for WorldDefaults {
    /// The settings of [`TelnetSettings::default`], no TLS, [`DEFAULT_LOGIN_TEMPLATE`], no
    /// autoconnect, the keepalive mode `idle`, and a reconnect from 1 to 60 seconds that stops
    /// after 30 attempts.
    fn default() -> Self {
        let telnet = TelnetSettings::default();
        Self {
            charset: telnet.charset,
            columns: telnet.columns,
            rows: telnet.rows,
            tls: false,
            login_template: DEFAULT_LOGIN_TEMPLATE,
            autoconnect: false,
            keepalive_mode: KeepaliveMode::Idle,
            reconnect: Backoff {
                initial: 1,
                max: 60,
                attempts: 30,
            },
        }
    }
}

/// How a world logs in. Its template holds `{password}`, and neither the template nor the name
/// holds a CR or an LF, so the login is one line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Login {
    template: String,
    name: String,
    secret: SecretRef,
}

impl Login {
    /// A login that sends `template`, with `{login}` and `{password}` in it, as `name`, with
    /// the password from `secret`.
    ///
    /// # Errors
    ///
    /// A [`FieldError`] when `template` holds no `{password}`, or `template` or `name` holds a
    /// CR or an LF.
    pub fn new(
        template: impl Into<String>,
        name: impl Into<String>,
        secret: SecretRef,
    ) -> Result<Self, FieldError> {
        let (template, name) = (template.into(), name.into());
        check::check_login(Some(&name), Some(&template))?;
        Ok(Self {
            template,
            name,
            secret,
        })
    }

    /// The line to send, with `{login}` and `{password}` in it.
    #[must_use]
    pub fn template(&self) -> &str {
        &self.template
    }

    /// The login name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Where the password comes from.
    #[must_use]
    pub fn secret(&self) -> &SecretRef {
        &self.secret
    }

    /// The login line for `password`.
    #[must_use]
    pub fn line(&self, password: &str) -> String {
        self.template
            .replace("{login}", &self.name)
            .replace("{password}", password)
    }
}

/// The text that a game reads as a line break inside one command, such as `%r`. It holds 1 to
/// 16 characters and no control character, so a text joined with it stays one line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineBreak(String);

impl LineBreak {
    /// A line break of the text `text`.
    ///
    /// # Errors
    ///
    /// A [`FieldError`] when `text` is empty, holds more than 16 characters, or holds a control
    /// character.
    pub fn new(text: impl Into<String>) -> Result<Self, FieldError> {
        let text = text.into();
        check::check_line_break(&text)?;
        Ok(Self(text))
    }

    /// The text of the line break.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// When a keepalive sends its line: the value of `keepalive.mode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeepaliveMode {
    /// After the set seconds with no write to the game.
    #[default]
    Idle,
    /// Every set seconds.
    Interval,
    /// Never.
    Off,
}

/// When a [`Keepalive`] sends its line: [`KeepaliveMode`] with no `Off`, because a world with
/// no keepalive has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Schedule {
    Idle,
    Interval,
}

/// The keepalive of a world: a mode, a line to send, and the seconds of the mode, from 1 to
/// 86400. The line holds no CR and no LF.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keepalive {
    schedule: Schedule,
    command: String,
    seconds: u32,
}

impl Keepalive {
    /// A keepalive that sends `command` after `seconds` with no write: the mode
    /// [`KeepaliveMode::Idle`].
    ///
    /// # Errors
    ///
    /// A [`FieldError`] when `command` holds a CR or an LF, or `seconds` is not from 1 to 86400.
    pub fn new(command: impl Into<String>, seconds: u32) -> Result<Self, FieldError> {
        let command = command.into();
        check::check_keepalive(Some(&command), Some(seconds))?;
        Ok(Self {
            schedule: Schedule::Idle,
            command,
            seconds,
        })
    }

    /// The same keepalive with `mode`. None for [`KeepaliveMode::Off`], because a world with no
    /// keepalive holds none.
    #[must_use]
    pub fn with_mode(self, mode: KeepaliveMode) -> Option<Self> {
        let schedule = match mode {
            KeepaliveMode::Idle => Schedule::Idle,
            KeepaliveMode::Interval => Schedule::Interval,
            KeepaliveMode::Off => return None,
        };
        Some(Self { schedule, ..self })
    }

    /// The mode.
    #[must_use]
    pub fn mode(&self) -> KeepaliveMode {
        match self.schedule {
            Schedule::Idle => KeepaliveMode::Idle,
            Schedule::Interval => KeepaliveMode::Interval,
        }
    }

    /// The line to send.
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// The seconds between two sends.
    #[must_use]
    pub fn seconds(&self) -> u32 {
        self.seconds
    }

    /// The time between two sends.
    #[must_use]
    pub fn period(&self) -> Duration {
        Duration::from_secs(u64::from(self.seconds))
    }

    /// The time of the next keepalive line, from the move of the world to `connected`, the last
    /// line written to the game and the last keepalive line. A keepalive line is also a line
    /// written to the game.
    #[must_use]
    pub fn deadline(
        &self,
        connected: Instant,
        last_write: Option<Instant>,
        last_keepalive: Option<Instant>,
    ) -> Instant {
        let start = match self.schedule {
            Schedule::Idle => [last_write, last_keepalive]
                .into_iter()
                .flatten()
                .fold(connected, Instant::max),
            Schedule::Interval => last_keepalive.map_or(connected, |last| last.max(connected)),
        };
        start + self.period()
    }
}

/// The value of `reconnect.mode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReconnectMode {
    /// Waits longer before each attempt.
    #[default]
    Backoff,
    /// Never reconnects.
    Off,
}

/// The waits of a reconnect: the first wait and the cap, in seconds, and the number of failed
/// attempts in a row before the world stops. The first wait is from 1 to 3600 and not greater
/// than the cap, the cap is from 1 to 86400, and the attempts are from 0 to 10000. With 0
/// attempts, the world never stops.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Backoff {
    initial: u32,
    max: u32,
    attempts: u32,
}

impl Backoff {
    /// A backoff from `initial` seconds up to `max` seconds, which stops after `attempts`.
    ///
    /// # Errors
    ///
    /// A [`FieldError`] when a value is out of its range, or `initial` is greater than `max`.
    pub fn new(initial: u32, max: u32, attempts: u32) -> Result<Self, FieldError> {
        check::check_reconnect(Some(initial), Some(max), Some(attempts))?;
        check::check_first_wait(initial, max)?;
        Ok(Self {
            initial,
            max,
            attempts,
        })
    }

    /// The first wait, in seconds.
    #[must_use]
    pub fn initial(&self) -> u32 {
        self.initial
    }

    /// The cap of each wait, in seconds.
    #[must_use]
    pub fn max(&self) -> u32 {
        self.max
    }

    /// The number of failed attempts in a row before the world stops, or 0 for no limit.
    #[must_use]
    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    /// The wait before the attempt `attempt`, from 1: `min(initial * 2^(attempt-1), max)`
    /// seconds, plus an extra time. `draw` sets the extra time, from 0 for none to `u32::MAX`
    /// for `min(0.05 * attempt, 0.20)` times the wait. Attempt 0 counts as attempt 1.
    #[must_use]
    pub fn wait(&self, attempt: u32, draw: u32) -> Duration {
        let max = u64::from(self.max);
        let base = 1_u64
            .checked_shl(attempt.saturating_sub(1))
            .and_then(|power| power.checked_mul(u64::from(self.initial)))
            .map_or(max, |wait| wait.min(max));
        let share = (0.05 * f64::from(attempt.max(1))).min(0.20);
        let extra = share * f64::from(draw) / f64::from(u32::MAX);
        Duration::from_secs(base).mul_f64(1.0 + extra)
    }

    /// Whether a connection that stayed up for `up` starts the count of attempts again.
    #[must_use]
    pub fn resets(&self, up: Duration) -> bool {
        up >= Duration::from_secs(u64::from(self.max))
    }

    /// Whether the world stops after `failed` failed attempts in a row.
    #[must_use]
    pub fn stops(&self, failed: u32) -> bool {
        self.attempts != 0 && failed >= self.attempts
    }
}

/// Whether a world reconnects after a drop that the player did not ask for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reconnect {
    /// It reconnects with these waits.
    Backoff(Backoff),
    /// It does not reconnect.
    Off,
}

impl Reconnect {
    /// The wait before the attempt `attempt`, as [`Backoff::wait`] gives it, or none with no
    /// reconnect.
    #[must_use]
    pub fn wait(&self, attempt: u32, draw: u32) -> Option<Duration> {
        match self {
            Self::Backoff(backoff) => Some(backoff.wait(attempt, draw)),
            Self::Off => None,
        }
    }
}

/// Whether an entry is a rule or a script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    /// An entry of `rules`.
    Rule,
    /// An entry of `scripts`.
    Script,
}

/// One rule or script: its id, the rest of its table, and the layer that added it.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// The id, unique among the rules and the scripts of a world.
    pub id: String,
    /// The whole table of the entry, with its `id`.
    pub table: toml::Table,
    /// The layer that added the entry.
    pub origin: LayerOrigin,
}

/// One world after the merge of its configuration layers.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedWorld {
    /// The id of the world.
    pub id: WorldId,
    /// The host of the game. The resolver never gives an empty host, but
    /// [`ResolvedWorld::new`] does.
    pub host: String,
    /// The port of the game. The resolver never gives port 0, but [`ResolvedWorld::new`] does.
    pub port: u16,
    /// Whether the connection uses TLS.
    pub tls: bool,
    /// Whether the world connects at the start of the server.
    pub autoconnect: bool,
    /// The character set of the text.
    pub charset: Charset,
    /// The width of the window.
    pub columns: u16,
    /// The height of the window.
    pub rows: u16,
    /// The text that the game reads as a line break inside one command, or none for a world
    /// that takes only a text of one line.
    pub line_break: Option<LineBreak>,
    /// The login, or none for a world that logs in by hand.
    pub login: Option<Login>,
    /// The mode of `keepalive.mode`, also for a world with no keepalive.
    pub keepalive_mode: KeepaliveMode,
    /// The keepalive, or none for the mode `off` or a world with no command and no seconds.
    pub keepalive: Option<Keepalive>,
    /// Whether the world reconnects after a drop.
    pub reconnect: Reconnect,
    /// The rules, in the order that their ids were added. An override keeps its place, and an
    /// id that is disabled and added again moves to the end.
    pub rules: Vec<Entry>,
    /// The scripts, in the same order as the rules.
    pub scripts: Vec<Entry>,
    /// The source of each field that the world has, by its path such as `login.name`.
    pub sources: BTreeMap<String, LayerOrigin>,
}

impl ResolvedWorld {
    /// A world with the values of [`WorldDefaults`], no host, no login and no source.
    #[must_use]
    pub fn new(id: WorldId) -> Self {
        let defaults = WorldDefaults::default();
        Self {
            id,
            host: String::new(),
            port: 0,
            tls: defaults.tls,
            autoconnect: defaults.autoconnect,
            charset: defaults.charset,
            columns: defaults.columns,
            rows: defaults.rows,
            line_break: None,
            login: None,
            keepalive_mode: defaults.keepalive_mode,
            keepalive: None,
            reconnect: Reconnect::Backoff(defaults.reconnect),
            rules: Vec::new(),
            scripts: Vec::new(),
            sources: BTreeMap::new(),
        }
    }

    /// A world that logs in with the default template and its character part as the name.
    #[must_use]
    pub fn with_default_login(id: WorldId, secret: SecretRef) -> Self {
        // The default template and the id rule pass the checks of `Login::new`.
        let login = Login {
            template: WorldDefaults::default().login_template.to_owned(),
            name: id.character().to_owned(),
            secret,
        };
        Self {
            login: Some(login),
            ..Self::new(id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keepalive(mode: KeepaliveMode) -> Keepalive {
        Keepalive::new("IDLE", 60).unwrap().with_mode(mode).unwrap()
    }

    fn secs(seconds: u64) -> Duration {
        Duration::from_secs(seconds)
    }

    #[test]
    fn idle_deadlines() {
        let t0 = Instant::now();
        let idle = keepalive(KeepaliveMode::Idle);
        for (last_write, last_keepalive, next) in [
            (None, None, 60),
            (Some(30), None, 90),
            (Some(30), Some(90), 150),
            (Some(100), Some(90), 160),
        ] {
            let at = |seconds: Option<u64>| seconds.map(|seconds| t0 + secs(seconds));
            assert_eq!(
                idle.deadline(t0, at(last_write), at(last_keepalive)),
                t0 + secs(next),
                "{last_write:?} {last_keepalive:?}"
            );
        }
    }

    #[test]
    fn interval_deadlines() {
        let t0 = Instant::now();
        let interval = keepalive(KeepaliveMode::Interval);
        for (last_write, last_keepalive, next) in [
            (None, None, 60),
            (Some(30), None, 60),
            (Some(70), Some(60), 120),
        ] {
            let at = |seconds: Option<u64>| seconds.map(|seconds| t0 + secs(seconds));
            assert_eq!(
                interval.deadline(t0, at(last_write), at(last_keepalive)),
                t0 + secs(next),
                "{last_write:?} {last_keepalive:?}"
            );
        }
    }

    fn backoff(initial: u32, max: u32) -> Backoff {
        Backoff::new(initial, max, 30).unwrap()
    }

    /// `min(initial * 2^(attempt-1), max)` in seconds, computed apart from [`Backoff::wait`].
    fn base_wait(initial: u32, max: u32, attempt: u32) -> f64 {
        let shift = attempt - 1;
        let wait = if shift >= 64 {
            u128::from(max)
        } else {
            (u128::from(initial) << shift).min(u128::from(max))
        };
        #[expect(
            clippy::cast_precision_loss,
            reason = "the wait is at most 86400 seconds"
        )]
        let wait = wait as f64;
        wait
    }

    #[test]
    fn waits_double_up_to_the_cap() {
        let waits: Vec<u64> = (1..=8)
            .map(|attempt| backoff(1, 60).wait(attempt, 0).as_secs())
            .collect();
        assert_eq!(waits, [1, 2, 4, 8, 16, 32, 60, 60]);
    }

    #[test]
    fn largest_extra_time_of_each_attempt() {
        for (attempt, share) in [(1, 0.05), (2, 0.10), (3, 0.15), (4, 0.20), (5, 0.20)] {
            let base = base_wait(1, 60, attempt);
            let wait = backoff(1, 60).wait(attempt, u32::MAX).as_secs_f64();
            assert!(
                (wait - base * (1.0 + share)).abs() < 1e-9,
                "attempt {attempt}: {wait}"
            );
        }
    }

    #[test]
    fn stable_connection_resets_the_count() {
        assert!(backoff(1, 60).resets(secs(60)));
    }

    #[test]
    fn short_connection_does_not_reset_the_count() {
        assert!(!backoff(1, 60).resets(secs(59)));
    }

    #[test]
    fn stops_after_the_last_attempt() {
        let three = Backoff::new(1, 60, 3).unwrap();
        assert_eq!([2, 3].map(|failed| three.stops(failed)), [false, true]);
        let no_limit = Backoff::new(1, 60, 0).unwrap();
        assert!(!no_limit.stops(u32::MAX));
    }

    #[test]
    fn reconnect_off_has_no_wait() {
        assert_eq!(Reconnect::Off.wait(1, 0), None);
    }

    proptest::proptest! {
        #[test]
        fn extra_time_reaches_its_cap(
            attempt in proptest::prop_oneof![
                proptest::strategy::Just(4_u32),
                proptest::strategy::Just(u32::MAX),
                4_u32..=u32::MAX,
            ],
            draws in proptest::collection::vec(proptest::arbitrary::any::<u32>(), 1000),
        ) {
            for (initial, max) in [(1, 60), (3_600, 86_400)] {
                let base = base_wait(initial, max, attempt);
                let mut top = 0.0_f64;
                for &draw in &draws {
                    let wait = backoff(initial, max).wait(attempt, draw).as_secs_f64();
                    proptest::prop_assert!(wait >= base, "{wait} < {base}");
                    proptest::prop_assert!(wait <= base * 1.20 + 1e-6, "{wait} > 1.2 * {base}");
                    top = top.max(wait / base - 1.0);
                }
                proptest::prop_assert!(top > 0.15, "the extra time reached only {top}");
            }
        }
    }

    #[test]
    fn off_is_no_keepalive() {
        let keepalive = Keepalive::new("IDLE", 60).unwrap();
        assert_eq!(keepalive.with_mode(KeepaliveMode::Off), None);
    }

    #[test]
    fn mode_round_trips() {
        for mode in [KeepaliveMode::Idle, KeepaliveMode::Interval] {
            let keepalive = Keepalive::new("IDLE", 60).unwrap().with_mode(mode).unwrap();
            assert_eq!(keepalive.mode(), mode);
        }
    }

    #[test]
    fn default_login_uses_the_character_part() {
        let world = ResolvedWorld::with_default_login(
            "alpha/aria".parse().unwrap(),
            "env:ALPHA_PW".parse().unwrap(),
        );
        assert_eq!(world.login.unwrap().line("pw"), "connect aria pw");
    }

    #[test]
    fn login_refuses_a_bad_template_or_name() {
        let secret = || "env:PW".parse::<SecretRef>().unwrap();
        for (template, name, field) in [
            ("connect {login}", "aria", "login.template"),
            ("connect {login} {password}\nlook", "aria", "login.template"),
            ("connect {login} {password}", "aria\r", "login.name"),
        ] {
            let error = Login::new(template, name, secret()).unwrap_err();
            assert_eq!(error.field(), field, "{template:?} {name:?}");
        }
        assert!(Login::new("connect {login} {password}", "aria", secret()).is_ok());
    }

    #[test]
    fn keepalive_refuses_a_bad_command_or_period() {
        for (command, seconds) in [("idle", 0), ("idle", 86_401), ("idle\n", 60)] {
            assert!(
                Keepalive::new(command, seconds).is_err(),
                "{command:?} {seconds}"
            );
        }
        let keepalive = Keepalive::new("idle", 86_400).unwrap();
        assert_eq!(keepalive.period().as_secs(), 86_400);
    }
}
