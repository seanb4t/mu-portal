//! The line rules of a world: a match part, and the actions that apply to a line that matches.
//! [`RuleState::route`] is the one function that decides the streams of a line from the game.

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use fancy_regex::{Captures, Regex, RegexBuilder};
use mu_config::ansi::{PLACEHOLDER, SCREEN, Sink, walk};
use mu_config::stream::{STREAMS_MAX, StreamName, Streams};

use crate::world::{ConfigError, Entry, FieldError};

/// The count of lines of a capture with no `lines`.
pub const LINES_DEFAULT: u16 = 100;

/// The most lines of one capture.
pub const LINES_MAX: u16 = 1000;

/// The most steps of one match before it stops: the default of `fancy-regex`.
const BACKTRACK_LIMIT: usize = 1_000_000;

/// The fields of a rule with an action.
const FIELDS: &[&str] = &["id", "pattern", "until", "lines", "stream"];

/// The fields that are actions.
const ACTIONS: &[&str] = &["stream"];

/// One entry of `stream`: a fixed name, or a text with `$` that the captures expand.
#[derive(Clone, Debug)]
pub enum Template {
    /// A name with no `$`, in lower case.
    Fixed(StreamName),
    /// A text with `$`, which the captures of a match expand.
    Expand(String),
}

/// What a rule does to a line that matches.
#[derive(Clone, Debug)]
pub enum Action {
    /// Send the line to the streams of the templates.
    Stream(Vec<Template>),
}

/// The match part of a rule.
#[derive(Clone, Debug)]
struct Match {
    pattern: Regex,
    until: Option<Regex>,
    lines: u16,
}

/// One rule with an action.
#[derive(Clone, Debug)]
pub struct Rule {
    id: String,
    matcher: Match,
    actions: Vec<Action>,
}

fn regex(id: &str, field: &str, value: &toml::Value) -> Result<Regex, FieldError> {
    let text = value
        .as_str()
        .ok_or_else(|| field_error(id, field, "must be a string"))?;
    RegexBuilder::new(text)
        .backtrack_limit(BACKTRACK_LIMIT)
        .build()
        .map_err(|error| field_error(id, field, format!("does not compile: {error}")))
}

fn field_error(id: &str, field: &str, message: impl Into<String>) -> FieldError {
    FieldError::new(format!("rules.{id}.{field}"), message)
}

fn lines(id: &str, value: &toml::Value) -> Result<u16, FieldError> {
    value
        .as_integer()
        .and_then(|count| u16::try_from(count).ok())
        .filter(|count| (1..=LINES_MAX).contains(count))
        .ok_or_else(|| field_error(id, "lines", "must be an integer from 1 to 1000"))
}

fn template(id: &str, text: &str) -> Result<Template, FieldError> {
    if text.contains('$') {
        return Ok(Template::Expand(text.to_owned()));
    }
    StreamName::new(text)
        .map(Template::Fixed)
        .map_err(|reason| {
            field_error(
                id,
                "stream",
                format!("`{text}` is not a stream name: {reason}"),
            )
        })
}

fn templates(id: &str, value: &toml::Value) -> Result<Vec<Template>, FieldError> {
    let texts: Vec<&str> = match value {
        toml::Value::String(text) => vec![text.as_str()],
        toml::Value::Array(items) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .ok_or_else(|| field_error(id, "stream", "each entry must be a string"))
            })
            .collect::<Result<_, _>>()?,
        _ => {
            return Err(field_error(
                id,
                "stream",
                "must be a string or a list of strings",
            ));
        }
    };
    if texts.is_empty() || texts.len() > STREAMS_MAX {
        return Err(field_error(id, "stream", "must hold 1 to 8 names"));
    }
    texts.into_iter().map(|text| template(id, text)).collect()
}

impl Rule {
    /// The rule `id` of `table`, or none for a table with no action, which does nothing.
    ///
    /// # Errors
    ///
    /// A [`FieldError`] that names the id and the field.
    pub fn compile(id: &str, table: &toml::Table) -> Result<Option<Self>, FieldError> {
        if !table.keys().any(|key| ACTIONS.contains(&key.as_str())) {
            return Ok(None);
        }
        if let Some(key) = table.keys().find(|key| !FIELDS.contains(&key.as_str())) {
            return Err(field_error(id, key, "is not a field of a rule"));
        }
        let pattern = table
            .get("pattern")
            .ok_or_else(|| field_error(id, "pattern", "a rule with an action must have it"))?;
        let matcher = Match {
            pattern: regex(id, "pattern", pattern)?,
            until: table
                .get("until")
                .map(|until| regex(id, "until", until))
                .transpose()?,
            lines: table
                .get("lines")
                .map(|count| lines(id, count))
                .transpose()?
                .unwrap_or(LINES_DEFAULT),
        };
        let mut actions = Vec::new();
        if let Some(stream) = table.get("stream") {
            actions.push(Action::Stream(templates(id, stream)?));
        }
        Ok(Some(Self {
            id: id.to_owned(),
            matcher,
            actions,
        }))
    }

