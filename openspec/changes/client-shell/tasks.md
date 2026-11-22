# Tasks: client-shell

Test files: `crates/mu-tui/tests/app.rs` holds the tests of `update` and `view` on `TestBackend`. `crates/mu-tui/tests/update.rs` and `keymap.rs` hold unit tests. `crates/mu-cli/tests/client.rs` holds the tests of the `mu` binary in a terminal. `crates/mu-cli/tests/world_config.rs` holds the tests of the configuration directory. The one-test command is in `openspec/config.yaml`.

## 1. Facts to check first

- [ ] 1.1 Read the code of `mu_core::world::parse_toml` and the current documentation of `toml` for the line and column of a parse error. Make sure that the stderr text of the scenarios "An unknown key" and "Not TOML" can name the key and the line.
- [ ] 1.2 Search crates.io and GitHub for a parser of key names such as `ctrl+shift+z`. It must fit Termina's `KeyEvent` or a neutral type, and the license policy of `deny.toml`. Record the result in design.md D3. Make sure that D3 names the crate, or names why each candidate does not fit.
- [ ] 1.3 Read the Termina documentation and the kitty keyboard protocol for how Alt+digit, Alt+Home and Alt+End arrive. Make sure that design.md lists each terminal (Ghostty, Terminal.app) where they do not arrive.

## 2. The registry and one dispatch

- [ ] 2.1 Load the `rust-best-practices` skill. Add `registry.rs` with `Entry` and the table of D1. Make `Action` hold data and drop `Copy`. Make the keymap bind chords to entries, and route every key through `act`. Replace `Msg::Action` with `Msg::Run` (D1). Make sure that the unit tests `entry_names_are_unique_and_kebab` and `default_keymap_binds_entries_with_no_required_argument` pass.
- [ ] 2.2 Make sure that the existing tests of the unchanged Enter scenarios still pass in `app.rs`: `a_line_goes_to_the_world_in_focus`, `an_empty_line`, `enter_with_no_world`, `a_refused_line`, `ctrl_m_sends`, `a_pose_of_two_lines` and `a_line_break_that_the_world_refuses`. Rename a test to these names where its name differs. Make sure that every test of `crates/mu-tui` passes.
- [ ] 2.3 Add `focus-1` to `focus-9`, `scroll-top` and `scroll-bottom` with their default keys. Make sure that the tests `the_second_world`, `no_world_at_that_place` (Alt+3 with two worlds), `back_to_the_first_world` and `top_and_bottom` in `app.rs` pass.
- [ ] 2.4 Add `ArgKind`, `candidates` and the parse that knows the place of the cursor (D10). Leave Tab, Shift+Tab and Ctrl+Space unbound. Make sure that `world_candidates_follow_the_top_row` and `default_keymap_leaves_tab_free` pass.
- [ ] 2.5 Update `architecture/model.c4` and `views.c4` with the part `registry` (D9). Make sure that `mise run docs:check` passes.

## 3. Client commands

- [ ] 3.1 Parse a command at a send with the checks of D4, and give a bad command a hint. Make sure that these tests in `app.rs` pass: `a_command_does_not_reach_the_game`, `several_lines_are_not_a_command`, `an_escaped_slash`, `an_unknown_command`, `a_missing_world`, `an_unknown_world`, `an_argument_to_an_entry_that_takes_none`, `an_editing_entry_is_not_a_command`, `connect_with_no_world_in_focus`, `a_command_with_no_world_in_focus`, `focus_by_name`, `a_command_enters_the_history_of_the_world_before_it`. Make sure that the parser tests in `update.rs` for `/connect `, `/`, `/ focus`, `/quit now` and `/ignore` pass, and the property test `trailing_spaces_are_no_argument`.
- [ ] 3.2 Add `Effect::Connect`, `Effect::Disconnect` and `Msg::Control`, and run them in `run.rs` with the 30 s limit. Resolve an omitted world in `act` (D4). Make sure that `connect_the_world_in_focus`, `a_trailing_space_is_no_argument`, `disconnect_a_named_world`, `disconnect_an_unknown_world` and `a_key_bound_to_connect_with_no_world_in_focus` in `client.rs` pass.
- [ ] 3.3 Add `/help`, `/quit` and the quit arm of D4. Make sure that `help_lists_the_entries` in `app.rs`, and `quit_by_command`, `quit_twice_while_a_text_waits` and `another_command_cancels_the_quit` in `client.rs`, pass. The last test also sends a game text and `/focus-next` between two `/quit` commands, and asserts that the client still runs.
- [ ] 3.4 Make sure that `a_key_and_a_command_give_the_same_focus` in `client.rs` passes, for both Ctrl+N and `/focus-next`.

## 4. Input history

- [ ] 4.1 Add `History` to each world and the walk state to the model (D5). Feed it from each send and each command. Make sure that these tests in `app.rs` pass: `a_send_enters_the_history`, `a_pose_is_one_entry`, `no_empty_entry`, `each_world_has_its_own_history`, `no_repeat`, `an_escaped_slash_in_the_history`, and the property tests `history_walk_returns_the_draft` and `history_keeps_its_invariants`. The second property test runs random sends, refusals and focus changes, and checks in each history at most 1000 entries, no empty entry and no entry equal to the one before it.
- [ ] 4.2 Bind Up and Down to the history on the first and last line. Add `set_focus` (D5), and route every focus change through it. Make sure that `back_to_the_draft`, `up_inside_a_pose_moves_the_cursor`, `a_focus_change_ends_a_walk` and `down_with_no_walk` in `app.rs` pass. `down_with_no_walk` also presses Down after a walk is back at the draft.
- [ ] 4.3 Make sure that the history keeps at most 1000 entries, with the unit test `history_drops_the_oldest_past_1000`.

