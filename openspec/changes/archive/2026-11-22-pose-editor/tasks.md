# Tasks

The scenarios of `input-editor` get their tests in `crates/mu-tui/tests/editor.rs` on a `TestBackend`, or in `crates/mu-cli/tests/client.rs` where they need a terminal. D10 of the design lists the place of every other test.

## 1. Sources

- [x] 1.1 Read each review thread and review comment of pull request #50. Make sure that the design names each defect that the threads found, or names why it does not apply to this change.
- [x] 1.2 Read the current documentation of the Renovate cargo manager for a git dependency in `[patch.crates-io]`. If it does not read one, plan the custom regex manager of D2 for task 3.4. Add a row to the table "Checked facts". Make sure that D2 states which of the two ways the change uses.
- [x] 1.3 Read the current documentation of cargo-deny for `allow-git` in `[sources]`. Add a row to the table "Checked facts". Make sure that the form that task 3.4 writes matches the documentation.

## 2. The line break of a world

Load the `rust-best-practices` skill before this group.

- [x] 2.1 Read the current documentation of TinyMUX, AresMUSH and Evennia. Find the text that each game reads as a line break inside one command. Read the current documentation of PennMUSH and RhostMUSH for the login line, the `IDLE` keepalive, the character set and the line break. Add a row for each game to the table "Checked facts". If PennMUSH or RhostMUSH differs from TinyMUX in a quirk, set that field in its type file of task 2.2. Make sure that each built-in value of D9 matches the documentation, or that the design and the `world-config` delta change to match it.
- [x] 2.2 Add `line_break` to the layers, `Quirks`, the resolver, `ResolvedWorld` and `mu world show`. Add `check_line_break` and the type `LineBreak` of D9, and call the check from `check_layer`. Set the field in the built-in types `tinymux`, `aresmush` and `evennia`. Add the built-in types `pennmush` and `rhostmush` of D11, and read `extends` in a built-in type. This task covers the new field of the requirements "Built-in types set only quirks", "Each field is known" and "Each value is in its range". Add these tests to `crates/mu-cli/tests/world_config.rs`:
  - `line_break_of_each_built_in_type`, for "The line break of each built-in type"
  - `game_overrides_the_line_break_of_its_type`, for "A game overrides the line break of its type"
  - `edges_of_the_line_break`, for "The edges of the line break", with each value of the scenario
  - `built_in_type_that_extends_another`, for "A built-in type that extends another"
  Add `line_break_outside_its_range` to `crates/mu-core/tests/world_config.rs`: `LineBreak::new` refuses an empty text and a text with LF, and takes `%r`. Add `built_in_type_extends_only_a_built_in_type` to the same file, for "A built-in type extends only a built-in type". In `built_in_types_hold_only_quirks`, add `rest.line_break = None` and the two new names. A type with `extends` can set no field, so keep its last assertion for a type with no `extends`. Make sure that the six new tests pass. Make sure that these tests still pass, each for the scenario of its name:
  - In `crates/mu-core/tests/world_config.rs`: `built_in_types_hold_only_quirks` and `quirks_refuse_other_fields`, for "Each built-in type reads as quirks"
  - In `crates/mu-cli/tests/world_config.rs`, for "Built-in types set only quirks": `global_preference_is_not_hidden_by_a_type` and `user_type_with_a_built_in_name`
  - In the same file, for "Each field is known": `unknown_field`, `port_out_of_range`, `template_with_no_password`, `charset_case`, `charset_the_engine_cannot_read` and `unknown_mode`
  - In the same file, for "Each value is in its range": `edges_of_each_range`, `edges_of_the_attempts` and `line_break_in_a_line_field`
