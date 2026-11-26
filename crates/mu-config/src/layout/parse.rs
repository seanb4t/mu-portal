//! The reader of one line. It knows the words of a sentence and no layout.

use std::fmt;

use winnow::ascii::{space0, space1};
use winnow::combinator::{alt, cut_err, eof, opt, peek, preceded};
use winnow::error::ContextError;
use winnow::token::take_while;
use winnow::{ModalResult, Parser};

use super::block::BlockKind;
use super::change::Change;
use super::step::{BlockId, EDITOR_MAX, Edge, PERCENT_MAX, PERCENT_MIN, Place, Side, Size, Step};
use crate::stream::{Pattern, Scope, Selection};

/// What is wrong with one line, before the line has a number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineError {
    /// The word that the reader stopped at.
    pub word: String,
    /// The column of that word, from 1.
    pub column: usize,
    /// What is wrong.
    pub reason: String,
}

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "column {}, `{}`: {}",
            self.column,
            shown(&self.word),
            self.reason
        )
    }
}

/// The word `word` of a line for the text of an error: its first 64 characters, with U+FFFD in
/// place of each control character. A word comes from a request or a file, so it can hold any
/// character.
pub(crate) fn shown(word: &str) -> String {
    word.chars()
        .take(64)
        .map(|c| if c.is_control() { '\u{fffd}' } else { c })
        .collect()
}

impl std::error::Error for LineError {}

/// The reason of a word that parses and is still wrong.
#[derive(Debug)]
struct Reason(&'static str);

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for Reason {}

const UNKNOWN_FORM: &str = "unknown form";
const BAD_SIZE: &str = "bad size: a size is a percent from 10% to 90%, a count of cells from 1 to 65535, or `max` and 1 to 100";

fn is_word_char(c: char) -> bool {
    !c.is_whitespace() && c != ','
}

fn word<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    take_while(1.., is_word_char).parse_next(input)
}

fn keyword<'a>(
    expected: &'static str,
) -> impl Parser<&'a str, &'a str, winnow::error::ErrMode<ContextError>> {
    word.verify(move |found: &str| found == expected)
}

fn block(input: &mut &str) -> ModalResult<BlockKind> {
    word.try_map(|name| BlockKind::named(name).ok_or(Reason("unknown block")))
        .parse_next(input)
}

fn id(input: &mut &str) -> ModalResult<BlockId> {
    word.try_map(|text| BlockId::new(text).map_err(Reason))
        .parse_next(input)
}

fn side(input: &mut &str) -> ModalResult<Side> {
    alt((
        keyword("above").value(Side::Above),
        keyword("below").value(Side::Below),
        (keyword("left"), space1, cut_err(keyword("of"))).value(Side::Left),
        (keyword("right"), space1, cut_err(keyword("of"))).value(Side::Right),
    ))
    .parse_next(input)
}

fn anchors(input: &mut &str) -> ModalResult<Vec<BlockId>> {
    let mut all = vec![id.parse_next(input)?];
    while opt((space1, keyword("and"), space1))
        .parse_next(input)?
        .is_some()
    {
        all.push(cut_err(id).parse_next(input)?);
    }
    Ok(all)
}

fn edge(input: &mut &str) -> ModalResult<Edge> {
    word.try_map(|name| {
        Edge::ALL
            .into_iter()
            .find(|edge| edge.word() == name)
            .ok_or(Reason(UNKNOWN_FORM))
    })
    .parse_next(input)
}

fn place(input: &mut &str) -> ModalResult<Place> {
    alt((
        (side, space1, cut_err(anchors)).map(|(side, _, anchors)| Place::Beside { side, anchors }),
        preceded((keyword("at"), space1), cut_err(edge)).map(Place::At),
        preceded((keyword("with"), space1), cut_err(id)).map(Place::With),
    ))
    .parse_next(input)
}

fn count(text: &str) -> Option<u16> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn max_rows(text: &str) -> Result<Size, Reason> {
    count(text)
        .filter(|rows| (1..=EDITOR_MAX).contains(rows))
        .map(Size::Max)
        .ok_or(Reason(BAD_SIZE))
}

