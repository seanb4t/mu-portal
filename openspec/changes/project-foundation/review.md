---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-15T00:53:07+00:00
head: 95814363a2de4fb8723994406a8fc1b0985d7857
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: ok
  invariants: ok
---
# Review

## Findings

| id | lens | severity | claim | evidence | action |
|---|---|---|---|---|---|
| app-1 | approach | should-fix | The Overview page is an MDX file but embeds no component. Decision 6 makes a page MDX only if it embeds a component, and design.md records no deviation for this page. | design.md > 6. Documentation site: Astro Starlight in `docs/` — "Pages are Markdown. If a page embeds a component, the page is MDX." | Rename docs/src/content/docs/index.mdx to index.md. The page has no import and no component, so its content stays unchanged. Or record in Decision 6 why the Overview page stays MDX. |
| fit-1 | fit | should-fix | The Rust check commands, with their flags, are written twice: once on the Contributing build page and once in the CI workflow. Nothing keeps the two lists equal. The docs side already uses a named script (model:check) that both places call, and the Rust side has no such name. | docs/src/content/docs/contributing/build.md:20 — "cargo clippy --workspace --all-targets -- -D warnings" | Add a Cargo alias in .cargo/config.toml, for example lint = "clippy --workspace --all-targets -- -D warnings". Call it from ci.yml and from build.md, as the docs job and the build page both call pnpm model:check. If the copy stays, name it in Decision 11 and state that task 7.1 is its only check. |
| fit-2 | fit | should-fix | Each crate's one-line description appears in three live places added by this change: the Cargo manifest, the LikeC4 crate element and the Crates table on the workspace page. The design names only the copy of the edges, not this copy of the descriptions. | crates/mu-core/Cargo.toml:3 — "description = "Server engine for mu"" | Remove the Crates table from workspace.md and keep the link to the crate layout view, which already shows each description. Either drop the description fields from the unpublished crate manifests, or name model.c4 as the source in Decision 1 and on the workspace page. |
| inv-1+inv-2 | invariants | should-fix | inv-1: The spikes workspace depends on the invariant that every directory in spikes/ other than target is a crate. Nothing states or checks it. A probe showed that a directory without Cargo.toml (spikes/notes or a hidden spikes/.idea) makes cargo refuse the whole spikes workspace, so every spike stops building.; inv-2: The second review added exclude = target to keep the glob from matching the build directory. No test loads the spikes workspace when a target directory is present, and CI does not build spikes/, so nothing would catch a regression of that fix. | spikes/Cargo.toml:3 — "members = ["*"]" | inv-1: Add a test to crates/mu-cli/tests/workspace_boundary.rs that runs MetadataCommand with no_deps on spikes/Cargo.toml and expects success, so CI loads the spikes workspace. Also add a sentence to the spikes Contributing page: every directory in spikes/ except target must hold a Cargo.toml.; inv-2: Add a test that copies spikes/Cargo.toml into a temporary directory with an empty target/ directory and one minimal spike crate. It runs cargo metadata --no-deps there and asserts that the members are exactly that one crate. |
| inv-3 | invariants | should-fix | Decision 1 says mu-tui cannot reach mu-core through another crate. The boundary function reads declared edges with no_deps. A third-party (git or registry) dependency that takes workspace = true passes every rule, and its own dependency on mu-core is never read. The design names this blind spot only for outside path crates. | design.md > 1. A virtual Cargo workspace with four crates — "`mu-tui` therefore cannot reach `mu-core`, directly or through another crate." | Pick one. Narrow the Decision 1 sentence and the workspace Contributing page to the declared edges of members and of path crates, and say that the edges of a third-party crate are out of scope. Or add a resolved-graph check: run cargo metadata without no_deps and assert that no package named mu-core is reachable from mu-tui. |
| inv-4 | invariants | should-fix | Fixture::append assumes that [dependencies] is the last table of each base fixture manifest, and nothing checks that. If a table is added after it, an append with no header of its own lands in that table. local_dependency_version_fails and optional_tui_to_core_fails would then test a different dependency kind, or fail to parse. | crates/mu-cli/tests/workspace_boundary.rs:217 — "/// Adds text to the end of a manifest. The last table of each base" | Make append assert the assumption: when text does not start with a table header, the last table header in the manifest must be [dependencies]. Or parse the manifest with toml and insert the entry into the named table. |
| fit-3 | fit | note | The CLI tests start the binary in three places. Each place builds the same Command and adds its own arguments or environment. The helper mu() covers only the plain-arguments case, so the other two tests repeat its body. | crates/mu-cli/tests/cli.rs:12 — "Command::new(env!("CARGO_BIN_EXE_mu"))" | Add a mu_command() helper that returns the Command. mu() and the two tests can then add arguments, the working directory and the environment to it. The parameter is the Command configuration. |

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| Help lists the verbs | crates/mu-cli/tests/cli.rs: help_lists_verbs | covered | pass |
| Help for one verb | crates/mu-cli/tests/cli.rs: help_for_one_verb | covered | pass |
| No verb | crates/mu-cli/tests/cli.rs: no_verb_equals_client | covered | pass |
| Version verb | crates/mu-cli/tests/cli.rs: version_verb | covered | pass |
| Version flag | crates/mu-cli/tests/cli.rs: version_flag_matches_verb | covered | pass |
| Unknown verb | crates/mu-cli/tests/cli.rs: unknown_verb_is_usage_error | covered | pass |
| Server verb before the server exists | crates/mu-cli/tests/cli.rs: server_not_implemented | covered | pass |
| Client verb before the client exists | crates/mu-cli/tests/cli.rs: client_not_implemented | covered | pass |
| A stub verb leaves the file system alone | crates/mu-cli/tests/cli.rs: stub_verbs_leave_files_alone | covered | pass |

