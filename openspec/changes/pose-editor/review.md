---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-21T15:59:42+00:00
head: e2c754f69328bbb4acadb6c9c1a3d6aac3408c94
round: 6
next_id: 85
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: ok
  invariants: ok
  verify: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-83 | test-coverage | should-fix | in-change | delta scenario 'A send while lines wait' has no matrix row | review > absent-row — "A send while lines wait" | add the row with its covering task or test |  |  | unmet-requirement |
| R-84 | test-coverage | should-fix | in-change | delta scenario 'The last line is refused' has no matrix row | review > absent-row — "The last line is refused" | add the row with its covering task or test |  |  | unmet-requirement |
| R-65 | verify | should-fix | in-change | PARTIAL: the fix 064a081061aaf4a039f9d178db451be781a71810 of R-65 did not hold. app-1: No decision keeps one line of a send of each line in flight when a plain Enter interleaves. Reply::Sent names no send, so the Sent of the plain line releases the next queued line before the queued line in flight is answered. Each send is its own spawned task, so two queued lines can then reach the server out of order, and a refusal of the first no longer stops the second. D13 calls only the place of the plain line unfixed.; inv-1: The rule that a line of a send of each line goes only after the server answered the line before it has no enforcement point. Reply::Sent names no send, and next_line advances the queue on any Sent. Sends run in spawned tasks, so the Sent of a plain Enter during the queue sends the next queued line while the earlier one is in flight. Two queued lines are then in flight at once, and their order at the server is not fixed. queue.sent is then overwritten, so a refusal of the earlier line no longer matches take_if and the rest of the script keeps going. | crates/mu-tui/src/update.rs:568 — ".filter(\|queue\| queue.world == world && queue.sent == text)?;" | fix it again | regression |  | wrong-behaviour |
| R-66 | verify | should-fix | in-change | ver-fix-R-66: REGRESSED: the fix 064a081061aaf4a039f9d178db451be781a71810 of R-66 did not hold. No decision says what a second Alt+Enter does while a queue waits. submit_lines assigns a new LineQueue over the old one, so the lines that the first Alt+Enter had not sent yet are dropped: they never go to the game and never come back to the editor.; ver-1: The guard of a send while a queue waits writes its notice with Model::notice, which also appends the line to model.notices. That list is replayed into the pane in focus on each ClientEvent::Reset and when a first world arrives. So each blocked Enter adds one more stale 'a send of each line is not done yet' line that comes back after every reset, also when no queue waits. world_to_send_to writes its notice of the same kind with model.push to the notices pane, which keeps it out of the replayed list. | crates/mu-tui/src/update.rs:410 — "model.notice("a send of each line is not done yet, so the text stays in the editor");" | ver-fix-R-66: fix it again; ver-1: Write the notice with model.push to the pane in focus, as world_to_send_to does, so it does not go into the replayed list. Add a test that a Reset after a blocked Enter does not show the notice again. | regression |  | wrong-behaviour |
| R-70 | verify | should-fix | in-change | REGRESSED: the fix 064a081061aaf4a039f9d178db451be781a71810 of R-70 did not hold. A second Alt+Enter while lines of an earlier send of each line still wait replaces model.queue. The waiting lines of the first script are dropped with no notice and never sent, and the reply of the in-flight line of the first script then advances the new queue, so two lines are in flight. No guard keeps one queue, and no test presses Alt+Enter twice. | crates/mu-tui/src/update.rs:409 — "if model.queue.is_some() {" | fix it again | regression |  | wrong-behaviour |
| R-77 | verify | suggestion | in-change | PARTIAL: the fix 064a081061aaf4a039f9d178db451be781a71810 of R-77 did not hold. The cursor of the editor stops only at a grapheme boundary after any sequence of moves and edits. That is a rule over every text and every key sequence, and its tests are fixed examples: four graphemes in the fork, and five scenarios in editor.rs. A move kind that the fork misses, such as a word move or an undo that restores a cursor, has no test. | crates/mu-tui/tests/editor.rs:402 — "/// A key that moves the cursor or deletes." | fix it again | regression | severity? should-fix 0.65 | unmet-requirement |

## Repeats

| id | lens | severity | claim | evidence |
|---|---|---|---|---|
| R-78 | test-coverage | should-fix | covered row 'A terminal that adds code point widths' has no verification run | review > not-run — "A terminal that adds code point widths" |
| R-79 | test-coverage | should-fix | covered row 'Control characters in a paste' has no verification run | review > not-run — "Control characters in a paste" |
| R-80 | test-coverage | should-fix | covered row 'An empty line' has no verification run | review > not-run — "An empty line" |
| R-81 | test-coverage | should-fix | covered row 'Each line is a command' has no verification run | review > not-run — "Each line is a command" |
| R-82 | test-coverage | should-fix | covered row 'A world that waits to reconnect' has no verification run | review > not-run — "A world that waits to reconnect" |

## Out of bound

None.

## Accepted

None.

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 292abde7d72d251f79350d74e99dea4d0bf91e52 | abe97c09035f1efefc04ccc353d6fb5c4ccab4352eef0572c8d6db6a785b88f3 | changes-requested | 42 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d | 4f9e0c4bcafc5313cee510560979e27f731d6d9f119103fa599edae2ac4ed7c6 | approve-with-notes | 6 | 0 | 0 | 42 | 0 |  |
| 3 | plan | follow-up | 2f16248711c31afd08727fadd04a3da042d1bbb6 | 3e62f93313c15dfff40c156abbf3da989267371482afc1a787f5c65e4689afe1 | approve-with-notes | 7 | 6 | 0 | 0 | 0 |  |
| 4 | plan | follow-up | 533e1bb37337a23ee482e7bf7ed03099b4449945 | b01d86a06b9f44dbb8c045703ada6e0155c16d47d90e87bfacaaa6a40470a3d3 | approve-with-notes | 7 | 12 | 0 | 1 | 0 |  |
| 5 | implementation | full | 1e283cd9eed30835f5cdca9cd37f29a4b1f51c9a | 43802f79263b59f8367004c232f37b37886658fa20c9a37b1d6f521985416bbf | changes-requested | 20 | 0 | 0 | 0 | 0 |  |
| 6 | implementation | follow-up | e2c754f69328bbb4acadb6c9c1a3d6aac3408c94 | 7e9fc30b47a3102a544d760591c74b0d34d85a08717ff6deb514e06bcc4ad610 | approve-with-notes | 2 | 5 | 4 | 12 | 0 |  |

## Judge

