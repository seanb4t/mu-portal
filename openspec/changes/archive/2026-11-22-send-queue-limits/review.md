---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-22T19:21:20+00:00
head: 678537b33820ade6045b1478b9f5970d6df6e650
round: 5
next_id: 38
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
| R-36 | test-coverage | should-fix | in-change | delta scenario 'A list that failed' has no matrix row | review > absent-row — "A list that failed" | add the row with its covering task or test |  |  | unmet-requirement |
| R-37 | test-coverage | should-fix | in-change | delta scenario 'A list with a new world' has no matrix row | review > absent-row — "A list with a new world" | add the row with its covering task or test |  |  | unmet-requirement |

## Repeats

| id | lens | severity | claim | evidence |
|---|---|---|---|---|
| R-34 | test-coverage | should-fix | covered row 'A reply near the limit' has no verification run | review > not-run — "A reply near the limit" |
| R-35 | test-coverage | should-fix | delta scenario 'The server resets while a list is with it' has no matrix row | review > absent-row — "The server resets while a list is with it" |

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-12 | should-fix | accepted | The probe cannot read the rest of a cut end mark before Termina starts; the proposal already says the rest of an open paste arrives as keys. Owner accepted it and tracks it. | https://github.com/seanb4t/mu-portal/issues/158 |
| R-17 | suggestion | deferred | Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. | https://github.com/seanb4t/mu-portal/issues/157 |
| R-19 | suggestion | deferred | Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. | https://github.com/seanb4t/mu-portal/issues/157 |
| R-20 | suggestion | deferred | Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. | https://github.com/seanb4t/mu-portal/issues/157 |
| R-21 | suggestion | deferred | Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. | https://github.com/seanb4t/mu-portal/issues/157 |
| R-22 | should-fix | accepted | Only a full round writes matrix rows; each scenario has a named passing test, and the full first round of the implementation review writes the rows. Owner accepted. |  |
| R-23 | should-fix | accepted | Only a full round writes matrix rows; each scenario has a named passing test, and the full first round of the implementation review writes the rows. Owner accepted. |  |
| R-24 | should-fix | accepted | Only a full round writes matrix rows; each scenario has a named passing test, and the full first round of the implementation review writes the rows. Owner accepted. |  |
| R-25 | should-fix | accepted | Only a full round writes matrix rows; each scenario has a named passing test, and the full first round of the implementation review writes the rows. Owner accepted. |  |
| R-33 | suggestion | deferred | A broader property test of an open paste; the cut cases have example tests. Owner chose to defer it. | https://github.com/seanb4t/mu-portal/issues/159 |
| R-35 | should-fix | accepted | Only a full round writes matrix rows; reset_during_a_list_asks_again_after_its_reply covers the scenario, passes, and tasks.md names it. Owner accepted. |  |
| R-36 | should-fix | accepted | Only a full round writes matrix rows; failed_list_names_its_reason_at_alt_r and list_adds_a_world_that_the_client_did_not_hold cover the scenarios and pass, and tasks.md names them. Owner accepted. |  |
| R-37 | should-fix | accepted | Only a full round writes matrix rows; failed_list_names_its_reason_at_alt_r and list_adds_a_world_that_the_client_did_not_hold cover the scenarios and pass, and tasks.md names them. Owner accepted. |  |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 09078fd397442e8a076d8489ccd35a3c7ed9a8cb | 3f67ca37e2f3f07842d8a278c4aa11f50c66efa10edbe742eac74b831b07f731 | changes-requested | 21 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 2e9ec7680a63ad888079b2c3907d15208b524ec2 | 3daad9b6805f725d68f5061772d2543502b26610e665da4e35d413fe0b96b258 | approve-with-notes | 4 | 0 | 3 | 14 | 0 |  |
| 3 | implementation | full | dd6f32ab242c347831b7aed1a5c28c00c5801157 | 3af5daad20d2335fe8ef8a2fd1701c9039a81a9106287e1801c41c592fb33987 | approve-with-notes | 9 | 0 | 0 | 0 | 0 |  |
| 4 | implementation | follow-up | b41497026abb8489e48b028165124f94180b278e | 4c529945f044515f38b6012d5a7351dccd729b2bfa46a65426d5018ad8b1175c | approve-with-notes | 1 | 1 | 0 | 9 | 0 |  |
| 5 | implementation | follow-up | 678537b33820ade6045b1478b9f5970d6df6e650 | 7f895883abfb10d5676a898d485d18cfa8a19acf526b4b01c27f2eb6059eb61a | approve-with-notes | 2 | 2 | 0 | 0 | 0 |  |

