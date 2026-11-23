# Design

## Context

See proposal.md for the motivation. The facts that shape the design:

- `mu-tui` parses `client.toml`. The key grammar is `Chord::parse` in `crates/mu-tui/src/keymap.rs`, and it builds `termina` key codes. The entry names are in `crates/mu-tui/src/registry.rs`, next to the functions that run each entry.
- `crates/mu-cli/tests/workspace_boundary.rs` lets `mu-server` depend on `mu-core` and `mu-proto` only. So the server cannot use the parser in `mu-tui`.
- `mu-cli` resolves the worlds with `mu_core::world::resolve` before it calls `mu_server::serve`. Nothing in the server reads the directory again.
- `mu-cli` reads `server.toml` before the socket opens, because the telemetry export starts first.
- The directory scan in `crates/mu-core/src/world/dir.rs` accepts `server.toml` and `client.toml` at the top level, and treats any other `.toml` file as a stray file.
- `ListWorlds` returns `config_dir` only for the directory notice that this change removes.

## Goals / Non-Goals

**Goals:**

- One owner, in `mu-server`, for the files of the configuration directory, with a single check for each client layer. A later write call (#173) uses the same check before a write lands.
- One grammar of key names and one list of entry names, used by the server to check and by the client to apply.
- A revision that a later compare-and-swap can use without a change to the read call.

**Non-Goals:**

- A general merge engine shared with the world layers. The client layers have two fields. The world merge has rules that the client layers do not need, such as `extends`, `disable` and arrays that accumulate.
- A change to how `mu world list` and `mu world show` read the directory. They still read the files directly.

## Decisions

### D1. A new crate, `mu-config`, holds the client fields and the key grammar

`mu-config` holds the serde types of a client layer and the grammar of a key name. It also holds the table of entry names, with the entries that need an argument. It has no `termina` type. A key name parses to a neutral value of modifiers and one key. `mu-tui` maps that value to a `termina` chord.

The crate edges become: `mu-config` depends on no member. `mu-server`, `mu-tui` and `mu-cli` can depend on it. Its allowed third-party crates are `serde` and `toml`.

Alternatives:

- The server keeps each kind's table as opaque text, and the client checks it. Rejected: the server cannot then check a write before it lands (#173), and an MCP server gets no error.
- The grammar goes in `mu-proto`. Rejected: `mu-proto` holds the wire format only.
- `mu-server` depends on `mu-tui`. Rejected: the crate rules forbid it, and the server then pulls in the terminal code.

The entry table in `mu-config` and the registry in `mu-tui` name the same entries. A test in `mu-tui` compares the two lists and fails on any difference.

### D2. The store lives in `mu-server` and keeps a snapshot

At the start, the store reads every `.toml` file under the configuration directory into a snapshot: each relative path with its bytes. From the snapshot it resolves the worlds with `mu_core::world::resolve`, as `mu-cli` does today. It parses and checks each file under `clients/` with `mu-config`. A good client layer keeps its parsed fields. A bad client layer keeps a reason that names the file, the field or the binding, and the line when there is one.

`server.toml` stays parsed in `mu-cli`, because the telemetry export must start before the server. The store keeps its bytes only, to see a change at reload.

A reload builds a new snapshot and compares it with the current one:

- Changed, added or removed client layers replace the client part of the snapshot.
- A changed world file or `server.toml` is listed as "applies at restart" and does not replace anything.
- Every file is checked. A bad world file is reported by the reload, and the running worlds stay.

One mutex serializes reloads. A reload swaps the client part as one value, so a `GetClientConfig` call sees the old or the new layers, never a mix.

Alternative: keep `mu-cli` as the reader and pass a callback to the server for reloads. Rejected: the reader and the reload then live in two crates, and #173 needs one owner.

### D3. The world scan skips `clients/` and rejects `client.toml`

`crates/mu-core/src/world/dir.rs` drops `client.toml` from the top-level files and skips the directory `clients`. A leftover `client.toml` becomes a stray file, and its error says to move the content to `clients/global.toml`. This uses the existing stray-file rule, so no new error path is needed.

### D4. The revision is a hash of the layers that apply

The revision of one client is a SHA-256 hash. Its input is the sorted list of the layer files that exist for the kind, the OS and the device of the client, each path with its bytes. A bad layer counts too, so a layer that turns bad changes the revision. The text form is the first 16 hex digits.

A hash keeps the revision the same across a restart, which a counter does not. A later write can also compare a revision with the files on disk after an edit by hand (#173).

Alternative: one revision for the whole directory. Rejected: an edit of one device file then makes every other client fetch again. A later compare-and-swap then gets conflicts across unrelated files.

### D5. The API

Two new calls on `PortalService`, and one new stream message:

```text
GetClientConfig(kind, os, device)
  -> revision, echo, echo_source, bindings[{key, entry, source}], notices[{path, reason}]
ReloadConfig()
  -> notices[{path, reason}], restart[{path}]
SubscribeResponse.config_changed {}     (not in the journal, no sequence number)
```

After a reload that changes any client layer, the server sends `config_changed` on every stream. Each client calls `GetClientConfig` again and applies the result if the revision differs. The server does not track the kind, OS and device of each stream. A broadcast costs one small call for each client, and the client keeps the comparison.

`ListWorlds.config_dir` stays. `mu config show` does not need it, but a remote client or an MCP server can use it to name the directory.

Alternative: put the configuration in `Hello`. Rejected: a reload then needs a new stream, and `Hello` gets data that has nothing to do with the journal.

### D6. The bootstrap values

The URL parser is the `url` crate, which already is in `Cargo.lock` through other crates. The path is percent-decoded. The command refuses a URL before any connection in these cases: a scheme that is not `unix`, a host, a user, a password, a fragment, or a path that is not absolute.

The host name comes from `rustix::system::uname().nodename()`. `rustix` is already a workspace dependency, and the change adds its `system` feature. A host name that is not a valid device name after the change to lowercase stops the command, and the message says to set `--device`.

`mu config show` and `mu config reload` take the same `--server` and `--device` flags, and never start a server.

### D7. The start of `mu client`

The order becomes:

1. Parse the bootstrap values.
2. Check the terminal.
3. If nothing listens, start a server.
4. Make the first list and the call `GetClientConfig`, both within one wait of `connect_wait`.
5. Open the screen. The notices of `GetClientConfig` go to the screen through the existing `notices` of `mu_tui::run`. The directory compare and its notice go away.

### D8. A live change in the client

`mu-tui` gets a message for `config_changed`. Its effect calls `GetClientConfig`. The result replaces the keymap and the echo setting of the model, and each new notice shows once. A key that is held during the swap uses the old or the new keymap, never a mix.

## Enforcement

| Rule | Enforced by |
|---|---|
| `mu-server` and `mu-tui` reach the key grammar only through `mu-config` | `workspace_boundary.rs`, in CI |
| The entry table of `mu-config` matches the registry of `mu-tui` | A test in `mu-tui`, in CI |
| A bad client layer never stops the server | The tests of the scenarios of the `config-store` spec |
| A reload never applies a world change | The test of "A reload does not apply a world change" |
| The client reads no configuration file | The test of "The client takes the flag", and review |
| No secret in a URL | Review alone. The URL grammar has no field for one, and an unknown query option is refused. |

## Risks / Trade-offs

- [A skipped device layer brings back a binding that the user removed with `none`] → The client shows a notice that names the file. Every entry is a client action, and none sends text to a game.
- [The user misses the notice] → The prominent notice of #172 is the follow-up.
- [A broadcast of `config_changed` reaches clients whose revision did not change] → Each such client makes one call and keeps its configuration. The number of clients of one server is small.
- [A reload reads a file while an editor writes it] → The file is bad for that reload, and the next reload fixes it. The command reports the bad file, so the user sees it.
- [A host name with characters that a device name refuses] → The command stops and names the host name and `--device`.
- [The entry table and the registry drift apart] → The test of D1.

## Migration Plan

There is no release, so no automatic migration. The user moves the content of `client.toml` to `clients/global.toml` and drops the table `[server]`. A leftover `client.toml` stops `mu server` and `mu world`, with an error that names the file and the new path. A script that sets `MU_SOCKET` for `mu client` changes to `MU_SERVER=unix://<path>`.

## Third-party facts to check

These facts are not checked against current documentation yet. A task reads the documentation before the work depends on each one:

- How the `url` crate parses `unix:///path`, `unix:path` and `unix://host/path`, and how it exposes the percent-encoded path.
- That `rustix::system::uname` needs the `system` feature, and what `nodename` returns on macOS and Linux.
- The `sha2` API for hashing several byte slices.
