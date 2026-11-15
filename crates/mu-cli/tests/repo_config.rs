//! Rules of the repository configuration: the CI workflow, the `mise` tasks,
//! the `hk` pins, the ruleset of `main` and the plan rules in `openspec/config.yaml`. Each rule is a function over the
//! text of the files, so a test can also pass it a text that breaks the rule.

use serde_json::Value as Json;
use std::collections::BTreeSet;

mod support;

use support::repo_file;

const CHECK_TASKS: [&str; 5] = ["fmt", "lint", "test", "doc", "deny"];
const AGGREGATE_JOB: &str = "ci-ok";
const AGGREGATE_ACTION: &str = "re-actors/alls-green@";
const PULL_REQUEST_JOB: &str = "commits";
const PULL_REQUEST_RULE: &str =
    r#"{"type": "pull_request", "parameters": {"allowed_merge_methods": ["squash"]}}"#;

fn yaml(text: &str) -> Json {
    serde_saphyr::from_str(text).expect("the text is YAML")
}

fn toml(text: &str) -> toml::Table {
    text.parse().expect("the text is TOML")
}

/// The jobs of a workflow, each with its id.
fn jobs(workflow: &Json) -> Vec<(&String, &Json)> {
    workflow["jobs"]
        .as_object()
        .expect("the workflow has jobs")
        .iter()
        .collect()
}

/// The names of the tasks that a `mise.toml` defines.
fn task_names(mise: &toml::Table) -> BTreeSet<String> {
    mise.get("tasks")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(toml::Table::keys)
        .cloned()
        .collect()
}

/// Each `run` step of the workflow that is not exactly `mise run <task>` with a defined task.
fn steps_that_are_not_tasks(workflow: &str, mise: &str) -> Vec<String> {
    let tasks = task_names(&toml(mise));
    let workflow = yaml(workflow);
    let mut found = Vec::new();
    for (_, job) in jobs(&workflow) {
        let steps = job["steps"].as_array().into_iter().flatten();
        for run in steps.filter_map(|step| step["run"].as_str()) {
            let task = run.strip_prefix("mise run ");
            if !task.is_some_and(|task| tasks.contains(task)) {
                found.push(run.to_owned());
            }
        }
    }
    found
}

