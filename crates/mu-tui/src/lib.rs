//! Terminal client for `mu`.
//!
//! The client follows the Elm architecture. Every input becomes a [`Msg`], [`update`] changes
//! the [`Model`] and returns the [`Effect`]s for the loop to perform, and [`view`] draws the
//! model. The layout of the screen is a plan of `mu-config`, and its solver gives each pane its
//! rectangle.

pub mod arrange;
pub mod command;
pub mod config;
pub mod convert;
mod diacritics;
mod editor;
pub mod history;
mod keymap;
pub mod kitty;
mod layouts;
mod model;
mod pane;
pub mod probe;
mod registry;
mod run;
pub mod screen;
pub mod statusline;
pub mod store;
mod text;
mod update;
mod view;
pub mod width;
pub mod wrap;

pub use config::Echo;
pub use editor::{Editor, EditorState};
pub use keymap::{Chord, Keymap, Wheel};
pub use model::{Model, PaneId};
pub use pane::{Caps, OutputPane, PaneState, Position};
pub use registry::{Arg, ArgKind, ENTRIES, Entry, EntryId, candidates};
pub use run::{CALL_LIMIT, control_of, ended, reply_of, run, timed, within};
pub use text::echo_rows;
pub use update::{Action, Effect, Msg, Reply, TICK, Then, Timer, update};
pub use view::{catalog_blocks, drawn_blocks, solved, solved_now, view};
