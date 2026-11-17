//! The id of a world: one character on one game.

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// The id `<game>/<character>` of a world.
///
/// Each part holds only `a` to `z`, digits and hyphens, and starts with a letter.
/// Ids sort by their game part and then by their character part.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorldId {
    text: String,
    slash: usize,
}

/// The error for a text that is not a world id.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("{text:?} is not a world id of the form <game>/<character>: each part {PART_RULE}")]
pub struct BadWorldId {
    text: String,
}

impl WorldId {
    /// The game part, before the slash.
    #[must_use]
    pub fn game(&self) -> &str {
        &self.text[..self.slash]
    }

    /// The character part, after the slash.
    #[must_use]
    pub fn character(&self) -> &str {
        &self.text[self.slash + 1..]
    }

    /// The id as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

/// The rule of one part of a world id, as the end of a sentence.
pub(crate) const PART_RULE: &str = "holds only a-z, 0-9 and '-', and starts with a letter";

/// Whether `part` is one part of a world id. [`PART_RULE`] states the rule.
pub(crate) fn is_part(part: &str) -> bool {
    let mut chars = part.chars();
    chars.next().is_some_and(|first| first.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

impl FromStr for WorldId {
    type Err = BadWorldId;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let bad = || BadWorldId {
            text: text.to_owned(),
        };
        let (game, character) = text.split_once('/').ok_or_else(bad)?;
        if !is_part(game) || !is_part(character) {
            return Err(bad());
        }
        Ok(Self {
            text: text.to_owned(),
            slash: game.len(),
        })
    }
}

impl fmt::Display for WorldId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl Ord for WorldId {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.game(), self.character()).cmp(&(other.game(), other.character()))
    }
}

impl PartialOrd for WorldId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn ids_outside_the_rule() {
        for text in [
            "-alpha/aria",
            "alpha",
            "alpha/b/c",
            "alpha/",
            "1alpha/aria",
            "alpha/arïa",
            "Alpha/aria",
            "alpha/a_b",
        ] {
            assert!(text.parse::<WorldId>().is_err(), "{text:?} is refused");
        }
    }

    #[test]
    fn ids_inside_the_rule() {
        for text in ["alpha/aria", "alpha-2/bea", "a/b-9"] {
            let id: WorldId = text.parse().expect("the id parses");
            assert_eq!(id.to_string(), text);
        }
    }

    #[test]
    fn ids_sort_by_game_then_character() {
        let mut ids: Vec<WorldId> = ["alpha-2/cy", "alpha/bea", "alpha/aria"]
            .iter()
            .map(|text| text.parse().unwrap())
            .collect();
        ids.sort();
        let sorted: Vec<&str> = ids.iter().map(WorldId::as_str).collect();
        assert_eq!(sorted, ["alpha/aria", "alpha/bea", "alpha-2/cy"]);
    }

    const PART: &str = "[a-z][a-z0-9-]{0,12}";

    proptest! {
        #[test]
        fn world_id_round_trip(game in PART, character in PART) {
            let text = format!("{game}/{character}");
            let id: WorldId = text.parse().unwrap();
            prop_assert_eq!(id.to_string(), text);
            prop_assert_eq!(id.game(), game.as_str());
            prop_assert_eq!(id.character(), character.as_str());
        }

        #[test]
        fn world_id_refuses_a_foreign_char(
            game in PART,
            character in PART,
            foreign in any::<char>().prop_filter("outside the rule", |c| {
                !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
            }),
            at in any::<prop::sample::Index>(),
        ) {
            let mut text: Vec<char> = format!("{game}/{character}").chars().collect();
            let index = at.index(text.len() + 1);
            text.insert(index, foreign);
            let text: String = text.into_iter().collect();
            prop_assert!(text.parse::<WorldId>().is_err(), "{:?} is refused", text);
        }
    }
}
