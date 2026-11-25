//! The scenarios of the `cli-entrypoint` capability, run on the built binary.

use std::collections::BTreeMap;
use std::process::Output;

use proptest::prelude::*;

use support::{mu, stderr, stdout};

mod support;

const VERSION_LINE: &str = concat!("mu ", env!("CARGO_PKG_VERSION"), "\n");

/// The verbs in the commands section of a help text, each with its description.
fn listed_verbs(help: &str) -> BTreeMap<String, String> {
    help.lines()
        .skip_while(|line| *line != "Commands:")
        .skip(1)
        .take_while(|line| !line.is_empty())
        .map(|line| {
            let (verb, description) = line.trim().split_once(' ').unwrap_or((line.trim(), ""));
            (verb.to_owned(), description.trim().to_owned())
        })
        .collect()
}

fn assert_usage_error(output: &Output, verb: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "stdout is empty");
    assert!(names_verb(&stderr(output), verb), "stderr names {verb:?}");
}

/// The error quotes the verb, so a short verb cannot match other text of the error.
fn names_verb(stderr: &str, verb: &str) -> bool {
    stderr.contains(&format!("'{verb}'"))
}

#[test]
fn help_lists_verbs() {
    let output = mu(&["--help"]);
    let help = stdout(&output);
    assert_eq!(output.status.code(), Some(0));
    assert!(help.lines().any(|line| line.starts_with("Usage: mu")));

    let listed = listed_verbs(&help);
    let names: Vec<&str> = listed.keys().map(String::as_str).collect();
    assert_eq!(
        names,
        ["client", "config", "layout", "server", "version", "world"]
    );
    for (verb, description) in &listed {
        assert!(!description.is_empty(), "{verb} has a description");
    }
}

#[test]
fn help_for_one_verb() {
    for verb in ["config", "layout", "server", "world"] {
        let description = listed_verbs(&stdout(&mu(&["--help"])))
            .remove(verb)
            .expect("the help lists the verb");

        let output = mu(&[verb, "--help"]);
        let help = stdout(&output);
        assert_eq!(output.status.code(), Some(0), "{verb}");
        assert!(help.contains(&format!("Usage: mu {verb}")), "{verb}");
        assert!(help.contains(&description), "{verb}");
    }
}

#[test]
fn no_verb_equals_client() {
    let bare = mu(&[]);
    let client = mu(&["client"]);
    assert_eq!(bare.stdout, client.stdout);
    assert_eq!(bare.stderr, client.stderr);
    assert_eq!(bare.status.code(), client.status.code());
}

#[test]
fn version_verb() {
    let output = mu(&["version"]);
    assert_eq!(stdout(&output), VERSION_LINE);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn version_flag_matches_verb() {
    let flag = mu(&["--version"]);
    assert_eq!(flag.stdout, mu(&["version"]).stdout);
    assert_eq!(flag.status.code(), Some(0));
}

#[test]
fn unknown_verb_is_usage_error() {
    assert_usage_error(&mu(&["frobnicate"]), "frobnicate");
}

#[test]
fn help_is_not_a_verb() {
    assert_usage_error(&mu(&["help"]), "help");
}

#[test]
fn verb_names_are_exact() {
    for near in ["", "s", "serv", "servers", "Server", "CLIENT", "versio"] {
        assert_usage_error(&mu(&[near]), near);
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_verb_is_usage_error() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let output = support::mu_command(&[], &[])
        .arg(OsStr::from_bytes(b"serv\xff"))
        .output()
        .expect("the mu binary runs");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "stdout is empty");
}

proptest! {
    #[test]
    fn unknown_verb_property(
        verb in "[a-z][a-z0-9_]{0,15}"
            .prop_filter("not a verb", |verb| !mu_cli::verb_names().contains(verb))
    ) {
        let output = mu(&[&verb]);
        prop_assert_eq!(output.status.code(), Some(2));
        prop_assert!(output.stdout.is_empty());
        prop_assert!(names_verb(&stderr(&output), &verb));
    }
}
