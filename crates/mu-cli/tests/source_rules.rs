//! Rules of the Rust source of `mu-core` that a later change can break without a failing
//! build. The test reads each Rust file of the crate as tokens, so a comment or a string
//! never counts. A hit names the file and the line.
//!
//! - The codes of telnet live only in `src/telnet.rs`.
//! - An option code after `WILL`, `WONT`, `DO`, `DONT` or `SB` is a constant of `telnet::opt`.
//! - No file of `tests` sleeps. A file of `src` sleeps only where `ALLOWED` names it.
//!
//! `ALLOWED` names each accepted hit with its count and a reason.

use std::fmt;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use proc_macro2::{Literal, TokenStream, TokenTree};

mod support;

/// The one file that holds the codes of telnet.
const TELNET_HOME: &str = "src/telnet.rs";

/// The codes of telnet that a literal can hold: the commands from EOR to IAC, and TIMING-MARK.
const CODES: [u8; 18] = [
    239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255, 6,
];

/// The names of the codes that an option code follows in a list of bytes.
const OPTION_VERBS: [&str; 5] = ["WILL", "WONT", "DO", "DONT", "SB"];

/// The names of a call that waits for time.
const SLEEPS: [&str; 2] = ["sleep", "sleep_until"];

/// A hit that the test accepts: a literal with a code of telnet that is not a code of telnet, or
/// a timer of the engine.
struct Allowed {
    rule: Rule,
    file: &'static str,
    /// The text of the hit in the source.
    text: &'static str,
    /// How many times the file holds it. A different count fails the test.
    count: usize,
    reason: &'static str,
}

const ALLOWED: &[Allowed] = &[
    Allowed {
        rule: Rule::TelnetCode,
        file: "src/secret.rs",
        text: "0xff",
        count: 1,
        reason: "a byte that is not UTF-8, in the test of a secret that is not Unicode",
    },
    Allowed {
        rule: Rule::Sleep,
        file: "src/session.rs",
        text: "sleep_until",
        count: 1,
        reason: "the timer of the session: the moment of the login",
    },
];

#[derive(Debug, PartialEq, Eq)]
enum Rule {
    TelnetCode,
    OptionCode,
    Sleep,
}

#[derive(Debug, PartialEq, Eq)]
struct Hit {
    file: String,
    line: usize,
    rule: Rule,
    /// The text of the source that broke the rule.
    text: String,
}

impl fmt::Display for Hit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.rule {
            Rule::TelnetCode => "a code of telnet outside src/telnet.rs",
            Rule::OptionCode => "an option code that is not a constant of telnet::opt",
            Rule::Sleep => "a sleep",
        };
        write!(f, "{}:{}: {what}: {}", self.file, self.line, self.text)
    }
}

/// The root of `mu-core`. This test lives in `mu-cli`, so it never scans itself.
fn crate_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../mu-core"))
}

/// Each Rust file of `src` and `tests`, with its path from the crate root.
fn rust_files() -> Vec<(String, String)> {
    ["src", "tests"]
        .iter()
        .flat_map(|dir| support::files_under(&crate_root().join(dir)))
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .map(|path| {
            let relative = path.strip_prefix(crate_root()).unwrap();
            let name = relative.to_string_lossy().replace('\\', "/");
            (name, fs::read_to_string(&path).unwrap())
        })
        .collect()
}

fn is_code(value: u128) -> bool {
    CODES.iter().any(|code| u128::from(*code) == value)
}

/// The hits of `file`, which holds `text`, with a code of telnet only if `telnet_rule` is set.
fn scan(file: &str, text: &str, telnet_rule: bool) -> Vec<Hit> {
    let tokens = TokenStream::from_str(text)
        .unwrap_or_else(|error| panic!("{file} is not a stream of tokens: {error}"));
    let mut hits = Vec::new();
    scan_tokens(file, tokens, telnet_rule, &mut hits);
    hits
}

