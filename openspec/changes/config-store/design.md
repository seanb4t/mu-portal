# Design

## Context

See proposal.md for the motivation. The facts that shape the design:

- `mu-cli` reads and parses `client.toml` through `config_file::read` into the type `ClientConfig` of `mu-tui`. The key grammar is `Chord::parse` in `crates/mu-tui/src/keymap.rs`, and it builds `termina` key codes. The entry names are in `crates/mu-tui/src/registry.rs`, next to the functions that run each entry.
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

- The world rules `extends`, `disable` and arrays that accumulate, for the client layers. The client layers reuse the merge library of the world layers, not the world rules.
- A change to how `mu world list` and `mu world show` read the directory. They still read the files directly.

## Decisions

### D1. A new crate, `mu-config`, holds the client fields and the key grammar

`mu-config` holds these parts:

- The serde types of a client layer.
- The grammar of a key name, and the check of a `[keys]` table.
- The table of entry names, each with the flag that says whether it needs an argument.
- The names of a client: the kind, the OS, and the device name with its rule of 1 to 64 characters.

It has no `termina` type and no TOML parser. A key name parses to a neutral value of modifiers and one key, and that value prints back as one canonical name. `mu-tui` maps the value to a `termina` chord. The server and `mu-cli` both check names with `mu-config`, so the device rule exists once.

The crate edges become: `mu-config` depends on no member. `mu-server`, `mu-tui` and `mu-cli` can depend on it. Its only allowed third-party crate is `serde`. The property test `key_names_read_back` moves with the grammar, with `proptest` as a dev-dependency. The row of `mu-config` in `ALLOWED_CRATES` names `serde` and `proptest`, because that check also reads dev-dependencies.

`ClientConfig` and its binding check leave `mu-tui`. The client gets its keymap from the reply of `GetClientConfig`, which the server checked.

Alternatives:

- The server keeps each kind's table as opaque text, and the client checks it. Rejected: the server cannot then check a write before it lands (#173), and an MCP server gets no error.
- The grammar goes in `mu-proto`. Rejected: `mu-proto` holds the wire format only.
- `mu-server` depends on `mu-tui`. Rejected: the crate rules forbid it, and the server then pulls in the terminal code.

The entry table in `mu-config` and the registry in `mu-tui` name the same entries with the same argument flag. A test in `mu-tui` compares the name and the flag of each entry and fails on any difference.

### D2. The store lives in `mu-server` and keeps a snapshot

The directory is read once for each start and each reload. `mu-core` gets a snapshot type, `DirSnapshot`. It walks the directory with the walk and the top-file rule of `TomlDirSource`, and keeps each relative path with its bytes. It keeps every file under `clients/`, whatever its name ends with, so the store can report a file that is not a layer file. The scan skips `clients` at the top level with the case rule of the top files. So `Clients/` on a file system that ignores case is the same directory. The snapshot also keeps the bytes of `server.toml`, only so that a reload can see a change to it. The walk stops today at the first entry that it cannot read, stat or list. The snapshot does not stop. It keeps each such entry as a path with its error, and the reader of that path decides:

- Under `clients/`, the directory `clients/` itself included, the entry is a bad client layer.
- A world file or a world directory with an error stops the world resolve, as today. At a reload, that error is reported and the client layers still apply.
- `server.toml` keeps its own reader, as today. A read error of `server.toml` is kept with its path, and the world read ignores it. So `mu world list` does not change when `server.toml` cannot be read.

The snapshot walks with the `walkdir` crate, which already is in `Cargo.lock`, with `follow_links(true)`. `walkdir` returns an error for each entry that it cannot read and goes on with the next entry. It also returns a loop of links as an error of the entry that closes the loop. So two links back to `.` end the walk. The top-file rule and the skip of `clients` stay as a filter on the entries. `walkdir` enters the `mu-core` row of `ALLOWED_CRATES` and `[workspace.dependencies]`. `DirSnapshot` implements `LayerSource`, and `TomlDirSource` becomes a snapshot followed by its layers. So the store resolves the worlds with `mu_core::world::resolve` from the same bytes that it keeps. `Store::open` returns the store, the resolved worlds and the notices. `mu-cli` builds each `WorldSetup` from those worlds, and checks the fake ids against them, as it does today with the result of `resolve`.

