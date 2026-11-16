# Proposal

Tracking issue: #51. The plan of Phase 1 is in #61.

## Why

The product exists only as decision pages and four empty crates. Phase 1 builds one layer in each change. Each layer plugs into a seam. So the first change builds every seam once and proves them end to end, before a real game is involved.

## What Changes

- `mu server` serves the API on a Unix socket. If another server serves the same socket, it refuses to start. It replaces a socket file that no server serves. It stops on SIGTERM or SIGINT and removes its socket.
- `mu` and `mu client` attach to the server on the socket. When no server runs, the client starts one in the background and then attaches. The client is a plain terminal view: a list of worlds, the lines of the focused world, and one input line.
- The server keeps several worlds connected at once. Each world has a `WorldId` and runs in its own session.
- The server records each event of each world in one journal with one sequence. A client reads all events from one `Subscribe` stream and resumes from a sequence number with no gap and no duplicate.
- Each event has a source: the game, the player, a script, the system or a credential. The server replaces each password in a login line with `****` before the event reaches the journal.
- A fake world serves as the game. It answers a login and echoes each line. A hidden server flag starts it, for tests and for development. Real game connections come in #54.
- The seams that later changes fill in. These are the session, the transport, the journal, the resolved world configuration and the secret store. Also the hooks and handlers, for each world and for the server, and the command set.
- Two new crates: `mu-server` holds the API adapter, and `mu-client` holds the client of the API with its resume. The boundary test no longer lets `mu-core` depend on `mu-proto`, and it refuses the protobuf crates in `mu-core`.
- **BREAKING**: `mu client` and `mu server` no longer print "not implemented".

## Capabilities

### New Capabilities

- `server-process`: the life of `mu server` on its socket, and one server for each socket. Also the stop on a signal, and the start of a server by the client.
- `event-journal`: the journal of events, its one sequence, the event sources, the redaction of credentials, and the `Subscribe` stream with resume.
- `world-sessions`: several worlds at once, the commands that name a world, and the fake world.
- `terminal-client`: what the terminal client shows and sends, and how it moves the focus between worlds.

### Modified Capabilities

- `cli-entrypoint`: the requirement for a verb without an implementation goes, because each verb now has one.

## Impact

- New crates `crates/mu-server` and `crates/mu-client`. New code in `mu-core`, `mu-proto`, `mu-tui` and `mu-cli`.
- `crates/mu-cli/tests/workspace_boundary.rs` (the allowlist), `architecture/model.c4` and the crate pages in `docs/src/content/docs/contributing/workspace.md`.
- `docs/src/content/docs/contributing/openspec.md`, because its rule for a verb without an implementation has no verb left. `README.md` and the start page of the docs, which say that the verbs are stubs. A new guide page and its sidebar group in `docs/astro.config.mjs`.
- New third-party crates through `[workspace.dependencies]`, among them `tokio`, `connectrpc`, `axum`, `backon`, `ratatui` and `crossterm`. The design names each one.
- The tests of `mu-cli` for the stub verbs change.

## Not in this change

- No real game connection, no telnet engine and no TLS. Those are #52, #54 and #58.
- No world configuration files, no layers and no `extends`. The worlds come from the hidden flag until #53.
- No journal on disk. A restart of the server loses the journal until #57.
- No output pane, pose editor or width profile. The view is plain until #55 and #56.
- No Lua, no rules and no `Invoke` call. The hook and handler seams have a no-op implementation and a fake for tests until #59 and #60.
- No secret store of an operating system, for example the macOS Keychain. The first store reads environment variables.
- No TCP listener and no client on another host. A connection that stops without a close is spike #43, in Phase 3.

## Acceptance

```bash
mise run check
```
