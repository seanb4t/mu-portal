# Tasks

## 1. Workspace and license

- [x] 1.1 Replace `LICENSE` with the unchanged Apache-2.0 text. Make sure that the file contains "Apache License" and "Version 2.0, January 2004".
- [x] 1.2 Add `rust-toolchain.toml` with the current stable release, `rustfmt` and `clippy`. Make sure that `rustc --version` in the repository reports the pinned release.
- [x] 1.3 Add the root virtual workspace with `[workspace.package]`, `[workspace.lints]` and `[workspace.dependencies]`, as Decision 1 describes. Add `spikes` to `exclude`. Make sure that `cargo metadata --no-deps` reads the manifest with no error.
- [x] 1.4 Add the crates `mu-proto`, `mu-core`, `mu-tui` (empty libraries) and `mu-cli` (binary `mu`), each with inherited package fields and lints. Make sure that `cargo build --workspace` passes and produces `target/debug/mu`.
- [x] 1.5 Add the violation function and the allowlist in `crates/mu-cli/tests/workspace_boundary.rs`, with `cargo_metadata` and `toml` as development dependencies. Add the test `real_workspace_has_no_violation`. Make sure that it passes.
- [x] 1.6 Add the fixture with all five allowed edges and the test `allowed_edges_have_no_violation`. Add the negative cases from Decision 2 and one test for each. The test names are in the list below. Make sure that each negative test finds exactly one violation and that `cargo test -p mu-cli --test workspace_boundary` passes.
  - `optional_tui_to_core_fails`, `build_tui_to_core_fails`, `dev_tui_to_core_fails`, `renamed_tui_to_core_fails`, `target_tui_to_core_fails`
  - `git_tui_to_core_fails`
  - `local_dependency_version_fails`, `local_dev_dependency_version_fails`, `local_target_dependency_version_fails`, `renamed_local_dependency_version_fails`
  - `core_to_tui_fails`, `proto_dependency_fails`, `spike_dependency_fails`, `outside_path_dependency_fails`, `unlisted_member_fails`
  - `missing_license_fails`, `wrong_license_fails`, `missing_lints_fails`, `local_lints_fail`
- [x] 1.7 Keep the Node template in `.gitignore` and add `target/` and `.archify/`. Make sure that `git status` shows no new file after `cargo build --workspace` and after `cargo test --workspace`.

## 2. Command grammar

