//! The client configuration of mu: the fields of a client layer, the grammar of key names, the
//! names of the entries that a key can run, and the names of a client. The server checks a
//! layer with this crate, and the terminal client applies it.

mod entries;
mod filter;
mod key;
mod label;
mod layer;
pub mod layout;
mod names;

pub use entries::{ENTRY_NAMES, EntryName};
pub use filter::{ControlMap, REPORT, STORE, filter};
pub use key::{Key, KeyName, KeyNameError, Modifiers};
pub use label::{FieldError, LABEL_DEFAULT, LABEL_MAX, Label, LabelField, Segment};
pub use layer::{Binding, ClientLayer, Echo, FIELD_PATHS, KeyError, NONE, check_keys};
pub use names::{DEVICE_MAX, DeviceName, Kind, NameError, Os, listed};
