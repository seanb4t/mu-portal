# Proposal

## Why

The terminal client has one input line of one row. A pose on a MUSH is often several paragraphs. Today the player cannot see the whole pose, cannot put a line break in it, and cannot paste it. A paste arrives as one key for each character. So each line break of the paste is an Enter, and it sends a part of the pose. The cursor of the input line counts code points. So it lands inside an emoji and out of line with the text. An input method then opens its box in the wrong cell. Issue #56 tracks this change. It is step 7 of the Phase 1 plan in #61, and the first evening of play waits for it, #101 and #102.

The game gets no line break today either. The API call `Send` refuses a text with LF, and no world says how its game writes a line break inside one command. TinyMUX and AresMUSH read `%r`, and Evennia reads `|/`. The owner decided on 2026-11-20 that the server joins the lines of a text with the line break of the world. So a pose of several paragraphs reaches the game as one command.

## What Changes

- The pose editor: the input line becomes an editor of several lines on `ratatui-textarea`, as the input-editor decision says. It wraps a long line at a word. It grows with its text, from 1 row up to a limit, and then scrolls.
- Enter sends the whole text of the editor. Alt+Enter and Ctrl+J insert a line break on every terminal. Shift+Enter inserts one where the terminal reports Shift with Enter, as with the kitty keyboard protocol. Ctrl+Z undoes the last edit.
- A fork of `ratatui-textarea` with the two patches of the decision and one more, through a `[patch.crates-io]` entry that Renovate can see:
  - The cursor moves by grapheme. Left, Right, Backspace and Delete each move over or delete one whole grapheme. Each other move, such as Up and Down, stops at a grapheme boundary.
  - The cursor position, the soft wrap and the render use one width function. The client gives it the width profile that the startup probe learned, so the editor and the output pane agree on each width.
  - A third, small patch gives the count of the wrapped rows, so the client can size the editor. Issue 38 upstream asks for it.
  The change offers each patch upstream as a pull request.
- Bracketed paste. The client turns on mode 2004. A paste arrives as one event, and the editor inserts it as one edit, which one undo removes. A paste replaces a selection, and a second undo restores the text of the selection. Each CR LF and each lone CR of the paste becomes LF. The editor drops each other control character of a paste, apart from the tab.
- The terminal cursor. After each frame, the client puts the terminal cursor at the insertion point of the editor, so an input method opens its box there.
- The kitty keyboard protocol. The startup probe asks whether the terminal knows the protocol. If it does, the client pushes the flag that disambiguates escape codes, and pops it at exit. Shift+Enter then arrives with its modifier.
- The world field `line_break`: the text that the server writes to the game in place of each LF of a sent text. The built-in types set it: `%r` for `tinymux` and `aresmush`, and `|/` for `evennia`. A world with no `line_break` still refuses a text with LF. The fake world of `mu server --fake-world` uses `%r`.
- The input-editor decision page gets the results of this change: the patches, their upstream pull requests, and the items under "Not tested" that this change tested.

## Not in this change

- A second editor, an editor in a pane that the player arranges, input history, a configurable keymap and client commands. #101 builds them. This change keys the editor state by an id, as the architecture decision keys each pane. So #101 can add an editor with no change to the widget.
- Spawns. #102 builds them.
- Search, selection with the mouse, and a copy from the editor. The library has its own search and selection, but no key of this change binds them.
- A line break from a Lua handler. A `Send` command of a handler gets the same join as the API call, because both reach the game through one path. No handler can use it before #60 adds Lua.
- A line break for a game that has no single-text form, such as a game that needs one command for each line. Such a world leaves `line_break` unset and refuses the text.
- Right-to-left text, tmux and Windows. The decision lists them under "Not tested", and this change keeps them so.
- A wait for upstream to accept a patch. The fork stays until each patch is in a release.

## Capabilities

### New Capabilities

- `input-editor`: the editor widget of the terminal client. It covers the text of several lines, the soft wrap and the growth up to a limit. It also covers the grapheme cursor, the width of each grapheme, line breaks, paste, undo and the terminal cursor. The pose editor is one instance of it, and each later input is another.

### Modified Capabilities

- `terminal-client`: the bottom row becomes the rows of the editor. Enter sends the whole text of the editor, line breaks included. The client turns on bracketed paste, asks for the kitty keyboard protocol and turns it on, and restores both at exit.
- `world-config`: the new field `line_break`, its range, and its value in each built-in type.
- `world-sessions`: `Send` joins the lines of a text with the line break of the world, and the fake world uses `%r`.

## Impact

- `crates/mu-tui`: the editor widget, the model, the keymap, `update`, `view`, the startup probe and the restore of the terminal.
- `crates/mu-core`: the field `line_break` in the layers, the resolver and `ResolvedWorld`, the three built-in types, and the join in the session before the line-break check. The type `LineBreak` holds the checked value.
- `crates/mu-cli`: the fake world gets `line_break = "%r"`, and `mu world show` lists the field.
- `Cargo.toml`: a `[patch.crates-io]` entry for `ratatui-textarea` that names the fork by a git commit. `deny.toml` allows that git source.
- A fork of `ratatui/ratatui-textarea` on GitHub, and three pull requests to upstream.
- Docs: the input-editor decision page and the configuration guide.
- The API: no change to the schema. `Send` takes a text with LF for a world with `line_break`, where it failed before.
