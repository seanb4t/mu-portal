---
title: Spikes
description: The rules for a time-limited experiment.
sidebar:
  order: 3
---

A spike is a time-limited experiment that answers one question. Spike code lives in `spikes/`, which is a separate Cargo workspace. The release build and CI do not compile it.

## Rules

- A spike answers one question. The GitHub issue of the spike states the question.
- A spike has a time limit. The issue states the limit.
- A spike ends in a written decision. The decision goes on the issue, and a project-wide decision also gets a page in the Decisions section.
- A spike is one crate in `spikes/`. Its name comes from its issue, for example `spikes/output-pane`.
- A spike can depend on a crate in `crates/` by path.
- No crate in `crates/` can depend on a spike. The boundary test enforces this rule.
- Only one spike is in progress at a time.

## After the decision

Spike code is evidence for its decision. It stays on the main branch while a later change depends on that evidence. You can then delete it.
