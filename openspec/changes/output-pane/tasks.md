# Tasks

The scenarios of `output-pane` get their tests in `crates/mu-tui/tests/pane.rs`, on a `TestBackend`. The scenarios of `terminal-client` get their tests in `crates/mu-tui/tests/app.rs` or, where they need a terminal, in `crates/mu-cli/tests/client.rs`.

## 1. Sources and the backend

- [x] 1.1 Read each review thread and review comment of pull requests #29, #30 and #50. Make sure that the design names each defect that the threads found, or names why it does not apply to this change.
- [x] 1.2 Read the current documentation of crossterm and of Termina for the facts of D1: the replies to terminal queries, the async event stream on tokio, the kitty keyboard protocol, bracketed paste, and a ratatui backend. Add a table "Checked facts" to the design, with one row for each fact and its source. Make sure that each difference from the sources is an edit of the design that its row names.
- [x] 1.3 Add the crate `spikes/terminal-backend` with one program for each backend, as D1 describes. Each program prints a report of the three checks. Make sure that `cargo build -p terminal-backend` passes in `spikes/`, and that the test `spike_crates_have_decision_pages` passes with the page of task 2.1.
- [x] 1.4 Run both programs in a Rex block through the `rex` CLI. Send Shift+Enter with `rex send-key`, and paste with `pbcopy` and the `pane.paste` action of the Rex app client. Copy the reports into `spikes/terminal-backend/results`. Choose the backend with the rule of D1. Make sure that the choice and the table of the three checks for each backend are in the design.

## 2. The architecture and the skeleton

Load the `rust-best-practices` skill before this group.

- [x] 2.1 Write the decision page `docs/src/content/docs/decisions/terminal-client-architecture.md`, with the sections Decision, Reason, Not tested and Source. Add a component view of `mu-tui` and a dynamic view of one key press to `architecture/`. Embed both in the new page `docs/src/content/docs/architecture/terminal-client.mdx`, and link that page from the decision page. Make sure that `pnpm model:check` and `mise run docs:check` pass.
- [x] 2.2 Do this task only for a choice of Termina in task 1.4. For a choice of crossterm, mark it done with no change. Add Termina to `[workspace.dependencies]`. Move `mu-tui` and the panic hook onto it. Remove `crossterm` from each crate that no longer uses it. Make the same edits in the `mu-tui` row of `ALLOWED_CRATES` in `crates/mu-cli/tests/workspace_boundary.rs`. Make sure that `cargo deny check` and `cargo nextest run --workspace --locked` pass.
- [x] 2.3 Add `Msg`, `Action`, `Effect`, the keymap and the layout tree to `mu-tui`, as D2 describes. Add the functions `update` and `view`. Move the behavior of `App` onto them with no change. Point the tests in `crates/mu-tui/tests/app.rs` at the new API, and keep each assertion. Make sure that every test in `crates/mu-tui/tests/app.rs` passes, and that every test in `crates/mu-cli/tests/client.rs` passes with no edit. Make sure that `world_order` still tests "The order of the worlds", `no_worlds` still tests "No worlds" and `line_prefixes` still tests "Prefixes", each with its assertion on the rows.
- [x] 2.4 Read the current documentation of `insta` for a snapshot of a ratatui `Buffer`. Add it to `[workspace.dependencies]` as a dev-dependency of `mu-tui`. Add it to the `mu-tui` row of `ALLOWED_CRATES` in `crates/mu-cli/tests/workspace_boundary.rs`. Add one snapshot test of the first view. Make sure that `cargo nextest run -p mu-tui --locked` passes, and that the snapshot file is in the commit.
- [x] 2.5 Add unit tests of `update` for each default entry of the keymap: Ctrl+C, Ctrl+N, Ctrl+P and Enter. Each test checks the model and the list of effects, with no terminal. Make sure that the tests pass.

## 3. The line store

Load the `rust-best-practices` skill before this group.

