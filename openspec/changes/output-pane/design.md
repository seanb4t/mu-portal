# Design

## Context

See proposal.md for the reason and the scope. The owner set the order of the work on 2026-11-18, in the comment on #55: the backend, then the architecture and the skeleton, then the pane.

The client today is three files in `mu-tui`. `App` in `app.rs` holds the worlds, the focus, the notices, one `TextArea` and a flag for a lost stream. `on_key` returns an `Action` (`Send` or `Quit`), and `draw` renders a fixed layout of three parts. `run.rs` runs one `tokio::select!` over the crossterm `EventStream`, the event stream of `mu-client` and the replies of calls. Each world keeps a `VecDeque<String>` of at most 2000 lines. `clean` turns every control character into U+FFFD. `line` ignores the event kind, so a prompt event shows as a line of its own.

The tests of the client run the built binary in a pseudo-terminal of 80 by 24 (`crates/mu-cli/tests/client.rs`). The test helper reads the screen with `vt100`. It answers no terminal query.

This change builds on three spikes:

| Spike | Decision page | Pull request |
|---|---|---|
| Output pane, issue #3 | `decisions/output-pane.md` | #29 |
| Character width, issue #8 | `decisions/character-width.md` | #30 |
| Input editor, issue #4 | `decisions/input-editor.md` | #50 |

The input editor spike matters here only for the backend criteria. #56 builds the editor.

## Goals / Non-Goals

**Goals:**

- One recorded architecture that #56, #101 and #102 add widgets to, with no change to the shape.
- A backend choice made against written criteria, with the result on the decision page.
- A pane that shows real game traffic: colors, unfinished lines, scrollback, selection and copy.
- The width probe and the image render path from the spikes, as production code under tests.

**Non-Goals:**

- A configurable layout, keymap or statusline. The shapes are data, but the client builds them in code. #101 adds the files.
- A second pane on the screen. The model holds panes by id, but the screen shows one pane, the one of the world in focus.
- Image sources and any fallback other than the URL line. #13 keeps them.
- A test of how a terminal behaves. The tests cover our code. Rex is the one terminal where the client is checked live. A scripted test helper acts as a terminal that lacks a feature.
- Better performance than the spike measured. The tests check behavior, not speed.

## Decisions

### D1. The backend is chosen by a comparison, before any other code

Task group 1 builds two small programs outside the workspace crates, one for crossterm and one for Termina. Each one runs on tokio and checks three things:

1. The width probe in the same event loop as the keys. The program sends XTVERSION, DECRQM 2027, cursor position queries and DA1, and it reads each reply as an event. A key pressed during the probe still arrives as a key.
2. The kitty keyboard protocol together with bracketed paste. Shift+Enter arrives as a key with its modifier, and a paste of several lines arrives as one event.
3. An async event stream that `tokio::select!` can read, beside a second stream.

The programs run in Rex. The comparison tests how each library parses and delivers the bytes, and that does not depend on the terminal. Rex answers each query and has the kitty keyboard protocol, so each check can pass there. The result goes on the decision page `terminal-client-architecture`, in a table of the three checks for each backend.

The rule for the choice: the backend that passes all three wins. If both pass, crossterm stays, because the client and ratatui already use it. Assume that only crossterm fails check 1. If the probe can run before the crossterm event stream starts, crossterm still wins. The probe then reads the terminal raw, as the char-width spike did. Then the decision page says that later queries, for example color reports, need a change of backend.

Alternative: a separate spike issue. The owner decided on 2026-11-18 that the comparison is the first task of #55.

The result, from the reports in `spikes/terminal-backend/results` (Rex, 2026-11-20):

| Check | crossterm 0.29.0 | Termina 0.3.3 |
|---|---|---|
| 1. Replies as events, and a key during the probe | Fails. The mode report, the cursor position and DA1 are dropped. The XTVERSION reply arrives as 23 keys, Alt+P first | Passes for the mode report, the cursor position and DA1. The XTVERSION reply is dropped, with no stray key. The key `x` arrives |
| 2. Shift+Enter, and a paste of three lines | Passes | Passes |
| 3. A stream in `tokio::select!` beside a second stream | Passes | Passes |

