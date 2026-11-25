//! The layouts that mu ships.

use super::Layout;

/// A layout that mu ships: its name and its lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltIn {
    /// The name.
    pub name: &'static str,
    /// The lines of the plan.
    pub lines: &'static [&'static str],
}

/// The name of the layout of a client whose configuration names none.
pub const DEFAULT_LAYOUT: &str = "classic";

/// Every built-in layout, in the order of the names.
pub const BUILT_IN: &[BuiltIn] = &[
    BuiltIn {
        name: "classic",
        lines: &["output", "tabs above output", "editor below output"],
    },
    BuiltIn {
        name: "focus",
        lines: &["output", "editor below output"],
    },
    BuiltIn {
        name: "sidebar",
        lines: &[
            "output",
            "tabs above output",
            "editor below output",
            "statusline below editor",
            "output as notices right of output, 30%",
        ],
    },
    BuiltIn {
        name: "status",
        lines: &[
            "output",
            "tabs above output",
            "editor below output",
            "statusline below editor",
        ],
    },
];

impl BuiltIn {
    /// The built-in layout named `name`.
    #[must_use]
    pub fn named(name: &str) -> Option<&'static Self> {
        BUILT_IN.iter().find(|each| each.name == name)
    }

    /// The checked layout.
    ///
    /// # Errors
    ///
    /// A [`super::LayoutError`] if the lines are bad, which the test of this table rules out.
    pub fn layout(&self) -> Result<Layout, super::LayoutError> {
        Layout::parse(self.lines)
    }
}
