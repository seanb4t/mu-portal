//! Hooks and handlers: code that changes lines and reacts to events. The binary registers
//! none.

use crate::command::Command;
use crate::event::Event;

/// The longest chain of handler commands. A command past it is dropped.
pub const DEPTH_LIMIT: u8 = 8;

/// The commands from handlers that a world accepts in one second, and its burst.
pub const RATE_PER_SECOND: u32 = 20;

/// What a hook does with a line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HookAction {
    /// The line goes on as it is.
    Pass,
    /// The line goes on with this text.
    Replace(String),
    /// The line stops here: it makes no event and does not reach the game.
    Gag,
}

/// Changes a line before the journal records it. A hook does no I/O.
pub trait Hook: Send + Sync + 'static {
    /// Runs on each line from the game.
    fn on_game_line(&self, _line: &str) -> HookAction {
        HookAction::Pass
    }

    /// Runs on each line from the player.
    fn on_player_line(&self, _line: &str) -> HookAction {
        HookAction::Pass
    }
}

/// Reacts to a recorded event with commands. A handler does no I/O.
pub trait Handler: Send + Sync + 'static {
    /// The commands for `event`.
    fn on_event(&self, event: &Event) -> Vec<Command>;
}

/// Runs each hook in order. A replace changes the text for the next hook, and a gag stops.
pub(crate) fn run_hooks(
    hooks: &[std::sync::Arc<dyn Hook>],
    line: &str,
    apply: impl Fn(&dyn Hook, &str) -> HookAction,
) -> Option<String> {
    let mut line = line.to_owned();
    for hook in hooks {
        match apply(hook.as_ref(), &line) {
            HookAction::Pass => {}
            HookAction::Replace(text) => line = text,
            HookAction::Gag => return None,
        }
    }
    Some(line)
}