- [x] 2.1 Add `clap` through `[workspace.dependencies]`. Define the single binary `mu` with its three verbs `client`, `server` and `version`, as Decision 3 describes: the explicit name `mu`, a doc comment on each verb, no `help` subcommand, and an optional subcommand with `client` as the default. Put the client arguments in one empty arguments struct that the `client` verb and the top-level command share. Make sure that `mu --help` lists exactly the three verbs.
- [x] 2.2 Add one function for a verb without an implementation. It takes the verb name, writes `<verb>: not implemented` to stderr and returns exit status 1. Call it from `client` and `server`. Make sure that `mu server` writes `server: not implemented` and exits with status 1.
- [x] 2.3 Add the version output: the `version` verb prints the string from `Command::render_long_version` and adds no second newline. Make sure that `mu version` and `mu --version` both print the line `mu`, one space, and the package version.
- [x] 2.4 Add integration tests in `crates/mu-cli/tests/cli.rs` that run the built binary, one test for each scenario in `specs/cli-entrypoint/spec.md`: `help_lists_verbs`, `help_for_one_verb`, `no_verb_equals_client`, `version_verb`, `version_flag_matches_verb`, `unknown_verb_is_usage_error`, `server_not_implemented`, `client_not_implemented`, `stub_verbs_leave_files_alone`. The two version tests compare the whole line, not a part of it. Make sure that `cargo test -p mu-cli --test cli` passes.
- [x] 2.5 Add the property test `unknown_verb_property` in the same file, with `proptest`. The strategy is the pattern `[a-z][a-z0-9_]{0,15}` with the verb names removed. The test reads the verb names from the command definition in the library target. That domain has no NUL byte, which a process argument cannot hold, and no leading dash. Add the fixed case `help_is_not_a_verb` for `mu help`, and the fixed cases `verb_names_are_exact` for strings next to a verb name. For each string it expects exit status 2, an empty stdout and the string on stderr. Make sure that it passes.
- [x] 2.6 Make sure that `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## 3. Spikes workspace

- [x] 3.1 Add `spikes/Cargo.toml` as a separate workspace. Add `spikes/README.md` as a short pointer to the Contributing page for spikes. Make sure that `cargo build --workspace` at the root does not compile anything in `spikes/`.

## 4. Documentation site and architecture model

- [x] 4.1 Add `docs/` from the Starlight starter, with `docs/.node-version` and the `packageManager` field. Keep the starter's `docs/.gitignore`. Make sure that `pnpm build` in `docs/` passes with the default page and that `git status` then shows no generated file.
- [x] 4.2 Read the current LikeC4 documentation for two details: the Vite plugin option that sets the model directory, and the command that reports model errors. Record both in `design.md` under Decision 7. If no such command exists, record that the site build is the check.
- [x] 4.3 Add the LikeC4 model in `architecture/` with the system context view. Configure the Vite plugin, add the Astro component that wraps the view, and embed the view on one MDX page. Make sure that the built page contains the view.
- [x] 4.4 Add the server-and-clients view and the crate layout view, each embedded on a page in the Architecture section. Make sure that the check from task 4.2 reports no error and that `pnpm build` passes.
- [x] 4.5 Add a relation to an element that does not exist in the model. Make sure that the check from task 4.2 exits with a non-zero status. Remove the relation and make sure that the check passes again.

## 5. Documentation content

- [x] 5.1 Add the Overview page and one Decisions page for each project-wide decision from the exploration. The pages are in the list below. Each page names its source, as Decision 6 describes. The Decisions pages for the diagram policy and for spikes hold no rules and link to their Contributing pages. Make sure that each page appears in the site sidebar.
  - server-centered shape, one binary with verbs, ConnectRPC API, pairing, embedded Tailscale
  - where the server runs, alternate-screen client, Lua 5.4, world-type profiles
  - structured journal, importers, diagram policy, license
- [x] 5.2 Add the Contributing pages: build and test, workspace layout, spikes, diagram policy, OpenSpec and the GitHub Project. The workspace layout page names `tests/workspace_boundary.rs` and the crate layout view, and tells the reader to change both in the same change. The diagram policy page is the single owner of the rules from Decision 8. The spikes page holds the rules for a spike: one question, a time limit, a written decision, and a name taken from its issue. The OpenSpec page states that a change that implements a verb modifies the not-implemented requirement. Make sure that each command on the build page runs as written.
- [x] 5.3 Replace the README body with the project name, its status and a link to the Contributing pages. Make sure that each link resolves.

## 6. Continuous integration

- [x] 6.1 Add `.github/workflows/ci.yml` with the `rust` job on Linux and macOS and the three commands from Decision 11. The job reads `rust-toolchain.toml` and states no Rust version. Pin each action by commit SHA with its version in a comment, taken from the current documentation of the action.
- [x] 6.2 Add the `docs` job on Linux with the three commands from Decision 11. The model check is the one that task 4.2 chose. The job reads `docs/.node-version` and states no version. Give the `pnpm` setup action the path `docs/package.json`, with the input name from the current documentation of the action.
- [x] 6.3 Add `renovate.json` that extends the Renovate best-practices preset. Read the preset in the Renovate documentation. If the preset does not pin GitHub Actions to digests, add the digest preset. Make sure that the configuration passes the Renovate configuration validator.
- [x] 6.4 Add two parked GitHub issues to the mu-portal Project: Open Knowledge Format alignment, and the collapse of the crate layout view into the boundary test allowlist. Make sure that both issues appear in the Project with the status Parked.
- [ ] 6.5 Open a pull request. Make sure that both jobs pass on it. After the merge, make sure that the Renovate dependency list names the Rust toolchain, the Node version, `pnpm`, each action and each crate. Add a custom manager for a pin that is absent.

## 7. Integration

- [ ] 7.1 From a clean clone, run the commands on the Contributing build page in order. Make sure that the build, the tests and the site build all pass on macOS.
- [ ] 7.2 Run `openspec validate project-foundation --strict`. Make sure that it passes.

## Workflow follow-up

- Run `/sb:openspec-review project-foundation implementation` before task group 7, and treat a changes-requested verdict as a blocker.
- Archive the change after the implementation review passes.
- Close GitHub issue #2.