/// The release of each tool in a `mise.toml`, and the Rust channel of a `rust-toolchain.toml`.
fn tool_releases(mise: &str, rust_toolchain: &str) -> Vec<String> {
    let mise = toml(mise);
    let tools = mise.get("tools").and_then(toml::Value::as_table);
    let mut releases: Vec<String> = tools
        .into_iter()
        .flatten()
        .filter_map(|(_, tool)| {
            // A tool is a release string or a table with a `version` key.
            tool.as_str()
                .or_else(|| tool.get("version").and_then(toml::Value::as_str))
        })
        .map(str::to_owned)
        .collect();
    assert!(!releases.is_empty(), "mise.toml holds a tool release");
    let channel = toml(rust_toolchain)
        .get("toolchain")
        .and_then(|toolchain| toolchain.get("channel"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    releases.extend(channel);
    releases
}

/// True if `text` holds `release` as a whole number, not as a part of a longer one.
fn holds_release(text: &str, release: &str) -> bool {
    let bytes = text.as_bytes();
    text.match_indices(release).any(|(start, _)| {
        let end = start + release.len();
        let digit_at = |index: usize| bytes.get(index).is_some_and(u8::is_ascii_digit);
        let before = start > 0 && (digit_at(start - 1) || bytes[start - 1] == b'.');
        let after = digit_at(end) || (bytes.get(end) == Some(&b'.') && digit_at(end + 1));
        !before && !after
    })
}

/// Each release that the workflow states, in the full form or with no patch number.
fn releases_in_workflow(workflow: &str, releases: &[String]) -> Vec<String> {
    releases
        .iter()
        .filter(|release| {
            let parts: Vec<&str> = release.split('.').collect();
            let short = (parts.len() == 3).then(|| parts[..2].join("."));
            holds_release(workflow, release)
                || short.is_some_and(|short| holds_release(workflow, &short))
        })
        .cloned()
        .collect()
}

/// The commands of a task: `run` is a string or a list, and a string can join commands.
fn task_commands(task: &toml::Value) -> Vec<String> {
    let run = task.get("run");
    let lines: Vec<&str> = match run {
        Some(toml::Value::String(line)) => vec![line.as_str()],
        Some(toml::Value::Array(lines)) => lines.iter().filter_map(toml::Value::as_str).collect(),
        _ => Vec::new(),
    };
    lines
        .into_iter()
        .flat_map(|line| line.split(['\n', ';', '|', '&']))
        .map(command_without_settings)
        .filter(|command| !command.is_empty())
        .collect()
}

/// A command with the environment settings before it removed: `A=b cargo test` is `cargo test`.
fn command_without_settings(command: &str) -> String {
    let words: Vec<&str> = command
        .split_whitespace()
        .skip_while(|word| word.contains('=') && !word.starts_with('-'))
        .collect();
    words.join(" ")
}

/// Each Cargo command in a task that has no `--locked`. `cargo fmt` resolves nothing.
fn cargo_commands_without_locked(mise: &str) -> Vec<String> {
    let mise = toml(mise);
    let tasks = mise.get("tasks").and_then(toml::Value::as_table);
    tasks
        .into_iter()
        .flatten()
        .flat_map(|(_, task)| task_commands(task))
        .filter(|command| command.starts_with("cargo ") && !command.starts_with("cargo fmt"))
        .filter(|command| !command.split_whitespace().any(|word| word == "--locked"))
        .collect()
}

/// Each of the five check tasks that the `check` task does not depend on.
fn tasks_missing_from_check(mise: &str) -> Vec<&'static str> {
    let mise = toml(mise);
    let depends: Vec<&str> = mise
        .get("tasks")
        .and_then(|tasks| tasks.get("check"))
        .and_then(|check| check.get("depends"))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .collect();
    CHECK_TASKS
        .into_iter()
        .filter(|task| !depends.contains(task))
        .collect()
}

/// The `hk` releases in the import lines of an `hk.pkl`: the download tag and the package release.
fn hk_pins(hk_pkl: &str) -> Vec<String> {
    hk_pkl
        .lines()
        .filter(|line| line.contains("package://github.com/jdx/hk/"))
        .flat_map(|line| {
            let tag = line
                .split_once("/download/v")
                .and_then(|(_, rest)| rest.split_once('/'))
                .map(|(tag, _)| tag);
            let package = line
                .split_once("/hk@")
                .and_then(|(_, rest)| rest.split_once('#'))
                .map(|(package, _)| package);
            [tag, package].map(|pin| pin.unwrap_or("").to_owned())
        })
        .collect()
}

/// True if the two import lines of `hk.pkl` hold four releases equal to the `hk` release of `mise.toml`.
fn hk_pins_match(hk_pkl: &str, mise: &str) -> bool {
    let mise = toml(mise);
    let Some(release) = mise
        .get("tools")
        .and_then(|tools| tools.get("hk"))
        .and_then(toml::Value::as_str)
    else {
        return false;
    };
    let pins = hk_pins(hk_pkl);
    pins.len() == 4 && pins.iter().all(|pin| pin == release)
}

/// The commands of one task of a `mise.toml`.
fn commands_of(mise: &str, task: &str) -> Vec<String> {
    let mise = toml(mise);
    mise.get("tasks")
        .and_then(|tasks| tasks.get(task))
        .into_iter()
        .flat_map(task_commands)
        .collect()
}

/// True if the `lint` task makes each Clippy warning an error, with every feature on.
fn lint_task_denies_warnings(mise: &str) -> bool {
    commands_of(mise, "lint").iter().any(|command| {
        let (cargo, clippy) = command.split_once(" -- ").unwrap_or((command, ""));
        cargo.starts_with("cargo clippy")
            && cargo
                .split_whitespace()
                .any(|word| word == "--all-features")
            && clippy.split_whitespace().collect::<Vec<_>>() == ["-D", "warnings"]
    })
}

/// True if the `doc` task makes each rustdoc warning an error.
fn doc_task_denies_warnings(mise: &str) -> bool {
    let mise = toml(mise);
    let flags = mise
        .get("tasks")
        .and_then(|tasks| tasks.get("doc"))
        .and_then(|doc| doc.get("env"))
        .and_then(|env| env.get("RUSTDOCFLAGS"))
        .and_then(toml::Value::as_str);
    flags.is_some_and(|flags| flags.split_whitespace().collect::<Vec<_>>() == ["-D", "warnings"])
}

/// True if the `test` task runs the doc tests too. cargo-nextest does not run them.
fn test_task_runs_doc_tests(mise: &str) -> bool {
    let commands = commands_of(mise, "test");
    let has = |start: &str, word: &str| {
        commands.iter().any(|command| {
            command.starts_with(start) && command.split_whitespace().any(|part| part == word)
        })
    };
    has("cargo nextest run", "--workspace") && has("cargo test", "--doc")
}

/// What a `deny.toml` holds against the dependency policy of the project.
fn dependency_policy_gaps(deny: &str) -> Vec<&'static str> {
    const COPYLEFT: [&str; 4] = ["GPL", "AGPL", "LGPL", "SSPL"];
    let deny = toml(deny);
    let text = |path: [&str; 2]| {
        deny.get(path[0])
            .and_then(|table| table.get(path[1]))
            .and_then(toml::Value::as_str)
    };
    let flag = |path: [&str; 2]| {
        deny.get(path[0])
            .and_then(|table| table.get(path[1]))
            .and_then(toml::Value::as_bool)
    };
    let licenses = deny.get("licenses");
    let allowed: Vec<&str> = licenses
        .and_then(|licenses| licenses.get("allow"))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .collect();
    let ignores_private = licenses
        .and_then(|licenses| licenses.get("private"))
        .and_then(|private| private.get("ignore"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);

    let mut gaps = Vec::new();
    if allowed.is_empty() {
        gaps.push("no allowed license");
    }
    if allowed
        .iter()
        .any(|license| COPYLEFT.iter().any(|prefix| license.starts_with(prefix)))
    {
        gaps.push("a copyleft license is allowed");
    }
    if ignores_private {
        gaps.push("a private crate is not checked");
    }
    if flag(["graph", "all-features"]) != Some(true) {
        gaps.push("optional dependencies are not checked");
    }
    if text(["bans", "wildcards"]) != Some("deny") {
        gaps.push("a wildcard dependency is allowed");
    }
    if text(["sources", "unknown-registry"]) != Some("deny")
        || text(["sources", "unknown-git"]) != Some("deny")
    {
        gaps.push("an unknown source is allowed");
    }
    gaps
}

/// The text of one hook block of an `hk.pkl`, from its name to the next hook or the end.
fn hook_block<'a>(hk_pkl: &'a str, hook: &str) -> &'a str {
    let Some(start) = hk_pkl.find(&format!("[\"{hook}\"]")) else {
        return "";
    };
    let rest = &hk_pkl[start..];
    let end = rest[1..]
        .find("\n    [\"")
        .map_or(rest.len(), |offset| offset + 1);
    &rest[..end]
}

