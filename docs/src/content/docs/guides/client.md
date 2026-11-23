---
title: The client file
description: Change the keys and the echo line of mu client with client.toml.
sidebar:
  order: 4
---

This guide shows how to change the keys and the echo line of `mu client`. The echo line is the row that shows each text that you send, after `> `.

## The client file

The file `client.toml` holds the configuration of the terminal client. It goes at the top level of the configuration directory, next to `global.toml`. The guide [The configuration files](/guides/configuration/) tells where that directory is. `mu client` takes the flag `--config <dir>` as `mu server` does. `client.toml` makes no world, and `mu world` ignores it.

The client reads the file once, before it opens the screen. If the file does not exist, the client uses the defaults. If the directory does not exist, the client also uses the defaults. If `--config` or `MU_CONFIG` names a directory that does not exist, the client writes a warning that names it.

If the client starts a server, it gives that server the same configuration directory.

```toml
# client.toml
echo = "show"

[keys]
"alt+n" = "focus-next"
"ctrl+n" = "none"
```

| Field | Meaning |
|---|---|
| `echo` | `show`, `dim` or `hide`. The default is `dim` |
| `[keys]` | A table from a key name to the name of an entry, or to `none` |

## The echo line

With `show`, the echo line has the normal style. With `dim`, it has the dim style. With `hide`, the client shows no echo line. The game can still show your text in its own words, such as `You say, "hello"`.

## The keys table

Each line of `[keys]` binds one key to one entry. An entry is one thing that the client can do, such as `focus-next` or `send`. The command `/help` in the client lists each entry with what it does. A binding replaces the default binding of that key. The value `none` removes the binding, so the key does nothing. Each key that the table does not name keeps its default.

An entry that needs a world, such as `focus`, cannot go on a key. Run it as a command instead, such as `/focus beta/bryn`.

### Key names

A key name is zero or more modifiers, then one key, joined with `+`. The modifiers are `ctrl`, `alt`, `shift` and `super`. Their order does not matter. Each modifier can occur once.

The key is one of these:

- One character, such as `n` or `1`. A letter is lowercase. To name a capital letter, use `shift` and the lowercase letter, such as `shift+n`.
- `plus` for the `+` key, and `space` for the space bar.
- The name of a special key. These are the special keys:

  | Group | Names |
  |---|---|
  | Edit | `enter`, `tab`, `backspace`, `esc`, `insert`, `delete` |
  | Move | `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown` |
  | Function | `f1` to `f12` |

`shift` goes only with a letter or a special key. The terminal sends Shift and another character as a different character, such as `!` for Shift+1. To bind that key, bind `!` itself.

A control character, such as a tab or a carriage return, is not a key name. Use the name of the key, such as `tab` or `enter`.

### Kept keys

The default keys leave Tab, Shift+Tab and Ctrl+Space with no binding. A later version of the client uses them for completion. You can bind them now, but a later version can change what they do by default.

### Keys that a terminal does not send

Some terminals do not send each key. In Terminal.app on macOS, the Option key types a character unless the setting "Use Option as Meta key" is on. So Alt+1 to Alt+9, Alt+Home and Alt+End do not reach the client. Terminal.app also sends Ctrl+Shift+Z as Ctrl+Z, so the key runs undo and not redo. In such a terminal, bind those entries to other keys in `[keys]`.

## Errors

The client does not open the screen for a bad file. It writes the error and exits with status 1. These are bad files:

- A file that the client cannot read. The error names the path.
- A file that is not TOML. The error names `client.toml`, the line and the column.
- An unknown field, or an `echo` value that is not `show`, `dim` or `hide`. The error names `client.toml` and the field.
- A bad binding. The error names `client.toml` and the key. A binding is bad if its key name is bad, or if its entry is unknown or needs a world. Two names of the same key are also bad, such as `ctrl+shift+z` and `shift+ctrl+z`.
