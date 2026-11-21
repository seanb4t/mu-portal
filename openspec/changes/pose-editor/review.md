---
verdict: approve-with-notes
phase: plan
reviewed_at: 2026-11-21T15:03:59+00:00
head: 533e1bb37337a23ee482e7bf7ed03099b4449945
round: 4
next_id: 63
lenses:
  spec-lint: ok
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
  verify: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-57 | test-coverage | should-fix | in-change | delta scenario 'Ctrl+U at the start of a line' has no matrix row | review > absent-row — "Ctrl+U at the start of a line" | add the row with its covering task or test |  |  | unmet-requirement |
| R-58 | test-coverage | should-fix | in-change | delta scenario 'Each line is a command' has no matrix row | review > absent-row — "Each line is a command" | add the row with its covering task or test |  |  | unmet-requirement |
| R-59 | test-coverage | should-fix | in-change | delta scenario 'A line that the server refuses' has no matrix row | review > absent-row — "A line that the server refuses" | add the row with its covering task or test |  |  | unmet-requirement |
| R-60 | test-coverage | should-fix | in-change | delta scenario 'The lines become one line' has no matrix row | review > absent-row — "The lines become one line" | add the row with its covering task or test |  |  | unmet-requirement |
| R-61 | test-coverage | should-fix | in-change | delta scenario 'A world with no line break' has no matrix row | review > absent-row — "A world with no line break" | add the row with its covering task or test |  |  | unmet-requirement |
| R-62 | test-coverage | should-fix | in-change | delta scenario 'The line break of a world' has no matrix row | review > absent-row — "The line break of a world" | add the row with its covering task or test |  |  | unmet-requirement |
| R-56 | verify | should-fix | in-change | D13 keeps only the lines in the queue and names no world for the later lines. The send path today takes the world from the focus at the time of the send, and Reply::Sent carries no world. So a Ctrl+N during a send of each line can send the rest of a script to another character. | design.md > D13. A send of each line — "The model keeps the lines in a queue. `update` returns `Effect::Send` for the first line. On the reply of a line, `updat" | In D13, keep the world of the Alt+Enter with the queue and send each later line to that world, whatever the focus. State it in the requirement, and add a test that moves the focus between two lines. |  |  | wrong-behaviour |

## Repeats

| id | lens | severity | claim | evidence |
|---|---|---|---|---|
| R-43 | test-coverage | should-fix | delta scenario 'A terminal of 17 rows' has no matrix row | review > absent-row — "A terminal of 17 rows" |
| R-44 | test-coverage | should-fix | delta scenario 'A terminal of 3 rows' has no matrix row | review > absent-row — "A terminal of 3 rows" |
| R-45 | test-coverage | should-fix | delta scenario 'A terminal of 1 row' has no matrix row | review > absent-row — "A terminal of 1 row" |
| R-46 | test-coverage | should-fix | delta scenario 'Up into an emoji with joiners' has no matrix row | review > absent-row — "Up into an emoji with joiners" |
| R-50 | test-coverage | should-fix | delta scenario 'Ctrl+U deletes to the start of the line' has no matrix row | review > absent-row — "Ctrl+U deletes to the start of the line" |
| R-51 | test-coverage | should-fix | delta scenario 'Undo and redo' has no matrix row | review > absent-row — "Undo and redo" |
| R-52 | test-coverage | should-fix | delta scenario 'Ctrl+R changes nothing' has no matrix row | review > absent-row — "Ctrl+R changes nothing" |
| R-47 | test-coverage | should-fix | delta scenario 'A paste with a selection' has no matrix row | review > absent-row — "A paste with a selection" |
| R-48 | test-coverage | should-fix | delta scenario 'The cursor after a full row' has no matrix row | review > absent-row — "The cursor after a full row" |
| R-53 | test-coverage | should-fix | delta scenario 'Ctrl+M sends' has no matrix row | review > absent-row — "Ctrl+M sends" |
| R-54 | test-coverage | should-fix | delta scenario 'A built-in type that extends another' has no matrix row | review > absent-row — "A built-in type that extends another" |
| R-55 | test-coverage | should-fix | delta scenario 'A built-in type extends only a built-in type' has no matrix row | review > absent-row — "A built-in type extends only a built-in type" |

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

## Judge

judge: 2 questions, 0.000 USD

## Coverage matrix

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

## Verification runs

None.

## Dropped

None.

## Lens notes

- verify: R-49: ratatui-textarea 0.9.4 delete_line_by_head calls delete_piece(0, cursor col), which returns false at column 0, and then delete_newline. So the guard of D5 at column 0 is needed, and the scenario 'Ctrl+U at the start of a line' with its test in task 5.2 checks it.
- verify: Home reaches TextArea::input as Key::Home (crates/mu-tui/src/update.rs:452) and has no keymap entry, so the Home step of the scenario 'Ctrl+U at the start of a line' moves the editor cursor.
- verify: Reply::Sent carries no world or text, so the reply of a plain Enter during a send of each line also advances the queue. D13 accepts that the place of that line is not fixed. It can also put two lines in flight at once, against 'The client SHALL send a line only after the server answered the line before it'.
- verify: review.md lines of diff-hunks.txt and of the fix_hunks of R-49 were not read, as the agent rules forbid it.

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
| R-56 | 4 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D13. A send of each line"],"The model keeps the lines in a queue. `update` returns `Effect::Send` for the first line. On the reply of a line, `update` returns the effect for the next one."] |  |  |  |  | D13 keeps only the lines in the queue and names no world for the later lines. The send path today takes the world from the focus at the time of the send, and Reply::Sent carries no world. So a Ctrl+N during a send of each line can send the rest of a script to another character. |
| R-57 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Ctrl+U at the start of a line"] |  |  |  |  | delta scenario 'Ctrl+U at the start of a line' has no matrix row |
| R-58 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Each line is a command"] |  |  |  |  | delta scenario 'Each line is a command' has no matrix row |
| R-59 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A line that the server refuses"] |  |  |  |  | delta scenario 'A line that the server refuses' has no matrix row |
| R-60 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The lines become one line"] |  |  |  |  | delta scenario 'The lines become one line' has no matrix row |
| R-61 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A world with no line break"] |  |  |  |  | delta scenario 'A world with no line break' has no matrix row |
| R-62 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The line break of a world"] |  |  |  |  | delta scenario 'The line break of a world' has no matrix row |