/// What an `hk.pkl` lacks: a step that calls a task, or a hook.
fn hook_gaps(hk_pkl: &str) -> Vec<&'static str> {
    let steps = hk_pkl.split("\nhooks {").next().unwrap_or(hk_pkl);
    let mut gaps = Vec::new();
    if !steps.contains("check = \"mise run fmt\"") {
        gaps.push("no format step that calls the fmt task");
    }
    if !steps.contains("check = \"mise run lint:actions\"") {
        gaps.push("no workflow step that calls the lint:actions task");
    }
    if !hook_block(hk_pkl, "commit-msg").contains("Builtins.check_conventional_commit") {
        gaps.push("no commit-msg hook with the conventional commit check");
    }
    if !hook_block(hk_pkl, "pre-push").contains("check = \"mise run check\"") {
        gaps.push("no pre-push hook that calls the check task");
    }
    gaps
}

/// True if the `lint:pr-title` task gives the title to the check in a file. A title that is
/// an argument can be read as an option.
fn pr_title_task_reads_a_file(mise: &str) -> bool {
    let commands = commands_of(mise, "lint:pr-title");
    let writes_file = commands
        .iter()
        .any(|command| command.starts_with("printf") && command.contains("\"$PR_TITLE\" >"));
    let checks_file = commands.iter().any(|command| {
        command.starts_with("hk util check-conventional-commit") && !command.contains("PR_TITLE")
    });
    writes_file && checks_file
}

/// What keeps CI from checking the title of each pull request, after an edit too.
fn pr_title_gaps(workflow: &str) -> Vec<&'static str> {
    let workflow = yaml(workflow);
    let mut gaps = Vec::new();
    let edited = workflow["on"]["pull_request"]["types"]
        .as_array()
        .is_some_and(|types| types.iter().any(|event| event == "edited"));
    if !edited {
        gaps.push("the workflow does not run for an edited title");
    }
    let title_step = jobs(&workflow).into_iter().any(|(_, job)| {
        job["steps"].as_array().into_iter().flatten().any(|step| {
            step["run"] == "mise run lint:pr-title"
                && step["env"]["PR_TITLE"] == "${{ github.event.pull_request.title }}"
        })
    });
    if !title_step {
        gaps.push("no step gives the title to the lint:pr-title task");
    }
    gaps
}

