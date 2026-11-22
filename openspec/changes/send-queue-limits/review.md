---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-22T17:29:39+00:00
head: 09078fd397442e8a076d8489ccd35a3c7ed9a8cb
round: 1
next_id: 22
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
| R-1 | approach,invariants | blocker | in-change | app-1: The modified Alt+R requirement still says that with no world in focus the client shows a `-- ` line that contains `line break`. D5 splits Alt+R into 'not known yet', 'no line break' and 'join', and names no outcome for no world in focus. The join_lines at head returns early through `?` before any hint, so Alt+R with no world in focus shows no line. The join_lines of main gave the 'sets no line break' hint in that case, so the change drops the behaviour.; inv-2: join_lines now returns early with no notice when no world is in focus. The old guard showed the line break hint there, and the requirement still demands it. | specs/terminal-client/spec.md > Requirement: Alt+R joins the lines in place — "If that world has no `line_break`, or no world is in focus, the client SHALL NOT change the editor." | app-1: Add the no-focus case to D5: with no world in focus, Alt+R shows the `line break` notice and asks for no list. In join_lines, give the hint when focused_world() is None instead of returning through `?`, and cover it with a test.; inv-2: Show the hint when focused_world() is None before the listed check, and add a test that presses Alt+R on failed_listing() (no world) and asserts a -- line with line break. |  |  | unmet-requirement |
| R-2 | test-coverage | blocker | in-change | Scenario 'A line break that the editor cannot hold' has no task and no named test. | specs/terminal-client/spec.md > Scenario: A line break that the editor cannot hold — "gives `alpha/aria` an empty `line_break`, or one with ESC or LF" | Add a task 2.2 that names a test in crates/mu-tui/tests/app.rs for an empty and an ESC/LF line_break, with `cargo nextest run -p mu-tui --test app --locked`. |  |  | unmet-requirement |
| R-3 | test-coverage | blocker | in-change | Scenarios 'A line that the server refuses', 'The last line is refused' and 'A refused send of Enter with a send after it' name no task or test. | tasks.md > 1. A send with no reply, and the texts that wait (#147) — "`failed_send_is_refused_with_its_reason` and `answered_send_is_sent` in `crates/mu-tui/tests/send.rs`" | Name in tasks 1.x the app.rs tests that prove each of the three scenarios, with a nextest command. |  | severity? should-fix 0.72 | unmet-requirement |
| R-4 | test-coverage | blocker | in-change | Scenarios 'The lines become one line', 'A terminal that adds code point widths', 'A terminal that does not answer' and 'A late answer' have no task. | tasks.md > 3. A paste during the probe (#149) — "Add `Skip::Paste`, `PASTE_START` and `PASTE_END`, and frame a paste as one event (D6)." | Name the test of each scenario in tasks 2.1 and 3.x (app.rs or probe.rs), or add a task that checks they still pass, with a nextest command. |  | severity? should-fix 0.72 | unmet-requirement |
| R-5 | fit | should-fix | deviates-design | D1 hand-wraps one call in tokio::time::timeout, where connectrpc gives each call a timeout. The new ListWorlds call gets no bound, so a list with no reply leaves Listing::Asked for the session. | design.md > D1. The limit of a send is in the event loop, and it gives a `Reply::Refused` — "Alternative: a timer message in the model. It needs a clock in `update`, or a second message for each send."; scope basis: design.md > D1. The limit of a send is in the event loop, and it gives a `Reply::Refused` — "`run::reply_of` wraps the call in `tokio::time::timeout` with `SEND_LIMIT` (10 s)." | Name connectrpc's per-call timeout (CallOptions::with_timeout through the generated *_with_options methods) in D1 and use it for send and list_worlds, or state why tokio::time::timeout is kept. Do not use ClientConfig's default timeout: it would also end the event stream. |  |  | wrong-behaviour |
| R-6 | fit | should-fix | deviates-design | SEND_LIMIT equals the server's own WRITE_TIMEOUT, which D1 does not name. A stalled game write gets the server's sure refusal at the same moment the client gives up with the vague reason. | crates/mu-tui/src/run.rs:134 — "pub const SEND_LIMIT: Duration = Duration::from_secs(10);"; scope basis: proposal.md > Choices the owner will notice — "The limit of a send is 10 seconds, fixed. No configuration changes it." | Name the server's WRITE_TIMEOUT (and the CONNECT_TIMEOUT a queued send can wait behind) in D1. Set SEND_LIMIT above the server's own bound so its answer arrives first, or state why the two are equal. |  | scope? in-change 0.64 | wrong-behaviour |
| R-7 | fit | should-fix | in-change | return_waiting reports a one-time event with Model::notice, which every later Reset shows again. The existing helper for a one-time line is Model::hint, which the Quit notice of this change uses. | crates/mu-tui/src/update.rs:636 — "model.notice(&format!(" | Use model.hint in return_waiting, and name the notice channel in D2. Otherwise a later Reset shows stale lines such as 'the client lost the server, so 2 lines that waited went back'. |  |  | wrong-behaviour |
| R-8 | invariants | should-fix | in-change | One list at a time holds only while each list call ends. ListWorlds has no time limit, so a list with no reply leaves Listing::Asked for good, and no reset, new world or Alt+R asks again. | specs/terminal-client/spec.md > Requirement: Alt+R joins the lines in place — "The client SHALL also ask for that list after the first event of a world that it did not hold, and after the server rese" | Bound the list call as D1 bounds a send (tokio::time::timeout with a limit, a late call becomes Msg::Listed(Err)), or set Listing::Idle on Reset. Add a task and a paused-clock test for a list with no answer. |  |  | wrong-behaviour |
| R-9 | invariants | should-fix | in-change | No test passes a reply on either side of SEND_LIMIT. The tests use a call that never ends and a call that is ready at once, so a limit that fires early still passes. | specs/terminal-client/spec.md > Requirement: A send with no reply is refused — "If the server gives no reply to a text within 10 seconds, the client SHALL count the text as refused." | On the paused clock, add a call that answers at SEND_LIMIT minus 1 ms (expect Sent) and one that answers at SEND_LIMIT plus 1 ms (expect Refused). |  |  | false-test |
| R-10 | invariants | should-fix | in-change | The Quit guard is waiting == 0, and the tests pass 0 and 3. No test passes 1, the line with the server alone, so a guard that counts as the top row does still passes. | crates/mu-tui/src/update.rs:350 — "if waiting == 0 \|\| model.quit_armed {" | Add a test with one line sent and no reply: the first Ctrl+C returns no effect and shows 1 line not sent yet, the second quits. |  |  | false-test |
| R-11 | invariants | should-fix | in-change | return_waiting promises the order of the worlds, but each loss and reset test holds one world. An order by the HashMap of sends passes too. | specs/terminal-client/spec.md > Requirement: A refused text stops the sends after it — "the client SHALL put each text that waits behind the text with the server back in the editor, in the order of the worlds" | Add a test with waiting lines in alpha/aria and beta/bryn, queued beta first, and assert that Lost puts alpha's lines before beta's. |  |  | false-test |
| R-12 | invariants | should-fix | in-change | At the end of the extra wait, an open paste is given as pending whole. If a read stopped inside the end mark, ESC and [20 go into the paste and 1~ arrives as keys. | design.md > D6. The probe framer holds a whole paste — "A paste still open after that becomes one paste of its first part." | Hold back a tail of pending that is a prefix of PASTE_END (and an incomplete UTF-8 character) at the timeout, and test read_answers with an open paste that ends in ESC [20. |  |  | wrong-behaviour |
| R-13 | spec-lint | should-fix | in-change | No task line covers the requirement 'The client measures character widths at startup' (mode 2027 query, cursor-position queries, device attributes end, the late answer). | specs/terminal-client/spec.md > Requirement: The client measures character widths at startup — "Before its first frame, the client SHALL ask the terminal for the state of mode 2027." | Add a task that names the width queries and the tests of the scenarios 'A terminal that adds code point widths', 'A terminal that does not answer' and 'A late answer'. |  |  | unmet-requirement |
| R-14 | spec-lint | should-fix | in-change | The THEN 'the client uses the unicode-width value of each grapheme' names no observable: no cell column, no rendered text, no returned value. | specs/terminal-client/spec.md > Scenario: A terminal that does not answer — "AND the client uses the `unicode-width` value of each grapheme" | Name the observable, for example the cell column of the next grapheme after a wide one, or the width value returned by the pane. |  |  | unmet-requirement |
| R-15 | test-coverage | should-fix | in-change | Tasks 1.3, 1.4, 1.5, 2.1, 3.2 and 3.3 name tests but only say 'Make sure that they pass', with no command. | tasks.md > 1. A send with no reply, and the texts that wait (#147) — "Make sure that they pass." | Add `cargo nextest run -p mu-tui --test app --locked` (and `--test probe`) to each of these tasks. |  |  | unmet-requirement |
| R-16 | approach | suggestion | in-change | D2 rejects clearing each whole queue because a late Refused would put the first text back after the others. put_back puts the returned lines before the text of the editor, and a refusal with no queue returns only its own text. So under a cleared queue a late Refused of `one` lands before `two` and `three`, in order. The real cost of clearing, a new send that goes while the old call is still open, against the Goal of one text per world with the server, is not stated. | design.md > D2. A loss or a reset returns the texts behind the one with the server — "Alternative: clear each whole queue. Then a late `Refused` puts the first text back after the others, out of order." | Restate the D2 rejection: a cleared queue lets the next send of the world go while the first call is still open, so two texts of one world are with the server at once. |  | severity? should-fix 0.61 |  |
| R-17 | fit | suggestion | in-change | D4 says the #101 statusline takes the count later, but the count lives inline in view::top_row. #101 must move it or copy the 'first one is with the server' rule, which return_waiting already repeats. | design.md > D4. The count in the top row — "The statusline of client-shell (#101) can take the count later." | Add a Model method for the texts that wait behind the one with the server. Have top_row and return_waiting read it, so #101 reads the same method. |  | severity? should-fix 0.80 |  |
| R-18 | fit | suggestion | in-change | The new Effect::ListWorlds arm of run::event_loop repeats the spawn-and-reply block of Effect::Send. Only the call and the Msg variant differ. | crates/mu-tui/src/run.rs:105 — "let client = client.clone();" | Write one spawn helper that takes the call future, which yields a Msg, and use it from both arms. The call future is the parameter, so a limit as in fit-1 applies to both arms. |  |  |  |
| R-19 | fit | suggestion | in-change | D6 writes its own bracketed-paste framing across reads. termina's Parser, which the framer already uses, holds an open paste across parse calls and gives one Event::Paste. D6 does not name it. | design.md > D6. The probe framer holds a whole paste — "When the framer finds `ESC [200~`, it sets `Skip::Paste`, and each later read goes into `pending` with no limit." | Name termina's incremental Parser as the alternative in D6, with the reason it is not used, for example its rescan of the held buffer on each parse call (parse.rs line 119). Or hold one Parser with maybe_more = true while a paste is open. |  |  |  |
| R-20 | fit | suggestion | in-change | Skip::Paste joins an enum of bytes to drop, but it holds bytes. skipped must make it a special case, and read_answers builds a paste event a second time. | crates/mu-tui/src/probe.rs:122 — "/// What the framer drops at the start of the next read: the rest of a sequence that passed" | Keep the open paste as its own framer state, apart from Skip, or rename Skip and its doc to the framer state. Build the paste event in one function. Name the choice in D6. |  | severity? should-fix 0.71 |  |
| R-21 | invariants | suggestion | in-change | The send queue has an invariant over any sequence of sends, replies, losses and resets: each text is once in the queue, the editor or the server, and one Send per world is open. | design.md > Goals / Non-Goals — "One text per world is with the server at a time, as the requirement "The sends to a world go in order" says." | Add a proptest (proptest 1.12, as in crates/mu-tui/tests/probe.rs) that drives update with random Enter, Alt+Enter, Sent, Refused, Lost and Reset steps and checks that no text is lost or doubled. |  |  |  |

## Repeats

None.

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-17 | suggestion | deferred | Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. | https://github.com/seanb4t/mu-portal/issues/157 |
| R-19 | suggestion | deferred | Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. | https://github.com/seanb4t/mu-portal/issues/157 |
| R-20 | suggestion | deferred | Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. | https://github.com/seanb4t/mu-portal/issues/157 |
| R-21 | suggestion | deferred | Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. | https://github.com/seanb4t/mu-portal/issues/157 |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 09078fd397442e8a076d8489ccd35a3c7ed9a8cb | 3f67ca37e2f3f07842d8a278c4aa11f50c66efa10edbe742eac74b831b07f731 | changes-requested | 21 | 0 | 0 | 0 | 0 |  |

## Judge

judge: 42 questions, 0.001 USD

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

- spe-3 (spec-lint, confidence 0.6): The THEN 'the client sends nothing more' names no observable: no server record and no screen line is named.

## Lens notes

- approach: D1, D3, D4, D5 and D6 match the code at head: reply_of with SEND_LIMIT in run.rs, quit_armed in on_input_key, top_row reads sends, Listing Idle/Asked with learn and World::listed, Skip::Paste with the 5-byte back search and the read_answers loop to deadline + wait.
- approach: The facts about third-party tools hold at the locked versions: tokio 1.54.1 timeout cancels the future on elapse, auto-advance under a paused clock is documented in time/clock.rs, test-util is in the dev-dependencies of mu-tui, and termina 0.3.3 has Event::Paste(String).
- approach: Every test name that D1 to D6 and tasks.md cite exists in crates/mu-tui/tests/send.rs, app.rs or probe.rs.
- approach: The commit hashes in tasks.md (35815e2, a183f32, d1c98f3, a5cf99d) were not checked: this lens has no git tool.
- approach: Keys that the Late states of on_key take before on_input_key do not turn quit_armed off. These keys exist only while late probe answers come in, so no finding is raised.
- fit: The library rows of openspec/config.yaml were read. The dev-dependency tokio test-util feature in crates/mu-tui/Cargo.toml follows the same line in five other crates.
- fit: D5 reuses Model::learn from Model::from_listing, so the startup list and the later list share one path for line breaks.
- fit: The plan-phase code is already on the branch at head, so the findings cite it with the decisions.
- invariants: I1: One text per world is with the server at a time \| enforced at update.rs enqueue (idle = queue.is_empty()) and next_send \| boundary test refused_send_stops_the_sends_after_it, top_row_counts_the_sends_that_wait (no property test, inv-7)
- invariants: I2: A send ends within SEND_LIMIT (10 s) \| enforced at run.rs:145 tokio::time::timeout \| boundary test none for both sides of the limit (inv-3)
- invariants: I3: A loss or reset leaves exactly the head text in each queue \| enforced at update.rs:630 queue.drain(1..); no empty queue stays in the map (enqueue adds at least one text, next_send and on_reply remove an emptied queue), so drain(1..) cannot panic \| boundary test lost_server_puts_the_waiting_lines_back (one world only, inv-5); the len 1 queue (no notice) is untested
- invariants: I4: Quit with any text in a queue asks again \| enforced at update.rs:350 \| boundary test none at waiting 1 (inv-4)
- invariants: I5: Any key other than Quit cancels the second Quit \| enforced at update.rs on_input_key \| boundary test another_key_cancels_the_second_quit. Keys that the late-answer states swallow (Alt+_ in Late::Waiting, cut CSI bytes) do not reach on_input_key and do not cancel.
- invariants: I6: At most one list of the worlds is with the server \| enforced at update.rs:486 list_worlds \| boundary test one_list_at_a_time; liveness unenforced (inv-1)
- invariants: I7: Alt+R never changes the editor for an unlisted world, a world with no line break, or no world in focus \| enforced at update.rs join_lines \| no-focus case lost its hint (inv-2), no test
- invariants: I8: A listed line_break is never empty and holds no control character \| enforced at model.rs learn filter \| boundary test line_break_that_the_editor_cannot_hold_is_no_line_break (empty, ESC, LF)
- invariants: I9: A bracketed paste framed by the probe is one Event::Paste with its whole text \| enforced at probe.rs paste and frame_to_paste \| boundary test paste_split_anywhere_gives_its_text (proptest, includes the empty text and cuts in both marks), paste_longer_than_the_cap_goes_through
- invariants: I10: The probe reads at most wait more for an open paste \| enforced at probe.rs read_answers loop condition \| boundary test open_paste_keeps_the_probe_reading (elapsed <= WAIT * 2)
- invariants: I11: The top row count is queue length minus one, never negative \| enforced at view.rs saturating_sub(1) \| boundary test top_row_counts_the_sends_that_wait (2, 1, 0)
- invariants: The enforcement points are Rust functions, not scripts, so no edge probe was run; the edges were traced by reading the code.
- invariants: The phase is plan, but the code of groups 1 to 3 is at head (tasks.md names the commits), so the enforcement points were read at head.
- invariants: The on_event helper of tests/app.rs now accepts any number of ListWorlds effects, so a duplicate list from one event would not fail the tests that use it; one_list_at_a_time covers the case for new worlds.
- invariants: prior.json is empty: first round.
- spec-lint: Artifacts read: proposal.md, specs/terminal-client/spec.md, design.md, tasks.md. prior.json is an empty list, so this is a first round.
- spec-lint: Checked: every requirement has a scenario; every scenario has GIVEN or WHEN and THEN; no BREAKING marker in proposal.md or the delta; design.md has no open-questions section, so assumed-answer does not apply.
- spec-lint: Build targets exist at head: mise.toml defines tasks check, test and deny; crates/mu-tui/tests/send.rs exists.
- spec-lint: Requirements 'Ctrl+C quits the client only', 'A refused text stops the sends after it', 'Alt+R joins the lines in place', 'A send with no reply is refused' and 'The top row counts the sends that wait' each have a task line that matches two or more of their words.
- spec-lint: Decisions D1 to D6 are not form classes of this lens; they were not judged against the approach.
- test-coverage: design.md was not read; matrix rests on specs and tasks.
- test-coverage: Task 4.1 runs mise run test as a whole, but no task ties it to a scenario.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: Alt+R joins the lines in place"],"If that world has no `line_break`, or no world is in focus, the client SHALL NOT change the editor."] |  |  |  |  | app-1: The modified Alt+R requirement still says that with no world in focus the client shows a `-- ` line that contains `line break`. D5 splits Alt+R into 'not known yet', 'no line break' and 'join', and names no outcome for no world in focus. The join_lines at head returns early through `?` before any hint, so Alt+R with no world in focus shows no line. The join_lines of main gave the 'sets no line break' hint in that case, so the change drops the behaviour.; inv-2: join_lines now returns early with no notice when no world is in focus. The old guard showed the line break hint there, and the requirement still demands it. |
| R-2 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Scenario: A line break that the editor cannot hold"],"gives `alpha/aria` an empty `line_break`, or one with ESC or LF"] |  |  |  |  | Scenario 'A line break that the editor cannot hold' has no task and no named test. |
| R-3 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["tasks.md","1. A send with no reply, and the texts that wait (#147)"],"`failed_send_is_refused_with_its_reason` and `answered_send_is_sent` in `crates/mu-tui/tests/send.rs`"] |  |  |  | severity? should-fix 0.72 | Scenarios 'A line that the server refuses', 'The last line is refused' and 'A refused send of Enter with a send after it' name no task or test. |
| R-4 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["tasks.md","3. A paste during the probe (#149)"],"Add `Skip::Paste`, `PASTE_START` and `PASTE_END`, and frame a paste as one event (D6)."] |  |  |  | severity? should-fix 0.72 | Scenarios 'The lines become one line', 'A terminal that adds code point widths', 'A terminal that does not answer' and 'A late answer' have no task. |
| R-5 | 1 | 1 | should-fix | deviates-design | wrong-behaviour | ["artifact",["design.md","D1. The limit of a send is in the event loop, and it gives a `Reply::Refused`"],"Alternative: a timer message in the model. It needs a clock in `update`, or a second message for each send."] |  |  |  |  | D1 hand-wraps one call in tokio::time::timeout, where connectrpc gives each call a timeout. The new ListWorlds call gets no bound, so a list with no reply leaves Listing::Asked for the session. |
| R-6 | 1 | 1 | should-fix | deviates-design | wrong-behaviour | ["path","crates/mu-tui/src/run.rs",134] |  |  |  | scope? in-change 0.64 | SEND_LIMIT equals the server's own WRITE_TIMEOUT, which D1 does not name. A stalled game write gets the server's sure refusal at the same moment the client gives up with the vague reason. |
| R-7 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/update.rs",636] |  |  |  |  | return_waiting reports a one-time event with Model::notice, which every later Reset shows again. The existing helper for a one-time line is Model::hint, which the Quit notice of this change uses. |
| R-8 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: Alt+R joins the lines in place"],"The client SHALL also ask for that list after the first event of a world that it did not hold, and after the server resets."] |  |  |  |  | One list at a time holds only while each list call ends. ListWorlds has no time limit, so a list with no reply leaves Listing::Asked for good, and no reset, new world or Alt+R asks again. |
| R-9 | 1 | 1 | should-fix | in-change | false-test | ["artifact",["specs/terminal-client/spec.md","Requirement: A send with no reply is refused"],"If the server gives no reply to a text within 10 seconds, the client SHALL count the text as refused."] |  |  |  |  | No test passes a reply on either side of SEND_LIMIT. The tests use a call that never ends and a call that is ready at once, so a limit that fires early still passes. |
| R-10 | 1 | 1 | should-fix | in-change | false-test | ["path","crates/mu-tui/src/update.rs",350] |  |  |  |  | The Quit guard is waiting == 0, and the tests pass 0 and 3. No test passes 1, the line with the server alone, so a guard that counts as the top row does still passes. |
| R-11 | 1 | 1 | should-fix | in-change | false-test | ["artifact",["specs/terminal-client/spec.md","Requirement: A refused text stops the sends after it"],"the client SHALL put each text that waits behind the text with the server back in the editor, in the order of the worlds"] |  |  |  |  | return_waiting promises the order of the worlds, but each loss and reset test holds one world. An order by the HashMap of sends passes too. |
| R-12 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. The probe framer holds a whole paste"],"A paste still open after that becomes one paste of its first part."] |  |  |  |  | At the end of the extra wait, an open paste is given as pending whole. If a read stopped inside the end mark, ESC and [20 go into the paste and 1~ arrives as keys. |
| R-13 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: The client measures character widths at startup"],"Before its first frame, the client SHALL ask the terminal for the state of mode 2027."] |  |  |  |  | No task line covers the requirement 'The client measures character widths at startup' (mode 2027 query, cursor-position queries, device attributes end, the late answer). |
| R-14 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Scenario: A terminal that does not answer"],"AND the client uses the `unicode-width` value of each grapheme"] |  |  |  |  | The THEN 'the client uses the unicode-width value of each grapheme' names no observable: no cell column, no rendered text, no returned value. |
| R-15 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","1. A send with no reply, and the texts that wait (#147)"],"Make sure that they pass."] |  |  |  |  | Tasks 1.3, 1.4, 1.5, 2.1, 3.2 and 3.3 name tests but only say 'Make sure that they pass', with no command. |
| R-16 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D2. A loss or a reset returns the texts behind the one with the server"],"Alternative: clear each whole queue. Then a late `Refused` puts the first text back after the others, out of order."] |  |  |  | severity? should-fix 0.61 | D2 rejects clearing each whole queue because a late Refused would put the first text back after the others. put_back puts the returned lines before the text of the editor, and a refusal with no queue returns only its own text. So under a cleared queue a late Refused of `one` lands before `two` and `three`, in order. The real cost of clearing, a new send that goes while the old call is still open, against the Goal of one text per world with the server, is not stated. |
| R-17 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D4. The count in the top row"],"The statusline of client-shell (#101) can take the count later."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/157; reason: Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. |  | severity? should-fix 0.80 | D4 says the #101 statusline takes the count later, but the count lives inline in view::top_row. #101 must move it or copy the 'first one is with the server' rule, which return_waiting already repeats. |
| R-18 | 1 | 1 | suggestion | in-change |  | ["path","crates/mu-tui/src/run.rs",105] |  |  |  |  | The new Effect::ListWorlds arm of run::event_loop repeats the spawn-and-reply block of Effect::Send. Only the call and the Msg variant differ. |
| R-19 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D6. The probe framer holds a whole paste"],"When the framer finds `ESC [200~`, it sets `Skip::Paste`, and each later read goes into `pending` with no limit."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/157; reason: Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. |  |  | D6 writes its own bracketed-paste framing across reads. termina's Parser, which the framer already uses, holds an open paste across parse calls and gives one Event::Paste. D6 does not name it. |
| R-20 | 1 | 1 | suggestion | in-change |  | ["path","crates/mu-tui/src/probe.rs",122] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/157; reason: Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. |  | severity? should-fix 0.71 | Skip::Paste joins an enum of bytes to drop, but it holds bytes. skipped must make it a special case, and read_answers builds a paste event a second time. |
| R-21 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","Goals / Non-Goals"],"One text per world is with the server at a time, as the requirement \\"The sends to a world go in order\\" says."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/157; reason: Design tidiness and a broader property test; none changes behaviour of this change. Owner chose to defer them to one ticket. |  |  | The send queue has an invariant over any sequence of sends, replies, losses and resets: each text is once in the queue, the editor or the server, and one Send per world is open. |