- [x] 2.3 Change the handling of `Command::Send` in `Session` to the order of D9, with the new message of `PortalError::LineBreak`. Add these tests to `crates/mu-core/tests/portal.rs` and `crates/mu-core/tests/scripts.rs`:
  - `line_break_that_the_world_joins`, for "A line break that the world joins"
  - `carriage_return_to_a_world_that_joins`, for "A carriage return to a world that joins"
  - `hook_sees_the_joined_line`, for "A hook sees the joined line"
  - `joined_text_is_one_line`, a property test with the `proptest` of the repository, as in `crates/mu-core/src/framer.rs`. For each text with no CR and each valid `line_break`, the game gets one line with no CR and no LF.
  Make sure that the four tests pass, and that `line_break_that_the_world_joins` fails while the session refuses each LF. The helper `refused_line_break` of `crates/mu-core/tests/portal.rs` compares with `PortalError::LineBreak`. Edit it for the new field of the error. This task changes the requirement "A command reaches only its world". Make sure that these tests of its other scenarios still pass:
  - In `crates/mu-core/tests/portal.rs`: `send_with_lf_is_refused`, for "A line break in the text", with no other edit, because its world has no `line_break`
  - In the same file: `send_with_cr_is_refused`, `two_characters_on_one_game`, `send_to_unknown_world`, `send_to_disconnected_world` and `send_to_connecting_world`
  - In the same file: `failed_write_records_disconnect`, for "A write that fails", and `empty_send_reaches_game`, for "An empty line" on a world with no `line_break`
  - In `crates/mu-core/tests/connection.rs`: `send_to_reconnecting_world`, for "A world that waits to reconnect"
  - In `crates/mu-core/tests/scripts.rs`: `hook_line_break_is_dropped`
- [x] 2.4 Give the fake world of `mu server --fake-world` the line break `%r`, through `LineBreak::new`. Add `fake_world_gets_a_joined_line` to `crates/mu-cli/tests/worlds.rs`, for "The fake world gets a joined line". In that file, remove `send_with_lf_is_refused`, because the fake world now joins. Keep `send_with_cr_is_refused` there. Make sure that the new test passes. Make sure that these tests of the requirement "The fake world stands in for a game" still pass:
  - In `crates/mu-cli/tests/worlds.rs`: `fake_world_answers`, `fake_world_flag_is_hidden`, for "The flag is hidden", `missing_password_variable` and `fake_world_does_not_reconnect`
  - In `crates/mu-fake-server/src/memory.rs`: `fake_game_answers_mark_in_order`, for "The fake world answers a mark in order"
- [x] 2.5 Add `line_break` to `docs/src/content/docs/guides/configuration.md`, with the value of each built-in type and the refusal of a world with none. Add `pennmush` and `rhostmush` to the list of the built-in types there, with a line that no live game checked them. Make sure that `mise run docs:check` passes.
- [x] 2.6 Add `optional string line_break = 4` to the message `World` in `crates/mu-proto`, as D12 says, and fill it in `ListWorlds`. Add `list_worlds_shows_the_line_break` to `crates/mu-cli/tests/worlds.rs`, for "The line break of a world". Make sure that it passes, and that `list_worlds_shows_reconnecting` still passes, for "A world that waits to reconnect".

## 3. The fork of ratatui-textarea

Each patch is one commit on its own branch of the fork, with its own unit tests, as D1 describes. Load the `rust-best-practices` skill before this group.

- [x] 3.1 With the owner's yes, fork `ratatui/ratatui-textarea` to `seanb4t/ratatui-textarea` with `gh repo fork`. Start the branch `grapheme-cursor` at the tag of 0.9.4. Add patch 1 of D1, with its tests. The tests cover a family emoji, a flag, a letter with a combining accent and an emoji with a skin tone. Each test covers Left, Right, Backspace and Delete. Add a test of Up and a test of Down, from a line of 3 letters to a line of `a` and a family emoji. Add a test of `Jump` to a column inside that emoji. Each one leaves the cursor at a grapheme boundary. Make sure that `cargo test` passes in the fork, and that each new test fails on the tag of 0.9.4.
- [x] 3.2 Add patch 2 of D1 on the branch `grapheme-width`, from the branch `grapheme-cursor`. Add tests where a width function gives 8 for a family emoji: the cursor position, the wrap at width 10 and the drawn cells each use 8. Add a test that the default function gives the widths of 0.9.4. Add a test that the render sets `CellDiffOption::ForcedWidth` to 8 for that emoji. Add a test with `WrapMode::WordOrGlyph` and a word wider than the area, where the split uses the function. Add a test of an area of 5 columns with the emoji of width 8: the render draws no cell outside the area. Add a row for that case to the table "Checked facts". Make sure that `cargo test` passes in the fork, and that each test of a width function fails on the tag of 0.9.4.
- [x] 3.3 Add patch 3 of D1 on the branch `wrapped-rows`, from the tag of 0.9.4. Add tests of the count for an empty text, a text of one row, a line wider than the width, and width 0. Add a test that the count uses the width function of patch 2 after the merge of task 3.4. Make sure that `cargo test` passes in the fork.
- [x] 3.4 Merge the three branches into the branch `mu`. Add the `[patch.crates-io]` entry of D1 to the root `Cargo.toml`, with the full commit id of `mu`. Allow the git URL in `deny.toml` with the form of task 1.3. Add the Renovate manager that task 1.2 chose. Add `textarea_comes_from_the_fork` to `crates/mu-cli/tests/repo_config.rs`. Make sure that `cargo deny check` and `cargo nextest run --workspace --locked` pass. Make sure that `textarea_comes_from_the_fork` fails with the patch entry removed.

