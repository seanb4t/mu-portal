# Design

## Context

See proposal.md for the motivation. The facts below shape the approach.

- `mu-tui` already depends on `ratatui-textarea` 0.9.4, with no default features. The model holds one `TextArea` of one line. `update` gives each key that the keymap does not map to `TextArea::input`, through `input_key`. `view` renders it in `Leaf::Input`, a row of `Constraint::Length(1)`.
- A key with modifiers can have no entry of its own. Then the keymap takes the entry of the plain key. So Shift+Enter today is Enter, and it sends the line.
- `TextArea::input` maps Ctrl+J to `delete_line_by_head` and Ctrl+M to a new line. Ctrl+U is its undo, and Ctrl+R its redo.
- The startup probe writes every query in one write and ends with DA1. `run.rs` keeps an `AtomicBool` for mode 2027, so the restore at exit and in the panic hook know whether to turn the mode off.
- The API call `Send` reaches `Session` in `mu-core` as `Command::Send`. The session refuses a text with CR or LF before the hooks run, with `PortalError::LineBreak`. A `Send` command of a handler takes the same path.
- `ratatui-textarea` upstream has no commit after 0.9.4, released on 2026-10-31. No issue or pull request there covers a cursor that moves by grapheme or a width function. Issue 38 upstream asks for the count of wrapped rows. Pull request 37, in 0.9.4, aligned the layout with the widths that ratatui draws. The spike measured 0.9.4, so its defects still hold.

## Goals / Non-Goals

**Goals:**

- One editor widget, keyed by an id, that each later input reuses.
- The editor and the output pane use one width for each grapheme.
- The fork stays small enough to offer upstream.
- If the patch stops applying, a test fails.

**Non-Goals:**

- A drawn cursor. The terminal cursor is the only cursor of the editor.
- A display of the line break of a world in the output pane. A player line of a joined text shows as `> one%rtwo`, as the game got it.
- A check of each terminal by hand beyond Rex. See "Using the spike".
- A line break from Shift+Enter on a terminal that sends it as CR or as ESC CR. Ctrl+J inserts a line break on every terminal.
- U+FFFD for a grapheme wider than the editor. D1 states what the editor does there.

## Decisions

### D1. A fork of ratatui-textarea with three patches

The fork is `seanb4t/ratatui-textarea` on GitHub. The branch `mu` starts at the tag of 0.9.4. The root `Cargo.toml` names it in `[patch.crates-io]` with a `git` URL and a full commit id in `rev`. `deny.toml` allows that one git URL in `[sources]`.

Each patch is one commit on its own branch, so each one can go upstream as its own pull request. The branch `grapheme-width` of patch 2 starts at the branch `grapheme-cursor` of patch 1. Both patches change the same mapping from a column to the data. Its pull request names the pull request of patch 1 as its base. The branch `wrapped-rows` of patch 3 starts at the tag. The branch `mu` merges them, with one more commit that gives the width function to the count of patch 3.

1. The cursor stops only at a grapheme boundary. `CursorMove::Forward` and `Back`, `delete_char` and `delete_next_char` step over the grapheme boundaries of `unicode-segmentation`. Each move that maps a column to the data stops at a boundary too. Those moves are Up, Down, Top, Bottom, the paragraph moves, `Jump`, and the mapping from a screen column in `char_offset_for_screen_col`. The patch does it in one place: after each move, `move_cursor_with_shift` moves the cursor to a grapheme boundary. The data cursor stays a char index, so the rest of the API does not change.
2. One width function. `TextArea::set_grapheme_width` takes a function from a grapheme to its width in cells. The default is the width that ratatui draws, so the behavior of upstream does not change. The cursor position, the wrap of the library and the render all call it. The wrap of the library is its own `src/wrap.rs`. It includes `split_range_by_grapheme_width`, the split inside a long word of `WrapMode::WordOrGlyph`. With a function set, the render gives each grapheme the width of the function as a forced width, with `CellDiffOption::ForcedWidth` of ratatui. So a grapheme takes exactly the cells that the function gives. The output pane forces a width with the same option of ratatui, so the fork copies no rule of the pane. With a function set, the render also draws a grapheme whole where the library puts its first char in the span of the cursor. The expansion of a tab into spaces still counts the widths of ratatui.
3. The count of the wrapped rows for a width, as issue 38 upstream asks. The client needs it to size the editor. The count uses the width function.