Neither backend passes all three checks, so the rule above does not decide. The owner chose Termina on 2026-11-20. Termina gives every reply that the client reads as an event, so a late reply can be dropped by kind. crossterm turns a late XTVERSION reply into keys that reach the input line. The client does not ask for XTVERSION, because Termina 0.3.3 and 0.4.0 parse no reply to it and no rule uses the name. The paste in check 2 is the bracketed paste bytes that `rex send` wrote, because the `pane.paste` action of the Rex app did not reach the pane.

Enforced by: review of the decision page. The programs stay out of `crates/`, so no test of the workspace depends on them.

### D2. The skeleton: TEA with an Action enum and widgets by pane id

```text
 sources, each one becomes a Msg           model, one, owned by the loop
 +---------------------------+            +--------------------------------+
 | terminal events (backend) |--+         | layout tree (data)             |
 | server stream (mu-client) |--+-> Msg ->| focus, worlds and their states |
 | replies of calls          |--+   |     | pane states by pane id         |
 | probe replies             |--+   |     | input line                     |
 +---------------------------+      |     +--------------------------------+
                                    v
                     keymap (data): key -> Action
                                    |
                                    v
              update(model, Msg or Action) -> Vec<Effect>
                                    |
              +---------------------+----------------------+
              v                                            v
 effects: Send, ClipboardWrite,                  view(model, frame):
   TransmitImage, Quit                             walk the layout tree,
                                                   render each widget
```

- `Msg` is every input of the model: a terminal event, a client event, a reply and a probe result.
- `Action` is what a key or a client command means: `Quit`, `FocusNext`, `FocusPrev`, `Submit`, `ScrollPage(Up|Down)`, `ScrollRows(n)`. A key with no action goes to the input line.
- The keymap is a table from a key with its modifiers to an `Action`. The client builds the default table in code. #101 reads it from a file.
- `update` changes the model and returns a list of `Effect`. It does no I/O. `run.rs` performs each effect: a call of `mu-client`, a write to the terminal, or the end of the loop.
- `view` walks the layout tree. The tree has three node kinds: a split with a direction and constraints, tabs, and a leaf. A leaf names a view: the world tabs, an output pane by id, or the input line. The client builds the tree of today's screen in code. #101 owns the schema and the file.
- Each widget is a ratatui `StatefulWidget`. Its state knows nothing of the app. The model holds the pane states in a map by pane id, and each world has one pane id.
- The render of a pane also stores the map from screen rows to lines and bytes in the pane state. `update` uses that map to turn a mouse position into a line and a byte. The map can be older than the last trim. So `update` moves a position on a line behind the first live line to the start of the oldest live line, the rule of D3 for a selection end. Each cell of a wide grapheme maps to the byte where that grapheme starts. A cell past the end of the text of a row maps to the end of that row. On the last row of a line, that is the end of the line. A cell of the scroll bar or of the unfinished line maps to no position, so a press there starts no selection.

The move of the plain view onto the skeleton comes first, with no change in behavior. The scenarios of `terminal-client` have tests at two levels today. `crates/mu-tui/tests/app.rs` drives `App` and draws it on a `TestBackend`. `crates/mu-cli/tests/client.rs` runs the built binary in a pseudo-terminal. After the move, the tests in `app.rs` call the new API, and each assertion stays the same. The tests in `client.rs` pass with no edit.

Alternatives: a component tree where each component owns its update (the Elm "nested TEA" shape, or the component pattern of the ratatui templates). One update at the top keeps the keymap and Lua (Phase 4) in one place, and the widgets stay plain ratatui widgets.

Enforced by: unit tests of `update` with no terminal, the unchanged pseudo-terminal tests, and review. No test can prove that `update` does no I/O. Its signature takes no handle to the terminal or the client, and review checks that it stays so.

### D3. The line store follows the output-pane decision

The store keeps lines in chunks of 4096 lines, the size that the spike measured. A line id is absolute and never used again. A chunk holds the text of its lines in one string, the style spans in one list, and one record for each line. The record holds the end of the text, the end of the spans, the display width and flags. A span is a byte offset and an id in the style table of the pane.

