//! Terminal client for `mu`.
//!
//! The client follows the Elm architecture. Every input becomes a [`Msg`], [`update`] changes
//! the [`Model`] and returns the [`Effect`]s for the loop to perform, and [`view`] draws the
//! model by walking its layout tree.

pub mod convert;
mod diacritics;
mod editor;
mod keymap;
pub mod kitty;
mod layout;
mod model;
mod pane;
pub mod probe;
mod run;
pub mod screen;
pub mod store;
mod text;
mod update;
mod view;
pub mod width;
pub mod wrap;

pub use editor::{Editor, EditorId, EditorState};
pub use keymap::{Chord, Keymap, Wheel};
pub use layout::{Leaf, Node, Size};
pub use model::{Model, PaneId};
pub use pane::{Caps, OutputPane, PaneState, Position};
pub use run::{CALL_LIMIT, reply_of, run, within};
pub use update::{Action, Effect, Msg, Reply, update};
pub use view::view;
