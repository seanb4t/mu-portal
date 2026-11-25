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
mod world_id;

pub use entries::{ENTRY_NAMES, EntryName};
pub use filter::{ControlMap, REPORT, STORE, filter};
pub use key::{Key, KeyName, KeyNameError, Modifiers};
pub use label::{FieldError, LABEL_DEFAULT, LABEL_MAX, Label, LabelField, Segment};
pub use layer::{
    Binding, ClientLayer, Echo, FIELD_PATHS, KEYS_TABLE, KeyError, NONE, Statusline, Tabs,
    WORLD_LAYOUTS_TABLE, check_fields, check_keys, keyed_path,
};
pub use names::{DEVICE_MAX, DeviceName, Kind, LayoutName, NameError, Os, listed};
pub use world_id::{
    BadWorldId, BadWorldKey, Choice, PART_RULE, WorldId, WorldKey, is_part, layout_for,
};
