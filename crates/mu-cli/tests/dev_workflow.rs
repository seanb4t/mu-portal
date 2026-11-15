//! The scenarios of the `developer-workflow` capability. Each test copies the
//! files of the repository, adds one defect to the copy and runs a task there.
//!
//! Every test here is ignored, so the `test` task does not run them. A `check`
//! in a copy would otherwise start these tests again, and each one makes a copy.
//! `mise run test:workflow` runs them.

use std::path::Path;
use std::process::{Command, Output};

use support::{TreeCopy, stderr, stdout};

mod support;

const SLOW: &str = "slow: runs a mise task on a copy of the repository";

fn repo_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// A command with the environment of the test runner removed, so that a
/// nested Cargo or nextest starts as it does for a contributor.
fn command(program: &str, directory: &Path) -> Command {
    let mut command = Command::new(program);
    command.current_dir(directory);
    for (name, _) in std::env::vars_os() {
        let name = name.to_string_lossy().into_owned();
        let from_runner = name.starts_with("NEXTEST") || name.starts_with("CARGO_");
        if from_runner && name != "CARGO_HOME" {
            command.env_remove(name);
        }
    }
    command
}

fn run(program: &str, arguments: &[&str], directory: &Path) -> Output {
    command(program, directory)
        .args(arguments)
        // mise does not run a configuration that it does not trust.
        .env("MISE_TRUSTED_CONFIG_PATHS", directory)
        .output()
        .unwrap_or_else(|error| panic!("{program} runs: {error}"))
}

/// A copy of the files that git tracks or does not ignore, as a git repository.
/// hk reads its file list from git. The copy has its own Cargo target directory.
fn copy_of_repository() -> TreeCopy {
    let listed = run(
        "git",
        &["ls-files", "--cached", "--others", "--exclude-standard"],
        repo_root(),
    );
    assert!(listed.status.success(), "git lists the files");
    let files: Vec<String> = stdout(&listed)
        .lines()
        .filter(|file| repo_root().join(file).is_file())
        .map(str::to_owned)
        .collect();
    let copy = TreeCopy::of_files(repo_root(), &files);
    for arguments in [&["init", "--quiet"][..], &["add", "--all"]] {
        assert!(run("git", arguments, copy.root()).status.success());
    }
    copy
}

fn mise_run(task: &str, copy: &TreeCopy) -> Output {
    run("mise", &["run", task], copy.root())
}

/// The output of a command with the color sequences removed. CI turns colors on.
fn text(output: &Output) -> String {
    let raw = stdout(output) + &stderr(output);
    let mut text = String::with_capacity(raw.len());
    let mut characters = raw.chars();
    while let Some(character) = characters.next() {
        if character == '\u{1b}' {
            // A color sequence is ESC, `[`, parameters and one final letter.
            for next in characters.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            text.push(character);
        }
    }
    text
}

/// Asserts that the task failed and that mise names `failed_task` as a failed task.
fn assert_task_failed(output: &Output, failed_task: &str) {
    let text = text(output);
    assert!(!output.status.success(), "the task fails:\n{text}");
    assert!(
        text.contains(&format!("[{failed_task}] ERROR")),
        "the {failed_task} task fails:\n{text}"
    );
}

fn core_library() -> &'static Path {
    Path::new("crates/mu-core/src/lib.rs")
}

fn with_formatting_defect() -> TreeCopy {
    copy_of_repository().append_file(core_library(), "\n/// Not formatted.\npub fn  ugly( ) {}\n")
}

#[test]
#[ignore = "slow: runs a mise task on a copy of the repository"]
fn check_passes_on_clean_tree() {
    let output = mise_run("check", &copy_of_repository());
    assert!(output.status.success(), "{SLOW}:\n{}", text(&output));
}

#[test]
#[ignore = "slow: runs a mise task on a copy of the repository"]
fn check_fails_on_formatting_defect() {
    let output = mise_run("check", &with_formatting_defect());
    assert_task_failed(&output, "fmt");
}

