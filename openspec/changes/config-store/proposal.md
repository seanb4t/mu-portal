# Proposal

## Why

Each `mu client` reads its own `client.toml`. So two clients of one server can show different keys and a different echo. A remote client has no file to read. #169 decided that the server owns all configuration. This change makes that real before `client-layout` (#172) adds the layout to the client configuration. It also lays the ground for writes through the API (#173), which remote clients and an MCP server (#175) need.

Tracked by #169.

## What Changes

- **Configuration store.** The server reads, checks and resolves the whole configuration directory in one place: the worlds, `server.toml` and the new `clients/` directory. The store gives the client configuration a revision.
- **Client layers on the server.** The client configuration is in `clients/` under the configuration directory: `global.toml`, `kinds/<kind>.toml`, `os/<os>.toml` and `devices/<device>.toml`. They merge in the order defaults, global, kind, OS, device. A table merges field by field, and `[keys]` merges key by key. The fields are the same as in `client.toml` today, `echo` and `[keys]`, without `[server]`.
- **A bad client layer is skipped.** The server checks every file at start and at reload. It skips a bad client layer as a whole and writes one line to stderr that names the file and the reason. When a client attaches, the server sends it a notice for each bad layer that applies to it. The server keeps running. A bad world or a bad `server.toml` still stops the start.
- **Read API.** A new call, `GetClientConfig`, takes the kind, the OS and the device of a client. It returns the resolved configuration, its revision, the layer that set each field, and the notices.
- **Reload.** `mu config reload` asks the server to read the directory again. It applies `clients/`, and each attached client gets the new configuration. A change to a world or to `server.toml` is not applied, and the command prints that it applies at restart.
- **Inspection.** `mu config show` prints the resolved client configuration of a kind, an OS and a device. Each field shows its source, as in `mu world show`.
- **Bootstrap.** **BREAKING** `mu client` takes `--server <url>` or `MU_SERVER` in place of `--socket` and `MU_SOCKET`. The URL is `unix://` and an absolute path, for example `unix:///run/user/501/mu/server.sock?connect_wait=5`. The query holds connection options only, and `connect_wait` is the only one. `mu client` takes `--device <name>` or `MU_DEVICE`, whose default is the host name.
- **BREAKING** `client.toml` is removed. `mu client` no longer takes `--config` and reads no configuration file.
- **BREAKING** The notice of #168 about a server that reads another directory is removed, because the client has no directory to compare.
- A new crate, `mu-config`, holds the client configuration fields, the grammar of key names and the names of the entries. The server and the terminal client both use it.

## Capabilities

### New Capabilities

- `config-store`: how the server reads, checks, revises and reloads the configuration directory, and how it reports a bad file.
- `client-bootstrap`: the values that a client needs before a server answers: the server URL with its options, and the device name.

### Modified Capabilities

- `client-config`: the client configuration moves from `client.toml` to the `clients/` layers on the server. A bad binding skips its layer and no longer stops the client.
- `server-process`: the client finds and starts a server through the URL. It no longer passes `--config`. The notice about another directory is removed.
- `world-config`: the configuration directory holds `clients/` in place of `client.toml`. `mu client` no longer takes `--config`.
- `terminal-client`: the first list waits for the time of `connect_wait` in the URL. The client applies a new configuration that the server sends.
- `client-commands`: the scenario that binds a key to `connect` uses a client layer.

## Choices you will notice in use

| Choice | Value |
|---|---|
| Server flag and variable of the client | `--server` and `MU_SERVER`, a URL. `MU_SOCKET` no longer affects `mu client`. |
| Default URL | `unix://` and the default socket path of `mu server`, as today. |
| URL schemes | `unix` only. Any other scheme is refused with a message that names it. |
| URL options | `connect_wait` in seconds, with a fraction allowed, default 5. An unknown option, a host, a user, a password or a fragment is refused. |
| Server flag of `mu server` | Unchanged: `--socket` and `MU_SOCKET`. |
| Device flag and variable | `--device` and `MU_DEVICE`. The default is the host name in lowercase. |
| Device names | 1 to 64 characters from `a` to `z`, `0` to `9`, `-`, `_` and `.`. Any other name is refused. |
| Kinds | `tui` only. |
| OS names | `macos` and `linux`. |
| Layer order | defaults, `global.toml`, `kinds/<kind>.toml`, `os/<os>.toml`, `devices/<device>.toml`. |
| A layer file for an unknown kind or OS | A bad layer, so that a misspelled name is reported. |
| A device with no file | Not an error. Any valid device name can have a file. |
| A bad client layer | Skipped as a whole. One stderr line at start and at reload. A notice on each client that it applies to. |
| `mu client --config` | Removed. A server that the client starts finds its directory from the environment. |
| `mu config reload` | If every file is good, exit 0. If a file is bad, one line for each bad file and exit 1. The good layers apply in both cases. |
| Reload trigger | The command only. No file watcher and no signal. |
| A live client after a reload | Gets the new keys and echo without a restart. |

## Not in this change

- Writes through the API, and compare-and-swap on the revision (#173).
- A reload of worlds or of `server.toml` (#174).
- A file watcher or a signal that starts a reload.
- A URL scheme other than `unix`, a remote client, and authentication.
- A `--listen` URL for `mu server`.
- Layers that combine a kind and an OS.
- The layout, the tab labels and the prominent notice (#172).

## Differences from convention

- A bad world stops the start, but a bad client layer is skipped. A bad world can connect to the wrong host. A bad client layer only loses some key bindings. If the server stopped for it, every client stops.
- The URL carries a client wait in its query. PostgreSQL connection URIs do the same with `connect_timeout`. Docker's `DOCKER_HOST` and gRPC targets use the same `unix:///path` form for the address.

## Impact

- Code: `crates/mu-cli` gets the client start, the URL and the new `config` verb. `crates/mu-server` gets the store, the new calls and the reload. `crates/mu-tui` takes its configuration from the server and applies a new one live. `crates/mu-proto` gets the new calls and messages. The new crate is `crates/mu-config`.
- The crate rules in `crates/mu-cli/tests/workspace_boundary.rs` get the edges to `mu-config`.
- New dependencies: a URL parser and a host name reader. If a crate in the workspace already does the work, no new dependency comes in.
- Documentation: the guides and the architecture pages that name `client.toml`, `--socket` for the client, or the directory notice, and a new decision page for #169.
- Diagrams: the LikeC4 model gets the configuration store and the crate `mu-config`, and two dynamic views show the start of a client and a reload.
