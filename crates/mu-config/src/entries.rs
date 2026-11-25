//! The names of the entries that a key can run.

/// The name of one entry, and whether it needs an argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntryName {
    /// The name, in lowercase words joined with `-`.
    pub name: &'static str,
    /// Whether the entry refuses to run without an argument, so that no key can run it.
    pub needs_arg: bool,
}

const fn plain(name: &'static str) -> EntryName {
    EntryName {
        name,
        needs_arg: false,
    }
}

const fn needs(name: &'static str) -> EntryName {
    EntryName {
        name,
        needs_arg: true,
    }
}

/// Every entry of the terminal client, in the order of its registry.
pub const ENTRY_NAMES: &[EntryName] = &[
    plain("send"),
    plain("send-lines"),
    plain("newline"),
    plain("join-lines"),
    plain("delete-to-line-start"),
    plain("undo"),
    plain("redo"),
    plain("ignore"),
    plain("history-prev"),
    plain("history-next"),
    plain("scroll-page-up"),
    plain("scroll-page-down"),
    plain("scroll-up"),
    plain("scroll-down"),
    plain("scroll-top"),
    plain("scroll-bottom"),
    plain("focus-1"),
    plain("focus-2"),
    plain("focus-3"),
    plain("focus-4"),
    plain("focus-5"),
    plain("focus-6"),
    plain("focus-7"),
    plain("focus-8"),
    plain("focus-9"),
    plain("focus-next"),
    plain("focus-prev"),
    plain("editor-next"),
    EntryName {
        name: "focus",
        needs_arg: true,
    },
    plain("connect"),
    plain("disconnect"),
    needs("place"),
    needs("pane-hide"),
    needs("pane-size"),
    plain("pane-next"),
    needs("layout-use"),
    needs("layout-save"),
    plain("layout-list"),
    plain("layout-next"),
    plain("layout-plan"),
    needs("layout-insert"),
    needs("layout-delete"),
    needs("layout-edit"),
    plain("layout-here"),
    plain("arrange"),
    plain("help"),
    plain("quit"),
];

impl EntryName {
    /// The entry named `name`.
    #[must_use]
    pub fn named(name: &str) -> Option<&'static Self> {
        ENTRY_NAMES.iter().find(|entry| entry.name == name)
    }
}