judge: 8 questions, 0.000 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A line wider than the editor | crates/mu-tui/tests/editor.rs: line_wider_than_the_editor | covered | pass |
| A word wider than the editor | crates/mu-tui/tests/editor.rs: word_wider_than_the_editor | covered | pass |
| Three lines | crates/mu-tui/tests/editor.rs: three_lines | covered | pass |
| More rows than the limit | crates/mu-tui/tests/editor.rs: more_rows_than_the_limit | covered | pass |
| A short terminal | crates/mu-tui/tests/editor.rs: short_terminal | covered | pass |
| A terminal of 17 rows | crates/mu-tui/tests/editor.rs: terminal_of_17_rows | covered | pass |
| A terminal of 3 rows | crates/mu-tui/tests/editor.rs: terminal_of_3_rows | covered | pass |
| A terminal of 1 row | crates/mu-tui/tests/editor.rs: terminal_of_1_row | partial | pass |
| Back to one row | crates/mu-tui/tests/editor.rs: back_to_one_row | covered | pass |
| Backspace after an emoji with joiners | crates/mu-tui/tests/editor.rs: backspace_after_an_emoji_with_joiners | covered | pass |
| Left over a flag | crates/mu-tui/tests/editor.rs: left_over_a_flag | covered | pass |
| Right over a combining accent | crates/mu-tui/tests/editor.rs: right_over_a_combining_accent | covered | pass |
| Delete before a skin tone | crates/mu-tui/tests/editor.rs: delete_before_a_skin_tone | covered | pass |
| Up into an emoji with joiners | crates/mu-tui/tests/editor.rs: up_into_an_emoji_with_joiners | covered | pass |
| A terminal that adds code point widths | crates/mu-tui/tests/editor.rs: editor_uses_the_probed_widths; crates/mu-cli/tests/client.rs: editor_cursor_with_probed_widths | covered | not-run |
| A terminal that does not answer | crates/mu-tui/tests/editor.rs: editor_uses_unicode_width | covered | pass |
| Each key of a line break | crates/mu-tui/tests/editor.rs: each_key_inserts_a_line_break | covered | pass |
| Ctrl+U deletes to the start of the line | crates/mu-tui/tests/editor.rs: ctrl_u_deletes_to_the_start_of_the_line | covered | pass |
| Ctrl+U at the start of a line | crates/mu-tui/tests/editor.rs: ctrl_u_at_the_start_of_a_line | covered | pass |
| Undo and redo | crates/mu-tui/tests/editor.rs: undo_and_redo | covered | pass |
| Ctrl+R changes nothing | crates/mu-tui/tests/editor.rs: ctrl_r_changes_nothing | covered | pass |
| A paste of several lines | crates/mu-tui/tests/editor.rs: paste_of_several_lines | covered | pass |
| Undo of a paste | crates/mu-tui/tests/editor.rs: undo_of_a_paste | covered | pass |
| Control characters in a paste | crates/mu-tui/tests/editor.rs: control_characters_in_a_paste; crates/mu-cli/tests/client.rs: paste_writes_no_escape | covered | not-run |
| A paste with a selection | crates/mu-tui/tests/editor.rs: paste_over_a_selection | covered | pass |
| The cursor after typing | crates/mu-cli/tests/client.rs: cursor_after_typing | covered | pass |
| The cursor on a wrapped row after a wide grapheme | crates/mu-cli/tests/client.rs: cursor_on_a_wrapped_row_after_a_wide_grapheme | covered | pass |
| The cursor after a full row | crates/mu-cli/tests/client.rs: cursor_at_a_full_row | covered | pass |
| The first view | crates/mu-tui/tests/app.rs: first_view | covered | pass |
| The order of the worlds | crates/mu-tui/tests/app.rs: world_order | covered | pass |
| No worlds | crates/mu-tui/tests/app.rs: no_worlds | covered | pass |
| A line goes to the world in focus | crates/mu-tui/tests/app.rs: enter_sends_to_world_in_focus | covered | pass |
| An empty line | crates/mu-tui/tests/app.rs: empty_enter_sends_empty_line; crates/mu-core/tests/portal.rs: empty_send_reaches_game | covered | not-run |
| Enter with no world | crates/mu-tui/tests/app.rs: enter_with_no_world | covered | pass |
| A refused line | crates/mu-tui/tests/app.rs: refused_line_keeps_input | covered | pass |
| Ctrl+M sends | crates/mu-tui/tests/app.rs: ctrl_m_sends | covered | pass |
| A pose of two lines | crates/mu-cli/tests/client.rs: pose_of_two_lines | covered | pass |
| A line break that the world refuses | crates/mu-cli/tests/client.rs: line_break_that_the_world_refuses | covered | pass |
| The modes at exit | crates/mu-cli/tests/client.rs: modes_at_exit | covered | pass |
| Mode 2027 was already on | crates/mu-cli/tests/client.rs: mode_2027_already_on | covered | pass |
| Bracketed paste at the start and at exit | crates/mu-cli/tests/client.rs: bracketed_paste_at_the_start_and_at_exit | covered | pass |
| Each line is a command | crates/mu-tui/tests/app.rs: each_line_is_a_command; crates/mu-cli/tests/client.rs: alt_enter_sends_each_line | covered | not-run |
| The focus moves between two lines | crates/mu-tui/tests/app.rs: focus_moves_between_two_lines | covered | pass |
| A line that the server refuses | crates/mu-tui/tests/app.rs: refused_line_stops_the_rest | covered | pass |
| The lines become one line | crates/mu-tui/tests/app.rs: lines_become_one_line | covered | pass |
| A world with no line break | crates/mu-tui/tests/app.rs: join_with_no_line_break | covered | pass |
| A terminal with the protocol | crates/mu-cli/tests/client.rs: terminal_with_kitty_keyboard | covered | pass |
| A terminal without the protocol | crates/mu-cli/tests/client.rs: terminal_without_kitty_keyboard | covered | pass |
| Shift+Enter with the protocol | crates/mu-cli/tests/client.rs: shift_enter_with_kitty_keyboard | covered | pass |
| A late answer of the kitty keyboard query | crates/mu-cli/tests/client.rs: late_kitty_keyboard_answer | covered | pass |
| Each built-in type reads as quirks | crates/mu-core/tests/world_config.rs: built_in_types_hold_only_quirks | covered | pass |
| A global preference is not hidden by a type | crates/mu-cli/tests/world_config.rs: global_preference_is_not_hidden_by_a_type | covered | pass |
| A user type with a built-in name | crates/mu-cli/tests/world_config.rs: user_type_with_a_built_in_name | covered | pass |
| The line break of each built-in type | crates/mu-cli/tests/world_config.rs: line_break_of_each_built_in_type | covered | pass |
| A built-in type that extends another | crates/mu-cli/tests/world_config.rs: built_in_type_that_extends_another | covered | pass |
| A built-in type extends only a built-in type | crates/mu-core/tests/world_config.rs: built_in_type_extends_only_a_built_in_type | covered | pass |
| A game overrides the line break of its type | crates/mu-cli/tests/world_config.rs: game_overrides_the_line_break_of_its_type | covered | pass |
| An unknown field | crates/mu-cli/tests/world_config.rs: unknown_field | covered | pass |
| A port out of range | crates/mu-cli/tests/world_config.rs: port_out_of_range | covered | pass |
| A template with no password | crates/mu-cli/tests/world_config.rs: template_with_no_password | covered | pass |
| The case of a character set | crates/mu-cli/tests/world_config.rs: charset_case | covered | pass |
| A character set that the engine cannot read | crates/mu-cli/tests/world_config.rs: charset_the_engine_cannot_read | covered | pass |
| An unknown mode | crates/mu-cli/tests/world_config.rs: unknown_mode | covered | pass |
| The edges of each range | crates/mu-cli/tests/world_config.rs: edges_of_each_range | covered | pass |
| The edges of the attempts | crates/mu-cli/tests/world_config.rs: edges_of_the_attempts | covered | pass |
| A line break in a line field | crates/mu-cli/tests/world_config.rs: line_break_in_a_line_field | covered | pass |
| The edges of the line break | crates/mu-cli/tests/world_config.rs: edges_of_the_line_break | covered | pass |
| The fake world answers | crates/mu-cli/tests/worlds.rs: fake_world_answers | covered | pass |
| The fake world answers a mark in order | crates/mu-fake-server/src/memory.rs: fake_game_answers_mark_in_order | covered | pass |
| The flag is hidden | crates/mu-cli/tests/worlds.rs: fake_world_flag_is_hidden | covered | pass |
| A missing password variable | crates/mu-cli/tests/worlds.rs: missing_password_variable | covered | pass |
| A fake world does not reconnect | crates/mu-cli/tests/worlds.rs: fake_world_does_not_reconnect | covered | pass |
| The fake world gets a joined line | crates/mu-cli/tests/worlds.rs: fake_world_gets_a_joined_line | covered | pass |
| A world that waits to reconnect | crates/mu-cli/tests/worlds.rs: list_worlds_shows_reconnecting; crates/mu-core/tests/connection.rs: send_to_reconnecting_world | covered | not-run |
| The line break of a world | crates/mu-cli/tests/worlds.rs: list_worlds_shows_the_line_break | covered | pass |
| Two characters on one game | crates/mu-core/tests/portal.rs: two_characters_on_one_game | covered | pass |
| An unknown world | crates/mu-core/tests/portal.rs: send_to_unknown_world | covered | pass |
| A world that is not connected | crates/mu-core/tests/portal.rs: send_to_disconnected_world | covered | pass |
| A write that fails | crates/mu-core/tests/portal.rs: failed_write_records_disconnect | covered | pass |
| A world that is still connecting | crates/mu-core/tests/portal.rs: send_to_connecting_world | covered | pass |
| A carriage return in the text | crates/mu-core/tests/portal.rs: send_with_cr_is_refused | covered | pass |
| A line break in the text | crates/mu-core/tests/portal.rs: send_with_lf_is_refused | covered | pass |
| A line break that the world joins | crates/mu-core/tests/portal.rs: line_break_that_the_world_joins | covered | pass |
| A carriage return to a world that joins | crates/mu-core/tests/portal.rs: carriage_return_to_a_world_that_joins | covered | pass |
| A hook sees the joined line | crates/mu-core/tests/scripts.rs: hook_sees_the_joined_line | covered | pass |