- The capacity of a pane is 100000 lines, a constant. #101 can make it a setting. The tests use a small capacity.
- The store keeps the id of its first live line. A trim moves that id forward one line at a time, so the store holds at most its capacity. A chunk is freed when all its lines are behind the first live line. A trim of an empty store, and a trim of zero lines, return with no change. The review of #29 found both cases.
- After a trim, an anchor or a selection end on a dropped line moves to the start of the oldest live line. A selection with both ends on dropped lines is cleared.
- The scroll position is an anchor: a line id and a byte. The top row is the row that holds the anchor byte. A pane at the bottom has no anchor and follows new lines. A scroll that brings the last row of the newest line onto the screen clears the anchor, so a scroll back by one row keeps an anchor and PageDown to the end clears it.
- The wrap breaks at a space where it can. It works only on the lines that the screen shows, so a resize costs the same at any store size. Every text line gives at least one row, an empty line too. A grapheme wider than the area takes a row of its own and draws as U+FFFD in one cell. An area 0 columns wide gives no rows.
- Graphemes come from `unicode-segmentation` and widths from `unicode-width`, the versions that ratatui already resolves in `Cargo.lock`.
- The scroll bar is the ratatui `Scrollbar` widget, with no arrows, so a pane of two rows still shows the thumb. Its `ScrollbarState` takes the count of live lines as the content length and the index of the top line as the position. It uses lines, not rows, because the pane keeps no index of rows for the whole store.
- The pane first wraps at the full width. If the rows do not fit in the height, it shows the bar and wraps again at the width less one. Thus the bar does not turn on and off from one render to the next.

Enforced by: unit tests of the store and the wrap in `mu-tui`, and the scenarios of `output-pane` on `TestBackend`. A property test checks the wrap at any width from 0 to 300. At width 0 it gives no rows. At a width above 0, it keeps every byte of the line, in order, in rows no wider than the area, and gives each line at least one row.

### D4. SGR to styles at the arrival of a line, with a parser from a crate

At the arrival of each game line, the client converts it. A parser of escape sequences splits the line into text, SGR sequences, other sequences and control characters. The SGR sequences become spans. Other sequences are dropped. A control character that remains goes through the same char mapping as `clean`: a tab becomes a space, and each other control character becomes U+FFFD. `clean` and the parser call that one mapping, so the rule has one home.

Each event starts a new parser in its ground state, so a read that ends inside a sequence, or an open OSC, cannot eat the text of a later event. Only the SGR style carries from one game line to the next, so a color that a game sets on one line and resets on a later line shows on both. A prompt is converted on a copy of the SGR style of its world. Only a line event stores the style at its end. The framer resends the whole line after a prompt, so the line parses the same bytes again from the stored style.

Candidates under the rule of reuse before writing: `vte` (the parser of Alacritty), `anstyle-parse`, and `ansi-to-tui`. `ansi-to-tui` builds ratatui text, but the store needs spans of its own shape, and a world needs SGR state across lines. So a state-machine parser such as `vte` fits better. Task 5.1 reads the current documentation of each candidate and records the choice in the design of this change before code depends on it.

A player line and a system line keep their prefixes, and their text goes through `clean` as today. Only game text has styles.

Enforced by: unit tests of the conversion, and the pseudo-terminal scenarios that read cell colors from `vt100`. The test data comes from the two telnet transcripts that hold plain escape bytes, `aresmush` and `rhostmush`. The pennmush and tinymux transcripts hold no escape byte, and the Evennia reads are compressed with MCCP. Task 5.3 decodes the hex of the two files once, runs them through the framer of `mu-core` outside `mu-tui`, and commits the game lines as test data. The 256 and 24-bit colors come from the scenarios, not from a transcript.

### D5. An unfinished line is state of the pane, not a line of the store

A prompt event sets the unfinished line of the pane of its world. A later prompt replaces it. A line event clears it and then adds the line to the store. The pane draws the unfinished line after the newest line, as its last row. A selection cannot start in the unfinished line, because its cells map to no position.

A Reset of the server clears the store, the unfinished line, the anchor and the selection of each pane, and sets the SGR style of each world to the default.

The telnet framer gives a prompt for any text with no line end at the end of a read. So a prompt is often the first part of a line that the next read finishes. MU games have no real prompt line. The owner decided on 2026-11-18 that an unfinished line shows as the last output row and that the client has no prompt line.

Enforced by: the scenarios of both specs. The pseudo-terminal test splits a line over two reads of the scripted fake game in `mu-fake-server`.

### D6. Scroll keys and the mouse are default entries of the keymap

