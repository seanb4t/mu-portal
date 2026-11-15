---
verdict: approve-with-notes
phase: plan
reviewed_at: 2026-11-15T00:16:20+00:00
head: a0f7a3f642f781ba4f2336740e675f0421e2c147
lenses:
  spec-lint: ok
  approach: partial
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
---
# Review

## Incomplete lenses

- approach: partial, gaps: 1

The verdict covers only what the lenses checked.

## Findings

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

## Coverage matrix

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

## Verification runs

None.

## Dropped

None.

## Lens notes

- approach: All five cli-entrypoint requirements have a producing decision in Decision 3, and no scenario THEN needs a step or field that the design does not create.
- approach: Repository facts in the Context section hold at head: README, MIT LICENSE, a Node-template .gitignore, the OpenSpec scaffold, two commits on main, and a remote ref renovate/configure.
- approach: The clap behaviors cited in Decision 3 (default name from CARGO_PKG_NAME, disable of the help subcommand, args_conflicts_with_subcommands in the git-derive stash example, render_long_version with its trailing newline, exit status 2 on usage errors) were checked from knowledge of clap 4. No Cargo.lock or vendored clap exists at head to cite.
- approach: The cargo_metadata claims in Decision 2 (no_deps writes no lock file, the dependency name is the package name with rename separate, path dependencies have no source) and the Cargo claim that inheriting from an absent workspace table fails were checked from knowledge of Cargo, with no pinned source at head.
- approach: The Renovate config:best-practices preset is known to include helpers:pinGitHubActionDigests, so the conditional in Decision 4 and task 6.3 will likely resolve to no extra preset. This was not read from the Renovate documentation.
- approach: Decision 2 says 'Two kinds of test use the function' and then lists three bullets. This is an internal wording inconsistency for the spec-lint or fit lens.
- approach: The Decision 1 table gives mu-cli the kind 'binary mu', while Decision 3 adds a library target to mu-cli. The table does not show the library target.
- fit: openspec/config.yaml has no context block, so the repository adds no library-preference rows. Only the generic rows were applied.
- fit: At head the repository holds only README.md, LICENSE, a Node-template .gitignore and the OpenSpec scaffold, so there is no existing code pattern for most components to deviate from.
- fit: Decision 2 names cargo-deny as the alternative and gives a reason, and Decision 3 names the clap git-derive default-subcommand example. Neither is reported as bespoke-over-library.
- fit: Decisions 4, 5, 8, 9, 10 and 11 and the five cli-entrypoint requirements were read. No fit finding came from them: the rules have one owner page, and the version flag and verb share one source, as do the default verb and client.
- invariants: Plan phase: the repository at head has no Rust, docs or CI code. Every enforcement point is planned, not present.
- invariants: I1: Every workspace edge is in the Decision 1 allowlist \| enforced at planned crates/mu-cli/tests/workspace_boundary.rs (task 1.5), path dependencies only (see inv-1) \| boundary test allowed_edges_have_no_violation plus the tui_to_core, core_to_tui and proto fixtures (task 1.6)
- invariants: I2: Every member is named in the allowlist \| enforced at workspace_boundary.rs \| boundary test unlisted_member_fails, real_workspace_has_no_violation
- invariants: I3: No crate in crates/ depends on a spike \| enforced at workspace_boundary.rs path rule and root exclude \| boundary test spike_dependency_fails. A probe confirmed that a path dependency into a non-excluded directory under the root becomes a member, as Decision 2 says.
- invariants: I4: No path dependency on a non-member outside spikes \| enforced at workspace_boundary.rs \| boundary test outside_path_dependency_fails. A probe confirmed that cargo metadata --no-deps reports an excluded path crate with a null source and its path.
- invariants: I5: Every member has license Apache-2.0 \| enforced at workspace_boundary.rs \| boundary tests missing_license_fails, wrong_license_fails (both sides of the equality)
- invariants: I6: Every member has [lints] workspace = true \| enforced at workspace_boundary.rs \| boundary tests missing_lints_fails, local_lints_fail
- invariants: I7: Every third-party dependency uses workspace = true \| enforced at workspace_boundary.rs via toml \| boundary test local_dependency_version_fails only (see inv-2, inv-3)
- invariants: I8: The boundary function writes no lock file into a fixture \| enforced by no_deps \| a probe confirmed that cargo metadata --no-deps writes no Cargo.lock. no_deps also matters for proto_dependency_fails: a probe showed that --no-deps accepts a proto-core cycle (exit 0) while a resolving cargo metadata fails with exit 101 on a cyclic package dependency.
- invariants: I9: mu --help lists exactly client, server, version \| enforced at clap disable_help_subcommand \| boundary test help_lists_verbs. A probe of the Decision 3 grammar with clap_builder 4.6.7 listed exactly the three verbs.
- invariants: I10: mu with no verb equals mu client \| enforced by construction (None calls the client function) \| boundary test no_verb_equals_client. The probe gave identical stderr and exit 1 for both.
- invariants: I11: mu version and mu --version print the same single line \| enforced at Command::render_long_version shared source \| boundary tests version_verb, version_flag_matches_verb. The probe printed mu 0.1.0 for version, --version and -V, also under a different argv0.
- invariants: I12: Unknown verb exits 2, empty stdout, names the verb on stderr \| enforced at clap parse \| boundary test unknown_verb_property, help_is_not_a_verb (see inv-4). The probe gave exit 2 and empty stdout for frobnicate, help, a, --frob and an extra argument.
- invariants: I13: A stub verb writes verb: not implemented, exits 1 and touches no file \| enforced at the single stub function (task 2.2) \| boundary tests server_not_implemented, client_not_implemented, stub_verbs_leave_files_alone
- invariants: I14: Each verb has a one-line description \| enforced at doc comments \| boundary test none for the absent case (see inv-5)
- invariants: I15: The Usage line starts with mu \| enforced by argv0, not by the command name: the probe printed Usage: foo when run through a symlink named foo. The tests pass because they run target/debug/mu, so Decision 3 credits the explicit name with an effect that comes from the binary name.
- invariants: I16: Each CI action is pinned by commit SHA \| enforced by Renovate digest pinning (Decision 4, task 6.3) and review \| boundary test none. Renovate is a named guard, so this is not reported as unenforced.
- invariants: I17: An Archify diagram in docs has its candidate JSON and SVG in one commit \| enforced at none (review guidance) \| boundary test none. It holds vacuously because this change adds no Archify diagram, and Decision 8 defers the step to the first change that adds one.
- invariants: I18: .archify/ and target/ never enter git \| enforced at .gitignore (task 1.7) \| boundary test is the task 1.7 git status check, not a repeatable test.
- invariants: No property-candidate finding: the only invariant over an unbounded domain (unknown verb) already has a proptest property in task 2.5.
- invariants: Probe inputs are under probe-inv/ in the handoff directory: a scratch workspace for cargo metadata and a minimal clap build of the Decision 3 grammar.
- spec-lint: Every scenario in specs/cli-entrypoint/spec.md has a WHEN or GIVEN line and a THEN line; every requirement has at least one scenario.
- spec-lint: Every requirement heading matches a task line with two or more distinctive words (tasks 2.1, 2.2, 2.3, 2.4, 2.5).
- spec-lint: No BREAKING marker appears in proposal.md or in the delta spec; the two agree.
- spec-lint: unknown-target was judged against the change itself: the build targets named in tasks.md (cli, workspace_boundary, the docs build) are created by the change's own tasks, so no existing target was checked against the repository tree.
- test-coverage: All nine scenarios share one task, 2.4, which names a test per scenario and one verification command.
- test-coverage: The change has no BREAKING requirement, so no breaking-unproven finding applies.
- test-coverage: The license replacement is stated as having no spec requirement; tasks 1.1 and 1.6 (missing_license_fails, wrong_license_fails) prove it, and no orphan test results.
- approach gap: The LikeC4 documentation was not read, so the mdBook rejection in Decision 6 (LikeC4 describes a Starlight integration and none for mdBook) and the Starlight setup cited in Decision 7 are not checked against it. This lens has no network tool.
