//! The registry: one table of the entries that the player can run. A key of the keymap and a
//! client command both choose an entry, and [`update`](crate::update) builds its [`Action`].

use crate::update::Action;

/// The kind of value that an argument takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgKind {
    /// The id of a world, such as `alpha/aria`.
    World,
}

/// The argument of an entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arg {
    /// The kind of its value.
    pub kind: ArgKind,
    /// Whether the entry refuses to run without it.
    pub required: bool,
}

/// One thing that the player can run.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    /// The name, in lowercase words joined with `-`.
    pub name: &'static str,
    /// What the entry does, in one line.
    pub description: &'static str,
    /// The argument, or `None` for an entry that takes none.
    pub arg: Option<Arg>,
    /// Whether the entry acts on the text of the pose editor. Such an entry does not run as a
    /// command, because the command is that text.
    pub edits: bool,
    /// The action of the entry with its argument.
    pub build: fn(Option<String>) -> Action,
}

/// The place of an entry in [`ENTRIES`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntryId(usize);

impl EntryId {
    /// The entry at this place.
    #[must_use]
    pub fn entry(self) -> &'static Entry {
        &ENTRIES[self.0]
    }

    /// The id of the entry named `name`.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        ENTRIES
            .iter()
            .position(|entry| entry.name == name)
            .map(Self)
    }

    /// The ids of every entry, in the order of the table.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..ENTRIES.len()).map(Self)
    }

    /// The id of the entry named `name`, for a name of the table.
    pub(crate) fn of(name: &str) -> Self {
        Self::named(name).unwrap_or_else(|| panic!("no entry is named {name}"))
    }
}

/// An entry that takes no argument.
const fn plain(
    name: &'static str,
    description: &'static str,
    edits: bool,
    build: fn(Option<String>) -> Action,
) -> Entry {
    Entry {
        name,
        description,
        arg: None,
        edits,
        build,
    }
}

/// The entry `focus-<n>` for the place `n` of the top row.
macro_rules! focus_place {
    ($n:literal) => {
        plain(
            concat!("focus-", $n),
            concat!("give the focus to world ", $n, " of the top row"),
            false,
            |_| Action::FocusIndex($n),
        )
    };
}

/// Every entry, in the order that the help lists them.
pub static ENTRIES: &[Entry] = &[
    plain(
        "quit",
        "leave the client; a second quit drops the lines not sent yet",
        false,
        |_| Action::Quit,
    ),
    plain(
        "focus-next",
        "give the focus to the next world",
        false,
        |_| Action::FocusNext,
    ),
    plain(
        "focus-prev",
        "give the focus to the previous world",
        false,
        |_| Action::FocusPrev,
    ),
    focus_place!(1),
    focus_place!(2),
    focus_place!(3),
    focus_place!(4),
    focus_place!(5),
    focus_place!(6),
    focus_place!(7),
    focus_place!(8),
    focus_place!(9),
    plain(
        "send",
        "send the text of the editor as one line",
        true,
        |_| Action::Submit,
    ),
    plain(
        "send-lines",
        "send each line of the editor as its own line",
        true,
        |_| Action::SubmitLines,
    ),
    plain("newline", "insert a line break", true, |_| Action::Newline),
    plain(
        "join-lines",
        "join the lines with the line break of the world",
        true,
        |_| Action::JoinLines,
    ),
    plain(
        "delete-to-line-start",
        "delete from the start of the line to the cursor",
        true,
        |_| Action::DeleteToLineStart,
    ),
    plain("undo", "undo the last edit", true, |_| Action::Undo),
    plain(
        "redo",
        "bring back the edit that undo removed",
        true,
        |_| Action::Redo,
    ),
    plain("ignore", "do nothing", false, |_| Action::Ignore),
    plain(
        "scroll-page-up",
        "scroll the pane back one page",
        false,
        |_| Action::ScrollPage(-1),
    ),
    plain(
        "scroll-page-down",
        "scroll the pane forward one page",
        false,
        |_| Action::ScrollPage(1),
    ),
    plain(
        "scroll-up",
        "scroll the pane back three rows",
        false,
        |_| Action::ScrollRows(-3),
    ),
    plain(
        "scroll-down",
        "scroll the pane forward three rows",
        false,
        |_| Action::ScrollRows(3),
    ),
    plain(
        "scroll-top",
        "scroll the pane to its oldest line",
        false,
        |_| Action::ScrollTop,
    ),
    plain(
        "scroll-bottom",
        "return the pane to the bottom",
        false,
        |_| Action::ScrollBottom,
    ),
];
