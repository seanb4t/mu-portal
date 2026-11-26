//! The conversion of game text: SGR sequences become style spans, and every other escape
//! sequence goes.
//!
//! Each call starts a new walk in its ground state. Only the SGR style carries from one call to
//! the next, through the style that the caller keeps for each world.

use mu_config::ansi::{SCREEN, Sink, walk};
use ratatui::style::{Color, Modifier, Style};

/// The text of a game line with no control character, and its style spans.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Converted {
    /// The printable text.
    pub text: String,
    /// Pairs of a byte offset in `text` and the style that starts there, in order.
    pub spans: Vec<(usize, Style)>,
}

impl From<&str> for Converted {
    /// Text with no style, with each control character as U+FFFD.
    fn from(text: &str) -> Self {
        Self {
            text: crate::text::clean(text),
            spans: Vec::new(),
        }
    }
}

struct Converter<'a> {
    style: &'a mut Style,
    /// The style of the last span, or the default style before the first span.
    last: Style,
    out: Converted,
}

impl Sink for Converter<'_> {
    fn char(&mut self, c: char) {
        if *self.style != self.last {
            self.out.spans.push((self.out.text.len(), *self.style));
            self.last = *self.style;
        }
        self.out.text.push(c);
    }

    fn sgr(&mut self, groups: &[&[u16]]) {
        sgr(self.style, groups);
    }
}

/// Converts `input` from the style `style`, and leaves in `style` the style at its end.
pub fn convert(input: &str, style: &mut Style) -> Converted {
    let mut converter = Converter {
        style,
        last: Style::default(),
        out: Converted::default(),
    };
    walk(input, &SCREEN, &mut converter);
    converter.out
}

/// One of the 16 named colors: `index` 0 to 7, and `bright` for 90 to 97 and 100 to 107.
const fn named(index: u16, bright: bool) -> Color {
    match (index, bright) {
        (0, false) => Color::Black,
        (1, false) => Color::Red,
        (2, false) => Color::Green,
        (3, false) => Color::Yellow,
        (4, false) => Color::Blue,
        (5, false) => Color::Magenta,
        (6, false) => Color::Cyan,
        (7, false) => Color::Gray,
        (0, true) => Color::DarkGray,
        (1, true) => Color::LightRed,
        (2, true) => Color::LightGreen,
        (3, true) => Color::LightYellow,
        (4, true) => Color::LightBlue,
        (5, true) => Color::LightMagenta,
        (6, true) => Color::LightCyan,
        _ => Color::White,
    }
}

fn byte(value: u16) -> Option<u8> {
    u8::try_from(value).ok()
}

/// The color of the semicolon form of 38 or 48, from the values after it: `5;n` for an indexed
/// color, or `2;r;g;b` for a 24-bit color.
fn extended(values: &mut impl Iterator<Item = u16>) -> Option<Color> {
    match values.next()? {
        5 => byte(values.next()?).map(Color::Indexed),
        2 => Some(Color::Rgb(
            byte(values.next()?)?,
            byte(values.next()?)?,
            byte(values.next()?)?,
        )),
        _ => None,
    }
}

/// The color of the colon form of 38 or 48: `5:n`, `2:r:g:b`, or `2:<color space>:r:g:b`.
fn colon(values: &[u16]) -> Option<Color> {
    match *values {
        [5, n] => byte(n).map(Color::Indexed),
        [2, r, g, b] | [2, _, r, g, b] => Some(Color::Rgb(byte(r)?, byte(g)?, byte(b)?)),
        _ => None,
    }
}

/// Applies the SGR parameter groups `params` to `style`.
fn sgr(style: &mut Style, params: &[&[u16]]) {
    if params.is_empty() {
        *style = Style::default();
        return;
    }
    let mut groups = params.iter().copied();
    while let Some(group) = groups.next() {
        let code = group.first().copied().unwrap_or(0);
        match code {
            0 => *style = Style::default(),
            1 => *style = style.add_modifier(Modifier::BOLD),
            2 => *style = style.add_modifier(Modifier::DIM),
            3 => *style = style.add_modifier(Modifier::ITALIC),
            4 => *style = style.add_modifier(Modifier::UNDERLINED),
            5 => *style = style.add_modifier(Modifier::SLOW_BLINK),
            6 => *style = style.add_modifier(Modifier::RAPID_BLINK),
            7 => *style = style.add_modifier(Modifier::REVERSED),
            8 => *style = style.add_modifier(Modifier::HIDDEN),
            9 => *style = style.add_modifier(Modifier::CROSSED_OUT),
            22 => *style = style.remove_modifier(Modifier::BOLD | Modifier::DIM),
            23 => *style = style.remove_modifier(Modifier::ITALIC),
            24 => *style = style.remove_modifier(Modifier::UNDERLINED),
            25 => *style = style.remove_modifier(Modifier::SLOW_BLINK | Modifier::RAPID_BLINK),
            27 => *style = style.remove_modifier(Modifier::REVERSED),
            28 => *style = style.remove_modifier(Modifier::HIDDEN),
            29 => *style = style.remove_modifier(Modifier::CROSSED_OUT),
            30..=37 => *style = style.fg(named(code - 30, false)),
            39 => *style = style.fg(Color::Reset),
            40..=47 => *style = style.bg(named(code - 40, false)),
            49 => *style = style.bg(Color::Reset),
            90..=97 => *style = style.fg(named(code - 90, true)),
            100..=107 => *style = style.bg(named(code - 100, true)),
            38 | 48 => {
                // The colon form holds the color in its own group, and the semicolon form in
                // the groups after it.
                let color = match group {
                    [_, values @ ..] if !values.is_empty() => colon(values),
                    _ => extended(
                        &mut groups
                            .by_ref()
                            .map(|next| next.first().copied().unwrap_or(0)),
                    ),
                };
                if let Some(color) = color {
                    *style = if code == 38 {
                        style.fg(color)
                    } else {
                        style.bg(color)
                    };
                }
            }
            _ => {}
        }
    }
}
