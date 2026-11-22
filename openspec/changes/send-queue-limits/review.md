---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-22T18:37:23+00:00
head: dd6f32ab242c347831b7aed1a5c28c00c5801157
round: 3
next_id: 35
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: ok
  invariants: ok
  verify: not-applicable
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-26 | approach,invariants | should-fix | in-change | app-1: The requirement and the proposal give every list of the worlds a 30 s limit, but D1 bounds only the list of Effect::ListWorlds. The first list at startup in run::run still calls client.list_worlds() with no limit. A server that never answers it keeps the client from its first frame, and that list never counts as failed.; inv-1: The spec says a list with no reply within 30 seconds counts as failed, but the first list at startup is awaited with no limit. A server that accepts the connection and never answers ListWorlds keeps the client before its first frame for good. Only the lists of Effect::ListWorlds go through within. | specs/terminal-client/spec.md > Requirement: Alt+R joins the lines in place — "A list with no reply within 30 seconds SHALL count as failed." | app-1: Name the startup list in D1, and wrap the call at run.rs:29 in `within` with CALL_LIMIT, so a list with no answer becomes the failed first list that from_listing already handles.; inv-1: Wrap the startup list in run::within with CALL_LIMIT (mapping the error to the notice of from_listing), or state in the requirement and D1 that the limit covers only the lists after startup. |  | severity? blocker 0.73 | unmet-requirement |
| R-27 | fit | should-fix | in-change | mu-tui now has two paths for ListWorlds. The new Effect::ListWorlds goes through run::within with CALL_LIMIT, but the startup list in run::run calls client.list_worlds() with no limit and another error type. The two paths already differ. | crates/mu-tui/src/run.rs:29 — "let mut model = Model::from_listing(client.list_worlds().await);" | Route the startup list through run::within with CALL_LIMIT too, and let Model::from_listing take the Result<_, String> that Msg::Listed carries. |  |  | drift |
| R-28 | fit | should-fix | in-change | The change adds an alt_r() helper to app.rs but leaves four inline copies of the same Alt+R key in the Alt+R tests beside it, two of them as a local named alt_r. Two mechanisms build one key. | crates/mu-tui/tests/app.rs:1264 — "fn alt_r() -> KeyEvent {" | Move alt_r() next to alt_enter() and call it at lines 1114, 1135, 1158 and 1171. |  | severity? suggestion 0.67 | drift |
| R-29 | fit | should-fix | in-change | The key table of the first-run guide restates the Ctrl+C rule as a plain quit. The change makes the first Ctrl+C with waiting lines show a notice and not quit, and the guide is not in the diff, so a user who follows the guide gets a client that does not quit. | docs/src/content/docs/guides/first-run.md:61 — "\| Ctrl+C \| Quits the client. The server and its worlds keep running. \|" | Update the Ctrl+C row of first-run.md: with lines that wait, the first Ctrl+C names their count and a second Ctrl+C quits. |  |  | wrong-behaviour |
| R-30 | invariants | should-fix | in-change | The spec says the client asks for the list after the server resets. If a list is Asked when the Reset comes, list_worlds returns None, and no list goes after the reset. That list was asked of the old server, so a failed or old answer leaves the line breaks of before the reset, and nothing asks again until a new world or Alt+R on an unlisted world. | specs/terminal-client/spec.md > Requirement: Alt+R joins the lines in place — "The client SHALL also ask for that list after the first event of a world that it did not hold, and after the server rese" | Keep a flag that a reset came while a list was Asked, and send ListWorlds at the Msg::Listed of that list. Add a test: unlisted_world, then Reset, then Listed(Err) gives [Effect::ListWorlds]. |  | repeat? R-8 0.76 | unmet-requirement |
| R-31 | invariants | should-fix | in-change | D6 says a cut UTF-8 character at the end of a paste that is still open is not part of its text, enforced by held_for_paste through unfinished_tail. No test ends an open paste inside a multi-byte character. cut_end_mark_is_not_text_of_the_paste only cuts the end mark, so a regression gives a U+FFFD in the editor and passes. | design.md > D6. The probe framer holds a whole paste — "A cut end mark or a cut UTF-8 character at its end is not part of that text." | Add a read_answers test with the bytes ESC [?62c ESC [200~ one F0 9F 91 (a cut thumbs-up), and assert the events are [Event::Paste("one")]. |  |  | wrong-behaviour |
| R-34 | test-coverage | should-fix | in-change | covered row 'A reply near the limit' has no verification run | review > not-run — "A reply near the limit" | run the covering test and record it in verification-runs.txt |  |  | unmet-requirement |
| R-32 | approach | suggestion | in-change | design.md says that a send to a world that connects waits behind the connect, up to 20 s on the server. At head the session refuses a Send at once unless the connection is Up and Live, so a send never waits behind a connect. The 30 s limit still holds over the server's bounds, but its stated reason is wrong. | design.md > Context — "A send to a world that connects waits behind the connect." | Replace the connect case in Context and D1 with the real one: the session handles commands one at a time, so a send can wait behind another write of up to WRITE_TIMEOUT, and a send during a connect or a login is refused at once. |  | severity? should-fix 0.82 |  |
| R-33 | invariants | suggestion | in-change | The text of a paste still open at the end of the probe is, for any cut point, a prefix of the pasted text with no byte of the end mark and no replacement character. The property test covers only a closed paste. | crates/mu-tui/tests/probe.rs:459 — "fn paste_split_anywhere_gives_its_text(" | Add a proptest in crates/mu-tui/tests/probe.rs, next to paste_split_anywhere_gives_its_text: cut the bytes of start mark, text and end mark at any point, run read_answers on the prefix, and assert the paste text is a prefix of the text with no U+FFFD. |  |  |  |

## Repeats

None.

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

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 09078fd397442e8a076d8489ccd35a3c7ed9a8cb | 3f67ca37e2f3f07842d8a278c4aa11f50c66efa10edbe742eac74b831b07f731 | changes-requested | 21 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 2e9ec7680a63ad888079b2c3907d15208b524ec2 | 3daad9b6805f725d68f5061772d2543502b26610e665da4e35d413fe0b96b258 | approve-with-notes | 4 | 0 | 3 | 14 | 0 |  |
| 3 | implementation | full | dd6f32ab242c347831b7aed1a5c28c00c5801157 | 3af5daad20d2335fe8ef8a2fd1701c9039a81a9106287e1801c41c592fb33987 | approve-with-notes | 9 | 0 | 0 | 0 | 0 |  |

## Judge

judge: 17 questions, 0.001 USD

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

## Dropped

None.

## Lens notes

- approach: Model::learn also adds each listed world that the model does not hold (model.rs:217-229). D5 says only that learn sets the line break and marks the world as listed. After a Reset, a list can thus add worlds with no event to the top row, and give the focus to one when none has it. No requirement forbids it; D5 can name it.
- approach: The third-party facts hold at the locked versions: connectrpc 0.9.1 sends CallOptions::with_timeout as connect-timeout-ms or grpc-timeout, its default_timeout bounds each message() poll of a stream, and the server drops the handler at that deadline; termina 0.3.3 has Event::Paste(String); the dev-dependency of tokio has test-util.
- approach: Every test that design.md and tasks.md name exists at head under the named file.
- approach: CONNECT_TIMEOUT and WRITE_TIMEOUT are 10 s each at crates/mu-core/src/session.rs:258 and :262, as design.md Context says.
- fit: mu-cli/src/client.rs:50 also calls client.list_worlds() with no limit before mu_tui::run starts. It is outside mu-tui, so fit-1 does not collapse it.
- fit: The rule 'the first send of a world is with the server' is read in three places (act Quit, view::top_row, return_waiting). D4 names #157 for the shared method, and R-17 is deferred, so no new finding.
- fit: R-19 (termina Parser holds a paste) and R-20 (Skip::Paste holds bytes, read_answers builds Event::Paste again) are closed. Their code at head is as the rows describe, with no new evidence.
- fit: The change adds three more inline ClientEvent::Reset { instance_id: "i" } literals in app.rs (1219, 1318, 1380) beside four older ones. A struct literal that the compiler checks cannot drift apart silently, so it is not filed.
- fit: Library rows of openspec/config.yaml were read. The tokio test-util dev-dependency follows the workspace pattern of the other crates.
- invariants: I1: One text per world is with the server at a time \| enforced at update.rs enqueue:437-440 (send only when the queue was empty), next_send:650, return_waiting drain(1..):633, on_reply remove:600 \| boundary test sends_to_a_world_go_one_at_a_time, lost_server_puts_the_waiting_lines_back (reply after a loss sends nothing). The property over any sequence is R-21, deferred, and is not raised again.
- invariants: I2: A send call ends within CALL_LIMIT, inclusive of the limit \| enforced at run.rs within:157 and reply_of:174 \| boundary test answer_just_inside_the_limit_is_sent and answer_just_past_the_limit_is_refused (limit minus and plus 1 ms on a paused clock).
- invariants: I3: A list call ends within CALL_LIMIT \| enforced at run.rs:122 for Effect::ListWorlds, none for the startup list at run.rs:29 \| boundary test list_with_no_answer_fails. See inv-1.
- invariants: I4: At most one list is with the server \| enforced at update.rs list_worlds:489, cleared at update.rs:158 \| boundary test one_list_at_a_time. A reset during an Asked list asks none; see inv-2.
- invariants: I5: Quit leaves at once if and only if no text of any world waits, the text with the server included \| enforced at update.rs:349-350 \| boundary test quit_with_no_waiting_send_leaves_at_once (0) and quit_with_only_the_line_with_the_server_asks_again (1).
- invariants: I6: Any key that is not Quit cancels the second Quit \| enforced at update.rs:294-296 \| boundary test another_key_cancels_the_second_quit. Only DISAMBIGUATE_ESCAPE_CODES is pushed (screen.rs:65), so a lone modifier press is no key event and does not cancel it.
- invariants: I7: The top row count is the queue length less one, and shows nothing at 0 \| enforced at view.rs:125-126 \| boundary test top_row_counts_the_sends_that_wait (2, 1, then 0 with one text still with the server).
- invariants: I8: A queue in Model::sends is never empty \| enforced by enqueue callers (submit passes one text, submit_lines at least one from split), next_send:655, on_reply:600 \| boundary test none. return_waiting calls drain(1..), which panics on an empty VecDeque, so this invariant is load-bearing.
- invariants: I9: Each key of Model::sends is a world of Model::worlds \| enforced at update.rs world_to_send_to:418, and worlds are never removed \| boundary test none needed. return_waiting iterates worlds, so a key outside them would never be returned.
- invariants: I10: A line break that is empty or holds a control character counts as none, from the startup list and from a later list \| enforced at model.rs learn:224-226, which from_listing and Msg::Listed both call \| boundary test line_break_that_the_editor_cannot_hold_is_no_line_break (empty, ESC, LF).
- invariants: I11: With no world in focus Alt+R asks for no list and keeps the editor \| enforced at update.rs:464-466 \| boundary test alt_r_with_no_world_names_the_line_break asserts no effect.
- invariants: I12: A bracketed paste split at any byte gives one paste with the whole text \| enforced at probe.rs frame:162 (end mark search starts PASTE_END.len()-1 bytes back) and paste:180 \| boundary test the proptest paste_split_anywhere_gives_its_text (empty text included). Its cuts are drawn from 0..80, so the end mark of the longer texts is never cut, but the short texts cover it.
- invariants: I13: The probe reads an open paste for at most one more wait \| enforced at probe.rs:573 \| boundary test open_paste_keeps_the_probe_reading asserts elapsed at most 2 * WAIT, and ended_paste_does_not_hold_the_probe.
- invariants: I14: A cut end mark or cut UTF-8 character at the end of an open paste is not text \| enforced at probe.rs held_for_paste:197 and :588 \| boundary test cut_end_mark_is_not_text_of_the_paste for the mark only; see inv-3. A Python port of held_for_paste and unfinished_tail (inv-probe/held.py under the handoff directory) gives: one+F0 9F 91 holds 3 bytes and yields one; one+ESC [20 holds 4; one+ESC x holds 2, so a pasted ESC and one byte at the very end are dropped, not given as keys.
- invariants: The held bytes of an open paste are dropped at probe.rs:588 rather than given as keys; that is R-12 (accepted) and is not raised again.
- invariants: The spec says the reason of a timed-out send says that the game can have the line or not (run.rs:174). send_with_no_answer_is_refused asserts only 'no answer in 30 s'. This is a coverage question for another lens, recorded here only as the enforcement point.
- test-coverage: Tests were read, not run. Task 3.3 claims the paste test fails with the framer of main; that was not checked.
- test-honesty: Read every added test in tests/send.rs, tests/app.rs (lines 1109-1387) and tests/probe.rs (424-514). Each asserts the THEN of its scenario against real update/framer/reply_of code; no mock-echo, tautology, disabled, or vacuous test found.
- test-honesty: The boundary tests in send.rs use start_paused with 1 ms either side of CALL_LIMIT, so an early or late limit fails them.
- test-honesty: lost_server_puts_back_the_lines_in_the_order_of_the_worlds loops 32 times to cover hash orders, so it can fail for a map-order bug.

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
| R-14 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Scenario: A terminal that does not answer"],"AND the client uses the `unicode-width` value of each grapheme"] | fixed | 9861b0d778c541f75189d0604b919e1312e7e045 |  |  | The THEN 'the client uses the unicode-width value of each grapheme' names no observable: no cell column, no rendered text, no returned value. |
| R-15 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["tasks.md","1. A send with no reply, and the texts that wait (#147)"],"Make sure that they pass."] | fixed | 9861b0d778c541f75189d0604b919e1312e7e045 |  |  | Tasks 1.3, 1.4, 1.5, 2.1, 3.2 and 3.3 name tests but only say 'Make sure that they pass', with no command. |
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
| R-26 | 3 | 3 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: Alt+R joins the lines in place"],"A list with no reply within 30 seconds SHALL count as failed."] | fixed | 7ef2ca230c93227b0461dacd4b5c44765d031260 |  | severity? blocker 0.73 | app-1: The requirement and the proposal give every list of the worlds a 30 s limit, but D1 bounds only the list of Effect::ListWorlds. The first list at startup in run::run still calls client.list_worlds() with no limit. A server that never answers it keeps the client from its first frame, and that list never counts as failed.; inv-1: The spec says a list with no reply within 30 seconds counts as failed, but the first list at startup is awaited with no limit. A server that accepts the connection and never answers ListWorlds keeps the client before its first frame for good. Only the lists of Effect::ListWorlds go through within. |
| R-27 | 3 | 3 | should-fix | in-change | drift | ["path","crates/mu-tui/src/run.rs",29] | fixed | 7ef2ca230c93227b0461dacd4b5c44765d031260 |  |  | mu-tui now has two paths for ListWorlds. The new Effect::ListWorlds goes through run::within with CALL_LIMIT, but the startup list in run::run calls client.list_worlds() with no limit and another error type. The two paths already differ. |
| R-28 | 3 | 3 | should-fix | in-change | drift | ["path","crates/mu-tui/tests/app.rs",1264] | fixed | 7ef2ca230c93227b0461dacd4b5c44765d031260 |  | severity? suggestion 0.67 | The change adds an alt_r() helper to app.rs but leaves four inline copies of the same Alt+R key in the Alt+R tests beside it, two of them as a local named alt_r. Two mechanisms build one key. |
| R-29 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["path","docs/src/content/docs/guides/first-run.md",61] | fixed | bda7c0d9dabb58add3edb90e16790bc765d838c7 |  |  | The key table of the first-run guide restates the Ctrl+C rule as a plain quit. The change makes the first Ctrl+C with waiting lines show a notice and not quit, and the guide is not in the diff, so a user who follows the guide gets a client that does not quit. |
| R-30 | 3 | 3 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: Alt+R joins the lines in place"],"The client SHALL also ask for that list after the first event of a world that it did not hold, and after the server resets."] | fixed | 7ef2ca230c93227b0461dacd4b5c44765d031260 |  | repeat? R-8 0.76 | The spec says the client asks for the list after the server resets. If a list is Asked when the Reset comes, list_worlds returns None, and no list goes after the reset. That list was asked of the old server, so a failed or old answer leaves the line breaks of before the reset, and nothing asks again until a new world or Alt+R on an unlisted world. |
| R-31 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. The probe framer holds a whole paste"],"A cut end mark or a cut UTF-8 character at its end is not part of that text."] | fixed | 7ef2ca230c93227b0461dacd4b5c44765d031260 |  |  | D6 says a cut UTF-8 character at the end of a paste that is still open is not part of its text, enforced by held_for_paste through unfinished_tail. No test ends an open paste inside a multi-byte character. cut_end_mark_is_not_text_of_the_paste only cuts the end mark, so a regression gives a U+FFFD in the editor and passes. |
| R-32 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","Context"],"A send to a world that connects waits behind the connect."] | fixed | bda7c0d9dabb58add3edb90e16790bc765d838c7 |  | severity? should-fix 0.82 | design.md says that a send to a world that connects waits behind the connect, up to 20 s on the server. At head the session refuses a Send at once unless the connection is Up and Live, so a send never waits behind a connect. The 30 s limit still holds over the server's bounds, but its stated reason is wrong. |
| R-33 | 3 | 3 | suggestion | in-change |  | ["path","crates/mu-tui/tests/probe.rs",459] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/159; reason: A broader property test of an open paste; the cut cases have example tests. Owner chose to defer it. |  |  | The text of a paste still open at the end of the probe is, for any cut point, a prefix of the pasted text with no byte of the end mark and no replacement character. The property test covers only a closed paste. |
| R-34 | 3 | 3 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"A reply near the limit"] | refuted | Both covering tests of the row ran and passed: verification-runs.txt has exit 0 for answer_just_inside_the_limit_is_sent and answer_just_past_the_limit_is_refused, on two lines because the row names two tests. |  |  | covered row 'A reply near the limit' has no verification run |

## Reviewed files

| path | content |
|---|---|
| crates/mu-tui/Cargo.toml | b89f2c80623814f900413f4e9e6e2c23752aeffc |
| crates/mu-tui/src/lib.rs | 17a66cb024d4d73909d76a8deaebb3d451ea9ede |
| crates/mu-tui/src/model.rs | d8fc6b611e1a813224bebeed123e6643b08407b1 |
| crates/mu-tui/src/probe.rs | e0c75ef039ae12c1020b382cea551a144e0da783 |
| crates/mu-tui/src/run.rs | ee85404f31c0122415432b6d0007a4abafe90f52 |
| crates/mu-tui/src/update.rs | f95db50e736e9e71035116cf923412c9f4bc2b9a |
| crates/mu-tui/src/view.rs | 415d0a746078255bc29ef49207aabe7ecb717650 |
| crates/mu-tui/tests/app.proptest-regressions | 81f7accb35b1da9e96d79f56e22f71eed5a9b06d |
| crates/mu-tui/tests/app.rs | e9757e9c67064b55ef6192e2e3e923985d1fafc1 |
| crates/mu-tui/tests/probe.rs | 5b4409784788b04156996c891501e02ea7368644 |
| crates/mu-tui/tests/send.rs | 2d320a2bd62ed7afab07f0831a651e9f3b55074a |
| openspec/changes/send-queue-limits/.openspec.yaml | b6809d6be7a6faad01a1bd5da9bea068ac829d8f |
| openspec/changes/send-queue-limits/design.md | 128685163ca9e1fc9c40dcb8dcac724123400e46 |
| openspec/changes/send-queue-limits/proposal.md | ee6551ac90d614c5a96f5a9370061e23de0edec7 |
| openspec/changes/send-queue-limits/specs/terminal-client/spec.md | 06395c06d23a39cbd23b218e92ac7cce29fb1515 |
| openspec/changes/send-queue-limits/tasks.md | sha256:d3497f96e3f1b7636fa4790e0eecd47f7d71b29a65f2e6090c192a5795909066 |
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
