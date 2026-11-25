---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-25T02:40:31+00:00
head: d2729150530ac1884073ee7207a64ed3a63ab418
round: 1
next_id: 36
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
| R-1 | approach,fit | blocker | in-change | app-1: No decision says how the first output draws, scrolls and selects over several streams in the order of arrival. D11 keeps one PaneState for each stream, and a PaneState holds its own line store, scroll anchor, selection and drawn rows, so two of them cannot draw as one merged pane.; fit-2: D11 makes the notices a stream with its own pane state, but the model already keeps the notices a second time in Model::notices and replays them into a world pane; the design does not say that the vector and the replay go away. | design.md > D11. Streams, and what #102 adds — "The model keeps one pane state for each pair of a world and a stream, and one for the stream `notices`." | app-1: State in D11 the state behind a merged output: either one line store that receives the lines of each stream it shows (and how a later output for that stream takes them), or a merged view with one scroll anchor and one selection over several stores.; fit-2: State in D11 that the stream notices is the one store of the notices, that Model::notices and replay_notices go away, and that Model::notices() reads the stream. |  | severity? should-fix 0.70 | unmet-requirement |
| R-2 | approach | blocker | in-change | The scenarios of a save into a directory with no `clients` directory need the server to make `clients/layouts/` and `clients/devices/`. The write path of D8 writes a temporary file in `clients/layouts/` and has no step that makes a missing directory. | design.md > D8. Layer 3: the write calls — "A layout file is written whole to a temporary file in `clients/layouts/`, and then renamed over the target with `tempfil" | Add to step 2 of D8 that the server makes `clients/`, `clients/layouts/` and `clients/devices/` when they are missing, and what it does when one of them is not a directory. |  |  | unmet-requirement |
| R-3 | approach | blocker | in-change | Each scenario of a bad layout file that runs `mu layout show mine` needs the notice of that file on stderr. Task 3.9 builds show over ListLayouts, which the spec limits to good files, and no decision adds a call or a field that returns the notice of a bad layout file. | specs/config-store/spec.md > Requirement: mu layout prints layouts — "If the name is of a bad layout file, it SHALL write the notice of the file to stderr and exit with status 1." | Add to D8 a field of ListLayoutsResponse, or a reply of DescribeLayout, that carries the notice of each bad layout file, and say that `mu layout show` prints it and exits 1. |  |  | unmet-requirement |
| R-4 | approach | blocker | in-change | No decision sizes a part that holds several editors side by side. D6 keeps the editor rule of today, which sizes one editor leaf in a split of rows and gives Min(0) in a split of columns, and its solver takes one input of editor rows, while each editor wraps at its own width. | design.md > D6. Layer 2: how the client draws — "The solver is a pure function of `mu-config`, `solve(layout, area, editor rows) -> Vec<Placed>`." | State in D6 the height of a split part that holds editors side by side (for example the most rows of its editors), and give solve a function from an editor id and a width to its rows in place of one count. |  |  | unmet-requirement |
| R-5 | approach | blocker | in-change | The requirement of a bad tab label has no decision. D5 adds only `layout` and `world_layouts` to ClientLayer, and the layer check in state() checks only [keys], so nothing in the design makes a bad `tabs.label` make its layer bad on the server. | specs/client-layout/spec.md > Requirement: A bad tab label makes its layer bad — "The layer SHALL be bad, and its notice SHALL name the file and `tabs.label`, in each of these cases:" | Add to D5 the table [tabs] of ClientLayer, its field path `tabs.label`, and the check of the Label in state() with a notice that names `tabs.label`. |  |  | unmet-requirement |
| R-6 | approach | blocker | in-change | The requirements of a bad statusline and of the list merge have no decision. D5 names no [statusline] table of ClientLayer, no field paths `statusline.left` and `statusline.right`, and no check of the segments in state(). | specs/client-statusline/spec.md > Requirement: A bad statusline makes its layer bad — "The layer SHALL be bad, and its notice SHALL name the file and the field, in each of these cases:" | Add to D5 the table [statusline] with `left` and `right` as two field paths that merge one by one, and the check of the segments in state() with a notice that names the field. |  |  | unmet-requirement |
| R-7 | approach | blocker | in-change | No decision produces the notice of `/layout use` when a world entry is the choice in use. D9 names the save rule, `layout-here` and `layout-next`, and says nothing of `/layout use` behind an entry. | specs/layout-editing/spec.md > Requirement: The layout here command sets the layout of one world — "If an entry is the choice in use, `/layout use` SHALL also show a notice that names `/layout here`." | Add to D9 that `layout-use` calls layout_for, and gives the notice when the choice in use is a world or game entry. |  | severity? should-fix 0.84 | unmet-requirement |
| R-8 | approach,fit,invariants | blocker | in-change | fit-3: The plan adds a third parser of the world id in mu-config (the key check of D5, the game of layout_for, the game and character of the label and of save_target), beside WorldId in mu-core and the split in mu-tui; the design names no shared one.; app-4: D5 has mu-config check a key of [world-layouts] with the rule of the world id. That rule is pub(crate) in mu-core, and the crate rules give mu-config no member to depend on, so mu-config cannot call it. A second copy can refuse a key for a world that the server accepts.; inv-3: D5 checks a key of `[world-layouts]` with the rule of the world id, but that rule refuses a game name such as `alpha`, and mu-config may depend on no crate of the workspace, so it cannot call it. | design.md > D5. Layer 2: layouts on the server — "A key is a world id or a game name, and `mu-config` checks it with the rule of the world id." | fit-3: Add to D5 one world-id type in mu-config with the part rule, game() and character(). Use it in the key check, layout_for, Label, save_target and the order of mu-tui. Either give mu-core the edge to mu-config for WorldId, or name a tracked task that does.; app-4: State in D5 where the one rule of a world id lives: move the part rule to mu-config and have mu-core call it, or name the copy and the test that keeps the two the same.; inv-3: Name in D5 a key check of mu-config that accepts one part, or two parts joined by one `/`, each part by the rule of a world id part, and add the unit tests `a_game_key_is_good`, `a_key_with_an_empty_part` and `a_key_with_an_uppercase_letter` to task 3.2a. |  |  | drift |
| R-9 | spec-lint,test-coverage | blocker | in-change | tes-1: The scenario 'An id in a later line' (tabs above an id defined by an earlier line) has no task and no named test.; spe-2: The THEN step names `notices`, but the layout of the scenario gives that block no line; its only output is `side`, so the step cannot be checked. | specs/client-layout/spec.md > Scenario: An id in a later line — "`output as side right of output` and `tabs above side`" | tes-1: Add a unit test `an_id_in_a_later_line` in crates/mu-config/tests/layout.rs to task 2.3 (or 2.2) that reads the four lines and asserts the tabs sit above the notices only.; spe-2: Change the THEN to name the block by its id, such as 'the tabs are above side only', or add an `output as notices` line so that notices exist. |  | severity? should-fix 0.77 | unmet-requirement |
| R-10 | approach | should-fix | deviates-design | The client writes `layout` and `[world-layouts]` into device layers and adds `clients/layouts/`. A rollback of the binary reads them with deny_unknown_fields and the old layer rule, so the device layer turns bad and drops its echo and keys, and each layout file gives a notice. The plan has no rollback step. | design.md > Migration Plan — "None. A configuration directory with no new field gives the screen of today."; scope basis: design.md > Migration Plan — "None. A configuration directory with no new field gives the screen of today." | Add a rollback step: before a rollback, run `mu layout forget` and remove `layout` from each device layer, or name the fields to delete and that `clients/layouts/` can stay with a notice. |  |  | wrong-behaviour |
| R-11 | approach | should-fix | in-change | D11 calls the merge of one global stream `notices` into the first output the screen of today. At head a notice goes into the pane of the world in focus only, so it does not show under another world. A global notices stream shows every notice under every world. | design.md > D11. Streams, and what #102 adds — "The first `output` merges its streams in the order of arrival, which is the screen of today." | Either keep a notices stream for each world, or drop the claim and state in D11 that a notice now shows under each world. Say also where Model::hint goes, which pushes to the pane in focus too. |  |  | wrong-behaviour |
| R-12 | approach | should-fix | in-change | D8 runs each write behind the lock that the reload uses and then runs the reload of ReloadConfig. At head Store::reload takes that same tokio Mutex itself, which is not reentrant, so the write path as written waits on itself and the save never returns. | design.md > D8. Layer 3: the write calls — "The writes run one at a time behind the lock that the reload uses." | Name in D8 the part of the reload that runs under the held lock, such as the snapshot and reload_from, so the write and the reload take the lock once. |  | severity? blocker 0.93 | wrong-behaviour |
| R-13 | fit | should-fix | in-change | D7a gives each editor a block id and an input field of type BlockId, but mu-tui already names editors with EditorId and keeps non-pose editors in Model::editors; the design keeps both names for one editor. | design.md > D7a. Layer 2: several editors — "`Model` gets `input: BlockId`, the editor with the input focus." | State in D7a that the block id of mu-config replaces EditorId, that Model::editors and EditorId::POSE go away with Node, Leaf and Size, and that Model::editor takes the block id. |  |  | drift |
| R-14 | invariants | should-fix | in-change | D2 says each split keeps a part that shares the rest, but nothing stops the sizes of one split from passing its width: two 60% parts, or a count of 500 rows, leave the anchor side a negative rest. | design.md > D2. Layer 1: the plan is the layout — "The anchor side never has a size, so each split always has a part that shares the rest." | State in D6 what the solver does when the sizes of a split pass its cells (take them in order, clamp each to what is left, keep the smallest size of the anchor side), and either refuse percents of one split that add to 100% or more in the check of D4 or state that the solver clamps them. Add the unit tests `two_percents_past_the_split` and `a_count_past_the_screen` to task 2.3 or 3.4. |  |  | wrong-behaviour |
| R-15 | invariants | should-fix | in-change | The tab label limit of 64 characters is tested only at 65; no test shows that 64 characters are good, or that 64 characters of more than one byte each are good. | specs/client-layout/spec.md > Requirement: The tab label — "The table `[tabs]` SHALL hold `label`, a template of 1 to 64 characters." | Add the unit tests `a_label_of_64_characters` and `a_label_of_64_wide_characters` (64 times `é`, 128 bytes) to task 2.5, both good. The nearby device name rule counts bytes, so the wide case guards a copy of it. |  |  | unmet-requirement |
| R-16 | invariants | should-fix | in-change | The limit of 64 characters on a saved name is tested only with a name of 71 characters; no test passes a name of exactly 64 (good) or 65 (bad) to `save_target`. | specs/layout-editing/spec.md > Scenario: A name that is too long — "the device name of the client has 60 characters" | Add the unit tests `a_saved_name_of_64_characters` (good) and `a_saved_name_of_65_characters` (error) to task 5.3. |  |  | unmet-requirement |
| R-17 | invariants | should-fix | in-change | The switch from `M:SS` to `H:MM:SS` at one hour is tested at 75 and 3725 seconds only; neither 3599 nor 3600 seconds is tested. | specs/client-statusline/spec.md > Requirement: The format of a time — "A time SHALL show as `M:SS` under one hour and as `H:MM:SS` from one hour." | Add the unit tests `a_time_one_second_under_an_hour` (3599 gives `59:59`) and `a_time_of_one_hour` (3600 gives `1:00:00`) to task 3.7. |  |  | unmet-requirement |
| R-18 | invariants | should-fix | in-change | The floor of 2 rows for a drag of a title row has no test: `a_drag_down` moves 4 rows of a 50% pane and stays inside the range. | specs/layout-editing/spec.md > Requirement: A drag of a title row resizes two panes — "Neither pane SHALL get smaller than 2 rows." | Add the test `a_drag_down_past_the_smallest_pane` to task 5.5: drag the title row of `notices` to the last row, and assert that the pane keeps 2 rows. |  |  | unmet-requirement |
| R-19 | invariants | should-fix | in-change | Arrange mode reads the undo key from the keymap, and `[keys]` may bind a plain lowercase letter, so a player who binds `a` to `undo` loses either the pick of block A or the undo. Nothing keeps the letters free. | specs/layout-editing/spec.md > Requirement: Arrange mode moves a block with three keys — "The keys of the mode that are not letters leave each letter free for a block." | State in D10 that a letter in arrange mode always picks a block, and that the mode ignores an `undo` binding to a plain letter. Add the test `a_letter_bound_to_undo_still_picks` to task 6.2. |  |  | wrong-behaviour |
| R-20 | invariants | should-fix | in-change | `save_target` names a save after the device, but the device name rule accepts `classic`, `focus`, `status`, `sidebar`, `.` and `..`, which the layout name check refuses, so every save on such a device fails. | crates/mu-config/src/names.rs:145 — ".all(\|byte\| matches!(byte, b'a'..=b'z' \| b'0'..=b'9' \| b'-' \| b'_' \| b'.'));" | Make `save_target` (D9) return an error with a notice that names `/layout save` for any name that the layout name check refuses, as it does for a name over 64 characters, and add the unit tests `a_device_with_a_built_in_name` and `a_device_named_dot` to task 5.3. |  |  | wrong-behaviour |
| R-21 | invariants | should-fix | in-change | The spec bounds an id to 32 characters, but the grammar of D4 has no length, and no task tests an id of 32 or 33 characters. | specs/client-layout/spec.md > Requirement: Each block has a stable id — "An id SHALL be 1 to 32 characters from `a` to `z`, `0` to `9` and `-`." | Write the bound of 32 into the grammar of D4, and add the unit tests `an_id_of_32_characters` (good) and `an_id_of_33_characters` (bad, names the line and the id) to task 2.3. |  |  | unmet-requirement |
| R-22 | invariants | should-fix | in-change | A drag or a `<`/`>` key can ask for a percent outside 10% to 90%: in a 300-column split the 10-column floor is 3%, and `>` from 90% gives 95%. D3 clamps only to the smallest pane, so the check refuses the result mid-drag. | design.md > D3. Layer 1: the changes — "Finds the placed block beside the border, and sets its size to a whole percent within the smallest sizes." | State in D3 that `resize_border` and the `<`/`>` keys clamp to 10%..90% as well as to the smallest pane, and add the unit tests `a_border_stops_at_ten_percent` (a split of 300 columns) to task 2.2 and `a_larger_block_stops_at_ninety_percent` to task 6.2. |  |  | wrong-behaviour |
| R-23 | invariants | should-fix | in-change | The percent range 10% to 90% is tested only at 95%; no test passes 9%, 90% or 91% to the reader. | specs/client-layout/spec.md > Scenario: A percent out of range — "holds the lines `editor` and `output above editor, 95%`" | Make `a_percent_out_of_range` of task 2.3 pass 9% and 91% (bad, the error names the word), and add `a_percent_of_90` (good). |  |  | unmet-requirement |
| R-24 | invariants | should-fix | in-change | The block counts are tested only one past each limit (five editors, nine outputs); no test shows that four editors and eight outputs are good. | specs/client-layout/spec.md > Requirement: The blocks of the screen — "A layout SHALL hold `editor` one to four times, `output` at most eight times, and each other block at most once." | Add the unit tests `four_editors` and `eight_outputs` (each a good layout) to task 2.1. |  |  | unmet-requirement |
| R-25 | invariants | should-fix | in-change | The editor `max` from 1 to 100 and a count of cells from 1 have one test, `a_bad_editor_size`; no test passes `max 0`, `max 1`, `max 100`, `max 101` or a count of 0. | specs/client-layout/spec.md > Requirement: The editor grows with its text — "A sentence SHALL set the most rows with `max N` after a comma, from 1 to 100." | Make `a_bad_editor_size` of task 2.3 pass `max 0` and `max 101`, add `an_editor_max_of_1_and_100` (good), and add `a_count_of_zero_cells` (bad). |  |  | unmet-requirement |
| R-26 | spec-lint | should-fix | in-change | A WHEN step gives the input focus to `scratch` with no key or click named, so the step names no concrete input. | specs/client-layout/spec.md > Scenario: Two editors keep two texts — "the user types `one`, gives the input focus to `scratch`, and types `two`" | Name the input in the WHEN step: the key Alt+E or a click in `scratch`, as the scenario 'The key moves the input focus' does. |  |  | false-test |
| R-27 | spec-lint | should-fix | in-change | The width of `notices` in the describe scenario (24) does not agree with the divider in column 56 of the sidebar scenario, which gives 23 columns for the pane beside it; the rectangle rule does not say whether the divider counts. | specs/config-store/spec.md > Scenario: Describe gives each block — "the entry of `notices` has the width 24, and the entry of `tabs` has the width 80 and the height 1" | Work the sidebar columns by the rounding rule of design.md D6 and state in D6 whether a rectangle includes its divider; then correct the width in the describe scenario or the divider column (task 1.2 covers this check). |  |  | false-test |
| R-28 | spec-lint | should-fix | in-change | The THEN step places the notices and the editor by 'the right part' and 'the left part' with no column or row count, so it names no checkable value. | specs/client-layout/spec.md > Scenario: A block beside two rows — "the notices take the right part of rows 2 to 23, and the editor is in the left part of row 23" | State the columns of the notices and of the editor, for example 'the notices take columns 41 to 79 of rows 2 to 23', as the value the rounding rule of D6 gives. |  |  | false-test |
| R-29 | test-coverage | should-fix | in-change | Two scenarios, 'The editor after the notices' and 'The editor before the notices', share one named test, the_order_of_the_lines_decides_the_shape, in task 2.3 (a task about errors); each THEN needs its own assertion. | tasks.md > 2.3 — "`the_order_of_the_lines_decides_the_shape`" | Name two tests, `the_editor_after_the_notices` and `the_editor_before_the_notices`, in layout.rs, each asserting the solved rectangles of its THEN. |  |  | unmet-requirement |
| R-30 | approach | suggestion | in-change | D12 and task 4.1 add `tempfile` to the row of mu-server in workspace_boundary.rs, but the row already lists it. What changes is that mu-server takes tempfile in [dependencies], where today it is a dev-dependency. | design.md > D12. What enforces each rule — "with `toml_edit` and `tempfile` added to the row of `mu-server`" | In D12 and task 4.1, add only toml_edit to the row, and move tempfile of mu-server from [dev-dependencies] to [dependencies]. |  | severity? should-fix 0.74 |  |
| R-31 | approach | suggestion | in-change | D9 says a press that hits no divider goes to the pane under the mouse, as today. At head a press, a drag and a release go to the pane of the world in focus, not to the pane under the mouse. | design.md > D9. Layer 4: commands and the mouse — "A press that hits no divider and no title row goes to the pane under the mouse, as today." | Drop `as today` in D9, and state that the hit test of `solve` picks the output pane for a press, a drag and a release. |  |  |  |
| R-32 | fit | suggestion | in-change | The tick of D7 is a second wait effect with a second one-at-a-time flag, beside RetryConfig and ConfigRetry, which already wait and then give back a message. | design.md > D7. Layer 2: the streams, the tab label and the statusline — "`update` returns `Effect::Tick` of one second. A flag stops a second tick." | Name the parameter: one wait effect that carries the wait and the message to give back, and one Idle/Waiting state for each timer, so the tick and the retry share the run-loop arm. |  |  |  |
| R-33 | invariants | suggestion | in-change | A screen change of step 1 of D3 writes one step back as a line of text, and `/layout edit` reads it again, but the two property tests of D4 work on whole trees and never compare one line with its parsed step. | design.md > D3. Layer 1: the changes — "Change only the line of the block, in its place." | Add a proptest `a_line_reads_back` to task 1.1 or 2.3, after `world_id_round_trip` in crates/mu-core/src/id.rs: for any step (each side, each edge, `with`, several anchors, `as`, each size and `max`), parsing the written line gives the same step. |  |  |  |
| R-34 | invariants | suggestion | in-change | `solve` maps any plan and any terminal size to rectangles, and the spec says the client never stops in a small terminal, but only one narrow size of 40 columns is tested and D12 names no property for the solver. | specs/client-layout/spec.md > Requirement: A small terminal keeps each pane usable — "The client SHALL NOT stop in a terminal with less room." | Add a proptest (the repository uses proptest 1.12, as in the `world_id_round_trip` block of crates/mu-core/src/id.rs) `solve_stays_inside_the_area` to task 3.4: for any valid plan and any area from 0x0 to 300x100, `solve` returns without a panic, each rectangle lies inside the area, and no two rectangles overlap. |  |  |  |
| R-35 | test-coverage | note | in-change | Most tasks say the tests pass but name no command; the command is given once in the task header (config.yaml). Rows are graded covered on that basis. | tasks.md > Tasks: client-layout — "The one-test command is in `openspec/config.yaml`." | No action if config.yaml gives the command; otherwise add it to the header. |  |  |  |

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