### Not run

- A terminal that adds code point widths: no verification run names this row
- Control characters in a paste: no verification run names this row
- An empty line: no verification run names this row
- Each line is a command: no verification run names this row
- A world that waits to reconnect: no verification run names this row

## Verification runs

- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=line_wider_than_the_editor)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=word_wider_than_the_editor)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=three_lines)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=more_rows_than_the_limit)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=short_terminal)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=terminal_of_17_rows)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=terminal_of_3_rows)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=terminal_of_1_row)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=back_to_one_row)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=backspace_after_an_emoji_with_joiners)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=left_over_a_flag)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=right_over_a_combining_accent)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=delete_before_a_skin_tone)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=up_into_an_emoji_with_joiners)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=editor_uses_the_probed_widths)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=editor_cursor_with_probed_widths)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=editor_uses_unicode_width)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=each_key_inserts_a_line_break)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=ctrl_u_deletes_to_the_start_of_the_line)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=ctrl_u_at_the_start_of_a_line)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=undo_and_redo)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=ctrl_r_changes_nothing)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=paste_of_several_lines)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=undo_of_a_paste)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=control_characters_in_a_paste)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=paste_writes_no_escape)'`
- exit 0: `cargo nextest run -p mu-tui --test editor --locked -E 'test(=paste_over_a_selection)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=cursor_after_typing)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=cursor_on_a_wrapped_row_after_a_wide_grapheme)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=cursor_at_a_full_row)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=first_view)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=world_order)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=no_worlds)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=enter_sends_to_world_in_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=empty_enter_sends_empty_line)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=empty_send_reaches_game)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=enter_with_no_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=refused_line_keeps_input)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=ctrl_m_sends)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=pose_of_two_lines)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=line_break_that_the_world_refuses)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=modes_at_exit)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=mode_2027_already_on)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=bracketed_paste_at_the_start_and_at_exit)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=each_line_is_a_command)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=alt_enter_sends_each_line)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=focus_moves_between_two_lines)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=refused_line_stops_the_rest)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=lines_become_one_line)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=join_with_no_line_break)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=terminal_with_kitty_keyboard)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=terminal_without_kitty_keyboard)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=shift_enter_with_kitty_keyboard)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=late_kitty_keyboard_answer)'`
- exit 0: `cargo nextest run -p mu-core --test world_config --locked -E 'test(=built_in_types_hold_only_quirks)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=global_preference_is_not_hidden_by_a_type)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=user_type_with_a_built_in_name)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=line_break_of_each_built_in_type)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=built_in_type_that_extends_another)'`
- exit 0: `cargo nextest run -p mu-core --test world_config --locked -E 'test(=built_in_type_extends_only_a_built_in_type)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=game_overrides_the_line_break_of_its_type)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=unknown_field)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=port_out_of_range)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=template_with_no_password)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=charset_case)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=charset_the_engine_cannot_read)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=unknown_mode)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=edges_of_each_range)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=edges_of_the_attempts)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=line_break_in_a_line_field)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=edges_of_the_line_break)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=fake_world_answers)'`
- exit 0: `cargo nextest run -p mu-fake-server --lib --locked -E 'test(/(^|::)fake_game_answers_mark_in_order$/)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=fake_world_flag_is_hidden)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=missing_password_variable)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=fake_world_does_not_reconnect)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=fake_world_gets_a_joined_line)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=list_worlds_shows_reconnecting)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=send_to_reconnecting_world)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=list_worlds_shows_the_line_break)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=two_characters_on_one_game)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=send_to_unknown_world)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=send_to_disconnected_world)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=failed_write_records_disconnect)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=send_to_connecting_world)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=send_with_cr_is_refused)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=send_with_lf_is_refused)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=line_break_that_the_world_joins)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=carriage_return_to_a_world_that_joins)'`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=hook_sees_the_joined_line)'`

## Dropped

None.

## Lens notes

