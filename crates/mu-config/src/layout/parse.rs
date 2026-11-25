//! The reader of one line. It knows the words of a sentence and no layout.

use std::fmt;

use winnow::ascii::{space0, space1};
use winnow::combinator::{alt, cut_err, eof, opt, preceded};
use winnow::error::ContextError;
use winnow::token::take_while;
use winnow::{ModalResult, Parser};

use super::block::BlockKind;
use super::step::{BlockId, EDITOR_MAX, Edge, PERCENT_MAX, PERCENT_MIN, Place, Side, Size, Step};

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
            self.column, self.word, self.reason
        )
    }
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

fn line(input: &mut &str) -> ModalResult<Step> {
    space0.parse_next(input)?;
    let kind = block.parse_next(input)?;
    let id = opt(preceded((space1, keyword("as"), space1), cut_err(id))).parse_next(input)?;
    let place = opt(preceded(space1, place)).parse_next(input)?;
    let size = opt(preceded((space0, ',', space0), cut_err(size))).parse_next(input)?;
    (space0, eof).parse_next(input)?;
    Ok(Step {
        kind,
        id: id.unwrap_or_else(|| BlockId::of(kind)),
        place: place.unwrap_or(Place::First),
        size,
    })
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
        line.parse(text).map_err(|error| {
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
        })
    }
}