PageUp and PageDown map to `ScrollPage`. The mouse wheel maps to `ScrollRows(3)` and `ScrollRows(-3)`. Enter maps to `Submit`, and `update` returns the pane of the world in focus to the bottom after a submit. Ctrl+End and End are not used, because the input line needs End.

Mouse capture is on, with SGR mouse reports. At exit, and in the panic hook of D8, the client turns mouse capture off. Today `ratatui::restore` does not do that. The decision page of the output pane already says that the terminal selection stays available with the modifier key of the terminal. The user guide says which key that is in Rex.

Enforced by: unit tests of `update` for each action, and the pseudo-terminal scenarios.

### D7. A drag selects, and the release copies through OSC 52

A press of the left button starts a selection at its line and byte. A drag moves its end, and a release ends it and returns an `Effect::ClipboardWrite`. `run.rs` writes the copy text with the OSC 52 command of the chosen backend: crossterm `CopyToClipboard` with its `osc52` feature, or the OSC 52 command of Termina. Thus the client needs no base64 crate of its own. A press with no drag clears the selection. The copy text joins the selected text of each line with LF. An image line gives its URL.

Enforced by: unit tests of the copy text, and a pseudo-terminal scenario that reads the raw bytes that the client writes.

### D8. The width probe runs before the first frame, and the test helper answers it

The probe follows the character-width decision, with one difference: it does not send XTVERSION, because Termina drops the reply (D1). It also asks for the size of a cell in pixels and sends a kitty graphics query, because the image path needs both. All queries go in one write, and DA1 ends them. The probe waits at most 500 milliseconds for the DA1 reply. With no reply, every class keeps the `unicode-width` value, and an image line shows its URL. A class that the terminal reports as 0 wide also keeps the `unicode-width` value, because `ForcedWidth` holds a `NonZeroU16`. After the probe ends, the event loop drops each terminal reply, so a late reply never reaches the input line.

If the probe turned mode 2027 on, the client turns it off at exit. If the mode was already on, the client leaves it on. The client sets its own panic hook after `ratatui::try_init`. The hook writes the sequences that turn off mouse capture and, if the probe turned it on, mode 2027. Then it calls the hook that `try_init` installed, which leaves raw mode and the alternate screen.

`ForcedWidth` is a ratatui cell option. The pane sets it on each cell of a risky class. After a cell wider than one, ratatui moves the cursor before the next cell, so the terminal and the buffer agree. The review of #30 found a row render that wrote past the buffer for a wide forced width. The pane wraps with the forced width, so no row is wider than the area. The scenario "A wide grapheme at the end of a row" tests that.

The pseudo-terminal test helper answers the queries from its `vt100` screen. It answers DA1 at once and answers each cursor position query from the cursor of the screen. It answers DECRQM 2027 as known and off. The scenario "The modes at exit" uses this default. Other variants answer with a scripted width table, with DECRQM 2027 as set, with no answer, or with every answer after the first frame. Thus every existing client test starts at once, and the tests need no wait on a clock.

Enforced by: unit tests of the reply parser and of the learned profile, with the replies in `spikes/char-width/results` copied into `mu-tui` as test data. Also the pseudo-terminal scenarios.

### D9. The image path: kitty placeholders, fed only by tests and an example

An image line holds an image id, a name and its size in cells, from 1 to 297 rows and columns. A size of 0 becomes 1, and a size above 297 becomes 297. The pane renders each cell as U+10EEEE with the diacritic of its row and of its column. The foreground color of the cell is the image id as a 24-bit color, so the client assigns ids from 1 to 2^24 - 1 and needs no third diacritic. The pane clips the columns of the image at the right edge of the area. ratatui passes these cells through its buffer with no change, as the spike showed.

The pixels go to the terminal once, as an effect. `Effect::TransmitImage` writes a kitty graphics command with a virtual placement before the next frame.

Before code depends on this design, task 10.1 checks `ratatui-image` under the rule of reuse. The check: can it render kitty unicode placeholders into the ratatui buffer as a line of fixed rows? If it can, the pane uses it. Otherwise the pane writes the placeholders itself, from the spike as an example only. Task 10.2 follows the result of 10.1.

The URL row goes through `clean`, because #13 will take the URL from the game. In this change, no event of the server holds an image. The tests give the pane an image line. A development example shows one in a real terminal, for the visual check that the spike did not do. The command is `cargo run -p mu-tui --example image-pane -- <file.png>`.

