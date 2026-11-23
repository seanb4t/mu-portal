# Proposal: client-shell

Tracking issue: #101. Part of the Phase 1 plan in #61.

## Why

The terminal client cannot start an evening. A configured world connects only through the API, and the client has no command, no input history and no configuration. The player cannot change a key, cannot recall a pose that they sent, and cannot dim the echo of a pose (#140). After a refusal, the sends that come back run together in the editor (#151).

This change is the first of two that #101 splits into. It gives the client a configuration file, the keys that it reads from that file, client commands, and input history. The second change, `client-layout`, adds the layout from configuration, the tab labels, the statusline and the command palette on top of it.

## What Changes

- The client reads `client.toml` from the configuration directory. `mu client` takes `--config <dir>`, and finds the directory by the same rule as `mu server` and `mu world`. A missing file gives the defaults. A file with an unknown key or a bad value stops the client before the screen opens, and stderr names the file and the key.
- One registry names each thing that the player can ask the client to do. Each entry has a name, a description and its argument. The keymap, the client commands and the later palette all read the registry.
- The `[keys]` table of `client.toml` binds a key to a registry entry, or unbinds it with `"none"`. The bindings change the default keymap. They do not replace it.
- A one-line text that starts with `/` is a client command. The commands are `/connect [world]`, `/disconnect [world]`, `/focus <world>`, `/quit` and `/help`. A text that starts with `//` goes to the game with one `/` removed. An unknown command shows a line with the prefix `-- `, and its text stays in the editor.
- Each world has its own pose editor and its own input history in memory. The editor on screen is the editor of the world in focus. Up on the first line of the editor recalls the older send, and Down on the last line the newer one.
- Alt+1 to Alt+9 give the focus to the world at that place in the top row. Alt+Home and Alt+End scroll the pane to its first and last line.
- The echo line of a send (#140) follows `echo` in `client.toml`: `show`, `dim` or `hide`. The default is `dim`. A joined pose shows as one row for each of its lines, split at the `line_break` of its world.
- A change to existing behavior (no release yet): after a refusal, a timeout, a stream end or a reset, only the first returned send goes back in the editor of its world. The other returned sends and the text of that editor go into the history of that world. Up then recalls them one at a time (#151). A notice says how many wait there.
- The client opens the screen before the first list of the worlds and shows `listing worlds` in the top row while it waits (#161, third item).

## Choices you will notice

- The command prefix is `/`, and `//` sends one `/`. The prefix is not configurable.
- A command is only a text of one line. If a text has several lines, it never runs a command. This holds also when a line starts with `/`. Alt+Enter on lines that start with `/` sends them to the game.
- `/connect` and `/disconnect` with no world act on the world in focus. A trailing space is no argument.
- A second `/quit` confirms a quit while texts wait. Typing it does not cancel the first one.
- The echo default is `dim`. Before this change it was `show`.
- History is per world, kept in memory only, at most 1000 sends for each world. It does not keep an empty send or a send equal to the one before it. Quitting the client loses it.
- Up and Down recall history only from the first and the last line of the editor. On other lines they move the cursor.
- A bad `client.toml` stops the client. The client does not start with the defaults and a warning. A `--config` directory that does not exist is not an error: the client warns on stderr and starts with the defaults.
- After a refusal, the editor gets one send back, not all of them. The rest wait in history.
- Each world has its own pose editor. A half-typed pose stays with its world when the focus moves, and the other world shows its own editor. Text typed before any world has the focus goes to the first world that gets it.
- A refusal from a world out of focus does not touch the editor on screen. The refused text waits in the editor of its world, and the top row shows `*` after that world.
- The key names of `[keys]` are lowercase and joined with `+`, for example `alt+enter` and `ctrl+shift+z`. The `+` key is `plus`. `"none"` makes a key do nothing.
- A key in `[keys]` cannot run an entry that needs a world, such as `focus`. A control character is not a key name. Shift with a key that is not a letter, such as `shift+1`, `shift+space` or `shift+plus`, is refused, because the terminal sends the shifted character instead.
- Shift+Up, Ctrl+Up and Shift+Down keep their editor moves. Only plain Up and Down recall history.
- A returned text goes in the editor with the cursor at its start, so Up recalls the next one at once.
- Tab, Shift+Tab and Ctrl+Space get no binding, because completion will use them later.
- The palette, the layout file, the tab labels and the statusline are not in this change.

## Not in this change

- The layout from configuration, tab labels, the statusline and the command palette: `client-layout`, the second change of #101.
- Completion and suggestions for command names and arguments, such as a fuzzy list of worlds. The design keeps room for them: typed arguments, a source of values for each kind, and Tab, Shift+Tab and Ctrl+Space left free.
- History on disk, and a history shared across worlds.
- A configurable command prefix, and commands defined by the player or by Lua (Phase 4).
- The queue of sends on disk (#156).
- The other two items of #161: a panic in a spawned call, and a world that a list does not hold.
- Spawns (#102).

## Where it differs from the convention of the field

- TinyFugue keeps one input line for all worlds. This client keeps one pose editor for each world, so a text that comes back from a world cannot be sent to another by mistake.
- TinyFugue and BeipMU keep one input history for all worlds. This client keeps one for each world, because a pose for one game is rarely right for another.
- TinyFugue lets a multi-line paste run several `/` commands. Here only a text of one line runs a command, so a pasted pose cannot run a command by accident.

## Capabilities

### New Capabilities

- `client-config`: the file `client.toml`, how the client finds and checks it, its `[keys]` table and its `echo` field.
- `client-commands`: the registry of what the player can ask the client to do, the `/` commands, and how a key or a command reaches an entry.

### Modified Capabilities

- `terminal-client`: input history, the focus by number, the scroll keys, the echo line, the return of refused sends to history, and the screen during the first list.
- `world-config`: `mu client` takes `--config` and finds the directory by the same rule. `client.toml` is a file of the directory that makes no world.

## Impact

- `crates/mu-tui`: the registry, the keymap from configuration, commands, history, echo, the return path of `update.rs`, and the startup in `run.rs`. It gains `serde` for the configuration types.
- `crates/mu-cli`: `mu client --config`, which it also passes to a server that it starts, and the read of `client.toml` with `mu_core::world::parse_toml`, as `server.toml` is read. The first list of the worlds gets the limit of 5 s.
- `crates/mu-core`: the world resolver (`world/dir.rs`) accepts `client.toml` beside `server.toml`.
- `crates/mu-cli/tests/workspace_boundary.rs`: `serde` joins the allowed crates of `mu-tui`.
- Docs: the terminal client page, a page for `client.toml`, and the configuration guide, which names the top-level files.
- No change to the API or the server.
