# Tasks: client-shell

Test files: `crates/mu-tui/tests/app.rs` holds the tests of `update` and `view` on `TestBackend`. `crates/mu-cli/tests/client.rs` holds the tests of the `mu` binary in a terminal. `crates/mu-cli/tests/world_config.rs` holds the tests of the configuration directory.

## 1. Facts to check first

- [ ] 1.1 Read the current documentation of `serde_path_to_error` and `toml` for the error path and the line of a parse error. Make sure that the stderr text of the scenarios "An unknown key" and "Not TOML" can name the key and the line.
- [ ] 1.2 Search crates.io and GitHub for a parser of key names such as `ctrl+shift+z`. It must fit Termina's `KeyEvent` or a neutral type, and the license policy of `deny.toml`. Record the result in design.md D3. Make sure that D3 names the crate, or names why each candidate does not fit.
- [ ] 1.3 Read the Termina documentation and the kitty keyboard protocol for how Alt+digit, Alt+Home and Alt+End arrive. Make sure that design.md lists each terminal (Ghostty, Terminal.app) where they do not arrive.

## 2. The registry and one dispatch

- [ ] 2.1 Load the `rust-best-practices` skill. Add `registry.rs` with `Entry` and the table of D1. Make `Action` hold data and drop `Copy`. Route every key through `act`. Make sure that the unit tests `entry_names_are_unique_and_kebab` and `default_keymap_uses_only_registry_entries` pass, and that every test of `crates/mu-tui` still passes.
- [ ] 2.2 Add `focus-1` to `focus-9`, `scroll-top` and `scroll-bottom` with their default keys. Make sure that the tests `the_second_world`, `no_world_at_that_place` and `top_and_bottom` in `app.rs` pass.
- [ ] 2.3 Add `ArgKind`, `candidates` and the parse that knows the place of the cursor (D10). Leave Tab, Shift+Tab and Ctrl+Space unbound. Make sure that `world_candidates_follow_the_top_row` and `default_keymap_leaves_tab_free` pass.
- [ ] 2.4 Update `architecture/model.c4` and `views.c4` with the part `registry` (D9). Make sure that `mise run docs:check` builds.

## 3. Client commands

- [ ] 3.1 Parse a command at a send (D4), with notices for a bad command. Make sure that these tests in `app.rs` pass: `a_command_does_not_reach_the_game`, `several_lines_are_not_a_command`, `an_escaped_slash`, `an_unknown_command`, `a_missing_world`, `an_unknown_world`, `an_editing_entry_is_not_a_command`, `focus_by_name`.
- [ ] 3.2 Add `Effect::Connect`, `Effect::Disconnect` and `Msg::Control`, and run them in `run.rs` with the 30 s limit. Make sure that `connect_the_world_in_focus` and `disconnect_a_named_world` in `client.rs` pass.
- [ ] 3.3 Add `/help` and `/quit`. Make sure that `help_lists_the_entries` in `app.rs` and `quit_by_command` in `client.rs` pass.
- [ ] 3.4 Make sure that `a_key_and_a_command_give_the_same_focus` in `client.rs` passes, for both Ctrl+N and `/focus-next`.

## 4. Input history

- [ ] 4.1 Add `History` to each world (D5), fed by each send and each command. Make sure that these tests in `app.rs` pass: `a_send_enters_the_history`, `a_pose_is_one_entry`, `each_world_has_its_own_history`, `no_repeat`, and the property test `history_walk_returns_the_draft`.
- [ ] 4.2 Bind Up and Down to the history on the first and last line. Make sure that `back_to_the_draft` and `up_inside_a_pose_moves_the_cursor` in `app.rs` pass.
- [ ] 4.3 Make sure that the history keeps at most 1000 entries, with the unit test `history_drops_the_oldest_past_1000`.

## 5. Returned sends go to history

- [ ] 5.1 Change `put_back` and `return_waiting` (D6). Make sure that these tests in `app.rs` pass, under their new names where the scenario changed: `a_refused_line_after_more_text`, `a_line_that_the_server_refuses`, `the_last_line_is_refused`, `a_refused_send_of_enter_with_a_send_after_it`, `the_sends_stay_apart`, `the_stream_ends_while_lines_wait`, `lines_of_two_worlds_wait`, `the_server_resets_while_lines_wait`, `a_refusal_after_the_stream_ended`.
- [ ] 5.2 Close the loop on #151 in the pull request text. Make sure that the PR body says `Fixes #151`.

## 6. Configuration

- [ ] 6.1 Add `ClientConfig` to `mu-tui` with `serde`, and `serde` to the `mu-tui` row of `ALLOWED_CRATES` in `workspace_boundary.rs`. Make sure that `cargo nextest run -p mu-cli --test workspace_boundary --locked` passes.
- [ ] 6.2 Add `--config` to `mu client`, find `client.toml` with the resolver of `config.rs`, and stop before the screen on a bad file. Make sure that these tests in `client.rs` pass: `a_file_that_sets_the_echo`, `no_file`, `an_unknown_key`, `a_bad_echo_value`, `not_toml`, `the_client_takes_the_flag`.
- [ ] 6.3 Accept `client.toml` in the world resolver. Make sure that `the_file_of_the_client` in `world_config.rs` passes, and that `a_stray_file` still passes.
- [ ] 6.4 Build the keymap from `[keys]` (D3). Make sure that these tests in `client.rs` pass: `a_new_binding`, `the_default_keys_stay`, `a_removed_binding`, `an_unknown_entry`, `an_unknown_key_name`, `an_entry_that_needs_an_argument`.
- [ ] 6.5 Write the guide page for `client.toml` in `docs/src/content/docs/guides/` with the simple-english rules. The page names `/help` as the command that lists the registry entries. Make sure that `mise run docs:check` builds.

## 7. Echo and the first list

- [ ] 7.1 Show the echo line by `echo`, split at the `line_break` (D7). Make sure that `the_default_dims`, `hide` and `a_pose_of_two_lines` (echo) in `app.rs` pass. Close #140 in the PR text.
- [ ] 7.2 Open the screen before the first list (D8). Make sure that `before_the_first_list` in `app.rs` passes, and that `the_first_view` in `client.rs` still passes.
- [ ] 7.3 Update the terminal client page in `docs/src/content/docs/architecture/terminal-client.mdx` for the registry, commands and history. Make sure that `mise run docs:check` builds.

## 8. Review and integration

- [ ] 8.1 Run `/sb:openspec-review client-shell implementation`. Make sure that the verdict is not changes-requested.
- [ ] 8.2 Run `/sb:suite-review` on the branch, and fix or triage each blocker and major finding. Make sure that `mise run check` passes on Linux and macOS in CI.

## Workflow follow-up

- Archive the change after merge with `/sb:post-merge`.
- Propose `client-layout`: layout file, tab labels, statusline and the command palette over the registry.