The client has its own wrap for the output pane, in `crates/mu-tui/src/wrap.rs`. The editor keeps the wrap of the library. The cursor moves, the scroll and the row count of the library all depend on that wrap. A patch that takes a wrap function from the client is larger, and upstream has no use for it. Both wraps use the same widths, break at a space, and break inside a word only where the word is wider than the row. They differ in two cases:

- In the pane, a space that does not fit hangs at the end of its row and takes no cell. The editor makes no such promise.
- In the pane, a grapheme wider than the row shows as U+FFFD. The editor makes no such promise. Task 3.2 makes sure that the editor draws no cell outside its area for such a grapheme.

The test `editor_and_pane_wrap_alike` gives one text to both wraps and compares the rows.

Alternatives: write a new editor, or use another crate such as `rat-text`. The input-editor decision rejected a new editor. Another crate reopens a decision that a spike settled, for the same two defects. The rule to reuse open source first holds with the fork.

Enforced by: the unit tests of each patch in the fork, which upstream runs too. The test `textarea_comes_from_the_fork` in `crates/mu-cli/tests/repo_config.rs` reads `Cargo.lock` and checks that `ratatui-textarea` has the git source of the fork. A patch can stop matching the version of the requirement, for example after a Renovate bump of the crate. Cargo then only warns, and the build uses the crate from crates.io. The test catches that.

### D2. Renovate sees the fork

Renovate's cargo manager reads a git dependency through the `git-refs` datasource, but it does not read `[patch.crates-io]`. Task 1.2 found that in its source. So the change uses a custom regex manager in `renovate.json`. It reads the `rev` of the patch entry as the digest of the branch `mu` of the fork. The regex manager edits only `Cargo.toml`. On its pull request, CI fails with `--locked` until the owner runs `cargo update -p ratatui-textarea` on the branch. The RhostMUSH commit of the telnet spike uses the same kind of manager in `renovate.json`.

Enforced by: the Renovate dependency dashboard, issue #26, which lists the fork after the change merges. Review checks the dashboard once.

### D3. The editor widget and its state

`EditorState` holds the `TextArea`, the width profile as `Arc<Caps>`, and the last area that it drew. `Editor` is a ratatui `StatefulWidget`. The model holds `editors: HashMap<EditorId, EditorState>`, and `EditorId::POSE` is the one editor of this change. `Leaf::Input` becomes `Leaf::Editor(EditorId)`. `Model::set_caps` gives the new caps to each editor as it does to each pane.

The pose editor has the focus for keys. A focus between editors comes with the second editor of #101.

The `TextArea` uses `WrapMode::WordOrGlyph`. `WrapMode::Word` never splits a word, so a word wider than the area runs off the editor.

`Model::input` returns the lines of the editor joined with LF. `submit` sends that text. `Model::set_input` splits its text at each LF into the lines of the `TextArea`. `submit` and `on_reply` use it. So a text that the server refuses goes back as the lines that the player wrote.

Enforced by: unit tests of `update` and `TestBackend` tests of the widget, and review of the state type. The state knows nothing of the app, as the architecture decision asks.

### D4. The editor grows through a sizing of the layout tree

A child of `Node::Split` has a `Constraint` today. It gets a `Size` instead: `Size::Fixed(Constraint)` or `Size::Editor { id, max_rows }`. Before it splits an area, `view` turns `Size::Editor` into `Constraint::Length(rows)`. `rows` is the count of wrapped rows of the text at the width of the area, from patch 3. It is at most `max_rows`, and at most half the rows of the area that the split divides, rounded down. In each case it is at least 1. The default layout uses `Size::Editor { id: EditorId::POSE, max_rows: 8 }` for the bottom child. Its split divides the whole terminal.

