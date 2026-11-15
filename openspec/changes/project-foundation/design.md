# Design

## Context

See proposal.md for the motivation. The `main` branch has two commits, both by the owner: the initial commit and the commit that added the OpenSpec scaffold. The repository tracks a README, an MIT license, a `.gitignore` from the GitHub Node template and the OpenSpec scaffold. No code exists. The remote also has a `renovate/configure` branch from the Renovate onboarding, which is not merged.

Constraints from the exploration:

- The product is one server and zero to many clients. A client reaches the server only through the API.
- The server must run on Linux and on macOS, in a container or not.
- One binary, `mu`, carries every function as a verb.
- Documentation is a deliverable. LikeC4 owns structural diagrams. Archify draws the rest.
- The license is Apache-2.0.

## Goals / Non-Goals

**Goals:**

- A workspace layout that makes the client and server boundary a compile-time fact.
- A command grammar that later changes extend without a rename.
- A documentation site and an architecture model that build in CI from the first day.
- A place for spikes that cannot leak into a release.

**Non-Goals:**

- Any behavior behind the verbs. The verbs report that they are not implemented.
- The content of the API schema.
- Hosting or publishing the documentation site.
- Release packaging, signing and container images.
- Alignment with the Open Knowledge Format (see Decision 9).

## Decisions

### 1. A virtual Cargo workspace with four crates

The root `Cargo.toml` is a virtual workspace. The crates live in `crates/`:

| Crate | Kind | Holds | Allowed workspace dependencies |
|---|---|---|---|
| `mu-proto` | library | API schema and generated code (empty in this change) | none |
| `mu-core` | library | server engine (empty in this change) | `mu-proto` |
| `mu-tui` | library | terminal client (empty in this change) | `mu-proto` |
| `mu-cli` | binary `mu` | command grammar and verb dispatch | `mu-proto`, `mu-core`, `mu-tui` |

This table is the allowlist of workspace edges. An edge that the table does not list is forbidden. `mu-tui` therefore cannot reach `mu-core`, directly or through another crate. That rule is the boundary decision from the exploration. A crate boundary lets the compiler and a test enforce it. Modules inside one crate cannot.

The root manifest holds three shared tables, and each crate inherits from them:

- `[workspace.package]`: version, edition, `license = "Apache-2.0"` and repository.
- `[workspace.lints]`: the lint levels.
- `[workspace.dependencies]`: every third-party dependency and its version. A crate names a dependency with `workspace = true`. A later change that shares a dependency between crates then edits one table.

The test from Decision 2 enforces the license, the lints table and the dependency table. The other inherited package fields are a review rule.

Alternative: one crate with modules. Rejected, because nothing stops a client module from importing a server module.

Alternative: a separate binary for the server. Rejected by the owner in favor of one binary with verbs.

### 2. A test enforces the workspace rules

`crates/mu-cli/tests/workspace_boundary.rs` holds a function that takes a workspace root and returns a list of violations. It uses the `cargo_metadata` crate to read the packages and the `toml` crate to read each manifest. Both are development dependencies of `mu-cli`.

The function calls `cargo_metadata` with the `no_deps` option, so the call writes no lock file into a fixture. An edge is named by the package name that `cargo_metadata` reports, not by the key in the manifest. A renamed dependency is therefore still found.

The function reads the declared dependencies of each package, not the resolved graph. Declared dependencies include optional ones and all three kinds: normal, build and development. The resolved graph leaves out an optional dependency behind a feature that is off.

The function sorts each declared dependency of a member into one of three classes:

- A member dependency is every other dependency that names a workspace member or a crate in the allowlist. The source does not matter: a path, a git URL and a registry all count. The edge rule applies to it.
- An outside path dependency has a path that is not the directory of a workspace member. The spike rule or the outside-path rule applies to it. The path decides before the name, because a crate outside the workspace can carry the name of a member.
- A third-party dependency is every other dependency. The dependency-table rule applies to it.

Each dependency therefore breaks one rule at most. The dependency-table rule reads the manifest alone and uses the key of each entry, so a renamed entry is still found.

The function reports a violation for each of these:

