---
verdict: changes-requested
phase: implementation
reviewed_at: 2026-11-25T14:21:40+00:00
head: d3bceb71013766854fc8e1c20fef7ff25f0d69e0
round: 6
next_id: 105
lenses:
  spec-lint: not-applicable
  approach: partial
  fit: ok
  test-coverage: ok
  test-honesty: partial
  invariants: partial
  verify: not-applicable
---
# Review

## Incomplete lenses

- approach: partial, gaps: 3
  - The grammar and error forms of D4 (winnow ParseError offset and cause) are checked by name only, not against every case of parse.rs.
  - The build rules of D2 (several anchors, edges, levels) in tree.rs are not read.
  - The writer of D4 in write.rs is not read.
- test-honesty: partial, gaps: 2
  - Sampled, not exhaustive: read crates/mu-tui/tests/arrange.rs in full, the new tail of crates/mu-tui/tests/update.rs, the save_target tests of crates/mu-config/tests/layout.rs and the is_err asserts of crates/mu-cli/tests/layout.rs. Not read: the rest of mu-config/tests/layout.rs, mu-cli/tests/client.rs, config_store.rs, mu-server/tests/store.rs, mu-tui/tests/app.rs and the in-file unit tests of mu-config and mu-tui src.
  - No regex for a test body with no assertion could run (ripgrep without look-around), so no-assertion was checked by sampling only.
- invariants: partial, gaps: 2
  - The requirements of specs/config-store, client-statusline, client-config and terminal-client were read by heading and limiting clause only. Their scenarios were not traced to tests.
  - D11 route() at the arrival of a line in mu-tui (the pane state of each world and output id) was not traced.