- verify: R-71: the new assertion of terminal_of_1_row checks the editor area, as the THEN asks. In a frame of 1 row no layout can give the editor more than 1 row, so only a panic (such as clamp(1, 0) with the floor removed) fails it. That follows from the scenario text, not from the test.
- verify: R-65 residue: next_line and the refusal arm match a reply to the queue by world and text. A plain send issued before the Alt+Enter with the same text as the first queued line (for example an empty line, or a repeated command) still moves the queue early, so two lines can be in flight and a later refusal takes the plain arm.
- verify: R-77 residue: the property test cursor_stays_at_a_grapheme_boundary draws its keys from Left, Right, Up, Down, Home, End, Backspace and Delete. Word moves (Alt+B, Alt+F, Ctrl+W), undo and redo, and a mouse jump, which the claim names, are still not in it.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/input-editor/spec.md","Requirement: The editor wraps a long line"],"It SHALL break inside a word only where the word is wider than the area."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  | severity? should-fix 0.69 | No decision names the wrap mode of the editor. Task 4.2 asks for 'a word wrap', and WrapMode::Word of ratatui-textarea 0.9.4 never splits a word wider than the area, so 'A word wider than the editor' fails. Only WrapMode::WordOrGlyph falls back to a split inside a long word. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: Enter sends the input to the world in focus"],"the editor still holds `one` and `two` on 2 rows"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | In 'A line break that the world refuses', the editor must still hold `one` and `two` on 2 rows. A refused text goes back through Model::set_input, which builds a TextArea of one line from the whole text, so the LF stays inside one line. D3 changes only Model::input, and no decision or task splits the refused text at LF. |
| R-3 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/input-editor/spec.md","Requirement: Keys insert a line break"],"Shift+Enter, Alt+Enter and Ctrl+J SHALL insert a line break at the cursor."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  | severity? should-fix 0.73 | The requirement makes Shift+Enter insert a line break with no condition on the terminal. D5 says that without the kitty flag most terminals send Shift+Enter as CR, so the client sends the text. So no decision produces the requirement on a terminal that does not support the protocol. |
| R-4 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","The fake world stands in for a game"],"#### Scenario: The fake world answers a mark in order"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'The fake world answers a mark in order'. |
| R-5 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Built-in types set only quirks"],"#### Scenario: A global preference is not hidden by a type"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'A global preference is not hidden by a type'. Task 2.2 changes the resolver and Quirks. |
| R-6 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Each field is known"],"#### Scenario: An unknown field"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'An unknown field'. Task 2.2 edits check_layer. |
| R-7 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Each field is known"],"#### Scenario: A template with no password"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'A template with no password'. |
| R-8 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Each field is known"],"#### Scenario: The case of a character set"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'The case of a character set'. |
| R-9 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Each field is known"],"#### Scenario: A character set that the engine cannot read"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'A character set that the engine cannot read'. |
| R-10 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Each field is known"],"#### Scenario: An unknown mode"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'An unknown mode'. |
| R-11 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Each value is in its range"],"#### Scenario: The edges of the attempts"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'The edges of the attempts'. Task 2.2 edits the range check beside it. |
| R-12 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","The fake world stands in for a game"],"#### Scenario: The flag is hidden"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'The flag is hidden'. |
| R-13 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","The fake world stands in for a game"],"#### Scenario: A missing password variable"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  | severity? should-fix 0.60 | No task names a test for 'A missing password variable'. |
| R-14 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","A command reaches only its world"],"#### Scenario: Two characters on one game"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'Two characters on one game'. |
| R-15 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","A command reaches only its world"],"#### Scenario: An unknown world"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'An unknown world'. |
| R-16 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","A command reaches only its world"],"#### Scenario: A world that is not connected"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'A world that is not connected'. Task 2.3 rewrites the Send order. |
| R-17 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","A command reaches only its world"],"#### Scenario: A write that fails"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | No task names a test for 'A write that fails'. Task 2.3 reorders the Send path it exercises. |
| R-18 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","A command reaches only its world"],"#### Scenario: A world that is still connecting"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  | severity? should-fix 0.68 | No task names a test for 'A world that is still connecting'. |
| R-19 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","A command reaches only its world"],"#### Scenario: A world that waits to reconnect"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  | severity? should-fix 0.60 | No task names a test for 'A world that waits to reconnect'. |
| R-20 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D9. The server joins the lines of a sent text"],"The test `send_with_lf_is_refused` moves to a world with no `line_break`, in `crates/mu-core/tests/portal.rs`."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | D9 and task 2.3 say that send_with_lf_is_refused moves from crates/mu-cli/tests/worlds.rs into crates/mu-core/tests/portal.rs. At head, portal.rs already holds a test of that name on a world with no line_break. The mu-cli copy runs on a fake world, which gets `%r`, so it fails after the change unless it is removed. The plan's step for the scenario 'A line break in the text' does not describe the work. |
| R-21 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","2. The line break of a world"],"Make sure that `port_out_of_range` and the test of \\"Each built-in type reads as quirks\\" pass with no edit."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | Task 2.2 says the test of 'Each built-in type reads as quirks' passes with no edit. built_in_types_hold_only_quirks clears only charset, keepalive and login.template, and then asserts that the rest is default. With line_break set in each built-in type, it fails until it also clears line_break. |
| R-22 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","Migration Plan"],"To roll back, revert the change. The fork can stay on GitHub with no effect."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  | scope? deviates-design 0.66; severity? blocker 0.62 | The rollback is 'revert the change'. Each layer file refuses an unknown field, so after a revert, a configuration directory that sets line_break fails to load, and the server refuses it. The plan has no step for deployed configuration. |
| R-23 | 1 | 2 | should-fix | deviates-design | drift | ["artifact",["design.md","D1. A fork of ratatui-textarea with three patches"],"With a function set, the render sets each cell itself, as the output pane does."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  | scope? in-change 0.81 | Patch 2 copies the per-cell draw of the output pane into the fork. The pane draw also forces the width of a risky class and draws U+FFFD for a grapheme wider than the row, and a width function alone cannot carry those rules, so the two draws can differ. |
| R-24 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","D1. A fork of ratatui-textarea with three patches"],"The cursor position, the wrap in `wrap.rs` and the render all call it."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | The client already has a word wrap with the width profile in wrap.rs, with the same rule that the input-editor spec states. The design wraps the editor with the wrap of the library and does not name wrap.rs, so two wraps of one rule can change apart. |
| R-25 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D9. The server joins the lines of a sent text"],"`check_layer` refuses an empty value, a value of more than 16 characters, and a value with a control character."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  | severity? blocker 0.61 | fit-3: Each checked field of ResolvedWorld is a type whose constructor runs the same check as check_layer: Login, Keepalive and Reconnect. D9 makes line_break a plain string checked only in check_layer, and the fake world and tests build ResolvedWorld directly, so a line_break with CR or LF can reach the session unchecked.; inv-3: The range of line_break is checked only in check_layer. ResolvedWorld is built by struct literal, as the fake world does, so an empty value or one with LF skips the check. Login and keepalive text get the same check in their constructors. |
| R-26 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D1. A fork of ratatui-textarea with three patches"],"`CursorMove::Forward` and `Back`, `delete_char` and `delete_next_char` step over the grapheme boundaries of `unicode-segmentation`."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | The cursor sits on a grapheme boundary only after Left, Right and the deletes. Up and Down map a cell column back through single chars, so the cursor can land inside an emoji with joiners. No task patches or tests that. |
| R-27 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. Bracketed paste"],"`update` cleans the text and calls `insert_str` once, so the paste is one edit and one undo step."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | A paste is one undo step only with no selection. Shift+arrow keys reach TextArea::input and select. insert_str then deletes the selection as its own history entry, so one Ctrl+Z leaves the selected text deleted. No test pastes over a selection. |
| R-28 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/world-config/spec.md","Scenario: The edges of the line break"],"an empty text, `%r`, 16 letters `x`, 17 letters `x`, and `%` with a tab"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | The edges of line_break use ASCII letters and a tab only. No value holds LF or CR, the values that would split a joined line. No non-ASCII value tells a count of characters from a count of bytes at 16. |
| R-29 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. The editor grows through a sizing of the layout tree"],"It is at least 1 and at most `min(8, terminal rows / 2)`, but at least 1."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | inv-5: The editor height floor of 1 row, when half the terminal rows is 0, has no test. The tests use 24 rows (cap 8) and 10 rows (cap 5), and none at 16 or 17 rows, where both caps are 8.; fit-4: The limit of 8 rows and half the terminal sits in view, not in the layout node. #101 makes the layout configurable and adds an editor in a pane, so it must change the shape of Size::Editor, and half the terminal is wrong for an editor inside a split. |
| R-30 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7. The terminal cursor is the only cursor"],"The cell comes from `screen_cursor` and `scroll_offset` of the patched `TextArea`, inside the area of the editor."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | When the text fills the last row exactly, the cursor column of 0.9.4 is the area width, one cell past the editor. Patch 3 counts no row for that cursor. The insertion point is then outside the editor, and no test types exactly the width. |
| R-31 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: The screen has three parts"],"The bottom rows SHALL hold the pose editor, which this spec also calls the input line."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | The requirement 'The screen has three parts' has no task line that names the screen or its three parts. |
| R-32 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","Requirement: A command reaches only its world"],"The API call `Send` SHALL send one line to the world it names, and no other world."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | The requirement 'A command reaches only its world' matches one task line on one word only ('Command'). |
| R-33 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Requirement: Each field is known"],"An unknown field, or a value of the wrong type or out of range, SHALL be an error that names the file and the field."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | The requirement 'Each field is known' matches only the word 'field' in the task lines; no task covers the unknown-field error. |
| R-34 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Requirement: Each value is in its range"],"The range of `port`, `columns` and `rows` SHALL be 1 to 65535."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | The requirement 'Each value is in its range' matches one task line on one word only ('range'). |
| R-35 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/input-editor/spec.md","Requirement: The editor grows with its text"],"The editor SHALL take one row for each wrapped row of its text, and at least one row."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | The requirement 'The editor grows with its text' matches one task line on one word only ('editor'). |
| R-36 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: The client restores the terminal"],"When it exits, it SHALL turn off mouse capture, bracketed paste and each mode that it turned on"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | The requirement 'The client restores the terminal' has no task line that holds two of its words (client, restores, terminal). |
| R-37 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","4. The editor widget"],"Make sure that the four tests pass, and that every test in `crates/mu-tui/tests/app.rs` passes."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | 'The order of the worlds' has only the blanket 'every test in app.rs passes' of task 4.2, with no named test. |
| R-38 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","4. The editor widget"],"every test in `crates/mu-tui/tests/app.rs` passes."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | 'No worlds' has only the blanket app.rs pass of task 4.2, with no named test, though task 4.3 changes view. |
| R-39 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","4. The editor widget"],"`empty_enter_sends_empty_line`"] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | 'An empty line' exists in terminal-client and world-sessions. Tasks name only the client test (4.5), none for the session one. |
| R-40 | 1 | 2 | suggestion | deviates-design |  | ["artifact",["design.md","D8. The kitty keyboard protocol"],"It sets a second `AtomicBool`, so `screen::leave` and the panic hook write `CSI <u` to pop it."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | screen::open and screen::leave already take one flag for mode 2027. D8 adds a second AtomicBool beside it, so each restore path gets one more argument and one more clone for each mode that a later change sets. The parameter is the set of modes to undo at exit. |
| R-41 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D6. Bracketed paste"],"It drops each other C0 control character except tab, and DEL and each C1 control character."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | fit-6: text.rs already decides which characters are control characters, with char::is_control, for the text on the screen. D6 lists the same set again for the paste clean and does not say where the clean lives. The parameter is what a control character becomes (U+FFFD or nothing) and whether tab and LF stay.; inv-7: The clean of a paste is a rule over every string: its output holds no control character other than tab and LF, and a second clean changes nothing. The planned tests use two fixed strings. |
| R-42 | 1 | 2 | suggestion | in-change |  | ["artifact",["specs/world-sessions/spec.md","Requirement: A command reaches only its world"],"If the world has a `line_break`, the server SHALL replace each LF of the text with it."] | resolved | resolved in round 2 from fixed 0d2d0ae46fce081185cf6d5c84f2c1cbbd2f969d |  |  | The join is a rule over every text with no CR: the game gets exactly one line with no CR and no LF, whatever the count and place of LF. The planned tests use fixed texts. |
| R-43 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A terminal of 17 rows"] | refuted | tasks.md names a test for each scenario: terminal_of_17_rows, terminal_of_3_rows and terminal_of_1_row in task 4.3, up_into_an_emoji_with_joiners in 4.4, paste_over_a_selection in 5.3, cursor_at_a_full_row in 5.5. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'A terminal of 17 rows' has no matrix row |
| R-44 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A terminal of 3 rows"] | refuted | tasks.md names a test for each scenario: terminal_of_17_rows, terminal_of_3_rows and terminal_of_1_row in task 4.3, up_into_an_emoji_with_joiners in 4.4, paste_over_a_selection in 5.3, cursor_at_a_full_row in 5.5. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'A terminal of 3 rows' has no matrix row |
| R-45 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A terminal of 1 row"] | refuted | tasks.md names a test for each scenario: terminal_of_17_rows, terminal_of_3_rows and terminal_of_1_row in task 4.3, up_into_an_emoji_with_joiners in 4.4, paste_over_a_selection in 5.3, cursor_at_a_full_row in 5.5. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'A terminal of 1 row' has no matrix row |
| R-46 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Up into an emoji with joiners"] | refuted | tasks.md names a test for each scenario: terminal_of_17_rows, terminal_of_3_rows and terminal_of_1_row in task 4.3, up_into_an_emoji_with_joiners in 4.4, paste_over_a_selection in 5.3, cursor_at_a_full_row in 5.5. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'Up into an emoji with joiners' has no matrix row |
| R-47 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A paste with a selection"] | refuted | tasks.md names a test for each scenario: terminal_of_17_rows, terminal_of_3_rows and terminal_of_1_row in task 4.3, up_into_an_emoji_with_joiners in 4.4, paste_over_a_selection in 5.3, cursor_at_a_full_row in 5.5. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'A paste with a selection' has no matrix row |
| R-48 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The cursor after a full row"] | refuted | tasks.md names a test for each scenario: terminal_of_17_rows, terminal_of_3_rows and terminal_of_1_row in task 4.3, up_into_an_emoji_with_joiners in 4.4, paste_over_a_selection in 5.3, cursor_at_a_full_row in 5.5. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'The cursor after a full row' has no matrix row |
| R-49 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. Keys of the editor are entries of the keymap"],"`DeleteToLineStart` calls `delete_line_by_head`."] | resolved | resolved in round 4 from fixed 96eec86aea5952c26d6e8d884a0c0c81d68bec6b |  |  | Ctrl+U calls delete_line_by_head, which at column 0 deletes the line break before the cursor. So Ctrl+U at the start of a second line joins it to the line above, where the requirement deletes only from the start of the line to the cursor. |
| R-50 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Ctrl+U deletes to the start of the line"] | refuted | tasks.md names a test for each scenario: ctrl_u_deletes_to_the_start_of_the_line, undo_and_redo, ctrl_r_changes_nothing and ctrl_m_sends in task 5.2, built_in_type_that_extends_another and built_in_type_extends_only_a_built_in_type in 2.2. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'Ctrl+U deletes to the start of the line' has no matrix row |
| R-51 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Undo and redo"] | refuted | tasks.md names a test for each scenario: ctrl_u_deletes_to_the_start_of_the_line, undo_and_redo, ctrl_r_changes_nothing and ctrl_m_sends in task 5.2, built_in_type_that_extends_another and built_in_type_extends_only_a_built_in_type in 2.2. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'Undo and redo' has no matrix row |
| R-52 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Ctrl+R changes nothing"] | refuted | tasks.md names a test for each scenario: ctrl_u_deletes_to_the_start_of_the_line, undo_and_redo, ctrl_r_changes_nothing and ctrl_m_sends in task 5.2, built_in_type_that_extends_another and built_in_type_extends_only_a_built_in_type in 2.2. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'Ctrl+R changes nothing' has no matrix row |
| R-53 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Ctrl+M sends"] | refuted | tasks.md names a test for each scenario: ctrl_u_deletes_to_the_start_of_the_line, undo_and_redo, ctrl_r_changes_nothing and ctrl_m_sends in task 5.2, built_in_type_that_extends_another and built_in_type_extends_only_a_built_in_type in 2.2. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'Ctrl+M sends' has no matrix row |
| R-54 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A built-in type that extends another"] | refuted | tasks.md names a test for each scenario: ctrl_u_deletes_to_the_start_of_the_line, undo_and_redo, ctrl_r_changes_nothing and ctrl_m_sends in task 5.2, built_in_type_that_extends_another and built_in_type_extends_only_a_built_in_type in 2.2. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'A built-in type that extends another' has no matrix row |
| R-55 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A built-in type extends only a built-in type"] | refuted | tasks.md names a test for each scenario: ctrl_u_deletes_to_the_start_of_the_line, undo_and_redo, ctrl_r_changes_nothing and ctrl_m_sends in task 5.2, built_in_type_that_extends_another and built_in_type_extends_only_a_built_in_type in 2.2. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'A built-in type extends only a built-in type' has no matrix row |
| R-56 | 4 | 6 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D13. A send of each line"],"The model keeps the lines in a queue. `update` returns `Effect::Send` for the first line. On the reply of a line, `update` returns the effect for the next one."] | resolved | resolved in round 6 from fixed 3ef9823cf9fd052907418f0f0b56ae2b43fe4bb2 |  |  | D13 keeps only the lines in the queue and names no world for the later lines. The send path today takes the world from the focus at the time of the send, and Reply::Sent carries no world. So a Ctrl+N during a send of each line can send the rest of a script to another character. |
| R-57 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Ctrl+U at the start of a line"] | refuted | tasks.md names a test for each scenario: ctrl_u_at_the_start_of_a_line in task 5.2, each_line_is_a_command and refused_line_stops_the_rest in 5.6, lines_become_one_line and join_with_no_line_break in 5.7, list_worlds_shows_the_line_break in 2.6. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'Ctrl+U at the start of a line' has no matrix row |
| R-58 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Each line is a command"] | refuted | tasks.md names a test for each scenario: ctrl_u_at_the_start_of_a_line in task 5.2, each_line_is_a_command and refused_line_stops_the_rest in 5.6, lines_become_one_line and join_with_no_line_break in 5.7, list_worlds_shows_the_line_break in 2.6. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'Each line is a command' has no matrix row |
| R-59 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A line that the server refuses"] | refuted | tasks.md names a test for each scenario: ctrl_u_at_the_start_of_a_line in task 5.2, each_line_is_a_command and refused_line_stops_the_rest in 5.6, lines_become_one_line and join_with_no_line_break in 5.7, list_worlds_shows_the_line_break in 2.6. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'A line that the server refuses' has no matrix row |
| R-60 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The lines become one line"] | refuted | tasks.md names a test for each scenario: ctrl_u_at_the_start_of_a_line in task 5.2, each_line_is_a_command and refused_line_stops_the_rest in 5.6, lines_become_one_line and join_with_no_line_break in 5.7, list_worlds_shows_the_line_break in 2.6. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'The lines become one line' has no matrix row |
| R-61 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A world with no line break"] | refuted | tasks.md names a test for each scenario: ctrl_u_at_the_start_of_a_line in task 5.2, each_line_is_a_command and refused_line_stops_the_rest in 5.6, lines_become_one_line and join_with_no_line_break in 5.7, list_worlds_shows_the_line_break in 2.6. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'A world with no line break' has no matrix row |
| R-62 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The line break of a world"] | refuted | tasks.md names a test for each scenario: ctrl_u_at_the_start_of_a_line in task 5.2, each_line_is_a_command and refused_line_stops_the_rest in 5.6, lines_become_one_line and join_with_no_line_break in 5.7, list_worlds_shows_the_line_break in 2.6. The matrix is carried from round 1, and a follow-up round does not rebuild it. |  |  | delta scenario 'The line break of a world' has no matrix row |
| R-63 | 5 | 6 | blocker | in-change | drift | ["path","crates/mu-tui/src/update.rs",431] | resolved | resolved in round 6 from fixed 064a081061aaf4a039f9d178db451be781a71810 |  |  | join_lines adds a third copy of the lookup of the world in focus. Model::focused and Model::focused_pane already map focus to a World with the same expression, so a change of how focus names a world must be made in three places. |
| R-64 | 5 | 6 | blocker | in-change | drift | ["path","crates/mu-tui/tests/editor.rs",57] | resolved | resolved in round 6 from fixed 064a081061aaf4a039f9d178db451be781a71810 |  | scope? beyond-scope 0.69; severity? suggestion 0.84 | tests/editor.rs adds a third copy of the test helper type_text, which app.rs and update.rs of the same test directory already hold. The repository keeps shared test helpers in a support module, as crates/mu-cli/tests/support/mod.rs and crates/mu-core/tests/support/mod.rs do. |
| R-65 | 5 | 6 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: Alt+Enter sends each line as a command"],"The client SHALL send a line only after the server answered the line before it."] |  | regressed from fixed 064a081061aaf4a039f9d178db451be781a71810 |  | severity? blocker 0.87 | app-1: No decision keeps one line of a send of each line in flight when a plain Enter interleaves. Reply::Sent names no send, so the Sent of the plain line releases the next queued line before the queued line in flight is answered. Each send is its own spawned task, so two queued lines can then reach the server out of order, and a refusal of the first no longer stops the second. D13 calls only the place of the plain line unfixed.; inv-1: The rule that a line of a send of each line goes only after the server answered the line before it has no enforcement point. Reply::Sent names no send, and next_line advances the queue on any Sent. Sends run in spawned tasks, so the Sent of a plain Enter during the queue sends the next queued line while the earlier one is in flight. Two queued lines are then in flight at once, and their order at the server is not fixed. queue.sent is then overwritten, so a refusal of the earlier line no longer matches take_if and the rest of the script keeps going. |
| R-66 | 5 | 6 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: Alt+Enter sends each line as a command"],"the client SHALL send each line of the editor to the world in focus as its own text. It SHALL send the lines in order and clear the editor."] |  | regressed from fixed 064a081061aaf4a039f9d178db451be781a71810 |  |  | No decision says what a second Alt+Enter does while a queue waits. submit_lines assigns a new LineQueue over the old one, so the lines that the first Alt+Enter had not sent yet are dropped: they never go to the game and never come back to the editor. |
| R-67 | 5 | 6 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D13. A send of each line"],"It puts the refused line and the later lines back with `set_input`, before any text that the player typed since."] | resolved | resolved in round 6 from fixed 064a081061aaf4a039f9d178db451be781a71810 |  |  | app-3: D13 puts a refused line back before the text that the player typed since. next_line clears the queue when it sends the last line, so a refusal of that last line takes the plain path, and set_input replaces the typed text with the refused line alone.; inv-3: D13 puts a refused line back before the text that the player typed since. next_line sets model.queue to None when it sends the last line, so a refusal of the last line takes the None arm and set_input replaces the typed text. refused_line_stops_the_rest refuses only the middle of three lines and types nothing in between, so neither edge is tested. |
| R-68 | 5 | 6 | should-fix | in-change | drift | ["path","crates/mu-tui/tests/editor.rs",64] | resolved | resolved in round 6 from fixed 064a081061aaf4a039f9d178db451be781a71810 |  |  | The change writes the test helper type_lines twice, in app.rs and editor.rs, with the same doc line and the same Ctrl+J between lines. If the key of a line break changes, one copy can be fixed and the other not. |
| R-69 | 5 | 6 | should-fix | in-change | drift | ["path","crates/mu-tui/tests/editor.rs",42] | resolved | resolved in round 6 from fixed 064a081061aaf4a039f9d178db451be781a71810 |  | severity? suggestion 0.87 | tests/editor.rs copies the helper press of tests/update.rs, which wraps a code and modifiers into a key message for update, line for line. |
| R-70 | 5 | 6 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/update.rs",417] |  | regressed from fixed 064a081061aaf4a039f9d178db451be781a71810 |  |  | A second Alt+Enter while lines of an earlier send of each line still wait replaces model.queue. The waiting lines of the first script are dropped with no notice and never sent, and the reply of the in-flight line of the first script then advances the new queue, so two lines are in flight. No guard keeps one queue, and no test presses Alt+Enter twice. |
| R-71 | 5 | 6 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/editor.rs",196] | resolved | resolved in round 6 from fixed 064a081061aaf4a039f9d178db451be781a71810 |  |  | tes-1: terminal_of_1_row asserts only the frame has 1 row, not that the editor takes at most 1 row as the THEN says.; hon-1: terminal_of_1_row asserts the screen has 1 row, but the test set the height to 1, so the assertion cannot fail. The THEN 'the editor takes at most 1 row' is not touched; only a panic would fail it. |
| R-72 | 5 | 6 | suggestion | in-change |  | ["artifact",["design.md","D3. The editor widget and its state"],"`EditorState` holds the `TextArea`, the width profile as `Arc<Caps>`, and the last area that it drew."] | resolved | resolved in round 6 from fixed 064a081061aaf4a039f9d178db451be781a71810 |  | severity? should-fix 0.66 | D3 says EditorState holds the width profile as Arc<Caps>. The state holds only the TextArea and the area. The caps live in the closure of the width function, and set_text takes them as an argument from the model. |
| R-73 | 5 | 6 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"The decision page and the configuration guide name Ctrl+J as the line break there."] | resolved | resolved in round 6 from fixed 064a081061aaf4a039f9d178db451be781a71810 |  | severity? should-fix 0.82 | The Terminal.app risk names the configuration guide as a place that names Ctrl+J as the line break. No task adds that to the guide, and the guide does not mention Ctrl+J or Terminal.app at head. Only task 7.1 covers the decision page. |
| R-74 | 5 | 6 | suggestion | in-change |  | ["artifact",["design.md","Facts about third-party tools to check"],"`screen_cursor` gives the column one past the area after a row that the text fills exactly \| 5.5"] | resolved | resolved in round 6 from fixed 064a081061aaf4a039f9d178db451be781a71810 |  | severity? should-fix 0.80 | The fact of screen_cursor after a full row is still listed as a fact to check under task 5.5. Task 5.5 is done, D7 states the answer, and the test cursor_at_a_full_row exists, but the row was not moved to Checked facts as the task asks. |
| R-75 | 5 | 6 | suggestion | in-change |  | ["path","crates/mu-tui/src/editor.rs",54] | resolved | resolved in round 6 from fixed 064a081061aaf4a039f9d178db451be781a71810 |  | scope? beyond-scope 0.66; severity? should-fix 0.83 | EditorState does not hold its caps, as PaneState does and as D3 says. Each set_text call must pass the caps of the model, where PaneState::clear keeps its own. |
| R-76 | 5 | 6 | suggestion | in-change |  | ["path","crates/mu-tui/src/view.rs",88] | resolved | resolved in round 6 from fixed 064a081061aaf4a039f9d178db451be781a71810 |  | severity? should-fix 0.64 | Size::Editor in a horizontal split wraps the text at the height of the split and sets the width of the editor to a row count. #101 puts an editor in a pane that the player arranges, so it must change this sizing, though D4 says it needs no new shape of Size. |
| R-77 | 5 | 6 | suggestion | in-change | unmet-requirement | ["artifact",["design.md","What enforces each rule"],"The tests of patch 1 in the fork, and the scenarios of \\"The cursor moves by grapheme\\""] |  | regressed from fixed 064a081061aaf4a039f9d178db451be781a71810 |  | severity? should-fix 0.65 | The cursor of the editor stops only at a grapheme boundary after any sequence of moves and edits. That is a rule over every text and every key sequence, and its tests are fixed examples: four graphemes in the fork, and five scenarios in editor.rs. A move kind that the fork misses, such as a word move or an undo that restores a cursor, has no test. |
| R-78 | 5 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"A terminal that adds code point widths"] | refuted | Both covering tests of each row ran at the head of the round and passed. verification-runs.txt holds one line with exit code 0 for each of the two tests. The merge matches a row to one run, and each of these rows names two tests in two crates. |  |  | covered row 'A terminal that adds code point widths' has no verification run |
| R-79 | 5 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"Control characters in a paste"] | refuted | Both covering tests of each row ran at the head of the round and passed. verification-runs.txt holds one line with exit code 0 for each of the two tests. The merge matches a row to one run, and each of these rows names two tests in two crates. |  |  | covered row 'Control characters in a paste' has no verification run |
| R-80 | 5 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"An empty line"] | refuted | Both covering tests of each row ran at the head of the round and passed. verification-runs.txt holds one line with exit code 0 for each of the two tests. The merge matches a row to one run, and each of these rows names two tests in two crates. |  |  | covered row 'An empty line' has no verification run |
| R-81 | 5 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"Each line is a command"] | refuted | Both covering tests of each row ran at the head of the round and passed. verification-runs.txt holds one line with exit code 0 for each of the two tests. The merge matches a row to one run, and each of these rows names two tests in two crates. |  |  | covered row 'Each line is a command' has no verification run |
| R-82 | 5 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"A world that waits to reconnect"] | refuted | Both covering tests of each row ran at the head of the round and passed. verification-runs.txt holds one line with exit code 0 for each of the two tests. The merge matches a row to one run, and each of these rows names two tests in two crates. |  |  | covered row 'A world that waits to reconnect' has no verification run |
| R-83 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A send while lines wait"] |  |  |  |  | delta scenario 'A send while lines wait' has no matrix row |
| R-84 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The last line is refused"] |  |  |  |  | delta scenario 'The last line is refused' has no matrix row |