- [x] 3.1 Add `unicode-segmentation` and `unicode-width` to `[workspace.dependencies]` at the versions in `Cargo.lock`, and to the `mu-tui` row of `ALLOWED_CRATES` in `crates/mu-cli/tests/workspace_boundary.rs`. Add the line store of D3 to `mu-tui`: chunks of 4096 lines, absolute line ids, one record for each line, spans as a byte offset and a style id, and a style table. Add the first live line of D3, a trim that moves it one line at a time, and the free of a chunk with no live line. A trim of an empty store or of zero lines returns with no change. Add `oldest_line_goes` for "The oldest line goes" and `empty_pane_trims` for "An empty pane". Add a property test: after any number of pushes into a capacity of 1 to 10000, the store holds at most its capacity and keeps the newest line. Make sure that the three tests pass, and that `empty_pane_trims` fails while the trim reads the first chunk with no check.
- [x] 3.2 Add the wrap of D3. It breaks at a space where it can and inside a word only where the word is wider than the area. Add a property test for any line and any width from 0 to 300. At width 0, the wrap gives no rows. At a width above 0, the rows keep every byte of the line in order, no row is wider than the width, and each line gives at least one row. The cases must include width 1 with `日`. Add `long_line_wraps_at_a_space`, `word_wider_than_the_pane`, `empty_line_takes_a_row` for "An empty line" and `grapheme_wider_than_the_pane` for "A grapheme wider than the pane". Make sure that the five tests pass.

## 4. The output pane widget

Load the `rust-best-practices` skill before this group.

- [x] 4.1 Read the current documentation of ratatui for the cell option `ForcedWidth`, for `TestBackend`, and for `Scrollbar` with `ScrollbarState`. Add the rows to the table "Checked facts". Make sure that each difference from the sources is an edit of the design.
- [x] 4.2 Add the pane as a `StatefulWidget` with the anchor of D3, the follow at the bottom and the clear of the anchor. Add the ratatui `Scrollbar` with the two-pass wrap of D3, and the map from cells to positions of D2. Move an anchor or a selection end on a dropped line as D3 says. Add these tests:
  - `follows_at_the_bottom`, for "At the bottom"
  - `holds_when_scrolled_back`, for "Scrolled back"
  - `holds_when_scrolled_back_one_row`, for "Scrolled back by one row"
  - `follows_again_at_the_bottom`, for "Back at the bottom"
  - `narrower_pane_keeps_the_top_line`, for "A narrower pane"
  - `resize_at_the_bottom`, for "A resize at the bottom"
  - `anchor_on_a_dropped_line`, for "The scroll position was on a dropped line"
  - `selection_on_a_dropped_line`, for "A selection on a dropped line"
  - `selection_cleared_when_both_ends_drop`, for "A selection with both ends on dropped lines"
  - `scroll_bar_halfway`, for "Halfway"
  - `no_scroll_bar_when_rows_fit`, for "All rows fit"
  - `no_scroll_bar_when_rows_equal_the_height`, for "Exactly as many rows as the pane"
  - `scroll_bar_takes_a_column`, for "The scroll bar takes a column"
  - `no_scroll_bar_when_rows_fit_only_at_full_width`, for "Rows that fit only at the full width"
  - `second_cell_of_a_wide_grapheme`, for "The second cell of a wide grapheme"
  - `past_the_end_of_a_line`, for "Past the end of a line"
  - `no_position`, for "No position"
  - `press_after_a_trim`, a unit test of `update`: a press maps through a row map drawn before a trim, onto a line that the trim dropped, and the selection starts at the oldest live line

  Make sure that the eighteen tests pass.
