//! The parse of a client command: a text of one line that starts with `/` and not with `//`.

/// Where the cursor is in a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorAt {
    /// In the name, or on the `/` before it.
    Name,
    /// In the argument, or in the spaces between the name and the argument.
    Arg,
    /// Past the end of the argument.
    Past,
}

/// A command, split into its name and its argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parse<'a> {
    /// The text between the `/` and the first space.
    pub name: &'a str,
    /// The text after the first run of spaces, with no space at either end, or `None` if that
    /// text is empty.
    pub arg: Option<&'a str>,
    /// Where the cursor is.
    pub cursor: CursorAt,
}

/// The command in `text` with the cursor at the byte `cursor`, or `None` if `text` is no
/// command: it has a line break, does not start with `/`, or starts with `//`.
#[must_use]
pub fn parse(text: &str, cursor: usize) -> Option<Parse<'_>> {
    let body = text.strip_prefix('/')?;
    if body.starts_with('/') || text.contains('\n') {
        return None;
    }
    let (name, rest) = body.split_once(' ').unwrap_or((body, ""));
    let name_end = 1 + name.len();
    let arg_start = text.len() - rest.trim_start_matches(' ').len();
    let trimmed = rest.trim_matches(' ');
    let arg_end = arg_start + trimmed.len();
    let cursor = if cursor <= name_end {
        CursorAt::Name
    } else if cursor <= arg_end || trimmed.is_empty() && cursor <= arg_start {
        CursorAt::Arg
    } else {
        CursorAt::Past
    };
    Some(Parse {
        name,
        arg: (!trimmed.is_empty()).then_some(trimmed),
        cursor,
    })
}
