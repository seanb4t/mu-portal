---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-25T15:44:09+00:00
head: 63a7fc667751c359594aa3aeb0af63dd0bab1692
round: 11
next_id: 156
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: partial
  test-coverage: ok
  test-honesty: partial
  invariants: partial
  verify: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-155 | test-coverage | should-fix | in-change | delta scenario 'The earlier lines of a block that stands in for main' has no matrix row | review > absent-row — "The earlier lines of a block that stands in for main" | add the row with its covering task or test |  |  | unmet-requirement |

## Repeats

| id | lens | severity | claim | evidence |
|---|---|---|---|---|
| R-96 | test-coverage | should-fix | covered row 'An unknown field' has no verification run | review > not-run — "An unknown field" |
| R-97 | test-coverage | should-fix | covered row 'The editor after the notices' has no verification run | review > not-run — "The editor after the notices" |
| R-98 | test-coverage | should-fix | covered row 'The editor before the notices' has no verification run | review > not-run — "The editor before the notices" |
| R-124 | test-coverage | should-fix | covered row 'A block beside two rows' has no verification run | review > not-run — "A block beside two rows" |
| R-125 | test-coverage | should-fix | covered row 'An id in a later line' has no verification run | review > not-run — "An id in a later line" |
| R-142 | test-coverage | should-fix | covered row 'The default layout' has no verification run | review > not-run — "The default layout" |
| R-143 | test-coverage | should-fix | covered row 'A label too long' has no verification run | review > not-run — "A label too long" |
| R-99 | test-coverage | should-fix | covered row 'No world in focus' has no verification run | review > not-run — "No world in focus" |
| R-144 | test-coverage | should-fix | covered row 'No worlds' has no verification run | review > not-run — "No worlds" |
| R-100 | test-coverage | should-fix | covered row 'Under an hour' has no verification run | review > not-run — "Under an hour" |
| R-101 | test-coverage | should-fix | covered row 'Over an hour' has no verification run | review > not-run — "Over an hour" |
| R-145 | test-coverage | should-fix | covered row 'A drag past the smallest pane' has no verification run | review > not-run — "A drag past the smallest pane" |
| R-150 | test-coverage | should-fix | delta scenario 'The main output is not the first output line' has no matrix row | review > absent-row — "The main output is not the first output line" |
| R-147 | test-coverage | should-fix | delta scenario 'A hide of the main output' has no matrix row | review > absent-row — "A hide of the main output" |
| R-148 | test-coverage | should-fix | delta scenario 'A new main output takes the lines of the world back' has no matrix row | review > absent-row — "A new main output takes the lines of the world back" |
| R-149 | test-coverage | should-fix | delta scenario 'A line of a world with no output' has no matrix row | review > absent-row — "A line of a world with no output" |
| R-151 | test-coverage | should-fix | delta scenario 'A save with a key' has no matrix row | review > absent-row — "A save with a key" |
| R-152 | test-coverage | should-fix | delta scenario 'A save with a key and no device' has no matrix row | review > absent-row — "A save with a key and no device" |
| R-153 | test-coverage | should-fix | delta scenario 'A command that changes nothing' has no matrix row | review > absent-row — "A command that changes nothing" |
| R-154 | test-coverage | should-fix | delta scenario 'A drag back to its start' has no matrix row | review > absent-row — "A drag back to its start" |

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-73 | should-fix | accepted | Task 3.5a of tasks.md names the test the_input_focus_after_a_change_of_the_world for this scenario. Only a full round writes a matrix row, and the operator chose no further full round. |  |
| R-102 | should-fix | accepted | the lens sampled large files in two full rounds, with a different sample each time; the operator accepted the gap, and the suite review of task 7.3 reads the branch again |  |
| R-103 | should-fix | accepted | the lens sampled large files in two full rounds, with a different sample each time; the operator accepted the gap, and the suite review of task 7.3 reads the branch again |  |
| R-104 | should-fix | accepted | the lens sampled large files in two full rounds, with a different sample each time; the operator accepted the gap, and the suite review of task 7.3 reads the branch again |  |
| R-146 | should-fix | accepted | the fit lens sampled in the third full round; the operator accepted the gap as for the other lenses, and the suite review of task 7.3 reads the branch again |  |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | d2729150530ac1884073ee7207a64ed3a63ab418 | 0face2499054203abac6a0977c535eb9937058b31597b2276d6765a7042c028d | changes-requested | 35 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | a89e8ef5614c7ab7bd977258c84bc8ba8bc1ac31 | 215c0bd88e3b28033d1218fa69f8e88e090deebc492d274c174eaf4de3a135a3 | approve-with-notes | 8 | 0 | 1 | 33 | 0 |  |
| 3 | plan | full | 12c5e1eb51d612f7a7ad524730010c42c27e625a | b1034da5f8859358c6cfa265503a39cca17afab80d8900ab6706bea5c9cb57db | changes-requested | 29 | 0 | 0 | 0 | 0 | the operator asked for a full round to rebuild the coverage matrix after the fixes |
| 4 | plan | follow-up | 6c78c3f9426d5a2dc94654a44d57d87b3c468073 | 30e945c0cdd12743d13dd5a1582f22da0658f442f9cf7266152530053a4e75f9 | approve-with-notes | 1 | 0 | 2 | 27 | 0 |  |
| 5 | plan | follow-up | dc8e5c3bdf083ba14f7b8ffa417a527aa6b701ad | a18bc81b621e5b0ba32cbccbbc069d6c4a573d3c823551ea6df1cbf68f8efdc8 | approve | 0 | 1 | 0 | 2 | 0 |  |
| 6 | implementation | full | d3bceb71013766854fc8e1c20fef7ff25f0d69e0 | 9f2cbbc88f8b829cf9317c43ed658d28a97311f1f529ca48b13c3dced4b0961a | changes-requested | 31 | 0 | 0 | 0 | 0 |  |
| 7 | implementation | full | a983b5c30a423d2c79d396fecf2679622b8c75ad | 743f642b3e1bcff96359d67603474b59625fa7315c2102f9842e22020c6c0acf | approve-with-notes | 21 | 0 | 9 | 0 | 0 | three lenses returned partial in round 6, and the operator asked for a full round after the fixes |
| 8 | implementation | follow-up | 3107d64acd179ba3b82f5f83b03be6d96b4dd5cd | 9896adbb614840e1c0d7515640e5a5f6168ce34c1c52aad82763bfe4c81ae59f | approve-with-notes | 4 | 8 | 0 | 19 | 0 |  |
| 9 | implementation | full | 3f3084a618f082bd7b2f43b6f3324b8d856fb5fc | 9896adbb614840e1c0d7515640e5a5f6168ce34c1c52aad82763bfe4c81ae59f | approve-with-notes | 17 | 10 | 0 | 0 | 0 | the round 7 fixes added four scenarios that no test-coverage lens graded, and the operator asked for a third full round |
| 10 | implementation | follow-up | 661b9163f5b3f87e7f993404dd272348b07f85a1 | 03b90dc57e53621491b3ff3fb0c920fb9053d953cd85fb154c8d75287768e01e | approve-with-notes | 8 | 12 | 1 | 12 | 0 |  |
| 11 | implementation | follow-up | 63a7fc667751c359594aa3aeb0af63dd0bab1692 | 0a37b2f94e75c1b74d6f988222e425c61dfdc819e2b6f8f59f4fdf4a6edb3129 | approve-with-notes | 1 | 20 | 0 | 1 | 0 |  |

## Judge

