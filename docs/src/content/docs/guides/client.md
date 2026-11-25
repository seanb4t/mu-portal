---
title: The client configuration
description: Choose the server and the device of mu client, and change its keys, its echo line, its tab label and its statusline with the client layers of the server.
sidebar:
  order: 4
---

This guide shows how `mu client` finds its server and how long it waits for it. It also shows how to change the keys, the echo line and the tab label of the client. The guide [Layouts](/guides/layouts/) shows how to change the screen. The echo line is the row that shows each text that you send, after `> `.

The server owns the configuration. The client reads no configuration file. It gets its keys, its echo line, its layout, its tab label and its statusline from the server that it attaches to.

## The server URL

The client finds the server through a URL. The URL is `unix://`, then the absolute path of the socket of the server, then an optional query. A space or another special character in the path goes in as a percent escape, such as `%20` for a space.

The client takes the URL in this order:

1. The flag `--server <URL>`.
2. The variable `MU_SERVER`. An empty value counts as unset.
3. `unix://` and the default socket of `mu server`. That socket is `mu/server.sock` under `XDG_RUNTIME_DIR`. If `XDG_RUNTIME_DIR` is not set, the socket is in the directory `mu-<uid>` under the temporary directory.

This example names a socket and a wait of two seconds:

```bash
mu client --server 'unix:///run/user/1000/mu/server.sock?connect_wait=2'
```

The URL cannot hold a host, a user, a password or a fragment. A scheme other than `unix` and a relative path are also errors. For each of these errors, the client exits with status 1 before it connects, and the error names the URL.

## The wait for a server

The query option `connect_wait` is the longest time, in seconds, that the client waits for a server to answer. The default is 5. A fraction such as `0.5` is a valid value. A value of 0 or less, a value under 1 ns, and a value that is not finite are errors. The only query option is `connect_wait`. An unknown option and an option that occurs twice are also errors. Each error names the option, and the exit status is 1.

Before the screen opens, the client lists the worlds and gets its configuration from the server. All of these steps share the one wait. If nothing listens on the socket, the client starts a server for that socket and waits for it. That server finds its configuration directory from the environment, as `mu server` does. It writes its output to `server.log` next to the socket. If no server answers in time, the client exits with status 1, and the error names the socket and the time.

If a server listens but the list fails for another reason, the client does not start a second server. It writes the error and exits with status 1.

## The device name

The device name tells the server which machine the client runs on. The server uses it to pick the device layer of the client. The client takes the name in this order:

1. The flag `--device <name>`.
2. The variable `MU_DEVICE`. An empty value counts as unset.
3. The host name of the machine, in lowercase.

A device name has 1 to 64 characters. Each character is `a` to `z`, `0` to `9`, `-`, `_` or `.`. Any other name stops the client with status 1, and the error names it. If the host name is not a valid device name, the client stops, and the error names `--device`. In that case, set `--device` or `MU_DEVICE`.

## The client layers

The client configuration is in the directory `clients/` in the configuration directory of the server. The guide [The configuration files](/guides/configuration/) tells where that directory is. Each file in `clients/` is a layer. A layer is a file that sets some fields for some clients. Each layer is optional:

| Layer | Applies to |
|---|---|
| `clients/global.toml` | Every client |
| `clients/kinds/<kind>.toml` | Each client of one kind. The only kind is `tui`, the terminal client |
| `clients/os/<os>.toml` | Each client on one OS: `macos` or `linux` |
| `clients/devices/<device>.toml` | Each client with that device name |

The layers merge in this order, from first to last: the defaults, the global layer, the kind, the OS and the device. A field takes its value from the last layer that sets it. The table `[keys]` merges key by key. So a later layer replaces the binding of each key that it names, and keeps the other bindings.

```toml
# clients/global.toml
echo = "show"

[keys]
"alt+n" = "focus-next"
"ctrl+n" = "none"
```

```toml
# clients/devices/laptop.toml
[keys]
"alt+n" = "none"
```

