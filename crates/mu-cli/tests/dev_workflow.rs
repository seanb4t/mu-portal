//! The scenarios of the `developer-workflow` capability. Each test copies the
//! files of the repository, adds one defect to the copy and runs a task there.
//!
//! These tests are slow, and each `check` in a copy runs the `test` task. The
//! test target therefore needs the feature `workflow-tests`, which the `test`
//! task does not set. `mise run test:workflow` sets it.

use std::path::Path;
use std::process::{Command, Output};

use support::{TreeCopy, repo_root, stderr, stdout};

mod support;

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
        // Each copy builds into its own directory, whatever the Cargo configuration of the user says.
        .env("CARGO_TARGET_DIR", directory.join("target"))
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
    anstream::adapter::strip_str(&(stdout(output) + &stderr(output))).to_string()
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
fn check_passes_on_clean_tree() {
    let output = mise_run("check", &copy_of_repository());
    assert!(
        output.status.success(),
        "the task passes:\n{}",
        text(&output)
    );
}

#[test]
fn check_fails_on_formatting_defect() {
    let output = mise_run("check", &with_formatting_defect());
    assert_task_failed(&output, "fmt");
}

#[test]
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
fn check_fails_on_failing_test() {
    // One line, so that the attribute in this text is not an attribute line of this file.
    let defect = "//! A test that fails.\n\n#[test]\nfn fails() {\n    assert!(std::env::var_os(\"MU_PORTAL_NEVER_SET\").is_some());\n}\n";
    let copy =
        copy_of_repository().append_file(Path::new("crates/mu-core/tests/failing.rs"), defect);
    assert_task_failed(&mise_run("check", &copy), "test");
}

#[test]
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
fn check_fails_on_broken_doc_link() {
    let defect = "\n/// See [`Missing`].\npub struct Documented;\n";
    let copy = copy_of_repository().append_file(core_library(), defect);
    assert_task_failed(&mise_run("check", &copy), "doc");
}

#[test]
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
fn hooks_check_fails_on_formatting_defect() {
    let copy = with_formatting_defect();
    assert!(run("git", &["add", "--all"], copy.root()).status.success());
    let output = mise_run("hooks:check", &copy);
    let text = text(&output);
    assert!(!output.status.success(), "the hook check fails:\n{text}");
    assert!(text.contains("[fmt]"), "the format step fails:\n{text}");
}

#[test]
fn hooks_check_passes_on_clean_tree() {
    let output = mise_run("hooks:check", &copy_of_repository());
    assert!(
        output.status.success(),
        "the task passes:\n{}",
        text(&output)
    );
}

fn pr_title_task(title: &str) -> Output {
    let copy = copy_of_repository();
    command("mise", copy.root())
        .args(["run", "lint:pr-title"])
        .env("MISE_TRUSTED_CONFIG_PATHS", copy.root())
        .env("PR_TITLE", title)
        .output()
        .expect("mise runs")
}

#[test]
fn pr_title_task_accepts_conventional_title() {
    let output = pr_title_task("feat(cli): add the pair verb");
    assert!(
        output.status.success(),
        "the task passes:\n{}",
        text(&output)
    );
}

#[test]
fn pr_title_task_rejects_plain_title() {
    for title in ["Add the pair verb", "feat add the pair verb", ""] {
        let output = pr_title_task(title);
        assert!(
            !output.status.success(),
            "the task fails for {title:?}:\n{}",
            text(&output)
        );
    }
}

/// Makes an empty commit in the copy, with the hooks of the repository installed there.
fn commit_in_copy(copy: &TreeCopy, message: &str) -> Output {
    let identity = [
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
    ];
    let arguments = ["commit", "--quiet", "--allow-empty", "--message", message];
    command("git", copy.root())
        .args(identity)
        .args(arguments)
        .env("MISE_TRUSTED_CONFIG_PATHS", copy.root())
        .output()
        .expect("git runs")
}

#[test]
fn commit_hook_checks_the_message() {
    let copy = copy_of_repository();
    let installed = run("hk", &["install", "--mise"], copy.root());
    assert!(
        installed.status.success(),
        "hk installs the hooks:\n{}",
        text(&installed)
    );

    let plain = commit_in_copy(&copy, "fixed stuff");
    assert!(
        !plain.status.success(),
        "a plain message fails:\n{}",
        text(&plain)
    );

    let conventional = commit_in_copy(&copy, "fix: handle the empty verb");
    assert!(
        conventional.status.success(),
        "a conventional message passes:\n{}",
        text(&conventional)
    );
}
