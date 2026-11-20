//! Terminal client for `mu`.
//!
//! The client follows the Elm architecture. Every input becomes a [`Msg`], [`update`] changes
//! the [`Model`] and returns the [`Effect`]s for the loop to perform, and [`view`] draws the
//! model by walking its layout tree.

mod keymap;
mod layout;
mod model;
mod pane;
mod run;
mod text;
mod update;
mod view;

pub use keymap::{Chord, Keymap};
pub use layout::{Leaf, Node};
pub use model::{Model, PaneId};
pub use pane::{OutputPane, PaneState};
pub use run::run;
pub use update::{Action, Effect, Msg, Reply, update};
pub use view::view;
