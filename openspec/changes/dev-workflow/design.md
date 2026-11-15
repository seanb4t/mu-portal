# Design

## Context

See proposal.md for the motivation. The state of the repository after `project-foundation`:

- The check commands exist twice: in `.github/workflows/ci.yml` and on the Contributing page for the build. The implementation review of `project-foundation` reported that copy.
- Three files pin tools: `rust-toolchain.toml`, `docs/.node-version` and the `packageManager` field of `docs/package.json`.
- The clone of the owner has `core.hooksPath` set to `.husky`. That directory holds only the `husky` helper, and no file in the repository configures a hook.
- The `main` branch had no classic branch protection at the time of the plan. The owner had added a ruleset named `protect-main` by hand, which the plan did not see, because the plan read only the classic protection. That ruleset requires a pull request and a squash merge. It requires no check.
- `mise`, `hk`, `cargo-deny` and `cargo-nextest` are on the machine of the owner. The repository does not pin them.
- Every test is a Rust integration test in `crates/mu-cli/tests/`. The boundary test there has a helper that copies a tree into a temporary directory and breaks it in one place.

## Goals / Non-Goals

**Goals:**

- One file that lists the tools and one file that lists the check commands, read by a contributor and by CI.
- A check that fails before a push for the defects that CI rejects.
- A dependency policy that a tool enforces from the first third-party crate.
- A `main` branch that holds only commits that passed the checks.
- Rust code that follows the Rust standard of the project, with lints that enforce it.
- A workspace that states its minimum Rust release and that nobody can publish by accident.
- The files that GitHub shows to a new contributor.

**Non-Goals:**

- Release tooling, a changelog and version policy. A parked issue tracks them.
- The name of the binary. A parked issue tracks that decision.
- Test coverage reports and benchmarks.
- Any change to the behavior of `mu`.

## Decisions

### 1. `mise.toml` holds the tools and the tasks

A root `mise.toml` has two parts:

- `[tools]` pins `hk`, `cargo-deny`, `cargo-nextest`, `actionlint`, Node and `pnpm`. It also pins each tool that an `hk` step needs, which task 2.1 finds.
- `[tasks]` defines the check commands. No other file states them.

The tasks are:

