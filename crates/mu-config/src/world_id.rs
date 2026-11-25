//! The id of a world: one character on one game.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use crate::layout::BuiltIn;
use crate::names::{DeviceName, LayoutName, NameError};

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BadWorldId {
    text: String,
}

impl fmt::Display for BadWorldId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} is not a world id of the form <game>/<character>: each part {PART_RULE}",
            self.text
        )
    }
}

impl std::error::Error for BadWorldId {}

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
pub const PART_RULE: &str = "holds only a-z, 0-9 and '-', and starts with a letter";

/// Whether `part` is one part of a world id. [`PART_RULE`] states the rule.
#[must_use]
pub fn is_part(part: &str) -> bool {
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

/// The key of one entry of `[world-layouts]`: a world, or every world of a game.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WorldKey {
    /// One world.
    World(WorldId),
    /// Each world of the game of this name.
    Game(GameName),
}

/// The name of a game: the first part of a world id.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameName(String);

impl GameName {
    /// Checks `text`, which is a name if it follows [`PART_RULE`].
    #[must_use]
    pub fn new(text: &str) -> Option<Self> {
        is_part(text).then(|| Self(text.to_owned()))
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The most characters of a world key: two parts of 64 characters and the `/`.
pub const WORLD_KEY_MAX: usize = 129;

impl WorldKey {
    /// The key as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::World(id) => id.as_str(),
            Self::Game(game) => game.as_str(),
        }
    }
}

/// The error for a text that is not a world key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BadWorldKey {
    text: String,
}

impl fmt::Display for BadWorldKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} is not a world id of the form <game>/<character>, and not the name of a game: each part {PART_RULE}, and a key has at most {WORLD_KEY_MAX} characters",
            self.text
        )
    }
}

impl std::error::Error for BadWorldKey {}

impl FromStr for WorldKey {
    type Err = BadWorldKey;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let bad = || BadWorldKey {
            text: text.to_owned(),
        };
        if text.chars().count() > WORLD_KEY_MAX {
            return Err(bad());
        }
        if let Some(game) = GameName::new(text) {
            return Ok(Self::Game(game));
        }
        text.parse().map(Self::World).map_err(|_| bad())
    }
}

impl fmt::Display for WorldKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What gives the layout of a world: an entry of `[world-layouts]`, or the field `layout`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Choice {
    /// The entry of this key.
    Entry(WorldKey),
    /// The field `layout`.
    Default,
}

/// The choice of the layout of `world`: the entry of its world id, then the entry of its game,
/// then the field `layout`. No world gives the field `layout`.
#[must_use]
pub fn layout_for<V>(world: Option<&WorldId>, entries: &BTreeMap<String, V>) -> Choice {
    let Some(world) = world else {
        return Choice::Default;
    };
    if entries.contains_key(world.as_str()) {
        Choice::Entry(WorldKey::World(world.clone()))
    } else if entries.contains_key(world.game()) {
        Choice::Entry(WorldKey::Game(GameName(world.game().to_owned())))
    } else {
        Choice::Default
    }
}

/// Where a save of a layout goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveTarget {
    /// The layout is a layout of the player, and the save replaces it. No choice changes.
    Same(LayoutName),
    /// The layout is built in, so the save writes a new layout with this name, and the choice
    /// in use then names it.
    New(LayoutName),
}

impl SaveTarget {
    /// The name of the layout that the save writes.
    #[must_use]
    pub const fn name(&self) -> &LayoutName {
        match self {
            Self::Same(name) | Self::New(name) => name,
        }
    }
}

/// Where a save of the layout `layout` goes, for the device named `device` and the choice in
/// use. A layout of the player keeps its name, and needs no device. A built-in layout with no
/// device is an error. A built-in layout gives a new layout: with the device
/// name for the field `layout`, and with the device name, `.` and the key with `.` in place of
/// `/` for an entry.
///
/// # Errors
///
/// A [`NameError`] for a name that is no good name of a layout file: a name of more than 64
/// characters, the name of a built-in layout, `.` or `..`.
pub fn save_target(
    choice: &Choice,
    layout: &str,
    device: Option<&str>,
) -> Result<SaveTarget, NameError> {
    if BuiltIn::named(layout).is_none() {
        return LayoutName::new(layout).map(SaveTarget::Same);
    }
    // A new layout takes its name from the device, so no device is the error of no name.
    let device = DeviceName::new(device.unwrap_or_default())?;
    let name = match choice {
        Choice::Default => device.as_str().to_owned(),
        Choice::Entry(key) => format!("{}.{}", device, key.as_str().replace('/', ".")),
    };
    LayoutName::new(&name).map(SaveTarget::New)
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
