---
title: Terminal client architecture
description: The terminal client uses the Elm architecture on the Termina backend, with widgets that keep their state by pane id.
sidebar:
  order: 22
---

## Decision

The terminal client uses the Elm architecture (TEA) at the top level, on the Termina backend of ratatui.

- One model holds the focus, the worlds, the input line, the layout tree and the state of each pane by pane id.
- Every input becomes a `Msg`: a terminal event, an event of the server, a reply to a call or a reply to a terminal query.
- A keymap is a table from a key with its modifiers to an `Action`. The client builds the default table in code.
- One `update` function changes the model and returns a list of effects. It does no I/O. The event loop performs each effect: a call of `mu-client`, a write to the terminal, or the end of the loop.
- Only `update` changes the model, with one exception. `view` records what each pane drew, and it returns a scrolled pane to the bottom when the newest row shows. The result of the startup probe is a `Msg` too. The event loop sends it before the first input event, and it draws no frame until `update` has handled it. So the first frame already has the widths and the graphics flag that the probe learned.
- One `view` function walks the layout tree and renders each widget. Each widget is a ratatui `StatefulWidget`, and its state knows nothing of the app.
- The layout is a tree of data with three node kinds: a split, tabs and a leaf.

The [terminal client architecture page](/architecture/terminal-client/) shows the parts and the path of one key press.

The backend is Termina, through the `termina` feature of ratatui. The client does not ask the terminal for its name with XTVERSION, because Termina drops the reply.

## Reason

A comparison checked three things in each backend, in Rex. The spike in `spikes/terminal-backend` ran the checks. To see the version of each backend that it used, run `cargo tree --manifest-path spikes/Cargo.toml -p terminal-backend --depth 1`.

| Check | crossterm | Termina |
|---|---|---|
| The replies to terminal queries arrive as events, and a key pressed during the queries still arrives as a key | Fails. The mode report, the cursor position and the device attributes reply are dropped. The XTVERSION reply arrives as 23 keys, Alt+P first | Passes for the mode report, the cursor position and the device attributes reply. The XTVERSION reply is dropped, with no stray key |
| Shift+Enter arrives with its modifier, and a paste of several lines arrives as one event | Passes | Passes |
| The event stream works in `tokio::select!` beside a second stream | Passes | Passes |

Neither backend passed all three checks. Termina gives every reply that the client reads as an event, so the client can drop a late reply by its kind. With crossterm, a late XTVERSION reply reaches the input line as keys. The terminal name is not part of any width rule.

One `update` at the top keeps the keymap in one place, and later the Lua scripts too. The other shape that the comparison considered gives each component its own update. That shape spreads the keymap over the components.

## Not tested

- The comparison ran in Rex on macOS only. It tests how each library parses bytes, and that does not depend on the terminal.
- The paste of check 2 was the bracketed paste bytes that `rex send` wrote. The paste action of the Rex app did not reach the pane.
- `ratatui-termina` needs an older minor version of termina than the newest one. No check covered the newest termina. To see the version that the client uses, run `cargo tree -p mu-tui -i termina`.
- The client on Termina ran in Rex only. There, the owner checked the copy by drag, the live resize and the kitty image by hand. No other terminal ran the client.

## Source

The comparison of issue 55, merged in pull request 114. The code is `spikes/terminal-backend`, and the reports are in `spikes/terminal-backend/results`. In `spikes/`, the commands `cargo run -p terminal-backend --bin crossterm-check` and `cargo run -p terminal-backend --bin termina-check` run the checks in the current terminal. Each one asks for the keys and the paste.