| Task | Runs |
|---|---|
| `fmt` | `cargo fmt --check` |
| `lint` | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` |
| `lint:actions` | `actionlint` |
| `lint:pr-title` | the conventional commit check of `hk` on the title in the `PR_TITLE` variable |
| `test` | `cargo nextest run --workspace --locked`, and then `cargo test --workspace --doc --locked` |
| `doc` | `cargo doc --workspace --no-deps --locked`, with `RUSTDOCFLAGS=-D warnings` |
| `deny` | `cargo deny --locked check` |
| `check` | the tasks `fmt`, `lint`, `test`, `doc` and `deny` |
| `hooks:check` | `hk check --all` |
| `docs:check` | in `docs/`: `pnpm install --frozen-lockfile`, `pnpm model:check`, `pnpm build` |

`mise` runs the tasks that `check` depends on in parallel. The design promises no order. It promises that a failure of one task makes `check` fail.

The table in this file is a working copy for the review. After the archive, `mise.toml` is the only live list. The Contributing page names the tasks and does not repeat the commands.

Enforcement: three configuration tests from Decision 7. `ci_steps_are_tasks` fails for a CI step that is not a task. `cargo_tasks_use_locked` fails for a Cargo command in a task that has no `--locked`, with `cargo fmt` as the one exception. `check_runs_every_task` fails if `check` does not depend on each of the five tasks.

Task 1.1 read the `mise` documentation:

- `mise` reads `rust-toolchain.toml` only with the setting `idiomatic_version_file_enable_tools = ["rust"]`. `mise.toml` sets it, so `mise.toml` lists no Rust release. Page: `docs/lang/rust.md`.
- A task lists other tasks in `depends`. `mise` runs them in parallel, and their order in the list sets no sequence. Pages: `docs/tasks/index.md` and `docs/tasks/task-configuration.md`.
- A task runs in the directory of `mise.toml`. The `dir` key changes it, and `docs:check` uses `dir = "docs"`. Page: `docs/tasks/task-configuration.md`.

`cargo-nextest` is not in the `mise` registry, so `mise.toml` names it with the `cargo:` backend.

Alternative: `just` or `make`. Rejected, because neither installs tools, and the owner already uses `mise` on every machine.

Alternative: `cargo xtask`. Rejected, because the docs site checks are not Rust, and a task runner in Rust must build before the first check runs.

### 2. Tool pins

`mise.toml` replaces two pin files. The Node pin moves from `docs/.node-version` to `mise.toml`, and that file is removed. The `pnpm` pin moves from the `packageManager` field to `mise.toml`, and that field is removed. This replaces part of Decision 4 of `project-foundation`.

`rust-toolchain.toml` stays the Rust pin, because `rustup` reads it. Task 1.1 reads the `mise` documentation for the way that `mise` reads that file. If `mise` reads it natively, `mise.toml` lists Rust with no release of its own. If it does not, `mise.toml` does not list Rust, and `rustup` installs the pinned release on the first Cargo command.

Each tool release therefore exists in one file. Enforcement: the test `ci_states_no_tool_release` fails for a release in the CI workflow. Nothing checks a release that a person adds to another file, so that part is a review rule.

Renovate must read each pin. Task 6.4 runs the Renovate validator. The check of the live dependency list needs the merge, so it is a workflow follow-up and not a tracked task.

### 3. `hk` hooks

`hk.pkl` defines the hook steps.

| Hook | Steps |
|---|---|
| `pre-commit` and `check` | `mise run fmt`, `mise run lint:actions`, the `hk` builtins for TOML and merge conflict markers, and a YAML step that parses each file with `yq` |
| `commit-msg` | the `hk` builtin that checks the commit message |
| `pre-push` | `mise run check` |

`hk.pkl` states the first list once, as its top-level steps. `hk` builds the `pre-commit` hook and the `check` hook from that one list, so the two cannot differ. CI runs the `check` hook on every file through the task `hooks:check`.

A step that has a `mise` task calls the task. The format command therefore exists once, in `mise.toml`. An `hk` builtin is used only for a check that has no task. No test reads `hk.pkl` for a raw check command, so this part is a review rule.

The hook before a commit stays fast, because a slow hook is a hook that people skip. The full task runs before a push.

`hk.pkl` pins the `hk` package in its two import lines. Those two lines and the `hk` release in `mise.toml` must agree. A probe showed that `hk validate` accepts a different pin, so `hk` itself is no guard. Enforcement: the test `hk_pins_match_mise` reads both files and fails for a difference. Each import line states the release twice, in the download tag and after `hk@`, and the test compares all four with the `mise.toml` release.

Renovate updates `mise.toml` and `hk.pkl` through two managers. Task 6.4 adds a Renovate group for `hk`, so one pull request moves all of them and the test stays green.

Task 2.1 read the `hk` documentation and the output of `hk builtins`:

- The builtins are `taplo` for TOML and `check_merge_conflict` for conflict markers. `taplo` needs the `taplo` tool, which `mise.toml` now pins. `check_merge_conflict` needs no tool. Page: `docs/builtins.md`.
- The `yq` builtin is a formatter, and its check fails for a file that it would rewrite. The YAML step is therefore a custom step that only parses each file with `yq`. `mise.toml` pins `yq`.
- A custom step has a `glob` and a `check` command. The command can be `mise run <task>`. Page: `docs/configuration.md`.
- The two import lines of `hk.pkl` hold the release in the download tag and after `hk@`. `hk` needs no separate `pkl` program. Page: `docs/pirate/pkl_introduction.md`.

`hk install --mise` installs the hooks through the git configuration, not through files in `.git/hooks`.

A contributor installs the hooks once with `hk install --mise`. The hooks are opt-in for each clone, as every git hook is. CI is the gate that nobody can skip.

The `.husky` directory in the clone of the owner is not tracked. Task 2.3 removed it and unset `core.hooksPath`. That clone also held hook files from Husky 4 in `.git/hooks`, written in 2022. They became active again without the hook path, so task 2.3 moved them to `.git/hooks.husky-v4.bak`.

Alternative: `lefthook` or `pre-commit`. Both can call `mise` tasks too. `hk` is the choice of the owner, and it comes from the author of `mise`, so the two tools change together.

### 4. Dependency policy with `cargo-deny`

`deny.toml` has five parts:

- **advisories**: fail for a crate with a known vulnerability, and for an unmaintained crate.
- **licenses**: an allowed list. The first list is `Apache-2.0`, `MIT`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Unicode-3.0` and `Zlib`. A crate with a GPL license fails, which the license decision of the project requires. The file does not turn on the option that ignores a private crate. A probe showed that this option lets a GPL crate with `publish = false` pass.
- **bans**: warn for two releases of one crate. Fail for a wildcard dependency. The file turns on the option that allows a path dependency with no version. A probe showed that the ban otherwise rejects a dependency between two workspace crates, which the boundary test allows.
- **graph**: read the dependency tree with all features on, so a crate behind an optional feature is in the policy.
- **sources**: allow crates.io only. A git source needs an entry in this file.

