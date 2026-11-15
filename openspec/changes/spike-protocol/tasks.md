# Tasks

## 1. The spike records

- [x] 1.1 In `crates/mu-cli/tests/support/mod.rs`, add `repo_file` (moved from `repo_config.rs`), `workspace_metadata(manifest)` (the `cargo_metadata` call of `violations()`) and `files_under(dir)` (the walk of `copy_tree`). Change `repo_config.rs`, `workspace_boundary.rs` and `copy_tree` to call them. Make sure that `cargo nextest run -p mu-cli --locked` passes.
- [x] 1.2 Add a Not tested section to `docs/src/content/docs/decisions/output-pane.md`, between Reason and Source. Take each item from issue 3 and `spikes/output-pane`: what the question asked for and the spike did not measure. Make sure that `mise run docs:check` passes and that the page headings are Decision, Reason, Not tested and Source.
- [x] 1.3 Add `crates/mu-cli/tests/spike_records.rs` with one function per rule over the text of the files. Read each decision page with `pulldown-cmark`. Read the spike crates with `support::workspace_metadata` on `spikes/Cargo.toml`, and take each crate path from the directory of its manifest. Add three tests:
  - `spike_crates_have_decision_pages` (scenario: each spike crate has a decision page). Passing texts: the name followed by a space, a full stop, a slash or a backtick, and at the end of the text, the name at the start of the text, and the name at the start of a wrapped line. Breaking texts: a crate on no page, a crate on two pages, a page that names only `spikes/<crate>-v2`, a page that names only `spikes/<crate>.v2`, a page that names only `oldspikes/<crate>`, a page that names the crate in Reason and not in Source, and a page whose name of the crate is in a section after Source.
  - `spike_pages_have_sections_in_order` (scenario: the decision page of a spike has the sections in order). Breaking texts: a page with no Not tested heading, a page with the headings in another order, a page with a fifth heading `## Results`, and a page with `## Source` twice.
  - `spike_sources_name_issue_and_pull_request` (scenario: the Source section names the issue and the pull request). Breaking texts: a Source section with no pull request, one with no issue, `the spike of the issue, merged in pull request 5`, `the spike of issue 5, merged in a pull request`, and a page whose Reason holds `pull request 5` while Source holds only `issue 3`.

  Each test also runs its function on the real crates and pages. Make sure that `cargo nextest run -p mu-cli --test spike_records --locked` passes.

## 2. The include boundary

- [x] 2.0 Read the current documentation of `proc-macro2` (`TokenStream::from_str`, `TokenTree`, `Literal`) of `pulldown-cmark` (`Parser`, heading events) and of `syn` (`Lit`, `IdentExt::unraw`) with Context7. Add the three to `[workspace.dependencies]` and as dev-dependencies of `crates/mu-cli`. Make sure that `mise run deny` passes.
- [x] 2.1 Add `Violation::SpikeInclude { file }` and `Violation::Untokenized { file }` to `crates/mu-cli/tests/workspace_boundary.rs`. Read each `.rs` file in the member directories, listed with `support::files_under`, as `proc-macro2` tokens by the rules of the design. Add the check to the real-tree test `real_workspace_has_no_violation`. Add the test `spike_include_fails` (scenario: no include of a spike file). It writes fixture files into a `TreeCopy`, each from a plain string literal. The fixtures:
  - one failing file for each of `include!`, `include_str!` and `include_bytes!`
  - one failing file with `#[path = "../../spikes/x.rs"]` before `mod x;`, and one with `#[cfg_attr(all(), path = "../../spikes/x.rs")]`
  - one failing file with a space before the bracket, one with the path on the next line, one with `include_str!(concat!("../../spi", "kes/x"))`, one with `#[path = "../../spikes"] mod x { }`, one with `assert_eq!(include_str!("../../spikes/x"), "")`, one with a `path` inside a `cfg_attr` inside another `cfg_attr`, one with a raw string, one with a raw string in `#[path]`, one with an escape such as `\x73` in the path, one with `r#include_str!`, and one file that does not tokenize, which gives `Untokenized`
  - one passing file that holds `spikes/` in a string with no macro
  - one passing file with `include_str!("data.txt");` followed by `const B: &str = "spikes/";`
  - one passing file with an include of a path outside `spikes/`, and one with `include_str!("../oldspikes/x.txt")`

  Make sure that `cargo nextest run -p mu-cli --test workspace_boundary --locked` passes.
- [x] 2.2 Add the new failure to the list in `docs/src/content/docs/contributing/workspace.md`, under "The boundary test". Make sure that `mise run docs:check` passes.

## 3. The plan rule

- [x] 3.1 Add a `design` rule to `openspec/config.yaml` for a change that builds on a spike. The rule tells the design to follow the part "Using a spike" of the spikes page, and does not repeat its steps. Add the test `design_rules_cover_spikes` (scenario: the design rules name the spikes page) to `repo_config.rs`, and read `rules.design` with `yaml()`. Its breaking texts put `builds on a spike` and `Using a spike` in two different rules, leave out `Using a spike`, and leave out `builds on a spike`. Make sure that `cargo nextest run -p mu-cli --test repo_config --locked -E 'test(=design_rules_cover_spikes)'` passes.

## 4. The spikes page

- [x] 4.1 Rewrite `docs/src/content/docs/contributing/spikes.md` with the parts Running a spike, The decision page, Using a spike and Deleting a spike, from the proposal. Name the tests from 1.3, 2.1 and 3.1 where the page states a rule that they enforce. Say that the results of a spike go under Reason and that a page has only the four sections. Say that review covers reads at run time and an include through a macro with a new name. Make sure that `mise run docs:check` passes, and that `uv run ~/.agents/plugins/sb/skills/review-lint/scripts/review_lint.py --base origin/main` exits 0.
- [ ] 4.2 Run `/sb:openspec-review spike-protocol implementation` and treat changes-requested as a blocker. Make sure that `review.md` has the verdict approve.
- [ ] 4.3 Run `mise run check`. Make sure that it passes.

## Workflow follow-up

- After the merge, archive the change with the post-merge skill. The acceptance command is `mise run check`.