The limit of 8 rows keeps most of a terminal of 24 rows for the output. The limit is a field of the node, and the half comes from the area of the split. So #101 can put an editor in a pane with another limit, with no new shape of `Size`.

Alternative: the editor sets a height in the model during `update`. Rejected, because `update` does not know the width of the area, and the height depends on the wrap.

Enforced by: the scenarios of "The editor grows with its text" on `TestBackend`. They include a terminal of 17 rows, where both limits are 8, and terminals of 3 rows and of 1 row.

### D5. Keys of the editor are entries of the keymap

The default keymap gets these entries:

| Key | Action |
|---|---|
| Shift+Enter | `Newline` |
| Alt+Enter | `SubmitLines` |
| Ctrl+Enter | `SubmitLines` |
| Alt+R | `JoinLines` |
| Ctrl+J | `Newline` |
| Ctrl+U | `DeleteToLineStart` |
| Ctrl+Z | `Undo` |
| Ctrl+Shift+Z | `Redo` |
| Ctrl+R | `Ignore` |
| Ctrl+M | `Submit` |

Each one needs its own entry. Without one, Shift+Enter, Alt+Enter and Ctrl+Enter fall back to the entry of Enter and send the text as one command. Without one, Ctrl+J reaches `TextArea::input` and deletes to the start of the line.

The editing keys follow readline, the convention of shells and of TinyFugue. The library differs from readline in four keys, and the entries above change them:

- Ctrl+U is the undo of the library. In readline it deletes to the start of the line. `DeleteToLineStart` calls `delete_line_by_head`. At column 0 it calls nothing, because `delete_line_by_head` there deletes the line break before the cursor.
- Ctrl+R is the redo of the library. In readline it searches the history. `Ignore` does nothing, so #101 can give the key to that search.
- Ctrl+M is a line break in the library. In readline it is Enter. A terminal with the kitty keyboard flag reports Ctrl+M as a key of its own.
- The library has no Ctrl+Z. It is the undo of most editors. Ctrl+Shift+Z is their redo. Its entry is the chord of the lowercase `z` with Ctrl and Shift, as Termina reports the key with the kitty keyboard flag. A terminal that does not report Shift sends Ctrl+Shift+Z as Ctrl+Z, so it has no key for redo.

These keys of the library already match readline and stay: Ctrl+A, Ctrl+E, Ctrl+K, Ctrl+W, Ctrl+D, Ctrl+H, Ctrl+B, Ctrl+F, Alt+B, Alt+F, Alt+D and Ctrl+Y. Ctrl+N and Ctrl+P stay the next and the previous world. In readline they move in the history, and #101 decides them with the input history.

Termina reads the byte LF as Ctrl+J and the byte CR as Enter. ESC CR is Enter with Alt. With the kitty keyboard flag, Shift+Enter is `CSI 13;2u`. Without the flag, most terminals send Shift+Enter as CR, so it sends the text. The spike found that Terminal.app sends it as ESC CR, which is Alt+Enter. So Shift+Enter sends each line there, and Ctrl+J is the line break. Without the flag, a terminal sends Ctrl+Enter as CR or as LF, so it sends the text as one command or inserts a line break.

Every other key goes to `TextArea::input` as today, with the editing keys of the library.

Enforced by: unit tests of `update` for each entry, the scenarios of "The editing keys follow readline", and the pseudo-terminal scenario of Shift+Enter with the kitty flag.

### D6. Bracketed paste

