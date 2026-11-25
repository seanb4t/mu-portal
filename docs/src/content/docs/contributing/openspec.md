---
title: OpenSpec and the GitHub Project
description: Where the content of the work lives, and where its status lives.
sidebar:
  order: 5
---

Two tools hold the work. OpenSpec holds the content. The GitHub Project named "mu-portal" holds the status.

## OpenSpec

`openspec/specs/` holds the requirements of each capability. `openspec/changes/` holds each change in progress: its proposal, its spec deltas, its design, its tasks and its review.

- The file `openspec/config.yaml` holds the project context and the rules for each artifact.
- A change gets a plan review before the work starts and an implementation review before the archive.
- A change that makes a project-wide decision adds or edits a page in the Decisions section in the same change.

## GitHub Project

Each change and each spike is one issue in the Project. The status of an issue is one of Parked, Next, In progress, In review and Done.

- Only one change and one spike are in progress at a time.
- A new idea becomes a Parked issue. It does not become a task in an open change.
- The owner reads the Parked issues at the end of each phase.
- The acceptance outcome of a phase decides that the phase is done. A finished task list does not.

### What sets the status

The workflows of the Project set most of the status. You set the rest by hand.

| Event | Status |
|---|---|
| An issue opens in the repository | The Project adds it as Parked |
| A pull request links to the issue | In review |
| The issue closes | Done |
| The issue opens again | Next |
| You choose the next item | Next, by hand |
| The work starts | In progress, by hand |

A pull request names its issue with `Closes #<number>` in its body. The merge then closes the issue, and the Project moves it to Done. A pull request with no issue, such as a dependency update or an archive, says so in the body.

### Labels and milestones

Each issue has one kind label or one type label.

- Kind labels: `kind:change`, `kind:spike`, `kind:decision`, `kind:docs`, `kind:plan`, `kind:idea` and `kind:followup`.
- Type labels: `bug`, `test`, `enhancement`, `refactor`, `dependencies` and `spec-needed`.

The milestone of an issue is the phase that it serves. An issue with no milestone is not planned for a phase yet. A change that closes gets the milestone of its phase.

Each phase has one plan issue with the label `kind:plan`. It lists the changes of the phase in order. When you archive a change, mark its row in the plan Done.

### Follow-up issues

A review can defer rows to follow-up issues. Each follow-up issue gets the label `kind:followup`. At the end of a phase, the owner groups the follow-up issues into themed parent issues with sub-issues, so that no deferred row is lost.

### The audit

The project skill `tracking-audit` compares the Project, the labels, the milestones and the OpenSpec folders, and it lists each gap. The script needs `uv` and a `gh` login with the `project` scope. Run it after each archive and at the end of each phase:

```bash
.agents/skills/tracking-audit/audit.py
```