Enforced by: the scenarios of `output-pane` on `TestBackend`, and the visual check in task 10.5.

### D10. Tests: update, TestBackend snapshots and the binary

- `update` gets unit tests with no terminal.
- The pane gets `TestBackend` tests for the scenarios of `output-pane`. Snapshots of the rendered buffer use a snapshot crate, `insta` being the candidate, named in task 2.4 after a read of its documentation.
- A scenario of `terminal-client` that only needs the model gets a test in `crates/mu-tui/tests/app.rs`, on a `TestBackend`, as today. A scenario that needs the terminal gets a pseudo-terminal test on the built binary in `crates/mu-cli/tests/client.rs`. Those are the probe, the modes at exit, the bytes of OSC 52 and a line in two reads.

No test sleeps or waits on a clock. The pseudo-terminal tests wait on the screen with the bound that the test helper already has.

### Using the spikes

The design follows the part "Using a spike" of the Contributing page for spikes.

1. The decision pages and the pull requests are in the table under Context.
2. The review threads: #29 found the empty trim and the 297-row limit (D3, D9). #30 found the row render past the buffer (D8). #50 is the editor and only shapes the backend criteria in D1. Its one finding, numbers on the editor page that differ from `bench.tsv`, concerns that page only and does not apply here. Task 1.1 read each thread of the three pull requests again on 2026-11-20 and found no other defect.
3. The items under Not tested:

   | Page | Item | Where it goes |
   |---|---|---|
   | output-pane | visual check of the image, a copy by drag and a live resize in Ghostty and Rex | task 10.5, in Rex. Ghostty is not a target of our tests |
   | output-pane | Linux | CI runs every test on `ubuntu-latest` and `macos-latest`. The visual check of task 10.5 runs on macOS only. The decision page keeps the visual check on Linux under Not tested |
   | output-pane | output from a game | task 5.3, the transcripts as test data |
   | output-pane | a wrap of all lines on more than one thread | non-goal: the pane never wraps all lines |
   | character-width | tmux | non-goal: tmux is not a test target |
   | character-width | an East Asian locale with wide ambiguous characters | non-goal: the ambiguous class has its own probe, and no tester has that locale |

4. New code goes under the tests of this change. No spike code is copied.
5. The evidence that the tests read is copied into `crates/mu-tui/tests/data/`: the width reports of `spikes/char-width/results` and the game lines of the `aresmush` and `rhostmush` transcripts.

### What enforces each rule

| Rule | Enforced by |
|---|---|
| `mu-tui` never depends on `mu-core` | the boundary test in `crates/mu-cli/tests/workspace_boundary.rs` |
| No file of a crate includes a file of `spikes/` | the boundary test |
| `update` does no I/O | review, and the signature of `update` |
| A widget state does not know the app | review |
| A third-party crate enters through `[workspace.dependencies]` | `cargo deny` and the workspace lint in CI |
| `mu-tui` uses only the crates that its row of `ALLOWED_CRATES` names | the boundary test in `crates/mu-cli/tests/workspace_boundary.rs` |
| No text of a game sends a sequence to the terminal | the scenarios "Control characters" and "Other escape sequences" |
| The terminal comes back at exit | the scenario "The modes at exit" |
| Each scenario has a test | the task that names it, and the implementation review |

### Facts about third-party tools to check

Each fact below comes from a spike or from memory, not from the current documentation. A task reads that documentation before the work depends on the fact.

| Fact | Task |
|---|---|
| The crossterm event reader does not give back the replies to terminal queries | 1.2 |
| Termina gives terminal replies as events, runs on tokio, and has a ratatui backend | 1.2 |
| ratatui has the cell option `ForcedWidth`, and `TestBackend` keeps it | 4.1 |
| `vte`, `anstyle-parse` and `ansi-to-tui` parse SGR as D4 needs | 5.1 |
| The format of OSC 52 that Rex accepts | 8.1 |
| The replies to DECRQM, the cell size query and the kitty graphics query | 9.1 |
| `vt100` gives the cursor position and the colors of a cell | 9.1 |
| The kitty unicode placeholder: U+10EEEE, the table of 297 diacritics, the virtual placement | 10.1 |
| `ratatui-image` can render kitty placeholders into the buffer | 10.1 |
| crossterm `CopyToClipboard` and the Termina OSC 52 command write the format that D7 needs | 8.1 |
| ratatui `Scrollbar` takes a content length and a position in lines | 4.2 |
| `insta` snapshots a ratatui `Buffer` | 2.4 |