/// The name that GitHub reports for a job: its `name`, or its id.
fn reported_name(workflow: &Json, id: &str) -> Option<String> {
    let job = workflow["jobs"].get(id)?;
    Some(job["name"].as_str().unwrap_or(id).to_owned())
}

/// True if the ruleset requires exactly one check, the reported name of the aggregate job.
fn ruleset_requires_aggregate(ruleset: &str, workflow: &str) -> bool {
    let ruleset: Json = serde_json::from_str(ruleset).expect("the ruleset is JSON");
    let contexts: Vec<&str> = ruleset["rules"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|rule| rule["type"] == "required_status_checks")
        .filter_map(|rule| rule["parameters"]["required_status_checks"].as_array())
        .flatten()
        .filter_map(|check| check["context"].as_str())
        .collect();
    // The title check guards the commit on main only if a squash is the one merge method.
    let squash_only = ruleset["rules"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|rule| rule["type"] == "pull_request")
        .any(|rule| rule["parameters"]["allowed_merge_methods"] == serde_json::json!(["squash"]));
    let name = reported_name(&yaml(workflow), AGGREGATE_JOB);
    squash_only && contexts.len() == 1 && name.as_deref() == Some(contexts[0])
}

/// What keeps the aggregate job from covering every job: a job it does not need, or no `always()`.
fn aggregate_gaps(workflow: &str) -> Vec<String> {
    let workflow = yaml(workflow);
    let Some(aggregate) = workflow["jobs"].get(AGGREGATE_JOB) else {
        return vec![format!("no {AGGREGATE_JOB} job")];
    };
    let needs: Vec<&str> = match &aggregate["needs"] {
        Json::String(need) => vec![need.as_str()],
        Json::Array(needs) => needs.iter().filter_map(Json::as_str).collect(),
        _ => Vec::new(),
    };
    let mut gaps: Vec<String> = jobs(&workflow)
        .into_iter()
        .map(|(id, _)| id.as_str())
        .filter(|id| *id != AGGREGATE_JOB && !needs.contains(id))
        .map(|id| format!("{AGGREGATE_JOB} does not need {id}"))
        .collect();
    let passes_results = aggregate["steps"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|step| {
            step["uses"]
                .as_str()
                .is_some_and(|action| action.starts_with(AGGREGATE_ACTION))
                && step["with"]["jobs"].as_str() == Some("${{ toJSON(needs) }}")
        });
    if !passes_results {
        gaps.push(format!(
            "{AGGREGATE_JOB} does not give the job results to {AGGREGATE_ACTION}"
        ));
    }
    // Only the job that runs for a pull request alone can be skipped, and no job can fail.
    let lenient = aggregate["steps"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|step| {
            let with = &step["with"];
            !with["allowed-failures"].is_null()
                || with["allowed-skips"]
                    .as_str()
                    .is_some_and(|skips| skips != PULL_REQUEST_JOB)
        });
    if lenient {
        gaps.push(format!("{AGGREGATE_JOB} allows a failed or a skipped job"));
    }
    if aggregate["if"].as_str() != Some("always()") {
        gaps.push(format!("{AGGREGATE_JOB} has no always() condition"));
    }
    gaps
}

const MISE: &str = r#"
[tools]
hk = "2.12.0"
node = { version = "26.13.1" }

[tasks.check]
depends = ["fmt", "lint", "test", "doc", "deny"]

[tasks.test]
run = "cargo nextest run --locked"
"#;

fn workflow_with_step(run: &str) -> String {
    format!("jobs:\n  rust:\n    steps:\n      - run: {run}\n")
}

#[test]
fn ci_steps_are_tasks() {
    let none: Vec<String> = Vec::new();
    assert_eq!(
        steps_that_are_not_tasks(
            &repo_file(".github/workflows/ci.yml"),
            &repo_file("mise.toml")
        ),
        none
    );
    assert_eq!(
        steps_that_are_not_tasks(&workflow_with_step("mise run check"), MISE),
        none
    );

    for run in [
        "mise run check && cargo test",
        "mise run check;cargo test",
        "cargo test --workspace",
        "mise run absent",
        "mise exec -- hk check --all",
    ] {
        assert_eq!(
            steps_that_are_not_tasks(&workflow_with_step(&format!("{run:?}")), MISE),
            [run],
            "one command: {run}"
        );
    }
    let two_lines = "jobs:\n  rust:\n    steps:\n      - run: |\n          mise run check\n          cargo test\n";
    assert_eq!(steps_that_are_not_tasks(two_lines, MISE).len(), 1);
}