The store parses each file under `clients/` with `mu_core::world::parse_toml` into the types of `mu-config`, so the line of an error comes from one parser. A client layer parses with `Hidden::All`, so no error quotes a value. Then it checks the `[keys]` table with `mu-config`. The fields of a client layer skip an unset value when they serialize, as the world types do. The reason is that `figment2` takes an empty later value over an earlier one. A good client layer keeps its parsed fields. A bad client layer keeps a reason that names the file, the field or the binding, and the line when there is one.

The merge of the client layers uses `figment2`, as the world merge does. The function `world` in `crates/mu-core/src/world/resolve.rs` merges named layers over the defaults. It also maps `find_metadata` back to a layer. That block moves to one helper in `mu-core`. The world resolve and the store both call it, so `mu-server` does not depend on `figment2`. A nested table merges key by key in `figment2` (`Value::coalesce` merges two dicts recursively, `src/coalesce.rs` in 0.11.5), so `[keys]` gets its rule from the library. Before the merge, each key name becomes its canonical name, so two names of one key in two layers meet as one key. `figment2` splits a key path at each `.` and has no escape, so no canonical name holds a dot. The key `.` has the canonical name `period`, as `+` has `plus`. The parser accepts both `.` and `period`.

`server.toml` stays parsed in `mu-cli`, because the telemetry export must start before the server. The start runs `server_config::read` and then the value checks of `telemetry::export`. The change moves those two steps into one function, `server_config::check`, and the start calls it before it builds the exporter. `mu-cli` gives the same function to the store, which calls it at each reload. So the start and the reload refuse the same files.

The store writes nothing to stderr. It returns its notices, and `mu-cli` writes each one through `warn`, which turns control characters into U+FFFD. So a file name under `clients/` cannot send an escape sequence to the terminal. `warn` keeps a line break, so the store also writes the path of each notice with each control character, the line break included, as U+FFFD. One bad layer then gives one line. `mu config show` and `mu config reload` print the notices of their reply through `warn` too. The restart lines of a reload name a file, so they go to stdout through the same filter.

A reload builds a new snapshot and compares it with the current one:

- Changed, added or removed client layers replace the client part of the snapshot.
- A changed world file or `server.toml` is listed as "applies at restart" and does not replace anything. The compare is always against the snapshot of the start, so each later reload lists the change again until a restart.
- Every file is checked. A bad world file is reported by the reload, and the running worlds stay.

One mutex serializes reloads. A reload swaps the client part as one value, so a `GetClientConfig` call sees the old or the new layers, never a mix. The client part holds its store hash (D5), computed from its own entries before the swap. `GetClientConfig` takes one reference to the client part, and reads the store hash and the layers of its reply from that one value.

Alternative: keep `mu-cli` as the reader and pass a callback to the server for reloads. Rejected: the reader and the reload then live in two crates, and #173 needs one owner. The `server.toml` check is the one callback, because its types belong to the telemetry code of `mu-cli`.

Alternative: a hand-written merge of the client layers. Rejected: `figment2` already merges the world layers with a source for each field, and #172 adds fields to the client layers.

### D3. The world scan skips `clients/` and rejects `client.toml`

`crates/mu-core/src/world/dir.rs` drops `client.toml` from the top-level files. It skips the directory `clients` at the top level only, so `characters/clients/<character>.toml` is still the world of a game named `clients`.

A leftover `client.toml` becomes a stray file. The stray-file error today lists the layer files. So the change adds one branch. For `client.toml` at the top level, the error says to move its content to `clients/global.toml`.

### D4. The revision is a hash of the layers that apply

The revision of one client is a SHA-256 hash. Its input is the sorted list of the layer files that exist for the kind, the OS and the device of the client. Each entry is framed, so no two lists give the same input:

- the length of the path as 8 bytes, big-endian, then the path
- one tag byte: 0 for a file that was read, 1 for a file that the server cannot read
- for tag 0, the length of the bytes as 8 bytes, big-endian, then the bytes
- for tag 1, the kind of the read error as text, with its length first

A bad layer counts too, so a layer that turns bad changes the revision. The error of a directory on the path of a layer of the client also counts, as an entry with tag 1. These directories are `clients/`, `clients/kinds`, `clients/os` and `clients/devices`. So a client with no layer file gets a new revision, and its notice, when `clients/` turns unlistable. A layer that turns unreadable is not the same as an empty one. The text form is the first 16 hex digits.