### Not run

None.

## Verification runs

- exit 0: `cargo test -p mu-cli --test cli -- --exact help_lists_verbs`
- exit 0: `cargo test -p mu-cli --test cli -- --exact help_for_one_verb`
- exit 0: `cargo test -p mu-cli --test cli -- --exact no_verb_equals_client`
- exit 0: `cargo test -p mu-cli --test cli -- --exact version_verb`
- exit 0: `cargo test -p mu-cli --test cli -- --exact version_flag_matches_verb`
- exit 0: `cargo test -p mu-cli --test cli -- --exact unknown_verb_is_usage_error`
- exit 0: `cargo test -p mu-cli --test cli -- --exact server_not_implemented`
- exit 0: `cargo test -p mu-cli --test cli -- --exact client_not_implemented`
- exit 0: `cargo test -p mu-cli --test cli -- --exact stub_verbs_leave_files_alone`

## Dropped

None.

## Lens notes

- approach: clap facts checked against clap_builder@4.6.7 and clap@4.6.7 in the local Cargo registry: render_long_version returns the name, one space, the version and one newline (command.rs line 4910). The usage-error exit code is 2 (util/mod.rs line 32). The git-derive example has the default-subcommand shape that Decision 3 cites (examples/git-derive.rs line 79).
- approach: LikeC4 facts checked against likec4@1.60.0 in docs/node_modules: the Vite plugin has a workspace option (dist/vite-plugin/index.d.mts line 100), and the CLI has a validate command.
- approach: The pinned Rust release 1.100.0 fits the six-week cadence for a stable release on 2026-11-12, so it is the current stable release on the day of the build.
- approach: Not checked, because this lens has no network tool: the mapping of each action commit SHA in ci.yml to its version comment, the input name package_json_file in the documentation of pnpm/action-setup, the content of the Renovate config:best-practices preset, and the GitHub issues of task 6.4.
- approach: Every requirement in specs/cli-entrypoint/spec.md has a decision in design.md that produces it, and the code in crates/mu-cli/src/lib.rs follows Decision 3. The boundary test follows Decision 2 in its classes, its rules and its negative cases.
- fit: The crate layout view in architecture/model.c4 copies ALLOWED_EDGES from workspace_boundary.rs. Decision 7 names this copy and task 6.4 tracks the collapse issue, so it is not a finding.
- fit: The library rows in openspec/config.yaml hold: every third-party crate comes from [workspace.dependencies], the shared package fields and lints are inherited, the toolchain pins are in their files and CI does not state them a second time, and the verb list comes from the clap command in crates/mu-cli/src/lib.rs.
- fit: The CLI follows the clap git-derive default-subcommand shape that Decision 3 names. The docs site comes from the Starlight starter. Renovate extends config:best-practices. None of these departs from a named pattern.
- fit: The model directory ../architecture is written in docs/astro.config.mjs:22 and docs/package.json:9. It is too small for a finding, and it is recorded here only.
- invariants: I1: mu --help lists exactly the verbs client, server and version, each with a description \| enforced at crates/mu-cli/src/lib.rs Verb enum doc comments and disable_help_subcommand \| boundary test help_lists_verbs (exact equality), help_is_not_a_verb
- invariants: I2: mu with no verb behaves exactly as mu client \| enforced at lib.rs run(), where None and Client both call client() \| boundary test no_verb_equals_client
- invariants: I3: mu version and mu --version print the same single line, mu and the package version \| enforced at lib.rs render_long_version, a single source \| boundary test version_verb (whole line), version_flag_matches_verb (byte equality)
- invariants: I4: an unknown verb exits 2, writes nothing to stdout and names the verb on stderr \| enforced at clap Cli::parse in main.rs \| boundary tests unknown_verb_is_usage_error, verb_names_are_exact (empty string, prefixes, one character past, case), non_utf8_verb_is_usage_error, proptest unknown_verb_property
- invariants: I5: a stub verb writes '<verb>: not implemented' to stderr and exits 1 \| enforced at lib.rs not_implemented \| boundary tests server_not_implemented, client_not_implemented
- invariants: I6: a stub verb never creates, changes or deletes a file \| enforced at lib.rs not_implemented, which opens no file \| boundary test stub_verbs_leave_files_alone (cwd, HOME and XDG_CONFIG_HOME in one directory; entries, content and mtime)
- invariants: I7: every workspace member is in ALLOWED_EDGES \| enforced at workspace_boundary.rs UnlistedMember \| boundary test unlisted_member_fails
- invariants: I8: every member edge is in the allowlist, whatever its kind or source \| enforced at ForbiddenEdge \| boundary tests optional/build/dev/renamed/target/git_tui_to_core_fails, core_to_tui_fails, proto_dependency_fails (empty allowlist)
- invariants: I9: no member has a path dependency into spikes/ or outside the members \| enforced at SpikeDependency and OutsidePathDependency, path checked before name \| boundary tests spike_dependency_fails, spike_named_like_member_fails, outside_path_dependency_fails
- invariants: I10: every member has license Apache-2.0 \| enforced at the License violation, exact equality \| boundary tests missing_license_fails (null), wrong_license_fails
- invariants: I11: every member has [lints] workspace = true \| enforced at LintsNotInherited \| boundary tests missing_lints_fails, local_lints_fail. The explicit workspace = false value has no test, but it goes through the same unwrap_or(false) branch as the missing-key case
- invariants: I12: every third-party dependency uses workspace = true \| enforced at DependencyNotFromWorkspace over every scope and table \| boundary tests local_dependency_version_fails, local_dev_dependency_version_fails, local_target_dependency_version_fails, renamed_local_dependency_version_fails
- invariants: I13: each dependency breaks one rule at most \| enforced at the has_own_rule and path-first ordering in violations() \| boundary test: each negative test asserts a vector of exactly one violation
- invariants: I14: the boundary function writes no lock file into a fixture \| enforced at the no_deps option \| boundary test none. A probe under probes/probeA in the handoff directory found no Cargo.lock after cargo metadata --no-deps
- invariants: I15: dependency paths and member manifest paths share one prefix form, so the starts_with and contains comparisons hold \| enforced by cargo metadata normalization \| probe: a trailing slash, a ./ segment and a non-canonical /var vs /private/var manifest path all came out normalized and consistent
- invariants: I16: the spikes workspace loads with no members, with a target directory and with a .DS_Store file \| enforced at spikes/Cargo.toml members * and exclude target, and by cargo dropping glob matches that are files \| probe under probes/spikesB passed. A directory with no manifest fails (see inv-1)
- invariants: I17: a spike can depend on a crate in crates/ by path \| enforced by cargo inheritance from the root of the dependency's own workspace \| probe under probes/C resolved mu-proto from a spike
- invariants: I18: the crate layout view equals ALLOWED_EDGES \| enforced at none; the design names it as a review rule and task 6.4 tracks it \| at head, architecture/model.c4 lines 63-67 match the allowlist
- invariants: I19: CI states no toolchain version a second time \| enforced at review only (Decision 4) \| at head, ci.yml uses rustup toolchain install, node-version-file and package_json_file
- invariants: I20: each action is pinned by SHA with a version comment \| enforced at review and the Renovate best-practices preset (Decision 11) \| at head, all three uses lines in ci.yml are SHA-pinned
- invariants: I21: an Archify diagram commits its JSON and SVG together \| enforced at none, by design, until the first diagram (Decision 8) \| no diagram exists at head
- invariants: A boundary bypass outside Cargo, such as a #[path] module or include! in mu-tui that reads mu-core sources, makes no Cargo edge, so the boundary function cannot see it. This is recorded here and not as a finding
- invariants: No property-candidate finding: the one invariant over an unbounded domain, the unknown verb, already has a proptest property. The other domains are bounded enumerations.
- invariants: Grep and Glob were not available. Repository search used git ls-files and Read. The probes ran only cargo metadata, on inputs under the handoff directory probes/.
- test-coverage: All nine scenarios have a test whose assertions check the THEN conditions. Tests were not run, per lens rules.
- test-coverage: Tasks 6.5, 7.1 and 7.2 are unchecked and have no scenario; not a coverage finding.
- test-honesty: Read cli.rs, workspace_boundary.rs, ci.yml and docs/package.json. Every test asserts the THEN of its scenario: exact version line, exit codes, stderr naming the verb, empty stdout, file state. No disabled tests, no assertion-free tests, no mock echo.
- test-honesty: Boundary negative tests assert the exact violation vector, so each can fail for the named reason.
- test-honesty: CI gates (fmt, clippy -D warnings, test, model:check, build) have no ignore-failure flags.

