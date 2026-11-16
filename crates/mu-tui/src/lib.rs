//! Terminal client for `mu`.

mod app;
mod run;

pub use app::{Action, App, Reply};
pub use run::run;
