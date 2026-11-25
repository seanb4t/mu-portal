//! The width of a grapheme on the terminal: one rule for each class of grapheme, learned by the
//! startup probe.
//!
//! For each class, the probe measures one grapheme and keeps the rule that gives that width:
//! the `unicode-width` value, the sum of the widths of the code points, or a constant. The zero
//! class sets the width of a zero-width format character, such as a joiner, inside a sum.

use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// A class of grapheme. Terminals can disagree on the width of a grapheme of a risky class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Class {
    /// An ASCII char.
    Ascii,
    /// A char one cell wide that is not ASCII.
    Narrow,
    /// A char two cells wide.
    Wide,
    /// An East Asian Ambiguous char: one cell in most locales, two in a CJK locale.
    Ambiguous,
    /// A base char with combining marks.
    Combining,
    /// A text-default char with VS16 (U+FE0F), the emoji presentation selector.
    Vs16,
    /// An emoji-default char with VS15 (U+FE0E), the text presentation selector.
    Vs15,
    /// An emoji sequence joined with ZWJ (U+200D).
    Zwj,
    /// A pair of regional indicators.
    Flag,
    /// An emoji with a skin tone modifier.
    SkinTone,
    /// A digit, `#` or `*` with VS16 and U+20E3.
    Keycap,
    /// A char of Symbols and Pictographs Extended-A (U+1FA70 to U+1FAFF).
    EmojiExtA,
    /// A grapheme of zero width on its own, such as U+200B.
    Zero,
}

impl Class {
    /// The class of the grapheme `g`.
    #[must_use]
    pub fn of(g: &str) -> Self {
        if g.is_ascii() {
            return Self::Ascii;
        }
        let chars: Vec<char> = g.chars().collect();
        let Some(&first) = chars.first() else {
            return Self::Zero;
        };
        if chars.len() > 1 && chars.contains(&'\u{200D}') {
            Self::Zwj
        } else if chars.len() == 2 && chars.iter().all(|&c| regional(c)) {
            Self::Flag
        } else if chars.contains(&'\u{20E3}') {
            Self::Keycap
        } else if chars.contains(&'\u{FE0F}') {
            Self::Vs16
        } else if chars.contains(&'\u{FE0E}') {
            Self::Vs15
        } else if chars.iter().any(|&c| skin_tone(c)) {
            Self::SkinTone
        } else if ('\u{1FA70}'..='\u{1FAFF}').contains(&first) {
            Self::EmojiExtA
        } else if g.width() == 0 {
            Self::Zero
        } else if chars.len() > 1 {
            Self::Combining
        } else if first.width() != first.width_cjk() {
            Self::Ambiguous
        } else if first.width() == Some(2) {
            Self::Wide
        } else {
            Self::Narrow
        }
    }

    /// True for a class where terminals can disagree with `unicode-width`.
    #[must_use]
    pub const fn risky(self) -> bool {
        !matches!(
            self,
            Self::Ascii | Self::Narrow | Self::Wide | Self::Ambiguous | Self::Combining
        )
    }
}

fn regional(c: char) -> bool {
    ('\u{1F1E6}'..='\u{1F1FF}').contains(&c)
}

fn skin_tone(c: char) -> bool {
    ('\u{1F3FB}'..='\u{1F3FF}').contains(&c)
}

/// A zero-width format char: ZWSP, the joiners, the word joiner, the BOM.
fn format_char(c: char) -> bool {
    matches!(c, '\u{200B}'..='\u{200F}' | '\u{2060}'..='\u{2064}' | '\u{FEFF}')
}

fn unicode(g: &str) -> u16 {
    u16::try_from(g.width()).unwrap_or(u16::MAX)
}

/// The width that a terminal gives `g` when it adds the width of each code point. `zero` is the
/// width that it gives a zero-width format char such as ZWJ.
fn char_sum(g: &str, zero: u16) -> u16 {
    g.chars()
        .map(|c| {
            if format_char(c) {
                zero
            } else {
                c.width().and_then(|w| u16::try_from(w).ok()).unwrap_or(0)
            }
        })
        .fold(0, u16::saturating_add)
}

/// How a class gets its width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// The `unicode-width` value.
    Unicode,
    /// The sum of the widths of the code points.
    CharSum,
    /// One width for each grapheme of the class.
    Const(u16),
}

/// The rule of each class, from the widths that the terminal gave the probe graphemes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Profile {
    rules: HashMap<Class, Rule>,
    /// The width that the terminal gives a zero-width format char inside a sum.
    zero: u16,
}

impl Profile {
    /// Learns the rule of each class from pairs of a probe grapheme and the width that the
    /// terminal gave it. A width of 0 keeps the `unicode-width` value of its class.
    #[must_use]
    pub fn learn(measured: &[(&str, u16)]) -> Self {
        let zero = measured
            .iter()
            .find(|(g, _)| Class::of(g) == Class::Zero)
            .map_or(0, |&(_, width)| width);
        let rules = measured
            .iter()
            .map(|&(g, width)| {
                let rule = if width == 0 || width == unicode(g) {
                    Rule::Unicode
                } else if width == char_sum(g, zero) {
                    Rule::CharSum
                } else {
                    Rule::Const(width)
                };
                (Class::of(g), rule)
            })
            .collect();
        Self { rules, zero }
    }

    /// The rule of `class`. A class that the probe did not measure has the `unicode-width` value.
    #[must_use]
    pub fn rule(&self, class: Class) -> Rule {
        self.rules.get(&class).copied().unwrap_or(Rule::Unicode)
    }

    /// The cells that the terminal gives the grapheme `g`.
    #[must_use]
    pub fn width(&self, g: &str) -> usize {
        let width = match self.rule(Class::of(g)) {
            Rule::Unicode => unicode(g),
            Rule::CharSum => char_sum(g, self.zero),
            Rule::Const(width) => width,
        };
        usize::from(width)
    }

    /// The cells that the terminal gives `text`.
    #[must_use]
    pub fn width_of(&self, text: &str) -> usize {
        text.graphemes(true).map(|each| self.width(each)).sum()
    }

    /// The part of `text` that fits in `width` cells, and the cells that it takes. It keeps
    /// whole graphemes: from the start of the text, or from its end if `from_end` is true.
    #[must_use]
    pub fn cut<'a>(&self, text: &'a str, width: usize, from_end: bool) -> (&'a str, usize) {
        let mut used = 0;
        let mut bytes = 0;
        let graphemes: Box<dyn Iterator<Item = &str>> = if from_end {
            Box::new(text.graphemes(true).rev())
        } else {
            Box::new(text.graphemes(true))
        };
        for grapheme in graphemes {
            let next = used + self.width(grapheme);
            if next > width {
                break;
            }
            used = next;
            bytes += grapheme.len();
        }
        let kept = if from_end {
            &text[text.len() - bytes..]
        } else {
            &text[..bytes]
        };
        (kept, used)
    }
}