## Judge

judge: 0 questions, 0.000 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| Quit | crates/mu-cli/tests/client.rs: ctrl_c_quits_client_only | covered | pass |
| Quit while lines wait | crates/mu-tui/tests/app.rs: quit_with_waiting_sends_asks_again | covered | pass |
| Quit while only the line with the server waits | crates/mu-tui/tests/app.rs: quit_with_only_the_line_with_the_server_asks_again | covered | pass |
| Another key between the two | crates/mu-tui/tests/app.rs: another_key_cancels_the_second_quit | covered | pass |
| A line that the server refuses | crates/mu-tui/tests/app.rs: refused_line_stops_the_rest | covered | pass |
| The last line is refused | crates/mu-tui/tests/app.rs: refused_last_line_goes_before_the_typed_text | covered | pass |
| A refused send of Enter with a send after it | crates/mu-tui/tests/app.rs: refused_send_stops_the_sends_after_it | covered | pass |
| The stream ends while lines wait | crates/mu-tui/tests/app.rs: lost_server_puts_the_waiting_lines_back | covered | pass |
| Lines of two worlds wait | crates/mu-tui/tests/app.rs: lost_server_puts_back_the_lines_in_the_order_of_the_worlds | covered | pass |
| The server resets while lines wait | crates/mu-tui/tests/app.rs: reset_puts_the_waiting_lines_back | covered | pass |
| A refusal after the stream ended | crates/mu-tui/tests/app.rs: refusal_after_a_loss_puts_back_only_its_line | covered | pass |
| The lines become one line | crates/mu-tui/tests/app.rs: lines_become_one_line | covered | pass |
| A world with no line break | crates/mu-tui/tests/app.rs: join_with_no_line_break | covered | pass |
| A line break that the editor cannot hold | crates/mu-tui/tests/app.rs: line_break_that_the_editor_cannot_hold_is_no_line_break | covered | pass |
| A world that the first list did not hold | crates/mu-tui/tests/app.rs: list_takes_the_line_break_of_a_new_world | covered | pass |
| No world in focus | crates/mu-tui/tests/app.rs: alt_r_with_no_world_names_the_line_break | covered | pass |
| A line break that the client does not know | crates/mu-tui/tests/app.rs: unknown_line_break_asks_the_server | covered | pass |
| A terminal that adds code point widths | crates/mu-cli/tests/client.rs: probe_adds_code_point_widths | covered | pass |
| A terminal that does not answer | crates/mu-cli/tests/client.rs: probe_with_no_answer | covered | pass |
| A late answer | crates/mu-cli/tests/client.rs: late_answer | covered | pass |
| A paste during the queries | crates/mu-tui/tests/app.rs: paste_during_the_probe_fills_the_editor | covered | pass |
| A paste split across reads | crates/mu-tui/tests/probe.rs: paste_split_anywhere_gives_its_text | covered | pass |
| A server that does not answer | crates/mu-tui/tests/send.rs: send_with_no_answer_is_refused | covered | pass |
| A reply near the limit | crates/mu-tui/tests/send.rs: answer_just_inside_the_limit_is_sent, answer_just_past_the_limit_is_refused | covered | not-run |
| Two lines wait | crates/mu-tui/tests/app.rs: top_row_counts_the_sends_that_wait | covered | pass |

### Not run

- A reply near the limit: no verification run names this row

## Verification runs

- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=ctrl_c_quits_client_only)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=quit_with_waiting_sends_asks_again)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=quit_with_only_the_line_with_the_server_asks_again)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=another_key_cancels_the_second_quit)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=refused_line_stops_the_rest)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=refused_last_line_goes_before_the_typed_text)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=refused_send_stops_the_sends_after_it)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=lost_server_puts_the_waiting_lines_back)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=lost_server_puts_back_the_lines_in_the_order_of_the_worlds)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=reset_puts_the_waiting_lines_back)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=refusal_after_a_loss_puts_back_only_its_line)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=lines_become_one_line)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=join_with_no_line_break)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=line_break_that_the_editor_cannot_hold_is_no_line_break)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=list_takes_the_line_break_of_a_new_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=alt_r_with_no_world_names_the_line_break)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=unknown_line_break_asks_the_server)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=probe_adds_code_point_widths)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=probe_with_no_answer)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=late_answer)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=paste_during_the_probe_fills_the_editor)'`
- exit 0: `cargo nextest run -p mu-tui --test probe --locked -E 'test(=paste_split_anywhere_gives_its_text)'`
- exit 0: `cargo nextest run -p mu-tui --test send --locked -E 'test(=send_with_no_answer_is_refused)'`
- exit 0: `cargo nextest run -p mu-tui --test send --locked -E 'test(=answer_just_inside_the_limit_is_sent)'`
- exit 0: `cargo nextest run -p mu-tui --test send --locked -E 'test(=answer_just_past_the_limit_is_refused)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=top_row_counts_the_sends_that_wait)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=reset_during_a_list_asks_again_after_its_reply)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=failed_list_names_its_reason_at_alt_r)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=list_adds_a_world_that_the_client_did_not_hold)'`

## Dropped

None.

## Lens notes