The verdict covers only what the lenses checked. While the `ctl-lens` row of an incomplete lens is open, the verdict cannot be `approve`.

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-8 | fit | blocker | in-change | Label::render splits the world text at '/' itself. That is a third copy of the split, beside WorldId and the `order` split of mu-tui, which D5 says the type replaces. | crates/mu-config/src/label.rs:131 — "let (game, character) = world.split_once('/').unwrap_or((world, ""));" | Make Label::render take a &WorldId and use game() and character(). Sort the worlds of mu-tui by the Ord of WorldId, and remove `order`. | new-evidence | severity? should-fix 0.68 | drift |
| R-74 | approach | should-fix | in-change | No decision produces `/pane size <id> fill`. D3 lists no fill operation, so the client adds Change::Fill on its own, while EditLayout parses the size with Size::parse, which refuses `fill`. `mu layout size <name> <id> fill` fails where the client command of the same name works. | specs/layout-editing/spec.md > Requirement: The pane commands hide and resize a block — "`/pane size <id> <size>` SHALL set the size on the line of the block, and a size of `fill` SHALL remove it." | Add `fill(id)` to the table of screen changes in D3, map the size `fill` of SizeChange to Change::Fill in the service, and test `mu layout size mine notices fill`. |  |  | unmet-requirement |
| R-75 | approach | should-fix | in-change | Enter in arrange mode saves nothing when the layout equals the layout from before the mode. D10 says Enter returns Effect::SaveLayout, and the spec says Enter saves; design.md records no deviation. | design.md > D10. Layer 5: arrange mode — "Enter drops the mode and returns `Effect::SaveLayout`." | Record in D10 and in the requirement that Enter with no change leaves the mode and saves nothing, or make Enter save in every case. |  |  | unmet-requirement |
| R-76 | approach,fit | should-fix | in-change | fit-2: D9 makes the save rule one function, save_target(choice, layout name, device). The code drops the layout name, so the client and EditLayout each write their own branch for a built-in layout around it.; app-3: D9 cites save_target with three inputs that gives the name and the entry or field. At head it takes the choice and the device only, and returns the LayoutName alone. | design.md > D9. Layer 4: commands and the mouse — "The save rule of the spec is one function of `mu-config`, `save_target(choice in use, layout name, device name)`." | fit-2: Give save_target the layout name, as D9 states. It returns the name to save under and whether the choice changes. The client and edit_layout then only act on that result.; app-3: Write `save_target(choice, device) -> LayoutName` in D9, and say that the caller sets the field or the entry from the choice. |  |  | drift |
| R-102 | approach | should-fix | in-change | lens approach reported partial, gaps: 3 | review > lens-status — "approach partial" | rerun the lens or accept the gap in the review |  |  | unmet-requirement |
| R-77 | fit | should-fix | in-change | Two places turn the text of a size into a Change, and they already differ. `/pane size <id> fill` gives Change::Fill. EditLayout runs Size::parse alone, which refuses `fill`, so `mu layout size` cannot do the same change. | crates/mu-tui/src/layouts.rs:138 — "if size == "fill" {" | Add one function to mu-config that reads an id and a size text, `fill` included, into Change::Size or Change::Fill. Call it from pane_size and from change_of. |  |  | drift |
| R-78 | fit | should-fix | in-change | `mu layout show` writes the path of a layout file by hand to find its notice. mu-server already exports layout_path, and mu-cli depends on mu-server. | crates/mu-cli/src/layout_verb.rs:287 — "let file = format!("clients/layouts/{name}.toml");" | Call mu_server::layout_path(name) in show. |  |  | drift |
| R-79 | fit | should-fix | in-change | The notice that mu wrote the whole plan again is written twice, once in the client and once in `mu layout`. D3 gives the flag one meaning for every caller. | crates/mu-tui/src/layouts.rs:11 — "const REWRITTEN: &str = "no change of one line gives this screen, so mu wrote the whole plan again";" | Move REWRITTEN to mu-config beside Changed, and use it in both places. |  |  | drift |
| R-80 | fit | should-fix | in-change | The new DEVICES_DIR repeats the literal that PLACES already holds for the device layers. The write path uses the constant and the read path uses PLACES, so the two can name different directories. | crates/mu-server/src/store.rs:220 — "dir: "clients/devices"," | Write `dir: DEVICES_DIR` in the device row of PLACES. |  |  | drift |
| R-81 | fit | should-fix | in-change | layout_file repeats the chain in layer_rule that takes a file name out of a path under one directory: strip the dir, strip '/', strip '.toml', and refuse a '/'. | crates/mu-server/src/store.rs:240 — "path.strip_prefix(LAYOUTS_DIR)?" | Add one function, name_in(path, dir) -> Option<&str>, and call it from layout_file and layer_rule. |  | severity? suggestion 0.69 | drift |
| R-82 | fit | should-fix | in-change | LayoutName::new calls DeviceName::new, then throws its error away and writes out the character rule of a device name a second time. | crates/mu-config/src/names.rs:153 — ""a device name has 1 to {DEVICE_MAX} characters from a to z, 0 to 9, -, _ and ."" | Hold the text of the character rule once in names.rs, and use it in both errors. |  |  | drift |
| R-83 | fit | should-fix | in-change | The rule of the block in front of a stack is read twice. view::front and pane_next each look up `fronts` by the first id and fall back to the first block. | crates/mu-tui/src/view.rs:127 — "let wanted = model.fronts.get(&first.id);" | Give Model one method that returns the block in front of a stack of blocks, and call it from view::front and pane_next. |  | severity? suggestion 0.68 | drift |
| R-84 | invariants | should-fix | in-change | percent_range caps the high bound at floor((total-least-divider)*100/total), which refuses whole percents that keep the anchor side at its smallest size. Because the solver rounds a percent down, a split of 21 columns has the good percents 48 to 52, yet the code gives no range and the drag changes nothing. At 80 columns the cap is 86, not 87. In a split of 4 rows it is 50, not 74. The only boundary tests use 20 columns and 3 rows, where no percent exists, so nothing tests one past the limit. | crates/mu-config/src/layout/change.rs:81 — "let high = u32::from(PERCENT_MAX).min(total.checked_sub(least + divider)? * 100 / total);" | Set high to the largest p with floor(p*total/100) <= total-least-divider, i.e. (100*(total-least-divider+1)-1)/total, capped at 90. Add unit tests: resize_border in a 21-column split gives 48%, and resize_by up in an 80-column split stops at 87%. |  |  | wrong-behaviour |
| R-104 | invariants | should-fix | in-change | lens invariants reported partial, gaps: 2 | review > lens-status — "invariants partial" | rerun the lens or accept the gap in the review |  |  | unmet-requirement |
| R-85 | test-coverage | should-fix | in-change | The test for a letter bound to undo asserts the plan lines, not the drawn row that the THEN step names. | crates/mu-tui/tests/arrange.rs:278 — "assert_eq!(app.layout().lines()[2], "tabs below output");" | In a_letter_bound_to_undo_still_picks, draw the model and assert that the row above the editor contains `alpha/aria]`, as the_tabs_go_below_the_output does with rows[21]. |  |  | false-test |
| R-96 | test-coverage | should-fix | in-change | covered row 'An unknown field' has no verification run | review > not-run — "An unknown field" | run the covering test and record it in verification-runs.txt |  |  | unmet-requirement |
| R-97 | test-coverage | should-fix | in-change | covered row 'The editor after the notices' has no verification run | review > not-run — "The editor after the notices" | run the covering test and record it in verification-runs.txt |  |  | unmet-requirement |
| R-98 | test-coverage | should-fix | in-change | covered row 'The editor before the notices' has no verification run | review > not-run — "The editor before the notices" | run the covering test and record it in verification-runs.txt |  |  | unmet-requirement |
| R-99 | test-coverage | should-fix | in-change | covered row 'No world in focus' has no verification run | review > not-run — "No world in focus" | run the covering test and record it in verification-runs.txt |  |  | unmet-requirement |
| R-100 | test-coverage | should-fix | in-change | covered row 'Under an hour' has no verification run | review > not-run — "Under an hour" | run the covering test and record it in verification-runs.txt |  |  | unmet-requirement |
| R-101 | test-coverage | should-fix | in-change | covered row 'Over an hour' has no verification run | review > not-run — "Over an hour" | run the covering test and record it in verification-runs.txt |  |  | unmet-requirement |
| R-103 | test-honesty | should-fix | in-change | lens test-honesty reported partial, gaps: 2 | review > lens-status — "test-honesty partial" | rerun the lens or accept the gap in the review |  |  | unmet-requirement |
| R-86 | approach | suggestion | in-change | D5 says the shared function of the keyed paths quotes a key as TOML needs. At head keyed_path joins the table and the key with no quote; the quote is added by key_part in mu-cli at render. | design.md > D5. Layer 2: layouts on the server — "That block becomes one function with the name of the table as its parameter, and it quotes a key as TOML needs." | State in D5 that keyed_path gives the unquoted source path, and that render quotes the key with key_part. |  |  |  |
| R-87 | approach | suggestion | in-change | D8 says the data source of DescribeLayout is a fixed text of the catalog, with `the journal of the world in focus` for output. At head BLOCKS holds no such text; describe() builds it per step, and the output text differs. | design.md > D8. Layer 3: the write calls — "The data source is a fixed text from the catalog, such as `the journal of the world in focus` for `output`." | State in D8 that describe() gives the data source, with the text of the first output and of a named stream. |  | severity? should-fix 0.69 |  |
| R-88 | approach | suggestion | in-change | D5 cites layout_for with a default parameter. At head it takes the world and the entries, and returns only the Choice; the caller looks up the layout. | design.md > D5. Layer 2: layouts on the server — "`mu-config` gets `layout_for(world, entries, default)`: the entry of the world id, then the entry of the game, then the " | Write `layout_for(world, entries) -> Choice` in D5. |  | scope? deviates-design 0.60 |  |
| R-89 | approach | suggestion | in-change | D7 and D11 say a map keyed by a world and an output id replaces the map of panes by PaneId. At head the map by PaneId stays, the first output uses the pane of the world, and only the other outputs are keyed by world and id. | design.md > D7. Layer 2: the streams, the tab label and the statusline — "The pane states of D11 replace the map of panes by `PaneId`." | State in D7 and D11 that the first output takes the pane of its world, and that extra_panes maps each other output to a PaneId. |  | severity? should-fix 0.63 |  |
| R-90 | approach | suggestion | in-change | D7a names Model::input() as the function that resolves the stored editor id. At head that is Model::input_id(); Model::input() returns the text of the editor. | design.md > D7a. Layer 2: several editors — "`Model::input()` resolves the stored id against `Model::layout()` at each read." | Name Model::input_id() in D7a and in the D12 row of the input focus. |  | severity? should-fix 0.70 |  |
| R-91 | approach | suggestion | in-change | D6 says solve returns Vec<Placed>. At head it returns Solved, which also holds the dividers and the borders that the drag and arrange mode use. | design.md > D6. Layer 2: how the client draws — "The solver is a pure function of `mu-config`, `solve(layout, area, rows_of) -> Vec<Placed>`." | Write `-> Solved` in D6 and name its panes, dividers and borders. |  |  |  |
| R-92 | approach | suggestion | in-change | D3 cites resize_by(id, steps). At head it also takes the cells of the block and the total of its split, which the percent range needs. | design.md > D3. Layer 1: the changes — "`resize_by(id, steps)`" | Write `resize_by(id, steps, cells, total)` in the table of D3. |  | scope? deviates-design 0.70 |  |
| R-93 | approach | suggestion | in-change | D5 names the message of a world entry WorldLayout. At head the field is `repeated NamedLayout world_layouts`, the message that ListLayouts also uses. | design.md > D5. Layer 2: layouts on the server — "`GetClientConfigResponse` gets `repeated WorldLayout world_layouts`, each with a key, a name and its sentences." | Write `repeated NamedLayout world_layouts` in D5. |  | severity? blocker 0.72 |  |
| R-94 | fit | suggestion | in-change | The statusline cuts text to a width with two loops by graphemes, one from the end and one from the start. D7 says the cut uses the functions of width.rs, which gets none. | crates/mu-tui/src/statusline.rs:63 — "for grapheme in right.graphemes(true).rev() {" | Add one cut function to width.rs, with the side to keep (start or end) as its parameter, and call it for both sides. |  | severity? should-fix 0.72 |  |
| R-95 | invariants | suggestion | in-change | The floor of 2 rows and 10 columns for an output pane ranges over any plan and any terminal size that has room. The property test solve_stays_inside_the_area checks only containment and overlap, and never the floor, so a solver change that drops the raise to the smallest size passes it. | specs/client-layout/spec.md > Requirement: A small terminal keeps each pane usable — "A pane of `output` SHALL keep at least 2 rows and 10 columns while the terminal has room for each pane at that size." | Use proptest in crates/mu-config/tests/layout.rs. Add a property next to solve_stays_inside_the_area: for any grown plan and an area at least 11 columns and 3 rows per output along each axis, such as 300 by 100 for at most 8 outputs, assert that each pane holding an output is at least 10 by 2. |  |  |  |

