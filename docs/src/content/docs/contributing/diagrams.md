---
title: Diagram policy
description: Which tool draws which diagram, and where the files go.
sidebar:
  order: 4
---

This page is the only place that states the diagram rules. The [decision](/decisions/diagram-policy/) page gives the reason.

## Which tool

| Diagram | Tool |
|---|---|
| The system, the server and its clients, the crates, deployment | LikeC4 |
| A sequence | A LikeC4 dynamic view first |
| A state machine, a data schema, a timeline, an explainer page | Archify |

## LikeC4

The model is in `architecture/`. The documentation site loads it through the LikeC4 Vite plugin. A page embeds a view with the `LikeC4View` component, so that page is an MDX file.

- If a change alters the architecture, update the model in the same change.
- To make sure that the model has no error, run `pnpm model:check` in `docs/`. CI runs the same command.

## Archify

Archify is an agent skill. It is not part of the build, and the build must not depend on it.

- If an Archify diagram enters the documentation, commit two files together: its candidate JSON and its exported SVG.
- Put both files in `docs/src/assets/diagrams/`.
- Do not commit the scratch directory `.archify/`. Git ignores it.

No step enforces the pairing of the two files. The first change that adds an Archify diagram also adds that step.