fn part_size(text: &str) -> Result<Size, Reason> {
    let size = match text.strip_suffix('%') {
        Some(percent) => count(percent)
            .and_then(|percent| u8::try_from(percent).ok())
            .filter(|percent| (PERCENT_MIN..=PERCENT_MAX).contains(percent))
            .map(Size::Percent),
        None => count(text).filter(|cells| *cells >= 1).map(Size::Cells),
    };
    size.ok_or(Reason(BAD_SIZE))
}

fn size(input: &mut &str) -> ModalResult<Size> {
    alt((
        preceded((keyword("max"), space1), cut_err(word.try_map(max_rows))),
        word.try_map(part_size),
    ))
    .parse_next(input)
}

/// A line as the reader takes it, before the checks of its scope and its patterns.
struct Draft<'a> {
    kind: BlockKind,
    id: BlockId,
    /// The word `of` of `of all worlds`.
    all_worlds: Option<&'a str>,
    /// The word `showing`, and each pattern after it.
    showing: Option<(&'a str, Vec<&'a str>)>,
    place: Place,
    size: Option<Size>,
}

fn all_worlds<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    let (of, ..) = (
        keyword("of"),
        space1,
        cut_err((keyword("all"), space1, keyword("worlds"))),
    )
        .parse_next(input)?;
    Ok(of)
}

fn at_end(input: &mut &str) -> ModalResult<()> {
    (space0, eof).void().parse_next(input)
}

fn place_next(input: &mut &str) -> ModalResult<()> {
    (space1, place).void().parse_next(input)
}

/// A comma and a size that ends the line.
/// The comma and the size at the end of a line.
fn size_tail(input: &mut &str) -> ModalResult<Size> {
    preceded(comma, cut_err(size)).parse_next(input)
}

fn size_next(input: &mut &str) -> ModalResult<()> {
    (size_tail, space0, eof).void().parse_next(input)
}

fn comma(input: &mut &str) -> ModalResult<()> {
    (space0, ',', space0).void().parse_next(input)
}

/// The word `showing` and the patterns after it. The list ends at a place, at a comma and a size
/// that end the line, or at the end of the line.
fn showing<'a>(input: &mut &'a str) -> ModalResult<(&'a str, Vec<&'a str>)> {
    let at = keyword("showing").parse_next(input)?;
    let mut patterns = Vec::new();
    loop {
        if peek(at_end).parse_next(input).is_ok() || peek(place_next).parse_next(input).is_ok() {
            break;
        }
        if !patterns.is_empty() && peek(size_next).parse_next(input).is_ok() {
            break;
        }
        let parted = if patterns.is_empty() {
            space1.void().parse_next(input)
        } else {
            comma.parse_next(input)
        };
        if parted.is_err() {
            break;
        }
        patterns.push(word.parse_next(input)?);
    }
    Ok((at, patterns))
}

fn line<'a>(input: &mut &'a str) -> ModalResult<Draft<'a>> {
    space0.parse_next(input)?;
    let kind = block.parse_next(input)?;
    let id = opt(preceded((space1, keyword("as"), space1), cut_err(id))).parse_next(input)?;
    let (all_worlds, showing) = if id.is_some() {
        (
            opt(preceded(space1, all_worlds)).parse_next(input)?,
            opt(preceded(space1, showing)).parse_next(input)?,
        )
    } else {
        (None, None)
    };
    let place = opt(preceded(space1, place)).parse_next(input)?;
    let size = opt(size_tail).parse_next(input)?;
    (space0, eof).parse_next(input)?;
    Ok(Draft {
        kind,
        id: id.unwrap_or_else(|| BlockId::of(kind)),
        all_worlds,
        showing,
        place: place.unwrap_or(Place::First),
        size,
    })
}

/// The column of `part`, a slice of `text`, from 1.
fn column_of(text: &str, part: &str) -> usize {
    let offset = (part.as_ptr() as usize).saturating_sub(text.as_ptr() as usize);
    text.get(..offset)
        .map_or(1, |before| before.chars().count() + 1)
}