judge: 0 questions, 0.000 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A server table | crates/mu-cli/tests/config_store.rs: a_server_table | covered | pass |
| An unknown field | crates/mu-cli/tests/config_store.rs: an_unknown_field; an_unknown_label_field | covered | not-run |
| Not TOML | crates/mu-cli/tests/config_store.rs: not_toml | covered | pass |
| A file that cannot be read | crates/mu-cli/tests/config_store.rs: a_file_that_cannot_be_read | covered | pass |
| The three new fields | crates/mu-cli/tests/config_store.rs: the_three_new_fields | covered | pass |
| A layout table | crates/mu-cli/tests/config_store.rs: a_layout_table | covered | pass |
| An unknown field of tabs | crates/mu-cli/tests/config_store.rs: an_unknown_field_of_tabs | covered | pass |
| A world layouts table | crates/mu-cli/tests/config_store.rs: a_world_layouts_table | covered | pass |
| No tabs | crates/mu-cli/tests/client.rs: no_tabs | covered | pass |
| The editor alone | crates/mu-cli/tests/client.rs: the_editor_alone | covered | pass |
| Nine outputs | crates/mu-config/tests/layout.rs: nine_outputs | covered | pass |
| One output shows the notices too | crates/mu-tui/tests/app.rs: one_output_shows_the_notices_too | covered | pass |
| An output for the notices | crates/mu-tui/tests/app.rs: an_output_for_the_notices | covered | pass |
| An output with no stream yet | crates/mu-tui/tests/app.rs: an_output_with_no_stream_yet | covered | pass |
| A refused line goes to the notices | crates/mu-tui/tests/app.rs: a_refused_line_goes_to_the_notices_block | covered | pass |
| A refused line of a world that lost the focus | crates/mu-tui/tests/app.rs: a_refused_line_of_another_world_waits_in_its_notices | covered | pass |
| A line with no world goes to the notices | crates/mu-tui/tests/app.rs: enter_with_no_world_goes_to_the_notices_block | covered | pass |
| An output that comes later starts with new lines | crates/mu-tui/tests/app.rs: an_output_that_comes_later_starts_with_new_lines | covered | pass |
| A notice shows under the world in focus only | crates/mu-tui/tests/app.rs: a_notice_shows_under_the_world_in_focus_only | covered | pass |
| Two editors keep two texts | crates/mu-tui/tests/app.rs: two_editors_keep_two_texts | covered | pass |
| Enter sends the editor with the input focus | crates/mu-tui/tests/app.rs: enter_sends_the_editor_with_the_input_focus | covered | pass |
| One text for each world | crates/mu-tui/tests/app.rs: one_text_for_each_world | covered | pass |
| A second editor with no id | crates/mu-config/tests/layout.rs: a_second_editor_with_no_id | covered | pass |
| Five editors | crates/mu-config/tests/layout.rs: five_editors | covered | pass |
| The key moves the input focus | crates/mu-tui/tests/app.rs: the_key_moves_the_input_focus | covered | pass |
| A click moves the input focus | crates/mu-tui/tests/app.rs: a_click_moves_the_input_focus | covered | pass |
| One editor | crates/mu-tui/tests/app.rs: one_editor | covered | pass |
| The input focus after a change of the world | crates/mu-cli/tests/client.rs: the_input_focus_after_a_change_of_the_world | covered | pass |
| The output above the tabs | crates/mu-cli/tests/client.rs: the_output_above_the_tabs | covered | pass |
| A block beside another | crates/mu-cli/tests/client.rs: a_block_beside_another | covered | pass |
| The numbers of the lines | crates/mu-cli/tests/layout.rs: the_numbers_of_the_lines | covered | pass |
| The editor after the notices | crates/mu-config/tests/layout.rs: the_editor_after_the_notices; crates/mu-tui/tests/app.rs: the_editor_after_the_notices_gets_its_rows | covered | not-run |
| The editor before the notices | crates/mu-config/tests/layout.rs: the_editor_before_the_notices; crates/mu-tui/tests/app.rs: the_editor_before_the_notices_gets_its_columns | covered | not-run |
| A block beside two rows | crates/mu-tui/tests/app.rs: a_block_beside_two_rows_gets_its_columns; crates/mu-config/tests/layout.rs: a_block_beside_two_rows | covered | not-run |
| Anchors that are not neighbors | crates/mu-config/tests/layout.rs: anchors_that_are_not_neighbors | covered | pass |
| An id in a later line | crates/mu-config/tests/layout.rs: an_id_in_a_later_line; crates/mu-cli/tests/client.rs: an_id_in_a_later_line | covered | not-run |
| An id twice | crates/mu-config/tests/layout.rs: an_id_twice | covered | pass |
| An id that is a word of the sentences | crates/mu-config/tests/layout.rs: an_id_that_is_a_word_of_the_sentences | covered | pass |
| The statusline at the bottom | crates/mu-cli/tests/client.rs: the_statusline_at_the_bottom | covered | pass |
| Notices behind the output | crates/mu-cli/tests/client.rs: notices_behind_the_output | covered | pass |
| The editor does not stack | crates/mu-config/tests/layout.rs: the_editor_does_not_stack | covered | pass |
| A percent | crates/mu-cli/tests/client.rs: a_percent | covered | pass |
| A count of cells | crates/mu-cli/tests/client.rs: a_count_of_cells | covered | pass |
| A size on the tabs | crates/mu-config/tests/layout.rs: a_size_on_the_tabs | covered | pass |
| A size on the first line | crates/mu-config/tests/layout.rs: a_size_on_the_first_line | covered | pass |
| The editor stops at its max | crates/mu-cli/tests/client.rs: the_editor_stops_at_its_max | covered | pass |
| The default max | crates/mu-tui/tests/app.rs: the_default_max | covered | pass |
| Two editors side by side | crates/mu-tui/tests/app.rs: two_editors_side_by_side | covered | pass |
| The shorter max does not stop the taller editor | crates/mu-tui/tests/app.rs: the_shorter_max_does_not_stop_the_taller_editor | covered | pass |
| An unknown block | crates/mu-config/tests/layout.rs: an_unknown_block | covered | pass |
| A block twice | crates/mu-config/tests/layout.rs: a_block_twice | covered | pass |
| A block that is not placed yet | crates/mu-config/tests/layout.rs: a_block_that_is_not_placed_yet | covered | pass |
| No editor | crates/mu-config/tests/layout.rs: no_editor | covered | pass |
| A percent out of range | crates/mu-config/tests/layout.rs: a_percent_out_of_range | covered | pass |
| Too many lines | crates/mu-config/tests/layout.rs: too_many_lines | covered | pass |
| Any layout reads back | crates/mu-config/tests/layout.rs: any_layout_reads_back | covered | pass |
| The default layout | crates/mu-config/tests/layout.rs: each_built_in_layout_is_valid; crates/mu-cli/tests/config_store.rs: the_default_layout | covered | not-run |
| The sidebar layout | crates/mu-cli/tests/client.rs: the_sidebar_layout | covered | pass |
| Two text panes | crates/mu-tui/tests/app.rs: two_text_panes_have_title_rows | covered | pass |
| One text pane has no title row | crates/mu-tui/tests/app.rs: one_text_pane_has_no_title_row | covered | pass |
| A percent in a narrow terminal | crates/mu-tui/tests/app.rs: a_percent_in_a_narrow_terminal | covered | pass |
| A reload changes the layout | crates/mu-cli/tests/client.rs: a_reload_changes_the_layout | covered | pass |
| A layout with no editor from the server | crates/mu-tui/tests/app.rs: a_layout_with_no_editor_from_the_server | covered | pass |
| A label with the number | crates/mu-cli/tests/client.rs: a_label_with_the_number | covered | pass |
| A label keeps the unseen mark | crates/mu-tui/tests/app.rs: a_label_keeps_the_unseen_mark | covered | pass |
| The game and the character | crates/mu-cli/tests/client.rs: the_game_and_the_character | covered | pass |
| Braces | crates/mu-cli/tests/client.rs: braces | covered | pass |
| An open brace | crates/mu-config/tests/layer.rs: an_open_brace | covered | pass |
| A label too long | crates/mu-config/tests/layer.rs: a_label_too_long; a_label_of_64_characters | covered | not-run |
| A world with its own layout | crates/mu-cli/tests/client.rs: a_world_with_its_own_layout | covered | pass |
| The focus moves to a world with no entry | crates/mu-cli/tests/client.rs: the_focus_moves_to_a_world_with_no_entry | covered | pass |
| A layout for a game | crates/mu-cli/tests/client.rs: a_layout_for_a_game | covered | pass |
| The world entry comes before the game entry | crates/mu-cli/tests/client.rs: the_world_entry_comes_before_the_game_entry | covered | pass |
| No world in focus | crates/mu-cli/tests/client.rs: no_world_in_focus_takes_the_default_layout; no_world_in_focus_for_layout_here | covered | not-run |
| A world keeps its text across a change of layout | crates/mu-cli/tests/client.rs: a_world_keeps_its_text_across_a_change_of_layout | covered | pass |
| The default segments | crates/mu-cli/tests/client.rs: the_default_segments | covered | pass |
| A narrow terminal | crates/mu-tui/tests/app.rs: a_narrow_statusline | covered | pass |
| The typed count | crates/mu-tui/tests/app.rs: the_typed_count | covered | pass |
| The state after a disconnect | crates/mu-cli/tests/client.rs: the_state_after_a_disconnect | covered | pass |
| The idle time after a send | crates/mu-cli/tests/client.rs: the_idle_time_after_a_send | covered | pass |
| The connected time | crates/mu-cli/tests/client.rs: the_connected_time | covered | pass |
| No connected time while disconnected | crates/mu-tui/tests/app.rs: no_connected_time_while_disconnected | covered | pass |
| No worlds | crates/mu-tui/tests/app.rs: no_world_in_the_statusline; no_worlds | covered | not-run |
| Under an hour | crates/mu-tui/tests/app.rs: a_time_under_an_hour; a_time_one_second_under_an_hour | covered | not-run |
| Over an hour | crates/mu-tui/tests/app.rs: a_time_over_an_hour; a_time_of_one_hour | covered | not-run |
| A tick moves the idle time | crates/mu-tui/tests/update.rs: a_tick_moves_the_idle_time | covered | pass |
| No tick without a time segment | crates/mu-tui/tests/update.rs: no_tick_without_a_time_segment | covered | pass |
| An unknown segment | crates/mu-config/tests/layer.rs: an_unknown_segment | covered | pass |
| A segment twice | crates/mu-config/tests/layer.rs: a_segment_twice | covered | pass |
| A device sets the left list | crates/mu-cli/tests/config_store.rs: a_device_sets_the_left_list | covered | pass |
| A layout file in use | crates/mu-cli/tests/config_store.rs: a_layout_file_in_use | covered | pass |
| A bad layout file at start | crates/mu-cli/tests/config_store.rs: a_bad_layout_file_at_start | covered | pass |
| A file with a built-in name | crates/mu-cli/tests/config_store.rs: a_file_with_a_built_in_name | covered | pass |
| An unknown field of a layout file | crates/mu-cli/tests/config_store.rs: an_unknown_field_of_a_layout_file | covered | pass |
| The device names another layout | crates/mu-cli/tests/config_store.rs: the_device_names_another_layout | covered | pass |
| A layout that does not exist | crates/mu-cli/tests/config_store.rs: a_layout_that_does_not_exist | covered | pass |
| A change to a layout file changes the revision | crates/mu-cli/tests/config_store.rs: a_change_to_a_layout_file_changes_the_revision | covered | pass |
| The list with one player layout | crates/mu-cli/tests/layout.rs: the_list_with_one_player_layout | covered | pass |
| The list with a bad layout file | crates/mu-cli/tests/layout.rs: the_list_with_a_bad_layout_file | covered | pass |
| A save writes the file | crates/mu-cli/tests/layout.rs: a_save_writes_the_file | covered | pass |
| A save of a bad layout | crates/mu-cli/tests/layout.rs: a_save_of_a_bad_layout | covered | pass |
| A save with a built-in name | crates/mu-cli/tests/layout.rs: a_save_with_a_built_in_name | covered | pass |
| A save with a bad name | crates/mu-cli/tests/layout.rs: a_save_with_a_bad_name | covered | pass |
| A save with the name of two dots | crates/mu-cli/tests/layout.rs: a_save_with_the_name_of_two_dots | covered | pass |
| A save through a linked directory | crates/mu-cli/tests/layout.rs: a_save_through_a_linked_directory | covered | pass |
| A clients path that is a file | crates/mu-cli/tests/layout.rs: a_clients_path_that_is_a_file | covered | pass |
| A device file with a comment | crates/mu-cli/tests/layout.rs: a_device_file_with_a_comment | covered | pass |
| No device file | crates/mu-cli/tests/layout.rs: no_device_file | covered | pass |
| An unknown layout | crates/mu-cli/tests/layout.rs: an_unknown_layout | covered | pass |
| A bad device layer | crates/mu-cli/tests/layout.rs: a_bad_device_layer | covered | pass |
| A use from the command line | crates/mu-cli/tests/layout.rs: a_use_from_the_command_line | covered | pass |
| The picture of classic | crates/mu-cli/tests/layout.rs: the_picture_of_classic | covered | pass |
| An open socket directory | crates/mu-cli/tests/layout.rs: an_open_socket_directory | covered | pass |
| No server | crates/mu-cli/tests/layout.rs: no_server | covered | pass |
| An edit places a block | crates/mu-cli/tests/layout.rs: an_edit_places_a_block | covered | pass |
| An edit gives the same file as the command | crates/mu-cli/tests/layout.rs: an_edit_gives_the_same_file_as_the_command | covered | pass |
| An insert from the command line | crates/mu-cli/tests/layout.rs: an_insert_from_the_command_line | covered | pass |
| A delete by id | crates/mu-cli/tests/layout.rs: a_delete_by_id | covered | pass |
| An edit of a built-in layout | crates/mu-cli/tests/layout.rs: an_edit_of_a_built_in_layout | covered | pass |
| A bad edit | crates/mu-cli/tests/layout.rs: a_bad_edit | covered | pass |
| Describe gives each block | crates/mu-cli/tests/layout.rs: describe_gives_each_block | covered | pass |
| Describe of an unknown layout | crates/mu-cli/tests/layout.rs: describe_of_an_unknown_layout | covered | pass |
| A device adds an entry | crates/mu-cli/tests/config_store.rs: a_device_adds_an_entry | covered | pass |
| A device replaces an entry | crates/mu-cli/tests/config_store.rs: a_device_replaces_an_entry | covered | pass |
| An entry for a world that the server does not have | crates/mu-cli/tests/config_store.rs: an_entry_for_a_world_that_the_server_does_not_have | covered | pass |
| A change to the layout file of an entry changes the revision | crates/mu-cli/tests/config_store.rs: a_change_to_the_layout_file_of_an_entry_changes_the_revision | covered | pass |
| A key with two slashes | crates/mu-cli/tests/config_store.rs: a_key_with_two_slashes | covered | pass |
| An entry with a layout that does not exist | crates/mu-cli/tests/config_store.rs: an_entry_with_a_layout_that_does_not_exist | covered | pass |
| A use for one world | crates/mu-cli/tests/layout.rs: a_use_for_one_world | covered | pass |
| A use for a game | crates/mu-cli/tests/layout.rs: a_use_for_a_game | covered | pass |
| Forget the layout of a world | crates/mu-cli/tests/layout.rs: forget_the_layout_of_a_world | covered | pass |
| A use with a bad world id | crates/mu-cli/tests/layout.rs: a_use_with_a_bad_world_id | covered | pass |
| An edit of the built-in layout of one world | crates/mu-cli/tests/layout.rs: an_edit_of_the_built_in_layout_of_one_world | covered | pass |
| Add the notices | crates/mu-tui/tests/app.rs: add_the_notices | covered | pass |
| Move the tabs | crates/mu-tui/tests/app.rs: move_the_tabs | covered | pass |
| A bad sentence | crates/mu-tui/tests/app.rs: a_bad_sentence | covered | pass |
| Hide the tabs | crates/mu-tui/tests/app.rs: hide_the_tabs | covered | pass |
| The editor stays | crates/mu-tui/tests/app.rs: the_editor_stays | covered | pass |
| Hide a second editor | crates/mu-tui/tests/app.rs: hide_a_second_editor | covered | pass |
| Resize the notices | crates/mu-tui/tests/app.rs: resize_the_notices | covered | pass |
| A size of the first block | crates/mu-tui/tests/app.rs: a_size_of_the_first_block | covered | pass |
| List the plan | crates/mu-tui/tests/app.rs: list_the_plan | covered | pass |
| Insert in the middle | crates/mu-tui/tests/app.rs: insert_in_the_middle | covered | pass |
| Edit a line by its id | crates/mu-tui/tests/app.rs: edit_a_line_by_its_id | covered | pass |
| Delete a line that others need | crates/mu-tui/tests/app.rs: delete_a_line_that_others_need | covered | pass |
| A resize changes one line | crates/mu-config/tests/layout.rs: a_resize_changes_one_line | covered | pass |
| A hide of an anchor writes the plan again | crates/mu-config/tests/layout.rs: a_hide_of_an_anchor_writes_the_plan_again | covered | pass |
| A click on a title | crates/mu-tui/tests/app.rs: a_click_on_a_title | covered | pass |
| A drag to the left | crates/mu-tui/tests/app.rs: a_drag_to_the_left | covered | pass |
| A drag past the smallest pane | crates/mu-tui/tests/app.rs: a_drag_past_the_smallest_pane; a_drag_down_past_the_smallest_pane | covered | not-run |
| A drag down | crates/mu-tui/tests/app.rs: a_drag_down | covered | pass |
| A selection beside a divider | crates/mu-tui/tests/app.rs: a_selection_beside_a_divider | covered | pass |
| The letters | crates/mu-tui/tests/arrange.rs: the_letters | covered | pass |
| Typing in arrange mode | crates/mu-tui/tests/arrange.rs: typing_in_arrange_mode | covered | pass |
| The tabs go below the output | crates/mu-tui/tests/arrange.rs: the_tabs_go_below_the_output | covered | pass |
| A refused stack | crates/mu-tui/tests/arrange.rs: a_refused_stack | covered | pass |
| Add the statusline | crates/mu-tui/tests/arrange.rs: add_the_statusline | covered | pass |
| Undo | crates/mu-tui/tests/arrange.rs: undo | covered | pass |
| A letter bound to undo still picks | crates/mu-tui/tests/arrange.rs: a_letter_bound_to_undo_still_picks | covered | pass |
| A drag to the right edge | crates/mu-tui/tests/arrange.rs: a_drag_to_the_right_edge | covered | pass |
| Enter with no change | crates/mu-tui/tests/arrange.rs: enter_with_no_change_saves_nothing | covered | pass |
| Esc restores | crates/mu-tui/tests/arrange.rs: esc_restores | covered | pass |
| A save from a built-in layout | crates/mu-cli/tests/client.rs: a_save_from_a_built_in_layout | covered | pass |
| A save to a player layout | crates/mu-cli/tests/client.rs: a_save_to_a_player_layout | covered | pass |
| A second client gets the change | crates/mu-cli/tests/client.rs: a_second_client_gets_the_change | covered | pass |
| The server refuses | crates/mu-cli/tests/client.rs: the_server_refuses | covered | pass |
| Use a built-in layout | crates/mu-cli/tests/client.rs: use_a_built_in_layout | covered | pass |
| Save under a name | crates/mu-cli/tests/client.rs: save_under_a_name | covered | pass |
| A save over a built-in name | crates/mu-cli/tests/client.rs: a_save_over_a_built_in_name | covered | pass |
| A save from the built-in layout of one world | crates/mu-cli/tests/client.rs: a_save_from_the_built_in_layout_of_one_world | covered | pass |
| The other world keeps its layout | crates/mu-cli/tests/client.rs: the_other_world_keeps_its_layout | covered | pass |
| A name that is too long | crates/mu-cli/tests/client.rs: a_name_that_is_too_long | covered | pass |
| A device with a built-in name | crates/mu-cli/tests/client.rs: a_device_with_a_built_in_name | covered | pass |
| A layout for this world | crates/mu-cli/tests/client.rs: a_layout_for_this_world | covered | pass |
| Remove the layout of this world | crates/mu-cli/tests/client.rs: remove_the_layout_of_this_world | covered | pass |
| An unknown layout for this world | crates/mu-cli/tests/client.rs: an_unknown_layout_for_this_world | covered | pass |
| A use behind an entry | crates/mu-cli/tests/client.rs: a_use_behind_an_entry | covered | pass |
| The next layout of a world entry | crates/mu-cli/tests/client.rs: the_next_layout_of_a_world_entry | covered | pass |
| The default layout puts the tabs first | crates/mu-cli/tests/client.rs: the_default_layout_puts_the_tabs_first | covered | pass |