## 5. Returned sends go to history

- [ ] 5.1 Change `put_back`, `return_waiting` and the refusal notice of `on_reply` (D6). Make sure that these tests in `app.rs` pass: `a_refused_line_after_more_text`, `a_line_that_the_server_refuses`, `the_last_line_is_refused`, `a_refused_send_of_enter_with_a_send_after_it`, `the_sends_stay_apart`, `the_stream_ends_while_lines_wait`, `lines_of_two_worlds_wait`, `the_server_resets_while_lines_wait`, `a_refusal_during_a_walk_of_the_history`, `a_refusal_after_the_stream_ended`. Assert the count of the refusal line in `a_line_that_the_server_refuses` (1) and `a_refused_send_of_enter_with_a_send_after_it` (2), and its absence in `a_refused_line` (0). In `a_line_that_the_server_refuses`, a second Up gives the next line and not an empty editor.
- [ ] 5.2 Make sure that the pull request body says `Fixes #151` and `Fixes #140`.

## 6. Configuration

- [ ] 6.1 Add `ClientConfig` to `mu-tui` with `serde`, and `serde` to the `mu-tui` row of `ALLOWED_CRATES` in `workspace_boundary.rs`. Make sure that `cargo nextest run -p mu-cli --test workspace_boundary --locked` passes.
- [ ] 6.2 Add `--config` to `mu client`. Read `client.toml` with `parse_toml` (D2), and stop before the screen on a bad file. Pass the directory to a server that the client starts, and bound the first list with `START_LIMIT` (D8). Make sure that these tests in `client.rs` pass: `a_file_that_sets_the_echo`, `no_file`, `an_unknown_key`, `a_bad_echo_value`, `a_file_that_cannot_be_read`, `not_toml`, `the_client_takes_the_flag`, `a_missing_directory_warns`. As root the file opens, so `a_file_that_cannot_be_read` then skips. Add `the_first_list_gives_up` beside `client_gives_up_after_five_seconds`: a listener that accepts and never answers, and the client leaves the first list within `START_LIMIT`.
- [ ] 6.3 Make the resolver of `crates/mu-core/src/world/dir.rs` take a list of top-level files (D11). Make sure that `the_file_of_the_client`, and the tests of `client.toml` in another case, unreadable and as a hard link, pass in `world_config.rs`. Make sure that `cargo nextest run -p mu-cli --test world_config --locked` passes, so every unchanged scenario of the configuration directory still holds.
- [ ] 6.4 Build the keymap from `[keys]` (D3), with `"none"` bound to `ignore`. Make sure that these tests in `client.rs` pass: `a_new_binding`, `the_default_keys_stay`, `a_removed_binding`, `a_binding_replaces_a_default`, `none_on_a_key_with_a_modifier`, `two_names_of_one_key`, `an_uppercase_key`, `an_unknown_entry`, `an_unknown_key_name`, `an_entry_that_needs_an_argument`. Make sure that the property test `key_names_read_back` and the parser tests in `keymap.rs` pass: `ctrl+plus` and `alt+1` are read, and `ctrl+N`, `ctrl+ctrl+n`, `ctrl++`, `+`, `ctrl+`, the empty name, `ctrl` alone and `shift+1` are refused with the name in the error. Make sure that `shift_and_a_letter_match_the_key` passes for `Char('A')` with and without `SHIFT`.
- [ ] 6.5 Write the guide page for `client.toml` in `docs/src/content/docs/guides/` with the simple-english rules. The page names `/help` as the command that lists the registry entries, and says that Tab, Shift+Tab and Ctrl+Space are kept. Update `guides/configuration.md` so that it names `client.toml` beside `server.toml`, with a link, and the stray-file exception names both files. Make sure that `mise run docs:check` passes.

## 7. Echo and the first list

- [ ] 7.1 Make `text::line` give the rows of a player line, and choose the style by `echo` in `update` (D7). Make sure that `the_default_dims`, `hide`, `a_hidden_echo_marks_a_world_out_of_focus` and `the_echo_of_a_pose_of_two_lines` in `app.rs` pass.
- [ ] 7.2 Open the screen before the first list, and make `Msg::Listed` the one path of every list (D8). Remove `Model::from_listing`. Make sure that `before_the_first_list`, `a_send_before_the_first_list`, `an_event_before_the_first_list`, `the_first_list_fails` and `a_reset_during_the_first_list` in `app.rs` pass, and that `the_first_view` in `client.rs` still passes.
- [ ] 7.3 Update the terminal client page in `docs/src/content/docs/architecture/terminal-client.mdx` for the registry, commands and history. Update `decisions/terminal-client-architecture.md`, so that a keymap maps a key to a registry entry, not to an `Action`. Make sure that `mise run docs:check` passes.

## 8. Review and integration

- [ ] 8.1 Run `/sb:openspec-review client-shell implementation`. Make sure that the verdict is not changes-requested.
- [ ] 8.2 Run `/sb:suite-review` on the branch, and fix or triage each blocker and major finding. Make sure that `mise run check` passes on Linux and macOS in CI.

## Workflow follow-up

- Archive the change after merge with `/sb:post-merge`.
- Propose `client-layout`: layout file, tab labels, statusline and the command palette over the registry.
