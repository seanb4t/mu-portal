//! The width profile, learned from the reports of real terminals.

use mu_tui::width::{Class, Profile, Rule};

const REPORTS: [(&str, &str); 3] = [
    ("ghostty", include_str!("data/char-width-ghostty.tsv")),
    ("rex", include_str!("data/char-width-rex.tsv")),
    ("terminal", include_str!("data/char-width-terminal.tsv")),
];

/// The text of the code points of a report row, such as `U+0065 U+0301`.
fn grapheme(points: &str) -> String {
    points
        .split(' ')
        .map(|point| {
            let hex = point.strip_prefix("U+").unwrap();
            char::from_u32(u32::from_str_radix(hex, 16).unwrap()).unwrap()
        })
        .collect()
}

fn number(field: &str) -> u16 {
    field.split_once('=').unwrap().1.parse().unwrap()
}

/// The rows of `kind` in a report: each grapheme, the width that the terminal gave it, and the
/// rule name of a probe row.
fn rows<'a>(report: &'a str, kind: &str) -> Vec<(String, u16, &'a str)> {
    report
        .lines()
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .filter(|fields| fields[0] == kind)
        .map(|fields| {
            (
                grapheme(fields.last().unwrap()),
                number(fields[3]),
                fields[4],
            )
        })
        .collect()
}

fn learned(report: &str) -> Profile {
    let probes = rows(report, "probe");
    let measured: Vec<(&str, u16)> = probes.iter().map(|(g, w, _)| (g.as_str(), *w)).collect();
    Profile::learn(&measured)
}

fn rule_name(rule: Rule) -> String {
    match rule {
        Rule::Unicode => "rule=unicode".to_owned(),
        Rule::CharSum => "rule=char-sum".to_owned(),
        Rule::Const(width) => format!("rule=const({width})"),
    }
}

#[test]
fn each_probe_class_learns_the_rule_of_its_report() {
    for (terminal, report) in REPORTS {
        let profile = learned(report);
        for (g, _, rule) in rows(report, "probe") {
            assert_eq!(
                rule_name(profile.rule(Class::of(&g))),
                rule,
                "{terminal}: {g:?}"
            );
        }
    }
}

#[test]
fn learned_profile_gives_each_measured_width() {
    for (terminal, report) in REPORTS {
        let profile = learned(report);
        for (g, width, _) in rows(report, "test") {
            assert_eq!(profile.width(&g), usize::from(width), "{terminal}: {g:?}");
        }
    }
}

#[test]
fn reported_width_of_zero_keeps_the_unicode_width() {
    let profile = Profile::learn(&[("👍🏽", 0)]);
    assert_eq!(profile.rule(Class::SkinTone), Rule::Unicode);
}

#[test]
fn width_with_no_rule_is_a_constant() {
    let profile = Profile::learn(&[("🇺🇸", 1)]);
    assert_eq!(profile.width("🇫🇷"), 1);
}