## Repeats

None.

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-73 | should-fix | accepted | Task 3.5a of tasks.md names the test the_input_focus_after_a_change_of_the_world for this scenario. Only a full round writes a matrix row, and the operator chose no further full round. |  |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | d2729150530ac1884073ee7207a64ed3a63ab418 | 0face2499054203abac6a0977c535eb9937058b31597b2276d6765a7042c028d | changes-requested | 35 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | a89e8ef5614c7ab7bd977258c84bc8ba8bc1ac31 | 215c0bd88e3b28033d1218fa69f8e88e090deebc492d274c174eaf4de3a135a3 | approve-with-notes | 8 | 0 | 1 | 33 | 0 |  |
| 3 | plan | full | 12c5e1eb51d612f7a7ad524730010c42c27e625a | b1034da5f8859358c6cfa265503a39cca17afab80d8900ab6706bea5c9cb57db | changes-requested | 29 | 0 | 0 | 0 | 0 | the operator asked for a full round to rebuild the coverage matrix after the fixes |
| 4 | plan | follow-up | 6c78c3f9426d5a2dc94654a44d57d87b3c468073 | 30e945c0cdd12743d13dd5a1582f22da0658f442f9cf7266152530053a4e75f9 | approve-with-notes | 1 | 0 | 2 | 27 | 0 |  |
| 5 | plan | follow-up | dc8e5c3bdf083ba14f7b8ffa417a527aa6b701ad | a18bc81b621e5b0ba32cbccbbc069d6c4a573d3c823551ea6df1cbf68f8efdc8 | approve | 0 | 1 | 0 | 2 | 0 |  |
| 6 | implementation | full | d3bceb71013766854fc8e1c20fef7ff25f0d69e0 | 9f2cbbc88f8b829cf9317c43ed658d28a97311f1f529ca48b13c3dced4b0961a | changes-requested | 31 | 0 | 0 | 0 | 0 |  |