`screen::enter` writes `CSI ?2004h`. `screen::leave` writes `CSI ?2004l`, and the panic hook runs the same restore. Termina gives a bracketed paste as `Event::Paste(String)`. `update` cleans the text and calls `insert_str` once, so the paste is one edit and one undo step. The clean turns CR LF and each lone CR into LF. It drops each other C0 control character except tab, and DEL and each C1 control character. The editor text then never holds ESC, so the render cannot write a sequence to the terminal.

The clean is `clean_paste` in `crates/mu-tui/src/text.rs`, beside `clean`. Both use `char::is_control` as the one test of a control character. That test covers C0, DEL and C1.

Shift with an arrow key selects in the library. A paste replaces a selection, as an editor does. `insert_str` deletes the selection as an edit of its own and then inserts the text. So the first undo removes the text of the paste, and a second undo restores the text of the selection. Typing over a selection undoes in the same two steps. One edit for both needs a new edit kind in the history of the library, and this change does not add it.

A paste with no editor in focus goes nowhere. In this change the pose editor always has the focus.

Enforced by: unit tests and a property test of the clean, unit tests of `update` with `Event::Paste`, and the pseudo-terminal scenarios.

### D7. The terminal cursor is the only cursor

`view` renders the editor and then calls `Frame::set_cursor_position` with the cell of the insertion point. The cell comes from `screen_cursor` and `scroll_offset` of the patched `TextArea`, inside the area of the editor. After a row that the text fills exactly, `screen_cursor` of 0.9.4 gives the column one past the area. `view` clamps the column to the last column of the editor. A terminal holds its own cursor at the last column in the same case. The cursor style of the `TextArea` is the default style, so the library draws no second cursor. The spike found that the drawn cursor and the terminal cursor disagree after a wide grapheme. With one cursor, they cannot disagree.

Enforced by: the pseudo-terminal scenarios, which read the cursor of `vt100`.

### D8. The kitty keyboard protocol

The probe writes `CSI ?u` before the DA1 query. Termina parses the answer `CSI ?<flags>u` as `Csi::Keyboard(Keyboard::ReportFlags)`. The probe gets the variant `Answer::Keyboard` for it. `Outcome` gets the flag `kitty_keyboard`. An answer before DA1 sets it. The client uses only the presence of the answer, not its flags. Termina 0.3.3 reads only the first byte after `?` as the flags, so a value of 10 or more reads wrong.

If `kitty_keyboard` is true, `run` writes `CSI >1u` after the probe. That is the flag that disambiguates escape codes. The restore at exit pops it with `Keyboard::PopFlags(1)`, which Termina writes as `CSI <1u`.

`run.rs` holds one `AtomicBool` for mode 2027 today. It becomes one shared `Restore` value, with a flag for mode 2027 and a flag for the kitty keyboard flag. `screen::open`, `screen::leave` and the panic hook take that one value. A later mode adds a field, and no restore path gets a new argument.

A late answer arrives as a `Csi` event, and `update` drops it, as it drops each terminal event that it does not handle. So the client does not push the flag after a late answer.

Only flag 1 is pushed, as in the spike. The flag for event types adds key release events. The flag for all keys as escape codes makes each text key an escape code.

Enforced by: unit tests of the probe with a scripted answer, and the pseudo-terminal scenarios of "The client turns on the kitty keyboard protocol".

### D9. The server joins the lines of a sent text

The layers get `line_break: Option<String>`. It is a quirk, so `Quirks` reads it and a built-in type can set it. The function `check_line_break` refuses an empty value, a value of more than 16 characters, and a value with a control character. `check_layer` calls it.

`ResolvedWorld` holds `line_break: Option<LineBreak>`. `LineBreak::new` runs `check_line_break`, as `Login::new` and `Keepalive::new` run the check of their fields. The fake world and the tests build a `ResolvedWorld` as a struct literal. With the type, no value outside the range reaches the session on that path.

In `Session`, the handling of `Command::Send` changes in this order:

1. A text with CR fails with `PortalError::LineBreak`.
2. A text with LF and a world with `line_break` gets each LF replaced.
3. A text with LF and a world with no `line_break` fails with `PortalError::LineBreak`.
4. The hooks, the journal and the write to the game get the joined text, as they get each text today.