### Checked facts

Each row is a fact that a task read in the current source or documentation. The Source column names where.

| Fact | Result | Source |
|---|---|---|
| crossterm gives the replies to terminal queries as events | No. A cursor position report and DA1 are internal events that `read` and `EventStream` drop. crossterm has no DCS parser, so an XTVERSION reply (`ESC P`) reads as Alt+P | crossterm 0.29.0, `src/event.rs` (`InternalEvent`), `src/event/sys/unix/parse.rs` |
| crossterm has an async event stream | Yes, `EventStream` with the `event-stream` feature, which the workspace already turns on | crossterm 0.29.0 README, feature table |
| crossterm has the kitty keyboard protocol and bracketed paste | Yes: `PushKeyboardEnhancementFlags`, `EnableBracketedPaste` and `Event::Paste` | crossterm 0.29.0, `src/event/commands.rs` |
| Termina gives the replies to terminal queries as events | Yes. `Event::Csi` holds a cursor position report, DA1 and a mode report. No `DcsResponse` matches the XTVERSION reply, so the parser drops it | termina 0.3.3 and 0.4.0, `event::Event`, `src/escape/dcs.rs`, `parse_dcs` in `src/parse.rs` |
| Termina has an async event stream | Yes, `EventStream` with the `event-stream` feature. It is a `futures` `Stream` that a helper thread wakes, as crossterm does | termina 0.3.3, `src/event/stream.rs` |
| Termina has the kitty keyboard protocol and bracketed paste | Yes: `KittyKeyboardFlags`, `Event::Paste`, and `Event::Key` with modifiers | termina 0.3.3, `escape::csi`, `event::Event` |
| ratatui has a Termina backend | Yes, the `termina` feature of ratatui 0.30.2 pulls `ratatui-termina` 0.1.0. That crate needs termina `^0.3`, while termina 0.4.0 is out | crates.io, ratatui 0.30.2 features and `ratatui-termina` 0.1.0 dependencies |
| A cell can carry a forced width | Yes, `Cell::set_diff_option(CellDiffOption::ForcedWidth(NonZeroU16))`. The diff writes the cell and skips the next width less one cells. `cell_width` gives the forced width | ratatui-core 0.1.2, `src/buffer/cell.rs`, `src/buffer/diff.rs` |
| A test can read the cells that a render drew | Yes, `TestBackend::buffer` gives the `Buffer`, and `TestBackend` formats as one quoted string for each row | ratatui-core 0.1.2, `src/backend/test.rs` |
| The scroll bar takes a length and a position | Yes, `ScrollbarState::new(content_length).position(n)`. The thumb starts at the position times the track length, divided by the last position plus the viewport length. The default vertical bar has an arrow at each end | ratatui-widgets 0.3.2, `src/scrollbar.rs` |

## Risks / Trade-offs

- [The change is large.] → The task groups land in order, and each group ends with passing tests. The skeleton group changes no behavior, so a review can read it alone.
- [Termina is young, and its ratatui backend can lag behind ratatui.] → D1 keeps crossterm on a tie, and the decision page records the gap.
- [Mouse capture takes the selection from the terminal.] → The modifier key still gives the terminal selection, and the user guide says so.
- [A terminal that answers DA1 late makes the client start slowly.] → The probe waits at most 500 milliseconds, once.
- [Some terminals turn OSC 52 off, or need a setting for it.] → The copy is a write with no reply. Thus the client cannot see a failure. The user guide names the setting in Rex.
- [A game can forget a reset. Then an SGR state that lasts across lines colors many lines.] → This matches what a terminal does with the same bytes. A player line and a system line do not take the state.
- [Snapshots can hide a wrong render behind an accepted change.] → Each snapshot test also asserts the cells that its scenario names.

## Migration Plan

No data moves. The client keeps its command and its flags. A player sees three changes at once: colors, scrollback with keys and the mouse, and the mouse capture. The user guide changes in the same pull request. A rollback is a revert of the pull request.
