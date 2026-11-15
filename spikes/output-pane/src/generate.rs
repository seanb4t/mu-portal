//! Deterministic MUD-like output: short and long lines, color spans, some
//! wide and emoji text, and an image line now and then.

use crate::store::{ImageRef, LineStore};
use crate::style::Style;

pub const IMAGE: ImageRef = ImageRef {
    id: 1,
    cols: 24,
    rows: 8,
};

const WORDS: &[&str] = &[
    "the", "a", "you", "see", "dark", "room", "north", "south", "exits", "goblin", "sword", "gold",
    "coins", "says", "tells", "guard", "lantern", "door", "opens", "slowly", "ancient", "stone",
    "river", "flows", "east", "hits", "misses", "you!", "chat:", "[ooc]", "worth", "while",
    "dragon", "keep", "market", "square", "crowd", "whispers", "of", "and", "to",
];
const WIDE: &[&str] = &["漢字", "🐉", "東京", "👍🏽", "카페", "é"];
const COLORS: &[u32] = &[
    0xe0_6c75, 0x98_c379, 0xe5_c07b, 0x61_afef, 0xc6_78dd, 0x56_b6c2,
];

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

pub fn push_line(
    store: &mut LineStore,
    rng: &mut Rng,
    text: &mut String,
    spans: &mut Vec<(usize, Style)>,
) {
    text.clear();
    spans.clear();
    let words = match rng.below(100) {
        0..10 => 60 + rng.below(80),
        _ => 3 + rng.below(16),
    };
    let wide = rng.below(100) < 5;
    for i in 0..words {
        if i > 0 {
            text.push(' ');
        }
        if rng.below(12) == 0 {
            let st = if rng.below(3) == 0 {
                Style::default()
            } else {
                Style {
                    bold: rng.below(4) == 0,
                    ..Style::fg(COLORS[rng.below(COLORS.len())])
                }
            };
            spans.push((text.len(), st));
        }
        if wide && rng.below(5) == 0 {
            text.push_str(WIDE[rng.below(WIDE.len())]);
        } else {
            text.push_str(WORDS[rng.below(WORDS.len())]);
        }
    }
    store.push_text(text, spans);
}

/// Appends `n` lines. Every 25 000th line and the 15th line from the end show the image.
pub fn fill(store: &mut LineStore, n: usize, seed: u64) {
    let mut rng = Rng::new(seed);
    let (mut text, mut spans) = (String::new(), Vec::new());
    for i in 0..n {
        if i % 25_000 == 24_999 || i + 15 == n {
            store.push_image(IMAGE);
        } else {
            push_line(store, &mut rng, &mut text, &mut spans);
        }
    }
}