## Judge

judge: 46 questions, 0.002 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A server table | crates/mu-cli/tests/config_store.rs: a_server_table | covered | pass |
| An unknown field | crates/mu-cli/tests/config_store.rs: an_unknown_field; crates/mu-config/tests/layer.rs: an_unknown_label_field | covered | not-run |
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
| A block beside two rows | crates/mu-tui/tests/app.rs: a_block_beside_two_rows_gets_its_columns | covered | pass |
| Anchors that are not neighbors | crates/mu-config/tests/layout.rs: anchors_that_are_not_neighbors | covered | pass |
| An id in a later line | crates/mu-cli/tests/client.rs: an_id_in_a_later_line | covered | pass |
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
| The default layout | crates/mu-cli/tests/config_store.rs: the_default_layout | covered | pass |
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
| An open brace | crates/mu-cli/tests/config_store.rs: an_open_brace | covered | pass |
| A label too long | crates/mu-cli/tests/config_store.rs: a_label_too_long | covered | pass |
| A world with its own layout | crates/mu-cli/tests/client.rs: a_world_with_its_own_layout | covered | pass |
| The focus moves to a world with no entry | crates/mu-cli/tests/client.rs: the_focus_moves_to_a_world_with_no_entry | covered | pass |
| A layout for a game | crates/mu-cli/tests/client.rs: a_layout_for_a_game | covered | pass |
| The world entry comes before the game entry | crates/mu-cli/tests/client.rs: the_world_entry_comes_before_the_game_entry | covered | pass |
| No world in focus | crates/mu-cli/tests/client.rs: no_world_in_focus_takes_the_default_layout; crates/mu-cli/tests/client.rs: no_world_in_focus_for_layout_here | covered | not-run |
| A world keeps its text across a change of layout | crates/mu-cli/tests/client.rs: a_world_keeps_its_text_across_a_change_of_layout | covered | pass |
| The default segments | crates/mu-cli/tests/client.rs: the_default_segments | covered | pass |
| A narrow terminal | crates/mu-tui/tests/app.rs: a_narrow_statusline | covered | pass |
| The typed count | crates/mu-tui/tests/app.rs: the_typed_count | covered | pass |
| The state after a disconnect | crates/mu-cli/tests/client.rs: the_state_after_a_disconnect | covered | pass |
| The idle time after a send | crates/mu-cli/tests/client.rs: the_idle_time_after_a_send | covered | pass |
| The connected time | crates/mu-cli/tests/client.rs: the_connected_time | covered | pass |
| No connected time while disconnected | crates/mu-tui/tests/app.rs: no_connected_time_while_disconnected | covered | pass |
| No worlds | crates/mu-tui/tests/app.rs: no_world_in_the_statusline | covered | pass |
| Under an hour | crates/mu-tui/tests/app.rs: a_time_under_an_hour; a_time_one_second_under_an_hour | covered | not-run |
| Over an hour | crates/mu-tui/tests/app.rs: a_time_over_an_hour; a_time_of_one_hour | covered | not-run |
| A tick moves the idle time | crates/mu-tui/tests/update.rs: a_tick_moves_the_idle_time | covered | pass |
| No tick without a time segment | crates/mu-tui/tests/update.rs: no_tick_without_a_time_segment | covered | pass |
| An unknown segment | crates/mu-cli/tests/config_store.rs: an_unknown_segment | covered | pass |
| A segment twice | crates/mu-cli/tests/config_store.rs: a_segment_twice | covered | pass |
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
| A drag past the smallest pane | crates/mu-tui/tests/app.rs: a_drag_past_the_smallest_pane | covered | pass |
| A drag down | crates/mu-tui/tests/app.rs: a_drag_down | covered | pass |
| A selection beside a divider | crates/mu-tui/tests/app.rs: a_selection_beside_a_divider | covered | pass |
| The letters | crates/mu-tui/tests/arrange.rs: the_letters | covered | pass |
| Typing in arrange mode | crates/mu-tui/tests/arrange.rs: typing_in_arrange_mode | covered | pass |
| The tabs go below the output | crates/mu-tui/tests/arrange.rs: the_tabs_go_below_the_output | covered | pass |
| A refused stack | crates/mu-tui/tests/arrange.rs: a_refused_stack | covered | pass |
| Add the statusline | crates/mu-tui/tests/arrange.rs: add_the_statusline | covered | pass |
| Undo | crates/mu-tui/tests/arrange.rs: undo | covered | pass |
| A letter bound to undo still picks | crates/mu-tui/tests/arrange.rs: a_letter_bound_to_undo_still_picks | partial | pass |
| A drag to the right edge | crates/mu-tui/tests/arrange.rs: a_drag_to_the_right_edge | covered | pass |
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
- No world in focus: no verification run names this row
- Under an hour: no verification run names this row
- Over an hour: no verification run names this row