- A workspace member that the allowlist in Decision 1 does not name.
- A member dependency that the allowlist does not list.
- A path dependency whose path is under `spikes/`. A spike is found by its manifest path, because a spike is not a workspace member.
- A path dependency on a crate that is not a member and is not under `spikes/`. The `no_deps` option hides the dependencies of such a crate, so a member can reach a forbidden crate through it.
- A member whose license is not `Apache-2.0`.
- A member whose manifest lacks `[lints]` with `workspace = true`.
- A third-party dependency of a member that does not use `workspace = true`.

The allowlist in the test file is the single source of the rule. The table in Decision 1 and the LikeC4 crate layout view are copies of it.

Three kinds of test use the function:

- One test runs it on the real workspace and expects no violation.
- One test runs it on a fixture workspace that declares all five allowed edges and expects no violation.
- One test for each negative case runs it on an edited copy of that fixture. It expects exactly one violation, of the rule that the edit breaks.

One fixture workspace is on disk, at `crates/mu-cli/tests/fixtures/base/`. It declares all five allowed edges. Its root has the three shared tables from Decision 1, because Cargo refuses a member that inherits from a table that is absent. It lists `spikes` and `outside` under `exclude`. Without those entries, Cargo makes a crate in either directory a member, and a negative case breaks two rules.

Each negative test copies the base fixture into a temporary directory and makes one edit. The negative cases therefore differ from the base by that edit alone, and the repository holds one fixture and not one for each case.

The negative cases are:

- a `mu-tui` to `mu-core` dependency of each kind: optional, build, development, renamed, target-specific and by git URL
- a third-party dependency with its own version and no `workspace = true`, in each of these forms: normal, development, target-specific and renamed
- a `mu-core` to `mu-tui` dependency
- a `mu-proto` dependency on a member
- a path dependency into `spikes/`, and one on a spike that carries the name of a member
- a path dependency on a crate in an excluded directory that is not `spikes/`
- a member that the allowlist does not name
- a member with no license, and a member with the license `MIT`
- a member with no `[lints]` table, and a member with a local `[lints.rust]` table and no `workspace = true`

The tests make each negative case permanent. Nobody has to break the real workspace by hand to prove the rule.

Alternative: a `cargo-deny` ban. Rejected for now, because it adds a tool and cannot express two of the rules: the member allowlist and the inherited `[lints]` table. If `cargo-deny` arrives later, that change can move the dependency and license rules to it.

### 3. Command grammar with `clap`

`mu-cli` defines the grammar with `clap` derive.

- The command name is set to `mu` explicitly. The default name is the package name, `mu-cli`, which the spec does not allow.
- Each verb is a variant with a doc comment. `clap` uses the doc comment as the one-line description in `mu --help` and in the help of the verb.
- The built-in `help` subcommand is off, so the commands section lists exactly the three verbs. The `--help` flag stays.
- The top-level command has an optional subcommand. If the subcommand is absent, the code calls the same function that `client` calls. That makes the default-verb requirement true by construction.
- The arguments of the client are one `clap` arguments struct, which is empty in this change. The `client` verb holds that struct. The top-level command flattens the same struct and sets `args_conflicts_with_subcommands`. This is the default-subcommand shape from the `git-derive` example in the `clap` repository. A later change adds a client flag to that one struct. `mu --flag` and `mu client --flag` then both accept it, and the top-level command needs no new structure.
- The `version` verb prints the string that `clap` renders for `--version`, through `Command::render_long_version`. The verb adds no second newline. The flag and the verb share that one source. With the name `mu` and the package version, the string is the line that the spec requires.
- One function takes the name of a verb, writes `<verb>: not implemented` to stderr and returns exit status 1. Every stub verb calls it with its own name. The function opens no file.
- `mu-cli` has a small library target that holds the command definition. The binary and the tests both use it. The property test reads the verb names from that definition, so a later verb needs no edit in its exclusion list. The scenario tests name the three verbs, as the spec scenarios do. A change that adds a verb edits that spec scenario and its test together.

Exit statuses follow the spec: 0 for help and version, 2 for a usage error (the `clap` default), 1 for a verb with no implementation. The code uses the `clap` parse call that exits by itself on help, version and usage errors.