#[test]
fn ci_states_no_tool_release() {
    let none: Vec<String> = Vec::new();
    let releases = tool_releases(&repo_file("mise.toml"), &repo_file("rust-toolchain.toml"));
    assert_eq!(
        releases_in_workflow(&repo_file(".github/workflows/ci.yml"), &releases),
        none
    );

    // The string form and the table form of a tool are both found, and the Rust channel too.
    let toolchain = "[toolchain]\nchannel = \"1.90.0\"\n";
    let found = tool_releases(MISE, toolchain);
    assert_eq!(found, ["2.12.0", "26.13.1", "1.90.0"]);

    assert_eq!(
        releases_in_workflow("node-version: 26.13.1", &found),
        ["26.13.1"]
    );
    assert_eq!(
        releases_in_workflow("node-version: 26.13", &found),
        ["26.13.1"]
    );
    assert_eq!(releases_in_workflow("toolchain: 1.90", &found), ["1.90.0"]);
    // A release inside a longer number is another number.
    assert_eq!(releases_in_workflow("uses: x@abc # v12.12.0", &found), none);
    assert_eq!(releases_in_workflow("uses: x@abc # v2.12.1", &found), none);
    assert_eq!(
        releases_in_workflow("uses: x@abc # v3.2.12.0", &found),
        none
    );
    assert_eq!(
        releases_in_workflow("uses: x@abc # v2.12.0.1", &found),
        none
    );
}

#[test]
#[should_panic(expected = "mise.toml holds a tool release")]
fn ci_states_no_tool_release_needs_a_release() {
    // The Rust channel alone does not count: a mise.toml in which the rule finds no tool is a failure.
    let _ = tool_releases(
        "[tasks.fmt]\nrun = \"cargo fmt\"\n",
        "[toolchain]\nchannel = \"1.90.0\"\n",
    );
}

#[test]
fn cargo_tasks_use_locked() {
    let none: Vec<String> = Vec::new();
    assert_eq!(cargo_commands_without_locked(&repo_file("mise.toml")), none);
    assert_eq!(
        cargo_commands_without_locked("[tasks.fmt]\nrun = \"cargo fmt --check\"\n"),
        none
    );

    let second = ["cargo test --workspace --doc"];
    let one_string =
        "[tasks.test]\nrun = \"cargo nextest run --locked && cargo test --workspace --doc\"\n";
    assert_eq!(cargo_commands_without_locked(one_string), second);
    let list =
        "[tasks.test]\nrun = [\"cargo nextest run --locked\", \"cargo test --workspace --doc\"]\n";
    assert_eq!(cargo_commands_without_locked(list), second);
    // A command after `||` or `|`, and a command after an environment setting, are commands too.
    for joined in [
        "cargo nextest run --locked || cargo test --workspace --doc",
        "cargo nextest run --locked | tee log; RUSTFLAGS=-Dwarnings cargo test --workspace --doc",
    ] {
        let task = format!("[tasks.test]\nrun = \"{joined}\"\n");
        assert_eq!(cargo_commands_without_locked(&task), second, "{joined}");
    }
}

#[test]
fn check_runs_every_task() {
    let none: Vec<&str> = Vec::new();
    assert_eq!(tasks_missing_from_check(&repo_file("mise.toml")), none);

    let four = "[tasks.check]\ndepends = [\"fmt\", \"lint\", \"test\", \"doc\"]\n";
    assert_eq!(tasks_missing_from_check(four), ["deny"]);
    assert_eq!(
        tasks_missing_from_check("[tasks.check]\nrun = \"true\"\n"),
        CHECK_TASKS
    );
}