## Judge

judge: 60 questions, 0.003 USD, stopped: cap of 60 questions

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A server table | 3.3 | covered |  |
| An unknown field | 3.3 | covered |  |
| Not TOML | 3.3 | covered |  |
| A file that cannot be read | 3.3 | covered |  |
| The three new fields | 3.3 | covered |  |
| A layout table | 3.3 | covered |  |
| An unknown field of tabs | 3.3 | covered |  |
| A world layouts table | 3.2a | covered |  |
| No tabs | 3.8 | covered |  |
| The editor alone | 3.8 | covered |  |
| Nine outputs | 2.1 | covered |  |
| One output shows the notices too | 3.5 | covered |  |
| An output for the notices | 3.5 | covered |  |
| An output with no stream yet | 3.5 | covered |  |
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
| The editor after the notices | 2.3 | partial |  |
| The editor before the notices | 2.3 | partial |  |
| A block beside two rows | 2.3 | covered |  |
| Anchors that are not neighbors | 2.3 | covered |  |
| An id in a later line |  | missing |  |
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
| The default segments | 3.8 | covered |  |
| A narrow terminal | 3.7 | covered |  |
| The typed count | 3.7 | covered |  |
| The state after a disconnect | 3.8 | covered |  |
| The idle time after a send | 3.8 | covered |  |
| The connected time | 3.8 | covered |  |
| No connected time while disconnected | 3.7 | covered |  |
| No worlds | 3.8 | covered |  |
| Under an hour | 3.7 | covered |  |
| Over an hour | 3.7 | covered |  |
| A tick moves the idle time | 3.7 | covered |  |
| No tick without a time segment | 3.7 | covered |  |
| An unknown segment | 3.3 | covered |  |
| A segment twice | 3.3 | covered |  |
| A device sets the left list | 3.3 | covered |  |
| A layout file in use | 3.2 | covered |  |
| A bad layout file at start | 3.2 | covered |  |
| A file with a built-in name | 3.2 | covered |  |
| An unknown field of a layout file | 3.2 | covered |  |
| The device names another layout | 3.2 | covered |  |
| A layout that does not exist | 3.2 | covered |  |
| A change to a layout file changes the revision | 3.2 | covered |  |
| The list with one player layout | 3.9 | covered |  |
| A save writes the file | 4.2 | covered |  |
| A save of a bad layout | 4.2 | covered |  |
| A save with a built-in name | 4.2 | covered |  |
| A save with a bad name | 4.2 | covered |  |
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
| A layout for this world | 5.4 | covered |  |
| Remove the layout of this world | 5.4 | covered |  |
| No world in focus | 5.4 | covered |  |
| An unknown layout for this world | 5.4 | covered |  |
| A use behind an entry | 5.4 | covered |  |
| The next layout of a world entry | 5.4 | covered |  |
| The default layout puts the tabs first | 3.8 | covered |  |

