# Design

## Context

See proposal.md, Why. The Contributing page for spikes owns the rules, and this change rewrites it. The spec delta holds only the parts of those rules that a test can decide on a file of the repository.

Three spike crates exist: `spikes/output-pane`, `spikes/char-width` and `spikes/telnet-engine`. Each decision page names its pull request in Source since pull request 35. The output pane page has no Not tested section.

Three test files hold the rules of the repository now:

- `repo_config.rs` reads the CI and tool configuration. Each rule is a function over the text of a file. A test gives it the real file and a text that breaks the rule. Its private helper `repo_file` reads a file from the repository root.
- `workspace_boundary.rs` reads the Cargo workspace with `cargo_metadata` and reports each break of the rules as a `Violation`. Its fixtures are copies of a small workspace (`TreeCopy`) that a test breaks in one place.
- `support/mod.rs` holds the shared helpers, for example `repo_root()` and `TreeCopy`. Its private `copy_tree` walks a directory tree.

## Goals / Non-Goals

Goals:

- A later change finds the review threads of a spike from its decision page.
- No crate in `crates/` can include a spike file at build time.

Non-Goals:

- A check that a change read the review threads. A tool cannot decide that a person or an agent read a thread.
- A check of the time to delete a spike crate. The rule depends on the archive of a later change, which no file records in a form that a test can read.
- A check of reads of spike files at run time. A path for a read at run time can come from a variable, so a text search cannot decide it.

## Decisions

### What enforces each rule

| Rule on the spikes page | Enforced by |
|---|---|
| The issue states the question, the time limit and the end product | Review alone |
| Branch `spike/<name>`, crate `spikes/<name>` | Review alone |
| Evidence is committed with the command that makes it again, and each outside server is a Dockerfile with pins that Renovate reads | Review alone. The review-lint skill flags a pin that Renovate cannot see |
| Each crate has a decision page with the four sections, and Source names the crate, the issue and the pull request | The tests `spike_crates_have_decision_pages`, `spike_pages_have_sections_in_order` and `spike_sources_name_issue_and_pull_request` |
| The review of the spike pull request is triaged before the merge, and each finding is fixed or listed under Not tested | Review alone, with the pr-triage skill |
| The pull request body says `Closes #n` | Review alone |
| A change that uses a spike follows the part "Using a spike" | The design rule in `openspec/config.yaml` and the plan review. The test `design_rules_cover_spikes` makes sure that the rule exists |
| Each Not tested item becomes a task or a non-goal | The plan review |
| The change copies the recorded evidence into its own crate as test data | The plan review, from the tasks of the change |
| Production code does not copy spike code | Review alone |
| Production code includes no spike file at build time | The test `real_workspace_has_no_violation` in `workspace_boundary.rs`. The test `spike_include_fails` tests the check itself |
| Production code reads no spike file at run time, for example with `fs::read`, or includes one through a macro with a new name, a symbolic link or a `stringify!` of several tokens | Review alone. The spikes page says so beside the boundary test |
| Delete the crate after the last change that cites its page is archived | Review alone, at the archive of that change |

### Where each test goes

- The three tests of the decision pages go in a new file, `crates/mu-cli/tests/spike_records.rs`. They read the docs and the spike workspace, which neither existing file covers.
- The include check is a boundary rule, so it goes in `workspace_boundary.rs` as the new violations `Violation::SpikeInclude { file }` and `Violation::Untokenized { file }`. The workspace page lists it with the other failures of the boundary test.
- `design_rules_cover_spikes` reads tool configuration, so it goes in `repo_config.rs` and reads `rules.design` with its `yaml()` helper.
- `repo_file` moves from `repo_config.rs` to `support/mod.rs`, so both `repo_config.rs` and `spike_records.rs` call it.
- Two more helpers go in `support/mod.rs`. `workspace_metadata(manifest)` runs `cargo_metadata` with `no_deps()`, and `violations()` in `workspace_boundary.rs` and `spike_records.rs` both call it. `files_under(dir)` lists the files under a directory, and `copy_tree`, the include check and the page list of `spike_records.rs` call it. `spike_records.rs` reads each page with `repo_file`.

### How the test finds a spike crate

The test calls `workspace_metadata` on `spikes/Cargo.toml` and takes `workspace_packages()`. That is the member set that Cargo reads, so the test copies no glob from `spikes/Cargo.toml`. The crate path is the directory of each package manifest, relative to the spikes workspace root, as in `workspace_boundary.rs`. A crate at `spikes/a/b` is then sought as `spikes/a/b`, so a package name that differs from its directory does not matter.

### How the test reads a decision page