A hash keeps the revision the same across a restart, which a counter does not. A later write can also compare a revision with the files on disk after an edit by hand (#173).

Alternative: one revision for the whole directory. Rejected: an edit of one device file then makes every other client fetch again. A later compare-and-swap then gets conflicts across unrelated files.

### D5. The API

Three new calls on `PortalService`:

```text
GetClientConfig(kind, os, device)
  -> store_hash, revision, echo, bindings[{key, entry}],
     sources{field path -> layer path}, notices[{path, reason}]
WatchConfig()                       (server stream)
  -> store_hash, then a new store_hash after each reload that changes it
ReloadConfig()
  -> notices[{path, reason}], restart[{path}]
```

The store hash is a hash of the whole `clients/` part of the snapshot, with the framing of D4. Its input is every path under `clients/` with its bytes or its read error, and the error of `clients/` itself. A reload computes it again. The server publishes the new hash only after the new client part is visible to `GetClientConfig`.

The server keeps no state about a client. `WatchConfig` sends the current store hash when it opens, and every watcher gets the same later hashes. The watch stream ends on the stopping receiver, as the event stream does, so a graceful shutdown does not wait for it. The client decides:

- If an announced hash differs from the store hash of its configuration, the client calls `GetClientConfig`.
- The client applies a reply only if the `store_hash` of the reply equals the latest hash that it has seen. Otherwise a newer fetch is already on its way, so two replies can never apply in the wrong order.
- When the watch stream ends, `mu-client` opens it again with the backoff of `Subscribe`. The wait flag, the sleep by the backoff and the new backoff at the first message are in `Resume::open` and `Resume::hello` today. They move to one type that both streams use. The first message is the current hash, so a reload that the client missed, or a restart of the server, shows as a new hash. The watch stream has no sequence number and nothing to replay.

A change to the layer of one device gives every client a new store hash. Each other client then makes one call and keeps its configuration, because its revision did not change. The number of clients of one server is small.

`mu-client` reports the watch stream as `ClientEvent::StoreHash(hash)`. The journal stream and its `Hello` do not change.

`GetClientConfig` checks the kind, the OS and the device name with `mu-config`, and refuses a bad name with an error that names it.

The sources are one map from the path of a field, such as `echo` or `keys.alt+period`, to the layer that set it. That is the form of `mu world show`. A field that #172 adds needs no new field in the reply.

The `revision` of D4 stays for each client. `mu config show` prints it, and the writes of #173 compare against it.

`ListWorlds.config_dir` stays. `mu config show` does not need it, but a remote client or an MCP server can use it to name the directory.

Alternatives:

- A `config_changed` message on the journal stream, and a fetch on each `Hello`. Rejected: a client misses the message in each gap of the stream, and two fetches can finish in the wrong order.
- A watch stream that sends the configuration of one client. Rejected: the server then keeps the kind, the OS and the device of each stream, and decides which clients a reload affects.

### D6. The bootstrap values

The URL parser is the `url` crate, which already is in `Cargo.lock` through other crates. The change adds it to `[workspace.dependencies]` and to the `mu-cli` row of `ALLOWED_CRATES`. The path is percent-decoded. The value of `connect_wait` goes through the existing `connect_wait` function of `crates/mu-cli/src/client.rs`, and a message prints the time with its `shown` function. The command refuses a URL before any connection. It refuses a scheme that is not `unix`, no `//` after the scheme, a host, a user, a password, a fragment, or a path that is not absolute. The query is read with `Url::query_pairs`. A name other than `connect_wait` is refused, and so is a name that is there twice.

`sha2` enters the `mu-server` row of `ALLOWED_CRATES` and `[workspace.dependencies]`. `proptest` enters the `mu-server` row for the property test of D4, as it is in the rows of `mu-core`, `mu-tui` and `mu-cli`. The rows of `ALLOWED_CRATES` cover every dependency table, so the change adds no other check.

The host name comes from `rustix::system::uname().nodename()`. `rustix` is already a workspace dependency, and the change adds its `system` feature. A host name that is not a valid device name after the change to lowercase stops the command, and the message says to set `--device`.

`mu config show` and `mu config reload` take the same `--server` and `--device` flags, and never start a server. `mu config show` also takes `--kind`, with the default `tui`, and `--os`, with the default of the machine. Both values are checked with `mu-config` before the call. Each waits for the answer of its call at most the time of `connect_wait`. A message about a server that did not answer names the URL and that time. Today that timeout and its message are inline in the start of `mu client`. They move to one function in `crates/mu-cli/src/client.rs` that takes the URL, the wait and a future. `mu client`, `mu config show` and `mu config reload` all call it.

### D7. The start of `mu client`

The order becomes:

1. Parse the bootstrap values.
2. Check the terminal.
3. If nothing listens, start a server.
4. Make the first list, open `WatchConfig`, and call `GetClientConfig` after the first hash. One future holds all three steps, and the timeout function of D6 wraps that future. So the three steps together wait at most the time of `connect_wait`.
5. Open the screen. The notices of `GetClientConfig` go to the screen through the existing `notices` of `mu_tui::run`. The directory compare and its notice go away.

### D8. A live change in the client

`mu-tui` gets a message for `ClientEvent::StoreHash` (D5). The `Echo` type and its default leave `mu-tui`, and the model holds the echo type of `mu-config`. If the hash differs from the store hash of its configuration, the effect calls `GetClientConfig`. A reply applies only if its store hash equals the latest hash. A reply with a new revision replaces the keymap and the echo setting of the model. The model keeps the notices that it showed, and shows only a notice that is not among them. So a notice that stays the same across two revisions shows once. A reply with the same revision changes nothing. A key that is held during the swap uses the old or the new keymap, never a mix.

### D9. The documentation and the diagrams

The diagrams follow `docs/src/content/docs/contributing/diagrams.md`: LikeC4 for the system, the crates and a sequence.

- `architecture/model.c4` gets the part `Configuration store` in `mu.server`, with its edges to the API, and the crate `mu-config` with its edges.
- `architecture/views.c4` gets two dynamic views. `clientStart` shows `mu client` from the URL to the first frame: the first list, `GetClientConfig`, the keymap and the notices. `configReload` shows `mu config reload` to a new keymap in each attached client. The description of `crateLayout` loses its count of crates, because live documentation carries no counts.
- `architecture/server-and-clients.mdx` embeds both views and describes the store. `architecture/crate-layout.mdx` describes `mu-config`. `architecture/terminal-client.mdx` says where the keymap comes from.
- The guides `client.md`, `configuration.md`, `first-run.md` and `telemetry.md` change, and so does `index.md`. They lose `--socket` and `client.toml` for the client, and gain the URL, the device, the `clients/` layers and `mu config`.
- The new decision page `decisions/configuration-ownership.md` records #169.

## Enforcement

| Rule | Enforced by |
|---|---|
| `mu-server` and `mu-tui` reach the key grammar only through `mu-config` | `workspace_boundary.rs`, in CI |
| The entry table of `mu-config` matches the registry of `mu-tui`, names and argument flags | A test in `mu-tui`, in CI |
| `sha2` and `url` enter only the crates that D6 names | `workspace_boundary.rs`, in CI |
| No canonical key name holds a dot | The property test `key_names_read_back` in `mu-config` |
| Two different layer lists give two different revision inputs | A property test of the framing in `mu-server` |
| The documentation and the LikeC4 model show the store and the new crate | `mise run docs:check` and `pnpm model:check`, in CI, and review |
| A client never keeps an old revision after a gap in its watch stream | The tests of D5: `a_reload_while_the_stream_is_down`, `a_reload_across_a_server_restart`, and the unit tests of the store hash in `update.rs` |
| A store hash goes out only after its configuration is visible | `a_fetch_on_a_new_hash_gets_the_new_revision` in `mu-server` |
| A bad client layer never stops the server | The tests of the scenarios of the `config-store` spec |
| A reload never applies a world change | The test of "A reload does not apply a world change" |
| The client reads no configuration file | The test of "The client takes the flag", and review |
| No secret in a URL | Review alone. The URL grammar has no field for one, and an unknown query option is refused. |

## Risks / Trade-offs

- [A skipped device layer brings back a binding that the user removed with `none`] → The client shows a notice that names the file. Every entry is a client action, and none sends text to a game.
- [The user misses the notice] → The prominent notice of #172 is the follow-up.
- [A new store hash reaches clients whose revision did not change] → Each such client makes one call and keeps its configuration. The number of clients of one server is small.
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
- That `find_metadata` names the provider of a key inside `[keys]`. The merge of a nested table is checked in D2.
