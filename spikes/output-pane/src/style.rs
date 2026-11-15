//! The text style of a span. Colors are 24-bit RGB.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Style {
    pub fg: Option<u32>,
    pub bg: Option<u32>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Style {
    pub const fn fg(rgb: u32) -> Self {
        Self {
            fg: Some(rgb),
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

pub const fn rgb(c: u32) -> (u8, u8, u8) {
    ((c >> 16) as u8, (c >> 8) as u8, c as u8)
}