- verify: prior.json has no row with the disposition fixed, so fixes is empty.
- verify: Every code and test range of diff-hunks.txt was read at head: model.rs 69-74 and 131, probe.rs 119-162 and 555-596, run.rs 122-148, update.rs 160-167, 472-489 and 651-675, tests/app.rs 939-947 and 1271-1436. No regression found in them.
- verify: The ranges of review.md in diff-hunks.txt were not read, because the agent rules forbid a read of review.md.
- verify: join_lines (update.rs 481-489): with a failed list on record and a later list still with the server, the Alt+R hint says 'so it asks again' while list_worlds returns None. A list is with the server then, so the user still gets the line break; the wording is only loose.
- verify: probe.rs 151-152: the doc comment of frame wraps unevenly ('...an open bracketed' then 'paste. If an open CSI or'). Cosmetic only.
- verify: return_waiting (update.rs 659) calls queue.drain(1..), which panics on an empty queue. No path at head leaves an empty queue in Model::sends: enqueue always adds at least one text, and next_send and on_reply remove an emptied entry.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: Alt+R joins the lines in place"],"If that world has no `line_break`, or no world is in focus, the client SHALL NOT change the editor."] | resolved | resolved in round 2 from fixed e00b743f4068bcb6332142677496eeff92ead7be |  |  | app-1: The modified Alt+R requirement still says that with no world in focus the client shows a `-- ` line that contains `line break`. D5 splits Alt+R into 'not known yet', 'no line break' and 'join', and names no outcome for no world in focus. The join_lines at head returns early through `?` before any hint, so Alt+R with no world in focus shows no line. The join_lines of main gave the 'sets no line break' hint in that case, so the change drops the behaviour.; inv-2: join_lines now returns early with no notice when no world is in focus. The old guard showed the line break hint there, and the requirement still demands it. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Scenario: A line break that the editor cannot hold"],"gives `alpha/aria` an empty `line_break`, or one with ESC or LF"] | resolved | resolved in round 2 from fixed c47deba0add0c4c3ddaf5726cc3852ddeec4d192 |  |  | Scenario 'A line break that the editor cannot hold' has no task and no named test. |
| R-3 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["tasks.md","1. A send with no reply, and the texts that wait (#147)"],"`failed_send_is_refused_with_its_reason` and `answered_send_is_sent` in `crates/mu-tui/tests/send.rs`"] | resolved | resolved in round 2 from fixed c47deba0add0c4c3ddaf5726cc3852ddeec4d192 |  | severity? should-fix 0.72 | Scenarios 'A line that the server refuses', 'The last line is refused' and 'A refused send of Enter with a send after it' name no task or test. |
| R-4 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["tasks.md","3. A paste during the probe (#149)"],"Add `Skip::Paste`, `PASTE_START` and `PASTE_END`, and frame a paste as one event (D6)."] | resolved | resolved in round 2 from fixed c47deba0add0c4c3ddaf5726cc3852ddeec4d192 |  | severity? should-fix 0.72 | Scenarios 'The lines become one line', 'A terminal that adds code point widths', 'A terminal that does not answer' and 'A late answer' have no task. |
| R-5 | 1 | 2 | should-fix | deviates-design | wrong-behaviour | ["artifact",["design.md","D1. The limit of a send is in the event loop, and it gives a `Reply::Refused`"],"Alternative: a timer message in the model. It needs a clock in `update`, or a second message for each send."] | resolved | resolved in round 2 from fixed c47deba0add0c4c3ddaf5726cc3852ddeec4d192 |  |  | D1 hand-wraps one call in tokio::time::timeout, where connectrpc gives each call a timeout. The new ListWorlds call gets no bound, so a list with no reply leaves Listing::Asked for the session. |
| R-6 | 1 | 2 | should-fix | deviates-design | wrong-behaviour | ["path","crates/mu-tui/src/run.rs",134] | resolved | resolved in round 2 from fixed c47deba0add0c4c3ddaf5726cc3852ddeec4d192 |  | scope? in-change 0.64 | SEND_LIMIT equals the server's own WRITE_TIMEOUT, which D1 does not name. A stalled game write gets the server's sure refusal at the same moment the client gives up with the vague reason. |
| R-7 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/update.rs",636] | resolved | resolved in round 2 from fixed e00b743f4068bcb6332142677496eeff92ead7be |  |  | return_waiting reports a one-time event with Model::notice, which every later Reset shows again. The existing helper for a one-time line is Model::hint, which the Quit notice of this change uses. |
| R-8 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: Alt+R joins the lines in place"],"The client SHALL also ask for that list after the first event of a world that it did not hold, and after the server resets."] | resolved | resolved in round 2 from fixed e00b743f4068bcb6332142677496eeff92ead7be |  |  | One list at a time holds only while each list call ends. ListWorlds has no time limit, so a list with no reply leaves Listing::Asked for good, and no reset, new world or Alt+R asks again. |
| R-9 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["specs/terminal-client/spec.md","Requirement: A send with no reply is refused"],"If the server gives no reply to a text within 10 seconds, the client SHALL count the text as refused."] | resolved | resolved in round 2 from fixed e00b743f4068bcb6332142677496eeff92ead7be |  |  | No test passes a reply on either side of SEND_LIMIT. The tests use a call that never ends and a call that is ready at once, so a limit that fires early still passes. |
| R-10 | 1 | 2 | should-fix | in-change | false-test | ["path","crates/mu-tui/src/update.rs",350] | resolved | resolved in round 2 from fixed e00b743f4068bcb6332142677496eeff92ead7be |  |  | The Quit guard is waiting == 0, and the tests pass 0 and 3. No test passes 1, the line with the server alone, so a guard that counts as the top row does still passes. |
| R-11 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["specs/terminal-client/spec.md","Requirement: A refused text stops the sends after it"],"the client SHALL put each text that waits behind the text with the server back in the editor, in the order of the worlds"] | resolved | resolved in round 2 from fixed e00b743f4068bcb6332142677496eeff92ead7be |  |  | return_waiting promises the order of the worlds, but each loss and reset test holds one world. An order by the HashMap of sends passes too. |
| R-12 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. The probe framer holds a whole paste"],"A paste still open after that becomes one paste of its first part."] | accepted | ticket: https://github.com/seanb4t/mu-portal/issues/158; reason: The probe cannot read the rest of a cut end mark before Termina starts; the proposal already says the rest of an open paste arrives as keys. Owner accepted it and tracks it. |  |  | At the end of the extra wait, an open paste is given as pending whole. If a read stopped inside the end mark, ESC and [20 go into the paste and 1~ arrives as keys. |
| R-13 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: The client measures character widths at startup"],"Before its first frame, the client SHALL ask the terminal for the state of mode 2027."] | resolved | resolved in round 2 from fixed c47deba0add0c4c3ddaf5726cc3852ddeec4d192 |  |  | No task line covers the requirement 'The client measures character widths at startup' (mode 2027 query, cursor-position queries, device attributes end, the late answer). |
| R-14 | 1 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Scenario: A terminal that does not answer"],"AND the client uses the `unicode-width` value of each grapheme"] | resolved | resolved in round 4 from fixed 9861b0d778c541f75189d0604b919e1312e7e045 |  |  | The THEN 'the client uses the unicode-width value of each grapheme' names no observable: no cell column, no rendered text, no returned value. |
| R-15 | 1 | 4 | should-fix | in-change | false-test | ["artifact",["tasks.md","1. A send with no reply, and the texts that wait (#147)"],"Make sure that they pass."] | resolved | resolved in round 4 from fixed 9861b0d778c541f75189d0604b919e1312e7e045 |  |  | Tasks 1.3, 1.4, 1.5, 2.1, 3.2 and 3.3 name tests but only say 'Make sure that they pass', with no command. |
| R-16 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D2. A loss or a reset returns the texts behind the one with the server"],"Alternative: clear each whole queue. Then a late `Refused` puts the first text back after the others, out of order."] | resolved | resolved in round 2 from fixed c47deba0add0c4c3ddaf5726cc3852ddeec4d192 |  | severity? should-fix 0.61 | D2 rejects clearing each whole queue because a late Refused would put the first text back after the others. put_back puts the returned lines before the text of the editor, and a refusal with no queue returns only its own text. So under a cleared queue a late Refused of `one` lands before `two` and `three`, in order. The real cost of clearing, a new send that goes while the old call is still open, against the Goal of one text per world with the server, is not stated. |
| R-17 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D4. The count in the top row"],"The statusline of client-shell (#101) can take the count later."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/157; reason: Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. |  | severity? should-fix 0.80 | D4 says the #101 statusline takes the count later, but the count lives inline in view::top_row. #101 must move it or copy the 'first one is with the server' rule, which return_waiting already repeats. |
| R-18 | 1 | 2 | suggestion | in-change |  | ["path","crates/mu-tui/src/run.rs",105] | resolved | resolved in round 2 from fixed e00b743f4068bcb6332142677496eeff92ead7be |  |  | The new Effect::ListWorlds arm of run::event_loop repeats the spawn-and-reply block of Effect::Send. Only the call and the Msg variant differ. |
| R-19 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D6. The probe framer holds a whole paste"],"When the framer finds `ESC [200~`, it sets `Skip::Paste`, and each later read goes into `pending` with no limit."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/157; reason: Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. |  |  | D6 writes its own bracketed-paste framing across reads. termina's Parser, which the framer already uses, holds an open paste across parse calls and gives one Event::Paste. D6 does not name it. |
| R-20 | 1 | 1 | suggestion | in-change |  | ["path","crates/mu-tui/src/probe.rs",122] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/157; reason: Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. |  | severity? should-fix 0.71 | Skip::Paste joins an enum of bytes to drop, but it holds bytes. skipped must make it a special case, and read_answers builds a paste event a second time. |
| R-21 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","Goals / Non-Goals"],"One text per world is with the server at a time, as the requirement \\"The sends to a world go in order\\" says."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/157; reason: Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. |  |  | The send queue has an invariant over any sequence of sends, replies, losses and resets: each text is once in the queue, the editor or the server, and one Send per world is open. |
| R-22 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Quit while only the line with the server waits"] | accepted | ticket: none; reason: Only a full round writes matrix rows; each scenario has a named passing test, and the full first round of the implementation review writes the rows. Owner accepted. |  |  | delta scenario 'Quit while only the line with the server waits' has no matrix row |
| R-23 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Lines of two worlds wait"] | accepted | ticket: none; reason: Only a full round writes matrix rows; each scenario has a named passing test, and the full first round of the implementation review writes the rows. Owner accepted. |  |  | delta scenario 'Lines of two worlds wait' has no matrix row |
| R-24 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"No world in focus"] | accepted | ticket: none; reason: Only a full round writes matrix rows; each scenario has a named passing test, and the full first round of the implementation review writes the rows. Owner accepted. |  |  | delta scenario 'No world in focus' has no matrix row |
| R-25 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A reply near the limit"] | accepted | ticket: none; reason: Only a full round writes matrix rows; each scenario has a named passing test, and the full first round of the implementation review writes the rows. Owner accepted. |  |  | delta scenario 'A reply near the limit' has no matrix row |
| R-26 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: Alt+R joins the lines in place"],"A list with no reply within 30 seconds SHALL count as failed."] | resolved | resolved in round 4 from fixed 7ef2ca230c93227b0461dacd4b5c44765d031260 |  | severity? blocker 0.73 | app-1: The requirement and the proposal give every list of the worlds a 30 s limit, but D1 bounds only the list of Effect::ListWorlds. The first list at startup in run::run still calls client.list_worlds() with no limit. A server that never answers it keeps the client from its first frame, and that list never counts as failed.; inv-1: The spec says a list with no reply within 30 seconds counts as failed, but the first list at startup is awaited with no limit. A server that accepts the connection and never answers ListWorlds keeps the client before its first frame for good. Only the lists of Effect::ListWorlds go through within. |
| R-27 | 3 | 4 | should-fix | in-change | drift | ["path","crates/mu-tui/src/run.rs",29] | resolved | resolved in round 4 from fixed 7ef2ca230c93227b0461dacd4b5c44765d031260 |  |  | mu-tui now has two paths for ListWorlds. The new Effect::ListWorlds goes through run::within with CALL_LIMIT, but the startup list in run::run calls client.list_worlds() with no limit and another error type. The two paths already differ. |
| R-28 | 3 | 4 | should-fix | in-change | drift | ["path","crates/mu-tui/tests/app.rs",1264] | resolved | resolved in round 4 from fixed 7ef2ca230c93227b0461dacd4b5c44765d031260 |  | severity? suggestion 0.67 | The change adds an alt_r() helper to app.rs but leaves four inline copies of the same Alt+R key in the Alt+R tests beside it, two of them as a local named alt_r. Two mechanisms build one key. |
| R-29 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["path","docs/src/content/docs/guides/first-run.md",61] | resolved | resolved in round 4 from fixed bda7c0d9dabb58add3edb90e16790bc765d838c7 |  |  | The key table of the first-run guide restates the Ctrl+C rule as a plain quit. The change makes the first Ctrl+C with waiting lines show a notice and not quit, and the guide is not in the diff, so a user who follows the guide gets a client that does not quit. |
| R-30 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: Alt+R joins the lines in place"],"The client SHALL also ask for that list after the first event of a world that it did not hold, and after the server resets."] | resolved | resolved in round 4 from fixed 7ef2ca230c93227b0461dacd4b5c44765d031260 |  | repeat? R-8 0.76 | The spec says the client asks for the list after the server resets. If a list is Asked when the Reset comes, list_worlds returns None, and no list goes after the reset. That list was asked of the old server, so a failed or old answer leaves the line breaks of before the reset, and nothing asks again until a new world or Alt+R on an unlisted world. |
| R-31 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. The probe framer holds a whole paste"],"A cut end mark or a cut UTF-8 character at its end is not part of that text."] | resolved | resolved in round 4 from fixed 7ef2ca230c93227b0461dacd4b5c44765d031260 |  |  | D6 says a cut UTF-8 character at the end of a paste that is still open is not part of its text, enforced by held_for_paste through unfinished_tail. No test ends an open paste inside a multi-byte character. cut_end_mark_is_not_text_of_the_paste only cuts the end mark, so a regression gives a U+FFFD in the editor and passes. |
| R-32 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","Context"],"A send to a world that connects waits behind the connect."] | resolved | resolved in round 4 from fixed bda7c0d9dabb58add3edb90e16790bc765d838c7 |  | severity? should-fix 0.82 | design.md says that a send to a world that connects waits behind the connect, up to 20 s on the server. At head the session refuses a Send at once unless the connection is Up and Live, so a send never waits behind a connect. The 30 s limit still holds over the server's bounds, but its stated reason is wrong. |
| R-33 | 3 | 3 | suggestion | in-change |  | ["path","crates/mu-tui/tests/probe.rs",459] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/159; reason: A broader property test of an open paste; the cut cases have example tests. Owner chose to defer it. |  |  | The text of a paste still open at the end of the probe is, for any cut point, a prefix of the pasted text with no byte of the end mark and no replacement character. The property test covers only a closed paste. |
| R-34 | 3 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"A reply near the limit"] | refuted | Both covering tests of the row ran and passed: verification-runs.txt has exit 0 for answer_just_inside_the_limit_is_sent and answer_just_past_the_limit_is_refused, on two lines because the row names two tests. |  |  | covered row 'A reply near the limit' has no verification run |
| R-35 | 4 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The server resets while a list is with it"] | accepted | ticket: none; reason: Only a full round writes matrix rows; reset_during_a_list_asks_again_after_its_reply covers the scenario, passes, and tasks.md names it. Owner accepted. |  |  | delta scenario 'The server resets while a list is with it' has no matrix row |
| R-36 | 5 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A list that failed"] | accepted | ticket: none; reason: Only a full round writes matrix rows; failed_list_names_its_reason_at_alt_r and list_adds_a_world_that_the_client_did_not_hold cover the scenarios and pass, and tasks.md names them. Owner accepted. |  |  | delta scenario 'A list that failed' has no matrix row |
| R-37 | 5 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A list with a new world"] | accepted | ticket: none; reason: Only a full round writes matrix rows; failed_list_names_its_reason_at_alt_r and list_adds_a_world_that_the_client_did_not_hold cover the scenarios and pass, and tasks.md names them. Owner accepted. |  |  | delta scenario 'A list with a new world' has no matrix row |