### Not run

- An unknown field: no verification run names this row
- The editor after the notices: no verification run names this row
- The editor before the notices: no verification run names this row
- A block beside two rows: no verification run names this row
- An id in a later line: no verification run names this row
- The default layout: no verification run names this row
- A label too long: no verification run names this row
- No world in focus: no verification run names this row
- No worlds: no verification run names this row
- Under an hour: no verification run names this row
- Over an hour: no verification run names this row
- A drag past the smallest pane: no verification run names this row

## Verification runs

- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_server_table)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_unknown_field)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_unknown_label_field)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=not_toml)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_file_that_cannot_be_read)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=the_three_new_fields)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_layout_table)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_unknown_field_of_tabs)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_world_layouts_table)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=no_tabs)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_editor_alone)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=nine_outputs)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=one_output_shows_the_notices_too)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_output_for_the_notices)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_output_with_no_stream_yet)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_refused_line_goes_to_the_notices_block)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_refused_line_of_another_world_waits_in_its_notices)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=enter_with_no_world_goes_to_the_notices_block)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_output_that_comes_later_starts_with_new_lines)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_notice_shows_under_the_world_in_focus_only)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=two_editors_keep_two_texts)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=enter_sends_the_editor_with_the_input_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=one_text_for_each_world)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=a_second_editor_with_no_id)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=five_editors)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_key_moves_the_input_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_click_moves_the_input_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=one_editor)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_input_focus_after_a_change_of_the_world)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_output_above_the_tabs)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_block_beside_another)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=the_numbers_of_the_lines)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=the_editor_after_the_notices)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_editor_after_the_notices_gets_its_rows)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=the_editor_before_the_notices)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_editor_before_the_notices_gets_its_columns)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_block_beside_two_rows_gets_its_columns)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=a_block_beside_two_rows)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=anchors_that_are_not_neighbors)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=an_id_in_a_later_line)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=an_id_in_a_later_line)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=an_id_twice)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=an_id_that_is_a_word_of_the_sentences)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_statusline_at_the_bottom)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=notices_behind_the_output)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=the_editor_does_not_stack)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_percent)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_count_of_cells)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=a_size_on_the_tabs)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=a_size_on_the_first_line)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_editor_stops_at_its_max)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_default_max)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=two_editors_side_by_side)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_shorter_max_does_not_stop_the_taller_editor)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=an_unknown_block)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=a_block_twice)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=a_block_that_is_not_placed_yet)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=no_editor)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=a_percent_out_of_range)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=too_many_lines)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=any_layout_reads_back)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=each_built_in_layout_is_valid)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=the_default_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_sidebar_layout)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=two_text_panes_have_title_rows)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=one_text_pane_has_no_title_row)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_percent_in_a_narrow_terminal)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_reload_changes_the_layout)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_layout_with_no_editor_from_the_server)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_label_with_the_number)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_label_keeps_the_unseen_mark)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_game_and_the_character)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=braces)'`
- exit 0: `cargo nextest run -p mu-config --test layer --locked -E 'test(=an_open_brace)'`
- exit 0: `cargo nextest run -p mu-config --test layer --locked -E 'test(=a_label_too_long)'`
- exit 0: `cargo nextest run -p mu-config --test layer --locked -E 'test(=a_label_of_64_characters)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_world_with_its_own_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_focus_moves_to_a_world_with_no_entry)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_layout_for_a_game)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_world_entry_comes_before_the_game_entry)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=no_world_in_focus_takes_the_default_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=no_world_in_focus_for_layout_here)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_world_keeps_its_text_across_a_change_of_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_default_segments)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_narrow_statusline)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_typed_count)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_state_after_a_disconnect)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_idle_time_after_a_send)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_connected_time)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=no_connected_time_while_disconnected)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=no_world_in_the_statusline)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=no_worlds)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_time_under_an_hour)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_time_one_second_under_an_hour)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_time_over_an_hour)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_time_of_one_hour)'`
- exit 0: `cargo nextest run -p mu-tui --test update --locked -E 'test(=a_tick_moves_the_idle_time)'`
- exit 0: `cargo nextest run -p mu-tui --test update --locked -E 'test(=no_tick_without_a_time_segment)'`
- exit 0: `cargo nextest run -p mu-config --test layer --locked -E 'test(=an_unknown_segment)'`
- exit 0: `cargo nextest run -p mu-config --test layer --locked -E 'test(=a_segment_twice)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_device_sets_the_left_list)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_layout_file_in_use)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_bad_layout_file_at_start)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_file_with_a_built_in_name)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_unknown_field_of_a_layout_file)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=the_device_names_another_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_layout_that_does_not_exist)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_change_to_a_layout_file_changes_the_revision)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=the_list_with_one_player_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=the_list_with_a_bad_layout_file)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_save_writes_the_file)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_save_of_a_bad_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_save_with_a_built_in_name)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_save_with_a_bad_name)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_save_with_the_name_of_two_dots)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_save_through_a_linked_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_clients_path_that_is_a_file)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_device_file_with_a_comment)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=no_device_file)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=an_unknown_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_bad_device_layer)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_use_from_the_command_line)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=the_picture_of_classic)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=an_open_socket_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=no_server)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=an_edit_places_a_block)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=an_edit_gives_the_same_file_as_the_command)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=an_insert_from_the_command_line)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_delete_by_id)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=an_edit_of_a_built_in_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_bad_edit)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=describe_gives_each_block)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=describe_of_an_unknown_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_device_adds_an_entry)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_device_replaces_an_entry)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_entry_for_a_world_that_the_server_does_not_have)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_change_to_the_layout_file_of_an_entry_changes_the_revision)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_key_with_two_slashes)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_entry_with_a_layout_that_does_not_exist)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_use_for_one_world)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_use_for_a_game)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=forget_the_layout_of_a_world)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_use_with_a_bad_world_id)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=an_edit_of_the_built_in_layout_of_one_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=add_the_notices)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=move_the_tabs)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_bad_sentence)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=hide_the_tabs)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_editor_stays)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=hide_a_second_editor)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=resize_the_notices)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_size_of_the_first_block)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=list_the_plan)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=insert_in_the_middle)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=edit_a_line_by_its_id)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=delete_a_line_that_others_need)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=a_resize_changes_one_line)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=a_hide_of_an_anchor_writes_the_plan_again)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_click_on_a_title)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_drag_to_the_left)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_drag_past_the_smallest_pane)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_drag_down_past_the_smallest_pane)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_drag_down)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_selection_beside_a_divider)'`
- exit 0: `cargo nextest run -p mu-tui --test arrange --locked -E 'test(=the_letters)'`
- exit 0: `cargo nextest run -p mu-tui --test arrange --locked -E 'test(=typing_in_arrange_mode)'`
- exit 0: `cargo nextest run -p mu-tui --test arrange --locked -E 'test(=the_tabs_go_below_the_output)'`
- exit 0: `cargo nextest run -p mu-tui --test arrange --locked -E 'test(=a_refused_stack)'`
- exit 0: `cargo nextest run -p mu-tui --test arrange --locked -E 'test(=add_the_statusline)'`
- exit 0: `cargo nextest run -p mu-tui --test arrange --locked -E 'test(=undo)'`
- exit 0: `cargo nextest run -p mu-tui --test arrange --locked -E 'test(=a_letter_bound_to_undo_still_picks)'`
- exit 0: `cargo nextest run -p mu-tui --test arrange --locked -E 'test(=a_drag_to_the_right_edge)'`
- exit 0: `cargo nextest run -p mu-tui --test arrange --locked -E 'test(=enter_with_no_change_saves_nothing)'`
- exit 0: `cargo nextest run -p mu-tui --test arrange --locked -E 'test(=esc_restores)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_save_from_a_built_in_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_save_to_a_player_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_second_client_gets_the_change)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_server_refuses)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=use_a_built_in_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=save_under_a_name)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_save_over_a_built_in_name)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_save_from_the_built_in_layout_of_one_world)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_other_world_keeps_its_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_name_that_is_too_long)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_device_with_a_built_in_name)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_layout_for_this_world)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=remove_the_layout_of_this_world)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=an_unknown_layout_for_this_world)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_use_behind_an_entry)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_next_layout_of_a_world_entry)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_default_layout_puts_the_tabs_first)'`

## Dropped

None.

## Lens notes

