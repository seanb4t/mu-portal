---
title: Dependency policy
description: Which crates the project can depend on, and what enforces it.
sidebar:
  order: 14
---

## Decision

A tool, `cargo-deny`, checks every crate in the dependency tree. The file `deny.toml` holds the policy, and the `check` task runs it.

- A crate with a known vulnerability fails. An unmaintained crate fails.
- A crate must have a license from the allowed list in `deny.toml`. The list holds permissive licenses only. A crate with a GPL license fails.
- A crate must come from crates.io. A git source needs an entry in `deny.toml`.
- A dependency with no version requirement fails. A path dependency between two crates of the workspace is the exception.

The policy applies to a crate that cannot be published as it applies to every other crate.

## Reason

The project uses the Apache-2.0 license, and a GPL dependency is not compatible with that choice. A tool finds a license problem or a vulnerability on the day it enters the tree. A person finds it much later.

## Source

The OpenSpec change `dev-workflow`.