#[test]
fn hk_pins_match_mise() {
    assert!(hk_pins_match(&repo_file("hk.pkl"), &repo_file("mise.toml")));

    let line = |keyword: &str, tag: &str, package: &str, file: &str| {
        format!(
            "{keyword} \"package://github.com/jdx/hk/releases/download/v{tag}/hk@{package}#/{file}.pkl\"\n"
        )
    };
    let pkl = |tag: &str, package: &str| {
        line("amends", tag, package, "Config") + &line("import", tag, package, "Builtins")
    };
    assert!(hk_pins_match(&pkl("2.12.0", "2.12.0"), MISE));
    // The tag and the package release differ inside one line.
    assert!(!hk_pins_match(&pkl("2.12.0", "2.11.0"), MISE));
    // The four releases agree, but not with mise.toml.
    assert!(!hk_pins_match(&pkl("2.13.0", "2.13.0"), MISE));
    // One import line is absent.
    assert!(!hk_pins_match(
        &line("amends", "2.12.0", "2.12.0", "Config"),
        MISE
    ));
}

#[test]
fn check_tasks_fail_on_warnings() {
    let mise = repo_file("mise.toml");
    assert!(lint_task_denies_warnings(&mise));
    assert!(doc_task_denies_warnings(&mise));

    let lint = |run: &str| format!("[tasks.lint]\nrun = \"{run}\"\n");
    assert!(lint_task_denies_warnings(&lint(
        "cargo clippy --all-features --locked -- -D warnings"
    )));
    assert!(!lint_task_denies_warnings(&lint(
        "cargo clippy --all-features --locked"
    )));
    assert!(!lint_task_denies_warnings(&lint(
        "cargo clippy --locked -- -D warnings"
    )));
    assert!(!lint_task_denies_warnings(&lint(
        "cargo clippy --all-features -- -W warnings"
    )));

    let doc = |env: &str| format!("[tasks.doc]\n{env}run = \"cargo doc --locked\"\n");
    assert!(doc_task_denies_warnings(&doc(
        "env = { RUSTDOCFLAGS = \"-D warnings\" }\n"
    )));
    assert!(!doc_task_denies_warnings(&doc("")));
    assert!(!doc_task_denies_warnings(&doc(
        "env = { RUSTDOCFLAGS = \"-W warnings\" }\n"
    )));
}

#[test]
fn test_task_runs_the_doc_tests() {
    assert!(test_task_runs_doc_tests(&repo_file("mise.toml")));

    let test = |run: &str| format!("[tasks.test]\nrun = {run}\n");
    assert!(test_task_runs_doc_tests(&test(
        "[\"cargo nextest run --workspace --locked\", \"cargo test --workspace --doc --locked\"]"
    )));
    assert!(!test_task_runs_doc_tests(&test(
        "\"cargo nextest run --workspace --locked\""
    )));
    assert!(!test_task_runs_doc_tests(&test(
        "\"cargo test --workspace --doc --locked\""
    )));
}

#[test]
fn dependency_policy_holds() {
    let none: Vec<&str> = Vec::new();
    assert_eq!(dependency_policy_gaps(&repo_file("deny.toml")), none);

    let policy = |licenses: &str, rest: &str| {
        format!(
            "[graph]\nall-features = true\n[licenses]\nallow = [{licenses}]\n[bans]\nwildcards = \"deny\"\n[sources]\nunknown-registry = \"deny\"\nunknown-git = \"deny\"\n{rest}"
        )
    };
    assert_eq!(dependency_policy_gaps(&policy("\"MIT\"", "")), none);
    assert_eq!(
        dependency_policy_gaps(&policy("\"MIT\", \"GPL-3.0-only\"", "")),
        ["a copyleft license is allowed"]
    );
    assert_eq!(
        dependency_policy_gaps(&policy("\"LGPL-2.1-or-later\"", "")),
        ["a copyleft license is allowed"]
    );
    assert_eq!(
        dependency_policy_gaps(&policy("", "")),
        ["no allowed license"]
    );
    assert_eq!(
        dependency_policy_gaps(&policy("\"MIT\"", "[licenses.private]\nignore = true\n")),
        ["a private crate is not checked"]
    );
    assert_eq!(
        dependency_policy_gaps("[licenses]\nallow = [\"MIT\"]\n"),
        [
            "optional dependencies are not checked",
            "a wildcard dependency is allowed",
            "an unknown source is allowed"
        ]
    );
}

