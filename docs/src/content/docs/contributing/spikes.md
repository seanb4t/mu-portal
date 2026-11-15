---
title: Spikes
description: How to run a time-limited experiment, and how a production change uses its result.
sidebar:
  order: 3
---

A spike is a time-limited experiment that answers one question. Spike code lives in `spikes/`, which is a separate Cargo workspace. The release build and CI do not compile it.

## Running a spike

- The GitHub issue of the spike states the question, the time limit and the end product. A spike answers one question.
- The work goes on the branch `spike/<name>`, in the crate `spikes/<name>`. The name comes from the issue, for example `spikes/output-pane`.
- A spike can depend on a crate in `crates/` by path. No crate in `crates/` can depend on a spike. The boundary test enforces this rule.
- The evidence is in the repository: the results, the recorded data and the command that makes them again. Each outside server runs from a Dockerfile in the spike crate, and each version pin in it is one that Renovate reads.
- The spike ends in a written decision on the issue. The pull request adds the decision page and says `Closes #<n>`.
- Before the merge, triage each review thread of the pull request. Fix each finding in the spike, or list it under Not tested on the decision page.
- Each follow-up becomes an issue of its own.

## The decision page

Each spike crate has one page in the Decisions section. The page has these sections, in this order, and no other:

- Decision: what the project will do.
- Reason: the measurements and the results that support the decision. Results go here, not in a section of their own.
- Not tested: what the question asked for, or implied, and the spike did not measure.
- Source: the issue, the pull request and the crate of the spike, and the commands that make the evidence again.

The tests in `crates/mu-cli/tests/spike_records.rs` enforce the page. `spike_crates_have_decision_pages` makes sure that the Source section of exactly one page names each crate. `spike_pages_have_sections_in_order` checks the sections. `spike_sources_name_issue_and_pull_request` makes sure that Source names the issue and the pull request, each with its number.

## Using a spike

A production change that builds on a spike does these things. The design rule in `openspec/config.yaml` points the design of the change to this list, and the plan review checks it. The test `design_rules_cover_spikes` makes sure that the rule exists.

1. Cite the decision page and the pull request of the spike in the design.
2. Read each review thread of that pull request. A thread can hold a defect that the spike code still has.
3. Turn each item under Not tested into a task or a non-goal of the change.
4. Write new code under the tests of the change. Do not copy spike code. Read the spike code as an example only.
5. Copy the recorded evidence, for example transcripts and results, into the crate of the change as test data. Do not put it in a directory named `spikes`, because the boundary test fails an include of a path with that name.

No file in `crates/` can include a file from `spikes/`. The [boundary test](/contributing/workspace/#the-boundary-test) enforces this rule. Review covers two cases that the test cannot see. One is a read of a spike file at run time, for example with `fs::read`. The other is an include through a macro with a new name.

## Deleting a spike

Spike code is evidence for its decision. It stays on the main branch while a change depends on that evidence. After you archive the last OpenSpec change that cites the decision page, delete the crate. If the change deletes the last spike crate, it also removes the check in `spike_crates_have_decision_pages` that the spikes workspace has crates. The decision page and the pull request stay as the record. Review checks this at the archive of that change.
