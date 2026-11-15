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
- A change that implements a verb modifies the not-implemented requirement of the `cli-entrypoint` capability in its own spec delta. That requirement describes a verb with no implementation, so it no longer applies to that verb.

## GitHub Project

Each change and each spike is one issue in the Project. The status of an issue is one of Parked, Next, In progress, In review and Done.

- Only one change and one spike are in progress at a time.
- A new idea becomes a Parked issue. It does not become a task in an open change.
- The owner reads the Parked issues at the end of each phase.
- The acceptance outcome of a phase decides that the phase is done. A finished task list does not.
