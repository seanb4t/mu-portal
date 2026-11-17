---
title: World-type profiles
description: A world inherits its configuration through layers, from global defaults to one character.
sidebar:
  order: 9
---

## Decision

A world is one character on one game. A game is one server, for example one TinyMUX game. One game can have several characters.

Each world has a type, for example TinyMUX, AresMUSH or Evennia. A profile for each type holds the differences between the servers. Examples are the login command, the keepalive, the text encoding and the supported telnet options.

The configuration of a world comes from five layers. Each layer names its parent with `extends`.

| Layer | Holds | Example |
|---|---|---|
| Global | The defaults of the owner for every world | `columns = 100` |
| Built-in type | The quirks of one server family. It ships inside the binary. | TinyMUX reads input as Latin-1 |
| User type | Defaults that several games share | a set of roleplay highlights |
| Game | One server | the host, the port and TLS |
| Character | One login | the login name and a reference to the password |

The layers merge in this order, from global to character:

- A field, for example the window width `columns`, takes its value from the last layer that sets it.
- Rules, macros and scripts accumulate. Each layer adds to the ones before it. A layer removes an inherited rule with `disable` and the id of the rule.

A built-in type sets only quirk fields, for example the encoding, the telnet options, the login and the keepalive. It does not set a preference such as `columns`. Thus a global default reaches every world, but a global default cannot override a quirk of a type by accident.

The files are TOML. The `figment2` crate merges the fields of the layers and records which file set each value. `figment2` is a fork of `figment` with the same API. The project uses the fork because `figment` has had no release for a long time and depends on an old `toml` crate. The rules and the scripts stay out of the merge, because a merge replaces a whole array. A loop in the `extends` chain is an error.

A password is never a value in a file. A character names a secret, and the server reads it through its secret store. A layer from a shared plugin cannot set the host, the port, TLS, the login name, the secret or the login template.

A later layer source can be Lua. It returns a table of fields, and the same merge rules apply to it. The [Lua decision](/decisions/lua-scripting/) describes it.

## Reason

The game servers differ in small ways that break a client that treats them alike. The telnet spike found examples. TinyMUX stops reading input after it accepts UTF-8 through CHARSET. AresMUSH asks for `utf-8` in lowercase. Evennia starts compression in the middle of a read. A profile puts those differences in data and keeps them out of the connection code. A new server type then needs a new profile and no new code path.

The owner plays several characters on one server. Without a character layer, each character repeats the whole game configuration.

TOML with `extends` covers the inheritance with no new language. Pkl gives typed inheritance, but it needs an external `pkl` program on each server host, and the product ships as one binary. A new DSL is a second language to learn and keep. Code generation fixes the types at build time, so a player cannot add one.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it. The Phase 1 exploration added the layers, and issue 61 tracks the changes that build them.