## Plan review (2026-11-15)

Verdict approve-with-notes at head a0f7a3f642f781ba4f2336740e675f0421e2c147.

### Findings

| id | lens | severity | claim | evidence | action |
|---|---|---|---|---|---|
| app-1 | approach | should-fix | The boundary test enforces that every third-party dependency of a member uses workspace = true, with its own negative fixture, but no requirement, non-goal or risk needs this rule. The proposal asks the test to enforce only the license, and the stated goal is the client and server boundary. | design.md > 1. A virtual Cargo workspace with four crates — "A later change that shares a dependency between crates then edits one table." | Either name the need in the proposal or in a risk (for example, version drift between crates), or drop the dependency-table rule, the toml-based check for it and the local_dependency_version_fails fixture. |
| app-2 | approach | should-fix | The boundary test enforces an inherited [lints] table in every member, with two negative fixtures, but no requirement, non-goal or risk needs it. Decision 1 gives the lints table no reason at all. | design.md > 1. A virtual Cargo workspace with four crates — "`[workspace.lints]`: the lint levels." | State the need that the lints rule serves in the proposal or in a risk, or drop the rule and the missing_lints_fails and local_lints_fail fixtures. |
| ctl-lens | approach | should-fix | lens approach reported partial, gaps: 1 | review > lens-status — "partial" | rerun the lens or accept the gap in the review |
| fit-1 | fit | should-fix | The change keeps the root Node-template .gitignore and adds the Starlight starter's docs/.gitignore, so the Node and build-output rules for docs/ live in two files. The root template is also a Node file in a Rust repository, and its bare patterns such as logs and dist match at any depth. The design gives no reason for keeping it. | .gitignore:41 — "node_modules/" | Make docs/.gitignore the one owner of the Node and Astro rules. Reduce the root .gitignore to the repository rules (target/, .archify/, editor and OS files) and change task 1.7 to match. Otherwise state in Decision 6 why the root Node template stays. |
| fit-2 | fit | should-fix | The boundary function has two rules for one thing, a path dependency on a crate that is not a workspace member. A spike is not a member, and the outside-path rule leaves spikes/ out only so that the two rules stay apart. | design.md > 2. A test enforces the workspace rules — "A path dependency whose path is under `spikes/`. A spike is found by its manifest path, because a spike is not a workspa" | Merge the two into one rule: a path dependency on a non-member is a violation. If a spike-specific message is wanted, let the one rule pick its message from the path. Keep both fixtures as cases of that one rule. |
| fit-3 | fit | should-fix | The change writes the crate-edge allowlist three times: in the boundary test, in the Decision 1 table and in the LikeC4 crate layout view. The view copy has no check. The collapse is only a parked issue (task 6.4) with no point by which it must land. | design.md > 2. A test enforces the workspace rules — "The allowlist in the test file is the single source of the rule. The table in Decision 1 and the LikeC4 crate layout vie" | Tie the issue from task 6.4 to a trigger: it must land before the first change that adds a crate or an edge. Otherwise add a check in this change that compares the relations of the crate layout view with the allowlist. Treat the Decision 1 table as history that the archive freezes, not as a live copy. |
| inv-1 | invariants | should-fix | The edge allowlist applies to path dependencies alone, so a git or registry dependency of mu-tui on the package mu-core (for example this repository by git URL, via workspace = true) produces no violation. A probe of cargo metadata --no-deps reports such a dependency with name mu-core and source git+https://github.com/seanb4t/mu-portal, which Decision 2 classes as third-party, and the dependency-table rule passes it because it uses workspace = true. | design.md > 1. A virtual Cargo workspace with four crates — "`mu-tui` therefore cannot reach `mu-core`, directly or through another crate." | In Decision 2, apply the edge allowlist to every declared dependency whose package name is an allowlisted crate name, whatever its source, and state which single rule such a dependency breaks. Add a negative fixture git_tui_to_core_fails (a git dependency on mu-core declared in [workspace.dependencies] and used with workspace = true) to the list in task 1.6. |
| inv-2 | invariants | should-fix | The dependency-table rule covers every dependency table that the toml reader must visit (normal, build, dev and target-specific), but only one negative fixture exercises it, with no kind named. A probe shows that cargo metadata reports dev and cfg(unix) third-party dependencies with a registry source, so the toml side must look in [dev-dependencies], [build-dependencies] and [target.*.*] too, and no test passes those edges. | design.md > 2. A test enforces the workspace rules — "a third-party dependency with its own version and no `workspace = true`" | Add negative fixtures for the dependency-table rule in a build, a dev and a target-specific table (for example local_dev_dependency_version_fails, local_build_dependency_version_fails, local_target_dependency_version_fails), as the tui-to-core rule already has for each kind. |
| inv-3 | invariants | should-fix | The dependency-table rule joins a cargo_metadata dependency (named by package name) to its toml entry (keyed by the rename). A probe shows a renamed dependency with name mu-core and rename mu-core-git, so a lookup by name misses the entry. The renamed case is tested for the edge rule only, not for the dependency-table rule. | design.md > 2. A test enforces the workspace rules — "An edge is named by the package name that `cargo_metadata` reports, not by the key in the manifest." | State in Decision 2 that the toml entry is found by the rename when cargo_metadata reports one. Add a negative fixture renamed_local_dependency_version_fails with an entry such as foo2 = { package = "foo", version = "1" }. |
| inv-4 | invariants | should-fix | The proptest strategy rarely hits strings next to a verb name (a unique prefix such as s or serv, an extension such as servers), and it cannot produce a case variant such as Server. Only clap's default of no subcommand inference makes those exit 2. A probe confirms that the Decision 3 grammar rejects unknown names, but no fixed case pins the near-verb edge. | tasks.md > 2. Command grammar — "The strategy is the pattern `[a-z][a-z0-9_]{0,15}` with the verb names removed." | Add fixed cases next to help_is_not_a_verb for a unique prefix (mu s, mu serv), an extension (mu servers) and a case variant (mu Server), each expecting exit status 2, empty stdout and the string on stderr. |
| inv-5 | invariants | should-fix | The requirement says each verb has a one-line description, and a doc comment on each variant provides it. No planned test passes the absent case: help_lists_verbs checks only the verb names, and help_for_one_verb checks the description of server alone. A later verb with no doc comment would pass every test. | specs/cli-entrypoint/spec.md > Requirement: One binary with verbs — "Each verb SHALL have a one-line description in the help output." | In task 2.4, make help_lists_verbs walk the subcommands of the command definition in the library target and assert that each has a non-empty about text that appears on its line in mu --help. |
| inv-6 | invariants | should-fix | The crate layout view must equal the allowlist in the boundary test, and nothing in this change compares the two. Task 6.4 only opens an issue. | design.md > 7. Architecture model: LikeC4 in `architecture/` — "Nothing compares the two in this change, so the copy is unenforced." | Add a task that enforces the copy now, for example a test that reads the relations of the crate layout view from the LikeC4 source (or from the LikeC4 JSON export in the docs job) and compares them with the allowlist. Otherwise record the gap as accepted in Decision 7. |
| inv-7 | invariants | should-fix | CI must take every toolchain version from the pin files and never state one again. The design says that no step enforces this. | design.md > 4. Toolchain pins — "A workflow that states a toolchain version a second time is a defect, and a reviewer rejects it. No step enforces that r" | Add a check to the CI workflow (or the boundary test) that fails when .github/workflows/ci.yml has a toolchain, node-version or pnpm version input. Or record the rule as review-only in an accepted risk. |
| inv-8 | invariants | should-fix | Each crate must inherit version, edition (2024) and repository from [workspace.package]. The boundary test checks the license value only, and the design leaves the other inherited fields to review, so a crate with a local edition = "2021" passes every test. | design.md > 1. A virtual Cargo workspace with four crates — "The other inherited package fields are a review rule." | Add a rule to the violation function: a member whose manifest does not set version, edition and repository with workspace = true. Add one negative fixture for it, such as local_edition_fails. |
| inv-9 | invariants | should-fix | LICENSE must hold the unchanged Apache-2.0 text, but task 1.1 checks only two substrings, so a truncated or edited text passes. | design.md > 10. License change — "`LICENSE` receives the full Apache-2.0 text, unchanged." | In task 1.1, compare LICENSE byte for byte (or by SHA-256) with the canonical text at https://www.apache.org/licenses/LICENSE-2.0.txt. |
| app-3 | approach | note | The mitigation for the two-toolchain cost is a claim that the Rust build needs no Node. No task tells a contributor this. The build page and the clean-clone check run the site build along with the Rust build. | design.md > Risks / Trade-offs — "[Two toolchains, Rust and Node, raise the cost for a contributor] → The Rust build never needs Node. Only the `docs` job" | Make task 5.2 say that the build page separates the Rust commands, which need no Node, from the documentation commands, so that the mitigation names a deliverable. |
| fit-4 | fit | note | About seventeen fixture workspaces each repeat the root with its three shared tables and the member manifests. They differ only in one manifest edit each. | design.md > 2. A test enforces the workspace rules — "Each fixture root has the three shared tables from Decision 1, because Cargo refuses a member that inherits from a table" | The parameter is the one manifest edit of each case. Keep one allowed-edges fixture. Have each negative test copy it into a temporary directory and apply its named edit, so a change to a shared table edits one fixture. |
| spe-1 | spec-lint | note | Tasks 4.4, 4.5 and 6.2 assume the LikeC4 model-check command that task 4.2 has not yet found. | design.md > 7. Architecture model: LikeC4 in `architecture/` — "Task 4.2 finds the LikeC4 command that reports model errors" | Name the fallback in tasks 4.4, 4.5 and 6.2 (the site build as the check) or state that the check command is fixed before task 4.4 starts. |

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| Help lists the verbs | 2.4 | covered |  |
| Help for one verb | 2.4 | covered |  |
| No verb | 2.4 | covered |  |
| Version verb | 2.4 | covered |  |
| Version flag | 2.4 | covered |  |
| Unknown verb | 2.4 | covered |  |
| Server verb before the server exists | 2.4 | covered |  |
| Client verb before the client exists | 2.4 | covered |  |
| A stub verb leaves the file system alone | 2.4 | covered |  |

### Not run

Plan phase: no verification runs.
