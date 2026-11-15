# Spec Delta

## Purpose

Defines how a contributor and CI install the project tools and run the project checks. Both use one tool list and one command list.

## ADDED Requirements

### Requirement: One task runs every check
The repository SHALL provide one task, `check`, that runs every check of the Rust workspace: formatting, lints, tests, documentation and dependency policy. The task SHALL exit with status 0 on a tree with no defect. If one check fails, the task SHALL exit with a non-zero status.

#### Scenario: Clean tree
- **WHEN** a contributor runs `mise run check` on a tree with no defect
- **THEN** the exit status is 0

#### Scenario: Formatting defect
- **GIVEN** a copy of the tree in which one Rust source file is not formatted
- **WHEN** a contributor runs `mise run check` in that copy
- **THEN** the exit status is not 0

#### Scenario: Lint defect
- **GIVEN** a copy of the tree with one added function that Clippy warns about
- **WHEN** a contributor runs `mise run check` in that copy
- **THEN** the exit status is not 0

#### Scenario: Test defect
- **GIVEN** a copy of the tree with one added test that fails
- **WHEN** a contributor runs `mise run check` in that copy
- **THEN** the exit status is not 0

#### Scenario: Doc test defect
- **GIVEN** a copy of the tree with one added doc test that fails
- **WHEN** a contributor runs `mise run check` in that copy
- **THEN** the exit status is not 0

#### Scenario: Documentation defect
- **GIVEN** a copy of the tree with one added doc comment that links to an item that does not exist
- **WHEN** a contributor runs `mise run check` in that copy
- **THEN** the exit status is not 0

### Requirement: Dependency policy
The repository SHALL provide a task, `deny`, that rejects a crate in the dependency tree whose license is not in the allowed list. The allowed list SHALL be a file in the repository. The policy SHALL apply to a crate that cannot be published as it applies to every other crate. The `check` task SHALL run the `deny` task.

#### Scenario: Crate with a license that is not allowed
- **GIVEN** a copy of the tree with one added workspace crate that cannot be published and whose license is `GPL-3.0-only`
- **AND** a lock file in that copy that lists the added crate
- **WHEN** a contributor runs `mise run deny` in that copy
- **THEN** the exit status is not 0
- **AND** the output names the added crate and its license

#### Scenario: The check task runs the dependency policy
- **WHEN** a reader lists the tasks that the `check` task depends on in `mise.toml`
- **THEN** the list holds `fmt`, `lint`, `test`, `doc` and `deny`

### Requirement: Hooks run the checks
The repository SHALL provide git hooks through `hk`. The hook checks SHALL exit with a non-zero status for a Rust source file that is not formatted.

#### Scenario: Hook check on a formatting defect
- **GIVEN** a copy of the tree in which one Rust source file is not formatted
- **WHEN** a contributor runs `mise run hooks:check` in that copy
- **THEN** the exit status is not 0

#### Scenario: Hook check on a clean tree
- **WHEN** a contributor runs `mise run hooks:check` on a tree with no defect
- **THEN** the exit status is 0

### Requirement: Commit messages are conventional commits
The repository SHALL provide a git hook that refuses a commit whose message is not in the Conventional Commits format. The repository SHALL provide a task, `lint:pr-title`, that fails for a pull request title that is not in that format. The CI workflow SHALL run that task for each pull request.

#### Scenario: Commit with a plain message
- **GIVEN** a copy of the tree, as a git repository with the hooks installed
- **WHEN** a contributor commits with the message `fixed stuff`
- **THEN** git refuses the commit

#### Scenario: Commit with a conventional message
- **GIVEN** a copy of the tree, as a git repository with the hooks installed
- **WHEN** a contributor commits with the message `fix: handle the empty verb`
- **THEN** git accepts the commit

#### Scenario: Pull request title that is not conventional
- **WHEN** the `lint:pr-title` task runs with the title `Add the pair verb`, with the title `feat add the pair verb`, or with an empty title
- **THEN** the exit status is not 0

#### Scenario: Conventional pull request title
- **WHEN** the `lint:pr-title` task runs with the title `feat(cli): add the pair verb`
- **THEN** the exit status is 0

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