`cargo-deny` and the boundary test both read the license of a workspace crate. The overlap is deliberate, and the two rules differ. The boundary test requires exactly `Apache-2.0` for a workspace crate. `deny.toml` holds the wider list for every crate in the dependency tree. A workspace crate therefore passes both only with `Apache-2.0`.

The allowed list is a project-wide decision. Task 4.5 adds a Decisions page for the dependency policy.

Enforcement: the `deny` task, which `check` runs and CI runs. The test `dependency_policy_holds` makes sure that `deny.toml` states the policy. The test `check_runs_every_task` makes sure that `check` runs `deny`.

Task 3.1 reads the `cargo-deny` documentation for the current configuration schema and for the two option names above.

### 5. Build hygiene in the workspace

- `[workspace.package]` gains `rust-version`. It is the release in `rust-toolchain.toml`, because no user needs an older one yet.
- `[workspace.package]` gains `publish = false`. A later change that publishes a crate changes this for that crate.
- Each crate inherits both fields with `workspace = true`.

Enforcement of the inheritance: the boundary test already has one rule of this shape, for the lints table. That rule becomes one function over a list of fields: `lints`, `rust-version` and `publish`. It reads the manifest of the member, because the `cargo_metadata` output cannot tell an inherited value from a local one. A member without `<field>.workspace = true` is one violation that names the member and the field. The three fields share one violation variant, which replaces the variant for lints. Each new field has two negative tests: the field is absent, and the field has a local value. The base fixture gains both fields in its root and in each member.

Enforcement of the value: the boundary test gains one more rule, next to the license rule, which has the same shape. A workspace whose `[workspace.package]` does not hold `publish = false` is a violation. A list of registry names is a violation too. The test is `publishable_workspace_fails`.

The parked change for release tooling publishes a crate. That change edits these two rules, because a local `publish` value is a violation here.

`rust-version` is a minimum, and it does not have to equal the release in `rust-toolchain.toml`. No test compares the two. Cargo itself refuses a toolchain that is older than `rust-version`, so a toolchain pin below the minimum fails every Cargo task. A Renovate update of the toolchain therefore needs no edit of `rust-version`.

The current entry `unsafe_code = "forbid"` stays.

`[workspace.lints]` follows the lint chapter of the `rust-best-practices` skill, with its levels and its priority numbers:

- `future-incompatible` at the level warn, and `nonstandard_style` at the level deny
- the Clippy group `all` at the level deny with priority 10, which replaces the current entry
- `redundant_clone` at deny with priority 9, and `manual_while_let_some` at deny with priority 4
- the Clippy group `pedantic` at the level warn with priority 3

Three entries are not in the skill's example:

- `missing_docs` at the level warn, so each public item has a doc comment. The skill asks for it in its documentation chapter.
- `clippy::allow_attributes` and `clippy::allow_attributes_without_reason` at the level deny. The skill asks for `#[expect]` with a reason and not `#[allow]`, and these two lints enforce that.

The `lint` task turns each warning into an error, so these levels fail the build. A probe of the current code found five warnings from these entries: one missing `#[must_use]` and four missing doc comments. Task 1.3 fixes them.

A higher priority overrides a lower one. A later lint that must be weaker than the `all` group therefore needs a priority above 10.