## Reviewed files

| path | content |
|---|---|
| Cargo.lock | ae12d6ee0d8d3f78423cd967de551b6b8bf019a7 |
| Cargo.toml | 8c6e01906f5d4c1d13721712f30516a9bf1f0dd0 |
| crates/mu-cli/src/server.rs | 5477927b25d519b2dcc8b953f8a94a3a10d120af |
| crates/mu-cli/src/world.rs | e1ad214479f938e9617f9698caddb3e672ecd3e2 |
| crates/mu-cli/tests/client.rs | 46b406d7b26399bef5f22ef14101d5ca941bd4a8 |
| crates/mu-cli/tests/repo_config.rs | acfb406ec19bcd48fcbdabeb8ce93ca9bc6b5aa2 |
| crates/mu-cli/tests/support/process.rs | 3947c4d7371b2fb4c968888f77905d126f0ed5a0 |
| crates/mu-cli/tests/world_config.rs | 33cc41ae22d77edad5fa8c3430ba95866c293efa |
| crates/mu-cli/tests/worlds.rs | 772be0fb8dab557ddcd0e9016649b11cbcaa657d |
| crates/mu-core/src/error.rs | 851676357ffb2a00c4f3836dea1707761dfc399d |
| crates/mu-core/src/lib.rs | d3fbf04fe45588e04ecfb946bc7faf34171dd895 |
| crates/mu-core/src/portal.rs | 826bbaea8064ea16f0b497edd34453f0db40f601 |
| crates/mu-core/src/session.rs | cb0aeb02b8c49990943f84cc4ceb915282254a41 |
| crates/mu-core/src/world/builtin.rs | 5e38aea2407ec1972cbf4e8a472be26caf4bff7d |
| crates/mu-core/src/world/check.rs | 27f0dbc04401aaf4231b55e63f576c7bea5a4382 |
| crates/mu-core/src/world/layer.rs | 2df3ab269a3c1a32a2243f602c2bc1d58169a409 |
| crates/mu-core/src/world/mod.rs | 8172ff2f8644b5fbdbcdd55dc910c8b59ad931ab |
| crates/mu-core/src/world/resolve.rs | 2a967da71ccf25d8281b494c54755bdc218547b5 |
| crates/mu-core/src/world/types/aresmush.toml | d04c9140140a793a55fdedc858790c3d94c02143 |
| crates/mu-core/src/world/types/evennia.toml | 9d997c5994471350af8d06cb01644159793df6be |
| crates/mu-core/src/world/types/pennmush.toml | 4cd3398caf4bed01690c32137a4a391ba118ce22 |
| crates/mu-core/src/world/types/rhostmush.toml | 69c3e18ed35c0ca28d86ecf0b8c45173dbf73bfe |
| crates/mu-core/src/world/types/tinymux.toml | 88718b0d2525d388d77b1bb6ca82847311cc9a34 |
| crates/mu-core/tests/portal.rs | 659f7c0430b3b8bcbcdb5450100b652b4826b279 |
| crates/mu-core/tests/scripts.rs | b1f2f3613c1af338a94bc9187cb028f350d3e362 |
| crates/mu-core/tests/support/mod.rs | d8a4c12253679261c9ef3e6af234f80a70f7013c |
| crates/mu-core/tests/world_config.rs | c0962142ca7c8a5c665a579656106d60ef4e0c4d |
| crates/mu-proto/proto/mu/v1/portal.proto | b449e2a455e9133a655b4bf45cf80885e1edad30 |
| crates/mu-server/src/convert.rs | 14370e58409b74a97d53fc004a2e05e8e05bd009 |
| crates/mu-server/src/service.rs | 81a0a07cd304f6624daf18c4625934d028ac6375 |
| crates/mu-tui/examples/image-pane.rs | 5c9ed2d0fdc5f3c9479cb73fe1584ee625615487 |
| crates/mu-tui/src/editor.rs | ec44faf4c97d7e9a8e14ae02ff06e4cd861fcb84 |
| crates/mu-tui/src/keymap.rs | 52e7cb0531f3d9857488ad8336d32abe88f50e90 |
| crates/mu-tui/src/layout.rs | a733e998b83f74af03cc745990fa789a56fafad7 |
| crates/mu-tui/src/lib.rs | f537f4780b2d6befe77c20fc8186da2f8be12fb4 |
| crates/mu-tui/src/model.rs | 4dd0b23d752552b47d834df5c4e75a0d1dfb1390 |
| crates/mu-tui/src/probe.rs | 1c2e4e998f6ac26c3fba93b7d69c4f5199ecbf5d |
| crates/mu-tui/src/run.rs | 5ff60fced916fb261a99a1cd0076fce5f7fe41cd |
| crates/mu-tui/src/screen.rs | c4ae98df0e16192c1d14e4423505b4de293da09a |
| crates/mu-tui/src/text.rs | 4ffb745e77024d2ac6f95384e3b84a8ba234c207 |
| crates/mu-tui/src/update.rs | a53597abd90609a8b89a685da955df22276a50b8 |
| crates/mu-tui/src/view.rs | 926cbffca7516b83317dd63b06f7af6bd05a9b28 |
| crates/mu-tui/tests/app.rs | 6fc7b5de7530d8eabde2eb63aa654359483fb01b |
| crates/mu-tui/tests/data/pose-4096.txt | c6433795dec0a1a12252e94772813e0fa9d85c42 |
| crates/mu-tui/tests/editor.rs | 08887da8821cdf91f05462edd5b2d4cdd0636e47 |
| crates/mu-tui/tests/probe.rs | 70e02261e628a96534d3ba314c5664ba7e884c29 |
| crates/mu-tui/tests/screen.rs | 4b0619a019e7ad24cf5dfc349ebab26ca78a1a82 |
| crates/mu-tui/tests/support/mod.rs | fc347cea3e5cd13187f6b106ea0b3ecbb1ef1de6 |
| crates/mu-tui/tests/update.rs | f8d4e93e11f7c3c852ed81fd763df785f6d7f87b |
| deny.toml | bc72cc7d12ae78717442d2a3f09e3fd4a136bd88 |
| docs/src/content/docs/decisions/input-editor.md | 8e16636bca0a3b47d6dc7ba5652f1da6900d5674 |
| docs/src/content/docs/guides/configuration.md | e79b09b090a962865543e3c3a01d0c47ee77501f |
| openspec/changes/pose-editor/.openspec.yaml | 5274a13b087156d95ab9dc0d0657588a63841492 |
| openspec/changes/pose-editor/design.md | cf94f0e6b0fa377893092d1d858dadd8e78f2b72 |
| openspec/changes/pose-editor/proposal.md | 9b911e9f4cd34aba0bea4ecc091c1ef42faa8014 |
| openspec/changes/pose-editor/specs/input-editor/spec.md | 93d59ed60bede447652e7528067b3934106c85de |
| openspec/changes/pose-editor/specs/terminal-client/spec.md | a3d6cd61680a28afc8fdbb5f6abd1ebb07c87c7a |
| openspec/changes/pose-editor/specs/world-config/spec.md | 2eeb164fcf41e83a3abffe9bea12526962651288 |
| openspec/changes/pose-editor/specs/world-sessions/spec.md | 9e742df53f7a26c80022063a2c3c8b9de7099fc8 |
| openspec/changes/pose-editor/tasks.md | sha256:fd69f0ae5f1eb5059b0838858e0abd663be4fa3ddbd9a03504e58907dd95786e |
| openspec/review.json | absent |
| renovate.json | 672ea39af9d44dc1b02542067e1cdb821e7fdc3d |