The message of `PortalError::LineBreak` names the field: `the text holds a line break, and the world sets no line_break`. For a CR, it stays `the text holds a line break`. The error gets a field that tells the two cases apart.

The fake world of `mu server --fake-world` sets the line break `%r` in its `ResolvedWorld`, through `LineBreak::new`. Most tests of the client run on fake worlds, so a pose of two lines can reach a game in a test.

The join is in the session and not in the client, so each client and each handler gets the same rule. The player event holds the joined text, because that is what the game got.

Enforced by: the scenarios of `world-config` and `world-sessions`, the constructor of `LineBreak`, and a property test of the join. `crates/mu-core/tests/portal.rs` already holds `send_with_lf_is_refused`, on a world with no `line_break`. It covers "A line break in the text" and stays as it is. `crates/mu-cli/tests/worlds.rs` holds a test of the same name on a fake world, which now joins. Task 2.4 removes that copy, and `fake_world_gets_a_joined_line` takes its place.

### D12. The client knows the line break of each world

The message `World` of `crates/mu-proto` gets `optional string line_break = 4`. `ListWorlds` fills it from the `ResolvedWorld`. The client keeps it in the `World` of its model. The value changes only with a restart of the server, as each other field of a configured world does.

The join of D9 stays on the server, as the one rule for each sender. The client uses the value only to edit: `JoinLines` replaces each LF of the editor with it, through the edit calls of the library, so undo steps restore the lines. A joined text holds no LF, so the join of the server then changes nothing. Both sides use the one configured value, so they cannot disagree.

The project has no release yet. So this change, and each later one before the first release, keeps no compatibility with an older client or server.

Enforced by: the scenario "The line break of a world", and the scenarios of "Alt+R joins the lines in place".

### D13. A send of each line

`SubmitLines` splits the text of the editor at each LF and clears the editor. The model keeps the lines in a queue, with the id of the world that had the focus at the key. `update` returns `Effect::Send` for the first line, to that world. On the reply of a line, `update` returns the effect for the next one. That line goes to the world of the queue, and not to the world in focus. A plain send takes the world from the focus, and its reply names no world. So without the id, a Ctrl+N between two lines sends the rest of a script to another character. So the lines reach the game in order, and `update` still does no I/O.

On a refusal, `update` empties the queue. It puts the refused line and the later lines back with `set_input`, before any text that the player typed since. It shows the reason as each refused send does today.

A plain Enter while the queue holds lines sends at once, and its place among the lines of the queue is not fixed. That case is rare, and an order rule for it can come with the client commands of #101.

Alternative: one API call with a list of lines. Rejected, because each line is an ordinary `Send`, and a hook and the journal see each one as a line of the player.

Enforced by: the scenarios of "Alt+Enter sends each line as a command" and of "A refused line stops a send of each line".

### D11. Built-in types extend built-in types

`Quirks` reads `extends` in a built-in type, and `parse_quirks` puts it in the `extends` of the layer. The resolver already follows the chain of a user type, so it follows this chain with no change. The file `types/pennmush.toml` holds only `extends = "tinymux"`. So the type gets the character set, the login template, the keepalive and the line break of `tinymux`. The file `types/rhostmush.toml` also sets `keepalive.command = "idle"`, the form that the help of RhostMUSH gives.

PennMUSH and RhostMUSH come from the same family of servers as TinyMUX. Task 2.1 read the help files and the source of each one for the four quirks. PennMUSH matches TinyMUX in all four. RhostMUSH differs in two. Its keepalive is the command `idle`. Without a toggle of the player, it removes each character outside ASCII. `iso-8859-1` holds ASCII and fits the toggle `ACCENTS`, so the type keeps it. A player with the toggle `UTF8` sets `charset = "utf-8"` in the game file. No task connects to a live game of either kind.