#[test]
fn hooks_call_the_tasks() {
    let none: Vec<&str> = Vec::new();
    assert_eq!(hook_gaps(&repo_file("hk.pkl")), none);

    let steps = "steps {\n    [\"fmt\"] {\n        check = \"mise run fmt\"\n    }\n    [\"actions\"] {\n        check = \"mise run lint:actions\"\n    }\n}\n";
    let commit = "    [\"commit-msg\"] {\n        steps {\n            [\"conventional\"] = Builtins.check_conventional_commit\n        }\n    }\n";
    let push = "    [\"pre-push\"] {\n        steps {\n            [\"check\"] {\n                check = \"mise run check\"\n            }\n        }\n    }\n";
    let pkl = |steps: &str, hooks: &str| format!("{steps}\nhooks {{\n{hooks}}}\n");
    assert_eq!(hook_gaps(&pkl(steps, &format!("{commit}{push}"))), none);
    assert_eq!(
        hook_gaps(&pkl(steps, push)),
        ["no commit-msg hook with the conventional commit check"]
    );
    assert_eq!(
        hook_gaps(&pkl(steps, commit)),
        ["no pre-push hook that calls the check task"]
    );
    // A raw command in a step is not a call of the task.
    let raw = steps.replace("mise run fmt", "cargo fmt --check");
    assert_eq!(
        hook_gaps(&pkl(&raw, &format!("{commit}{push}"))),
        ["no format step that calls the fmt task"]
    );
    // The check task in the wrong hook does not count for the pre-push hook.
    let wrong_hook = commit.replace(
        "Builtins.check_conventional_commit",
        "check = \"mise run check\"",
    );
    assert_eq!(hook_gaps(&pkl(steps, &wrong_hook)).len(), 2);
}

#[test]
fn pr_title_reaches_the_check_as_a_file() {
    assert!(pr_title_task_reads_a_file(&repo_file("mise.toml")));

    let task = |run: &str| format!("[tasks.\"lint:pr-title\"]\nrun = \'\'\'\n{run}\n\'\'\'\n");
    assert!(pr_title_task_reads_a_file(&task(
        "printf '%s\\n' \"$PR_TITLE\" > \"$title_file\"\nhk util check-conventional-commit \"$title_file\""
    )));
    // The title as an argument can be read as an option, for example `--help`.
    assert!(!pr_title_task_reads_a_file(&task(
        "hk util check-conventional-commit \"$PR_TITLE\""
    )));
    assert!(!pr_title_task_reads_a_file(&task(
        "cog verify \"$PR_TITLE\""
    )));
}

#[test]
fn ci_checks_the_pr_title() {
    let none: Vec<&str> = Vec::new();
    assert_eq!(pr_title_gaps(&repo_file(".github/workflows/ci.yml")), none);

    let workflow = |types: &str, env: &str| {
        format!(
            "on:\n  pull_request:\n{types}jobs:\n  commits:\n    steps:\n      - run: mise run lint:pr-title\n{env}"
        )
    };
    let types = "    types: [opened, edited, synchronize, reopened]\n";
    let env = "        env:\n          PR_TITLE: ${{ github.event.pull_request.title }}\n";
    assert_eq!(pr_title_gaps(&workflow(types, env)), none);
    assert_eq!(
        pr_title_gaps(&workflow("", env)),
        ["the workflow does not run for an edited title"]
    );
    assert_eq!(
        pr_title_gaps(&workflow(types, "")),
        ["no step gives the title to the lint:pr-title task"]
    );
}

#[test]
fn ruleset_requires_ci_ok() {
    assert!(ruleset_requires_aggregate(
        &repo_file(".github/rulesets/main.json"),
        &repo_file(".github/workflows/ci.yml")
    ));

    let ruleset = |contexts: &[&str]| {
        let checks: Vec<String> = contexts
            .iter()
            .map(|context| format!("{{\"context\": \"{context}\"}}"))
            .collect();
        format!(
            "{{\"rules\": [{PULL_REQUEST_RULE}, {{\"type\": \"required_status_checks\", \"parameters\": {{\"required_status_checks\": [{}]}}}}]}}",
            checks.join(", ")
        )
    };
    let workflow = "jobs:\n  ci-ok:\n    steps: []\n";
    assert!(ruleset_requires_aggregate(&ruleset(&["ci-ok"]), workflow));
    // GitHub reports the name of a job, not its id.
    let renamed = "jobs:\n  ci-ok:\n    name: all green\n    steps: []\n";
    assert!(!ruleset_requires_aggregate(&ruleset(&["ci-ok"]), renamed));
    assert!(ruleset_requires_aggregate(
        &ruleset(&["all green"]),
        renamed
    ));
    assert!(!ruleset_requires_aggregate(&ruleset(&[]), workflow));
    assert!(!ruleset_requires_aggregate(
        &ruleset(&["ci-ok", "docs"]),
        workflow
    ));
    assert!(!ruleset_requires_aggregate("{\"rules\": []}", workflow));
    // A merge commit or a rebase puts the commits of the branch on main, not the title.
    let with_merge = ruleset(&["ci-ok"]).replace("[\"squash\"]", "[\"squash\", \"merge\"]");
    assert!(!ruleset_requires_aggregate(&with_merge, workflow));
}