    /// The id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The streams of the action `stream` for a match with `captures`, or the bad name.
    fn streams(&self, captures: &Captures<'_, str>) -> Result<Option<Streams>, String> {
        let Some(Action::Stream(templates)) = self.actions.first() else {
            return Ok(None);
        };
        let mut names = Vec::with_capacity(templates.len());
        for template in templates {
            match template {
                Template::Fixed(name) => names.push(name.clone()),
                Template::Expand(text) => {
                    let mut expanded = String::new();
                    captures.expand(text, &mut expanded);
                    let lower = expanded.to_ascii_lowercase();
                    names.push(StreamName::new(&lower).map_err(|_| lower)?);
                }
            }
        }
        // The compile checked 1 to 8 templates, so the list is good.
        Ok(Streams::new(names).ok())
    }
}

/// The rules of one world in their merged order: each entry, and the rule of each entry that has
/// an action.
#[derive(Clone, Debug, Default)]
pub struct Rules(Arc<Inner>);

#[derive(Debug, Default)]
struct Inner {
    entries: Vec<Entry>,
    compiled: Vec<Rule>,
}

impl PartialEq for Rules {
    fn eq(&self, other: &Self) -> bool {
        self.0.entries == other.0.entries
    }
}

impl Rules {
    /// The rules of `entries`.
    ///
    /// # Errors
    ///
    /// A [`ConfigError`] at the layer of the first entry whose rule does not compile.
    pub fn new(entries: Vec<Entry>) -> Result<Self, ConfigError> {
        let mut compiled = Vec::new();
        for entry in &entries {
            let rule = Rule::compile(&entry.id, &entry.table)
                .map_err(|error| ConfigError::of_field(&entry.origin, &error))?;
            compiled.extend(rule);
        }
        Ok(Self(Arc::new(Inner { entries, compiled })))
    }

    /// Each entry, in order.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.0.entries
    }

    /// The rules with an action, in order.
    #[must_use]
    pub fn compiled(&self) -> &[Rule] {
        &self.0.compiled
    }
}

/// The kind of an error of a rule on one line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorKind {
    /// A name from the captures is not a stream name.
    BadName,
    /// A match passed the backtrack limit.
    SlowMatch,
}

/// An error of a rule on one line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleError {
    /// The id of the rule.
    pub rule: String,
    /// The kind of the error.
    pub kind: ErrorKind,
    /// The text of the error, which names the rule.
    pub text: String,
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// The streams of one line, and the errors of the rules on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    /// The streams.
    pub streams: Streams,
    /// The errors, in order.
    pub errors: Vec<RuleError>,
}

#[derive(Clone, Debug)]
struct Capture {
    rule: String,
    until: Option<Regex>,
    lines: u16,
    count: u16,
    streams: Streams,
}

/// The state of the rules of one world: the capture that runs, and the errors already reported.
#[derive(Clone, Debug, Default)]
pub struct RuleState {
    capture: Option<Capture>,
    reported: BTreeSet<(String, ErrorKind)>,
}

impl RuleState {
    /// The streams of the line `text` from the game, by `rules`.
    pub fn route(&mut self, rules: &Rules, text: &str) -> Route {
        let plain = plain(text);
        let mut errors = Vec::new();
        if let Some(capture) = &mut self.capture {
            capture.count += 1;
            let streams = capture.streams.clone();
            let ended = match capture
                .until
                .as_ref()
                .map(|until| until.is_match(plain.as_ref()))
            {
                Some(Ok(found)) => found,
                Some(Err(_)) => {
                    let rule = capture.rule.clone();
                    self.report(&mut errors, &rule, ErrorKind::SlowMatch, slow(&rule));
                    false
                }
                None => false,
            };
            if ended
                || self
                    .capture
                    .as_ref()
                    .is_some_and(|capture| capture.count >= capture.lines)
            {
                self.capture = None;
            }
            return Route { streams, errors };
        }
        for rule in rules.compiled() {
            let captures = match rule.matcher.pattern.captures(plain.as_ref()) {
                Ok(Some(captures)) => captures,
                Ok(None) => continue,
                Err(_) => {
                    self.report(&mut errors, &rule.id, ErrorKind::SlowMatch, slow(&rule.id));
                    continue;
                }
            };
            let streams = match rule.streams(&captures) {
                Ok(Some(streams)) => streams,
                Ok(None) => Streams::main(),
                Err(name) => {
                    let text = format!(
                        "rule {}: `{name}` is not a stream name, so the line goes to main",
                        rule.id
                    );
                    self.report(&mut errors, &rule.id, ErrorKind::BadName, text);
                    return Route {
                        streams: Streams::main(),
                        errors,
                    };
                }
            };
            if rule.matcher.until.is_some() && rule.matcher.lines > 1 {
                self.capture = Some(Capture {
                    rule: rule.id.clone(),
                    until: rule.matcher.until.clone(),
                    lines: rule.matcher.lines,
                    count: 1,
                    streams: streams.clone(),
                });
            }
            return Route { streams, errors };
        }
        Route {
            streams: Streams::main(),
            errors,
        }
    }

