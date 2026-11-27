//! The names of the streams of a line, and the lists of them that an event carries.

use std::fmt;

use crate::layout::{BlockId, WORDS, is_id_byte};

/// The stream of each line that no rule sends elsewhere. It is also the id of an `output` whose
/// line has no `as`.
pub const MAIN: &str = "main";

/// The stream of the notices of the client.
pub const NOTICES: &str = "notices";

/// The most names of one list of streams.
pub const STREAMS_MAX: usize = 8;

/// The name of a stream. It follows the rule of a block id, so it is always lower case.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StreamName(BlockId);

impl StreamName {
    /// Checks `name`, after it turns each upper-case letter to lower case.
    ///
    /// # Errors
    ///
    /// The reason that `name` is no stream name.
    pub fn new(name: &str) -> Result<Self, &'static str> {
        BlockId::new(name).map(Self)
    }

    /// The stream [`MAIN`].
    #[must_use]
    pub fn main() -> Self {
        Self(BlockId::of(crate::layout::BlockKind::Output))
    }

    /// The stream [`NOTICES`].
    #[must_use]
    pub fn notices() -> Self {
        Self(BlockId::notices())
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<BlockId> for StreamName {
    fn from(id: BlockId) -> Self {
        Self(id)
    }
}

impl AsRef<str> for StreamName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for StreamName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The streams of one line: 1 to [`STREAMS_MAX`] names, no name twice, in the order given.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Streams(Vec<StreamName>);

impl Streams {
    /// The list of `names`, with each name after its first time dropped.
    ///
    /// # Errors
    ///
    /// The reason when the list holds no name, or more than [`STREAMS_MAX`] names.
    pub fn new(names: impl IntoIterator<Item = StreamName>) -> Result<Self, String> {
        let mut kept: Vec<StreamName> = Vec::new();
        for name in names {
            if !kept.contains(&name) {
                kept.push(name);
            }
        }
        if kept.is_empty() {
            return Err("a list of streams holds at least 1 name".to_owned());
        }
        if kept.len() > STREAMS_MAX {
            return Err(format!(
                "a list of streams holds at most {STREAMS_MAX} names"
            ));
        }
        Ok(Self(kept))
    }

    /// The list of the one stream [`MAIN`].
    #[must_use]
    pub fn main() -> Self {
        Self(vec![StreamName::main()])
    }

    /// The streams of an event as the wire gives them. A list that is empty, too long, or holds a
    /// bad name gives [`Streams::main`].
    #[must_use]
    pub fn from_wire<S: AsRef<str>>(names: &[S]) -> Self {
        names
            .iter()
            .map(|name| StreamName::new(name.as_ref()))
            .collect::<Result<Vec<_>, _>>()
            .ok()
            .and_then(|names| Self::new(names).ok())
            .unwrap_or_else(Self::main)
    }

    /// The names, in order.
    #[must_use]
    pub fn names(&self) -> &[StreamName] {
        &self.0
    }

    /// The names as text, in order, as the wire carries them.
    #[must_use]
    pub fn to_wire(&self) -> Vec<String> {
        self.0.iter().map(|name| name.as_str().to_owned()).collect()
    }
}

impl fmt::Display for Streams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, name) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str(",")?;
            }
            f.write_str(name.as_str())?;
        }
        Ok(())
    }
}

/// Which worlds an `output` shows its streams for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Scope {
    /// The world in focus.
    #[default]
    World,
    /// Every world, in one pane.
    AllWorlds,
}

/// One pattern after `showing`: a glob with `*` and `?`, which can exclude.
#[derive(Clone, Debug)]
pub struct Pattern {
    text: String,
    exclude: bool,
    glob: wildmatch::WildMatch,
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.exclude == other.exclude
    }
}

impl Eq for Pattern {}

impl Pattern {
    /// Reads a pattern, after it turns each upper-case letter to lower case.
    ///
    /// # Errors
    ///
    /// The reason that `text` is no pattern.
    pub fn new(text: &str) -> Result<Self, &'static str> {
        if text.starts_with("re:") || text.starts_with('/') {
            return Err("a pattern that starts with `re:` or `/` is kept for a later syntax");
        }
        let lower = text.to_ascii_lowercase();
        let (exclude, glob) = match lower.strip_prefix('!') {
            Some(rest) => (true, rest),
            None => (false, lower.as_str()),
        };
        if glob.is_empty() {
            return Err("a pattern holds a glob after `!`");
        }
        if !glob
            .bytes()
            .all(|byte| is_id_byte(byte) || byte == b'*' || byte == b'?')
        {
            return Err("a pattern holds only a to z, 0 to 9, -, * and ?, after one first !");
        }
        if glob.starts_with(|c: char| c.is_ascii_digit()) || WORDS.contains(&glob) {
            return Err(
                "a pattern starts with no digit and is no word of the sentences, as a stream name",
            );
        }
        Ok(Self {
            glob: wildmatch::WildMatch::new(glob),
            text: glob.to_owned(),
            exclude,
        })
    }

    /// The pattern of exactly the stream `name`.
    #[must_use]
    pub fn exactly(name: &str) -> Self {
        Self {
            glob: wildmatch::WildMatch::new(name),
            text: name.to_owned(),
            exclude: false,
        }
    }

    /// Whether the pattern excludes.
    #[must_use]
    pub fn excludes(&self) -> bool {
        self.exclude
    }

    /// Whether the glob matches `name`.
    #[must_use]
    pub fn matches(&self, name: &StreamName) -> bool {
        self.glob.matches(name.as_str())
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.exclude {
            f.write_str("!")?;
        }
        f.write_str(&self.text)
    }
}

/// The streams that an `output` shows, and for which worlds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    /// The worlds.
    pub scope: Scope,
    patterns: Vec<Pattern>,
}

impl Selection {
    /// The selection of `patterns`.
    ///
    /// # Errors
    ///
    /// The reason when the list is empty or holds only exclusions.
    pub fn new(scope: Scope, patterns: Vec<Pattern>) -> Result<Self, &'static str> {
        if patterns.is_empty() {
            return Err("`showing` takes at least 1 pattern");
        }
        if patterns.iter().all(Pattern::excludes) {
            return Err("`showing` takes at least 1 pattern without `!`");
        }
        Ok(Self { scope, patterns })
    }

    /// The selection of an `output` with no `showing`: its id, or [`MAIN`] and [`NOTICES`] for
    /// the id [`MAIN`].
    #[must_use]
    pub fn default_for(id: &BlockId, scope: Scope) -> Self {
        let patterns = if id.as_str() == MAIN {
            vec![Pattern::exactly(MAIN), Pattern::exactly(NOTICES)]
        } else {
            vec![Pattern::exactly(id.as_str())]
        };
        Self { scope, patterns }
    }

    /// The patterns, in the order of the line.
    #[must_use]
    pub fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }

    /// Whether the patterns are the default of the output `id`.
    #[must_use]
    pub fn is_default_for(&self, id: &BlockId) -> bool {
        self.patterns == Self::default_for(id, self.scope).patterns
    }

    /// Whether the output shows the stream `name`: one pattern without `!` matches it, and no
    /// pattern with `!` does.
    #[must_use]
    pub fn shows(&self, name: &StreamName) -> bool {
        let mut shown = false;
        for pattern in &self.patterns {
            if pattern.matches(name) {
                if pattern.excludes() {
                    return false;
                }
                shown = true;
            }
        }
        shown
    }
}