- [x] 4.3 Draw the spans with their styles. A wide grapheme that does not fit at the end of a row goes on the next row. Add `style_follows_its_text_across_a_wrap` for "A red word that wraps" and `wide_grapheme_at_the_end_of_a_row` for "A wide grapheme at the end of a row". Make sure that both tests pass.
- [x] 4.4 Add the unfinished line of D5 to the pane. Add `unfinished_line_replaced_by_line` for "Replaced by the finished line" and `unfinished_line_replaced_by_unfinished_line` for "Replaced by a new unfinished line". Make sure that both tests pass.
- [x] 4.5 Add the selection: two positions, the cells in reverse and the copy text. Add `selection_copies_game_lines` for "A selection across a wrapped line" and `selection_shows_in_reverse` for "The selection shows". Add `press_on_the_unfinished_line` to `crates/mu-tui/tests/app.rs`: a press on the unfinished row starts no selection. Make sure that the three tests pass.
- [x] 4.6 Replace the list of 2000 strings of each world with one pane by pane id, with a capacity of 100000 lines. Make sure that every test in `crates/mu-tui/tests/app.rs` and `crates/mu-cli/tests/client.rs` passes, and that the test of "The first view" still passes.

## 5. Colors

Load the `rust-best-practices` skill before this group.

- [x] 5.1 Read the current documentation of `vte`, `anstyle-parse` and `ansi-to-tui`. Choose one with the rule of D4, and add it to `[workspace.dependencies]`. Add it to the `mu-tui` row of `ALLOWED_CRATES` in `crates/mu-cli/tests/workspace_boundary.rs`. Add the rows to the table "Checked facts", and the choice to D4. Make sure that `cargo deny check` passes.
- [x] 5.2 Convert each game line at its arrival, with a parser in its ground state for each event and an SGR style for each world, as D4 describes. Move the char mapping of `clean` into one function that `clean` and the parser both call. Update `control_characters_are_replaced` in `crates/mu-tui/tests/app.rs` to the scenario "Control characters". Add these tests to `crates/mu-tui/tests/app.rs`:
  - `colors`, for "Colors"
  - `other_escape_sequences`, for "Other escape sequences"
  - `color_across_two_lines`, for "A color across two lines"
  - `indexed_and_24_bit_colors`, for "Indexed and 24-bit colors"

  Restate the property test `no_control_character_is_drawn` for game lines. For any byte string with ESC, CSI, OSC and C1 bytes, no drawn cell holds a control character, and the text equals the printable chars that the parser gives. Keep the old rule, ESC shows as U+FFFD, for player, system and credential lines only. Make sure that the six tests pass.
- [x] 5.3 Take the game lines of the `aresmush` and `rhostmush` transcripts in `crates/mu-core/tests/fixtures/telnet`, as D4 describes. Decode the hex once, run the bytes through the framer of `mu-core` outside `mu-tui`, and commit the game lines to `crates/mu-tui/tests/data/`. Write in the data file the command that made it. Add `transcript_lines_convert` to `crates/mu-tui/tests/app.rs`: each line converts with no control character in its text, and the data holds at least one SGR sequence. Make sure that the test passes, and that no file in `crates/` reads a path in `spikes/`.
- [x] 5.4 Add a pseudo-terminal test `colors_on_the_terminal` to `crates/mu-cli/tests/client.rs`. A scripted fake game sends the line of the scenario "Colors", and the test reads the colors of the cells from `vt100`. Make sure that the test passes.

## 6. Unfinished lines

Load the `rust-best-practices` skill before this group.

- [x] 6.1 Send a prompt event to the pane of its world as its unfinished line. A line event adds a line and clears it. Convert a prompt on a copy of the SGR style of its world, as D4 describes. Clear each pane and each SGR style at a Reset, as D5 describes. Add these tests to `crates/mu-tui/tests/app.rs`:
  - `prompt_with_no_line_end`, for "A prompt with no line end"
  - `read_ends_inside_an_escape`, for "A read that ends inside an escape sequence"
  - `prompt_does_not_carry_its_style`, for "A prompt does not carry its style"
  - `open_osc`, for "An open OSC"
  - `reset_clears_each_pane`, for "A reset"

  Make sure that the five tests pass.
