# Proposal

Tracking issue: #36.

## Why

The spikes page says how a spike starts, but not how it ends or how a production change uses it. The review of pull request 32 found a defect in the spike code after the merge. No rule sent a later change to that review, and no rule stopped a later change from copying the spike code with its defect.

## What Changes

- The Contributing page for spikes gets two parts: how to run a spike, and how a production change uses one.
- Running a spike: the issue, the branch and the crate, and evidence that a reader can make again. Also the decision page, the triage of the review before the merge, and follow-up issues.
- Using a spike: the change cites the decision page and the pull request of the spike, and reads each review thread. Each item under Not tested becomes a task or a non-goal. The change writes new code under its own tests and does not copy spike code. It copies the recorded evidence into its crate as test data.
- Deleting a spike: after the last OpenSpec change that cites its decision page is archived, the crate goes. The decision page and the pull request stay as the record.
- A repository test makes sure that each crate in `spikes/` has a decision page with the right sections. The boundary test also fails when a file in `crates/` includes a file from `spikes/` at build time.
- The design rules in `openspec/config.yaml` get a rule that sends a change that builds on a spike to the spikes page.
- The output pane decision page gets a Not tested section, because it has none.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `developer-workflow`: adds a requirement for the record of each spike, a requirement that production code includes no spike file, and a requirement for the design rule.

## Impact

- `docs/src/content/docs/contributing/spikes.md`, `docs/src/content/docs/contributing/workspace.md` and `docs/src/content/docs/decisions/output-pane.md`.
- `openspec/config.yaml`, the `design` rules.
- A new test file `crates/mu-cli/tests/spike_records.rs`, a new violation in `workspace_boundary.rs`, a new test in `repo_config.rs`, and shared helpers in `crates/mu-cli/tests/support/mod.rs`.
- `proc-macro2` and `pulldown-cmark` as new dev-dependencies of `mu-cli`, through `[workspace.dependencies]`.

## Not in this change

- No change to the spike crates or their code.
- No production code that uses a spike. The first change that does will follow these rules.
- No automatic check that a change read the review threads of a spike. The plan review checks it.

## Acceptance

```bash
mise run check
```
