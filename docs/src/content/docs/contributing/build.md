---
title: Build and test
description: The commands that build and test the project.
sidebar:
  order: 1
---

The project has two toolchains. Rust builds the `mu` binary. Node builds this documentation site and the architecture model. The Rust build never needs Node.

## Rust

You need `rustup`. It reads `rust-toolchain.toml` and installs the pinned release by itself.

Run these commands in the root of the repository:

```sh
cargo build --workspace
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

The build writes the binary to `target/debug/mu`.

## Documentation site

You need Node and `pnpm`. The file `docs/.node-version` states the Node release. The `packageManager` field in `docs/package.json` states the `pnpm` release.

Run these commands in the root of the repository:

```sh
cd docs
pnpm install --frozen-lockfile
pnpm model:check
pnpm build
```

`pnpm model:check` reports errors in the architecture model. `pnpm build` writes the site to `docs/dist/`. To read the site while you edit, run `pnpm dev`.

## Continuous integration

The workflow `.github/workflows/ci.yml` runs the same commands on each pull request. The Rust commands run on Linux and on macOS. The workflow reads the toolchain releases from the pin files and states no release itself.