One detail differs from the skill's example. The two rustc groups, `future_incompatible` and `nonstandard_style`, have the priority -1. Clippy refuses a lint group with the same priority as a single lint, and `unsafe_code` and `missing_docs` are single lints at the default priority.

### 6. CI

`.github/workflows/ci.yml` has five jobs:

- **rust**, on Linux and macOS: `mise run check`.
- **docs**, on Linux: `mise run docs:check`.
- **hooks**, on Linux: `mise run hooks:check`.
- **commits**, on Linux, for a pull request only: `mise run lint:pr-title`. The title reaches the task through an environment variable of the step, so the title is never part of a shell command.
- **ci-ok**, on Linux: it needs the four jobs above and has the condition `always()`. Its one step is an existing action that fails unless each needed job succeeded. Task 5.1 reads the documentation of that action. Without the condition, GitHub skips the job after a failed job, and a skipped required check counts as a pass.

The `commits` job does not run for a push to `main`. The aggregate action therefore allows that one job to be skipped.

Every `run` step is therefore `mise run` and one task. The `ci-ok` step is a `uses` step, so the step rule does not apply to it.

Each job except `ci-ok` installs the tools with the `mise` action. The Rust job also uses a build cache action. Each action is pinned by commit SHA, as `project-foundation` decided. Task 5.1 reads the documentation of both actions for their inputs.

The `mise` action replaces the Node setup action and the `pnpm` setup action.

`ci-ok` gives the ruleset of Decision 10 one check name. That name stays the same after a job gains or loses a matrix entry. The job states `name: ci-ok`, so the name that GitHub reports equals the job id.

### 7. Tests of the configuration

The project tests what it owns: its code, its configuration, its assumptions and its requirements. It does not test a tool that it does not own. A test here therefore reads a configuration file of the repository and makes sure that the file states what the project requires. No test proves that `rustfmt` finds bad formatting, that Clippy warns, or that `cargo-deny` rejects a license. Those are properties of the tools.

The tests are Rust integration tests in `crates/mu-cli/tests/repo_config.rs`. They are fast and run in the `test` task. Each rule is a function over the text of one or two files. Each test also passes its rule a text that breaks the rule.

| Scenario | Test |
|---|---|
| The check task runs each check | `check_runs_every_task` |
| A warning fails the lint task and the doc task | `check_tasks_fail_on_warnings` |
| The test task runs the doc tests | `test_task_runs_the_doc_tests` |
| The policy file holds the policy | `dependency_policy_holds` |
| The hook file wires the tasks | `hooks_call_the_tasks` |
| The title reaches the check as a file | `pr_title_reaches_the_check_as_a_file` |
| CI checks the title of each pull request | `ci_checks_the_pr_title` |
| Workflow steps | `ci_steps_are_tasks` |
| No tool release in the workflow | `ci_states_no_tool_release` |
| The ruleset names the aggregate check | `ruleset_requires_ci_ok` |
| The aggregate check covers every job | `ci_ok_covers_every_job` |

Two more tests have no scenario. `cargo_tasks_use_locked` fails for a Cargo command in a task with no `--locked`. `hk_pins_match_mise` fails if the releases in `hk.pkl` differ from the `hk` release in `mise.toml`.

The tests read `ci.yml` with a YAML reader and the ruleset with a JSON reader. Task 6.1 chose `serde-saphyr` as the YAML reader, with `serde_json` for the value type and for the ruleset. `serde_yaml` is no longer maintained. `hk.pkl` has no reader in Rust, so its test searches the text of the file.

The first build had a second test file, `dev_workflow.rs`. Each of its tests copied the repository, added one defect and ran a `mise` task on the copy. Those tests proved the behavior of `rustfmt`, Clippy, `cargo-nextest`, `rustdoc`, `cargo-deny` and `hk`. They were slow, and one failed at random: two copies fetched the advisory database of `cargo-deny` at the same time. The owner stated the rule above, and the file was removed. The Cargo feature, the task `test:workflow` and the CI step for it went with it.

The copy helper of the boundary test stays in a shared test module, with the helper for the repository root and the helpers that read the output of a command.

### 8. Commit format

Each commit message and each pull request title follows the Conventional Commits format. The conventional commit check of `hk` decides what the format is, in the hook and in CI. The first build used `cocogitto`. It stops with an error on a machine with no git identity, and a CI runner has none.