- [x] 6.2 Add `line_in_two_reads` to `crates/mu-cli/tests/client.rs` for "A line in two reads". A scripted fake game in `mu-fake-server` sends `hel` and then, after it reads a byte from the client, `lo` with a line end. Make sure that the test passes, and that it fails while the client shows a prompt event as a line.

## 7. Scrollback keys and the mouse

Load the `rust-best-practices` skill before this group.

- [x] 7.1 Add the actions `ScrollPage` and `ScrollRows`, with their default keys of D6 and the mouse wheel. After `Submit`, return the pane to the bottom. Compute the page step with `saturating_sub`, and make it at least one row. Add these tests to `crates/mu-tui/tests/app.rs`:
  - `back_and_forward`, for "Back and forward"
  - `new_line_while_scrolled_back`, for "A new line while scrolled back"
  - `enter_returns_to_the_bottom`, for "Enter returns to the bottom"
  - `mouse_wheel_scrolls_three_rows`, for "The mouse wheel"
  - `back_to_a_scrolled_world`, for "Back to a scrolled world"
  - `page_up_in_a_pane_of_one_row`, for "A pane of one row"
  - `page_keys_in_a_pane_of_no_rows`, for "A pane of no rows"

  Make sure that the seven tests pass.
- [x] 7.2 Describe the keys, the mouse wheel and the modifier key for the terminal selection in Rex in `docs/src/content/docs/guides/first-run.md`. Make sure that `mise run docs:check` passes, and that each key in the page has a test of task 7.1.

## 8. Selection and copy

Load the `rust-best-practices` skill before this group.

- [x] 8.1 Read the current documentation of OSC 52 for Rex, and of the OSC 52 command of the chosen backend that D7 names. Enable the `osc52` feature of crossterm for a choice of crossterm. Add the rows to the table "Checked facts". Make sure that `cargo deny check` passes.
- [x] 8.2 Turn a press, a drag and a release of the left button into a selection, and the release into `Effect::ClipboardWrite`. `run.rs` writes it with the OSC 52 command of the backend. Add `click_clears_selection` to `crates/mu-tui/tests/app.rs` for "A click clears", and `copy_by_drag` to `crates/mu-cli/tests/client.rs` for "Copy by drag". `copy_by_drag` reads the raw bytes that the client writes. Make sure that both tests pass.
- [x] 8.3 Add the OSC 52 setting of Rex to `docs/src/content/docs/guides/first-run.md`. Make sure that `mise run docs:check` passes.

## 9. The width probe

Load the `rust-best-practices` skill before this group.

- [x] 9.1 Read the current documentation of the replies to DECRQM, DA1 and the kitty graphics query, and of the pixel fields of the Termina window size. Read how `vt100` gives the cursor position and the cell colors. Add the rows to the table "Checked facts". Make sure that each difference from the sources is an edit of the design.
- [x] 9.2 Add the probe of D8: the queries in one write, the read of `/dev/tty` before the Termina reader starts, the reply parser with APC, the `termina::Parser` for the other bytes, and the profile of one rule for each class. Add mode 2027 and the wait of at most 500 milliseconds. Copy the reports of `spikes/char-width/results` into `crates/mu-tui/tests/data/`, and add unit tests that learn the profile of each terminal from its report. Add a unit test of the profile with a reply that gives a width of 0: the class keeps the `unicode-width` value. Add `cells_take_the_learned_width` and `cells_take_the_default_width` to `crates/mu-tui/tests/pane.rs`. Add to the wrap property test of task 3.2 the case width 5 with a learned width of 8. Make sure that the tests pass.
- [x] 9.3 Make the pseudo-terminal test helper answer the queries from its `vt100` screen, with the default answers and the variants of D8. Drop each terminal reply that arrives after the probe ends. Add `probe_adds_code_point_widths` for "A terminal that adds code point widths", `probe_with_no_answer` for "A terminal that does not answer" and `late_answer` for "A late answer" to `crates/mu-cli/tests/client.rs`. Make sure that the three tests pass, and that every other test in the file still passes.
- [x] 9.4 Turn on mouse capture at the start. At exit, turn off mouse capture, and mode 2027 if the probe turned it on. Make the Termina panic hook of D8 write the same sequences. Add `modes_at_exit` for "The modes at exit" and `mode_2027_already_on` for "Mode 2027 was already on" to `crates/mu-cli/tests/client.rs`. Make sure that both tests pass, and that `ctrl_c_quits_client_only` still passes.
- [x] 9.5 Update the Not tested part of `docs/src/content/docs/decisions/character-width.md` with what the production probe settles. Make sure that `mise run docs:check` passes.

