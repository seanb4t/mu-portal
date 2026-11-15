//! The width classes of a grapheme cluster: the groups where terminals disagree.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Class {
    Ascii,
    Narrow,
    Wide,
    /// East Asian Ambiguous: one cell in most locales, two in a CJK locale.
    Ambiguous,
    /// A base character with combining marks.
    Combining,
    /// A text-default character with VS16 (U+FE0F), the emoji presentation selector.
    Vs16,
    /// An emoji-default character with VS15 (U+FE0E), the text presentation selector.
    Vs15,
    /// An emoji sequence joined with ZWJ (U+200D).
    Zwj,
    /// A pair of regional indicators.
    Flag,
    /// An emoji with a skin tone modifier.
    SkinTone,
    /// A digit, `#` or `*` with VS16 and U+20E3.
    Keycap,
    /// Symbols and Pictographs Extended-A (U+1FA70..U+1FAFF), the block of most new emoji.
    EmojiExtA,
    /// A grapheme of zero width on its own, such as U+200B.
    Zero,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Ascii => "ascii",
            Class::Narrow => "narrow",
            Class::Wide => "wide",
            Class::Ambiguous => "ambiguous",
            Class::Combining => "combining",
            Class::Vs16 => "vs16",
            Class::Vs15 => "vs15",
            Class::Zwj => "zwj",
            Class::Flag => "flag",
            Class::SkinTone => "skin-tone",
            Class::Keycap => "keycap",
            Class::EmojiExtA => "emoji-ext-a",
            Class::Zero => "zero",
        }
    }

    /// A class where the spike expects terminals to disagree with `unicode-width`.
    pub fn risky(self) -> bool {
        !matches!(
            self,
            Class::Ascii | Class::Narrow | Class::Wide | Class::Ambiguous | Class::Combining
        )
    }
}

fn regional(c: char) -> bool {
    ('\u{1F1E6}'..='\u{1F1FF}').contains(&c)
}

fn skin_tone(c: char) -> bool {
    ('\u{1F3FB}'..='\u{1F3FF}').contains(&c)
}

pub fn classify(g: &str) -> Class {
    if g.is_ascii() {
        return Class::Ascii;
    }
    let chars: Vec<char> = g.chars().collect();
    if chars.len() > 1 && chars.contains(&'\u{200D}') {
        return Class::Zwj;
    }
    if chars.len() == 2 && chars.iter().all(|&c| regional(c)) {
        return Class::Flag;
    }
    if chars.contains(&'\u{20E3}') {
        return Class::Keycap;
    }
    if chars.contains(&'\u{FE0F}') {
        return Class::Vs16;
    }
    if chars.contains(&'\u{FE0E}') {
        return Class::Vs15;
    }
    if chars.iter().any(|&c| skin_tone(c)) {
        return Class::SkinTone;
    }
    if ('\u{1FA70}'..='\u{1FAFF}').contains(&chars[0]) {
        return Class::EmojiExtA;
    }
    if g.width() == 0 {
        return Class::Zero;
    }
    if chars.len() > 1 {
        return Class::Combining;
    }
    let c = chars[0];
    if c.width() != c.width_cjk() {
        Class::Ambiguous
    } else if c.width() == Some(2) {
        Class::Wide
    } else {
        Class::Narrow
    }
}

/// A zero-width format character: ZWSP, the joiners, the word joiner, the BOM.
fn format_char(c: char) -> bool {
    matches!(c, '\u{200B}'..='\u{200F}' | '\u{2060}'..='\u{2064}' | '\u{FEFF}')
}

/// The width a terminal gets when it adds the width of each code point.
/// `zw` is the width it gives a zero-width format character such as ZWJ.
pub fn char_sum(g: &str, zw: u16) -> u16 {
    g.chars()
        .map(|c| {
            if format_char(c) {
                zw
            } else {
                c.width().unwrap_or(0) as u16
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes() {
        let cases = [
            ("a", Class::Ascii),
            ("é", Class::Narrow),
            ("漢", Class::Wide),
            ("😀", Class::Wide),
            ("①", Class::Ambiguous),
            ("e\u{301}", Class::Combining),
            ("❤\u{FE0F}", Class::Vs16),
            ("⌚\u{FE0E}", Class::Vs15),
            ("👨\u{200D}👩\u{200D}👧", Class::Zwj),
            ("🏳\u{FE0F}\u{200D}🌈", Class::Zwj),
            ("🇺🇸", Class::Flag),
            ("👍🏽", Class::SkinTone),
            ("1\u{FE0F}\u{20E3}", Class::Keycap),
            ("🫩", Class::EmojiExtA),
            ("\u{200B}", Class::Zero),
        ];
        for (g, want) in cases {
            assert_eq!(classify(g), want, "{g:?}");
        }
    }

    #[test]
    fn sum_counts_each_code_point() {
        assert_eq!(char_sum("👨\u{200D}👩\u{200D}👧", 0), 6);
        assert_eq!(char_sum("👨\u{200D}👩\u{200D}👧", 1), 8);
        assert_eq!(char_sum("❤\u{FE0F}", 1), 1);
    }
}