## 4. The editor widget

Load the `rust-best-practices` skill before this group.

- [x] 4.1 Read the current documentation of ratatui for `Frame::set_cursor_position` and `StatefulWidget`. Add the rows to the table "Checked facts". Make sure that each difference from the sources is an edit of the design.
- [x] 4.2 Add `EditorId`, `EditorState` and the `Editor` widget of D3. Replace `Leaf::Input` with `Leaf::Editor(EditorId)`, and give the caps of the probe to each editor in `Model::set_caps`. Set the width function of patch 2 from the caps, and the wrap mode to `WrapMode::WordOrGlyph`, as D3 says. Add these tests to `crates/mu-tui/tests/editor.rs`:
  - `editor_uses_the_probed_widths`, for the cell of `x` in "A terminal that adds code point widths"
  - `editor_uses_unicode_width`, for "A terminal that does not answer"
  - `word_wider_than_the_editor`, for "A word wider than the editor"
  - `line_wider_than_the_editor`, for "A line wider than the editor", with the Enter part through `update` and its `Effect::Send`
  - `editor_and_pane_wrap_alike`, which gives the texts of the two tests above to the editor and to `wrap::wrap`, and compares the rows
  Make sure that the five tests pass, and that every test in `crates/mu-tui/tests/app.rs` passes.
- [x] 4.3 Add `Size` to the layout tree, with `Size::Editor { id, max_rows }`, and the sizing of D4 to `view`. The editor then grows with its text up to the limit. Add these tests to `crates/mu-tui/tests/editor.rs`, for the requirement "The editor grows with its text":
  - `three_lines`, for "Three lines"
  - `more_rows_than_the_limit`, for "More rows than the limit"
  - `short_terminal`, for "A short terminal"
  - `terminal_of_17_rows`, for "A terminal of 17 rows"
  - `terminal_of_3_rows`, for "A terminal of 3 rows"
  - `terminal_of_1_row`, for "A terminal of 1 row", where `view` does not panic
  - `back_to_one_row`, for "Back to one row"
  Make sure that the seven tests pass. The screen still has three parts: the top row, the output pane and the rows of the editor. Add the row of the editor to `first_view` for "The first view". Make sure that these tests of the requirement "The screen has three parts" pass in `crates/mu-tui/tests/app.rs`: `first_view`, `first_view_snapshot`, `world_order`, for "The order of the worlds", and `no_worlds`, for "No worlds".
- [x] 4.4 Add these tests of grapheme moves through `update` to `crates/mu-tui/tests/editor.rs`:
  - `backspace_after_an_emoji_with_joiners`, for "Backspace after an emoji with joiners"
  - `left_over_a_flag`, for "Left over a flag"
  - `right_over_a_combining_accent`, for "Right over a combining accent"
  - `delete_before_a_skin_tone`, for "Delete before a skin tone"
  - `up_into_an_emoji_with_joiners`, for "Up into an emoji with joiners"
  - `cursor_stays_at_a_grapheme_boundary`, a property test over each sequence of moves and deletes
  Make sure that the six tests pass, and that each of the first five fails with the patch entry of task 3.4 removed.
- [x] 4.5 Make `Model::input` return the lines of the editor joined with LF, as D3 says. Make `Model::set_input` split its text at each LF into the lines of the editor, so a refused text goes back as its lines. Add these tests:
  - `pose_of_two_lines`, in `crates/mu-cli/tests/client.rs`, for "A pose of two lines"
  - `line_break_that_the_world_refuses`, in `crates/mu-cli/tests/client.rs` on a scripted game, for "A line break that the world refuses". Its game file extends no built-in type, because `ConfigDir::tcp_world` extends `tinymux`, which sets a `line_break`.
  Make sure that both tests pass, and that `enter_sends_to_world_in_focus`, `empty_enter_sends_empty_line`, `enter_with_no_world` and `refused_line_keeps_input` still pass.

## 5. Keys, paste and the terminal cursor

Load the `rust-best-practices` skill before this group.