## Reviewed files

| path | content |
|---|---|
| crates/mu-tui/Cargo.toml | b89f2c80623814f900413f4e9e6e2c23752aeffc |
| crates/mu-tui/src/lib.rs | 17a66cb024d4d73909d76a8deaebb3d451ea9ede |
| crates/mu-tui/src/model.rs | 4089d239901159c2e9f9f3076b0f4961bc207ac9 |
| crates/mu-tui/src/probe.rs | 3664685f03a1b40c17555215f1a2075be7d9d1b1 |
| crates/mu-tui/src/run.rs | 9c60d87dae4ec9dd551d4bf57a5f6ea66904c172 |
| crates/mu-tui/src/update.rs | 891d501c8ff67b2a2e46a5e8085f8d22ffb1c5a7 |
| crates/mu-tui/src/view.rs | 415d0a746078255bc29ef49207aabe7ecb717650 |
| crates/mu-tui/tests/app.proptest-regressions | 81f7accb35b1da9e96d79f56e22f71eed5a9b06d |
| crates/mu-tui/tests/app.rs | 3e6f6177d77103658167a9bb7945c2d46898724b |
| crates/mu-tui/tests/probe.rs | 6144897a220543586f8b362da5dc52a6126b7f50 |
| crates/mu-tui/tests/send.rs | 2d320a2bd62ed7afab07f0831a651e9f3b55074a |
| docs/src/content/docs/guides/first-run.md | 45c3bd9abe50c2a84915b867e636cc11d741248e |
| openspec/changes/send-queue-limits/.openspec.yaml | b6809d6be7a6faad01a1bd5da9bea068ac829d8f |
| openspec/changes/send-queue-limits/design.md | 12fc18c6277a80c1a16e45a04d916dd2c6d0130f |
| openspec/changes/send-queue-limits/proposal.md | ac8d0c8aaba9ec56752d16a9019144271e110386 |
| openspec/changes/send-queue-limits/specs/terminal-client/spec.md | c88ca257a92c082ac965c031d6addff8948cded7 |
| openspec/changes/send-queue-limits/tasks.md | sha256:6d9ac976945f85109865953aaec9892d7d9e56948fb2e5c1117dc2d1efcf45e7 |
| openspec/review.json | absent |

