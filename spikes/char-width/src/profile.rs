//! A width profile learned from the startup probe.
//!
//! For each class the probe measures one grapheme and keeps the rule that gives that
//! width: the `unicode-width` value, the sum of the code points, or a constant.
//! The zero class sets the width of a zero-width format character inside a sum.

use std::collections::HashMap;

use unicode_width::UnicodeWidthStr;

use crate::class::{Class, char_sum, classify};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Unicode,
    CharSum,
    Const(u16),
}

impl Rule {
    pub fn name(self) -> String {
        match self {
            Rule::Unicode => "unicode".into(),
            Rule::CharSum => "char-sum".into(),
            Rule::Const(n) => format!("const({n})"),
        }
    }
}

pub fn unicode(g: &str) -> u16 {
    g.width() as u16
}

#[derive(Debug, Default)]
pub struct Profile {
    pub rules: HashMap<Class, Rule>,
    /// The width the terminal gives a zero-width format character such as ZWJ.
    pub zw: u16,
}

impl Profile {
    pub fn learn(measured: &[(&str, u16)]) -> Self {
        let zw = measured
            .iter()
            .find(|(g, _)| classify(g) == Class::Zero)
            .map_or(0, |&(_, m)| m);
        let mut rules = HashMap::new();
        for &(g, m) in measured {
            let rule = if m == unicode(g) {
                Rule::Unicode
            } else if m == char_sum(g, zw) {
                Rule::CharSum
            } else {
                Rule::Const(m)
            };
            rules.insert(classify(g), rule);
        }
        Profile { rules, zw }
    }

    pub fn width(&self, g: &str) -> u16 {
        match self
            .rules
            .get(&classify(g))
            .copied()
            .unwrap_or(Rule::Unicode)
        {
            Rule::Unicode => unicode(g),
            Rule::CharSum => char_sum(g, self.zw),
            Rule::Const(n) => n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learns_one_rule_per_class() {
        let p = Profile::learn(&[("❤\u{FE0F}", 2), ("👨\u{200D}👩\u{200D}👧", 6), ("🫩", 1)]);
        assert_eq!(p.rules[&Class::Vs16], Rule::Unicode);
        assert_eq!(p.rules[&Class::Zwj], Rule::CharSum);
        assert_eq!(p.rules[&Class::EmojiExtA], Rule::Const(1));
    }

    #[test]
    fn counts_joiners_as_the_zero_class_measures() {
        let p = Profile::learn(&[("\u{200B}", 1), ("👨\u{200D}👩\u{200D}👧", 8)]);
        assert_eq!(p.rules[&Class::Zwj], Rule::CharSum);
        assert_eq!(p.width("👩\u{200D}💻"), 5);
        assert_eq!(p.width("🏳\u{FE0F}\u{200D}🌈"), 4);
    }

    #[test]
    fn applies_the_rule_to_other_members() {
        let p = Profile::learn(&[("👨\u{200D}👩\u{200D}👧", 6), ("🫩", 1)]);
        assert_eq!(p.width("👩\u{200D}💻"), 4);
        assert_eq!(p.width("🫠"), 1);
        assert_eq!(p.width("漢"), 2);
    }
}