- [x] 5.1 Read the current documentation of the kitty keyboard protocol. Find the bytes of Shift+Enter, Alt+Enter, Ctrl+Enter, Ctrl+J, Ctrl+M and Ctrl+Shift+Z with only the flag that disambiguates escape codes. Add a row to the table "Checked facts". Make sure that D5 and D8 match the documentation, or change them and the `terminal-client` delta to match it.
- [x] 5.2 Add the actions `Newline`, `Undo`, `Redo`, `DeleteToLineStart` and `Ignore`, and the entries of D5 to the default keymap. Add a unit test of `update` for each entry that checks the text of the editor and the list of effects. Add these tests to `crates/mu-tui/tests/editor.rs`:
  - `each_key_inserts_a_line_break`, for "Each key of a line break"
  - `ctrl_u_deletes_to_the_start_of_the_line`, for "Ctrl+U deletes to the start of the line"
  - `ctrl_u_at_the_start_of_a_line`, for "Ctrl+U at the start of a line"
  - `undo_and_redo`, for "Undo and redo"
  - `ctrl_r_changes_nothing`, for "Ctrl+R changes nothing"
  Add `ctrl_m_sends` to `crates/mu-tui/tests/app.rs`, for "Ctrl+M sends". Make sure that the six tests pass. Make sure that `each_key_inserts_a_line_break` fails for Shift+Enter with its keymap entry removed, and that `ctrl_r_changes_nothing` fails with the entry of Ctrl+R removed.
- [x] 5.3 Add the clean of D6 as `clean_paste` in `crates/mu-tui/src/text.rs`. Add the handling of `Event::Paste` in `update`. Write the sample pose of the spike once with `cargo run -p input-editor -- sample 4096` in `spikes/`, and commit it as `crates/mu-tui/tests/data/pose-4096.txt`. Add these tests to `crates/mu-tui/tests/editor.rs`:
  - `paste_of_several_lines`, for "A paste of several lines"
  - `undo_of_a_paste`, for "Undo of a paste", with `pose-4096.txt`
  - `control_characters_in_a_paste`, for the text of the editor in "Control characters in a paste"
  - `paste_of_64_kib`, a paste of 64 KiB that the editor holds whole, with one undo back to empty
  - `paste_over_a_selection`, for "A paste with a selection"
  - `clean_paste_holds_no_control_character`, a property test with the `proptest` of the repository, as in `crates/mu-tui/tests/app.rs`. For each string, the clean holds no control character but tab and LF, and a second clean changes nothing.
  Make sure that the six tests pass.
- [x] 5.4 Turn on bracketed paste in `screen::enter`, and turn it off in `screen::leave` and in the panic path. The client then restores the terminal at exit and on a panic, with each mode that it set. Add these tests to `crates/mu-cli/tests/client.rs`:
  - `bracketed_paste_at_the_start_and_at_exit`, for "Bracketed paste at the start and at exit"
  - `paste_writes_no_escape`, for the terminal output in "Control characters in a paste"
  Make sure that both tests pass. Make sure that these tests of the requirement "The client restores the terminal" still pass: `modes_at_exit`, for "The modes at exit", and `mode_2027_already_on`, for "Mode 2027 was already on".
- [x] 5.5 Set the terminal cursor in `view` and the default cursor style, with the clamp to the last column, as D7 says. Add these tests to `crates/mu-cli/tests/client.rs`:
  - `cursor_after_typing`, for "The cursor after typing"
  - `cursor_on_a_wrapped_row_after_a_wide_grapheme`, for "The cursor on a wrapped row after a wide grapheme"
  - `editor_cursor_with_probed_widths`, for the terminal cursor in "A terminal that adds code point widths"
  - `cursor_at_a_full_row`, for "The cursor after a full row"
  Make sure that the four tests pass. Add a row to the table "Checked facts" for the column that `screen_cursor` gives after a full row.