- verify: R-130: design.md D11 now decides that a stand-in main output shows the pane of the world and keeps its own pane state off the screen. specs/client-layout/spec.md line 58 states the same rule, and the scenario at line 71 checks it. Model::pane_of in crates/mu-tui/src/model.rs gives the world pane to the main output and reads extra_panes for any other output. No code path removes an entry of extra_panes, so the earlier lines come back when main returns.
- verify: The test the_earlier_lines_of_a_stand_in_come_back_with_main in crates/mu-tui/tests/app.rs 3487-3503 checks each THEN step of the new scenario on the drawn rows of the block notices.
- verify: The review.md ranges of diff-hunks.txt and the fix_hunks entry review.md 635 8 were not read, because the agent rules forbid reading review.md.
- verify: Requirement 'An output shows the lines of its id' (spec line 30) says an output shows the stream with its id. While notices stands in for main, its earlier notices lines do not show. The rule at spec line 58 states this exception, so no finding is raised. A spec lens may still want the exception named in line 30.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | fixed_after | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D11. Streams, and what #102 adds"],"The model keeps one pane state for each pair of a world and a stream, and one for the stream `notices`."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  | severity? should-fix 0.70 | app-1: No decision says how the first output draws, scrolls and selects over several streams in the order of arrival. D11 keeps one PaneState for each stream, and a PaneState holds its own line store, scroll anchor, selection and drawn rows, so two of them cannot draw as one merged pane.; fit-2: D11 makes the notices a stream with its own pane state, but the model already keeps the notices a second time in Model::notices and replays them into a world pane; the design does not say that the vector and the replay go away. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D8. Layer 3: the write calls"],"A layout file is written whole to a temporary file in `clients/layouts/`, and then renamed over the target with `tempfile`."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The scenarios of a save into a directory with no `clients` directory need the server to make `clients/layouts/` and `clients/devices/`. The write path of D8 writes a temporary file in `clients/layouts/` and has no step that makes a missing directory. |
| R-3 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu layout prints layouts"],"If the name is of a bad layout file, it SHALL write the notice of the file to stderr and exit with status 1."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | Each scenario of a bad layout file that runs `mu layout show mine` needs the notice of that file on stderr. Task 3.9 builds show over ListLayouts, which the spec limits to good files, and no decision adds a call or a field that returns the notice of a bad layout file. |
| R-4 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D6. Layer 2: how the client draws"],"The solver is a pure function of `mu-config`, `solve(layout, area, editor rows) -> Vec<Placed>`."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | No decision sizes a part that holds several editors side by side. D6 keeps the editor rule of today, which sizes one editor leaf in a split of rows and gives Min(0) in a split of columns, and its solver takes one input of editor rows, while each editor wraps at its own width. |
| R-5 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: A bad tab label makes its layer bad"],"The layer SHALL be bad, and its notice SHALL name the file and `tabs.label`, in each of these cases:"] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The requirement of a bad tab label has no decision. D5 adds only `layout` and `world_layouts` to ClientLayer, and the layer check in state() checks only [keys], so nothing in the design makes a bad `tabs.label` make its layer bad on the server. |
| R-6 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-statusline/spec.md","Requirement: A bad statusline makes its layer bad"],"The layer SHALL be bad, and its notice SHALL name the file and the field, in each of these cases:"] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The requirements of a bad statusline and of the list merge have no decision. D5 names no [statusline] table of ClientLayer, no field paths `statusline.left` and `statusline.right`, and no check of the segments in state(). |
| R-7 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Requirement: The layout here command sets the layout of one world"],"If an entry is the choice in use, `/layout use` SHALL also show a notice that names `/layout here`."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  | severity? should-fix 0.84 | No decision produces the notice of `/layout use` when a world entry is the choice in use. D9 names the save rule, `layout-here` and `layout-next`, and says nothing of `/layout use` behind an entry. |
| R-8 | 1 | 6 | blocker | in-change | drift | ["artifact",["design.md","D5. Layer 2: layouts on the server"],"A key is a world id or a game name, and `mu-config` checks it with the rule of the world id."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  | severity? should-fix 0.68 | fit-3: The plan adds a third parser of the world id in mu-config (the key check of D5, the game of layout_for, the game and character of the label and of save_target), beside WorldId in mu-core and the split in mu-tui; the design names no shared one.; app-4: D5 has mu-config check a key of [world-layouts] with the rule of the world id. That rule is pub(crate) in mu-core, and the crate rules give mu-config no member to depend on, so mu-config cannot call it. A second copy can refuse a key for a world that the server accepts.; inv-3: D5 checks a key of `[world-layouts]` with the rule of the world id, but that rule refuses a game name such as `alpha`, and mu-config may depend on no crate of the workspace, so it cannot call it. |
| R-9 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: An id in a later line"],"`output as side right of output` and `tabs above side`"] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  | severity? should-fix 0.77 | tes-1: The scenario 'An id in a later line' (tabs above an id defined by an earlier line) has no task and no named test.; spe-2: The THEN step names `notices`, but the layout of the scenario gives that block no line; its only output is `side`, so the step cannot be checked. |
| R-10 | 1 | 2 | should-fix | deviates-design | wrong-behaviour | ["artifact",["design.md","Migration Plan"],"None. A configuration directory with no new field gives the screen of today."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The client writes `layout` and `[world-layouts]` into device layers and adds `clients/layouts/`. A rollback of the binary reads them with deny_unknown_fields and the old layer rule, so the device layer turns bad and drops its echo and keys, and each layout file gives a notice. The plan has no rollback step. |
| R-11 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D11. Streams, and what #102 adds"],"The first `output` merges its streams in the order of arrival, which is the screen of today."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | D11 calls the merge of one global stream `notices` into the first output the screen of today. At head a notice goes into the pane of the world in focus only, so it does not show under another world. A global notices stream shows every notice under every world. |
| R-12 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. Layer 3: the write calls"],"The writes run one at a time behind the lock that the reload uses."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  | severity? blocker 0.93 | D8 runs each write behind the lock that the reload uses and then runs the reload of ReloadConfig. At head Store::reload takes that same tokio Mutex itself, which is not reentrant, so the write path as written waits on itself and the save never returns. |
| R-13 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","D7a. Layer 2: several editors"],"`Model` gets `input: BlockId`, the editor with the input focus."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | D7a gives each editor a block id and an input field of type BlockId, but mu-tui already names editors with EditorId and keeps non-pose editors in Model::editors; the design keeps both names for one editor. |
| R-14 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. Layer 1: the plan is the layout"],"The anchor side never has a size, so each split always has a part that shares the rest."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | D2 says each split keeps a part that shares the rest, but nothing stops the sizes of one split from passing its width: two 60% parts, or a count of 500 rows, leave the anchor side a negative rest. |
| R-15 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: The tab label"],"The table `[tabs]` SHALL hold `label`, a template of 1 to 64 characters."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The tab label limit of 64 characters is tested only at 65; no test shows that 64 characters are good, or that 64 characters of more than one byte each are good. |
| R-16 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Scenario: A name that is too long"],"the device name of the client has 60 characters"] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The limit of 64 characters on a saved name is tested only with a name of 71 characters; no test passes a name of exactly 64 (good) or 65 (bad) to `save_target`. |
| R-17 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-statusline/spec.md","Requirement: The format of a time"],"A time SHALL show as `M:SS` under one hour and as `H:MM:SS` from one hour."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The switch from `M:SS` to `H:MM:SS` at one hour is tested at 75 and 3725 seconds only; neither 3599 nor 3600 seconds is tested. |
| R-18 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Requirement: A drag of a title row resizes two panes"],"Neither pane SHALL get smaller than 2 rows."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The floor of 2 rows for a drag of a title row has no test: `a_drag_down` moves 4 rows of a 50% pane and stays inside the range. |
| R-19 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/layout-editing/spec.md","Requirement: Arrange mode moves a block with three keys"],"The keys of the mode that are not letters leave each letter free for a block."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | Arrange mode reads the undo key from the keymap, and `[keys]` may bind a plain lowercase letter, so a player who binds `a` to `undo` loses either the pick of block A or the undo. Nothing keeps the letters free. |
| R-20 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-config/src/names.rs",145] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | `save_target` names a save after the device, but the device name rule accepts `classic`, `focus`, `status`, `sidebar`, `.` and `..`, which the layout name check refuses, so every save on such a device fails. |
| R-21 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: Each block has a stable id"],"An id SHALL be 1 to 32 characters from `a` to `z`, `0` to `9` and `-`."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The spec bounds an id to 32 characters, but the grammar of D4 has no length, and no task tests an id of 32 or 33 characters. |
| R-22 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Layer 1: the changes"],"Finds the placed block beside the border, and sets its size to a whole percent within the smallest sizes."] | fixed | 84d7e01f4efbcab51679c61d4cb9a4184697c852 | 2 |  |  | A drag or a `<`/`>` key can ask for a percent outside 10% to 90%: in a 300-column split the 10-column floor is 3%, and `>` from 90% gives 95%. D3 clamps only to the smallest pane, so the check refuses the result mid-drag. |
| R-23 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: A percent out of range"],"holds the lines `editor` and `output above editor, 95%`"] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The percent range 10% to 90% is tested only at 95%; no test passes 9%, 90% or 91% to the reader. |
| R-24 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: The blocks of the screen"],"A layout SHALL hold `editor` one to four times, `output` at most eight times, and each other block at most once."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The block counts are tested only one past each limit (five editors, nine outputs); no test shows that four editors and eight outputs are good. |
| R-25 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: The editor grows with its text"],"A sentence SHALL set the most rows with `max N` after a comma, from 1 to 100."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The editor `max` from 1 to 100 and a count of cells from 1 have one test, `a_bad_editor_size`; no test passes `max 0`, `max 1`, `max 100`, `max 101` or a count of 0. |
| R-26 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["specs/client-layout/spec.md","Scenario: Two editors keep two texts"],"the user types `one`, gives the input focus to `scratch`, and types `two`"] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | A WHEN step gives the input focus to `scratch` with no key or click named, so the step names no concrete input. |
| R-27 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["specs/config-store/spec.md","Scenario: Describe gives each block"],"the entry of `notices` has the width 24, and the entry of `tabs` has the width 80 and the height 1"] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The width of `notices` in the describe scenario (24) does not agree with the divider in column 56 of the sidebar scenario, which gives 23 columns for the pane beside it; the rectangle rule does not say whether the divider counts. |
| R-28 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["specs/client-layout/spec.md","Scenario: A block beside two rows"],"the notices take the right part of rows 2 to 23, and the editor is in the left part of row 23"] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The THEN step places the notices and the editor by 'the right part' and 'the left part' with no column or row count, so it names no checkable value. |
| R-29 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","2.3"],"`the_order_of_the_lines_decides_the_shape`"] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | Two scenarios, 'The editor after the notices' and 'The editor before the notices', share one named test, the_order_of_the_lines_decides_the_shape, in task 2.3 (a task about errors); each THEN needs its own assertion. |
| R-30 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D12. What enforces each rule"],"with `toml_edit` and `tempfile` added to the row of `mu-server`"] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  | severity? should-fix 0.74 | D12 and task 4.1 add `tempfile` to the row of mu-server in workspace_boundary.rs, but the row already lists it. What changes is that mu-server takes tempfile in [dependencies], where today it is a dev-dependency. |
| R-31 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D9. Layer 4: commands and the mouse"],"A press that hits no divider and no title row goes to the pane under the mouse, as today."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | D9 says a press that hits no divider goes to the pane under the mouse, as today. At head a press, a drag and a release go to the pane of the world in focus, not to the pane under the mouse. |
| R-32 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D7. Layer 2: the streams, the tab label and the statusline"],"`update` returns `Effect::Tick` of one second. A flag stops a second tick."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The tick of D7 is a second wait effect with a second one-at-a-time flag, beside RetryConfig and ConfigRetry, which already wait and then give back a message. |
| R-33 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D3. Layer 1: the changes"],"Change only the line of the block, in its place."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | A screen change of step 1 of D3 writes one step back as a line of text, and `/layout edit` reads it again, but the two property tests of D4 work on whole trees and never compare one line with its parsed step. |
| R-34 | 1 | 2 | suggestion | in-change |  | ["artifact",["specs/client-layout/spec.md","Requirement: A small terminal keeps each pane usable"],"The client SHALL NOT stop in a terminal with less room."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | `solve` maps any plan and any terminal size to rectangles, and the spec says the client never stops in a small terminal, but only one narrow size of 40 columns is tested and D12 names no property for the solver. |
| R-35 | 1 | 1 | note | in-change |  | ["artifact",["tasks.md","Tasks: client-layout"],"The one-test command is in `openspec/config.yaml`."] |  |  |  |  |  | Most tasks say the tests pass but name no command; the command is given once in the task header (config.yaml). Rows are graded covered on that basis. |
| R-36 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An output that comes later starts with new lines"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 2 |  |  | delta scenario 'An output that comes later starts with new lines' has no matrix row |
| R-37 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A notice shows under the world in focus only"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 2 |  |  | delta scenario 'A notice shows under the world in focus only' has no matrix row |
| R-38 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Two editors side by side"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 2 |  |  | delta scenario 'Two editors side by side' has no matrix row |
| R-39 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The shorter max does not stop the taller editor"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 2 |  |  | delta scenario 'The shorter max does not stop the taller editor' has no matrix row |
| R-40 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The list with a bad layout file"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 2 |  |  | delta scenario 'The list with a bad layout file' has no matrix row |
| R-41 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A clients path that is a file"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 2 |  |  | delta scenario 'A clients path that is a file' has no matrix row |
| R-42 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A letter bound to undo still picks"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 2 |  |  | delta scenario 'A letter bound to undo still picks' has no matrix row |
| R-43 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A device with a built-in name"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 2 |  |  | delta scenario 'A device with a built-in name' has no matrix row |
| R-44 | 3 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Requirement: The layout commands choose and name a layout"],"`/layout list` SHALL show the name of each layout. The entry `layout-next` SHALL set the choice in use to the next layout in the order of the names"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | `/layout list` and `layout-next` need the names of every layout in the client, but no decision gives the client a ListLayouts call or an effect for it. GetClientConfigResponse carries only the layout in use and the world entries. |
| R-45 | 3 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Requirement: The pane commands hide and resize a block"],"For the block of the first line, it SHALL change nothing, and SHALL show a notice that names the blocks beside it that take a size."] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The THEN of 'A size of the first block' needs a notice that names `notices`, the sized block beside the first block. D3 refuses the plan through build and the check, whose error names only the line and the word, so no decision produces the names of the blocks beside it. |
| R-46 | 3 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Scenario: A drag down"],"the user drags the title row of `notices` down by 4 rows and releases"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? should-fix 0.69 | In 'A drag down' the notices take 12 of 24 rows; 4 rows less is 8 rows, 33.3%. A whole percent of 33 solves to 7 rows by the round-down rule of D6, so the pane is 5 rows smaller. Only rounding up to 34% gives 8 rows, and D3 names no rounding. |
| R-47 | 3 | 4 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D7. Layer 2: the streams, the tab label and the statusline"],"`Effect::RetryConfig` becomes `Effect::Wait { after, then }`, which carries the wait and the message to give back."] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | D7 makes the tick a fixed message `Msg::Tick(i64)` that `Effect::Wait` gives back, and names a clock read only for the first tick before the first draw. No decision says where a later tick gets the current time, so `now_ms` does not move and `idle` and `connected` do not count. |
| R-48 | 3 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Scenario: A file with a built-in name"],"stderr holds one line that contains `clients/layouts/classic.toml` and `built-in`"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? should-fix 0.66 | 'A file with a built-in name' needs a notice with `built-in` for `clients/layouts/classic.toml`. D5 checks the name of a layout file with the rule of a device name, which accepts `classic`, and the reader of D4 knows no file name; only `save_target` of D9 refuses a built-in name. |
| R-49 | 3 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Scenario: A save from the built-in layout of one world"],"it holds `layout = \\"classic\\"` with the source `default`"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? should-fix 0.71 | A save from a built-in layout behind a world entry must set the entry, not the field `layout`. SaveLayout with a device sets only `layout`, and D9 says only that `Effect::SaveLayout` makes 'the call', so no decision makes the client set the entry after the write. |
| R-50 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","3. Layer 2: named layouts"],"and that `the_first_view`, `the_order_of_the_worlds` and `no_worlds` in `client.rs` still pass."] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | Task 3.8 names `the_first_view`, `the_order_of_the_worlds` and `no_worlds` in `client.rs`. At head no test has the first two names, and `first_view` and `no_worlds` are in `crates/mu-tui/tests/app.rs`, so the check that `classic` keeps the screen of today names tests that cannot run. |
| R-51 | 3 | 5 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. Layer 2: layouts on the server"],"`layer_rule` gets this place, with the name rule of a device."] | resolved | resolved in round 5 from fixed aec3aa33ba5cb2104eb027818f8b69a34a70505f | 4 |  |  | The rule of a layout name has three statements: the store checks a layout file with the device name rule (D5), the write path adds a refusal of . and .. (D8), and save_target runs a check of a layout name with built-in names, . and .. (D9). No type holds the rule, so the reader accepts a file clients/layouts/..toml (name .) that the writer refuses. |
| R-52 | 3 | 4 | should-fix | in-change | drift | ["path","crates/mu-cli/src/config_verb.rs",65] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The verb layout must follow the private-socket rule of mu config, but the one function that does that check and call, call in config_verb.rs, is private to that verb, and no decision or task names it, so task 3.9 writes a second copy. |
| R-53 | 3 | 4 | should-fix | in-change | drift | ["artifact",["design.md","D3. Layer 1: the changes"],"The keys work out the new percent, keep it from 10 to 90, and then call `resize`."] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? suggestion 0.77 | The range of 10 to 90 percent for a resize is kept twice: in resize_border of mu-config, and in the keys < and > of arrange mode in mu-tui, which work out and clamp the percent themselves before they call resize. |
| R-54 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-cli/src/config_verb.rs",157] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | mu config show prints the keyed table [keys] under a table header, but the spec prints the keyed table [world-layouts] as dotted lines; D5 names neither the [keys] form nor a reason. render prints the bindings last, so a dotted line printed after the [keys] header reads as a binding. |
| R-55 | 3 | 4 | should-fix | in-change | next-change | ["path","openspec/changes/archive/2026-11-24-config-store/design.md",18] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The store has one check of a client layer, state(), and config-store said a write runs that check before it lands. D8 checks the parts of the request and the old device file, but not the edited device text, and names neither state() nor a reason, so #173 must change the write path. |
| R-56 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7a. Layer 2: several editors"],"`apply_config` and each change of the layout set `input` to the first editor of the plan if the old one is gone."] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The input focus stays an editor of the layout on the screen only at apply_config and at a layout change; a change of the world focus to a world with another layout (D6) passes neither, so `input` can name an editor that is not on the screen. |
| R-57 | 3 | 10 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D11. Streams, and what #102 adds"],"gives the id of the `output` for a stream: the `output` with that id, or the first `output` of the plan"] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  |  | `route` has no answer for a plan with no `output`, which the spec allows (the layout `editor` alone); no decision says where a world line or a notice goes then, and the two route tests cover only plans with an output. |
| R-58 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Layer 1: the changes"],"It keeps the percent from 10 to 90, and each pane at its smallest size or more."] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? blocker 0.70 | `resize_border` keeps the percent from 10 to 90 and each pane at its smallest size, but in a split of 20 columns or less, or 3 rows or less, the two ranges cross (probe: W=20 needs p>=50 and p<=49); D3 states no rule for that case, and the only edge test uses 300 columns. |
| R-59 | 3 | 5 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Layer 1: the changes"],"\| `insert(number, step)` \| Puts a line at that number. \|"] | resolved | resolved in round 5 from fixed aec3aa33ba5cb2104eb027818f8b69a34a70505f | 4 |  |  | The line edits take a line number from a command or an EditLayout call, but D3 states no range for it, and every named test uses a number in the middle; 0, n+1 for replace and delete, and n+2 for insert are not tested (a direct Vec::insert past the end panics). |
| R-60 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: Too many lines"],"stderr holds one line that contains `clients/layouts/mine.toml` and `32`"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The limit of 32 lines can never be the binding rule: the block counts cap a valid plan at 14 lines (4 editors, 8 outputs, tabs, statusline), so a plan of 33 lines also breaks a block count, and `too_many_lines` names `32` only if the count runs before `build`, which D4 does not order. |
| R-61 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: A bad sentence makes its layout bad"],"more than 32 lines, or more than 8 levels of splits"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The limit of 8 levels of splits has one test, `a_tree_too_deep`, with no stated depth; no test shows that a tree of exactly 8 levels is good, and no decision says how a level is counted, so 8 and 9 can each land on either side. |
| R-62 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-layout/spec.md","Requirement: The size of a block"],"or a whole count of cells from 1"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | A count of cells has no upper bound in the spec or the grammar, and D6 adds counts in the solver; no test passes a count at or past the top of the integer type to the reader, or two such counts in one split to `solve`, where a debug build panics on overflow and a release build wraps. |
| R-63 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7. Layer 2: the streams, the tab label and the statusline"],"If the layout holds `statusline`, and its segments hold `connected` or `idle`, `update` returns a wait of one second."] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The tick starts once before the first draw and continues while the layout has a time segment; no test covers a model that gains a time segment later (a reload to `status`, `/layout use status`, or a focus change to a world entry of `status`), where a missing wait leaves the clock frozen. |
| R-64 | 3 | 4 | should-fix | in-change | false-test | ["artifact",["specs/client-layout/spec.md","Scenario: The editor after the notices"],"the editor is below the output only, and the notices take the whole height"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | THEN step 'the notices take the whole height' names no row count and the GIVEN and WHEN give no terminal size, so no test can check the screen it describes. |
| R-65 | 3 | 4 | should-fix | in-change | false-test | ["artifact",["specs/client-layout/spec.md","Scenario: The editor before the notices"],"the editor takes the whole width, and the notices are beside the output only"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | THEN step 'the editor takes the whole width' names no column count and the scenario gives no terminal size, so the screen it describes cannot be checked. |
| R-66 | 3 | 4 | should-fix | in-change | false-test | ["artifact",["tasks.md","2. Layer 1: blocks and the layout model"],"Each of the three tests of a shape asserts the tree that `build` gives."] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The scenario 'A block beside two rows' checks columns 55 to 80, a divider in column 54 and the editor in columns 1 to 53. Its only test, in task 2.3, asserts the tree that build gives, which holds no column. |
| R-67 | 3 | 4 | suggestion | in-change |  | ["path","crates/mu-server/src/store.rs",327] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | resolve builds the field paths of [keys] with one block for that table. [world-layouts] merges by key the same way, so the block repeats with another table name and a quoted key. |
| R-68 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","D8. Layer 3: the write calls"],"The change sets the field `layout`, or sets or removes one entry of `[world-layouts]`."] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The device edit of D8 knows two changes only, the field layout and one entry of [world-layouts]. #173 writes keys and echo into the same device file, so it must widen or wrap this edit. |
| R-69 | 3 | 4 | suggestion | in-change |  | ["path","crates/mu-tui/src/registry.rs",261] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? should-fix 0.78 | The registry lists the values of an argument by its kind in candidates(). D9 gives the new commands one free-text kind, so a block id or a layout name has no candidates, and the palette of #167 must change the kinds of these entries. |
| R-70 | 3 | 4 | suggestion | in-change |  | ["path","crates/mu-config/src/names.rs",22] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? should-fix 0.60 | mu-config writes its errors by hand with Display and Error, and depends only on serde. D5 adds thiserror to mu-config for the one moved error type, and names neither the pattern of the crate nor a reason to leave it. |
| R-71 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","D9. Layer 4: commands and the mouse"],"It runs the check of a layout name on that name"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | `save_target` must never give a name that the store refuses, over any device name and any world key; only example tests at 64 and 65 characters, a built-in name and `.` check it. |
| R-72 | 3 | 4 | suggestion | in-change |  | ["artifact",["specs/config-store/spec.md","Requirement: UseLayout sets the layout of a device"],"and SHALL keep each other line of the file, its comments included"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The device write keeps each other line of the file over an unbounded domain of device files, and only two example scenarios with one comment check it. |
| R-73 | 4 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The input focus after a change of the world"] | accepted | ticket: none; reason: Task 3.5a of tasks.md names the test the_input_focus_after_a_change_of_the_world for this scenario. Only a full round writes a matrix row, and the operator chose no further full round. |  |  |  | delta scenario 'The input focus after a change of the world' has no matrix row |
| R-74 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Requirement: The pane commands hide and resize a block"],"`/pane size <id> <size>` SHALL set the size on the line of the block, and a size of `fill` SHALL remove it."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | No decision produces `/pane size <id> fill`. D3 lists no fill operation, so the client adds Change::Fill on its own, while EditLayout parses the size with Size::parse, which refuses `fill`. `mu layout size <name> <id> fill` fails where the client command of the same name works. |
| R-75 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D10. Layer 5: arrange mode"],"Enter drops the mode and returns `Effect::SaveLayout`."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | Enter in arrange mode saves nothing when the layout equals the layout from before the mode. D10 says Enter returns Effect::SaveLayout, and the spec says Enter saves; design.md records no deviation. |
| R-76 | 6 | 6 | should-fix | in-change | drift | ["artifact",["design.md","D9. Layer 4: commands and the mouse"],"The save rule of the spec is one function of `mu-config`, `save_target(choice in use, layout name, device name)`."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | fit-2: D9 makes the save rule one function, save_target(choice, layout name, device). The code drops the layout name, so the client and EditLayout each write their own branch for a built-in layout around it.; app-3: D9 cites save_target with three inputs that gives the name and the entry or field. At head it takes the choice and the device only, and returns the LayoutName alone. |
| R-77 | 6 | 6 | should-fix | in-change | drift | ["path","crates/mu-tui/src/layouts.rs",138] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | Two places turn the text of a size into a Change, and they already differ. `/pane size <id> fill` gives Change::Fill. EditLayout runs Size::parse alone, which refuses `fill`, so `mu layout size` cannot do the same change. |
| R-78 | 6 | 6 | should-fix | in-change | drift | ["path","crates/mu-cli/src/layout_verb.rs",287] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | `mu layout show` writes the path of a layout file by hand to find its notice. mu-server already exports layout_path, and mu-cli depends on mu-server. |
| R-79 | 6 | 6 | should-fix | in-change | drift | ["path","crates/mu-tui/src/layouts.rs",11] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | The notice that mu wrote the whole plan again is written twice, once in the client and once in `mu layout`. D3 gives the flag one meaning for every caller. |
| R-80 | 6 | 6 | should-fix | in-change | drift | ["path","crates/mu-server/src/store.rs",220] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | The new DEVICES_DIR repeats the literal that PLACES already holds for the device layers. The write path uses the constant and the read path uses PLACES, so the two can name different directories. |
| R-81 | 6 | 6 | should-fix | in-change | drift | ["path","crates/mu-server/src/store.rs",240] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  | severity? suggestion 0.69 | layout_file repeats the chain in layer_rule that takes a file name out of a path under one directory: strip the dir, strip '/', strip '.toml', and refuse a '/'. |
| R-82 | 6 | 6 | should-fix | in-change | drift | ["path","crates/mu-config/src/names.rs",153] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | LayoutName::new calls DeviceName::new, then throws its error away and writes out the character rule of a device name a second time. |
| R-83 | 6 | 6 | should-fix | in-change | drift | ["path","crates/mu-tui/src/view.rs",127] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  | severity? suggestion 0.68 | The rule of the block in front of a stack is read twice. view::front and pane_next each look up `fronts` by the first id and fall back to the first block. |
| R-84 | 6 | 6 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-config/src/layout/change.rs",81] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | percent_range caps the high bound at floor((total-least-divider)*100/total), which refuses whole percents that keep the anchor side at its smallest size. Because the solver rounds a percent down, a split of 21 columns has the good percents 48 to 52, yet the code gives no range and the drag changes nothing. At 80 columns the cap is 86, not 87. In a split of 4 rows it is 50, not 74. The only boundary tests use 20 columns and 3 rows, where no percent exists, so nothing tests one past the limit. |
| R-85 | 6 | 6 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/arrange.rs",278] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | The test for a letter bound to undo asserts the plan lines, not the drawn row that the THEN step names. |
| R-86 | 6 | 6 | suggestion | in-change |  | ["artifact",["design.md","D5. Layer 2: layouts on the server"],"That block becomes one function with the name of the table as its parameter, and it quotes a key as TOML needs."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | D5 says the shared function of the keyed paths quotes a key as TOML needs. At head keyed_path joins the table and the key with no quote; the quote is added by key_part in mu-cli at render. |
| R-87 | 6 | 6 | suggestion | in-change |  | ["artifact",["design.md","D8. Layer 3: the write calls"],"The data source is a fixed text from the catalog, such as `the journal of the world in focus` for `output`."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  | severity? should-fix 0.69 | D8 says the data source of DescribeLayout is a fixed text of the catalog, with `the journal of the world in focus` for output. At head BLOCKS holds no such text; describe() builds it per step, and the output text differs. |
| R-88 | 6 | 6 | suggestion | in-change |  | ["artifact",["design.md","D5. Layer 2: layouts on the server"],"`mu-config` gets `layout_for(world, entries, default)`: the entry of the world id, then the entry of the game, then the default."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  | scope? deviates-design 0.60 | D5 cites layout_for with a default parameter. At head it takes the world and the entries, and returns only the Choice; the caller looks up the layout. |
| R-89 | 6 | 6 | suggestion | in-change |  | ["artifact",["design.md","D7. Layer 2: the streams, the tab label and the statusline"],"The pane states of D11 replace the map of panes by `PaneId`."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  | severity? should-fix 0.63 | D7 and D11 say a map keyed by a world and an output id replaces the map of panes by PaneId. At head the map by PaneId stays, the first output uses the pane of the world, and only the other outputs are keyed by world and id. |
| R-90 | 6 | 6 | suggestion | in-change |  | ["artifact",["design.md","D7a. Layer 2: several editors"],"`Model::input()` resolves the stored id against `Model::layout()` at each read."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  | severity? should-fix 0.70 | D7a names Model::input() as the function that resolves the stored editor id. At head that is Model::input_id(); Model::input() returns the text of the editor. |
| R-91 | 6 | 6 | suggestion | in-change |  | ["artifact",["design.md","D6. Layer 2: how the client draws"],"The solver is a pure function of `mu-config`, `solve(layout, area, rows_of) -> Vec<Placed>`."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | D6 says solve returns Vec<Placed>. At head it returns Solved, which also holds the dividers and the borders that the drag and arrange mode use. |
| R-92 | 6 | 6 | suggestion | in-change |  | ["artifact",["design.md","D3. Layer 1: the changes"],"`resize_by(id, steps)`"] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  | scope? deviates-design 0.70 | D3 cites resize_by(id, steps). At head it also takes the cells of the block and the total of its split, which the percent range needs. |
| R-93 | 6 | 6 | suggestion | in-change |  | ["artifact",["design.md","D5. Layer 2: layouts on the server"],"`GetClientConfigResponse` gets `repeated WorldLayout world_layouts`, each with a key, a name and its sentences."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  | severity? blocker 0.72 | D5 names the message of a world entry WorldLayout. At head the field is `repeated NamedLayout world_layouts`, the message that ListLayouts also uses. |
| R-94 | 6 | 6 | suggestion | in-change |  | ["path","crates/mu-tui/src/statusline.rs",63] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  | severity? should-fix 0.72 | The statusline cuts text to a width with two loops by graphemes, one from the end and one from the start. D7 says the cut uses the functions of width.rs, which gets none. |
| R-95 | 6 | 6 | suggestion | in-change |  | ["artifact",["specs/client-layout/spec.md","Requirement: A small terminal keeps each pane usable"],"A pane of `output` SHALL keep at least 2 rows and 10 columns while the terminal has room for each pane at that size."] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | The floor of 2 rows and 10 columns for an output pane ranges over any plan and any terminal size that has room. The property test solve_stays_inside_the_area checks only containment and overlap, and never the floor, so a solver change that drops the raise to the smallest size passes it. |
| R-96 | 6 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"An unknown field"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command | 6 |  |  | covered row 'An unknown field' has no verification run |
| R-97 | 6 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"The editor after the notices"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command | 6 |  |  | covered row 'The editor after the notices' has no verification run |
| R-98 | 6 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"The editor before the notices"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command | 6 |  |  | covered row 'The editor before the notices' has no verification run |
| R-99 | 6 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"No world in focus"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command | 6 |  |  | covered row 'No world in focus' has no verification run |
| R-100 | 6 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"Under an hour"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command | 6 |  |  | covered row 'Under an hour' has no verification run |
| R-101 | 6 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"Over an hour"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command | 6 |  |  | covered row 'Over an hour' has no verification run |
| R-102 | 6 | 7 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"approach partial"] | accepted | ticket: none; reason: the lens sampled large files in two full rounds, with a different sample each time; the operator accepted the gap, and the suite review of task 7.3 reads the branch again | 6 |  |  | lens approach reported partial, gaps: 3 |
| R-103 | 6 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"test-honesty partial"] | accepted | ticket: none; reason: the lens sampled large files in two full rounds, with a different sample each time; the operator accepted the gap, and the suite review of task 7.3 reads the branch again | 6 |  |  | lens test-honesty reported partial, gaps: 2 |
| R-104 | 6 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"invariants partial"] | accepted | ticket: none; reason: the lens sampled large files in two full rounds, with a different sample each time; the operator accepted the gap, and the suite review of task 7.3 reads the branch again | 6 |  |  | lens invariants reported partial, gaps: 2 |
| R-105 | 7 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D11. Streams, and what #102 adds"],"A notice goes into the pane that `route` gives for the stream `notices` and the world in focus."] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | D11 sends a notice to the pane that route gives for the stream notices. The notice that no world has the focus is pushed straight into PaneId::NOTICES, the pane of the first output, so with `output as notices` it shows in `output`. |
| R-106 | 7 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D11. Streams, and what #102 adds"],"`Model::notice` and `Model::hint` push to the pane that `route` gives for `notices` and the world in focus."] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | The notice of a failed send is pushed into the pane of the world of the send, not through route for the stream notices, so with `output as notices` a `-- ` line still shows in `output`. D11 records no such exception. |
| R-107 | 7 | 8 | should-fix | in-change | drift | ["path","crates/mu-tui/src/layouts.rs",67] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  | severity? suggestion 0.93 | The save of the layout on the screen under a new name is written twice in layouts.rs: the New branch of save_shown and layout_save each take then_choose, call set_chosen and build the same Effect::SaveLayout. |
| R-108 | 7 | 8 | should-fix | in-change | drift | ["path","crates/mu-tui/src/model.rs",609] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | The rule that the first output of the plan shows the pane of the world, and each other output a pane in extra_panes, is written in pane_of_block (the draw and the mouse) and again in pane_for (where lines go). If one changes alone, lines go into a pane that the draw does not show. |
| R-109 | 7 | 8 | should-fix | in-change | drift | ["path","crates/mu-tui/src/view.rs",137] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | The layout of a title row has two mechanisms: title_row in view.rs writes the ids with brackets and one space, and click_title in update.rs works out the columns of each name again from the same brackets and space. A change to the text of the row moves the names, and a click then picks the wrong block. |
| R-110 | 7 | 8 | should-fix | in-change | drift | ["path","crates/mu-server/src/store.rs",234] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | The store builds the path of a file under clients/ as <dir>/<name>.toml in three places: layout_path, the layer files of resolve, and the device file of put_choice. name_in holds the inverse once. The write path and the read path of a device layer each format the path on their own. |
| R-111 | 7 | 8 | should-fix | in-change | drift | ["path","crates/mu-tui/src/arrange.rs",105] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | Arrange mode states the rule of thirds twice: drop_at picks the part of a drop with its own third closure, and drop_area marks that part with a second copy. If one changes, the mark no longer shows where the block lands. |
| R-112 | 7 | 8 | should-fix | in-change | drift | ["path","crates/mu-tui/src/arrange.rs",218] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | arrange_change and arrange_size each clone the layout, show the changed layout, push the old one on the undo stack and clear or set the message. A change to the undo rule must be made in both. |
| R-113 | 7 | 8 | should-fix | in-change | unmet-requirement | ["path","crates/mu-tui/src/run.rs",86] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | The early Msg::Tick of the run loop sets TickWait to Idle while the wait from the probe message is pending, so a starting layout with a time segment runs two tick chains and draws twice a second. No test sends a tick while a wait is pending that did not come from that wait. |
| R-114 | 7 | 8 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-server/src/write.rs",18] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | ensure_dir uses symlink_metadata, so a clients or clients/layouts path that is a link to a directory fails with 'is not a directory', while the snapshot follows that link and reads the layers. No test passes a link, and D8 does not say a linked directory is refused. |
| R-115 | 7 | 8 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/client.rs",2309] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | The test of 'A name that is too long' checks the text '/layout save' in the joined rows but not that the notice is a line that starts with '-- '. |
| R-116 | 7 | 8 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/client.rs",2326] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | The test of 'A device with a built-in name' shares not_saved, so it also never asserts the '-- ' start of the notice line. |
| R-117 | 7 | 8 | should-fix | in-change | false-test | ["path","crates/mu-config/tests/layout.rs",1095] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | The property re-validates a LayoutName with LayoutName::new, the constructor that save_target already used, and an Err result asserts nothing. |
| R-118 | 7 | 8 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/arrange.rs",236] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | The test starts at 90% and expects 90% after the larger-key. A no-op key, a missing clamp that the check then refuses, and a working clamp all give the same lines. |
| R-119 | 7 | 8 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/app.rs",3303] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | The drag past the smallest pane asserts only `>= 2` rows. A drag that moved nothing (12 rows left) passes. The sibling test for columns asserts the exact 10. |
| R-120 | 7 | 8 | suggestion | in-change |  | ["artifact",["design.md","D5. Layer 2: layouts on the server"],"`layer_rule` gets this place, and checks the name with `LayoutName`."] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  | scope? deviates-design 0.64; severity? should-fix 0.65 | D5 says layer_rule gets the place of the layout files and checks the name with LayoutName. At head layer_rule only walks PLACES; state() sends a layout file to layout_state, which runs LayoutName::new. |
| R-121 | 7 | 8 | suggestion | in-change |  | ["path","crates/mu-cli/src/config_verb.rs",130] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | key_part writes the TOML rule of a bare key by hand to print an entry of [world-layouts]. toml_edit, already a workspace dependency, gives the same with Key::display_repr, and D5 names no alternative. |
| R-122 | 7 | 8 | suggestion | in-change |  | ["path","crates/mu-config/src/layout/tree.rs",190] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  | severity? should-fix 0.62 | The range check of build, which guards resize for any Size a caller makes, is tested only at 95%. The reader refuses 9% and 91% before build runs, so a_percent_out_of_range never reaches this check, and no test passes 9%, 91%, a count of 0 or max 0 and 101 to resize. |
| R-123 | 7 | 8 | suggestion | in-change |  | ["path","crates/mu-config/src/layout/change.rs",106] | resolved | resolved in round 8 from fixed 726d0a7d734f1b1964784993f7822b3026df52f5 | 7 |  |  | percent_range must give exactly the whole percents from 10 to 90 whose solved cells keep both sides at the smallest size, for any split total; the tests pin only the totals 20, 21, 80 and 300. |
| R-124 | 7 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"A block beside two rows"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command |  |  |  | covered row 'A block beside two rows' has no verification run |
| R-125 | 7 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"An id in a later line"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command |  |  |  | covered row 'An id in a later line' has no verification run |
| R-126 | 8 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A refused line goes to the notices"] | fixed | 726d0a7d734f1b1964784993f7822b3026df52f5 | 8 |  |  | delta scenario 'A refused line goes to the notices' has no matrix row |
| R-127 | 8 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A refused line of a world that lost the focus"] | fixed | 726d0a7d734f1b1964784993f7822b3026df52f5 | 8 |  |  | delta scenario 'A refused line of a world that lost the focus' has no matrix row |
| R-128 | 8 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A line with no world goes to the notices"] | fixed | 726d0a7d734f1b1964784993f7822b3026df52f5 | 8 |  |  | delta scenario 'A line with no world goes to the notices' has no matrix row |
| R-129 | 8 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A save through a linked directory"] | fixed | 726d0a7d734f1b1964784993f7822b3026df52f5 | 8 |  |  | delta scenario 'A save through a linked directory' has no matrix row |
| R-130 | 9 | 11 | should-fix | deviates-design | wrong-behaviour | ["artifact",["specs/client-layout/spec.md","Requirement: A line stays in the output where it arrived"],"A later change of the layout SHALL NOT move the line."] | resolved | resolved in round 11 from fixed cd0fe4f5bbd39298163b06361d93ef510effe949 | 10 |  | scope? in-change 0.84 | No decision keeps a line in its output when a change of the layout changes which output is first. D11 ties the pane of the world to whichever output is first, so `/pane hide output` on `sidebar` makes `notices` show the world's lines, and the notices that arrived in `notices` stop showing. |
| R-131 | 9 | 10 | should-fix | in-change | drift | ["path","crates/mu-cli/tests/client.rs",1526] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  | severity? suggestion 0.82 | The test helper that writes the text of a layout file is written three times, in client.rs, config_store.rs and layout.rs of mu-cli/tests. The repository keeps shared test helpers in tests/support/mod.rs. |
| R-132 | 9 | 10 | should-fix | in-change | drift | ["path","crates/mu-cli/tests/client.rs",2139] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  | severity? suggestion 0.84 | The test helper place_of, which reads the lines of a saved layout file, has the same body in client.rs and in layout.rs of mu-cli/tests. |
| R-133 | 9 | 10 | should-fix | in-change | drift | ["path","crates/mu-tui/src/layouts.rs",170] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  | severity? suggestion 0.68 | The numbered form of a plan line, which the proposal makes the form of each listing, is written in three places. These are /layout plan in the client, mu layout show and mu config show. Each one numbers the lines itself. |
| R-134 | 9 | 10 | should-fix | in-change | drift | ["path","crates/mu-config/src/layout/mod.rs",188] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  |  | The rule for which output is the first output is spelled out in three places: route, describe and Model::pane_of. #102 changes where streams go, and these three can then disagree on the output that shows the world. |
| R-135 | 9 | 10 | should-fix | deviates-design | drift | ["path","crates/mu-server/src/store.rs",978] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  | scope? in-change 0.89; severity? suggestion 0.71 | There are two mechanisms that save a layout and then point a world entry at it. The store does it in one locked write in edit_layout, with put_layout and then put_choice. The client builds it from two calls, SaveLayout and then UseLayout, because SaveLayout takes no key. |
| R-136 | 9 | 10 | should-fix | in-change | false-test | ["path","crates/mu-config/src/layout/mod.rs",202] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  |  | The limit of 32 lines is tested only at 33 lines. No test passes exactly 32 lines, so the check `count > LINES_MAX` can become `>=` or a lower limit and every test still passes. |
| R-137 | 9 | 10 | should-fix | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Requirement: Each finished change saves the layout"],"a command that changed it, the release of a drag that resized a pane, and Enter in arrange mode"] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  |  | The spec saves only after a drag that resized a pane, but release_left saves whenever the mouse moved. A drag that returns to its start, or whose resize_border gives the same plan or an error, still saves; on a built-in layout that writes a new layout file and changes the device choice. |
| R-138 | 9 | 10 | should-fix | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Requirement: Each finished change saves the layout"],"The client SHALL save the layout after each of these: a command that changed it"] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  |  | The spec saves after a command that changed the layout, but change_layout saves after every change that the rules accept. `/pane size notices 30%` on sidebar, or `/place tabs above output` on classic, gives the same plan and still saves a new layout for the device. |
| R-139 | 9 | 10 | suggestion | in-change |  | ["artifact",["design.md","D7. Layer 2: the streams, the tab label and the statusline"],"The run loop sends one tick before the first draw."] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  | severity? should-fix 0.80 | D7 says the run loop sends one tick before the first draw. At head the run loop sends Msg::Clock, which sets the time and leaves the wait of a tick alone. An early Msg::Tick is the double tick chain that R-113 removed. |
| R-140 | 9 | 10 | suggestion | in-change |  | ["artifact",["design.md","D9. Layer 4: commands and the mouse"],"A press and a release on one name with no move is a click, and calls `front`."] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  |  | D9 says a click on a name of a title row calls `front`. At head `front` in view.rs only reads the block in front. The click runs click_title, which writes Model::fronts. |
| R-141 | 9 | 10 | suggestion | in-change |  | ["artifact",["tasks.md","3. Layer 2: named layouts"],"`the_default_comes_last` and `no_world_gives_the_default` in `layout.rs` of `mu-config` pass"] | resolved | resolved in round 10 from fixed ee7a74a0c580486e3f3de9df7b6504047114f42e | 9 |  |  | Task 3.2a puts the four unit tests of layout_for in layout.rs of mu-config. At head they are in crates/mu-config/tests/names.rs. |
| R-142 | 9 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"The default layout"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command |  |  |  | covered row 'The default layout' has no verification run |
| R-143 | 9 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"A label too long"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command |  |  |  | covered row 'A label too long' has no verification run |
| R-144 | 9 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"No worlds"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command |  |  |  | covered row 'No worlds' has no verification run |
| R-145 | 9 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"A drag past the smallest pane"] | refuted | review_merge.py matches a run only against the whole covered-by text, so a row that names two tests matches no command |  |  |  | covered row 'A drag past the smallest pane' has no verification run |
| R-146 | 9 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"fit partial"] | accepted | ticket: none; reason: the fit lens sampled in the third full round; the operator accepted the gap as for the other lenses, and the suite review of task 7.3 reads the branch again |  |  |  | lens fit reported partial, gaps: 4 |
| R-147 | 10 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A hide of the main output"] | refuted | a follow-up round has no test-coverage lens, so a scenario that a fix adds gets no matrix row; the operator chose this disposition before the round |  |  |  | delta scenario 'A hide of the main output' has no matrix row |
| R-148 | 10 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A new main output takes the lines of the world back"] | refuted | a follow-up round has no test-coverage lens, so a scenario that a fix adds gets no matrix row; the operator chose this disposition before the round |  |  |  | delta scenario 'A new main output takes the lines of the world back' has no matrix row |
| R-149 | 10 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A line of a world with no output"] | refuted | a follow-up round has no test-coverage lens, so a scenario that a fix adds gets no matrix row; the operator chose this disposition before the round |  |  |  | delta scenario 'A line of a world with no output' has no matrix row |
| R-150 | 10 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The main output is not the first output line"] | refuted | a follow-up round has no test-coverage lens, so a scenario that a fix adds gets no matrix row; the operator chose this disposition before the round |  |  |  | delta scenario 'The main output is not the first output line' has no matrix row |
| R-151 | 10 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A save with a key"] | refuted | a follow-up round has no test-coverage lens, so a scenario that a fix adds gets no matrix row; the operator chose this disposition before the round |  |  |  | delta scenario 'A save with a key' has no matrix row |
| R-152 | 10 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A save with a key and no device"] | refuted | a follow-up round has no test-coverage lens, so a scenario that a fix adds gets no matrix row; the operator chose this disposition before the round |  |  |  | delta scenario 'A save with a key and no device' has no matrix row |
| R-153 | 10 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A command that changes nothing"] | refuted | a follow-up round has no test-coverage lens, so a scenario that a fix adds gets no matrix row; the operator chose this disposition before the round |  |  |  | delta scenario 'A command that changes nothing' has no matrix row |
| R-154 | 10 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A drag back to its start"] | refuted | a follow-up round has no test-coverage lens, so a scenario that a fix adds gets no matrix row; the operator chose this disposition before the round |  |  |  | delta scenario 'A drag back to its start' has no matrix row |
| R-155 | 11 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The earlier lines of a block that stands in for main"] | refuted | a follow-up round has no test-coverage lens, so a scenario that a fix adds gets no matrix row; the operator chose this disposition before the round |  |  |  | delta scenario 'The earlier lines of a block that stands in for main' has no matrix row |

## Reviewed files

| path | content |
|---|---|
| Cargo.lock | bedf076990a2723de953a29dc5560bcba160a376 |
| Cargo.toml | 9c941dec15e79db61ba820e3e17faf13abf2f11a |
| architecture/model.c4 | f0e5a47894b832d4143111d80038c5aa918e6eee |
| architecture/views.c4 | 806536588b18b5861f18eff3f24b8cd30ab71aa0 |
| crates/mu-cli/Cargo.toml | aa31266cf493719333b6f144b42fe70e721ad9b3 |
| crates/mu-cli/src/config_verb.rs | 86ac1fcb1aa254a4e0f781a4851ae1c36aee0541 |
| crates/mu-cli/src/layout_verb.rs | 666e55a720c79151162c0a5e9e79573b595d31f1 |
| crates/mu-cli/src/lib.rs | 0d47527b1944ed448a29d739ab2d93370f08fcc0 |
| crates/mu-cli/src/remote.rs | 7a523c62388d9ffc3f9c2c6497d16b38c6174acd |
| crates/mu-cli/tests/cli.rs | 7797d83b8a7c4b599925ecce556bca6dc23de847 |
| crates/mu-cli/tests/client.rs | 1602188ef51e90cbfe51119ec56147d2477ac723 |
| crates/mu-cli/tests/config_store.rs | e9218c52758b01ce1459877cb8c332e3ea1ddacc |
| crates/mu-cli/tests/layout.rs | f3c90b6d0433ea94cc0fd94ce8e73368c4a4b846 |
| crates/mu-cli/tests/support/mod.rs | e639f97ff567d14798ba98add497a663ad402b8e |
| crates/mu-cli/tests/workspace_boundary.rs | e6dda3c06a3f15aa11ab63f739bfb53373eed874 |
| crates/mu-client/src/lib.rs | 79d2c32b8ea7dee53436383732546bf05c1859d5 |
| crates/mu-config/Cargo.toml | 2b11bb4e888bd4ecd3e6da65bec8760b2466f9d4 |
| crates/mu-config/src/entries.rs | 173656ccc3da020a40c5ac3bd5738ef1d18ee64f |
| crates/mu-config/src/label.rs | 10288472d774157a446a15d5dea3ef12c94b848f |
| crates/mu-config/src/layer.rs | 3d1f69cd2e17f8b9b62ee67b849f03da4c51f18f |
| crates/mu-config/src/layout/block.rs | be68431a61b6d18b5a7e6810819c41d7824cbce9 |
| crates/mu-config/src/layout/builtin.rs | e7786e40b0c711fe55d5aaaa15dfb6009709460a |
| crates/mu-config/src/layout/change.rs | d36610e0b3526c565202636c0cff5d4fb45bdcc1 |
| crates/mu-config/src/layout/mod.rs | af7503e8dda1322eb695861bc2f80c37c164b6be |
| crates/mu-config/src/layout/parse.rs | 5c31fa357e2adaa39939fa78ea26655583fcc32d |
| crates/mu-config/src/layout/solve.rs | 25c6bb39cecfc5e0936b4a8cf39c905e4937fe67 |
| crates/mu-config/src/layout/step.rs | 8c5267090900d57e286ad2eb576bd1d0f8eecec1 |
| crates/mu-config/src/layout/tree.rs | 6d21b09e3bf45728de6536655a194e7dbc415f41 |
| crates/mu-config/src/layout/write.rs | db02174d0541c4c79d6ea4b8d0c05686f4b9ace5 |
| crates/mu-config/src/lib.rs | a0bd98df708f7e54f36df123ccf800b72c6927c0 |
| crates/mu-config/src/names.rs | fba014820cf321b8290d76fc672d7fd65e8a362f |
| crates/mu-config/src/world_id.rs | 520b3410e6ff6307e9d0ca8c5a16eedfc290336a |
| crates/mu-config/tests/layer.rs | f245d723f398870adc217b2ff2c62b554b72c0b6 |
| crates/mu-config/tests/layout.proptest-regressions | e8336239f3b196ee48d2b252e7d4e1dc16e9bb0d |
| crates/mu-config/tests/layout.rs | 7017cd2f2f1c0e5780c82293300d0a5bcea345ea |
| crates/mu-config/tests/names.rs | a3bf5e9376ac783fe25303f3f603aaf66f888785 |
| crates/mu-core/Cargo.toml | 89937e8493ca6d837f8eb28a58ff84981adc7d8f |
| crates/mu-core/src/id.rs | 453174368922ff0a1b221f12e9dabe7de3b907b0 |
| crates/mu-core/src/world/merge.rs | 92c82461f2c8b54c6cde46715e4eda6e05367d83 |
| crates/mu-proto/proto/mu/v1/portal.proto | 6e36604c8205956969c5dedbe6468ecd2b3b81ab |
| crates/mu-server/Cargo.toml | a53455fa1b31b187adcf337d18ed20c05f497e6d |
| crates/mu-server/src/lib.rs | 8428a97973fcb851b3de9aaf01748fb20d847625 |
| crates/mu-server/src/service.rs | e5c00f71d6d911e7250f313941d21ac7ca57f6fe |
| crates/mu-server/src/store.rs | fe5b1c6e9d60eea3a682130c342c031341fb5a74 |
| crates/mu-server/src/write.rs | 2abaab243c8f771156208db416fcdf0e1ed809a2 |
| crates/mu-server/tests/store.rs | ede35486d76925e09c0a15a35a8cd41b297eeed4 |
| crates/mu-tui/src/arrange.rs | 2e0578589a1756907c42e682fbeb24523213aba5 |
| crates/mu-tui/src/config.rs | 31dc99ee32b0d94efa67a8c214d408fd81db6681 |
| crates/mu-tui/src/editor.rs | bf65a6b7efd456bb35a3accc565eaca4b471a3c8 |
| crates/mu-tui/src/keymap.rs | 60e9ef84b8124b829806a350bc4145fc460bb4b2 |
| crates/mu-tui/src/layout.rs | absent |
| crates/mu-tui/src/layouts.rs | 0d687ee4522ec07890bcab385118c618e4f811be |
| crates/mu-tui/src/lib.rs | 8e55962cb616d2f0f7dfdc89ce6353b6780640f2 |
| crates/mu-tui/src/model.rs | f726f2908d5663b87c9ff9642112a57e889b654d |
| crates/mu-tui/src/registry.rs | 6acfabe5ca36a9f4249aaeb78e8f63f7d6fb1ad3 |
| crates/mu-tui/src/run.rs | 30098c5d48dc39e61f96359ad8e103c3d6cbc8c4 |
| crates/mu-tui/src/statusline.rs | df28af7fb7db26ca5d9085079b17fa3bd1ada49c |
| crates/mu-tui/src/update.rs | e15804682fad23ec6b3946d3a38dccbea3c14e5f |
| crates/mu-tui/src/view.rs | cc34fea8d6740abb51d4cde0f4cab2c34896f590 |
| crates/mu-tui/src/width.rs | 8894728414954329d5eab91707dd99f25dee4204 |
| crates/mu-tui/tests/app.rs | 69610c85267f61b3e6455c768d6a288dbc2a57cc |
| crates/mu-tui/tests/arrange.rs | 5d3d741e12ef7791e7d4144480f61504072753e4 |
| crates/mu-tui/tests/editor.rs | c7992ae680d306d88c22f875eb27a1f19a575a86 |
| crates/mu-tui/tests/snapshots/app__a_percent_and_a_count_of_cells.snap | 185ad1cc3fc2071b5e92a5cf7c930976a373efd3 |
| crates/mu-tui/tests/snapshots/app__the_picture_of_each_built_in_layout.snap | d96114ee98416bb0b914415f3baf834d4d81bfe4 |
| crates/mu-tui/tests/update.rs | 6d35a0f7dcbcc96d1e5cb2b95c1f5306b699e32f |
| docs/src/content/docs/architecture/server-and-clients.mdx | e26f7a7035c6adf0535056d29b03930cc195db77 |
| docs/src/content/docs/architecture/terminal-client.mdx | 85d2f6949411c56fe3e935541ebefcab776a2d6c |
| docs/src/content/docs/decisions/layout-model.md | 387b996cce62927addf2042db6ffddbd3097025c |
| docs/src/content/docs/guides/client.md | 1998f8025e8130dad06ff91480087d779112027e |
| docs/src/content/docs/guides/configuration.md | 1a6d9215d8fc6a948b4cd0eabebd881f0f5d9379 |
| docs/src/content/docs/guides/layouts.md | c8487d2acb8eb016a032f4f3406efc02b4e0a378 |
| openspec/changes/client-layout/.openspec.yaml | 880dcbddfbd01d2f90c77d58dcf594b444480b33 |
| openspec/changes/client-layout/design.md | ba7e2d5f44e098d846afc54ad7cd37046cd66d6c |
| openspec/changes/client-layout/proposal.md | aeea3d73493fb740ca4ad21d70a4e5d9c4086c80 |
| openspec/changes/client-layout/specs/client-config/spec.md | 732875b42d24bcce5855b8e077ae1a36f615591b |
| openspec/changes/client-layout/specs/client-layout/spec.md | 58febc36f7a9533e0f595e0339ab3a36e21e0126 |
| openspec/changes/client-layout/specs/client-statusline/spec.md | ac3a3ba76dafc334996beeb1f82b4623b5a0b352 |
| openspec/changes/client-layout/specs/config-store/spec.md | 4a8b7666a0810695e6352e63aa2586b6f32f9dee |
| openspec/changes/client-layout/specs/layout-editing/spec.md | 494472933c1841717661065358777498fd4e4c68 |
| openspec/changes/client-layout/specs/terminal-client/spec.md | a42fcbd9684d5fea1a7bf612a60c2f0e2ea56f95 |
| openspec/changes/client-layout/tasks.md | sha256:c0d205d887620b58b0c34229d39edb6a645749fbc2945663fc4d64f637687706 |
| openspec/review.json | absent |

## Plan review (2026-11-25)

Verdict approve at head dc8e5c3bdf083ba14f7b8ffa417a527aa6b701ad.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| No tabs | 3.8 | covered |  |
| The editor alone | 3.8 | covered |  |
| Nine outputs | 2.1 | covered |  |
| One output shows the notices too | 3.5 | covered |  |
| An output for the notices | 3.5 | covered |  |
| An output with no stream yet | 3.5 | covered |  |
| An output that comes later starts with new lines | 3.5 | covered |  |
| A notice shows under the world in focus only | 3.5 | covered |  |
| Two editors keep two texts | 3.5a | covered |  |
| Enter sends the editor with the input focus | 3.5a | covered |  |
| One text for each world | 3.5a | covered |  |
| A second editor with no id | 2.1 | covered |  |
| Five editors | 2.1 | covered |  |
| The key moves the input focus | 3.5a | covered |  |
| A click moves the input focus | 3.5a | covered |  |
| One editor | 3.5a | covered |  |
| The output above the tabs | 3.8 | covered |  |
| A block beside another | 3.8 | covered |  |
| The numbers of the lines | 4.4 | covered |  |
| The editor after the notices | 2.3 | covered |  |
| The editor before the notices | 2.3 | covered |  |
| A block beside two rows | 2.3 | partial |  |
| Anchors that are not neighbors | 2.3 | covered |  |
| An id in a later line | 3.8 | covered |  |
| An id twice | 2.3 | covered |  |
| An id that is a word of the sentences | 2.3 | covered |  |
| The statusline at the bottom | 3.8 | covered |  |
| Notices behind the output | 3.8 | covered |  |
| The editor does not stack | 2.3 | covered |  |
| A percent | 3.8 | covered |  |
| A count of cells | 3.8 | covered |  |
| A size on the tabs | 2.3 | covered |  |
| A size on the first line | 2.3 | covered |  |
| The editor stops at its max | 3.8 | covered |  |
| The default max | 3.8 | covered |  |
| Two editors side by side | 3.5a | covered |  |
| The shorter max does not stop the taller editor | 3.5a | covered |  |
| An unknown block | 2.3 | covered |  |
| A block twice | 2.3 | covered |  |
| A block that is not placed yet | 2.3 | covered |  |
| No editor | 2.3 | covered |  |
| A percent out of range | 2.3 | covered |  |
| Too many lines | 2.3 | covered |  |
| Any layout reads back | 1.1 | covered |  |
| The default layout | 3.3 | covered |  |
| The sidebar layout | 3.8 | covered |  |
| Two text panes | 3.4 | covered |  |
| One text pane has no title row | 3.4 | covered |  |
| A percent in a narrow terminal | 3.4 | covered |  |
| A reload changes the layout | 3.8 | covered |  |
| A layout with no editor from the server | 3.5 | covered |  |
| A label with the number | 3.6 | covered |  |
| A label keeps the unseen mark | 3.6 | covered |  |
| The game and the character | 3.6 | covered |  |
| Braces | 3.6 | covered |  |
| An unknown field | 3.3 | covered |  |
| An open brace | 3.3 | covered |  |
| A label too long | 3.3 | covered |  |
| A world with its own layout | 3.5b | covered |  |
| The focus moves to a world with no entry | 3.5b | covered |  |
| A layout for a game | 3.5b | covered |  |
| The world entry comes before the game entry | 3.5b | covered |  |
| No world in focus | 3.5b | covered |  |
| A world keeps its text across a change of layout | 3.5b | covered |  |
| A layout file in use | 3.2 | covered |  |
| A bad layout file at start | 3.2 | covered |  |
| A file with a built-in name | 3.2 | covered |  |
| An unknown field of a layout file | 3.2 | covered |  |
| The device names another layout | 3.2 | covered |  |
| A layout that does not exist | 3.2 | covered |  |
| A change to a layout file changes the revision | 3.2 | covered |  |
| The list with one player layout | 3.9 | covered |  |
| The list with a bad layout file | 3.9 | covered |  |
| A save writes the file | 4.2 | covered |  |
| A save of a bad layout | 4.2 | covered |  |
| A save with a built-in name | 4.2 | covered |  |
| A save with a bad name | 4.2 | covered |  |
| A save with the name of two dots | 4.2 | covered |  |
| A clients path that is a file | 4.2 | covered |  |
| A device file with a comment | 4.3 | covered |  |
| No device file | 4.3 | covered |  |
| An unknown layout | 4.3 | covered |  |
| A bad device layer | 4.3 | covered |  |
| A use from the command line | 4.3 | covered |  |
| The picture of classic | 3.9 | covered |  |
| An open socket directory | 3.9 | covered |  |
| No server | 3.9 | covered |  |
| An edit places a block | 4.4 | covered |  |
| An edit gives the same file as the command | 4.4 | covered |  |
| An insert from the command line | 4.4 | covered |  |
| A delete by id | 4.4 | covered |  |
| An edit of a built-in layout | 4.4 | covered |  |
| A bad edit | 4.4 | covered |  |
| Describe gives each block | 4.4 | covered |  |
| Describe of an unknown layout | 4.4 | covered |  |
| A device adds an entry | 3.2a | covered |  |
| A device replaces an entry | 3.2a | covered |  |
| An entry for a world that the server does not have | 3.2a | covered |  |
| A change to the layout file of an entry changes the revision | 3.2a | covered |  |
| A key with two slashes | 3.2a | covered |  |
| An entry with a layout that does not exist | 3.2a | covered |  |
| A use for one world | 4.3 | covered |  |
| A use for a game | 4.3 | covered |  |
| Forget the layout of a world | 4.3 | covered |  |
| A use with a bad world id | 4.3 | covered |  |
| An edit of the built-in layout of one world | 4.4 | covered |  |
| Add the notices | 5.2 | covered |  |
| Move the tabs | 5.2 | covered |  |
| A bad sentence | 5.2 | covered |  |
| Hide the tabs | 5.2 | covered |  |
| The editor stays | 5.2 | covered |  |
| Hide a second editor | 5.2 | covered |  |
| Resize the notices | 5.2 | covered |  |
| A size of the first block | 5.2 | covered |  |
| List the plan | 5.2a | covered |  |
| Insert in the middle | 5.2a | covered |  |
| Edit a line by its id | 5.2a | covered |  |
| Delete a line that others need | 5.2a | covered |  |
| A resize changes one line | 2.2 | covered |  |
| A hide of an anchor writes the plan again | 2.2 | covered |  |
| A click on a title | 5.5 | covered |  |
| A drag to the left | 5.5 | covered |  |
| A drag past the smallest pane | 5.5 | covered |  |
| A drag down | 5.5 | covered |  |
| A selection beside a divider | 5.5 | covered |  |
| The letters | 6.1 | covered |  |
| Typing in arrange mode | 6.1 | covered |  |
| The tabs go below the output | 6.2 | covered |  |
| A refused stack | 6.2 | covered |  |
| Add the statusline | 6.2 | covered |  |
| Undo | 6.2 | covered |  |
| A letter bound to undo still picks | 6.2 | covered |  |
| A drag to the right edge | 6.3 | covered |  |
| Esc restores | 6.2 | covered |  |
| A save from a built-in layout | 5.3 | covered |  |
| A save to a player layout | 5.3 | covered |  |
| A second client gets the change | 5.3 | covered |  |
| The server refuses | 5.3 | covered |  |
| Use a built-in layout | 5.4 | covered |  |
| Save under a name | 5.4 | covered |  |
| A save over a built-in name | 5.4 | covered |  |
| A save from the built-in layout of one world | 5.3 | covered |  |
| The other world keeps its layout | 5.3 | covered |  |
| A name that is too long | 5.3 | covered |  |
| A device with a built-in name | 5.3 | covered |  |
| A layout for this world | 5.4 | covered |  |
| Remove the layout of this world | 5.4 | covered |  |
| No world in focus | 5.4 | covered |  |
| An unknown layout for this world | 5.4 | covered |  |
| A use behind an entry | 5.4 | covered |  |
| The next layout of a world entry | 5.4 | covered |  |
| The default segments | 3.8 | covered |  |
| A narrow terminal | 3.7 | covered |  |
| The typed count | 3.7 | covered |  |
| The state after a disconnect | 3.8 | covered |  |
| The idle time after a send | 3.8 | covered |  |
| The connected time | 3.8 | covered |  |
| No connected time while disconnected | 3.7 | covered |  |
| No worlds | 3.7 | covered |  |
| Under an hour | 3.7 | covered |  |
| Over an hour | 3.7 | covered |  |
| A tick moves the idle time | 3.7 | covered |  |
| No tick without a time segment | 3.7 | covered |  |
| An unknown segment | 3.3 | covered |  |
| A segment twice | 3.3 | covered |  |
| A device sets the left list | 3.3 | covered |  |
| A server table | 3.3 | covered |  |
| Not TOML | 3.3 | covered |  |
| A file that cannot be read | 3.3 | covered |  |
| The three new fields | 3.3 | covered |  |
| A layout table | 3.3 | covered |  |
| An unknown field of tabs | 3.3 | covered |  |
| A world layouts table | 3.2a | covered |  |
| The default layout puts the tabs first | 3.8 | covered |  |

### Not run

Plan phase: no verification runs.