## Plan review (2026-11-21)

Verdict approve-with-notes at head 533e1bb37337a23ee482e7bf7ed03099b4449945.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-57 | test-coverage | should-fix | in-change | delta scenario 'Ctrl+U at the start of a line' has no matrix row | review > absent-row — "Ctrl+U at the start of a line" | add the row with its covering task or test |  |  | unmet-requirement |
| R-58 | test-coverage | should-fix | in-change | delta scenario 'Each line is a command' has no matrix row | review > absent-row — "Each line is a command" | add the row with its covering task or test |  |  | unmet-requirement |
| R-59 | test-coverage | should-fix | in-change | delta scenario 'A line that the server refuses' has no matrix row | review > absent-row — "A line that the server refuses" | add the row with its covering task or test |  |  | unmet-requirement |
| R-60 | test-coverage | should-fix | in-change | delta scenario 'The lines become one line' has no matrix row | review > absent-row — "The lines become one line" | add the row with its covering task or test |  |  | unmet-requirement |
| R-61 | test-coverage | should-fix | in-change | delta scenario 'A world with no line break' has no matrix row | review > absent-row — "A world with no line break" | add the row with its covering task or test |  |  | unmet-requirement |
| R-62 | test-coverage | should-fix | in-change | delta scenario 'The line break of a world' has no matrix row | review > absent-row — "The line break of a world" | add the row with its covering task or test |  |  | unmet-requirement |
| R-56 | verify | should-fix | in-change | D13 keeps only the lines in the queue and names no world for the later lines. The send path today takes the world from the focus at the time of the send, and Reply::Sent carries no world. So a Ctrl+N during a send of each line can send the rest of a script to another character. | design.md > D13. A send of each line — "The model keeps the lines in a queue. `update` returns `Effect::Send` for the first line. On the reply of a line, `updat" | In D13, keep the world of the Alt+Enter with the queue and send each later line to that world, whatever the focus. State it in the requirement, and add a test that moves the focus between two lines. |  |  | wrong-behaviour |

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A line wider than the editor | 4.2 | covered |  |
| A word wider than the editor | 4.2 | covered |  |
| Three lines | 4.3 | covered |  |
| More rows than the limit | 4.3 | covered |  |
| A short terminal | 4.3 | covered |  |
| Back to one row | 4.3 | covered |  |
| Backspace after an emoji with joiners | 4.4 | covered |  |
| Left over a flag | 4.4 | covered |  |
| Right over a combining accent | 4.4 | covered |  |
| Delete before a skin tone | 4.4 | covered |  |
| A terminal that adds code point widths | 4.2, 5.5 | covered |  |
| A terminal that does not answer | 4.2 | covered |  |
| Each key of a line break | 5.2 | covered |  |
| A paste of several lines | 5.3 | covered |  |
| Undo of a paste | 5.3 | covered |  |
| Control characters in a paste | 5.3, 5.4 | covered |  |
| The cursor after typing | 5.5 | covered |  |
| The cursor on a wrapped row after a wide grapheme | 5.5 | covered |  |
| The first view | 4.3 | covered |  |
| The order of the worlds | 4.2 | partial |  |
| No worlds | 4.2 | partial |  |
| A line goes to the world in focus | 4.5 | covered |  |
| An empty line | 4.5 | partial |  |
| Enter with no world | 4.5 | covered |  |
| A refused line | 4.5 | covered |  |
| A pose of two lines | 4.5 | covered |  |
| A line break that the world refuses | 4.5 | covered |  |
| The modes at exit | 5.4, 6.2 | covered |  |
| Mode 2027 was already on | 5.4 | covered |  |
| Bracketed paste at the start and at exit | 5.4 | covered |  |
| A terminal with the protocol | 6.2 | covered |  |
| A terminal without the protocol | 6.2 | covered |  |
| Shift+Enter with the protocol | 6.2 | covered |  |
| A late answer of the kitty keyboard query | 6.2 | covered |  |
| The fake world answers | 2.4 | covered |  |
| The fake world answers a mark in order |  | missing |  |
| The flag is hidden |  | missing |  |
| A missing password variable |  | missing |  |
| A fake world does not reconnect | 2.4 | covered |  |
| The fake world gets a joined line | 2.4 | covered |  |
| Two characters on one game |  | missing |  |
| An unknown world |  | missing |  |
| A world that is not connected |  | missing |  |
| A write that fails |  | missing |  |
| A world that is still connecting |  | missing |  |
| A world that waits to reconnect |  | missing |  |
| A carriage return in the text | 2.3 | covered |  |
| A line break in the text | 2.3 | covered |  |
| A line break that the world joins | 2.3 | covered |  |
| A carriage return to a world that joins | 2.3 | covered |  |
| A hook sees the joined line | 2.3 | covered |  |
| Each built-in type reads as quirks | 2.2 | covered |  |
| A global preference is not hidden by a type |  | missing |  |
| A user type with a built-in name | 2.2 | covered |  |
| The line break of each built-in type | 2.2 | covered |  |
| A game overrides the line break of its type | 2.2 | covered |  |
| An unknown field |  | missing |  |
| A port out of range | 2.2 | covered |  |
| A template with no password |  | missing |  |
| The case of a character set |  | missing |  |
| A character set that the engine cannot read |  | missing |  |
| An unknown mode |  | missing |  |
| The edges of each range | 2.2 | covered |  |
| The edges of the attempts |  | missing |  |
| A line break in a line field | 2.2 | covered |  |
| The edges of the line break | 2.2 | covered |  |

### Not run

Plan phase: no verification runs.
