---
title: Workspace layout
description: The crates, their allowed dependencies and the test that enforces them.
sidebar:
  order: 2
---

## Directories

| Path | Holds |
|---|---|
| `crates/` | The four crates of the product |
| `spikes/` | Time-limited experiments, in a separate Cargo workspace |
| `docs/` | This documentation site |
| `architecture/` | The LikeC4 architecture model |
| `openspec/` | The specs and the changes |

## Crates

| Crate | Holds | Can depend on |
|---|---|---|
| `mu-proto` | API schema and generated code | no other crate of the workspace |
| `mu-core` | Server engine | `mu-proto` |
| `mu-tui` | Terminal client | `mu-proto` |
| `mu-cli` | The `mu` binary and its verbs | `mu-proto`, `mu-core`, `mu-tui` |

`mu-tui` cannot depend on `mu-core`. The terminal client reaches the server only through the API.

## The boundary test

The test file `crates/mu-cli/tests/workspace_boundary.rs` enforces the workspace rules. Its allowlist is the source of the dependency rule. The table above and the [crate layout](/architecture/crate-layout/) view are copies.

The test fails for each of these:

- a crate that the allowlist does not name
- a dependency between two crates that the allowlist does not list
- a dependency on a crate in `spikes/`
- a path dependency on a crate outside the workspace
- a crate with a license that is not Apache-2.0
- a crate that does not inherit the workspace lints
- a third-party dependency that does not come from `[workspace.dependencies]`

## How to change the crates

To add a crate or a dependency between crates, change these in the same change:

1. The allowlist in `crates/mu-cli/tests/workspace_boundary.rs`.
2. The crate layout view in `architecture/model.c4`.
3. The table on this page.

To add a third-party dependency, add it to `[workspace.dependencies]` in the root `Cargo.toml`. Then name it in the crate with `workspace = true`.