Alternative: hand-written argument parsing. Rejected, because later verbs need flags, help text and shell completion.

### 4. Toolchain pins

- `rust-toolchain.toml` pins a stable Rust release, with `rustfmt` and `clippy`. The release is the current stable one on the day the change is built.
- The workspace uses edition 2024.
- `docs/.node-version` pins Node. `docs/package.json` pins `pnpm` through its `packageManager` field.

CI reads these files. A workflow that states a toolchain version a second time is a defect, and a reviewer rejects it. No step enforces that rule. The Rust job lets `rustup` read `rust-toolchain.toml`. The docs job gives `docs/.node-version` to the Node setup action. It gives the path `docs/package.json` to the `pnpm` setup action, which reads `packageManager` from that file. Task 6.2 takes the name of that input from the current documentation of the action.

This change adds `renovate.json`, because the Renovate onboarding is still open. The configuration extends the Renovate best-practices preset. Task 6.3 reads the preset. If the preset does not pin GitHub Actions to commit digests, the task adds the digest preset. Task 6.5 makes sure that Renovate lists each pin.

### 5. Spikes are a second workspace

`spikes/` has its own `Cargo.toml` workspace. Its member list is the glob `*`, so each spike crate is a member with no edit to that file. It excludes `target`, because the glob also matches the build directory after the first build. The root workspace lists `spikes` under `exclude`. A spike is one crate in `spikes/`, named after its GitHub issue. A spike can depend on a crate in `crates/` by path. No crate in `crates/` can depend on a spike. The test from Decision 2 enforces that rule.

The rules for a spike are written once, on the Contributing page for spikes. `spikes/README.md` is a short pointer to that page.

CI does not build `spikes/`. A spike ends in a written decision, and its code can then be deleted.

Alternative: spikes as branches. Rejected, because a spike's code is evidence for its decision and must stay readable on the main branch for a while.

### 6. Documentation site: Astro Starlight in `docs/`

- Pages are Markdown. If a page embeds a component, the page is MDX.
- The site has four sections at the start: Overview, Decisions, Architecture and Contributing.
- The Contributing section covers the build, the workspace layout, spikes, the diagram policy, and how OpenSpec and the GitHub Project divide the work.

The Decisions section is the durable record of project-wide decisions. Each decision is one short page that states the decision and the reason, and names the OpenSpec change that made it. A Decisions page holds no working rules. If a decision has rules that a contributor follows, the rules are on one Contributing page, and the Decisions page links to it. The diagram policy follows that pattern. The spikes have a Contributing page and no Decisions page, because their rules are working rules and not a product decision. The `design.md` of a change is the working record while the change is open, and it becomes history after the archive. A later change that makes a project-wide decision adds or edits a Decisions page in the same change. The pages of this change name the first exploration as their source.

The site comes from the Starlight starter and keeps the starter's `docs/.gitignore`. That file ignores the output that Astro generates.

The README stays short. It names the project, states its status and links to the Contributing pages.

Alternative: mdBook. Rejected, because the LikeC4 documentation describes an integration with Starlight and none with mdBook.

Alternative: EmDash. Rejected, because it is a CMS with a database, not a documentation tool.

### 7. Architecture model: LikeC4 in `architecture/`

The model lives in `architecture/` as LikeC4 source files. The first model has three views:

- system context: the person, the clients, the server and the games
- server and clients: the paired devices, the API and the parts of the server
- crate layout: the four crates and their allowed dependencies

The documentation site loads the model through the LikeC4 Vite plugin. Each view is embedded on an MDX page in the Architecture section. The LikeC4 documentation shows this setup: the plugin in the Astro configuration, and a small Astro component that wraps the React view from `likec4:react`.

CI must fail on a model error. Task 4.2 finds the LikeC4 command that reports model errors and records it here. If no such command exists, the site build is the check. Task 4.5 proves the chosen check with a deliberate model error.

Task 4.2 read the LikeC4 documentation and found both details:

- The Vite plugin takes the model directory in its `workspace` option. The Astro configuration sets it to `../architecture`.
- The model check is `likec4 validate <directory>`. It exits with a non-zero status on a model error. `docs/package.json` holds it as the script `model:check`, so CI and a contributor run `pnpm model:check`.

The crate layout view is a copy of the allowlist in the boundary test. Nothing compares the two in this change, so the copy is unenforced. Task 6.4 opens the issue that tracks the collapse: generate the relations of the view from the allowlist. Until that issue lands, the Contributing page tells the reader to change both together.

### 8. Diagram policy

LikeC4 owns every structural diagram. Archify draws state machines, data schemas, timelines and explainer pages. For a sequence, a LikeC4 dynamic view comes first.

The Contributing page for the diagram policy is the single owner of these rules. The Decisions page for the diagram policy states the decision and the reason, and links to that page.

Archify is an agent skill and is not part of the build. If an Archify diagram enters the documentation, its candidate JSON and its exported SVG go into `docs/src/assets/diagrams/` in the same commit. That pairing is guidance for a reviewer. No step enforces it, because this change adds no Archify diagram. The first change that adds one also adds the enforcing step.

### 9. No Open Knowledge Format alignment in this change

OKF reads `.md` files only and gives `index.md` a reserved meaning, normally with no frontmatter. Starlight needs a title in the frontmatter of every page. The two formats do not share one directory tree without extra tooling.

This change does not add an OKF bundle. Pages stay Markdown by default, which keeps a later conversion cheap. A parked GitHub issue records the idea.

### 10. License change

`LICENSE` receives the full Apache-2.0 text, unchanged. `[workspace.package]` sets `license = "Apache-2.0"`, and the test from Decision 2 fails for a crate without it. The owner is the only author on `main`, so the change needs no agreement from another person.

The license is a property of the repository, not a behavior of the `mu` binary. It has no requirement in the `cli-entrypoint` spec. Task 1.1 and the license rule of the boundary test are its checks.

### 11. CI on GitHub Actions

One workflow, `.github/workflows/ci.yml`, runs on a pull request and on a push to `main`. It has two jobs:

- **rust**, on a matrix of Linux and macOS: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- **docs**, on Linux, in `docs/`: `pnpm install --frozen-lockfile`, the model check that task 4.2 chooses (see Decision 7), `pnpm build`.

Each action is pinned by commit SHA with its version in a comment. The author of the workflow reads the current documentation of each action and takes the version from there. Renovate keeps the pins as digests through the preset from Decision 4. No CI step rejects a tag reference. Review and Renovate are the two guards.

## Risks / Trade-offs

- [The LikeC4 Vite plugin needs React inside Astro] → The LikeC4 documentation shows this setup with Starlight. Task 4.3 proves it with one embedded view before more pages depend on it.
- [Two LikeC4 details are unknown. One is the command for a model check. The other is the plugin option for the model directory.] → Task 4.2 reads the current LikeC4 documentation before the model is written. Task 4.5 proves that the chosen check fails on a model error.
- [Renovate does not see one of the pin files] → Task 6.5 reads the Renovate dependency list after the configuration is merged. A pin that Renovate does not list gets a custom manager in the same task.
- [Two toolchains, Rust and Node, raise the cost for a contributor] → The Rust build never needs Node. Only the `docs` job and a documentation author need it.
- [The crate layout view and the boundary test can disagree] → The test is the source and fails the build. The view is a picture. Task 5.2 names both on the Contributing page, and task 6.4 opens the issue that tracks the collapse.
- [Stub verbs are behavior that later changes remove] → A later change that implements a verb modifies the not-implemented requirement in its own delta. Task 5.2 states that rule on the Contributing page for OpenSpec.
- [macOS runners are slower and cost more] → The matrix stays, because the server must build on macOS. The docs job runs on Linux only.

## Migration Plan

The change adds files and replaces `LICENSE`. Rollback is a revert of the commits. No data or deployed system exists.

A revert also removes `renovate.json`. Renovate then stops its update requests and opens its onboarding request again. The owner closes any update request that is still open.

## Open Questions

- Where the documentation site is hosted. No task in this change depends on the answer.