    /// Ends a capture, and forgets the errors already reported.
    pub fn reset(&mut self) {
        self.capture = None;
        self.reported.clear();
    }

    fn report(&mut self, errors: &mut Vec<RuleError>, rule: &str, kind: ErrorKind, text: String) {
        if self.reported.insert((rule.to_owned(), kind)) {
            errors.push(RuleError {
                rule: rule.to_owned(),
                kind,
                text,
            });
        }
    }
}

fn slow(rule: &str) -> String {
    format!("rule {rule}: the match passed the backtrack limit, so it counts as no match")
}

struct Plain(String);

impl Sink for Plain {
    fn char(&mut self, c: char) {
        self.0.push(c);
    }

    fn sgr(&mut self, _groups: &[&[u16]]) {}
}

/// The text of `line` as the screen shows it, without its colors. A line with no control
/// character comes back borrowed.
#[must_use]
pub fn plain(line: &str) -> Cow<'_, str> {
    if !line.chars().any(|c| c.is_control() || c == PLACEHOLDER) {
        return Cow::Borrowed(line);
    }
    let mut sink = Plain(String::with_capacity(line.len()));
    walk(line, &SCREEN, &mut sink);
    Cow::Owned(sink.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::LayerOrigin;

    fn table(text: &str) -> toml::Table {
        text.parse().expect("a table")
    }

    fn compile(text: &str) -> Result<Option<Rule>, FieldError> {
        Rule::compile("r", &table(text))
    }

    fn rules(texts: &[(&str, &str)]) -> Rules {
        let entries = texts
            .iter()
            .map(|(id, text)| Entry {
                id: (*id).to_owned(),
                table: table(text),
                origin: LayerOrigin::File("global.toml".to_owned()),
            })
            .collect();
        Rules::new(entries).expect("good rules")
    }

    fn names(route: &Route) -> String {
        route.streams.to_string()
    }

    /// The streams of each of `lines`, through one state.
    fn walk_lines(rules: &Rules, lines: &[&str]) -> Vec<String> {
        let mut state = RuleState::default();
        lines
            .iter()
            .map(|line| names(&state.route(rules, line)))
            .collect()
    }

    // The fields.

    #[test]
    fn lines_below_zero_is_refused() {
        let error = compile("pattern = 'x'\nstream = 'who'\nlines = -1").expect_err("below zero");
        assert_eq!(error.field(), "rules.r.lines");
    }

    #[test]
    fn lines_as_a_string_is_refused() {
        let error = compile("pattern = 'x'\nstream = 'who'\nlines = '3'").expect_err("a string");
        assert_eq!(error.field(), "rules.r.lines");
    }

    #[test]
    fn lines_of_1000_loads() {
        assert!(matches!(
            compile("pattern = 'x'\nstream = 'who'\nuntil = 'y'\nlines = 1000"),
            Ok(Some(_))
        ));
    }

    #[test]
    fn a_list_of_eight_streams_loads() {
        let list = (1..=8)
            .map(|n| format!("'s{n}'"))
            .collect::<Vec<_>>()
            .join(", ");
        assert!(matches!(
            compile(&format!("pattern = 'x'\nstream = [{list}]")),
            Ok(Some(_))
        ));
    }

    // The text that a pattern sees.

    #[test]
    fn plain_removes_color_codes() {
        assert_eq!(plain("\u{1b}[1m[Public]\u{1b}[0m hi"), "[Public] hi");
    }

    #[test]
    fn plain_shows_a_tab_as_a_space() {
        assert_eq!(plain("a\tb"), "a b");
    }

    #[test]
    fn plain_borrows_a_line_with_no_escape() {
        assert!(matches!(plain("[Public] hi"), Cow::Borrowed(_)));
    }

    // The order of the rules.

    #[test]
    fn the_earlier_rule_wins() {
        let rules = rules(&[
            ("pub", "pattern = '^\\[Public\\]'\nstream = 'public'"),
            ("chan", "pattern = '^\\['\nstream = 'chat'"),
        ]);
        assert_eq!(
            walk_lines(&rules, &["[Public] hi", "[Help] yo"]),
            ["public", "chat"]
        );
    }

    #[test]
    fn no_rule_matches() {
        let rules = rules(&[("chan", "pattern = '^\\['\nstream = 'chat'")]);
        assert_eq!(walk_lines(&rules, &["Bob says, \"hi\""]), ["main"]);
    }

    #[test]
    fn a_rule_with_no_action_does_nothing() {
        let rules = rules(&[("hl", "pattern = 'pages'\ncolour = 'red'")]);
        assert_eq!(
            (
                rules.compiled().len(),
                walk_lines(&rules, &["Bob pages: hi"])
            ),
            (0, vec!["main".to_owned()])
        );
    }

    // The templates.

    fn one(stream: &str, pattern: &str, line: &str) -> Route {
        let rules = rules(&[(
            "chan",
            &format!("pattern = '{pattern}'\nstream = '{stream}'"),
        )]);
        RuleState::default().route(&rules, line)
    }

    #[test]
    fn a_template_with_a_number() {
        assert_eq!(
            names(&one("chan-$1", "^\\[(\\w+)\\] ", "[Ooc] hi")),
            "chan-ooc"
        );
    }

    #[test]
    fn a_template_with_a_name() {
        assert_eq!(
            names(&one("chan-$chan", "^\\[(?<chan>\\w+)\\] ", "[Ooc] hi")),
            "chan-ooc"
        );
    }

    #[test]
    fn a_template_with_braces() {
        assert_eq!(names(&one("${1}a", "^\\[(\\w+)\\] ", "[Ooc] hi")), "ooca");
    }

    #[test]
    fn a_template_with_a_dollar() {
        let route = one("a$$", "^x", "x");
        assert_eq!(
            (
                names(&route),
                route.errors.first().map(|error| error.text.contains("a$"))
            ),
            ("main".to_owned(), Some(true))
        );
    }

    #[test]
    fn a_group_that_did_not_match_is_empty() {
        assert_eq!(names(&one("chan$2", "^(x)(y)?", "x")), "chan");
    }

    #[test]
    fn a_name_is_lower_case() {
        assert_eq!(
            names(&one("chan-$1", "^\\[([\\w-]+)\\] ", "[RP-Help] yo")),
            "chan-rp-help"
        );
    }

    #[test]
    fn a_fixed_name_is_lower_case() {
        assert_eq!(names(&one("Chat", "^\\[", "[Public] hi")), "chat");
    }

    #[test]
    fn a_list_of_streams() {
        let rules = rules(&[(
            "page",
            "pattern = 'pages: '\nstream = ['pages', 'main', 'pages']",
        )]);
        assert_eq!(walk_lines(&rules, &["Bob pages: hi"]), ["pages,main"]);
    }

    // The bad names.

    fn bad_name_rules() -> Rules {
        rules(&[(
            "chan",
            "pattern = '^\\[([^\\]]+)\\] '\nstream = ['chat', 'chan-$1']",
        )])
    }

    #[test]
    fn one_bad_name_gives_main() {
        let route = RuleState::default().route(&bad_name_rules(), "[Role Play] hi");
        assert_eq!((names(&route), route.errors.len()), ("main".to_owned(), 1));
    }

    #[test]
    fn a_bad_name_is_reported_once() {
        let rules = bad_name_rules();
        let mut state = RuleState::default();
        let first = state.route(&rules, "[Role Play] one");
        let second = state.route(&rules, "[Role Play] two");
        assert_eq!(
            (
                first.errors.len(),
                second.errors.len(),
                first.errors[0].kind
            ),
            (1, 0, ErrorKind::BadName)
        );
        assert!(
            first.errors[0].text.contains("chan")
                && first.errors[0].text.contains("chan-role play"),
            "{}",
            first.errors[0].text
        );
    }

    #[test]
    fn reset_reports_a_bad_name_again() {
        let rules = bad_name_rules();
        let mut state = RuleState::default();
        state.route(&rules, "[Role Play] one");
        state.reset();
        assert_eq!(state.route(&rules, "[Role Play] two").errors.len(), 1);
    }

    /// A pattern that passes the backtrack limit on a line of 40 `a` characters.
    const SLOW: &str = "^(a|aa)+\\\\1b";

    #[test]
    fn a_rule_reports_each_kind_once() {
        let rules = rules(&[(
            "both",
            &format!("pattern = \"{SLOW}|^\\\\[([^\\\\]]+)\\\\] \"\nstream = 'chan-$2'"),
        )]);
        let mut state = RuleState::default();
        let slow = state.route(&rules, &"a".repeat(40));
        let bad = state.route(&rules, "[Role Play] hi");
        let again = state.route(&rules, &"a".repeat(40));
        assert_eq!(
            (
                slow.errors
                    .iter()
                    .map(|error| error.kind)
                    .collect::<Vec<_>>(),
                bad.errors
                    .iter()
                    .map(|error| error.kind)
                    .collect::<Vec<_>>(),
                again.errors.len()
            ),
            (vec![ErrorKind::SlowMatch], vec![ErrorKind::BadName], 0)
        );
    }

    #[test]
    fn two_errors_on_one_line() {
        let rules = rules(&[
            ("slow", &format!("pattern = \"{SLOW}\"\nstream = 'slow'")),
            ("chan", "pattern = '^(a+)'\nstream = 'chan-$1 x'"),
        ]);
        let route = RuleState::default().route(&rules, &"a".repeat(40));
        assert_eq!(
            route
                .errors
                .iter()
                .map(|error| (error.rule.as_str(), error.kind))
                .collect::<Vec<_>>(),
            [("slow", ErrorKind::SlowMatch), ("chan", ErrorKind::BadName)]
        );
    }

    #[test]
    fn a_slow_match_counts_as_no_match() {
        let rules = rules(&[
            ("slow", &format!("pattern = \"{SLOW}\"\nstream = 'slow'")),
            ("all", "pattern = '^a'\nstream = 'rest'"),
        ]);
        let route = RuleState::default().route(&rules, &"a".repeat(40));
        assert_eq!((names(&route), route.errors.len()), ("rest".to_owned(), 1));
    }

    // The captures.

    fn who(extra: &str) -> Rules {
        rules(&[
            (
                "who",
                &format!("pattern = '^WHO'\nuntil = '^End'\nstream = 'who'\n{extra}"),
            ),
            ("chan", "pattern = '^\\['\nstream = 'chat'"),
        ])
    }

    #[test]
    fn a_capture_ends_at_until() {
        assert_eq!(
            walk_lines(&who(""), &["WHO", "[Public] hi", "End", "[Public] yo"]),
            ["who", "who", "who", "chat"]
        );
    }

    #[test]
    fn the_first_line_does_not_end_a_capture() {
        let rules = rules(&[("who", "pattern = '^WHO'\nuntil = 'WHO'\nstream = 'who'")]);
        assert_eq!(
            walk_lines(&rules, &["WHO list", "Aria", "end of WHO", "x"]),
            ["who", "who", "who", "main"]
        );
    }

    #[test]
    fn a_capture_ends_at_its_limit() {
        assert_eq!(
            walk_lines(&who("lines = 2"), &["WHO", "Aria", "Bob", "WHO"]),
            ["who", "who", "main", "who"]
        );
    }

    #[test]
    fn a_capture_of_one_line() {
        assert_eq!(
            walk_lines(&who("lines = 1"), &["WHO", "Aria"]),
            ["who", "main"]
        );
    }

    #[test]
    fn a_bad_name_starts_no_capture() {
        let rules = rules(&[(
            "who",
            "pattern = '^WHO (.+)$'\nuntil = '^End'\nstream = 'who-$1'",
        )]);
        assert_eq!(walk_lines(&rules, &["WHO a b", "Aria"]), ["main", "main"]);
    }

    #[test]
    fn a_slow_until_keeps_the_capture() {
        let rules = rules(&[(
            "who",
            &format!("pattern = '^WHO'\nuntil = \"{SLOW}\"\nstream = 'who'"),
        )]);
        let mut state = RuleState::default();
        state.route(&rules, "WHO");
        let slow = state.route(&rules, &"a".repeat(40));
        let next = state.route(&rules, "Aria");
        assert_eq!(
            (names(&slow), slow.errors.len(), names(&next)),
            ("who".to_owned(), 1, "who".to_owned())
        );
    }

    #[test]
    fn reset_ends_a_capture() {
        let rules = who("");
        let mut state = RuleState::default();
        state.route(&rules, "WHO");
        state.reset();
        assert_eq!(names(&state.route(&rules, "Aria")), "main");
    }
}