Alternative: copy the fields of `tinymux` into two files. Rejected, because a later change of `tinymux` then misses the copies.

Enforced by: the scenarios "A built-in type that extends another" and "A built-in type extends only a built-in type".

### D10. Tests

| Scenarios | Where |
|---|---|
| `input-editor`, apart from the terminal cursor | `crates/mu-tui/tests/editor.rs`, on `TestBackend` |
| The terminal cursor, paste and kitty keys through a terminal | `crates/mu-cli/tests/client.rs`, on a pseudo-terminal with `vt100` |
| `terminal-client` | `crates/mu-tui/tests/app.rs`, or `crates/mu-cli/tests/client.rs` where a scenario needs a terminal |
| `world-config` | `crates/mu-cli/tests/world_config.rs`, and `crates/mu-core/tests/world_config.rs` for "Each built-in type reads as quirks" and the type `LineBreak` |
| `terminal-client`, a send of each line and a join in place | `crates/mu-tui/tests/app.rs`, with the replies of a test server, and `crates/mu-cli/tests/client.rs` for "Each line is a command" |
| `world-sessions` | `crates/mu-cli/tests/worlds.rs` for the fake world, and `crates/mu-core/tests/portal.rs`, `connection.rs` and `scripts.rs` for a world with a chosen `line_break`. "The fake world answers a mark in order" has its test in `crates/mu-fake-server/src/memory.rs`. |

The pseudo-terminal tests run on Linux in CI and on macOS on the owner's machine.

### Using the spike

The change builds on the input-editor spike: the decision page `docs/src/content/docs/decisions/input-editor.md`, and pull request #50.

- Task 1.1 read each review thread of pull request #50. It has no thread and one bot summary. The summary found one thing: the timings in the table of the decision page differ a little from `spikes/input-editor/results/bench.tsv`. No decision of this change depends on a timing. Task 7.1 aligns the table.
- The new code is written under the tests of this change. The spike code in `spikes/input-editor/src/editor.rs` is an example only.
- The sample pose of 4096 bytes in 51 lines is evidence of the spike. Task 5.3 writes it once with `spikes/input-editor`, and commits it as `crates/mu-tui/tests/data/pose-4096.txt`.

Each item under "Not tested" of the decision:

| Item | In this change |
|---|---|
| Ghostty without herdr | Non-goal. Only Rex is driven live. Task 7.3 opens a Parked issue. |
| A paste of non-ASCII text in Terminal.app with a UTF-8 clipboard | Non-goal. Task 7.3 opens a Parked issue. |
| The two patches | Tasks 3.1 to 3.3 write them and a third, and task 3.5 offers them upstream. |
| The input method box on a row after a wide emoji | The pseudo-terminal scenario "The cursor on a wrapped row after a wide grapheme" checks the cell. Task 7.2 checks the box live in Rex. |
| tmux, Linux and Windows | tmux and Windows are non-goals. The pseudo-terminal tests run on Linux in CI. |
| A paste larger than 4096 bytes, and a terminal that cuts a long paste | Task 5.3 tests a paste of 64 KiB in `update`. A cut paste is a non-goal, because Termina holds an unfinished paste until its end marker. |
| Right-to-left text | Non-goal. |

### What enforces each rule

| Rule | Enforced by |
|---|---|
| The patch of `ratatui-textarea` applies | The test `textarea_comes_from_the_fork` |
| The editor and the pane use one width | The scenarios of "The editor uses the widths of the terminal" |
| A paste is one edit and holds no ESC | Unit tests of the clean and the pseudo-terminal scenario |
| The client restores each mode it sets | The pseudo-terminal scenarios at exit, and review of the panic path, which no test can reach |
| `update` does no I/O | Review, as the architecture decision says |
| The lines of a send of each line reach the game in order | The scenario "Each line is a command" and the unit tests of the queue |
| An `extends` of a built-in type names a built-in type, with no loop | The test `built_in_type_extends_only_a_built_in_type` |
| A world joins only with its own `line_break` | The scenarios of `world-sessions`, and the property test `joined_text_is_one_line` |
| A `line_break` of a `ResolvedWorld` is in its range | The constructor of `LineBreak` |
| The cursor of the editor is at a grapheme boundary | The tests of patch 1 in the fork, and the scenarios of "The cursor moves by grapheme" |
| The editor and the pane wrap one text alike | The test `editor_and_pane_wrap_alike` |
| Docs prose is plain English | The simple-english hook and review |

