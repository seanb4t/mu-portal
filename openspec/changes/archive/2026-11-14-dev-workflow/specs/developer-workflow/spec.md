# Spec Delta

## Purpose

Defines how a contributor and CI install the project tools and run the project checks. Both use one tool list and one command list.

## ADDED Requirements

### Requirement: One task runs every check
The repository SHALL provide one task, `check`, that runs every check of the Rust workspace: formatting, lints, tests, documentation and dependency policy. A lint warning and a documentation warning SHALL each fail their task. The tests SHALL include the doc tests.

#### Scenario: The check task runs each check
- **WHEN** a reader lists the tasks that the `check` task depends on in `mise.toml`
- **THEN** the list holds `fmt`, `lint`, `test`, `doc` and `deny`

#### Scenario: A warning fails the lint task and the doc task
- **WHEN** a reader reads the `lint` task and the `doc` task in `mise.toml`
- **THEN** the `lint` task gives Clippy the arguments `-D warnings` and turns on every feature
- **AND** the `doc` task sets `RUSTDOCFLAGS` to `-D warnings`

#### Scenario: The test task runs the doc tests
- **WHEN** a reader reads the `test` task in `mise.toml`
- **THEN** the task holds a `cargo nextest run` command for the workspace
- **AND** the task holds a `cargo test` command with `--doc`

### Requirement: Dependency policy
The repository SHALL hold the dependency policy in `deny.toml`, and the `check` task SHALL run it. The policy SHALL allow no copyleft license. The policy SHALL apply to a crate that cannot be published and to a dependency behind an optional feature. The policy SHALL refuse a wildcard dependency and an unknown source.

#### Scenario: The policy file holds the policy
- **WHEN** a reader reads `deny.toml`
- **THEN** the allowed list holds at least one license and no license of the GPL, AGPL, LGPL or SSPL families
- **AND** the option that ignores a private crate is off
- **AND** the dependency tree is read with all features on
- **AND** a wildcard dependency, an unknown registry and an unknown git source are each denied

### Requirement: Hooks call the tasks
The repository SHALL provide git hooks through `hk`. The steps before a commit SHALL call the `fmt` task and the `lint:actions` task. The hook before a push SHALL call the `check` task. The hook for a commit message SHALL hold the conventional commit check.

#### Scenario: The hook file wires the tasks
- **WHEN** a reader reads `hk.pkl`
- **THEN** a top-level step calls `mise run fmt` and another calls `mise run lint:actions`
- **AND** the `pre-push` hook calls `mise run check`
- **AND** the `commit-msg` hook holds the conventional commit check

### Requirement: Pull request titles are conventional commits
The repository SHALL provide a task, `lint:pr-title`, that checks a pull request title against the Conventional Commits format. The task SHALL give the title to the check in a file, so that no title is read as an option. The CI workflow SHALL run the task for each pull request, and again after an edit of the title.

#### Scenario: The title reaches the check as a file
- **WHEN** a reader reads the `lint:pr-title` task in `mise.toml`
- **THEN** the task writes the `PR_TITLE` variable to a file
- **AND** the task gives that file to the conventional commit check, and not the title

#### Scenario: CI checks the title of each pull request
- **WHEN** a reader reads `.github/workflows/ci.yml`
- **THEN** one step runs `mise run lint:pr-title` with the title of the pull request in the `PR_TITLE` variable
- **AND** the workflow runs for the `edited` event of a pull request

### Requirement: CI runs the tasks
Each command step of the CI workflow SHALL be one `mise` task that `mise.toml` defines. The workflow SHALL NOT state a check command that a task already holds. The workflow SHALL NOT state the release of a tool that a pin file holds.

#### Scenario: Workflow steps
- **WHEN** a reader lists the `run` steps of `.github/workflows/ci.yml`
- **THEN** each step is exactly `mise run` and one task name
- **AND** `mise.toml` defines that task

#### Scenario: No tool release in the workflow
- **WHEN** a reader searches `.github/workflows/ci.yml` for the release of each tool in `mise.toml` and for the Rust release in `rust-toolchain.toml`
- **THEN** the search finds none of them, in the full form or in the form with no patch number
- **AND** a release that is part of a longer number does not count as a match

### Requirement: The main branch requires the checks
The repository SHALL hold a ruleset file for the `main` branch. The ruleset SHALL require one check, and that check SHALL fail unless every other job of the CI workflow succeeded.

#### Scenario: The ruleset names the aggregate check
- **WHEN** a reader compares `.github/rulesets/main.json` with `.github/workflows/ci.yml`
- **THEN** the one required check of the ruleset is the name that GitHub reports for the `ci-ok` job

#### Scenario: The aggregate check covers every job
- **WHEN** a reader lists the jobs of `.github/workflows/ci.yml`
- **THEN** the `ci-ok` job needs every other job
- **AND** the `ci-ok` job runs after a needed job fails
