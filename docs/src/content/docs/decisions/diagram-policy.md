---
title: Diagram policy
description: LikeC4 for structure, Archify for the rest.
sidebar:
  order: 12
---

## Decision

The project uses two diagram tools, and each one has its own area. LikeC4 owns every structural diagram. Archify draws the diagrams that LikeC4 does not model.

The rules are on the Contributing page for the [diagram policy](/contributing/diagrams/). That page is the only place that states them.

## Reason

A LikeC4 model is text in the repository. One model gives many views, and a model error fails the build. That fits structure, which changes with the code.

Archify is an agent skill and produces single drawings. It is good for a state machine or an explainer. It is not a build tool, so the build must not depend on it.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it.