### Facts about third-party tools to check

| Fact | Task |
|---|---|
| A terminal with the kitty keyboard flag sends Alt+Enter as `CSI 13;3u` and Ctrl+Enter as `CSI 13;5u` | 7.2, in Rex |
| `screen_cursor` gives the column one past the area after a row that the text fills exactly | 5.5 |

### Checked facts

| Fact | Source |
|---|---|
| Termina gives a bracketed paste as `Event::Paste(String)` | `termina` 0.3.3, `src/event.rs` and `parse_csi_bracketed_paste` in `src/parse.rs` |
| Termina parses `CSI ?<flags>u` as `Csi::Keyboard(Keyboard::ReportFlags)`, and reads only the first byte as the flags | `termina` 0.3.3, `src/parse.rs` |
| Termina has `Keyboard::QueryFlags`, `PushFlags` and `PopFlags` | `termina` 0.3.3, `src/escape/csi.rs` |
| Termina reads the byte LF as Ctrl+J and CR as Enter | `termina` 0.3.3, `parse_event` in `src/parse.rs` |
| `TextArea::input` maps Ctrl+U to `undo`, Ctrl+R to `redo` and Ctrl+M to a line break, and has no entry for Ctrl+Z | `ratatui-textarea` 0.9.4, `src/textarea.rs` |
| `TextArea::input` maps Ctrl+J to `delete_line_by_head` | `ratatui-textarea` 0.9.4, `src/textarea.rs` |
| `TextArea` has `screen_cursor`, `scroll_offset`, `insert_str` and `delete_selection`, which `insert_str` calls first | `ratatui-textarea` 0.9.4, `src/textarea.rs` |
| Termina writes `Keyboard::PopFlags(n)` as `CSI <nu` | `termina` 0.3.3, `src/escape/csi.rs` |
| The output pane forces the width of a cell with `CellDiffOption::ForcedWidth` | `crates/mu-tui/src/pane.rs`, `draw_text` |
| With a width function, the fork draws no cell outside its area for a grapheme wider than the area. It leaves the row of that grapheme empty | The test `grapheme_wider_than_the_area_draws_no_cell_outside_it` of the fork |
| `char::is_control` is the test of a control character in `clean` | `crates/mu-tui/src/text.rs` |
| The widths of the library come from `char_width` in `util.rs`, `display_width_to` in `wrap.rs` and `drawn_graphemes` in `widget.rs` | `ratatui-textarea` 0.9.4 |
| The latest release is 0.9.4, and upstream has no commit after it | crates.io and the commit list of `ratatui/ratatui-textarea`, read on 2026-11-20 |
| The cargo manager of Renovate supports the datasources `crate`, `git-refs`, `git-tags`, `github-tags` and `gitlab-tags` | docs.renovatebot.com, modules/manager/cargo |
| TinyMUX, PennMUSH and RhostMUSH turn `%r` into a line break | The topic SUBSTITUTIONS of `help.txt` of TinyMUX and of RhostMUSH and of `penntop.hlp` of PennMUSH, and the evaluator source of each, read on 2026-11-21 |
| AresMUSH turns `%r` into a line break in a pose | `plugins/utils/help/en/formatting.md` of aresmush/aresmush |
| Evennia turns `\|/` into a line break in text from a player | `docs/source/Concepts/Colors.md` and `evennia/utils/ansi.py` of evennia/evennia |
| The login line is `connect <name> <password>` on each of the five servers | The connect help or the connect screen of each repository |
| PennMUSH has the command `IDLE`, in capitals, and its default character set is Latin-1 | `penncmd.hlp`, `penntop.hlp` and `src/bsd.c` of pennmush/pennmush |
| RhostMUSH has the command `idle`, and removes each character outside ASCII for a player with no toggle | The topics IDLE and UNICODE SUPPORT of `help.txt` of RhostMUSH/trunk |
| The cargo manager of Renovate reads no `[patch]` table | `lib/modules/manager/cargo/schema.ts` of renovatebot/renovate, read on 2026-11-21 |
| A regex manager with the datasource `git-refs` takes the branch as `currentValue` and the commit as `currentDigest` | docs.renovatebot.com, modules/datasource/git-refs and modules/manager/regex |
| cargo-deny allows one git source with `allow-git` in `[sources]`. The URL must match with no regard to a `.git` at its end | the cargo-deny book, checks/sources/cfg |
| `Frame::set_cursor_position` shows the cursor at the position after the frame, and a frame that does not call it hides the cursor | docs.rs/ratatui, `Frame` |
| `StatefulWidget` has `type State` and `render(self, area, buf, state)`, and `Frame::render_stateful_widget` calls it | docs.rs/ratatui, `StatefulWidget` |
| The kitty keyboard query is `CSI ?u`, the push is `CSI >1u`, and `CSI <u` pops one entry, as `CSI <1u` does | sw.kovidgoyal.net/kitty/keyboard-protocol |
| With flag 1, a terminal sends Ctrl or Alt with a letter as `CSI <code>;<modifiers>u`: Ctrl+J, Ctrl+M, Ctrl+U, Ctrl+Z, Ctrl+Shift+Z and Alt+R | sw.kovidgoyal.net/kitty/keyboard-protocol |
| The page states the bytes of plain Enter with flag 1, and not those of Enter with a modifier. With the flag, the spike got Shift+Enter as a key of its own in Rex and in Ghostty | sw.kovidgoyal.net/kitty/keyboard-protocol, and the input-editor decision page |
| A terminal keeps one stack of keyboard flags for the main screen and one for the alternate screen | sw.kovidgoyal.net/kitty/keyboard-protocol |

