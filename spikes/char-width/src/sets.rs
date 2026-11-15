//! The graphemes of the spike.
//!
//! The probe set has one grapheme for each class. The startup probe measures only these.
//! The test set has other graphemes of the same classes, so a profile learned from the
//! probe set is checked on graphemes it never saw.

pub const PROBE: &[&str] = &[
    "a",
    "é",
    "漢",
    "①",
    "e\u{301}",
    "❤\u{FE0F}",
    "⌚\u{FE0E}",
    "👨\u{200D}👩\u{200D}👧",
    "🇺🇸",
    "👍🏽",
    "1\u{FE0F}\u{20E3}",
    "🫩",
    "\u{200B}",
];

pub const TEST: &[&str] = &[
    // narrow and wide
    "ñ",
    "ж",
    "字",
    "ア",
    "한",
    "😀",
    "🎉",
    // ambiguous
    "§",
    "±",
    "→",
    // combining
    "a\u{308}",
    "n\u{303}",
    "o\u{302}\u{323}",
    // vs16
    "☀\u{FE0F}",
    "✈\u{FE0F}",
    "☺\u{FE0F}",
    "⚠\u{FE0F}",
    // vs15
    "☕\u{FE0E}",
    "⚡\u{FE0E}",
    // zwj
    "👩\u{200D}💻",
    "🧑\u{200D}🤝\u{200D}🧑",
    "🏳\u{FE0F}\u{200D}🌈",
    "🐈\u{200D}⬛",
    // flag
    "🇯🇵",
    "🇩🇪",
    // skin tone
    "👋🏿",
    "✋🏻",
    // keycap
    "#\u{FE0F}\u{20E3}",
    "9\u{FE0F}\u{20E3}",
    // emoji-ext-a, Unicode 13 to 16
    "🪐",
    "🫠",
    "🫨",
    "🪿",
    "🫟",
    // zero
    "\u{2060}",
];

pub fn code_points(g: &str) -> String {
    g.chars()
        .map(|c| format!("U+{:04X}", c as u32))
        .collect::<Vec<_>>()
        .join(" ")
}
