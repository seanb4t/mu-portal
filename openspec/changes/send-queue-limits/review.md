---
verdict: approve-with-notes
phase: plan
reviewed_at: 2026-11-22T18:11:32+00:00
head: 2e9ec7680a63ad888079b2c3907d15208b524ec2
round: 2
next_id: 26
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
| R-22 | test-coverage | should-fix | in-change | delta scenario 'Quit while only the line with the server waits' has no matrix row | review > absent-row — "Quit while only the line with the server waits" | add the row with its covering task or test |  |  | unmet-requirement |
| R-23 | test-coverage | should-fix | in-change | delta scenario 'Lines of two worlds wait' has no matrix row | review > absent-row — "Lines of two worlds wait" | add the row with its covering task or test |  |  | unmet-requirement |
| R-24 | test-coverage | should-fix | in-change | delta scenario 'No world in focus' has no matrix row | review > absent-row — "No world in focus" | add the row with its covering task or test |  |  | unmet-requirement |
| R-25 | test-coverage | should-fix | in-change | delta scenario 'A reply near the limit' has no matrix row | review > absent-row — "A reply near the limit" | add the row with its covering task or test |  |  | unmet-requirement |
| R-12 | verify | should-fix | in-change | PARTIAL: the fix e00b743f4068bcb6332142677496eeff92ead7be of R-12 did not hold. At the end of the extra wait, an open paste is given as pending whole. If a read stopped inside the end mark, ESC and [20 go into the paste and 1~ arrives as keys. | crates/mu-tui/src/probe.rs:588 — "let text = &pending[..pending.len() - held_for_paste(&pending)];" | fix it again | regression |  | wrong-behaviour |
| R-14 | verify | should-fix | in-change | The fix of R-14 puts `x` at column 2, but the sibling scenario counts columns from 1 (width 8 gives column 9). Counted that way, `x` after a width-2 grapheme is at column 3, which is what probe_with_no_answer checks. | specs/terminal-client/spec.md > Scenario: A terminal that does not answer — "after the game line `👨‍👩‍👧x`, the cell of `x` is at column 2, the `unicode-width` value of `👨‍👩‍👧`" | Write column 3 in 'A terminal that does not answer', counted from 1 as the sibling scenario counts, or count both scenarios from 0 and say so. | regression |  | unmet-requirement |
| R-15 | verify | should-fix | in-change | PARTIAL: the fix c47deba0add0c4c3ddaf5726cc3852ddeec4d192 of R-15 did not hold. Tasks 1.3, 1.4, 1.5, 2.1, 3.2 and 3.3 name tests but only say 'Make sure that they pass', with no command. | tasks.md > 3. A paste during the probe (#149) — "Make sure that it fails with the framer of `main` and passes with D6." | fix it again | regression |  | false-test |

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

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 09078fd397442e8a076d8489ccd35a3c7ed9a8cb | 3f67ca37e2f3f07842d8a278c4aa11f50c66efa10edbe742eac74b831b07f731 | changes-requested | 21 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 2e9ec7680a63ad888079b2c3907d15208b524ec2 | 3daad9b6805f725d68f5061772d2543502b26610e665da4e35d413fe0b96b258 | approve-with-notes | 4 | 0 | 3 | 14 | 0 |  |

## Judge

judge: 6 questions, 0.000 USD

## Coverage matrix

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

## Verification runs

None.

## Dropped

None.

## Lens notes

- verify: R-12: proposal.md 'Choices the owner will notice' accepts that the rest of a paste still open after the extra 500 ms arrives as keys. The answer PARTIAL holds the claim literally: the held bytes of a cut end mark are dropped, and the bytes of the mark after the cut still reach Termina as keys.
- verify: R-11: lost_server_puts_back_the_lines_in_the_order_of_the_worlds queues beta/bryn before alpha/aria and expects alpha first, so it tells the order of the worlds from the order of the sends. An order by the HashMap still passes it in about half of the runs.

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