## Verification runs

- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_server_table)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_unknown_field)'`
- exit 0: `cargo nextest run -p mu-config --test layer --locked -E 'test(=an_unknown_label_field)'`
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
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=anchors_that_are_not_neighbors)'`
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
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_open_brace)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_label_too_long)'`
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
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_time_under_an_hour)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_time_over_an_hour)'`
- exit 0: `cargo nextest run -p mu-tui --test update --locked -E 'test(=a_tick_moves_the_idle_time)'`
- exit 0: `cargo nextest run -p mu-tui --test update --locked -E 'test(=no_tick_without_a_time_segment)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_unknown_segment)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_segment_twice)'`
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
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_time_one_second_under_an_hour)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_time_of_one_hour)'`

## Dropped

None.

## Lens notes

- approach: R-22 (fixed): percent_range in crates/mu-config/src/layout/change.rs keeps the percent from 10 to 90 and both panes at their smallest size, and returns the plan unchanged when the bounds cross, so it is not raised again.
- approach: The stale-citation findings app-3 to app-11 are graded suggestion, not should-fix: each is a wrong name or signature in design.md with none of the five consequences.
- approach: Each test name of the D12 table that was searched exists at head.
- fit: TickWait (crates/mu-tui/src/model.rs:84) has the same two variants as ConfigRetry (model.rs:200). D7 asks for a state of idle or waiting for each, so this is recorded as information and not as a finding.
- fit: The repository rows of the library-preference table (openspec/config.yaml context) were read. The change takes winnow, toml_edit and tempfile through [workspace.dependencies], and no repository helper does an atomic write or a TOML field edit, so write.rs found no existing copy.
- fit: remote::call (crates/mu-cli/src/remote.rs) is the one private-socket call for mu config and mu layout, as task 3.9 asks.
- invariants: I1: A percent on a line is from 10 to 90 \| enforced at crates/mu-config/src/layout/tree.rs made() and parse.rs part_size \| boundary test a_percent_out_of_range (9, 91, 95), a_percent_of_90
- invariants: I2: A drag or a key never gives a percent outside 10..90 or a pane below its smallest size \| enforced at crates/mu-config/src/layout/change.rs percent_range and resize_within; the high bound is too strict (inv-1) \| boundary test a_border_stops_at_ten_percent, a_larger_block_stops_at_ninety_percent, a_border_in_a_split_of_20_columns, a_border_in_a_split_of_3_rows (none one past the limit)
- invariants: I3: An insert number is 1..=n+1, and a replace or delete number is 1..=n \| enforced at change.rs insert() and index() before any indexing \| boundary test an_insert_at_zero, an_insert_at_the_end, an_insert_past_the_end, a_delete_of_line_zero, a_replace_of_a_line_past_the_end
- invariants: I4: A plan has at most 32 lines and 8 levels of splits \| enforced at layout/mod.rs count_lines and tree.rs apply (levels) \| boundary test too_many_lines, the_largest_plan, a_tree_of_8_levels, a_tree_too_deep
- invariants: I5: An id has 1 to 32 characters, starts with a letter and is not a word \| enforced at layout/step.rs BlockId::new \| boundary test an_id_of_32_characters, an_id_of_33_characters, an_id_that_is_a_word_of_the_sentences
- invariants: I6: A count of cells is from 1 to 65535 and a max from 1 to 100 \| enforced at parse.rs part_size and tree.rs made() \| boundary test a_count_of_zero_cells, a_count_of_65535, a_count_of_65536, an_editor_max_of_1_and_100, a_bad_editor_size
- invariants: I7: A layout name follows the device name rule and is not a built-in name, . or .. \| enforced at crates/mu-config/src/names.rs LayoutName::new, used by store layout_state, save_layout, edit_layout and save_target \| boundary test a_saved_name_of_64_characters, a_saved_name_of_65_characters, a_device_named_dot, a_layout_name_of_one_dot
- invariants: I8: save_target never gives a name the store refuses \| enforced by type: save_target returns LayoutName::new \| the property a_save_target_is_a_good_name re-checks LayoutName::new on a LayoutName, so it holds by construction
- invariants: I9: A device write never leaves a bad layer \| enforced at crates/mu-server/src/store.rs put_choice (bad_at before, state() on the edited text before the rename) \| boundary test a_write_that_makes_a_bad_layer_is_refused, a_bad_device_layer
- invariants: I10: The server writes only under clients/layouts and clients/devices \| enforced at store.rs put_layout and put_choice, whose paths come from LayoutName and DeviceName, which allow no /; ensure_dir refuses a non-directory \| boundary test a_save_with_a_bad_name, a_save_with_the_name_of_two_dots, a_clients_path_that_is_a_file
- invariants: I11: A split never holds a split of its own direction, and each split has a part with no size \| enforced at tree.rs put_beside, beside() and without() \| boundary test any_layout_reads_back and a_screen_change_gives_the_wanted_screen (property)
- invariants: I12: solve never leaves the area and never overlaps \| enforced at solve.rs sizes() and split() with saturating arithmetic \| boundary test solve_stays_inside_the_area (0..300 by 0..100, counts to u16::MAX), two_large_counts_in_one_split
- invariants: I13: A tab label has 1 to 64 characters \| enforced at crates/mu-config/src/label.rs chars().count() \| boundary test an_empty_label, a_label_of_64_characters, a_label_of_64_wide_characters, a_label_too_long
- invariants: I14: A plain lowercase letter in arrange mode always picks a block \| enforced at crates/mu-tui/src/arrange.rs arrange_key, which tests the letter before the keymap \| boundary test a_letter_bound_to_undo_still_picks
- invariants: I15: The last editor cannot be hidden \| enforced at change.rs hide() (held <= row.min) \| boundary test hide_refuses_the_editor, hide_a_second_editor
- invariants: Probe: scratchpad probe/percent_range.py mirrors percent_range in Python against the round-down rule of D6. It gives code None vs real (48, 52) at 21 columns, (13, 86) vs (13, 87) at 80 columns, and (50, 50) vs (50, 74) at 4 rows. The enforcement is Rust, so the probe mirrors it and does not run it.
- invariants: No weakened check was found in the diff of merge_layers, the WorldId move or the keymap.
- test-coverage: Assertions were read for the arrange-mode, drag, input-focus-after-world-change and config-edit scenarios. The other rows are graded covered because a named test exists in the file that tasks.md names, with confidence 0.6 for them.
- test-coverage: The one-test command is in openspec/config.yaml, so tasks are not graded down for naming no command (R-35).
- test-coverage: The client-config scenario An unknown field and the client-layout scenario An unknown field share one heading, as do the two scenarios No world in focus. Each pair has one row.
- test-coverage: The only MODIFIED section is in client-config. No BREAKING requirement exists, so there is no breaking-unproven finding.
- test-honesty: Grep over crates found no #[ignore], todo!, unimplemented!, assert!(true) or \|\| true: no disabled or vacuous-gate finding.
- test-honesty: The proptest a_save_target_is_a_good_name in mu-config/tests/layout.rs checks its property only when save_target returns Ok, so it passes if save_target always errs. The example tests a_saved_name_of_64_characters and a_game_entry_gives_a_name_with_the_game cover the Ok path, so it is not raised as a finding.
- test-honesty: The is_err asserts in mu-cli/tests/layout.rs (a_save_with_a_bad_name, a_use_with_an_empty_name_and_no_key) also assert that no file is written, so they are accepted.

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
| R-57 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D11. Streams, and what #102 adds"],"gives the id of the `output` for a stream: the `output` with that id, or the first `output` of the plan"] | resolved | resolved in round 4 from fixed 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | `route` has no answer for a plan with no `output`, which the spec allows (the layout `editor` alone); no decision says where a world line or a notice goes then, and the two route tests cover only plans with an output. |
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
| R-96 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"An unknown field"] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | covered row 'An unknown field' has no verification run |
| R-97 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"The editor after the notices"] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | covered row 'The editor after the notices' has no verification run |
| R-98 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"The editor before the notices"] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | covered row 'The editor before the notices' has no verification run |
| R-99 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"No world in focus"] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | covered row 'No world in focus' has no verification run |
| R-100 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"Under an hour"] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | covered row 'Under an hour' has no verification run |
| R-101 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"Over an hour"] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | covered row 'Over an hour' has no verification run |
| R-102 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"approach partial"] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | lens approach reported partial, gaps: 3 |
| R-103 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"test-honesty partial"] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | lens test-honesty reported partial, gaps: 2 |
| R-104 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"invariants partial"] | fixed | f663eaeeb5ce9fba12d8dd5767d7f88958b1d2b3 | 6 |  |  | lens invariants reported partial, gaps: 2 |

