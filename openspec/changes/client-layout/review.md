---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-25T03:06:39+00:00
head: 12c5e1eb51d612f7a7ad524730010c42c27e625a
round: 3
next_id: 73
lenses:
  spec-lint: ok
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
  verify: not-applicable
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-44 | approach | blocker | in-change | `/layout list` and `layout-next` need the names of every layout in the client, but no decision gives the client a ListLayouts call or an effect for it. GetClientConfigResponse carries only the layout in use and the world entries. | specs/layout-editing/spec.md > Requirement: The layout commands choose and name a layout — "`/layout list` SHALL show the name of each layout. The entry `layout-next` SHALL set the choice in use to the next layou" | Add to D9 an effect that calls ListLayouts from the client, and state that `/layout list` shows its names and `layout-next` takes the next name from its reply. |  |  | unmet-requirement |
| R-45 | approach | blocker | in-change | The THEN of 'A size of the first block' needs a notice that names `notices`, the sized block beside the first block. D3 refuses the plan through build and the check, whose error names only the line and the word, so no decision produces the names of the blocks beside it. | specs/layout-editing/spec.md > Requirement: The pane commands hide and resize a block — "For the block of the first line, it SHALL change nothing, and SHALL show a notice that names the blocks beside it that t" | State in D3 that `resize` of the block of the first line returns an error that names the blocks of its splits that hold a size. |  |  | unmet-requirement |
| R-46 | approach | blocker | in-change | In 'A drag down' the notices take 12 of 24 rows; 4 rows less is 8 rows, 33.3%. A whole percent of 33 solves to 7 rows by the round-down rule of D6, so the pane is 5 rows smaller. Only rounding up to 34% gives 8 rows, and D3 names no rounding. | specs/layout-editing/spec.md > Scenario: A drag down — "the user drags the title row of `notices` down by 4 rows and releases" | State in D3 that `resize_border` takes the smallest whole percent whose solved size reaches the dragged size, and add the rows of 'A drag down' to the check of task 1.2. |  | severity? should-fix 0.69 | unmet-requirement |
| R-47 | approach | blocker | in-change | D7 makes the tick a fixed message `Msg::Tick(i64)` that `Effect::Wait` gives back, and names a clock read only for the first tick before the first draw. No decision says where a later tick gets the current time, so `now_ms` does not move and `idle` and `connected` do not count. | design.md > D7. Layer 2: the streams, the tab label and the statusline — "`Effect::RetryConfig` becomes `Effect::Wait { after, then }`, which carries the wait and the message to give back." | State in D7 that the run loop reads the clock when a wait for a tick ends and gives back `Msg::Tick` with that time. |  |  | wrong-behaviour |
| R-48 | approach | blocker | in-change | 'A file with a built-in name' needs a notice with `built-in` for `clients/layouts/classic.toml`. D5 checks the name of a layout file with the rule of a device name, which accepts `classic`, and the reader of D4 knows no file name; only `save_target` of D9 refuses a built-in name. | specs/config-store/spec.md > Scenario: A file with a built-in name — "stderr holds one line that contains `clients/layouts/classic.toml` and `built-in`" | State in D5 that the store checks the name of a layout file with the check of a layout name of D9, which refuses a built-in name, `.` and `..`. |  | severity? should-fix 0.66 | unmet-requirement |
| R-49 | approach | blocker | in-change | A save from a built-in layout behind a world entry must set the entry, not the field `layout`. SaveLayout with a device sets only `layout`, and D9 says only that `Effect::SaveLayout` makes 'the call', so no decision makes the client set the entry after the write. | specs/layout-editing/spec.md > Scenario: A save from the built-in layout of one world — "it holds `layout = "classic"` with the source `default`" | State in D9 which calls `Effect::SaveLayout` makes for each result of `save_target`: SaveLayout with the device for the field, or SaveLayout and then UseLayout with the world key for an entry. |  | severity? should-fix 0.71 | unmet-requirement |
| R-50 | approach | should-fix | in-change | Task 3.8 names `the_first_view`, `the_order_of_the_worlds` and `no_worlds` in `client.rs`. At head no test has the first two names, and `first_view` and `no_worlds` are in `crates/mu-tui/tests/app.rs`, so the check that `classic` keeps the screen of today names tests that cannot run. | tasks.md > 3. Layer 2: named layouts — "and that `the_first_view`, `the_order_of_the_worlds` and `no_worlds` in `client.rs` still pass." | Name in task 3.8 the tests that cover the scenarios 'The first view', 'The order of the worlds' and 'No worlds' at head, with their files. |  |  | unmet-requirement |
| R-51 | fit | should-fix | in-change | The rule of a layout name has three statements: the store checks a layout file with the device name rule (D5), the write path adds a refusal of . and .. (D8), and save_target runs a check of a layout name with built-in names, . and .. (D9). No type holds the rule, so the reader accepts a file clients/layouts/..toml (name .) that the writer refuses. | design.md > D5. Layer 2: layouts on the server — "`layer_rule` gets this place, with the name rule of a device." | Add a LayoutName type beside DeviceName in names.rs whose new() holds the device rule, the built-in names, . and .., and name it in D5 (the Place check), D8 step 1 and D9 save_target. |  |  | drift |
| R-52 | fit | should-fix | in-change | The verb layout must follow the private-socket rule of mu config, but the one function that does that check and call, call in config_verb.rs, is private to that verb, and no decision or task names it, so task 3.9 writes a second copy. | crates/mu-cli/src/config_verb.rs:65 — "fn call<T, F: Future<Output = Result<T, mu_client::ClientError>>>(" | State in task 3.9 that call moves out of config_verb.rs to a shared module of mu-cli, and that mu config and mu layout both use it. |  |  | drift |
| R-53 | fit | should-fix | in-change | The range of 10 to 90 percent for a resize is kept twice: in resize_border of mu-config, and in the keys < and > of arrange mode in mu-tui, which work out and clamp the percent themselves before they call resize. | design.md > D3. Layer 1: the changes — "The keys work out the new percent, keep it from 10 to 90, and then call `resize`." | Put the clamp in one function of mu-config that resize_border and the keys < and > both call, and name it in D3. |  | severity? suggestion 0.77 | drift |
| R-54 | fit | should-fix | in-change | mu config show prints the keyed table [keys] under a table header, but the spec prints the keyed table [world-layouts] as dotted lines; D5 names neither the [keys] form nor a reason. render prints the bindings last, so a dotted line printed after the [keys] header reads as a binding. | crates/mu-cli/src/config_verb.rs:157 — "text.push_str("\\n[keys]\\n");" | Print [world-layouts] as a table like [keys] and change the scenarios, or state in D5 the dotted form, why, and that it prints before the [keys] header. |  |  | wrong-behaviour |
| R-55 | fit | should-fix | in-change | The store has one check of a client layer, state(), and config-store said a write runs that check before it lands. D8 checks the parts of the request and the old device file, but not the edited device text, and names neither state() nor a reason, so #173 must change the write path. | openspec/changes/archive/2026-11-24-config-store/design.md:18 — "A later write call (#173) uses the same check before a write lands." | In D8 step 3, run state() on the edited device text before the rename, and refuse the write with its notice if the layer is bad. |  |  | next-change |
| R-56 | invariants | should-fix | in-change | The input focus stays an editor of the layout on the screen only at apply_config and at a layout change; a change of the world focus to a world with another layout (D6) passes neither, so `input` can name an editor that is not on the screen. | design.md > D7a. Layer 2: several editors — "`apply_config` and each change of the layout set `input` to the first editor of the plan if the old one is gone." | Resolve `input` against `Model::layout()` at each read (fall back to its first editor), or reset it at each change of the world focus. Put focus-next and focus-prev with a [world-layouts] entry into the generator of `the_input_focus_is_an_editor_of_the_plan`, and add a test: `scratch` has the input focus under the layout of alpha/aria, focus-next to beta/bryn with `classic`, type x, the editor of beta/bryn holds x. |  |  | wrong-behaviour |
| R-57 | invariants | should-fix | in-change | `route` has no answer for a plan with no `output`, which the spec allows (the layout `editor` alone); no decision says where a world line or a notice goes then, and the two route tests cover only plans with an output. | design.md > D11. Streams, and what #102 adds — "gives the id of the `output` for a stream: the `output` with that id, or the first `output` of the plan" | State in D11 that `route` returns no output for a plan with none, and that the line is then not put in any pane (a notice stays in `Model::notices`). Add the unit test `a_plan_with_no_output` for `route`, and an app.rs test that a world line and a notice under the layout `editor` draw with no panic. |  |  | wrong-behaviour |
| R-58 | invariants | should-fix | in-change | `resize_border` keeps the percent from 10 to 90 and each pane at its smallest size, but in a split of 20 columns or less, or 3 rows or less, the two ranges cross (probe: W=20 needs p>=50 and p<=49); D3 states no rule for that case, and the only edge test uses 300 columns. | design.md > D3. Layer 1: the changes — "It keeps the percent from 10 to 90, and each pane at its smallest size or more." | State in D3 what `resize_border` does when no percent meets both rules (for example, keep the size as it is), so no clamp runs with crossed bounds (`Ord::clamp` panics if min > max). Add the unit tests `a_border_in_a_split_of_20_columns` and `a_border_in_a_split_of_3_rows`. |  | severity? blocker 0.70 | wrong-behaviour |
| R-59 | invariants | should-fix | in-change | The line edits take a line number from a command or an EditLayout call, but D3 states no range for it, and every named test uses a number in the middle; 0, n+1 for replace and delete, and n+2 for insert are not tested (a direct Vec::insert past the end panics). | design.md > D3. Layer 1: the changes — "\| `insert(number, step)` \| Puts a line at that number. \|" | State in D3 the ranges: insert from 1 to n+1, replace and delete from 1 to n, and an error that names the number for any other. Add the unit tests `an_insert_at_zero`, `an_insert_at_the_end`, `an_insert_past_the_end` and `a_delete_of_a_line_past_the_end`, and one EditLayout test in crates/mu-cli/tests/layout.rs with a number past the plan. |  |  | wrong-behaviour |
| R-60 | invariants | should-fix | in-change | The limit of 32 lines can never be the binding rule: the block counts cap a valid plan at 14 lines (4 editors, 8 outputs, tabs, statusline), so a plan of 33 lines also breaks a block count, and `too_many_lines` names `32` only if the count runs before `build`, which D4 does not order. | specs/client-layout/spec.md > Scenario: Too many lines — "stderr holds one line that contains `clients/layouts/mine.toml` and `32`" | Either state in D4 that the reader counts the lines before it parses any line, or replace the limit with the catalog cap. Add a test of the largest valid plan (14 lines) and keep `too_many_lines` asserting the count error. |  |  | unmet-requirement |
| R-61 | invariants | should-fix | in-change | The limit of 8 levels of splits has one test, `a_tree_too_deep`, with no stated depth; no test shows that a tree of exactly 8 levels is good, and no decision says how a level is counted, so 8 and 9 can each land on either side. | specs/client-layout/spec.md > Requirement: A bad sentence makes its layout bad — "more than 32 lines, or more than 8 levels of splits" | Define a level in D2 (for example, the root split is level 1), add the unit test `a_tree_of_8_levels` that builds, and make `a_tree_too_deep` build a tree of exactly 9 levels. |  |  | unmet-requirement |
| R-62 | invariants | should-fix | in-change | A count of cells has no upper bound in the spec or the grammar, and D6 adds counts in the solver; no test passes a count at or past the top of the integer type to the reader, or two such counts in one split to `solve`, where a debug build panics on overflow and a release build wraps. | specs/client-layout/spec.md > Requirement: The size of a block — "or a whole count of cells from 1" | Name the integer type of a count in D4, and make the reader refuse a count past it as a bad size naming the word. Make `solve` add with saturating arithmetic. Add the reader tests `a_count_of_65535` and `a_count_of_65536` (or the type's limits), a `solve` test with two such counts in one split, and let the generator of `solve_stays_inside_the_area` draw counts up to the type maximum. |  |  | wrong-behaviour |
| R-63 | invariants | should-fix | in-change | The tick starts once before the first draw and continues while the layout has a time segment; no test covers a model that gains a time segment later (a reload to `status`, `/layout use status`, or a focus change to a world entry of `status`), where a missing wait leaves the clock frozen. | design.md > D7. Layer 2: the streams, the tab label and the statusline — "If the layout holds `statusline`, and its segments hold `connected` or `idle`, `update` returns a wait of one second." | State in D7 that any `update` returns the wait when the layout in use has a time segment and no wait is pending. Add update.rs tests `a_new_layout_with_a_time_segment_starts_the_tick` (apply_config from classic to status) and `a_focus_change_to_a_status_layout_starts_the_tick`. |  |  | wrong-behaviour |
| R-64 | spec-lint | should-fix | in-change | THEN step 'the notices take the whole height' names no row count and the GIVEN and WHEN give no terminal size, so no test can check the screen it describes. | specs/client-layout/spec.md > Scenario: The editor after the notices — "the editor is below the output only, and the notices take the whole height" | Give the terminal size in the GIVEN or the WHEN, and state the rows of the notices and of the editor in the THEN, as the scenario 'A block beside two rows' does. |  |  | false-test |
| R-65 | spec-lint | should-fix | in-change | THEN step 'the editor takes the whole width' names no column count and the scenario gives no terminal size, so the screen it describes cannot be checked. | specs/client-layout/spec.md > Scenario: The editor before the notices — "the editor takes the whole width, and the notices are beside the output only" | Give the terminal size in the GIVEN or the WHEN, and state the columns of the editor and of the notices in the THEN. |  |  | false-test |
| R-66 | test-coverage | should-fix | in-change | The scenario 'A block beside two rows' checks columns 55 to 80, a divider in column 54 and the editor in columns 1 to 53. Its only test, in task 2.3, asserts the tree that build gives, which holds no column. | tasks.md > 2. Layer 1: blocks and the layout model — "Each of the three tests of a shape asserts the tree that `build` gives." | In task 3.4 add a `solve` test, `a_block_beside_two_rows_gets_its_columns` in `crates/mu-tui/tests/app.rs`, that asserts the rectangles of notices (columns 55 to 80, rows 2 to 23) and of the editor (columns 1 to 53, row 23) at 80x24. Or add the binary test to task 3.8. |  |  | false-test |
| R-67 | fit | suggestion | in-change | resolve builds the field paths of [keys] with one block for that table. [world-layouts] merges by key the same way, so the block repeats with another table name and a quoted key. | crates/mu-server/src/store.rs:327 — "paths.extend(keys.iter().map(\|key\| format!("keys.{key}")));" | Make the paths of a keyed table one function with the table name as its parameter, and the key quoted as TOML needs, for keys and world-layouts. |  |  |  |
| R-68 | fit | suggestion | in-change | The device edit of D8 knows two changes only, the field layout and one entry of [world-layouts]. #173 writes keys and echo into the same device file, so it must widen or wrap this edit. | design.md > D8. Layer 3: the write calls — "The change sets the field `layout`, or sets or removes one entry of `[world-layouts]`." | Shape the device edit as one function that sets or removes the value at one field path, such as layout or world-layouts.<key>, so #173 adds echo and keys.<key> with no new edit code. |  |  |  |
| R-69 | fit | suggestion | in-change | The registry lists the values of an argument by its kind in candidates(). D9 gives the new commands one free-text kind, so a block id or a layout name has no candidates, and the palette of #167 must change the kinds of these entries. | crates/mu-tui/src/registry.rs:261 — "pub fn candidates(model: &Model, kind: ArgKind) -> Vec<String> {" | Give the arguments that name a block or a layout their own kinds, with candidates from the plan and from the layout list, and keep free text for a sentence. |  | severity? should-fix 0.78 |  |
| R-70 | fit | suggestion | in-change | mu-config writes its errors by hand with Display and Error, and depends only on serde. D5 adds thiserror to mu-config for the one moved error type, and names neither the pattern of the crate nor a reason to leave it. | crates/mu-config/src/names.rs:22 — "impl std::error::Error for NameError {}" | Write Display and Error for BadWorldId by hand as NameError does, and drop thiserror from the row of mu-config, or state in D5 why mu-config changes its pattern. |  | severity? should-fix 0.60 |  |
| R-71 | invariants | suggestion | in-change | `save_target` must never give a name that the store refuses, over any device name and any world key; only example tests at 64 and 65 characters, a built-in name and `.` check it. | design.md > D9. Layer 4: commands and the mouse — "It runs the check of a layout name on that name" | Add a proptest property `a_save_target_is_a_good_name` in crates/mu-config/tests/layout.rs, after the strategies of crates/mu-core/src/id.rs: for any device name of the device rule and any world key, the result is an error or a name that the layout file rule accepts and that is not built in. |  |  |  |
| R-72 | invariants | suggestion | in-change | The device write keeps each other line of the file over an unbounded domain of device files, and only two example scenarios with one comment check it. | specs/config-store/spec.md > Requirement: UseLayout sets the layout of a device — "and SHALL keep each other line of the file, its comments included" | Add a proptest property in crates/mu-server/tests/store.rs that generates device layers with comments, `[keys]`, `[world-layouts]` and blank lines, sets and removes `layout` and one world entry, and checks that every other line is byte-equal. |  |  |  |

## Repeats

None.

## Out of bound

None.

## Accepted

None.

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | d2729150530ac1884073ee7207a64ed3a63ab418 | 0face2499054203abac6a0977c535eb9937058b31597b2276d6765a7042c028d | changes-requested | 35 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | a89e8ef5614c7ab7bd977258c84bc8ba8bc1ac31 | 215c0bd88e3b28033d1218fa69f8e88e090deebc492d274c174eaf4de3a135a3 | approve-with-notes | 8 | 0 | 1 | 33 | 0 |  |
| 3 | plan | full | 12c5e1eb51d612f7a7ad524730010c42c27e625a | b1034da5f8859358c6cfa265503a39cca17afab80d8900ab6706bea5c9cb57db | changes-requested | 29 | 0 | 0 | 0 | 0 | the operator asked for a full round to rebuild the coverage matrix after the fixes |

## Judge

judge: 58 questions, 0.003 USD

## Coverage matrix

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

## Verification runs

None.

## Dropped

- spe-3 (spec-lint, confidence 0.6): GIVEN 'any valid layout' names no concrete input; the scenario is a property with no named generator or fixture to check it against.

## Lens notes

- approach: The behaviour of `toml_edit`, `tempfile::NamedTempFile::persist`, the error type of `winnow` and the mouse events of termina were not checked against their documentation; tasks 1.4, 1.5 and 1.6 hold those checks.
- approach: The cited facts of Context, D5 to D9 and D12 hold at head: `Store::reload` takes the `reload` Mutex, `WorldId` lives in crates/mu-core/src/id.rs with `game()` and `character()`, the row of mu-server lists `tempfile`, `Cargo.lock` holds `winnow` through `toml`, the editor rule clamps to half the split, and the proto has `EVENT_KIND_CONNECTED`, `SOURCE_PLAYER` and `WORLD_STATE_RECONNECTING`.
- approach: R-22 (fixed) was checked: D3 now keeps 10 to 90 percent in `resize_border` and the keys, and `resize` refuses other percents; its defect is not raised again.
- fit: D5 moves the part rule of a world id to mu-config. mu-core world/dir.rs also uses is_part and PART_RULE for game file names, so the rule must be public in mu-config for one home.
- fit: D7 gives the tick its own state of idle or waiting beside ConfigRetry in model.rs. One general type for both fields keeps one copy; R-32 is resolved, so no finding is raised.
- invariants: I1: A layout holds editor 1 to 4 times, output at most 8, each other block at most once \| enforced at BLOCKS and build (task 2.1) \| boundary test the_editor_is_required, four_editors, five_editors, eight_outputs, nine_outputs
- invariants: I2: A plan has 1 to 32 lines \| enforced at the reader (D4, order not stated) \| boundary test too_many_lines (33) only; 32 is unreachable, see inv-5
- invariants: I3: A plan has at most 8 levels of splits \| enforced at build \| boundary test a_tree_too_deep only, see inv-6
- invariants: I4: An id is 1 to 32 characters, starts with a letter, is no word of the grammar, is unique \| enforced at the reader and build \| boundary test an_id_of_32_characters, an_id_of_33_characters, an_id_that_is_a_word_of_the_sentences, an_id_twice
- invariants: I5: A percent is 10 to 90 \| enforced at the reader and resize \| boundary test a_percent_out_of_range (9, 91, 95), a_percent_of_90, a_resize_out_of_range_is_refused
- invariants: I6: A count of cells is 1 or more \| enforced at the reader \| boundary test a_count_of_zero_cells; no upper edge, see inv-7
- invariants: I7: An editor max is 1 to 100 \| enforced at the reader \| boundary test a_bad_editor_size (max 0, max 101), an_editor_max_of_1_and_100
- invariants: I8: The first line takes no size; tabs and statusline refuse a size \| enforced at build and resize \| boundary test a_size_on_the_first_line, a_size_of_the_first_block_is_refused, a_size_on_the_tabs
- invariants: I9: Only outputs stack \| enforced at BLOCKS \| boundary test only_outputs_stack, the_editor_does_not_stack
- invariants: I10: Anchors of one line are neighbors in one split \| enforced at build \| boundary test anchors_that_are_not_neighbors
- invariants: I11: resize_border and the keys < > keep 10 to 90 and each pane at its smallest size \| enforced at resize_border (D3) \| boundary test a_border_stops_at_ten_percent (300 columns), a_larger_block_stops_at_ninety_percent, a_border_stops_at_the_smallest_pane; crossed ranges untested, see inv-3. R-22 (fixed) is fixed: D3 now clamps to 10 to 90
- invariants: I12: solve never fails, stays in the area, never overlaps \| enforced at solve (D6) \| boundary test solve_stays_inside_the_area (0x0 to 300x100), two_percents_past_the_split, a_count_past_the_screen; integer top untested, see inv-7
- invariants: I13: A written plan reads back to the same tree and ids; a written line parses to the same step \| enforced at the writer (D4) \| boundary test any_layout_reads_back, a_line_reads_back (property tests, proptest)
- invariants: I14: A screen change gives the wanted screen and a valid plan \| enforced at the three steps of D3 \| boundary test a_screen_change_gives_the_wanted_screen
- invariants: I15: The input focus is an editor of the layout on the screen \| enforced at apply_config and layout changes (D7a) \| boundary test the_input_focus_is_an_editor_of_the_plan; world focus changes not enforced, see inv-1
- invariants: I16: Each line goes into exactly one output \| enforced at route (D11) \| boundary test a_stream_goes_to_the_output_with_its_id, a_stream_with_no_output_goes_to_the_first; null case (no output) untested, see inv-2
- invariants: I17: hide refuses the last editor; delete refuses a line that a later line anchors on \| enforced at hide and delete (D3) \| boundary test hide_refuses_the_editor, a_delete_of_an_anchor_names_its_lines
- invariants: I18: A line number of a line edit is in range \| enforced at none (D3 states no range) \| boundary test none, see inv-4
- invariants: I19: A saved name is 1 to 64 characters, not built in, not . or .. \| enforced at save_target (D9) and the device name rule at crates/mu-config/src/names.rs:142 \| boundary test a_saved_name_of_64_characters, a_saved_name_of_65_characters, a_device_with_a_built_in_name, a_device_named_dot
- invariants: I20: The server writes only under clients/layouts and clients/devices \| enforced at the device name rule (no /) plus the . and .. check (D8) \| boundary test a_save_with_a_bad_name (../x), a_save_with_the_name_of_two_dots
- invariants: I21: A path under clients/ that is not a directory makes the write fail and write nothing \| enforced at step 2 of D8 \| boundary test a_clients_path_that_is_a_file
- invariants: I22: A device file keeps each other line after a write \| enforced at toml_edit (D8) \| boundary test a_device_file_with_a_comment, a_use_for_one_world; property candidate inv-9
- invariants: I23: A world key is one part or two parts with one / \| enforced at WorldKey (D5) \| boundary test a_key_with_two_slashes, a_key_with_an_empty_part, a_key_with_an_uppercase_letter
- invariants: I24: A tab label is 1 to 64 characters with known fields and balanced braces \| enforced at Label (D5) \| boundary test an_empty_label, a_label_of_64_characters, a_label_too_long, a_label_of_64_wide_characters, an_open_brace, a_lone_closing_brace
- invariants: I25: A segment list holds known segments, each once \| enforced at Segment (D5) \| boundary test an_unknown_segment, a_segment_twice
- invariants: I26: A time is M:SS under an hour and H:MM:SS from an hour, and 0:00 before the clock \| enforced at the statusline draw (D7) \| boundary test a_time_one_second_under_an_hour, a_time_of_one_hour, a_time_before_the_clock_is_zero
- invariants: I27: A tick runs only while a time segment shows, one at a time \| enforced at update and Effect::Wait (D7) \| boundary test no_tick_without_a_time_segment, one_tick_waits_at_a_time; the start of a tick after a layout change is untested, see inv-8
- invariants: I28: The writes and the reloads run one at a time \| enforced at the reload lock taken once by the write path (D8), crates/mu-server/src/store.rs:446 \| boundary test none needed beyond review
- invariants: Probe of inv-3: border.py under the handoff probe directory gives W=20 no percent (p>=50 and p<=49), W=21 48 to 52, W=300 10 to 90; rows.py gives H=3 none, H=4 50 to 74.
- invariants: The repository property-test library is proptest 1.12.0 (Cargo.toml line 52), already a dev-dependency of mu-config, mu-server, mu-tui and mu-core.
- invariants: A leftover temporary file of a crashed write in clients/layouts/ is kept by the snapshot (crates/mu-core/src/world/snapshot.rs keeps every file under clients/) and becomes a bad entry with a notice at each reload; not raised as a finding.
- spec-lint: Checked every scenario of the six delta specs for a WHEN or GIVEN line, a THEN line, and a scenario under each requirement: no malformed scenario found.
- spec-lint: Checked each requirement heading against the task lines by distinctive words: each requirement has a task line that names its tests. No untasked requirement found.
- spec-lint: Checked tasks.md for a named command, test or file: each tracked task names one. No unverifiable task found.
- spec-lint: Checked build targets against the repository: crates mu-config, mu-core, mu-tui, mu-server, mu-proto, mu-cli, mu-client and mu-fake-server exist; test files named in tasks.md exist or are marked new; mise tasks check and docs:check and the pnpm script model:check exist. No unknown target found.
- spec-lint: No BREAKING word appears in proposal.md or in any delta spec. The proposal's 'A step back to an older binary needs two manual steps' is not marked BREAKING, so no breaking-mismatch finding is raised.
- spec-lint: No assumed-answer finding: the one open question of design.md (the 40 by 12 picture size) is not assumed by any task.
- spec-lint: Form note outside this lens's classes: tasks.md numbers 1.6 before 1.5, and uses 3.1a, 3.2a, 3.5a, 3.5b and 5.2a after their parents. Ask the controller to decide whether a renumber is wanted.
- spec-lint: No finding of this round raises a row of prior.json.
- test-coverage: Scenarios named in several specs (An unknown field, No world in focus, An unknown layout, A device with a built-in name) have one row each, graded on the task that names the test for each.
- test-coverage: No requirement is BREAKING, so no breaking-unproven finding.

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
| R-8 | 1 | 2 | blocker | in-change | drift | ["artifact",["design.md","D5. Layer 2: layouts on the server"],"A key is a world id or a game name, and `mu-config` checks it with the rule of the world id."] | resolved | resolved in round 2 from fixed 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | fit-3: The plan adds a third parser of the world id in mu-config (the key check of D5, the game of layout_for, the game and character of the label and of save_target), beside WorldId in mu-core and the split in mu-tui; the design names no shared one.; app-4: D5 has mu-config check a key of [world-layouts] with the rule of the world id. That rule is pub(crate) in mu-core, and the crate rules give mu-config no member to depend on, so mu-config cannot call it. A second copy can refuse a key for a world that the server accepts.; inv-3: D5 checks a key of `[world-layouts]` with the rule of the world id, but that rule refuses a game name such as `alpha`, and mu-config may depend on no crate of the workspace, so it cannot call it. |
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
| R-44 | 3 | 3 | blocker | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Requirement: The layout commands choose and name a layout"],"`/layout list` SHALL show the name of each layout. The entry `layout-next` SHALL set the choice in use to the next layout in the order of the names"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | `/layout list` and `layout-next` need the names of every layout in the client, but no decision gives the client a ListLayouts call or an effect for it. GetClientConfigResponse carries only the layout in use and the world entries. |
| R-45 | 3 | 3 | blocker | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Requirement: The pane commands hide and resize a block"],"For the block of the first line, it SHALL change nothing, and SHALL show a notice that names the blocks beside it that take a size."] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The THEN of 'A size of the first block' needs a notice that names `notices`, the sized block beside the first block. D3 refuses the plan through build and the check, whose error names only the line and the word, so no decision produces the names of the blocks beside it. |
| R-46 | 3 | 3 | blocker | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Scenario: A drag down"],"the user drags the title row of `notices` down by 4 rows and releases"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? should-fix 0.69 | In 'A drag down' the notices take 12 of 24 rows; 4 rows less is 8 rows, 33.3%. A whole percent of 33 solves to 7 rows by the round-down rule of D6, so the pane is 5 rows smaller. Only rounding up to 34% gives 8 rows, and D3 names no rounding. |
| R-47 | 3 | 3 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D7. Layer 2: the streams, the tab label and the statusline"],"`Effect::RetryConfig` becomes `Effect::Wait { after, then }`, which carries the wait and the message to give back."] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | D7 makes the tick a fixed message `Msg::Tick(i64)` that `Effect::Wait` gives back, and names a clock read only for the first tick before the first draw. No decision says where a later tick gets the current time, so `now_ms` does not move and `idle` and `connected` do not count. |
| R-48 | 3 | 3 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Scenario: A file with a built-in name"],"stderr holds one line that contains `clients/layouts/classic.toml` and `built-in`"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? should-fix 0.66 | 'A file with a built-in name' needs a notice with `built-in` for `clients/layouts/classic.toml`. D5 checks the name of a layout file with the rule of a device name, which accepts `classic`, and the reader of D4 knows no file name; only `save_target` of D9 refuses a built-in name. |
| R-49 | 3 | 3 | blocker | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Scenario: A save from the built-in layout of one world"],"it holds `layout = \\"classic\\"` with the source `default`"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? should-fix 0.71 | A save from a built-in layout behind a world entry must set the entry, not the field `layout`. SaveLayout with a device sets only `layout`, and D9 says only that `Effect::SaveLayout` makes 'the call', so no decision makes the client set the entry after the write. |
| R-50 | 3 | 3 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","3. Layer 2: named layouts"],"and that `the_first_view`, `the_order_of_the_worlds` and `no_worlds` in `client.rs` still pass."] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | Task 3.8 names `the_first_view`, `the_order_of_the_worlds` and `no_worlds` in `client.rs`. At head no test has the first two names, and `first_view` and `no_worlds` are in `crates/mu-tui/tests/app.rs`, so the check that `classic` keeps the screen of today names tests that cannot run. |
| R-51 | 3 | 3 | should-fix | in-change | drift | ["artifact",["design.md","D5. Layer 2: layouts on the server"],"`layer_rule` gets this place, with the name rule of a device."] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The rule of a layout name has three statements: the store checks a layout file with the device name rule (D5), the write path adds a refusal of . and .. (D8), and save_target runs a check of a layout name with built-in names, . and .. (D9). No type holds the rule, so the reader accepts a file clients/layouts/..toml (name .) that the writer refuses. |
| R-52 | 3 | 3 | should-fix | in-change | drift | ["path","crates/mu-cli/src/config_verb.rs",65] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The verb layout must follow the private-socket rule of mu config, but the one function that does that check and call, call in config_verb.rs, is private to that verb, and no decision or task names it, so task 3.9 writes a second copy. |
| R-53 | 3 | 3 | should-fix | in-change | drift | ["artifact",["design.md","D3. Layer 1: the changes"],"The keys work out the new percent, keep it from 10 to 90, and then call `resize`."] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? suggestion 0.77 | The range of 10 to 90 percent for a resize is kept twice: in resize_border of mu-config, and in the keys < and > of arrange mode in mu-tui, which work out and clamp the percent themselves before they call resize. |
| R-54 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-cli/src/config_verb.rs",157] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | mu config show prints the keyed table [keys] under a table header, but the spec prints the keyed table [world-layouts] as dotted lines; D5 names neither the [keys] form nor a reason. render prints the bindings last, so a dotted line printed after the [keys] header reads as a binding. |
| R-55 | 3 | 3 | should-fix | in-change | next-change | ["path","openspec/changes/archive/2026-11-24-config-store/design.md",18] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The store has one check of a client layer, state(), and config-store said a write runs that check before it lands. D8 checks the parts of the request and the old device file, but not the edited device text, and names neither state() nor a reason, so #173 must change the write path. |
| R-56 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7a. Layer 2: several editors"],"`apply_config` and each change of the layout set `input` to the first editor of the plan if the old one is gone."] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The input focus stays an editor of the layout on the screen only at apply_config and at a layout change; a change of the world focus to a world with another layout (D6) passes neither, so `input` can name an editor that is not on the screen. |
| R-57 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D11. Streams, and what #102 adds"],"gives the id of the `output` for a stream: the `output` with that id, or the first `output` of the plan"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | `route` has no answer for a plan with no `output`, which the spec allows (the layout `editor` alone); no decision says where a world line or a notice goes then, and the two route tests cover only plans with an output. |
| R-58 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Layer 1: the changes"],"It keeps the percent from 10 to 90, and each pane at its smallest size or more."] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? blocker 0.70 | `resize_border` keeps the percent from 10 to 90 and each pane at its smallest size, but in a split of 20 columns or less, or 3 rows or less, the two ranges cross (probe: W=20 needs p>=50 and p<=49); D3 states no rule for that case, and the only edge test uses 300 columns. |
| R-59 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Layer 1: the changes"],"\| `insert(number, step)` \| Puts a line at that number. \|"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The line edits take a line number from a command or an EditLayout call, but D3 states no range for it, and every named test uses a number in the middle; 0, n+1 for replace and delete, and n+2 for insert are not tested (a direct Vec::insert past the end panics). |
| R-60 | 3 | 3 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: Too many lines"],"stderr holds one line that contains `clients/layouts/mine.toml` and `32`"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The limit of 32 lines can never be the binding rule: the block counts cap a valid plan at 14 lines (4 editors, 8 outputs, tabs, statusline), so a plan of 33 lines also breaks a block count, and `too_many_lines` names `32` only if the count runs before `build`, which D4 does not order. |
| R-61 | 3 | 3 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: A bad sentence makes its layout bad"],"more than 32 lines, or more than 8 levels of splits"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The limit of 8 levels of splits has one test, `a_tree_too_deep`, with no stated depth; no test shows that a tree of exactly 8 levels is good, and no decision says how a level is counted, so 8 and 9 can each land on either side. |
| R-62 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-layout/spec.md","Requirement: The size of a block"],"or a whole count of cells from 1"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | A count of cells has no upper bound in the spec or the grammar, and D6 adds counts in the solver; no test passes a count at or past the top of the integer type to the reader, or two such counts in one split to `solve`, where a debug build panics on overflow and a release build wraps. |
| R-63 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7. Layer 2: the streams, the tab label and the statusline"],"If the layout holds `statusline`, and its segments hold `connected` or `idle`, `update` returns a wait of one second."] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The tick starts once before the first draw and continues while the layout has a time segment; no test covers a model that gains a time segment later (a reload to `status`, `/layout use status`, or a focus change to a world entry of `status`), where a missing wait leaves the clock frozen. |
| R-64 | 3 | 3 | should-fix | in-change | false-test | ["artifact",["specs/client-layout/spec.md","Scenario: The editor after the notices"],"the editor is below the output only, and the notices take the whole height"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | THEN step 'the notices take the whole height' names no row count and the GIVEN and WHEN give no terminal size, so no test can check the screen it describes. |
| R-65 | 3 | 3 | should-fix | in-change | false-test | ["artifact",["specs/client-layout/spec.md","Scenario: The editor before the notices"],"the editor takes the whole width, and the notices are beside the output only"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | THEN step 'the editor takes the whole width' names no column count and the scenario gives no terminal size, so the screen it describes cannot be checked. |
| R-66 | 3 | 3 | should-fix | in-change | false-test | ["artifact",["tasks.md","2. Layer 1: blocks and the layout model"],"Each of the three tests of a shape asserts the tree that `build` gives."] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The scenario 'A block beside two rows' checks columns 55 to 80, a divider in column 54 and the editor in columns 1 to 53. Its only test, in task 2.3, asserts the tree that build gives, which holds no column. |
| R-67 | 3 | 3 | suggestion | in-change |  | ["path","crates/mu-server/src/store.rs",327] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | resolve builds the field paths of [keys] with one block for that table. [world-layouts] merges by key the same way, so the block repeats with another table name and a quoted key. |
| R-68 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","D8. Layer 3: the write calls"],"The change sets the field `layout`, or sets or removes one entry of `[world-layouts]`."] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The device edit of D8 knows two changes only, the field layout and one entry of [world-layouts]. #173 writes keys and echo into the same device file, so it must widen or wrap this edit. |
| R-69 | 3 | 3 | suggestion | in-change |  | ["path","crates/mu-tui/src/registry.rs",261] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? should-fix 0.78 | The registry lists the values of an argument by its kind in candidates(). D9 gives the new commands one free-text kind, so a block id or a layout name has no candidates, and the palette of #167 must change the kinds of these entries. |
| R-70 | 3 | 3 | suggestion | in-change |  | ["path","crates/mu-config/src/names.rs",22] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  | severity? should-fix 0.60 | mu-config writes its errors by hand with Display and Error, and depends only on serde. D5 adds thiserror to mu-config for the one moved error type, and names neither the pattern of the crate nor a reason to leave it. |
| R-71 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","D9. Layer 4: commands and the mouse"],"It runs the check of a layout name on that name"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | `save_target` must never give a name that the store refuses, over any device name and any world key; only example tests at 64 and 65 characters, a built-in name and `.` check it. |
| R-72 | 3 | 3 | suggestion | in-change |  | ["artifact",["specs/config-store/spec.md","Requirement: UseLayout sets the layout of a device"],"and SHALL keep each other line of the file, its comments included"] | fixed | 166817131d4bbd2290dc7b47ba4f6981b5e13133 | 3 |  |  | The device write keeps each other line of the file over an unbounded domain of device files, and only two example scenarios with one comment check it. |
