# Proposal

## Why

The `project-foundation` change built the workspace, but a contributor still installs each tool by hand and types each check command. The check commands also exist twice, in the CI workflow and on the build page. A review of the repository against open source and Rust practice found gaps in three areas: supply-chain checks, build hygiene and community files. These gaps are cheap to close now, while the dependency tree is small and no contributor depends on the current commands.

## What Changes

- Add a `mise` configuration at the root. It pins each tool that a contributor or CI needs, and it defines the tasks that run the checks. One command installs the tools, and one command runs every check.
- Make the `mise` tasks the single list of check commands. CI calls the tasks. The build page names the tasks and no longer lists the commands.
- Move the Node pin and the `pnpm` pin into the `mise` configuration. Remove `docs/.node-version` and the `packageManager` field. `rust-toolchain.toml` stays the Rust pin, and `mise` reads it.
- Add `hk` git hooks. The hook before a commit runs the fast checks. The hook before a push runs the full check task. CI runs the same `hk` checks on every file.
- Replace the local `.husky` hook directory, which no file in the repository configures.
- Guard the commit format. A hook refuses a commit message that is not a conventional commit. A CI job refuses a pull request title that is not one, because a squash merge makes the title the commit message on `main`.
- Add `cargo-deny` with a configuration for advisories, licenses, banned crates and sources.
- Add build hygiene to the workspace. Each crate states a minimum Rust release, so Cargo reports a toolchain that is too old. Each crate states `publish = false`, so nobody publishes one by accident. Each Cargo command in a task uses `--locked`. A `cargo doc` step fails on a warning. `cargo-nextest` is the test runner. The workspace lints gain the pedantic Clippy group and a lint for a public item with no doc comment.
- Add a build cache to the Rust CI job.
- Add community files: `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, issue templates and a pull request template. `CONTRIBUTING.md` is a short pointer to the Contributing pages.
- Protect the `main` branch with a ruleset that requires a pull request and one CI check. The ruleset is a file in the repository.
- Add two Decisions pages: the dependency policy, and the developer workflow.
- Add two parked issues: release tooling, and the name of the binary. Homebrew already has a formula named `mu`.

Not in this change: release tooling, release notes, a changelog tool, the name of the binary, a change of license, test coverage reports, and any product behavior of `mu`.

## Capabilities

### New Capabilities

- `developer-workflow`: How a contributor and CI install the tools and run the checks. It covers the single check command and its failure on a defect. It also covers the dependency policy, the git hooks, and the rule that CI runs the same tasks. The last part is the check that the `main` branch requires.

### Modified Capabilities

None.

## Impact

- **New tool files**: `mise.toml`, `hk.pkl` and `deny.toml`.
- **New tests**: one Rust test file in `crates/mu-cli/tests/` for the configuration of the repository.
- **New community files**: `CONTRIBUTING.md`, `SECURITY.md` and `CODE_OF_CONDUCT.md`.
- **New GitHub files**: `.github/ISSUE_TEMPLATE/`, `.github/pull_request_template.md` and `.github/rulesets/main.json`.
- **Changed files**: `.github/workflows/ci.yml`, the root `Cargo.toml`, each crate manifest and `docs/package.json`. The Contributing page for the build also changes. If Renovate does not read a new pin, `renovate.json` changes too.
- **Removed files**: `docs/.node-version`.
- **Earlier decision**: this change replaces part of Decision 4 of `project-foundation`. The Node pin and the `pnpm` pin move from two files in `docs/` to `mise.toml`.
- **Contributors**: a contributor needs `mise`. A contributor without `mise` reads the tool releases and the commands in `mise.toml`.
- **Branch rules**: the ruleset changes how a commit reaches `main`. A direct push to `main` stops working.
- **GitHub**: this change is tracked by issue #18. It depends on issue #2 and pull request #17.
