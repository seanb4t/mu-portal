//! Terminal client for `mu`.
//!
//! The client follows the Elm architecture. Every input becomes a [`Msg`], [`update`] changes
//! the [`Model`] and returns the [`Effect`]s for the loop to perform, and [`view`] draws the
//! model by walking its layout tree.

pub mod command;
pub mod config;
pub mod convert;
mod diacritics;
mod editor;
pub mod history;
mod keymap;
pub mod kitty;
mod layout;
mod model;
mod pane;
pub mod probe;
mod registry;
mod run;
pub mod screen;
pub mod store;
mod text;
mod update;
mod view;
pub mod width;
pub mod wrap;

pub use config::{ClientConfig, Echo, KeyError, Settings};
pub use editor::{Editor, EditorId, EditorState};
pub use keymap::{Chord, KeyNameError, Keymap, Wheel};
pub use layout::{Leaf, Node, Size};
pub use model::{Model, PaneId};
pub use pane::{Caps, OutputPane, PaneState, Position};
pub use registry::{Arg, ArgKind, ENTRIES, Entry, EntryId, candidates};
pub use run::{CALL_LIMIT, control_of, reply_of, run, within};
pub use text::echo_rows;
pub use update::{Action, Effect, Msg, Reply, update};
pub use view::view;