## Reviewed files

| path | content |
|---|---|
| Cargo.lock | fb3f774566e3792dc3f09b879ba8b237a2c73cd6 |
| Cargo.toml | 9c941dec15e79db61ba820e3e17faf13abf2f11a |
| architecture/model.c4 | f0e5a47894b832d4143111d80038c5aa918e6eee |
| architecture/views.c4 | 806536588b18b5861f18eff3f24b8cd30ab71aa0 |
| crates/mu-cli/Cargo.toml | 15bf06e9513529c4a630b520070d6f6a2980c4a8 |
| crates/mu-cli/src/config_verb.rs | e26575f54193b4632f2cf2c1b5b28932ec98a4f9 |
| crates/mu-cli/src/layout_verb.rs | c8748eb7e07dd2e4690c0d9635aa1a529df5bb52 |
| crates/mu-cli/src/lib.rs | 0d47527b1944ed448a29d739ab2d93370f08fcc0 |
| crates/mu-cli/src/remote.rs | 7a523c62388d9ffc3f9c2c6497d16b38c6174acd |
| crates/mu-cli/tests/cli.rs | 7797d83b8a7c4b599925ecce556bca6dc23de847 |
| crates/mu-cli/tests/client.rs | 76e91962347f72d8edc2d2dd12fa8ff98077df0e |
| crates/mu-cli/tests/config_store.rs | 8a7805eab0f2d8b365573ca29aa141856abdddd4 |
| crates/mu-cli/tests/layout.rs | 7103f22d3b39aa4d0020e0ebcf5833855ad9db91 |
| crates/mu-cli/tests/workspace_boundary.rs | 8bb68830ea58ccf595b37f60e5f6afadbcca8ac1 |
| crates/mu-client/src/lib.rs | e5993a64c4d28d09133da7708c9920aa92718edf |
| crates/mu-config/Cargo.toml | 2b11bb4e888bd4ecd3e6da65bec8760b2466f9d4 |
| crates/mu-config/src/entries.rs | 173656ccc3da020a40c5ac3bd5738ef1d18ee64f |
| crates/mu-config/src/label.rs | d7565b753582c6edbb8455891e7ea7ee06194ffc |
| crates/mu-config/src/layer.rs | 3d1f69cd2e17f8b9b62ee67b849f03da4c51f18f |
| crates/mu-config/src/layout/block.rs | 517cada593c0a77968a5f847ce006487717e5a14 |
| crates/mu-config/src/layout/builtin.rs | 0837059c61e290513f61c4473ddae5150678e4bd |
| crates/mu-config/src/layout/change.rs | 79c67c639f1a4442158f64d92bb2d9282c7eea31 |
| crates/mu-config/src/layout/mod.rs | 177e936bed83b199710702f8f505c92d7e12df1d |
| crates/mu-config/src/layout/parse.rs | 5c31fa357e2adaa39939fa78ea26655583fcc32d |
| crates/mu-config/src/layout/solve.rs | cc3ca0d7c96e07f762c9e15f938b5d97a6121220 |
| crates/mu-config/src/layout/step.rs | 5a855e1068c9142fef75e73f52ebc910fdf43d2a |
| crates/mu-config/src/layout/tree.rs | 234eec42524fe22e023105c526b9f836e419e2a0 |
| crates/mu-config/src/layout/write.rs | db02174d0541c4c79d6ea4b8d0c05686f4b9ace5 |
| crates/mu-config/src/lib.rs | 010df7d12ca79d0b1ca9558413323aff1186b850 |
| crates/mu-config/src/names.rs | 01fd4c5f300c09a0bf0da3e82673e6193820195f |
| crates/mu-config/src/world_id.rs | b5586c5e9c5e7c3fecaca68986da8ec47e3e1119 |
| crates/mu-config/tests/layer.rs | 9b34d6a6e204bf76520efcb5abafd505a6c7c8cc |
| crates/mu-config/tests/layout.proptest-regressions | 554ac208771e9ba606ab342bc66c8ab000597490 |
| crates/mu-config/tests/layout.rs | a9cd6b4c75707c2a4d4b8c0cf955b853b883f9e7 |
| crates/mu-config/tests/names.rs | a3bf5e9376ac783fe25303f3f603aaf66f888785 |
| crates/mu-core/Cargo.toml | 89937e8493ca6d837f8eb28a58ff84981adc7d8f |
| crates/mu-core/src/id.rs | 453174368922ff0a1b221f12e9dabe7de3b907b0 |
| crates/mu-core/src/world/merge.rs | 92c82461f2c8b54c6cde46715e4eda6e05367d83 |
| crates/mu-proto/proto/mu/v1/portal.proto | 58f2ac83fdf68b02ca8ed30cb9efd7dd53f4e009 |
| crates/mu-server/Cargo.toml | a53455fa1b31b187adcf337d18ed20c05f497e6d |
| crates/mu-server/src/lib.rs | 8428a97973fcb851b3de9aaf01748fb20d847625 |
| crates/mu-server/src/service.rs | 20077fd5b72a6c0ddea13d159c0207a174fd0e14 |
| crates/mu-server/src/store.rs | 6f2ddff96360f2024a37cb0cc1a9936928068256 |
| crates/mu-server/src/write.rs | e121f8f9cb795d1215d72d4653594b24ced935b4 |
| crates/mu-server/tests/store.rs | ede35486d76925e09c0a15a35a8cd41b297eeed4 |
| crates/mu-tui/src/arrange.rs | 5efc06332ffcef72982c8773b99c15e0bae67fbe |
| crates/mu-tui/src/config.rs | 31dc99ee32b0d94efa67a8c214d408fd81db6681 |
| crates/mu-tui/src/editor.rs | bf65a6b7efd456bb35a3accc565eaca4b471a3c8 |
| crates/mu-tui/src/keymap.rs | 60e9ef84b8124b829806a350bc4145fc460bb4b2 |
| crates/mu-tui/src/layout.rs | absent |
| crates/mu-tui/src/layouts.rs | ba4074ed3729fcaadb25f0435b66d6f20f586922 |
| crates/mu-tui/src/lib.rs | 8e55962cb616d2f0f7dfdc89ce6353b6780640f2 |
| crates/mu-tui/src/model.rs | 34f34aa8f7dead6f186502dd75d5df5c02dd7b21 |
| crates/mu-tui/src/registry.rs | 6acfabe5ca36a9f4249aaeb78e8f63f7d6fb1ad3 |
| crates/mu-tui/src/run.rs | 5732277738b85a820dcead564d708427f77ddd3b |
| crates/mu-tui/src/statusline.rs | 79cc27bad1211f7c7c7049018a0bbc8a03389541 |
| crates/mu-tui/src/update.rs | a7e305068b06d70b8ea5eeea606d99e2bb91b2a3 |
| crates/mu-tui/src/view.rs | 6307aed91be434bcdb9f2d09e51f69ea6ee4099e |
| crates/mu-tui/tests/app.rs | 697e4bedd8a49e3d101c09db7ec7ddff9d430a82 |
| crates/mu-tui/tests/arrange.rs | 5a0f76daf79df64ecfb926d029fc7a8a5959b1de |
| crates/mu-tui/tests/editor.rs | c7992ae680d306d88c22f875eb27a1f19a575a86 |
| crates/mu-tui/tests/snapshots/app__a_percent_and_a_count_of_cells.snap | 551c7cbabc7e84f7c7f2d3cb69e97351ba373833 |
| crates/mu-tui/tests/snapshots/app__the_picture_of_each_built_in_layout.snap | fd1fea48a5b4d0818d1599c7c63f92659f8f8e74 |
| crates/mu-tui/tests/update.rs | 94a480debec7985c9c0214dd19674da0a256099e |
| docs/src/content/docs/architecture/server-and-clients.mdx | e26f7a7035c6adf0535056d29b03930cc195db77 |
| docs/src/content/docs/architecture/terminal-client.mdx | 85d2f6949411c56fe3e935541ebefcab776a2d6c |
| docs/src/content/docs/decisions/layout-model.md | 36f7206a7b1c80b03a505387060fdb887a29ef26 |
| docs/src/content/docs/guides/client.md | 1998f8025e8130dad06ff91480087d779112027e |
| docs/src/content/docs/guides/configuration.md | 1a6d9215d8fc6a948b4cd0eabebd881f0f5d9379 |
| docs/src/content/docs/guides/layouts.md | 201b4249b3526fd20d2190003c8917a4763eaec4 |
| openspec/changes/client-layout/.openspec.yaml | 880dcbddfbd01d2f90c77d58dcf594b444480b33 |
| openspec/changes/client-layout/design.md | c59cd7424552313108ff545c2ff1a7b0609b367e |
| openspec/changes/client-layout/proposal.md | d550f0fe0cdff8f42df13f17023995bb988e33f0 |
| openspec/changes/client-layout/specs/client-config/spec.md | 732875b42d24bcce5855b8e077ae1a36f615591b |
| openspec/changes/client-layout/specs/client-layout/spec.md | ccf85dc7d4056bb523a04c87b4530b1e348c1696 |
| openspec/changes/client-layout/specs/client-statusline/spec.md | ac3a3ba76dafc334996beeb1f82b4623b5a0b352 |
| openspec/changes/client-layout/specs/config-store/spec.md | 1a19c989ed3db5c593eb792eee96e03e2f1d6ae8 |
| openspec/changes/client-layout/specs/layout-editing/spec.md | 6d9fef7a3e9c3e335f05ed1d62a12de0b70b630c |
| openspec/changes/client-layout/specs/terminal-client/spec.md | a42fcbd9684d5fea1a7bf612a60c2f0e2ea56f95 |
| openspec/changes/client-layout/tasks.md | sha256:a64969ccfc3ca0cb5c8573b319b6b034c97dbae873c354a7c4f1dd63e656844e |
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