## Risks / Trade-offs

- [Upstream rejects a patch, or takes it with another API] → The fork stays until a release holds each patch. A change of API is one edit in `EditorState`.
- [A Renovate bump of `ratatui-textarea` drops the patch with only a warning] → The test `textarea_comes_from_the_fork` fails. The owner then rebases the fork on the new release.
- [The fork falls behind upstream fixes] → The same test fails on each bump, so each upstream release gets a rebase.
- [A crash with SIGKILL leaves the kitty flag and bracketed paste on] → The panic hook covers a panic. After a SIGKILL, the user runs `reset`. The decision page says so.
- [A player line shows `%r` in the output pane] → It is what the game got. A display of the line break can come with #101.
- [In Terminal.app, Shift+Enter sends each line of an unfinished pose] → The decision page and the configuration guide name Ctrl+J as the line break there. #101 makes the keymap configurable.
- [The documentation of PennMUSH or RhostMUSH misses a quirk that differs from TinyMUX] → The owner overrides the field in the game file. The first evening on such a game finds it.
- [A game reads `%r` in some commands only] → Task 2.1 checks the docs of each game. A game that extends no built-in type has no `line_break`, so its worlds refuse a text of several lines.

## Migration Plan

No data moves. `line_break` is optional, so each configuration directory stays valid. A world that extends a built-in type gets the line break of its type at the next start of the server. To roll back, first remove `line_break` from each game, character and user type file of the configuration directory. Then revert the change. Each layer refuses an unknown field. So with a file that still sets `line_break`, the old server does not start, and its error names the file and `line_break`. The fork can stay on GitHub with no effect.