fn scan_tokens(file: &str, tokens: TokenStream, telnet_rule: bool, hits: &mut Vec<Hit>) {
    let trees: Vec<TokenTree> = tokens.into_iter().collect();
    for (index, tree) in trees.iter().enumerate() {
        let found = match tree {
            TokenTree::Group(group) => {
                scan_tokens(file, group.stream(), telnet_rule, hits);
                None
            }
            TokenTree::Literal(literal) if telnet_rule && holds_code(literal) => Some((
                Rule::TelnetCode,
                literal.span().start().line,
                literal.to_string(),
            )),
            TokenTree::Ident(ident) => {
                let name = ident.to_string();
                let line = ident.span().start().line;
                let after = &trees[index + 1..];
                if SLEEPS.contains(&name.as_str()) {
                    Some((Rule::Sleep, line, name))
                } else if telnet_rule && name == "u8" && is_u8_max(after) {
                    Some((Rule::TelnetCode, line, "u8::MAX".to_owned()))
                } else if telnet_rule && name == "u8" && is_assigned_six(after) {
                    Some((Rule::TelnetCode, line, "u8 = 6".to_owned()))
                } else if telnet_rule && OPTION_VERBS.contains(&name.as_str()) {
                    option_literal(after)
                        .map(|text| (Rule::OptionCode, line, format!("{name}, {text}")))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some((rule, line, text)) = found {
            hits.push(Hit {
                file: file.to_owned(),
                line,
                rule,
                text,
            });
        }
    }
}

/// True if `literal` is an integer, a byte or a byte string that holds a code of telnet.
/// A plain 6 is a code only in the forms that name it a `u8`, which other functions find.
fn holds_code(literal: &Literal) -> bool {
    match syn::Lit::new(literal.clone()) {
        syn::Lit::Int(int) => match int.base10_digits().parse::<u128>() {
            Ok(value) => (value != 6 && is_code(value)) || (value == 6 && int.suffix() == "u8"),
            Err(_) => false,
        },
        syn::Lit::Byte(byte) => is_code(u128::from(byte.value())),
        syn::Lit::ByteStr(bytes) => bytes.value().iter().any(|byte| is_code(u128::from(*byte))),
        _ => false,
    }
}

/// The integer literal in `after` if it starts with `, <integer>`, an option code after a verb.
fn option_literal(after: &[TokenTree]) -> Option<String> {
    match after {
        [TokenTree::Punct(comma), TokenTree::Literal(value), ..]
            if comma.as_char() == ','
                && matches!(syn::Lit::new(value.clone()), syn::Lit::Int(_)) =>
        {
            Some(value.to_string())
        }
        _ => None,
    }
}

/// True if `after` starts with `::MAX`, the rest of `u8::MAX`.
fn is_u8_max(after: &[TokenTree]) -> bool {
    matches!(
        after,
        [TokenTree::Punct(first), TokenTree::Punct(second), TokenTree::Ident(name), ..]
            if first.as_char() == ':' && second.as_char() == ':' && name == "MAX"
    )
}

/// True if `after` starts with `= 6`, so that the `u8` before it is the type of a 6.
fn is_assigned_six(after: &[TokenTree]) -> bool {
    matches!(
        after,
        [TokenTree::Punct(equals), TokenTree::Literal(value), ..]
            if equals.as_char() == '='
                && matches!(syn::Lit::new(value.clone()), syn::Lit::Int(int)
                    if int.base10_digits() == "6")
    )
}

/// The hits of `files` that no entry of `allowed` accepts, and each entry of `allowed` with
/// another count.
fn crate_hits<'a>(
    files: &[(String, String)],
    allowed: impl IntoIterator<Item = &'a Allowed>,
) -> Vec<String> {
    let mut hits: Vec<Hit> = files
        .iter()
        .flat_map(|(name, text)| scan(name, text, name != TELNET_HOME))
        .collect();
    let mut problems = Vec::new();
    for allowed in allowed {
        let (accepted, rest): (Vec<Hit>, Vec<Hit>) = hits.into_iter().partition(|hit| {
            hit.rule == allowed.rule && hit.file == allowed.file && hit.text == allowed.text
        });
        hits = rest;
        if accepted.len() != allowed.count {
            problems.push(format!(
                "{} holds `{}` {} times, and the allowed count is {} ({})",
                allowed.file,
                allowed.text,
                accepted.len(),
                allowed.count,
                allowed.reason
            ));
        }
    }
    problems.extend(hits.iter().map(Hit::to_string));
    problems
}

#[test]
fn source_of_mu_core_keeps_its_rules() {
    assert_eq!(crate_hits(&rust_files(), ALLOWED), Vec::<String>::new());
}

/// The entries of `ALLOWED` for `file`.
fn entries_of(file: &str) -> impl Iterator<Item = &'static Allowed> {
    ALLOWED.iter().filter(move |allowed| allowed.file == file)
}

#[test]
fn allowed_sleep_holds_only_in_its_file() {
    let sleep = "fn f() { tokio::time::sleep_until(t).await; }";
    let files = vec![
        ("src/session.rs".to_owned(), sleep.to_owned()),
        ("tests/other.rs".to_owned(), sleep.to_owned()),
    ];
    assert_eq!(
        crate_hits(&files, entries_of("src/session.rs")),
        ["tests/other.rs:1: a sleep: sleep_until"]
    );
}

/// The rules on `text` as a file other than `src/telnet.rs`.
fn probe(text: &str) -> Vec<Hit> {
    scan("src/probe.rs", text, true)
}

fn rules_of(text: &str) -> Vec<Rule> {
    probe(text).into_iter().map(|hit| hit.rule).collect()
}

#[test]
fn telnet_code_is_found_in_each_form() {
    let forms = [
        "const IAC: u8 = 255;",
        "const IAC: u8 = 0xFF;",
        "const IAC: u8 = 0xff;",
        "const IAC: u8 = 255u8;",
        "const WILL: u8 = 251_u8;",
        "const DO: u8 = 0xfd;",
        "const TM: u8 = 6;",
        "const TM: u8 = 0x06;",
        "const TM: u8 = 6u8;",
        "const TM: u8 = 0x6_u8;",
        "const MARK: [u8; 3] = [255, 253, 6];",
        "const IAC: u8 = u8::MAX;",
        "const IAC: u8 = b'\\xff';",
        "const DO: u8 = b'\\xfd';",
        r#"const MARK: &[u8] = b"\xff\xfd\x06";"#,
        "fn f() { g(&[1, 2, 0xfb]); }",
        "const SB: u8 = 250;",
        "const EOR: u8 = 239;",
        "const _: () = { let _ = u8::MAX; };",
    ];
    for form in forms {
        assert_eq!(rules_of(form).first(), Some(&Rule::TelnetCode), "{form}");
    }
}

#[test]
fn telnet_code_is_found_in_a_macro_body() {
    assert_eq!(
        probe("fn f() {\n    assert_eq!(read(), vec![\n 0xff]);\n}"),
        [Hit {
            file: "src/probe.rs".to_owned(),
            line: 3,
            rule: Rule::TelnetCode,
            text: "0xff".to_owned(),
        }]
    );
}

#[test]
fn bytes_that_are_not_codes_pass() {
    let forms = [
        "const A: u8 = 5;",
        "const B: u8 = 0x66;",
        "const C: usize = 6;",
        "let x = 238;",
        "let y = 256;",
        "let z = [0, 1, 6];",
        "let w = 6u32;",
        r#"let s = "255 0xff \\xff sleep";"#,
        "// 255 0xff u8::MAX sleep(1)",
        "/// 255 and sleep",
        "use crate::telnet::{DO, IAC, opt};",
        "let line = [IAC, DO, opt::TIMING_MARK];",
        "const MAX: u8 = 7; let m = u16::MAX;",
        r#"let b = b"abc";"#,
        "let c = b'a';",
        "let g = [IAC, WILL, opt::GMCP];",
        "let h = [IAC, SB, opt::TTYPE, 1, IAC, SE];",
    ];
    for form in forms {
        assert_eq!(probe(form), [], "{form}");
    }
}

#[test]
fn option_code_is_found_after_each_verb() {
    for verb in OPTION_VERBS {
        let form = format!("let x = [IAC, {verb}, 201];");
        assert_eq!(rules_of(&form), [Rule::OptionCode], "{form}");
    }
}

#[test]
fn telnet_home_may_hold_the_codes() {
    assert_eq!(scan(TELNET_HOME, "pub const IAC: u8 = 255;", false), []);
}

#[test]
fn sleep_is_found_in_each_form() {
    let forms = [
        "fn f() { sleep(d); }",
        "fn f() { tokio::time::sleep(d).await; }",
        "fn f() { std::thread::sleep(d); }",
        "fn f() { thread::sleep (d); }",
        "fn f() { sleep_until(t).await; }",
        "use tokio::time::sleep;",
        "fn f() { let _ = foo(|| sleep(d)); }",
    ];
    for form in forms {
        assert!(rules_of(form).contains(&Rule::Sleep), "{form}");
    }
}

#[test]
fn sleep_is_found_in_the_home_of_the_codes() {
    assert_eq!(
        scan(TELNET_HOME, "fn f() { sleep(d); }", false)
            .first()
            .map(|hit| &hit.rule),
        Some(&Rule::Sleep)
    );
}

#[test]
fn time_that_does_not_wait_passes() {
    let forms = [
        "fn f() { tokio::time::timeout(d, g).await; }",
        "fn f() { tokio::time::advance(d).await; }",
        "let asleep = true;",
    ];
    for form in forms {
        assert_eq!(probe(form), [], "{form}");
    }
}

#[test]
fn an_allowed_literal_with_another_count_fails() {
    let files = vec![(
        "src/secret.rs".to_owned(),
        "fn f() { let _ = [0xff, 0xff]; }".to_owned(),
    )];
    let problems = crate_hits(&files, entries_of("src/secret.rs"));
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("2 times"), "{problems:?}");
}

#[test]
fn an_allowed_literal_that_is_gone_fails() {
    let files = vec![("src/secret.rs".to_owned(), "fn f() {}".to_owned())];
    assert_eq!(crate_hits(&files, entries_of("src/secret.rs")).len(), 1);
}

#[test]
fn a_code_outside_the_list_fails_with_file_and_line() {
    let files = vec![
        (
            "src/secret.rs".to_owned(),
            "fn f() { let _ = 0xff; }".to_owned(),
        ),
        (
            "tests/other.rs".to_owned(),
            "fn f() {}\nconst IAC: u8 = 0xff;".to_owned(),
        ),
    ];
    assert_eq!(
        crate_hits(&files, entries_of("src/secret.rs")),
        ["tests/other.rs:2: a code of telnet outside src/telnet.rs: 0xff"]
    );
}