### Not run

Plan phase: no verification runs.

## Verification runs

None.

## Dropped

- spe-5 (spec-lint, confidence 0.6): The GIVEN step of 'Any layout reads back' is 'any valid layout', which names no concrete input; the property test generates it, but the scenario does not say what counts as valid.

## Lens notes

- approach: The wheel and PageUp/PageDown scroll the pane in focus at head (update.rs:408). With several outputs, no decision says which output they scroll. The requirement is in the main terminal-client spec, which the change does not modify.
- approach: D2 says each part of each split comes from one step, but a line with several anchors wraps them in a new part that no step placed. Its size is not stated, though the scenario 'A block beside two rows' works if it shares the rest.
- approach: The requirement of `/pane size` on the first block wants a notice that names the blocks beside it that take a size. D3 and D4 state only that an error names the line and the id.
- approach: D6 says the tests of layers 1 and 2 use preview with insta, but the row of mu-config in workspace_boundary.rs has no insta, and D12 adds only winnow there. Tasks put the snapshot tests in mu-tui, which has insta.
- approach: D9 names `/pane hide` and `/layout use` as two-word commands. The spec also needs `/pane size`, `/layout save`, `/layout list`, `/layout here`, `/layout plan`, `/layout insert`, `/layout delete` and `/layout edit`. command.rs at head splits only the name from one argument.
- approach: The divider columns of the spec scenarios (60, 56, 40, and width 24 for notices) match the D6 rule when a percent is of the whole split width, with the divider taken from the anchor side.
- fit: D5 says layer_rule gets the place clients/layouts, but PLACES in crates/mu-server/src/store.rs is the merge order of the layer directories, and state() parses each file under clients/ as a ClientLayer. Whether a layout file fits that pattern is a question for the approach and invariants lenses.
- fit: The spec has every mu layout command follow the private-socket rule of mu config. That rule lives in the private fn call of crates/mu-cli/src/config_verb.rs:65. Tasks 3.9 and 4.x should share it rather than copy it into a layout verb module.
- fit: D6 says the tests of layers 1 and 2 use preview with insta, but mu-config allows only proptest, serde and toml in workspace_boundary.rs, and task 1.6 adds only winnow. This is for the invariants lens.
- fit: The statusline segment state shows the four names of WorldState::name in crates/mu-core/src/session.rs:56. mu-tui cannot use mu-core, so the client needs its own mapping from the proto enum. I raised no finding because no collapse target has two call sites.
- fit: toml_edit 0.25.19 and winnow 1.0.5 are already in Cargo.lock, so the two crates that the proposal calls new add no new source to the build.
- invariants: I1: A layout holds 1 to 4 editors, at most 8 outputs, and at most one tabs and one statusline \| enforced at the BLOCKS catalog check of build (D1, task 2.1) \| boundary test five_editors and nine_outputs; 4 and 8 not tested (inv-8)
- invariants: I2: A plan has 1 to 32 lines \| enforced at the reader (task 2.3) \| boundary test too_many_lines at 33. Each line places one new block, so a good plan has at most 14 lines (4+8+1+1); the limit of 32 never binds, and scenario 'Too many lines' gives `32` only if the line count check runs before the block counts. No order is stated.
- invariants: I3: A tree has at most 8 levels of splits \| enforced at build \| boundary test a_tree_too_deep, and any_layout_reads_back generates trees up to 8 levels (task 1.1)
- invariants: I4: A percent lies in 10%..90% \| enforced at the reader \| boundary test a_percent_out_of_range at 95% only (inv-7); resize_border and </> do not clamp to it (inv-6)
- invariants: I5: The sizes of one split leave a free part with room \| enforced at none \| boundary test none (inv-1). Probe: probe-inv/d6_split.py models the D6 rule; two 60% parts in 80 columns leave -18 columns, a count of 500 in 24 rows leaves -477.
- invariants: I6: An editor max lies in 1..100 and a count of cells is at least 1 \| enforced at the reader \| boundary test a_bad_editor_size only (inv-9)
- invariants: I7: An id is 1 to 32 characters, starts with a letter, is no grammar word, and is unique \| enforced at the reader for the form and words, at build for uniqueness, at none for the length (inv-5) \| boundary test an_id_twice, an_id_that_is_a_word_of_the_sentences
- invariants: I8: An id never changes under any edit \| enforced at the property tests any_layout_reads_back and a_screen_change_gives_the_wanted_screen \| boundary test edit_a_line_by_its_id
- invariants: I9: Each plan holds an editor, and the input focus is an editor of the plan \| enforced at build (no editor) and apply_config / hide (D7a) \| boundary test no_editor, hide_refuses_the_editor, the_input_focus_is_an_editor_of_the_plan (property)
- invariants: I10: A written plan builds to the same tree with the same ids \| enforced at the writer of D4 \| boundary test any_layout_reads_back (property). D4 writer steps name only side forms and `as`; stacks (`with`) and editor `max` are not named in the steps, so the generator must include them for the property to cover them.
- invariants: I11: A key of [world-layouts] is a world id or a game name \| enforced at a check named as the world id rule, which refuses a game name and is not reachable from mu-config (inv-3) \| boundary test a_key_with_two_slashes
- invariants: I12: A layout name follows the device name rule, is not `.` or `..`, and is not a built-in name \| enforced at the store read and SaveLayout check (D8) \| boundary test a_save_with_a_bad_name, a_save_with_the_name_of_two_dots, a_file_with_a_built_in_name; save_target can produce a refused name (inv-4)
- invariants: I13: A saved name is at most 64 characters \| enforced at save_target \| boundary test a_name_over_64_characters_is_an_error at 71 (inv-11)
- invariants: I14: The server writes only under clients/layouts/ and clients/devices/ \| enforced at the name rule (no `/`) plus `.toml` suffix: `..` + `.toml` is the file `...toml`, not a parent path \| boundary test a_save_with_a_bad_name (`../x`)
- invariants: I15: A layout file write is whole to a reader \| enforced at tempfile rename in the same directory, behind the reload lock \| boundary test none needed for the lock; a temp file left by a crash in clients/layouts/ is kept by DirSnapshot (every non-directory under clients/) and becomes a 'not a layer' notice until removed.
- invariants: I16: A tab label is 1 to 64 characters with balanced braces and known fields \| enforced at the Label check (task 2.5) \| boundary test an_empty_label, a_label_too_long at 65 (inv-10)
- invariants: I17: A statusline list holds known segments, each once \| enforced at the Segment check \| boundary test an_unknown_segment, a_segment_twice
- invariants: I18: A time before the clock shows 0:00; M:SS under 3600 s, H:MM:SS from 3600 s \| enforced at the statusline draw \| boundary test a_time_before_the_clock_is_zero; 3599/3600 not tested (inv-12)
- invariants: I19: A pane of output keeps 2 rows and 10 columns while the screen has room \| enforced at the solver and resize_border \| boundary test a_border_stops_at_the_smallest_pane, a_drag_past_the_smallest_pane, a_percent_in_a_narrow_terminal; title row floor untested (inv-13)
- invariants: I20: A key in arrange mode that is a letter picks a block \| enforced at none for a rebound undo (inv-14)
- invariants: I21: A change to a named layout file changes the client revision \| enforced at resolve (D5) \| boundary test a_change_to_a_layout_file_changes_the_revision, a_change_to_the_layout_file_of_an_entry_changes_the_revision
- invariants: I22: The MODIFIED requirement 'The fields of a client layer' keeps all four base scenarios; no check is weakened.
- invariants: I23: A statusline list in a later layer replaces the whole earlier list and keeps the other list \| enforced at merge_layers (figment merge, which replaces arrays and merges tables by key) \| boundary test a_device_sets_the_left_list
- invariants: I24: At most one clock tick waits at a time, and no tick runs without a connected or idle segment \| enforced at the tick flag of D7, which follows the RetryConfig wait flag of crates/mu-tui/src/model.rs:107 \| boundary test one_tick_waits_at_a_time, no_tick_without_a_time_segment
- invariants: I25: Each editor state draws once per frame \| enforced at the unique ids of a plan (build) and the key (world, editor id) of D7a \| boundary test a_hidden_editor_keeps_its_text, two_editors_keep_two_texts
- invariants: I26: A drag result does not drift across reports \| enforced at resize_border on the plan from the press (D9) \| boundary test a_drag_does_not_drift
- spec-lint: prior.json is an empty list: first round.
- spec-lint: No BREAKING marker in proposal.md or in any of the six delta specs, and the proposal and design state no migration, so breaking-mismatch has no instance.
- spec-lint: Every build target named in tasks.md exists: mise run check and mise run docs:check in mise.toml, pnpm model:check in docs/package.json, and crates/mu-cli/tests/workspace_boundary.rs for the cargo nextest line.
- spec-lint: Every requirement of the six delta specs has at least one scenario, and every scenario has a GIVEN or WHEN line and a THEN line.
- spec-lint: Every requirement heading matched at least two distinctive words in a task line of tasks.md; no untasked-requirement instance.
- spec-lint: The open question in design.md (the 40 by 12 picture) is not an answer that a task assumes; no assumed-answer instance.
- spec-lint: The drag scenario 'A drag to the left' (divider at column 40 gives 50%) agrees with the sidebar divider rule; the arithmetic of the other scenarios is left to task 1.2 as planned.
- spec-lint: Workflow follow-up items are bullets, not checkbox tasks, and were not counted as tracked tasks.
- test-coverage: Duplicate headings: 'An unknown field' and 'No world in focus' each appear in two capabilities; both are graded.
- test-coverage: The change has no BREAKING or REMOVED requirement.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | fixed_after | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D11. Streams, and what #102 adds"],"The model keeps one pane state for each pair of a world and a stream, and one for the stream `notices`."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  | severity? should-fix 0.70 | app-1: No decision says how the first output draws, scrolls and selects over several streams in the order of arrival. D11 keeps one PaneState for each stream, and a PaneState holds its own line store, scroll anchor, selection and drawn rows, so two of them cannot draw as one merged pane.; fit-2: D11 makes the notices a stream with its own pane state, but the model already keeps the notices a second time in Model::notices and replays them into a world pane; the design does not say that the vector and the replay go away. |
| R-2 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D8. Layer 3: the write calls"],"A layout file is written whole to a temporary file in `clients/layouts/`, and then renamed over the target with `tempfile`."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The scenarios of a save into a directory with no `clients` directory need the server to make `clients/layouts/` and `clients/devices/`. The write path of D8 writes a temporary file in `clients/layouts/` and has no step that makes a missing directory. |
| R-3 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu layout prints layouts"],"If the name is of a bad layout file, it SHALL write the notice of the file to stderr and exit with status 1."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | Each scenario of a bad layout file that runs `mu layout show mine` needs the notice of that file on stderr. Task 3.9 builds show over ListLayouts, which the spec limits to good files, and no decision adds a call or a field that returns the notice of a bad layout file. |
| R-4 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D6. Layer 2: how the client draws"],"The solver is a pure function of `mu-config`, `solve(layout, area, editor rows) -> Vec<Placed>`."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | No decision sizes a part that holds several editors side by side. D6 keeps the editor rule of today, which sizes one editor leaf in a split of rows and gives Min(0) in a split of columns, and its solver takes one input of editor rows, while each editor wraps at its own width. |
| R-5 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: A bad tab label makes its layer bad"],"The layer SHALL be bad, and its notice SHALL name the file and `tabs.label`, in each of these cases:"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The requirement of a bad tab label has no decision. D5 adds only `layout` and `world_layouts` to ClientLayer, and the layer check in state() checks only [keys], so nothing in the design makes a bad `tabs.label` make its layer bad on the server. |
| R-6 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-statusline/spec.md","Requirement: A bad statusline makes its layer bad"],"The layer SHALL be bad, and its notice SHALL name the file and the field, in each of these cases:"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The requirements of a bad statusline and of the list merge have no decision. D5 names no [statusline] table of ClientLayer, no field paths `statusline.left` and `statusline.right`, and no check of the segments in state(). |
| R-7 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Requirement: The layout here command sets the layout of one world"],"If an entry is the choice in use, `/layout use` SHALL also show a notice that names `/layout here`."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  | severity? should-fix 0.84 | No decision produces the notice of `/layout use` when a world entry is the choice in use. D9 names the save rule, `layout-here` and `layout-next`, and says nothing of `/layout use` behind an entry. |
| R-8 | 1 | 1 | blocker | in-change | drift | ["artifact",["design.md","D5. Layer 2: layouts on the server"],"A key is a world id or a game name, and `mu-config` checks it with the rule of the world id."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | fit-3: The plan adds a third parser of the world id in mu-config (the key check of D5, the game of layout_for, the game and character of the label and of save_target), beside WorldId in mu-core and the split in mu-tui; the design names no shared one.; app-4: D5 has mu-config check a key of [world-layouts] with the rule of the world id. That rule is pub(crate) in mu-core, and the crate rules give mu-config no member to depend on, so mu-config cannot call it. A second copy can refuse a key for a world that the server accepts.; inv-3: D5 checks a key of `[world-layouts]` with the rule of the world id, but that rule refuses a game name such as `alpha`, and mu-config may depend on no crate of the workspace, so it cannot call it. |
| R-9 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: An id in a later line"],"`output as side right of output` and `tabs above side`"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  | severity? should-fix 0.77 | tes-1: The scenario 'An id in a later line' (tabs above an id defined by an earlier line) has no task and no named test.; spe-2: The THEN step names `notices`, but the layout of the scenario gives that block no line; its only output is `side`, so the step cannot be checked. |
| R-10 | 1 | 1 | should-fix | deviates-design | wrong-behaviour | ["artifact",["design.md","Migration Plan"],"None. A configuration directory with no new field gives the screen of today."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The client writes `layout` and `[world-layouts]` into device layers and adds `clients/layouts/`. A rollback of the binary reads them with deny_unknown_fields and the old layer rule, so the device layer turns bad and drops its echo and keys, and each layout file gives a notice. The plan has no rollback step. |
| R-11 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D11. Streams, and what #102 adds"],"The first `output` merges its streams in the order of arrival, which is the screen of today."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | D11 calls the merge of one global stream `notices` into the first output the screen of today. At head a notice goes into the pane of the world in focus only, so it does not show under another world. A global notices stream shows every notice under every world. |
| R-12 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. Layer 3: the write calls"],"The writes run one at a time behind the lock that the reload uses."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  | severity? blocker 0.93 | D8 runs each write behind the lock that the reload uses and then runs the reload of ReloadConfig. At head Store::reload takes that same tokio Mutex itself, which is not reentrant, so the write path as written waits on itself and the save never returns. |
| R-13 | 1 | 1 | should-fix | in-change | drift | ["artifact",["design.md","D7a. Layer 2: several editors"],"`Model` gets `input: BlockId`, the editor with the input focus."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | D7a gives each editor a block id and an input field of type BlockId, but mu-tui already names editors with EditorId and keeps non-pose editors in Model::editors; the design keeps both names for one editor. |
| R-14 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. Layer 1: the plan is the layout"],"The anchor side never has a size, so each split always has a part that shares the rest."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | D2 says each split keeps a part that shares the rest, but nothing stops the sizes of one split from passing its width: two 60% parts, or a count of 500 rows, leave the anchor side a negative rest. |
| R-15 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: The tab label"],"The table `[tabs]` SHALL hold `label`, a template of 1 to 64 characters."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The tab label limit of 64 characters is tested only at 65; no test shows that 64 characters are good, or that 64 characters of more than one byte each are good. |
| R-16 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Scenario: A name that is too long"],"the device name of the client has 60 characters"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The limit of 64 characters on a saved name is tested only with a name of 71 characters; no test passes a name of exactly 64 (good) or 65 (bad) to `save_target`. |
| R-17 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-statusline/spec.md","Requirement: The format of a time"],"A time SHALL show as `M:SS` under one hour and as `H:MM:SS` from one hour."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The switch from `M:SS` to `H:MM:SS` at one hour is tested at 75 and 3725 seconds only; neither 3599 nor 3600 seconds is tested. |
| R-18 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/layout-editing/spec.md","Requirement: A drag of a title row resizes two panes"],"Neither pane SHALL get smaller than 2 rows."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The floor of 2 rows for a drag of a title row has no test: `a_drag_down` moves 4 rows of a 50% pane and stays inside the range. |
| R-19 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/layout-editing/spec.md","Requirement: Arrange mode moves a block with three keys"],"The keys of the mode that are not letters leave each letter free for a block."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | Arrange mode reads the undo key from the keymap, and `[keys]` may bind a plain lowercase letter, so a player who binds `a` to `undo` loses either the pick of block A or the undo. Nothing keeps the letters free. |
| R-20 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-config/src/names.rs",145] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | `save_target` names a save after the device, but the device name rule accepts `classic`, `focus`, `status`, `sidebar`, `.` and `..`, which the layout name check refuses, so every save on such a device fails. |
| R-21 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: Each block has a stable id"],"An id SHALL be 1 to 32 characters from `a` to `z`, `0` to `9` and `-`."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The spec bounds an id to 32 characters, but the grammar of D4 has no length, and no task tests an id of 32 or 33 characters. |
| R-22 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Layer 1: the changes"],"Finds the placed block beside the border, and sets its size to a whole percent within the smallest sizes."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | A drag or a `<`/`>` key can ask for a percent outside 10% to 90%: in a 300-column split the 10-column floor is 3%, and `>` from 90% gives 95%. D3 clamps only to the smallest pane, so the check refuses the result mid-drag. |
| R-23 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: A percent out of range"],"holds the lines `editor` and `output above editor, 95%`"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The percent range 10% to 90% is tested only at 95%; no test passes 9%, 90% or 91% to the reader. |
| R-24 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: The blocks of the screen"],"A layout SHALL hold `editor` one to four times, `output` at most eight times, and each other block at most once."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The block counts are tested only one past each limit (five editors, nine outputs); no test shows that four editors and eight outputs are good. |
| R-25 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: The editor grows with its text"],"A sentence SHALL set the most rows with `max N` after a comma, from 1 to 100."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The editor `max` from 1 to 100 and a count of cells from 1 have one test, `a_bad_editor_size`; no test passes `max 0`, `max 1`, `max 100`, `max 101` or a count of 0. |
| R-26 | 1 | 1 | should-fix | in-change | false-test | ["artifact",["specs/client-layout/spec.md","Scenario: Two editors keep two texts"],"the user types `one`, gives the input focus to `scratch`, and types `two`"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | A WHEN step gives the input focus to `scratch` with no key or click named, so the step names no concrete input. |
| R-27 | 1 | 1 | should-fix | in-change | false-test | ["artifact",["specs/config-store/spec.md","Scenario: Describe gives each block"],"the entry of `notices` has the width 24, and the entry of `tabs` has the width 80 and the height 1"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The width of `notices` in the describe scenario (24) does not agree with the divider in column 56 of the sidebar scenario, which gives 23 columns for the pane beside it; the rectangle rule does not say whether the divider counts. |
| R-28 | 1 | 1 | should-fix | in-change | false-test | ["artifact",["specs/client-layout/spec.md","Scenario: A block beside two rows"],"the notices take the right part of rows 2 to 23, and the editor is in the left part of row 23"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The THEN step places the notices and the editor by 'the right part' and 'the left part' with no column or row count, so it names no checkable value. |
| R-29 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","2.3"],"`the_order_of_the_lines_decides_the_shape`"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | Two scenarios, 'The editor after the notices' and 'The editor before the notices', share one named test, the_order_of_the_lines_decides_the_shape, in task 2.3 (a task about errors); each THEN needs its own assertion. |
| R-30 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D12. What enforces each rule"],"with `toml_edit` and `tempfile` added to the row of `mu-server`"] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  | severity? should-fix 0.74 | D12 and task 4.1 add `tempfile` to the row of mu-server in workspace_boundary.rs, but the row already lists it. What changes is that mu-server takes tempfile in [dependencies], where today it is a dev-dependency. |
| R-31 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D9. Layer 4: commands and the mouse"],"A press that hits no divider and no title row goes to the pane under the mouse, as today."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | D9 says a press that hits no divider goes to the pane under the mouse, as today. At head a press, a drag and a release go to the pane of the world in focus, not to the pane under the mouse. |
| R-32 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D7. Layer 2: the streams, the tab label and the statusline"],"`update` returns `Effect::Tick` of one second. A flag stops a second tick."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | The tick of D7 is a second wait effect with a second one-at-a-time flag, beside RetryConfig and ConfigRetry, which already wait and then give back a message. |
| R-33 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D3. Layer 1: the changes"],"Change only the line of the block, in its place."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | A screen change of step 1 of D3 writes one step back as a line of text, and `/layout edit` reads it again, but the two property tests of D4 work on whole trees and never compare one line with its parsed step. |
| R-34 | 1 | 1 | suggestion | in-change |  | ["artifact",["specs/client-layout/spec.md","Requirement: A small terminal keeps each pane usable"],"The client SHALL NOT stop in a terminal with less room."] | fixed | 1e3c4218c1753172d4fc238c72f917be061d1bb5 | 1 |  |  | `solve` maps any plan and any terminal size to rectangles, and the spec says the client never stops in a small terminal, but only one narrow size of 40 columns is tested and D12 names no property for the solver. |
| R-35 | 1 | 1 | note | in-change |  | ["artifact",["tasks.md","Tasks: client-layout"],"The one-test command is in `openspec/config.yaml`."] |  |  |  |  |  | Most tasks say the tests pass but name no command; the command is given once in the task header (config.yaml). Rows are graded covered on that basis. |
