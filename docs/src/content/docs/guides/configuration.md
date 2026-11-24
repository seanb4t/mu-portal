---
title: The configuration files
description: Configure the worlds with TOML files, and see the result with mu world.
sidebar:
  order: 2
---

This guide shows how to configure your worlds. A world is one character on one game. The configuration is a directory of TOML files. `mu server` reads it at the start, and `mu world` shows what it holds. The server owns the configuration, so `mu client` reads no file of it.

## Where the directory is

`mu server` and `mu world` find the directory in this order:

1. The flag `--config <dir>`.
2. The variable `MU_CONFIG`.
3. `$XDG_CONFIG_HOME/mu`.
4. `$HOME/.config/mu`.

`mu` skips a variable that is empty. It also skips an `XDG_CONFIG_HOME` or a `HOME` that is not an absolute path.

If the directory does not exist, there are no configured worlds.

If `mu client` starts a server, that server finds the directory by the same rule, from the environment of the client.

The directory can also hold `server.toml`, the configuration of the server, and the directory `clients/`, the configuration of the clients. Neither makes a world. The guide [The server file and telemetry](/guides/telemetry/) describes `server.toml`. The guide [The client configuration](/guides/client/) describes the layers in `clients/`.

## The layout

The directory holds four kinds of files:

| File | What it configures |
|---|---|
| `global.toml` | Defaults for every world |
| `types/<name>.toml` | Defaults that several games share |
| `games/<game>.toml` | One game: the host, the port and the type |
| `characters/<game>/<character>.toml` | One world: a character on the game |