## Plan review (2026-11-22)

Verdict approve-with-notes at head 2e9ec7680a63ad888079b2c3907d15208b524ec2.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-22 | test-coverage | should-fix | in-change | delta scenario 'Quit while only the line with the server waits' has no matrix row | review > absent-row — "Quit while only the line with the server waits" | add the row with its covering task or test |  |  | unmet-requirement |
| R-23 | test-coverage | should-fix | in-change | delta scenario 'Lines of two worlds wait' has no matrix row | review > absent-row — "Lines of two worlds wait" | add the row with its covering task or test |  |  | unmet-requirement |
| R-24 | test-coverage | should-fix | in-change | delta scenario 'No world in focus' has no matrix row | review > absent-row — "No world in focus" | add the row with its covering task or test |  |  | unmet-requirement |
| R-25 | test-coverage | should-fix | in-change | delta scenario 'A reply near the limit' has no matrix row | review > absent-row — "A reply near the limit" | add the row with its covering task or test |  |  | unmet-requirement |
| R-12 | verify | should-fix | in-change | PARTIAL: the fix e00b743f4068bcb6332142677496eeff92ead7be of R-12 did not hold. At the end of the extra wait, an open paste is given as pending whole. If a read stopped inside the end mark, ESC and [20 go into the paste and 1~ arrives as keys. | crates/mu-tui/src/probe.rs:588 — "let text = &pending[..pending.len() - held_for_paste(&pending)];" | fix it again | regression |  | wrong-behaviour |
| R-14 | verify | should-fix | in-change | The fix of R-14 puts `x` at column 2, but the sibling scenario counts columns from 1 (width 8 gives column 9). Counted that way, `x` after a width-2 grapheme is at column 3, which is what probe_with_no_answer checks. | specs/terminal-client/spec.md > Scenario: A terminal that does not answer — "after the game line `👨‍👩‍👧x`, the cell of `x` is at column 2, the `unicode-width` value of `👨‍👩‍👧`" | Write column 3 in 'A terminal that does not answer', counted from 1 as the sibling scenario counts, or count both scenarios from 0 and say so. | regression |  | unmet-requirement |
| R-15 | verify | should-fix | in-change | PARTIAL: the fix c47deba0add0c4c3ddaf5726cc3852ddeec4d192 of R-15 did not hold. Tasks 1.3, 1.4, 1.5, 2.1, 3.2 and 3.3 name tests but only say 'Make sure that they pass', with no command. | tasks.md > 3. A paste during the probe (#149) — "Make sure that it fails with the framer of `main` and passes with D6." | fix it again | regression |  | false-test |

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| Quit | 1.4 | partial |  |
| Quit while lines wait | 1.4 | partial |  |
| Another key between the two | 1.4 | partial |  |
| A line that the server refuses |  | missing |  |
| The last line is refused |  | missing |  |
| A refused send of Enter with a send after it |  | missing |  |
| The stream ends while lines wait | 1.3 | partial |  |
| The server resets while lines wait | 1.3 | partial |  |
| A refusal after the stream ended | 1.3 | partial |  |
| The lines become one line |  | missing |  |
| A world with no line break | 2.1 | partial |  |
| A line break that the editor cannot hold |  | missing |  |
| A world that the first list did not hold | 2.1 | partial |  |
| A line break that the client does not know | 2.1 | partial |  |
| A terminal that adds code point widths |  | missing |  |
| A terminal that does not answer |  | missing |  |
| A late answer |  | missing |  |
| A paste during the queries | 3.3 | partial |  |
| A paste split across reads | 3.2 | partial |  |
| A server that does not answer | 1.2 | covered |  |
| Two lines wait | 1.5 | partial |  |

### Not run

Plan phase: no verification runs.
