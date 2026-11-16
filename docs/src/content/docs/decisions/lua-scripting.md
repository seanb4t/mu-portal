---
title: Lua 5.4 scripting
description: Scripts use stock Lua 5.4 for configuration and for behavior, with limits on shared plugins.
sidebar:
  order: 8
---

## Decision

Scripts use stock Lua 5.4. The scripts of the owner are trusted. A shared plugin runs with limits: no file access, no process access, no module loading, no bytecode, a memory limit and an instruction limit.

Simple triggers and highlights do not need a script. A rule file in TOML states them. The rules use the priority and fall-through model of TinyFugue.

Scripts run on the server and never block a client.

### Two uses of Lua

| | Configuration Lua | Behavior Lua |
|---|---|---|
| Runs | Once, at the load or the reload of a world | On each event and each input |
| Does | Returns a table of fields | Registers triggers, macros, hooks and handlers |
| Effect | One more layer of the [world configuration](/decisions/world-type-profiles/) | Acts through commands |
| Limits | Pure: no I/O, no time, no random numbers, no network | The limits above, for a shared plugin |

Every layer, from global to character, can have a TOML file, a Lua file or both. Configuration Lua can read the values of its parent layer.

### Hooks and handlers

A hook runs inside the pipeline, before an event is final. It can pass, replace or gag a line from the game or an input from the player. A hook must finish within a small budget, because the session waits for it.

A handler runs after the event is in the journal. It cannot change the event. It returns commands, for example send a line, echo, notify, emit a custom event, set a variable or start a timer.

A script never does I/O inside a session. It returns commands, and the server checks and runs them. The result of each command is a new event in the journal.

### Events and calls

Scripts see the same events that the journal records. These are connect, disconnect, a line, a prompt, a sent input, GMCP and MSDP data, a timer, a custom event and a script error. Each event has a source: the game, the player, a script, the system or a credential.

A Lua call starts from one of these sources:

- an event
- a timer
- a client command that matches a macro
- the API call `Invoke`, from a client or from an agent
- a custom event that another script emits

### Lua states

Each world has one Lua state. It runs inside the session of that world, so a world handles its events one at a time, in sequence order. A script for one character cannot touch the state of another world. A slow script stalls only its own world.

The global layer can also define server scripts. They run once, in one state of the server, and they see the events of every world. They can send to any world.

A command from a handler makes later events. It does not call the handler again at once. A limit on the depth of custom events stops a loop.

A script error is an event that names the file and the line. It does not stop the session.

### Trust

- A shared plugin cannot set the host, the port, TLS or a credential of a world. It can read the login name.
- No Lua state ever holds a password, trusted or not. The session fills the login template itself. A script that starts a login sends a `login` command, and the server adds the credentials.
- The journal records the login line with the password replaced by `****`.
- A script can send to its own world. A send to another world needs a trusted layer of the owner, never a shared plugin.

The runtime is the `mlua` crate with Lua vendored, so the host needs no Lua library.

## Reason

Lua is the scripting language that MU* players already know from MUSHclient. Stock Lua keeps existing habits and libraries. The limits exist because a plugin from another person is not trusted code.

Configuration Lua runs once and returns data, so each value of a world still comes from one file. Behavior Lua acts only through commands, so a test can feed events in and check the commands that come out. One state for each world keeps the worlds apart. A password that never enters Lua cannot leak from a script.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it. The Phase 1 exploration added the two uses, the hooks, the events and the trust rules, and issue 61 tracks the changes that build them.
