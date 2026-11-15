---
title: Developer workflow
description: One tool list and one command list, for a contributor and for CI.
sidebar:
  order: 15
---

## Decision

`mise` installs the tools of the project and runs its tasks. The file `mise.toml` is the only list of both. `hk` runs the git hooks, and each hook step calls a `mise` task.

CI runs the same tasks. Each step of the CI workflow is one `mise` task.

The `main` branch accepts a change only through a pull request, and only after the check `ci-ok` passes. That check needs every other job of the workflow.

Rust code follows the `rust-best-practices` skill. The workspace lints enforce the parts that a lint can check.

The rules are on the Contributing page for the [build](/contributing/build/).

## Reason

A check command that exists in two places drifts. With one list, a contributor runs exactly what CI runs, and a failure in CI can be repeated on a laptop with one command.

A hook finds a defect before the push. CI is still the gate, because a hook is a choice of each clone.

## Source

The OpenSpec change `dev-workflow`.