#[test]
#[ignore = "slow: runs a mise task on a copy of the repository"]
fn check_fails_on_lint_defect() {
    // Clippy warns about a `&Vec` parameter. The code is formatted and documented.
    let defect = "
/// Counts the items.
#[must_use]
pub fn count(items: &Vec<u8>) -> usize {
    items.len()
}
";
    let copy = copy_of_repository().append_file(core_library(), defect);
    assert_task_failed(&mise_run("check", &copy), "lint");
}

#[test]
#[ignore = "slow: runs a mise task on a copy of the repository"]
fn check_fails_on_failing_test() {
    // One line, so that the attribute in this text is not an attribute line of this file.
    let defect = "//! A test that fails.\n\n#[test]\nfn fails() {\n    assert!(std::env::var_os(\"MU_PORTAL_NEVER_SET\").is_some());\n}\n";
    let copy =
        copy_of_repository().append_file(Path::new("crates/mu-core/tests/failing.rs"), defect);
    assert_task_failed(&mise_run("check", &copy), "test");
}

#[test]
#[ignore = "slow: runs a mise task on a copy of the repository"]
fn check_fails_on_failing_doc_test() {
    let defect = "
/// Adds one.
///
/// ```
/// assert_eq!(mu_core::add_one(1), 3);
/// ```
#[must_use]
pub fn add_one(value: u8) -> u8 {
    value + 1
}
";
    let copy = copy_of_repository().append_file(core_library(), defect);
    let output = mise_run("check", &copy);
    assert_task_failed(&output, "test");
    // nextest does not run doc tests, so the failure comes from the doc test step.
    assert!(text(&output).contains("add_one"), "the doc test fails");
}

#[test]
#[ignore = "slow: runs a mise task on a copy of the repository"]
fn check_fails_on_broken_doc_link() {
    let defect = "\n/// See [`Missing`].\npub struct Documented;\n";
    let copy = copy_of_repository().append_file(core_library(), defect);
    assert_task_failed(&mise_run("check", &copy), "doc");
}

#[test]
#[ignore = "slow: runs a mise task on a copy of the repository"]
fn deny_rejects_disallowed_license() {
    // The added crate inherits `publish = false`, so it is a private crate.
    let copy = copy_of_repository()
        .add_crate("crates/mu-gpl", "mu-gpl")
        .replace(
            "mu-gpl",
            "license.workspace = true",
            "license = \"GPL-3.0-only\"",
        );
    // A new member changes the lock file, and the deny task uses `--locked`.
    let locked = run(
        "cargo",
        &["metadata", "--format-version", "1", "--offline"],
        copy.root(),
    );
    assert!(
        locked.status.success(),
        "cargo updates the lock file:\n{}",
        text(&locked)
    );

    let output = mise_run("deny", &copy);
    let text = text(&output);
    assert!(!output.status.success(), "the deny task fails:\n{text}");
    assert!(
        text.contains("mu-gpl"),
        "the output names the crate:\n{text}"
    );
    assert!(
        text.contains("GPL-3.0-only"),
        "the output names the license:\n{text}"
    );
}

#[test]
#[ignore = "slow: runs a mise task on a copy of the repository"]
fn hooks_check_fails_on_formatting_defect() {
    let copy = with_formatting_defect();
    assert!(run("git", &["add", "--all"], copy.root()).status.success());
    let output = mise_run("hooks:check", &copy);
    assert!(
        !output.status.success(),
        "the hook check fails:\n{}",
        text(&output)
    );
}

#[test]
#[ignore = "slow: runs a mise task on a copy of the repository"]
fn hooks_check_passes_on_clean_tree() {
    let output = mise_run("hooks:check", &copy_of_repository());
    assert!(output.status.success(), "{SLOW}:\n{}", text(&output));
}
