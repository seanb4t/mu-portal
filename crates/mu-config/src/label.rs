//! The label of a world tab, and the segments of the statusline.

use std::fmt;

use crate::names::listed;
use crate::world_id::WorldId;

/// The longest label, in characters.
pub const LABEL_MAX: usize = 64;
/// The label of a tab whose configuration gives none.
pub const LABEL_DEFAULT: &str = "{world}";

/// A bad value of a field. Its text is the reason alone, so the reader of the file puts the
/// file and the field before it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldError {
    /// What is wrong with the value.
    pub reason: String,
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.reason)
    }
}

impl std::error::Error for FieldError {}

fn bad<T>(reason: impl Into<String>) -> Result<T, FieldError> {
    Err(FieldError {
        reason: reason.into(),
    })
}

/// One field of a label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelField {
    /// The world id.
    World,
    /// The part of the world id before the first `/`.
    Game,
    /// The part of the world id after the first `/`.
    Character,
    /// The place of the world in the tabs, from 1.
    Number,
}

impl LabelField {
    /// Every field.
    pub const ALL: [Self; 4] = [Self::World, Self::Game, Self::Character, Self::Number];

    /// The name of the field between the braces.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::World => "world",
            Self::Game => "game",
            Self::Character => "character",
            Self::Number => "number",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Piece {
    Text(String),
    Field(LabelField),
}

/// A checked label: text with fields in braces. `{{` gives `{`, and `}}` gives `}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label(Vec<Piece>);

impl Label {
    /// Checks `text`.
    ///
    /// # Errors
    ///
    /// A [`FieldError`] for an empty label, a label of more than [`LABEL_MAX`] characters, an
    /// unknown field, a `{` with no closing `}`, and a lone `}`.
    pub fn parse(text: &str) -> Result<Self, FieldError> {
        let count = text.chars().count();
        if !(1..=LABEL_MAX).contains(&count) {
            return bad(format!(
                "a label has 1 to {LABEL_MAX} characters, and this one has {count}"
            ));
        }
        let mut pieces = Vec::new();
        let mut plain = String::new();
        let mut rest = text;
        while let Some(at) = rest.find(['{', '}']) {
            plain.push_str(&rest[..at]);
            let tail = &rest[at..];
            if let Some(after) = tail.strip_prefix("{{") {
                plain.push('{');
                rest = after;
            } else if let Some(after) = tail.strip_prefix("}}") {
                plain.push('}');
                rest = after;
            } else if tail.starts_with('}') {
                return bad("a lone `}`: write `}}` for the character");
            } else {
                let Some(end) = tail.find('}') else {
                    return bad("a `{` with no closing `}`: write `{{` for the character");
                };
                let name = &tail[1..end];
                let Some(field) = LabelField::ALL.into_iter().find(|each| each.name() == name)
                else {
                    let names: Vec<&str> = LabelField::ALL.iter().map(|each| each.name()).collect();
                    return bad(format!(
                        "no field is named {name}: the fields are {}",
                        listed(&names)
                    ));
                };
                if !plain.is_empty() {
                    pieces.push(Piece::Text(std::mem::take(&mut plain)));
                }
                pieces.push(Piece::Field(field));
                rest = &tail[end + 1..];
            }
        }
        plain.push_str(rest);
        if !plain.is_empty() {
            pieces.push(Piece::Text(plain));
        }
        Ok(Self(pieces))
    }

    /// The label of the world `world`, which is tab `number` from 1.
    #[must_use]
    pub fn render(&self, world: &WorldId, number: usize) -> String {
        let (game, character) = (world.game(), world.character());
        let world = world.as_str();
        let mut text = String::new();
        for piece in &self.0 {
            match piece {
                Piece::Text(plain) => text.push_str(plain),
                Piece::Field(LabelField::World) => text.push_str(world),
                Piece::Field(LabelField::Game) => text.push_str(game),
                Piece::Field(LabelField::Character) => text.push_str(character),
                Piece::Field(LabelField::Number) => text.push_str(&number.to_string()),
            }
        }
        text
    }
}

impl Default for Label {
    fn default() -> Self {
        Self(vec![Piece::Field(LabelField::World)])
    }
}

/// One segment of the statusline, about the world in focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Segment {
    /// The world id.
    World,
    /// Whether the world is connected.
    State,
    /// The time since the world connected.
    Connected,
    /// The time since the last send to the world.
    Idle,
    /// The count of characters in the editor with the input focus.
    Typed,
}

impl Segment {
    /// Every segment.
    pub const ALL: [Self; 5] = [
        Self::World,
        Self::State,
        Self::Connected,
        Self::Idle,
        Self::Typed,
    ];
    /// The segments of the left side of a statusline whose configuration gives none.
    pub const LEFT_DEFAULT: [Self; 2] = [Self::World, Self::State];
    /// The segments of the right side of a statusline whose configuration gives none.
    pub const RIGHT_DEFAULT: [Self; 1] = [Self::Idle];

    /// The name of the segment in a list.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::World => "world",
            Self::State => "state",
            Self::Connected => "connected",
            Self::Idle => "idle",
            Self::Typed => "typed",
        }
    }

    /// Whether the segment shows a time, so that it changes with the clock.
    #[must_use]
    pub const fn is_time(self) -> bool {
        matches!(self, Self::Connected | Self::Idle)
    }

    /// Checks one list of segment names.
    ///
    /// # Errors
    ///
    /// A [`FieldError`] for an unknown segment, and for a segment twice in the list.
    pub fn check<S: AsRef<str>>(names: &[S]) -> Result<Vec<Self>, FieldError> {
        let mut segments = Vec::new();
        for name in names {
            let name = name.as_ref();
            let Some(segment) = Self::ALL.into_iter().find(|each| each.name() == name) else {
                let all: Vec<&str> = Self::ALL.iter().map(|each| each.name()).collect();
                return bad(format!(
                    "no segment is named {name}: the segments are {}",
                    listed(&all)
                ));
            };
            if segments.contains(&segment) {
                return bad(format!("the segment {name} is in the list twice"));
            }
            segments.push(segment);
        }
        Ok(segments)
    }
}