The test reads each page with `pulldown-cmark`, a Markdown parser, and does not parse Markdown by hand.

- The headings are the level-two headings of the page, compared by their text.
- The Source section is the text of the events after the heading Source, up to the next level-two heading or the end of the page.
- A crate name counts only as a whole path, or as a path under it. Before `spikes/` the character must be a backtick, a space, a line end or the start of the text. After `spikes/<crate>` it must be a backtick, a slash, a space, a line end or the end of the text. A full stop also ends the path, but only if the sentence ends there. The Source text has a line end at each line break and at the end of each block: a paragraph, a list item, a list, a code block, a block quote, a table cell, a heading or an HTML block. It has no line end at the end of emphasis or a link, so a name at the start of a wrapped line counts. So `spikes/telnet-engine/transcripts` names `telnet-engine`, and `spikes/telnet-engine-v2` and `spikes/telnet-engine.v2` do not.

### How the boundary test finds an include of a spike file

The check reads each `.rs` file in the member directories that `violations()` already takes from `cargo_metadata`. It turns the text into tokens with `proc-macro2` (`TokenStream::from_str`), the token library that Rust parsers use. It then walks the tokens, also into the body of every macro and every bracket group. A token walk sees code as code, so spaces, line ends and a string that only looks like a macro do not change the result. A walk of a `syn` syntax tree does not work here, because `syn` does not look into the body of a macro such as `assert_eq!`.

- Macros: a token `include`, `include_str` or `include_bytes`, then `!`, then a group, is an include. The check compares the identifier without a raw prefix (`syn::ext::IdentExt::unraw`), so `r#include_str!` is also an include. The check joins the values of all string and char literals in that group, in order and at any depth, and the token text of each `stringify!` group, as `concat!` does. `syn::Lit` decodes each literal, so an escape such as `\x73` and a raw string give their real value. A path that `concat!` builds from parts is then one string. The token text of a `stringify!` of several tokens has spaces between the tokens, so that case is left to review.
- Attributes: a `#`, or a `#!` for an inner attribute, then a bracket group that holds `path = "..."` is a path attribute. That covers `#[path = "..."]` and `path` inside a `cfg_attr`, at any depth. The check takes only the string after `path =`, so a string in the predicate of a `cfg_attr` does not join into the path.
- A joined string fails when one of its path components is `spikes`, so `../../spikes/x.rs`, `../../spikes` and `spikes` fail, and `oldspikes/x.rs` passes. The rule for path dependencies resolves a path and tests that it starts with `<workspace root>/spikes`. The include check cannot resolve every path in the same way, because a path that `concat!` builds from `env!("CARGO_MANIFEST_DIR")` has no fixed base. So the check uses the stricter rule by component. One cost is that a directory named `spikes` inside a crate also fails. The spikes page says that copied evidence must not go in such a directory.
- If a file does not turn into tokens, it is a `Violation::Untokenized { file }`, so the check never skips a file in silence.
- `Violation::SpikeInclude { file }` names the file only. A line number would need the `span-locations` feature of `proc-macro2`, and the file is enough to find the include.

The fixtures of the check are files that the test writes into a `TreeCopy`. In the source of `workspace_boundary.rs` each fixture is a plain string literal. A string literal is one token, so the walk does not see an include in it, and the check of the real tree passes with no exception.

A macro with a new name, for example `macro_rules! inc { ... }` called as `inc!(...)`, hides the include from the check. A symbolic link from a crate into `spikes/` also hides it, because the check reads the files that the link points to as files of the crate. Review covers these cases, and the spikes page says so.

### The design rule points to the page

The rule in `openspec/config.yaml` names the part "Using a spike" of the spikes page and does not repeat its steps. The project context in that file says that the Contributing pages own the spike rules.

## Risks / Trade-offs

- [A later spike needs another heading, for example "Results"] → The spikes page says that the results go under Reason, as the three pages do now. If a spike needs another section, its change also changes the requirement.
- [`proc-macro2`, `syn` and `pulldown-cmark` are new dev-dependencies] → `proc-macro2` and `syn` are already in `Cargo.lock` through other crates, and `syn` takes no feature, because its literal types and `IdentExt` need none. All three enter through `[workspace.dependencies]` like any other crate, and `cargo deny` checks their licenses in the `check` task.
- [`cargo_metadata` on the spikes workspace is slower than a directory list] → The call uses `no_deps()`, so Cargo resolves no dependency.

## Migration Plan

- Add a Not tested section to `output-pane.md`. Its items come from issue 3 and the output pane spike: what the question asked for and the spike did not measure.
- Rewrite `spikes.md`, and add the new failure to `workspace.md`. A revert of the commits is the rollback.
