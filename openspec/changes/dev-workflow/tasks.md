# Tasks

## 1. Tools and tasks

- [ ] 1.1 Read the current `mise` documentation for three details: how `mise` reads `rust-toolchain.toml`, how a task depends on other tasks, and how a task sets its directory. Record each answer in `design.md` under Decision 1 or Decision 2. Make sure that each of the three answers names its documentation page.
- [ ] 1.2 Add `mise.toml` with the `[tools]` table from Decision 1. Take each release from `mise latest <tool>`. Make sure that `mise install` exits with status 0 and that `mise ls --current` lists each tool.
- [ ] 1.3 Add `rust-version` and `publish = false` to `[workspace.package]`, and inherit both in each crate. Add the lint entries from Decision 5 to `[workspace.lints]`, and fix the warnings that they report. Make sure that `cargo metadata --no-deps` reports both values for each crate and that `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` passes.
- [ ] 1.4 Add the tasks `fmt`, `lint`, `lint:actions`, `test`, `doc` and `check` from Decision 1. Make sure that `mise run check` exits with status 0.
- [ ] 1.5 Move the Node pin and the `pnpm` pin to `mise.toml`. Remove `docs/.node-version` and the `packageManager` field. Add the task `docs:check`. Make sure that `docs/pnpm-lock.yaml` holds no `pnpm` release after `pnpm install`, that `mise run docs:check` exits with status 0 and that `git grep -n "node-version"` finds no file outside `openspec/`.

## 2. Hooks

- [ ] 2.1 Read the current `hk` documentation for three details. The first is the builtin names for the TOML, YAML and conflict marker checks, and the tool that each one needs. The second is how a step runs a `mise` task. The third is the form of the package pins in `hk.pkl`. Record each answer in `design.md` under Decision 3. Make sure that each of the three answers names its documentation page.
- [ ] 2.2 Add `hk.pkl` with the top-level steps and the `pre-push` hook from Decision 3. Add each tool that a step needs to `mise.toml`, and add the task `hooks:check`. Make sure that `hk validate` and `mise run hooks:check` both exit with status 0.
- [ ] 2.3 Remove the untracked `.husky` directory from the clone and run `hk install --mise`. Make sure that `git config core.hooksPath` no longer prints `.husky` and that a commit of an unformatted Rust file fails.

## 3. Dependency policy and workspace rules

- [ ] 3.1 Read the current `cargo-deny` documentation for the configuration schema and for the two options that Decision 4 names. Add `deny.toml` with the dependency policy from Decision 4, and add the task `deny` to `check`. Add a dependency between two workspace crates once, and make sure that `mise run deny` still exits with status 0. Remove that dependency. Make sure that `mise run deny` exits with status 0.
- [ ] 3.2 Move the copy helper of the boundary test to a shared module in `crates/mu-cli/tests/`. Add `rust-version` and `publish` to the root and to each member of the base fixture. Make sure that `cargo nextest run -p mu-cli --test workspace_boundary --locked` passes with no change to a test.
- [ ] 3.3 Change the lints rule in `crates/mu-cli/tests/workspace_boundary.rs` into the one inheritance rule from Decision 5, over the fields `lints`, `rust-version` and `publish`. Add the rule for the `publish` value of the workspace. Add the tests `missing_rust_version_fails`, `local_rust_version_fails`, `missing_publish_fails`, `local_publish_fails` and `publishable_workspace_fails`. Make sure that each new test finds exactly one violation and that the two existing lints tests still pass.
- [ ] 3.4 Update the rule list on the Contributing page for the workspace layout for the inheritance rule. Make sure that the page lists each rule of the boundary test once.

## 4. Community files and decision record

- [ ] 4.1 Add `CONTRIBUTING.md` as a pointer to the Contributing pages, with `mise` as the first tool to install. Replace the link list in the Contributing section of `README.md` with one link to `CONTRIBUTING.md`. Make sure that each link resolves and that `CONTRIBUTING.md` states no rule.
- [ ] 4.2 Add `SECURITY.md`, and turn on private vulnerability reporting for the repository. Make sure that the GitHub API reports the feature as enabled.
- [ ] 4.3 Add `CODE_OF_CONDUCT.md` with the unchanged text of the Contributor Covenant and the contact address from Decision 8. Make sure that the file names its source and its release, and that the address in the file equals the address in Decision 8.
- [ ] 4.4 Add `.github/ISSUE_TEMPLATE/` with a bug report and an idea template, and add `.github/pull_request_template.md`. Make sure that the new-issue page of the repository offers both templates after the push.
- [ ] 4.5 Add a Decisions page for the dependency policy, and a Decisions page for the developer workflow with `mise` and `hk`. Make sure that both pages appear in the site sidebar and that `mise run docs:check` passes.

## 5. CI

- [ ] 5.1 Read the current documentation of the `mise` action, of the Rust build cache action and of the aggregate check action for their inputs. Rewrite `.github/workflows/ci.yml` so that CI runs the tasks, with the four jobs from Decision 6. Pin each action by commit SHA with its release in a comment. Make sure that `mise run lint:actions` reports no error.
- [ ] 5.2 Update the Contributing page for the build. Name `mise install`, `mise run check` and `mise run docs:check`, and remove the command lists. State that `mise` is the supported way to get Node. Add the instruction for the ruleset from Decision 9. Make sure that each command on the page runs as written and that `mise run docs:check` passes.

## 6. Tests and Renovate

- [ ] 6.1 Add `crates/mu-cli/tests/repo_config.rs` with the eight configuration tests from Decision 7 and the breaking texts that the decision lists. Choose a maintained YAML reader and record its name in `design.md`. Make sure that `cargo nextest run -p mu-cli --test repo_config --locked` passes.
- [ ] 6.2 Add `crates/mu-cli/tests/dev_workflow.rs` with the nine workflow tests from Decision 7, each with `#[ignore]` and a reason. The license test asserts that the output names the added crate and its license. Add the task `test:workflow`. Record the run time in `design.md`. Make sure that `mise run test:workflow` passes. For each defect test, remove the defect once and make sure that the test then fails.
- [ ] 6.3 Update the `context` field of `openspec/config.yaml`. The one-test command uses `cargo nextest` and runs an ignored test too. The library row for the toolchain pins names `mise.toml`. Make sure that the one-test command runs `deny_rejects_disallowed_license` by name.
- [ ] 6.4 Add a Renovate group for `hk` to `renovate.json`, with a custom manager for the package pins in `hk.pkl`. Make sure that `npx --yes --package renovate -- renovate-config-validator renovate.json` exits with status 0.

## 7. Integration

- [ ] 7.1 Add two parked GitHub issues to the mu-portal Project: release tooling, and the name of the binary. Make sure that both issues appear in the Project with the status Parked.
- [ ] 7.2 Open a pull request. Add `.github/rulesets/main.json` and apply it with the GitHub API, as Decision 9 describes. Make sure that the check `ci-ok` passes on the pull request and that the GitHub API lists the ruleset as active.
- [ ] 7.3 From a clean clone, run `mise install`, `mise run check`, `mise run docs:check` and `mise run test:workflow` in order. Make sure that each command exits with status 0 on macOS.
- [ ] 7.4 Run `openspec validate dev-workflow --strict`. Make sure that it passes.

## Workflow follow-up

- Run `/sb:openspec-review dev-workflow implementation` before task group 7, and treat a changes-requested verdict as a blocker.
- After the merge, read the Renovate dependency list. It must name each tool in `mise.toml` and the `hk` package pins. Open an issue for a pin that is absent.
- Archive the change after the implementation review passes.
- Close GitHub issue #18.
