---
title: Configuration ownership
description: The server owns all configuration, the worlds and the client layers too. A client brings only its server URL and its device name.
sidebar:
  order: 23
---

## Decision

The server owns all configuration. The configuration store, a part of the server, reads and checks the whole configuration directory. That directory holds the worlds, `server.toml` and the client layers in `clients/`. A client reads no configuration file. It gets its configuration from the server through the API.

### The bootstrap values

A client needs two values before a server answers. These values are the bootstrap values, and they come only from flags and the environment:

| Value | Flag | Variable | Default |
|---|---|---|---|
| The server URL | `--server` | `MU_SERVER` | `unix://` and the default socket of `mu server` |
| The device name | `--device` | `MU_DEVICE` | The host name, in lowercase |

The server URL is `unix://` and an absolute path. Its query holds only connection options. The one option is `connect_wait`, the longest wait in seconds for an answer of the server. An empty variable counts as unset. The guide [The client configuration](/guides/client/) gives the rules of each value.

### The layer order

The client configuration merges from these layers, from first to last:

1. The defaults.
2. `clients/global.toml`.
3. `clients/kinds/<kind>.toml`. The only kind is `tui`.
4. `clients/os/<os>.toml`, for `macos` or `linux`.
5. `clients/devices/<device>.toml`.

A field takes its value from the last layer that sets it. The table `[keys]` merges key by key. The call `GetClientConfig` returns the merged configuration of one kind, one OS and one device. The reply also holds the revision, the layer that set each field, and the notices.

### A bad client layer

A bad world or a bad `server.toml` stops the start of the server. A bad client layer does not. The server skips the whole layer, writes one line to stderr, and sends a notice to each client that the layer applies to. The client runs with the good layers.

### A reload

`mu config reload` makes the server read its directory again. The server applies the client layers, and each attached client gets the new configuration with no restart. A change to a world or to `server.toml` applies only at the restart of the server. The page [Server and clients](/architecture/server-and-clients/) shows the start of a client and a reload.

## Reason

Before this decision, each `mu client` read its own configuration file. So two clients of one server with two different files showed different keys and a different echo line. A remote client has no file to read. One owner of the configuration gives each client of one server the same configuration. It also gives one place for a later write through the API.

The server skips a bad client layer and does not stop. A bad world can connect to the wrong host, so it stops the start. A bad client layer only loses some key bindings. If the server stopped for it, every client of that server stops.

The client needs the server URL and the wait before a server answers. So these values cannot come from the server.

## Later work

- Writes through the API, with a compare of the revision before each write (issue 173). The server checks each write with the same checks as a layer file. Remote clients and an MCP server use these writes.
- A prominent notice for a bad layer (issue 172).
- A reload of the worlds and of `server.toml` (issue 174).

## Source

Issue 169. The OpenSpec change `config-store` recorded it.
