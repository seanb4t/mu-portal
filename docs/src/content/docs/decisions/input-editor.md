---
title: Input editor
description: The pose editor uses ratatui-textarea, with patches for grapheme cursor moves, one width function and the count of the wrapped rows.
sidebar:
  order: 19
---

## Decision

The pose editor and each other input of the terminal client use `ratatui-textarea`. The project does not write its own editor.

The project patches `ratatui-textarea` in three places and offers each patch upstream:

- The cursor moves by grapheme, not by code point. A grapheme is one character as the reader sees it, for example an emoji with joiners. Right, Left, Backspace and Delete each move over or delete one grapheme. Each other move, such as Up and Down, stops at a grapheme boundary.
- The cursor position, the soft wrap and the render use one width function. The client gives that function the width profile from the [character width decision](/decisions/character-width/). With the function, the render gives each cell its width as a forced width.
- The library gives the count of the wrapped rows for a width. The client sizes the editor with it.

Until upstream accepts each patch, the client builds on the fork `seanb4t/ratatui-textarea`. The branch `mu` of the fork holds the three patches. The entry `[patch.crates-io]` of the root `Cargo.toml` names one commit of that branch. A Renovate regex manager follows the branch, because the cargo manager of Renovate reads no `[patch]` table. If the patch does not apply, the test `textarea_comes_from_the_fork` fails. After a Renovate update of the commit, run `cargo update -p ratatui-textarea` on its branch.

The pull requests to upstream open after the pose editor merges.

The client does these things itself, outside the library:

- It turns on bracketed paste. A paste arrives as one event, and the client inserts it as one edit. The library ignores a paste event.
- It turns each CR and each CR LF of a paste into LF, and drops each other control character apart from the tab.
- It places the terminal cursor at the insertion point after each frame. The library only draws a styled cell as its cursor, and an input method opens its candidate box at the terminal cursor.
- If the terminal answers the query of the kitty keyboard flags, the client pushes the flag that disambiguates escape codes. It pops the flag at exit.

The keys of the editor are entries of the keymap:

| Key | What it does |
|---|---|
| Enter, Ctrl+M | Sends the text as one command. The server writes the line break of the world in place of each line break. |
| Alt+Enter, Ctrl+Enter | Sends each line as its own command, in order. Ctrl+Enter needs the kitty keyboard protocol. |
| Ctrl+J, Shift+Enter | Inserts a line break. Shift+Enter needs a terminal that reports Shift with Enter. |
| Alt+R | Replaces each line break with the line break of the world in focus. |
| Ctrl+U | Deletes to the start of the line, as readline does. |
| Ctrl+Z, Ctrl+Shift+Z | Undo, and redo. Redo needs a terminal that reports Shift. |

Ctrl+R does nothing, so a later change can give it to a search of the history. Each other editing key is a key of the library, and those keys follow readline.

Terminal.app sends Shift+Enter as Alt+Enter. So in Terminal.app, Shift+Enter sends each line, and Ctrl+J is the key of a line break.

The client restores the terminal at exit and after a panic. After a kill with SIGKILL, the terminal keeps the modes of the client. Then type `reset`.

## Reason

The spike measured the library with a pose of 4096 bytes and 51 lines. The text had accents, Japanese, emoji with joiners and selectors, a flag and a long URL.

| Measure | Result |
|---|---|
| Insert of the paste | 18.7 µs, one undo step |
| Full frame with the paste, width 40 to 120 | 10.2 µs to 26.5 µs |
| The same paste as one key event for each character, with a frame after each | 217.5 ms, and undo cannot go back to empty |
| Cursor positions where the terminal cursor and the drawn cursor disagree, width 20 to 80 | 73 to 262 of 3928 |
| Cursor positions inside a grapheme | 24 of 3928 |

The library adds the width of each code point to place the cursor, but it draws each grapheme with the width of the whole grapheme. After `🕯️` the library reports column 1, and it draws the cursor in column 2. After the family emoji `👨‍👩‍👧` it reports column 6, and it draws column 2. A family emoji takes 5 presses of Right and 5 of Backspace. A flag and a letter with a combining accent take 2 each. On the rows after such a grapheme, an input method opens its box in the wrong cell.

Without bracketed paste, each line break of a paste is an Enter. The client then sends each line as its own pose.

The library is small, under 6000 lines, and it is MIT licensed. Its latest release fixed a width problem of the same kind, so upstream accepts this kind of fix. A fork with two patches costs less than a new editor with soft wrap, undo, selection and search.

The spike ran a guided test in three terminals. Each step had the result below.

| Step | Rex | Ghostty through herdr | Terminal.app |
|---|---|---|---|
| Kitty keyboard protocol | yes | yes | no |
| Shift+Enter | line break | line break | line break, sent as Alt+Enter |
| Option+Enter | line break | line break | sends the pose |
| Ctrl+J | line break | line break | line break |
| Paste of 4096 bytes | one event, LF, text kept | one event, LF, text kept | one event, CR, line breaks kept |
| Second input with F2 | works | works | works |
| Input method box | at the cursor | at the cursor | at the cursor |
| Japanese text from the input method | kept | kept | kept |

In the Terminal.app run, `pbcopy` ran with no UTF-8 locale and changed the text before the paste. So that run shows the line breaks, but it does not show whether Terminal.app keeps other characters.

## Not tested

- Ghostty without herdr. herdr ran between Ghostty and the editor, and it can change the keys and the paste.
- A paste of non-ASCII text in Terminal.app with a UTF-8 clipboard.
- tmux and Windows.
- A terminal that cuts a long paste.
- The types `pennmush` and `rhostmush` on a live game.

The pose editor change tested these items, which the spike did not:

- The three patches, with their tests in the fork and the tests of the editor in the client.
- The terminal cursor on a row after a wide grapheme, in a pseudo-terminal. A test checks its cell.
- Linux. The pseudo-terminal tests run there in CI.
- A paste of 64 KiB, as one edit with one undo.
- Right-to-left text.

## Source

The spike of issue 4, merged in pull request 50. The code is `spikes/input-editor`. The command `cargo run --release -p input-editor -- bench` in `spikes/` gives the measurements, and `spikes/input-editor/results/bench.tsv` holds the last run.

The logs of each terminal are in `spikes/input-editor/results`. To repeat a run, in `spikes/`:

```bash
cargo build --release -p input-editor
./target/release/input-editor sample 4096 | LANG=en_US.UTF-8 pbcopy
./target/release/input-editor run --log input-editor/results/<terminal>.tsv
```

The editor shows each step on the screen. The Rex run has two parts: `rex-1.tsv` holds steps 1 to 4, and `rex.tsv` holds steps 5 to 7. `rex-send-key.tsv` is a run in Rex that `rex send-key` drove, with no person at the keys.