A layer holds only these fields:

| Field | Meaning |
|---|---|
| `echo` | `show`, `dim` or `hide`. The default is `dim` |
| `layout` | The name of the layout of the screen. The default is `classic` |
| `[tabs]` | The table of the tab label, with the field `label` |
| `[statusline]` | The table of the statusline, with the lists `left` and `right` |
| `[world-layouts]` | A table from a world id or a game name to the name of a layout |
| `[keys]` | A table from a key name to the name of an entry, or to `none` |

The guide [The layout of the client](/guides/layouts/) describes `layout` and `[world-layouts]`.

The server reads the layers at its start. If you change a layer while the server runs, run `mu config reload`. Each attached client gets the new configuration with no restart.

## The echo line

With `show`, the echo line has the normal style. With `dim`, it has the dim style. With `hide`, the client shows no echo line. The game can still show your text in its own words, such as `You say, "hello"`.

## The tab label

The field `label` of the table `[tabs]` sets the text of each world tab. The default is `{world}`, which is the world id. A label has 1 to 64 characters.

```toml
# clients/global.toml
[tabs]
label = "{number}:{character}"
```

| Field | Value |
|---|---|
| `{world}` | The world id |
| `{game}` | The part of the id before the `/` |
| `{character}` | The part of the id after the `/` |
| `{number}` | The place of the world in the tabs, from 1 |

Write `{{` for the character `{`, and `}}` for the character `}`. The client keeps its marks around the label. The marks are square brackets for the world in focus, `*` for unseen lines, and the count of the sends that wait.

## The statusline

The block `statusline` shows one row of segments about the world in focus. A layout must hold the block, as the built-in layout `status` does. The table `[statusline]` picks the segments of the left side and of the right side.

```toml
# clients/global.toml
layout = "status"

[statusline]
left = ["world", "state"]
right = ["idle"]
```

| Segment | Value |
|---|---|
| `world` | The world id |
| `state` | `connecting`, `connected`, `disconnected` or `reconnecting` |
| `connected` | The time since the world connected, while it is connected |
| `idle` | The time since the last send to the world from any client |
| `typed` | The count of characters in the editor with the input focus |

The defaults are `left = ["world", "state"]` and `right = ["idle"]`. A segment with no value does not show. A time shows as `M:SS` under one hour, and as `H:MM:SS` from one hour. A layer that sets one list replaces the whole list of an earlier layer, and keeps the other list.

If the two sides do not fit in the row, the right side shows in full and the left side is cut. With no world in focus, the row shows `no world`.

## The keys table

Each line of `[keys]` binds one key to one entry. An entry is one thing that the client can do, such as `focus-next` or `send`. The command `/help` in the client lists each entry with what it does. A binding replaces the default binding of that key. The value `none` removes the binding, so the key does nothing. Each key that no layer names keeps its default.

An entry that needs a world, such as `focus`, cannot go on a key. Run it as a command instead, such as `/focus beta/bryn`.

### Key names

A key name is zero or more modifiers, then one key, joined with `+`. The modifiers are `ctrl`, `alt`, `shift` and `super`. Their order does not matter. Each modifier can occur once.

The key is one of these:

- One character, such as `n` or `1`. A letter is lowercase. To name a capital letter, use `shift` and the lowercase letter, such as `shift+n`.
- `plus` for the `+` key, and `space` for the space bar. The names `.` and `period` are the same key.
- The name of a special key. These are the special keys:

  | Group | Names |
  |---|---|
  | Edit | `enter`, `tab`, `backspace`, `esc`, `insert`, `delete` |
  | Move | `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown` |
  | Function | `f1` to `f12` |

`shift` goes only with a letter or a special key. The terminal sends Shift and another character as a different character, such as `!` for Shift+1. To bind that key, bind `!` itself.

A control character, such as a tab or a carriage return, is not a key name. Use the name of the key, such as `tab` or `enter`.

### Kept keys

