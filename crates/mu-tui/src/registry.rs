//! The registry: one table of the entries that the player can run. A key of the keymap and a
//! client command both choose an entry, and [`update`](crate::update) builds its [`Action`].

use crate::model::Model;
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
    /// command, because the command is that text. `ignore` has it too, so `/ignore` is no
    /// command.
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

/// An entry that takes a world, the world in focus if `required` is false and no world is given.
const fn on_world(
    name: &'static str,
    description: &'static str,
    required: bool,
    build: fn(Option<String>) -> Action,
) -> Entry {
    Entry {
        name,
        description,
        arg: Some(Arg {
            kind: ArgKind::World,
            required,
        }),
        edits: false,
        build,
    }
}

/// Every entry, in the order that the help lists them. The entries that the player uses most
/// come last, so the end of the help, next to the editor, shows them.
pub static ENTRIES: &[Entry] = &[
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
    plain("ignore", "do nothing", true, |_| Action::Ignore),
    plain(
        "history-prev",
        "on the first line, put the previous entry of the history in the editor",
        true,
        |_| Action::HistoryPrev,
    ),
    plain(
        "history-next",
        "on the last line, put the next entry of the history, or the draft, in the editor",
        true,
        |_| Action::HistoryNext,
    ),
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
    on_world("focus", "give the focus to a world", true, |world| {
        Action::Focus(world.unwrap_or_default())
    }),
    on_world(
        "connect",
        "connect a world, the world in focus by default",
        false,
        Action::Connect,
    ),
    on_world(
        "disconnect",
        "disconnect a world, the world in focus by default",
        false,
        Action::Disconnect,
    ),
    plain("help", "show one line for each entry", false, |_| {
        Action::Help
    }),
    plain(
        "quit",
        "leave the client; a second quit drops the lines not sent yet",
        false,
        |_| Action::Quit,
    ),
];

/// The values that an argument of `kind` can take now. For [`ArgKind::World`] they are the
/// worlds in the order of the top row.
#[must_use]
pub fn candidates(model: &Model, kind: ArgKind) -> Vec<String> {
    match kind {
        ArgKind::World => model.world_ids().into_iter().map(str::to_owned).collect(),
    }
}
