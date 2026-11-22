---
title: First run
description: Start a server with two fake worlds and attach the terminal client.
sidebar:
  order: 1
---

This guide starts `mu server` with two fake worlds and attaches `mu client` to it. A fake world is a game that runs inside the server. It needs no network and no account on a real game.

## Before you start

Build the binary from the root of the repository:

```bash
cargo build -p mu-cli
```

The binary is `target/debug/mu`. The steps below use two terminals.

## Start the server

A fake world takes an id of the form `<game>/<character>` and the name of an environment variable that holds the password. The server reads the password from that variable. It never writes the password to an event or to its log.

In the first terminal, set the two passwords and start the server:

```bash
export ALPHA_PW='s3cret-pw' BETA_PW='s3cret-pw'
target/debug/mu server \
  --fake-world alpha/aria=env:ALPHA_PW \
  --fake-world beta/bryn=env:BETA_PW
```

The server writes one line to stderr when it serves. The line names the socket, for example `mu server: serving the API on /run/user/1000/mu/server.sock (process 4242)`. If `XDG_RUNTIME_DIR` is not set, the socket is in the temporary directory, in `mu-<uid>`.

`--fake-world` is a test flag. It does not show in `mu server --help`.

The server also holds the worlds of the [configuration directory](../configuration/). A configured world connects at the start if it sets `autoconnect = true`, or when a client asks it to connect. A world with `tls = true` cannot connect yet. A fake world cannot have the id of a configured world. If the configuration directory has the world `alpha/aria` or `beta/bryn`, the server stops.

## Attach the client

In the second terminal, start the client:

```bash
target/debug/mu client
```

`mu` with no verb also starts the client. The client finds the server on the same socket. If no server answers, the client starts one in the background with only the configured worlds. That server writes its output to `server.log` next to the socket.

The top row lists the worlds. The world in focus is in square brackets: `[alpha/aria] beta/bryn`. The rows below show the lines of the world in focus. The bottom row is the pose editor, where you write the text to send. It grows with its text.

## Use the client

| Key | Action |
|---|---|
| Enter | Sends the text of the pose editor to the world in focus, and returns its lines to the bottom |
| Ctrl+J | Inserts a line break in the pose editor |
| PageUp | Scrolls the lines back by the height of the pane less one row |
| PageDown | Scrolls the lines forward by the height of the pane less one row |
| Ctrl+N | Moves the focus to the next world |
| Ctrl+P | Moves the focus to the world before |
| Ctrl+C | Quits the client. The server and its worlds keep running. |

The [input editor decision](/decisions/input-editor/) lists the other keys of the pose editor.

Type `hello` and press Enter. The screen shows `> hello`, and then the answer of the fake world, `You say, "hello"`. A line from the system has the prefix `-- `, for example `-- connect aria ****`. The server replaced the password with `****`.

Each step of the mouse wheel scrolls the lines by 3 rows. When the lines do not fit, a scroll bar shows in the last column. If you scroll back, new lines do not move the rows on the screen. If you scroll forward to the newest line, the pane follows new lines again. Each world keeps its own scroll position when the focus moves.

To copy text, drag over it with the left mouse button. The client shows the selection in reverse. When you release the button, the client sends the text to the clipboard of the terminal with an OSC 52 sequence. A click with no drag clears the selection. Rex takes the copy with no setting, but only from the pane in focus.

The client reads the mouse, so a plain drag does not make a selection of the terminal. In Rex, hold Shift while you drag to select text with the terminal.

A world out of focus that gets a line shows `*` after its id. The mark goes when that world gets the focus.

## Stop the server

Press Ctrl+C in the first terminal. The server disconnects each world, removes its socket, and exits.