- [x] 5.6 Add the action `SubmitLines` and the queue of D13, with the entries of Alt+Enter and Ctrl+Enter. Add these tests to `crates/mu-tui/tests/app.rs`, for the requirement "Alt+Enter sends each line as a command":
  - `each_line_is_a_command`, for "Each line is a command", once for each of the two keys, with an empty line among the lines
  - `focus_moves_between_two_lines`, for "The focus moves between two lines"
  - `send_goes_after_the_lines_that_wait`, for "A send while lines wait", of the requirement "The sends to a world go in order"
  - `sends_to_a_world_go_one_at_a_time`, for "Two sends of Enter"
  - `lines_go_after_a_send_with_the_same_text`, for "A send of each line while a send waits"
  - `send_to_another_world_does_not_wait`, for "A send to another world"
  - `refused_send_stops_the_sends_after_it`, for "A refused send of Enter with a send after it"
  - `refused_last_line_goes_before_the_typed_text`, for "The last line is refused"
  - `hint_does_not_come_back_after_a_reset`, a unit test of the notice
  - `refused_line_stops_the_rest`, for "A line that the server refuses", of the requirement "A refused text stops the sends after it"
  Add `alt_enter_sends_each_line` to `crates/mu-cli/tests/client.rs`, for the screen of "Each line is a command" on a fake world. Make sure that the four tests pass, and that `each_line_is_a_command` fails with the entry of Alt+Enter removed.
- [x] 5.7 Keep the `line_break` of each world in the model, from `ListWorlds`. Add the action `JoinLines` of D12, with the entry of Alt+R. Add these tests to `crates/mu-tui/tests/app.rs`, for the requirement "Alt+R joins the lines in place":
  - `lines_become_one_line`, for "The lines become one line"
  - `join_with_no_line_break`, for "A world with no line break"
  Make sure that both tests pass.

## 6. The kitty keyboard protocol

Load the `rust-best-practices` skill before this group.

- [x] 6.1 Add the query `CSI ?u` before DA1 in `probe::queries`. Add the variant `Answer::Keyboard` and the field `keyboard` of `Outcome`, as D8 says. Add unit tests of the probe in `crates/mu-tui/tests/probe.rs` for an answer before DA1, no answer, and an answer of `CSI ?15u`. Add unit tests of the restore in `crates/mu-tui/tests/screen.rs`. Make sure that the tests pass.
- [x] 6.2 Replace the `AtomicBool` of mode 2027 with the shared `Restore` value of D8. Push the flag in `run`, and pop it with `Keyboard::PopFlags(1)` in `screen::leave` and in the panic path. Teach the scripted probe answers of the test terminal the kitty keyboard answer. Add these tests to `crates/mu-cli/tests/client.rs`:
  - `terminal_with_kitty_keyboard`, for "A terminal with the protocol"
  - `terminal_without_kitty_keyboard`, for "A terminal without the protocol"
  - `shift_enter_with_kitty_keyboard`, for "Shift+Enter with the protocol"
  - `late_kitty_keyboard_answer`, for "A late answer of the kitty keyboard query"
  Make sure that the four tests pass, and that `late_answer`, `ctrl_c_quits_client_only`, `modes_at_exit` and `mode_2027_already_on` still pass with the flag pushed.

## 7. Docs and live checks

- [x] 7.1 Update `docs/src/content/docs/decisions/input-editor.md`. Add the three patches with the links of their upstream pull requests, and the fork and its pin. Add the kitty flag that the client pushes, and the items under "Not tested" that this change tested. Add the keys of the editor, with the line that Shift+Enter sends each line in Terminal.app. Add a line on `reset` after a SIGKILL. Align the timings of its table with `spikes/input-editor/results/bench.tsv`. Make sure that `mise run docs:check` passes.
- [x] 7.2 Run `mu client` in a Rex block through the `rex` CLI, on a server with a fake world. Type a pose of two paragraphs with Shift+Enter, paste the sample pose, and send each one. Press Alt+Enter and Ctrl+Enter on a text of two lines, and record whether each one sends each line. Check by eye that an input method opens its box at the cursor on a row after `👨‍👩‍👧`. Make sure that the game answers each pose as one line, and that the PR body records the result of each step.
- [x] 7.3 Open one Parked issue for Ghostty without herdr. Open one for a paste of non-ASCII text in Terminal.app with a UTF-8 clipboard. Make sure that both issues are in the GitHub Project with the status Parked.
- [x] 7.4 Run the checks of the whole workspace. Make sure that `mise run check` passes.

## Workflow follow-up

- After this change merges, and with the owner's yes, open one pull request to `ratatui/ratatui-textarea` for each of the three branches. The owner chose on 2026-11-21 to wait for the merge. Link issue 38 upstream from the pull request of patch 3, and add each link to the input-editor decision page.
- Run `/sb:openspec-review pose-editor implementation` before group 7, and treat a verdict of changes-requested as a blocker.
- Run `/sb:suite-review` on the branch before the pull request.
- Archive the change after the merge, with an implementation review whose verdict is not changes-requested.