The key Alt+E runs `editor-next`, which gives the input focus to the next editor of the layout. The entries `pane-next` and `layout-next` have no default key. The guide [The layout of the client](/guides/layouts/) describes them.

The default keys leave Tab, Shift+Tab and Ctrl+Space with no binding. A later version of the client uses them for completion. You can bind them now, but a later version can change what they do by default.

### Keys that a terminal does not send

Some terminals do not send each key. In Terminal.app on macOS, the Option key types a character unless the setting "Use Option as Meta key" is on. So Alt+1 to Alt+9, Alt+Home and Alt+End do not reach the client. Terminal.app also sends Ctrl+Shift+Z as Ctrl+Z, so the key runs undo and not redo. In such a terminal, bind those entries to other keys in `[keys]`.

## A bad layer

A bad layer does not stop the server or the client. The server skips the whole layer and uses the good layers. A skipped layer counts as a file that does not exist. No field and no binding of a bad layer applies. The server writes one line to stderr that names the file and the reason. Each client that the layer applies to shows a notice with the same text. The client shows the notice once, until you fix the layer.

These layers are bad:

- A file that the server cannot read.
- A file that is not TOML. The notice names the file and the line.
- An unknown field or table, such as `[server]`, or a value of the wrong type. The notice names the file and the field.
- An `echo` value that is not `show`, `dim` or `hide`.
- A layer with a bad binding. The notice names the file and the binding. These bindings are bad: a bad key name, an unknown entry, and an entry that needs a world. Two names of the same key in one layer are also bad, such as `ctrl+shift+z` and `shift+ctrl+z`.
- A file under `clients/` that is not one of the layer files, such as `clients/extra.toml` or `clients/os/windows.toml`.

## See the configuration of a client

To see the configuration that one client gets, run:

```bash
mu config show --device laptop
```

The first line is `# revision <text>`. A change to a layer of that client gives a new revision. The output is TOML. A comment after each field and each binding names the layer that set it, or `default`. The notices of the bad layers of that client go to stderr.

`mu config show` takes the flags `--server` and `--device` with the same rules as `mu client`. It also takes `--kind`, with the default `tui`, and `--os`, with the default of the machine. It never starts a server. If no server answers, it exits with status 1, and the error names the URL.

## Reload the configuration

To make the server read its configuration directory again, run:

```bash
mu config reload
```

`mu config reload` takes the flag `--server` with the same rules as `mu client`. It never starts a server. The server applies the new client layers, and each attached client gets its new keys, echo line, layout, tab label and statusline.

The server does not apply a change to a world or to `server.toml`. For each such file that changed after the start of the server, the command prints one line on stdout: `<path>: the change applies at the restart of the server`. To apply such a change, stop the server and start it again.

The command writes one line to stderr for each bad file. If a file is bad, the exit status is 1. Otherwise it is 0. The good layers apply in both cases.

## Move from client.toml

An earlier `mu client` read the file `client.toml` at the top of the configuration directory. The client no longer reads it. If `client.toml` is still there, `mu server` and `mu world` stop with an error that names the file and `clients/global.toml`. To move it:

1. Move the content of `client.toml` to `clients/global.toml` in the configuration directory of the server.
2. Remove the table `[server]` from `clients/global.toml`. A layer cannot hold it.
3. If you set `connect_wait_seconds` in that table, use the option `connect_wait` of the server URL instead.
4. If a script sets `MU_SOCKET` or `--socket` for the client, change it to `MU_SERVER=unix://<path>` or `--server unix://<path>`. `MU_SOCKET` and `--socket` still name the socket of `mu server`.

`mu client` no longer takes `--config` or `--socket`. Each of them is a usage error with exit status 2.

## Before a downgrade

The guide [The layout of the client](/guides/layouts/#before-a-step-back-to-an-earlier-mu) gives the steps for a `mu` from before the layouts.

An earlier `mu` does not know `clients/`. It reads each file in it as a stray file, and `mu world` and `mu server` stop with an error. Before you install an earlier binary, move `clients/` out of the configuration directory.