#[test]
fn ci_ok_covers_every_job() {
    let none: Vec<String> = Vec::new();
    assert_eq!(aggregate_gaps(&repo_file(".github/workflows/ci.yml")), none);

    let step = "    steps:\n      - uses: re-actors/alls-green@0000000\n        with:\n          jobs: ${{ toJSON(needs) }}\n";
    let workflow = |needs: &str, condition: &str, steps: &str| {
        format!(
            "jobs:\n  rust: {{}}\n  docs: {{}}\n  ci-ok:\n    needs: {needs}\n{condition}{steps}"
        )
    };
    let always = "    if: always()\n";
    assert_eq!(
        aggregate_gaps(&workflow("[rust, docs]", always, step)),
        none
    );
    assert_eq!(
        aggregate_gaps(&workflow("[rust]", always, step)),
        ["ci-ok does not need docs"]
    );
    assert_eq!(
        aggregate_gaps(&workflow("[rust, docs]", "", step)),
        ["ci-ok has no always() condition"]
    );
    // A step that does not read the job results cannot fail for a failed job.
    assert_eq!(
        aggregate_gaps(&workflow(
            "[rust, docs]",
            always,
            "    steps:\n      - run: mise run check\n"
        )),
        ["ci-ok does not give the job results to re-actors/alls-green@"]
    );
    for allowed in [
        "          allowed-failures: docs\n",
        "          allowed-skips: rust\n",
    ] {
        let lenient = format!("{step}{allowed}");
        assert_eq!(
            aggregate_gaps(&workflow("[rust, docs]", always, &lenient)),
            ["ci-ok allows a failed or a skipped job"],
            "{allowed}"
        );
    }
    let skips_commits = format!("{step}          allowed-skips: commits\n");
    assert_eq!(
        aggregate_gaps(&workflow("[rust, docs]", always, &skips_commits)),
        none
    );
    assert_eq!(aggregate_gaps("jobs:\n  rust: {}\n"), ["no ci-ok job"]);
}

/// True if one rule of `rules.design` names both a change that builds on a spike and the part
/// "Using a spike" of the spikes page.
fn design_rules_name_spikes_page(config: &str) -> bool {
    yaml(config)["rules"]["design"]
        .as_array()
        .is_some_and(|rules| {
            rules
                .iter()
                .filter_map(Json::as_str)
                .any(|rule| rule.contains("builds on a spike") && rule.contains("Using a spike"))
        })
}

/// True if the page has the level-two heading `Using a spike` that the design rule names.
fn has_using_a_spike(page: &str) -> bool {
    page.lines()
        .any(|line| line.trim_end() == "## Using a spike")
}

#[test]
fn design_rules_cover_spikes() {
    assert!(design_rules_name_spikes_page(&repo_file(
        "openspec/config.yaml"
    )));
    assert!(has_using_a_spike(&repo_file(
        "docs/src/content/docs/contributing/spikes.md"
    )));
    assert!(
        !has_using_a_spike("## Using spikes\n"),
        "another heading breaks the rule"
    );
    assert!(
        !has_using_a_spike("Using a spike\n"),
        "plain text breaks the rule"
    );

    let config = |rules: &[&str]| {
        let list: String = rules
            .iter()
            .map(|rule| ["    - ", rule, "\n"].concat())
            .collect();
        format!("rules:\n  design:\n{list}")
    };
    assert!(design_rules_name_spikes_page(&config(&[
        "If the change builds on a spike, follow Using a spike"
    ])));
    for (case, rules) in [
        (
            "two rules",
            vec![
                "If the change builds on a spike, read it",
                "Follow Using a spike",
            ],
        ),
        (
            "no Using a spike",
            vec!["If the change builds on a spike, read its page"],
        ),
        ("no builds on a spike", vec!["Follow Using a spike"]),
    ] {
        assert!(
            !design_rules_name_spikes_page(&config(&rules)),
            "{case} breaks the rule"
        );
    }
}