## 10. Images

Load the `rust-best-practices` skill before this group.

- [x] 10.1 Read the current documentation of the kitty graphics protocol for unicode placeholders, the table of 297 diacritics and the virtual placement. Read the documentation of `ratatui-image`, and decide with the check of D9 whether the pane uses it. Add the rows to the table "Checked facts", and the result to D9. Make sure that each difference from the sources is an edit of the design.
- [x] 10.2 Start this task after task 10.1. If 10.1 chose `ratatui-image`, add it to `[workspace.dependencies]` and to the `mu-tui` row of `ALLOWED_CRATES`, and draw the placeholder cells with it. Otherwise, write the placeholder cells in the pane. Add the image line to the store, the size limits of 1 to 297 rows and columns, the clip at the right edge, and the URL fallback through `clean`. Add these tests to `crates/mu-tui/tests/pane.rs`:
  - `image_rows_do_not_wrap`, for "An image at two widths"
  - `image_placeholder_cells`, for "Placeholder cells"
  - `image_rows_capped_at_297`, for "The size limits of an image"
  - `image_id_above_255`, for "An image id above 255"
  - `image_wider_than_the_pane`, for "An image wider than the pane"
  - `image_url_fallback`, for "No kitty graphics"
  - `url_with_control_characters`, for "A URL with control characters"
  - `image_copies_its_url`, for "The copy of an image"

  Make sure that the eight tests pass, and that `image_rows_capped_at_297` fails while the pane takes the row count of the image with no limit.
- [x] 10.3 Add `Effect::TransmitImage`, and write the kitty graphics command with a virtual placement in `run.rs`. Add a unit test of the bytes of the command for a small image. Make sure that the test passes.
- [x] 10.4 Add the example `crates/mu-tui/examples/image-pane.rs`. It shows the pane with lines and one image from a PNG file. Make sure that `cargo build -p mu-tui --examples --locked` passes.
- [x] 10.5 Ask the owner to check in Rex: the image of the example, a copy by drag into another program, and a live resize of the client with a long scrollback. Record the result in the Not tested parts of `decisions/output-pane.md` and `decisions/terminal-client-architecture.md`. Make sure that `mise run docs:check` passes.

## 11. Integration

- [ ] 11.1 Run `cargo nextest run --workspace --locked`, `cargo clippy --workspace --all-targets --locked`, `cargo deny check` and `mise run docs:check`. Make sure that each one passes, and that no test in the diff sleeps or waits on a clock.
- [ ] 11.2 Run `openspec validate output-pane --strict`. Make sure that it passes, and that each scenario of both specs has the test that a task of this file names.

## Workflow follow-up

- Run `/sb:openspec-review output-pane implementation` before group 11, as `openspec/config.yaml` asks, and treat changes-requested as a blocker.
- Run `/sb:suite-review` on the branch before the pull request opens.
- The pull request closes #55. Ask the owner before a comment on #13 that says which part of the image work this change did.
- Archive the change after the merge, with an implementation review that is not changes-requested.
