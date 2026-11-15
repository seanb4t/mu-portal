# Proposal

## Why

The repository holds only a README, an MIT license, a `.gitignore` from a Node template and the OpenSpec scaffold. The Phase 0 spikes and every later change need a workspace, a command structure, a documentation site and CI before they can start. The exploration also fixed several project-wide decisions, and this change puts them in the repository so that later changes inherit them.

## What Changes

- Add a Cargo workspace that builds one binary named `mu`. The binary has verbs, in the style of `kubectl`, `docker` and `bd`. `mu` with no verb starts the client. `mu server` is the entry point for the server. Later changes add verbs such as `mu pair` and `mu import`.
- Split the workspace into library crates behind that binary, so that the server, the API schema and the terminal client stay separate: `mu-core` (server engine), `mu-proto` (ConnectRPC schema and generated code), `mu-tui` (terminal client) and `mu-cli` (the binary and its verbs). In this change the verbs exist and report that they are not implemented yet.
- Replace the MIT license with Apache-2.0. Add the license field to every crate, and add a test that fails for a crate without it. The license is a property of the repository and has no spec requirement.
- Add a `spikes/` directory for time-limited experiments. It is a separate workspace, so spike code never enters a release build.
- Add a documentation site built with Astro Starlight. Pages are Markdown by default. If a page embeds a component, such as an architecture view, the page uses MDX.
- Add a LikeC4 architecture model in the repository, with the first views: system context, the server and its clients, and the crate layout. The documentation site renders these views through the LikeC4 Vite plugin. A change that alters the architecture updates the model in the same change.
- Set a diagram policy with two tools. LikeC4 owns every structural diagram: the system, the server and its clients, the crates and deployment. Archify draws what LikeC4 does not model: state machines, data schemas, timelines and explainer pages. For a sequence, use a LikeC4 dynamic view first.
- Keep Archify out of the build. If an Archify diagram enters the documentation, commit its candidate JSON and its exported SVG together. Scratch output in `.archify/` stays out of git.
- Add CI that builds and tests on Linux and macOS, with formatting and lint steps. CI also makes sure that the LikeC4 model is correct, and it builds the documentation site.
- Add contributor documentation: how to build, how the workspace is laid out, how spikes work, and how OpenSpec and the GitHub Project divide the work.
- Record each project-wide decision from the exploration as one page in the Decisions section of the documentation site. Task 5.1 in tasks.md lists the pages. That section is the durable record of decisions.
- Add a Renovate configuration that also pins GitHub Actions to commit digests.

Not in this change: any network code, any terminal interface, the ConnectRPC schema content, Tailscale, pairing, and publishing the documentation site to a host.

## Capabilities

### New Capabilities

- `cli-entrypoint`: The single `mu` binary and its command grammar. It covers the default verb (the client), verb dispatch, help and version output, and the exit status for a verb that is not implemented.

### Modified Capabilities

None.

## Impact

- **Repository layout**: new `Cargo.toml` workspace, `crates/`, `spikes/`, `docs/` (Starlight site) and `architecture/` (LikeC4 model).
- **License**: `LICENSE` changes from MIT to Apache-2.0. The project cannot copy code from GPL-3 clients such as Blightmud or smudgy.
- **Toolchains**: Rust for the workspace, and Node for the documentation site and the LikeC4 tooling. CI needs both.
- **Dependencies**: `clap` (or an equivalent) for the command grammar. Astro, Starlight, React and `likec4` for the documentation site.
- **Platforms**: Linux and macOS are both first-class from this change, because the server must run on either.
- **Open Knowledge Format (OKF)**: this change adds no OKF bundle. OKF reads `.md` files only, so it does not combine with MDX pages. Pages stay Markdown by default, and a parked GitHub issue records the idea. See design.md, Decision 9.
- **GitHub**: this change is tracked by issue #2 in the mu-portal Project.