impl Draft<'_> {
    /// The step, with the checks of the scope and the patterns.
    fn finish(self, text: &str) -> Result<Step, LineError> {
        let bad = |part: &str, reason: &str| LineError {
            word: part.to_owned(),
            column: column_of(text, part),
            reason: reason.to_owned(),
        };
        if self.kind != BlockKind::Output {
            if let Some(of) = self.all_worlds {
                return Err(bad(of, "only an `output` takes `of all worlds`"));
            }
            if let Some((at, _)) = self.showing {
                return Err(bad(at, "only an `output` takes `showing`"));
            }
            return Ok(Step::new(self.kind, self.id, self.place, self.size));
        }
        let scope = if self.all_worlds.is_some() {
            Scope::AllWorlds
        } else {
            Scope::World
        };
        let selection = match self.showing {
            None => Selection::default_for(&self.id, scope),
            Some((at, texts)) => {
                let patterns = texts
                    .iter()
                    .map(|each| Pattern::new(each).map_err(|reason| bad(each, reason)))
                    .collect::<Result<Vec<_>, _>>()?;
                Selection::new(scope, patterns).map_err(|reason| bad(at, reason))?
            }
        };
        Ok(Step {
            kind: self.kind,
            id: self.id,
            place: self.place,
            size: self.size,
            selection: Some(selection),
        })
    }
}

/// The most characters of one line of a plan.
pub const LINE_MAX: usize = 256;

impl Size {
    /// Reads a size, such as `30%`, `12` or `max 3`.
    ///
    /// # Errors
    ///
    /// A [`LineError`] that names the whole text.
    pub fn parse(text: &str) -> Result<Self, LineError> {
        let bad = || LineError {
            word: text.trim().to_owned(),
            column: 1,
            reason: BAD_SIZE.to_owned(),
        };
        (space0, size, space0, eof)
            .map(|(_, size, _, _)| size)
            .parse(text)
            .map_err(|_| bad())
    }
}

impl Step {
    /// Reads one line.
    ///
    /// # Errors
    ///
    /// A [`LineError`] that names the word where the reader stopped, and its column.
    pub fn parse(text: &str) -> Result<Self, LineError> {
        read(text).map(|(step, _)| step)
    }
}

impl Change {
    /// The change of the command `/place <text>`: [`Change::Move`] for an `output` with no
    /// `of all worlds` and no `showing`, and [`Change::Place`] for each other line.
    ///
    /// # Errors
    ///
    /// A [`LineError`] as [`Step::parse`] gives it.
    pub fn place(text: &str) -> Result<Self, LineError> {
        read(text).map(|(step, named)| {
            if step.kind == BlockKind::Output && !named {
                Self::Move(step)
            } else {
                Self::Place(step)
            }
        })
    }
}

/// The step of one line, and whether the line names the streams or the scope of an `output`.
fn read(text: &str) -> Result<(Step, bool), LineError> {
    {
        if text.trim().is_empty() {
            return Err(LineError {
                word: String::new(),
                column: 1,
                reason: "an empty line".to_owned(),
            });
        }
        if text.chars().count() > LINE_MAX {
            return Err(LineError {
                word: text.chars().take(16).collect(),
                column: 1,
                reason: format!("a line has at most {LINE_MAX} characters"),
            });
        }
        let draft = line.parse(text).map_err(|error| {
            let offset = error.offset();
            let rest = &text[offset..];
            let found: String = rest.chars().take_while(|c| is_word_char(*c)).collect();
            let word = if found.is_empty() {
                match rest.chars().next() {
                    Some(c) if !c.is_whitespace() => c.to_string(),
                    _ => text
                        .split(|c: char| !is_word_char(c))
                        .rfind(|part| !part.is_empty())
                        .unwrap_or_default()
                        .to_owned(),
                }
            } else {
                found
            };
            let reason = error
                .inner()
                .cause()
                .map_or_else(|| UNKNOWN_FORM.to_owned(), ToString::to_string);
            LineError {
                word,
                column: text[..offset].chars().count() + 1,
                reason,
            }
        })?;
        let named = draft.showing.is_some() || draft.all_worlds.is_some();
        draft.finish(text).map(|step| (step, named))
    }
}
