# Proposal

## Why

The terminal client cannot show a real game. It shows each control character as U+FFFD, so the colors of AresMUSH and RhostMUSH arrive as `�[1;31m`. It keeps 2000 lines for each world, with no scrollback and no selection. It shows an unfinished line twice: once as a prompt and again as the finished line. Issue #55 tracks this change. It is step 6 of the Phase 1 plan in #61, and the first evening of play waits for it, #56, #101 and #102.

The client also has no recorded architecture. The plain view grew into an Elm-like shape by chance, and every spike used crossterm by default. #56, #101 and #102 each add a widget, so the skeleton and the backend must be fixed first. The owner decided on 2026-11-18 that this change does that, in this order: choose the backend, then record the architecture and move the plain view onto it, then build the output pane.

## What Changes

- A comparison of crossterm and Termina as the backend of ratatui, and a choice of one. The comparison checks three things. The width probe must run in the same event loop as the keys. The kitty keyboard protocol must work with bracketed paste. The events must come as an async stream on tokio.
- A new decision page, `terminal-client-architecture`, with a LikeC4 component view of `mu-tui` and a dynamic view of one key press. The client takes the Elm architecture (TEA) at the top: one model, one `update` that returns effects, and one `view`. An `Action` enum and a keymap that is data from keys to actions. Each widget is a ratatui `StatefulWidget` whose state does not know the app, and the model holds the states by pane id. The layout is a tree of data. Input and output happen only through effects.
- The plain view moves onto that skeleton with no change in behavior. The scenarios of `terminal-client` that exist today guard the move.
- The output pane, as the first widget. It replaces the list of 2000 strings with a line store in chunks. It wraps only the lines on the screen. It keeps its scroll position as an anchor, so a resize keeps the same line at the top.
- Colors. When a game line arrives, the client converts its SGR sequences to styles. It drops each other escape sequence, and it shows the control characters that remain as U+FFFD, as today.
- Unfinished lines. A prompt event of a world shows as the last row of its pane. A later prompt or line of the same world replaces that row.
- Scrollback. PageUp, PageDown and the mouse wheel scroll the pane. A scroll bar shows the position. The pane follows new lines only at the bottom. Enter returns the pane to the bottom.
- Selection and copy. A drag of the mouse selects game lines, not wrapped rows. The release copies the selection to the clipboard through OSC 52. Mouse capture is on, and the modifier key of the terminal still gives the selection of the terminal.
- The character width probe at startup, from the character-width decision. The pane forces the learned width on each cell of a risky class. A terminal that gives no answer gets the `unicode-width` values.
- The render path of inline images. An image is one line of its own. The pane shows it with kitty unicode placeholders and a virtual placement, at most 297 rows and 297 columns. A terminal without the kitty graphics protocol gets the URL of the image as one text line. In this change only tests and a development example give the pane an image.
- The production pane keeps the two limits from the review of the spike in pull request #29: a trim that works on an empty store and on `trim(0)`, and the limit of 297 placeholder rows.

## Not in this change

- The sources of images. #13 keeps them: Pueblo, MXP and GMCP media, and the fetch of a remote image. #13 also keeps the allow setting for each world, the cache, and any fallback other than the URL line.
- The pose editor, bracketed paste in the editor and the kitty keyboard protocol in the editor. #56 builds them. This change only checks that the chosen backend can give them.
- A layout that the player configures, tabs, the statusline, input history, a configurable keymap and client commands. #101 builds them on the skeleton of this change.
- Spawns and the routing of lines to named panes. #102 builds them.
- Search in the scrollback and OSC 8 hyperlinks. No issue tracks either one yet.
- Floating windows and layouts from Lua. They come after Phase 1.
- A limit on the journal events that an attached client reads. The client reads from sequence 1, as today.
- A test of how a terminal behaves. Rex is the one terminal where this change checks the client live. The tests of the client act as other terminals with scripted replies to the probe.
- tmux. It is not a test target for now.

## Capabilities

### New Capabilities

- `output-pane`: the output widget of the terminal client. It covers the line store and its limits, the wrap of the lines on the screen, the scroll anchor and the scroll bar. It also covers the selection and its copy text, the forced width of cells, and image lines. Each pane and each later spawn is one instance of it.

### Modified Capabilities

- `terminal-client`: game colors show as styles, and other escape sequences are dropped. An unfinished line shows once as the last row and is replaced. New requirements cover the scrollback keys, the mouse, and the selection with copy through OSC 52. Others cover the width probe at startup and the terminal modes that the client turns on and restores at exit.

## Impact

- Code: `mu-tui` gains the TEA skeleton: the model, `update`, effects, `view`, `Action` and the keymap. It also gains the output pane widget and its line store. The SGR conversion, the width probe and the kitty placeholder cells come with the pane. `run.rs` turns effects into calls of `mu-client` and writes to the terminal.
- Backend: if Termina wins the comparison, `crossterm` leaves `mu-tui` and Termina enters through `[workspace.dependencies]`. If crossterm wins, the probe reads its replies outside the event stream of crossterm, and the design names how.
- Dependencies: a parser of escape sequences from an existing crate, and a snapshot test library, each chosen under the rule of reuse before writing. Each one joins `[workspace.dependencies]` and the `mu-tui` row of `ALLOWED_CRATES` in the boundary test. The OSC 52 write uses the command of the chosen backend. The design names the candidates and the checks.
- Tests: the pseudo-terminal tests in `crates/mu-cli/tests/client.rs` answer the probe queries from the `vt100` screen, so the client starts at once in tests. New tests in `mu-tui` test `update` and the widget on `TestBackend`.
- Test data: the game lines of the `aresmush` and `rhostmush` transcripts in `crates/mu-core/tests/fixtures/telnet` feed the color tests, as copies in `mu-tui`. The other transcripts hold no plain escape bytes.
- Docs: the new decision page, the LikeC4 views in `architecture/`, and a new architecture page that embeds them. The output-pane and character-width pages change where the production code settles an item under Not tested.
- Issues: closes #55. Leaves #13, #56, #101 and #102 open.