Each character file makes one world. Its id is `<game>/<character>`. A name holds only `a` to `z`, `0` to `9` and `-`, and starts with a letter. Any other file that ends in `.toml` is an error. The exceptions are `server.toml` at the top of the directory and the files in `clients/`. If the file system ignores case, the same name in another case is that file too. A file `client.toml` at the top of the directory is an error. The error tells you to move its content to `clients/global.toml`, as the guide [The client configuration](/guides/client/#move-from-clienttoml) says.

This example configures the world `alpha/aria`:

```toml
# games/alpha.toml
extends = "tinymux"
host = "mux.example.org"
port = 4201
```

```toml
# characters/alpha/aria.toml
[login]
name = "Aria"
secret = "env:ARIA_PW"
```

## The fields

| Field | Meaning |
|---|---|
| `host`, `port`, `tls` | The place to connect |
| `autoconnect` | Whether the world connects at the start of the server |
| `charset` | The text encoding, `utf-8` or `iso-8859-1` |
| `line_break` | The text that the game reads as a line break inside one command, such as `%r` |
| `columns`, `rows` | The size of the window that the game sees |
| `login.template` | The login line, with `{login}` and `{password}` in it |
| `login.name` | The login name |
| `login.secret` | A reference to the password, such as `env:ARIA_PW` |
| `keepalive.mode`, `keepalive.command`, `keepalive.seconds` | When to send a line that keeps the connection open, the line, and its seconds |
| `reconnect.mode`, `reconnect.initial`, `reconnect.max`, `reconnect.attempts` | Whether the world connects again after a drop, and how long it waits |
| `rules`, `scripts` | Entries, each with an `id` |
| `disable` | The ids of inherited rules and scripts to remove |

A password is never a value in a file. `login.secret` names a place that holds it. Each world must set a host and a port. `login.name` and `login.secret` go together. A world with neither of them logs in by hand.

## Connect, keepalive and reconnect

Each of these fields has a default and a range:

| Field | Default | Range |
|---|---|---|
| `autoconnect` | `false` | `true` or `false` |
| `keepalive.mode` | `"idle"` | `"idle"`, `"interval"` or `"off"` |
| `keepalive.seconds` | none | 1 to 86400 |
| `reconnect.mode` | `"backoff"` | `"backoff"` or `"off"` |
| `reconnect.initial` | `1` | 1 to 3600 |
| `reconnect.max` | `60` | 1 to 86400 |
| `reconnect.attempts` | `30` | 0 to 10000 |

If no file sets one of these fields, `mu world show` names the source `default` for it.

A world with `autoconnect = true` connects at the start of the server. Any other world waits for a client to ask it to connect.

The connection is plain TCP. The login line goes to the game with no encryption, so a host on the network path can read the password. A wrong `host` or `port` sends the line to another service. A world with `tls = true` cannot connect yet. Use a password for the game that you use nowhere else.

The keepalive sends `keepalive.command` to the game. In the mode `"idle"`, the server sends it after `keepalive.seconds` with no write to the game. In the mode `"interval"`, the server sends it every `keepalive.seconds`. In the mode `"off"`, the server sends nothing. Unless the mode is `"off"`, a world that sets the command or the seconds must set both. A world that sets neither has no keepalive.

After a drop that the player did not ask for, the world connects again. The first wait is `reconnect.initial` seconds. Each next wait is twice as long, up to `reconnect.max` seconds. The server adds a random extra time of at most 20% of the wait. `reconnect.initial` must not be greater than `reconnect.max`.

An attempt fails unless its connection stays up for `reconnect.max` seconds. After `reconnect.attempts` failed attempts in a row, the world stops. With `reconnect.attempts = 0`, the world never stops. With `reconnect.mode = "off"`, the world does not connect again.

This game connects at the start of the server. It sends the keepalive line of `tinymux` every 60 seconds. Between two attempts, it waits from 5 to 300 seconds, with no limit on the attempts:

```toml
# games/alpha.toml
extends = "tinymux"
host = "mux.example.org"
port = 4201
autoconnect = true

[keepalive]
mode = "interval"

[reconnect]
initial = 5
max = 300
attempts = 0
```

## The layers

The files of one world merge in this order, from first to last:

1. `global.toml`.
2. The types, from the most general to the type that the game names.
3. The game.
4. The character.

A field takes its value from the last file that sets it. A table, such as `login`, merges field by field. Thus the game can set the login template and the character can set the name.

## Types and extends

A game must name its type with `extends`. A type in `types/` names its parent with `extends` too. The chain ends at a built-in type or at a type in `types/` with no `extends`. A built-in type sets only the quirks of its server, for example the encoding, the login template, the keepalive and the line break. It does not set a preference such as `columns`, so a value in `global.toml` reaches each world.

The built-in types are:

| Type | Line break | Notes |
|---|---|---|
| `tinymux` | `%r` | |
| `pennmush` | `%r` | Extends `tinymux`. |
| `rhostmush` | `%r` | Extends `tinymux`, with the keepalive line `idle`. |
| `aresmush` | `%r` | |
| `evennia` | `\|/` | |

The types `pennmush` and `rhostmush` come from the documentation of those servers. No live game checked them. If a game differs, set the field in its game file.

## The line break of a world

A pose can have several lines. The server sends it to the game as one command, and writes the `line_break` of the world in place of each line break. A built-in type sets the value for its server. A game file or a type in `types/` can set another value of 1 to 16 characters:

```toml
# games/alpha.toml
line_break = "%r%t"
```

A world with no `line_break` refuses a text of several lines. That is a world whose type chain ends at a type in `types/` that sets none.

A file in `types/` cannot have the name of a built-in type. A loop in the chain is an error. The global file and the character files cannot use `extends`.

## Rules, scripts and disable

Rules and scripts do not replace each other. Each layer adds its entries to the entries of the layers before it. An entry with the id of an inherited entry replaces that entry in its place. To remove an inherited entry, put its id in `disable`:

```toml
# characters/alpha/aria.toml
disable = ["hl-ooc"]
```

An id in `disable` must name an inherited entry. A rule and a script cannot have the same id.

## See the result

To list the configured worlds, run:

```bash
mu world list
```

To see one world after the merge, run:

```bash
mu world show alpha/aria
```

The output is TOML. A comment after each field names the file that set it, or `default`. A comment above each rule and script names its file. `mu world show` does not read the password.

If a file has an error, `mu world` and `mu server` print the file, the field and the problem, and stop with the exit status 1.

## A change while the server runs

The server reads the worlds and `server.toml` once, at its start. A change to one of these files applies at the next start of the server. The command `mu config reload` applies only the files in `clients/`. It prints one line for each changed world file and for a changed `server.toml`. The line says that the change applies at the restart of the server.
