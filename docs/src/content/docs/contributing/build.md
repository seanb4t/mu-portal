---
title: Build and test
description: The commands that build and test the project.
sidebar:
  order: 1
---

The project uses [`mise`](https://mise.jdx.dev) for its tools and its tasks. The file `mise.toml` in the root is the only list of both. `mise` is the supported way to get Node, `pnpm` and the other tools.

Rust is the one exception. The file `rust-toolchain.toml` states the Rust release, and `mise` reads it.

## Set up

Run this command in the root of the repository:

```sh
mise install
```

To run the checks before each commit and each push, install the git hooks once:

```sh
hk install --mise
```

## Check the Rust workspace

```sh
mise run check
```

This task runs the formatting check, the lints, the tests, the documentation build and the dependency policy. To see each task and its command, run `mise tasks`.

The build writes the binary to `target/debug/mu`.

## Check the documentation site

```sh
mise run docs:check
```

This task installs the packages of the site, reports errors in the architecture model and builds the site into `docs/dist/`. To read the site while you edit, run `pnpm dev` in `docs/`.

## Slow tests

```sh
mise run test:workflow
```

These tests copy the repository, add one defect to the copy and make sure that a check fails. They take longer than the other tests, so no hook runs them.

## Continuous integration

The workflow `.github/workflows/ci.yml` runs the same tasks on each pull request. Each step of the workflow is one `mise` task, and the workflow states no command and no tool release itself.

## Rules for the main branch

The file `.github/rulesets/main.json` holds the rules for the `main` branch. A change needs a pull request, and the check `ci-ok` must pass.

To change the rules, change the file first. Then apply it to the repository. The command needs the id of the ruleset, which the first command prints:

```sh
gh api repos/seanb4t/mu-portal/rulesets
gh api --method PUT repos/seanb4t/mu-portal/rulesets/<id> --input .github/rulesets/main.json
```