- The `commit-msg` hook refuses a commit with another message. A hook is a choice of each clone.
- The `commits` job in CI checks the title of the pull request. The repository merges with a squash, so the title becomes the commit message on `main`. The commits of the branch do not reach `main`, and CI does not check them.

Enforcement: the `commits` job, which `ci-ok` needs. The ruleset requires `ci-ok`. The workflow also runs for the `edited` event of a pull request, so a new title is checked again. The task writes the title to a file and gives the file to the check, so a title such as `--help` is a title and not an option.

Release notes and a changelog tool are not in this change. They need a release process, and the parked issue for release tooling holds them. `changie`, `git-cliff` and `release-plz` are the candidates.

### 9. Community files

- `CONTRIBUTING.md` is a short pointer to the Contributing pages of the documentation site. The pages own the rules, as `project-foundation` decided. The file holds no rule. The Contributing section of `README.md` becomes one link to `CONTRIBUTING.md`, so the list of pages exists once.
- `SECURITY.md` tells a reporter to use the private vulnerability reporting of GitHub. Task 4.2 turns that feature on.
- `CODE_OF_CONDUCT.md` is the Contributor Covenant, unchanged, with the contact method filled in. The owner chose the address `conduct_mu_portal@fastmail.us` for it.
- Two issue templates: a bug report and an idea. The idea template links to the Contributing page for OpenSpec and the GitHub Project, and it restates no rule.
- One pull request template. It asks for the issue, the OpenSpec change and the result of `mise run check`.

Enforcement: none. These files are text for people, and a reviewer reads them.

### 10. Protection of `main`

`.github/rulesets/main.json` holds a repository ruleset for `main`:

- a change needs a pull request, with no required approval, because the project has one maintainer
- the only merge method is a squash, so the title of the pull request is the commit message on `main`
- each review thread must be resolved, and an unattributed change needs an extra approval. Both settings come from the ruleset `protect-main` of the owner
- the check `ci-ok` must pass
- no force push and no deletion

Task 7.2 applied the file with the GitHub API. The repository now has two active rulesets, `protect-main` and `main`. They do not conflict, and `main` adds the required check. The owner decided that `main` takes over the two settings that only `protect-main` had, and that `protect-main` is deleted after the merge of this change.

Task 7.2 applies the file with the GitHub API. The task runs after the pull request of this change is open, because `ci-ok` exists only from this change.

Enforcement of the check name: the tests `ruleset_requires_ci_ok` and `ci_ok_covers_every_job`. The file and the live ruleset can still differ, and nothing in this change compares them. Task 5.2 adds the instruction to the Contributing page for the build: change the file, and then apply it.

Alternative: classic branch protection. Rejected, because a ruleset is a JSON document that the repository can hold.

## Risks / Trade-offs

- [A contributor has no `mise`] → `mise.toml` is readable text with each release and each command. `CONTRIBUTING.md` names `mise` as the one tool to install first.
- [The Node pin leaves `docs/`, so a tool that reads `.node-version` finds none] → `mise` is the supported way to get Node. Task 5.2 states it on the build page.
- [The ruleset blocks the owner from a direct push to `main`] → That is the purpose. The owner can turn the ruleset off in the repository page in an emergency.
- [Some facts about the tools are not checked against their documentation] → Tasks 1.1, 2.1, 3.1 and 5.1 each read the current documentation first.
- [The community files have no test] → They are text for people. The goal that names them is their reason, and a reviewer reads them.
- [`cargo-nextest` does not run doc tests] → The `test` task runs `cargo test --doc` after it, and the test `test_task_runs_the_doc_tests` makes sure that the command stays.

## Migration Plan

1. Open the pull request of this change. CI then shows the new check names.
2. Apply the ruleset.

Rollback has four steps, in this order:

1. Remove the ruleset in the repository page. The revert drops the `ci-ok` job, and an active ruleset then blocks the revert.
2. Revert the commits. The revert restores `docs/.node-version` and the `packageManager` field.
3. Run `hk uninstall` in each clone that installed the hooks.
4. Turn off private vulnerability reporting.

## Open Questions

None.
