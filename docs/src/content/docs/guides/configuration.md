---
title: The configuration files
description: Configure the worlds with TOML files, and see the result with mu world.
sidebar:
  order: 2
---

This guide shows how to configure your worlds. A world is one character on one game. The configuration is a directory of TOML files. `mu server` reads it at the start, and `mu world` shows what it holds.

## Where the directory is

`mu` finds the directory in this order:

1. The flag `--config <dir>`.
2. The variable `MU_CONFIG`.
3. `$XDG_CONFIG_HOME/mu`.
4. `$HOME/.config/mu`.

`mu` skips a variable that is empty. It also skips an `XDG_CONFIG_HOME` or a `HOME` that is not an absolute path.

If the directory does not exist, there are no configured worlds.

## The layout

The directory holds four kinds of files:

| File | What it configures |
|---|---|
| `global.toml` | Defaults for every world |
| `types/<name>.toml` | Defaults that several games share |
| `games/<game>.toml` | One game: the host, the port and the type |
| `characters/<game>/<character>.toml` | One world: a character on the game |

Each character file makes one world. Its id is `<game>/<character>`. A name holds only `a` to `z`, `0` to `9` and `-`, and starts with a letter. Any other file that ends in `.toml` is an error.

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
| `charset` | The text encoding, `utf-8` or `iso-8859-1` |
| `columns`, `rows` | The size of the window that the game sees |
| `login.template` | The login line, with `{login}` and `{password}` in it |
| `login.name` | The login name |
| `login.secret` | A reference to the password, such as `env:ARIA_PW` |
| `keepalive.command`, `keepalive.seconds` | A line to send, and the time between two sends |
| `rules`, `scripts` | Entries, each with an `id` |
| `disable` | The ids of inherited rules and scripts to remove |

A password is never a value in a file. `login.secret` names a place that holds it. Each world must set a host and a port. `login.name` and `login.secret` go together. A world with neither of them logs in by hand.

## The layers

The files of one world merge in this order, from first to last:

1. `global.toml`.
2. The types, from the most general to the type that the game names.
3. The game.
4. The character.

A field takes its value from the last file that sets it. A table, such as `login`, merges field by field. Thus the game can set the login template and the character can set the name.

## Types and extends

A game must name its type with `extends`. A type in `types/` names its parent with `extends` too. The chain ends at a built-in type: `tinymux`, `aresmush` or `evennia`. A built-in type sets only the quirks of its server, for example the encoding, the login template and the keepalive. It does not set a preference such as `columns`, so a value in `global.toml` reaches each world.

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
