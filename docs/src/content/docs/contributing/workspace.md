---
title: Workspace layout
description: The crates, their allowed dependencies and the test that enforces them.
sidebar:
  order: 2
---

## Directories

| Path | Holds |
|---|---|
| `crates/` | The six crates of the product |
| `spikes/` | Time-limited experiments, in a separate Cargo workspace |
| `docs/` | This documentation site |
| `architecture/` | The LikeC4 architecture model |
| `openspec/` | The specs and the changes |

## Crates

| Crate | Holds |
|---|---|
| `mu-proto` | API schema and generated code |
| `mu-core` | Server engine, with no knowledge of the wire format |
| `mu-server` | API server: the adapter between the engine and the schema |
| `mu-client` | API client, with the resume of the event stream |
| `mu-tui` | Terminal client |
| `mu-cli` | The `mu` binary and its verbs |

`mu-tui` cannot depend on `mu-core` or `mu-server`. The terminal client reaches the server only through the API. `mu-core` cannot depend on `mu-proto` or on a crate of the wire format. The [crate layout](/architecture/crate-layout/) view shows every allowed dependency.

## The boundary test

The test file `crates/mu-cli/tests/workspace_boundary.rs` enforces the workspace rules. It has two allowlists. The first is the source of the dependency rule between crates. The second names the third-party crates that each crate can use. The [crate layout](/architecture/crate-layout/) view is a copy.

The test fails for each of these:

- a crate that the allowlist does not name
- a dependency between two crates that the allowlist does not list
- a third-party crate that the second allowlist does not list for that crate
- a dependency on a crate in `spikes/`
- a Rust file in which `include!`, `include_str!`, `include_bytes!` or a `path` attribute names a path with a component `spikes`. The test also looks inside other macros and `cfg_attr`, and it joins the parts that `concat!` joins.
- a Rust file that the test cannot read as Rust tokens
- a symbolic link in a crate whose target is a file or a directory in `spikes/`
- a path dependency on a crate outside the workspace
- a crate with a license that is not Apache-2.0
- a crate that does not inherit the lints, the minimum Rust release or the `publish` field from the workspace
- a workspace that does not state `publish = false`
- a third-party dependency that does not come from `[workspace.dependencies]`

## How to change the crates

To add a crate or a dependency between crates, change these in the same change:

1. The allowlist in `crates/mu-cli/tests/workspace_boundary.rs`.
2. The crate layout view in `architecture/model.c4`.

To add a third-party dependency, add it to `[workspace.dependencies]` in the root `Cargo.toml`. Then name it in the crate with `workspace = true`. Also add it to the second allowlist in `crates/mu-cli/tests/workspace_boundary.rs`, for each crate that uses it.
