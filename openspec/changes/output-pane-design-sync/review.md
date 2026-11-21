---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-20T22:13:49+00:00
head: 46b47d6fe3609743a6fa53e066577631893f5000
round: 4
next_id: 37
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
| R-27 | fit | should-fix | in-change | The ECMA-48 CSI byte classes are written out three times: in probe::scan, in probe::skipped and in the cut-reply rebuild of update. D3 wants update to rebuild a cut reply by the framer's rules, but each copy can change apart. | crates/mu-tui/src/probe.rs:270 — "0x40..=0x7e => return Scan::Csi(at + 1)," | Add pub const fns in probe.rs for the CSI final byte (0x40..=0x7e) and the CSI body byte (0x20..=0x3f), and call them from scan, skipped and on_cut_reply_key. |  |  | drift |
| R-28 | fit | should-fix | in-change | The image-pane session joins the show error and the delete error with its own match. That match repeats screen::finish, which D4 makes the one function for a step and its cleanup. | crates/mu-tui/examples/image-pane.rs:94 — "format!("{shown} (the delete of the image also failed: {deleted})")," | Replace the match in session with screen::finish(shown, deleted). If the message must name the delete, give finish a label argument for the second step. |  |  | drift |
| R-29 | invariants | should-fix | in-change | The framer cap is tested only at PENDING_MAX + 2 bytes. No test holds exactly PENDING_MAX bytes or drops at PENDING_MAX + 1, so a guard of > PENDING_MAX + 1 passes every probe test and holds more than 4 KiB. | crates/mu-tui/src/probe.rs:176 — "if pending.len() > PENDING_MAX {" | Add a probe test that frames ESC _ plus PENDING_MAX - 2 bytes and asserts pending holds PENDING_MAX bytes, and one with PENDING_MAX - 1 bytes that asserts pending is empty and skip is Skip::Apc. A scratch run at head gave exactly these results. |  |  | false-test |
| R-30 | invariants | should-fix | in-change | The hold of a split UTF-8 character is tested only for a 2-byte character cut after its first byte. A 4-byte character cut after 3 bytes is the maximum, and a take(1) or take(2) in unfinished_tail passes utf8_split_across_reads. | crates/mu-tui/src/probe.rs:230 — "for (back, &byte) in run.iter().rev().enumerate().take(3) {" | Extend utf8_split_across_reads with a 4-byte character cut after 3 bytes (F0 9F 98 \| 80) and a 3-byte one cut after 2 bytes, each giving one key. A scratch run at head gave one key for both. |  |  | wrong-behaviour |
| R-31 | invariants | should-fix | in-change | After a lone ESC in a cut reply, the rule has two continuations, [ and _. Only [ is tested. With the _ arm removed, the _ and the rest of a cut kitty graphics reply go to the input line, and every update test still passes. | crates/mu-tui/src/update.rs:236 — "Some(b'_') if bytes.len() == 1 => {" | Add an update test: Msg::CutReply of a lone ESC after an incomplete outcome, then the keys _, Gi=31;OK and Alt+\\, then the keys hi. Assert that the input line is hi. |  |  | wrong-behaviour |
| R-32 | invariants | should-fix | in-change | Late::GraphicsEscape has two sides: a \\ ends the reply, and any other key replays Alt+_ and that key. Only the \\ side is tested, so a GraphicsEscape arm that drops the other key passes every test. | crates/mu-tui/src/update.rs:203 — "if text(key) == Some('\\\\') {" | Add an update test: an incomplete outcome, Alt+_, Escape, then the key x. Assert that the input line is x. |  |  | wrong-behaviour |
| R-33 | approach | suggestion | in-change | Goals name one exception to the update-only rule: view records what each pane drew. At head the render of view also clears the anchor of a scrolled pane, so view changes scroll state too, and the model docs say so. | design.md > Goals / Non-Goals — "The one exception is `view`, which records what each pane drew." | Name the second thing view changes in the Goals line: it returns a scrolled pane to the bottom when the newest row shows. |  |  |  |
| R-34 | approach | suggestion | in-change | D4 says one public function in screen keeps the first error, and names four places. The example image-pane keeps a fifth, hand-written copy of the same rule for show and the delete of the image, which D4 does not name. | design.md > D4. The read error wins over the restore error — "One public function in `screen` does this, and the four places call it." | Call screen::finish(shown, deleted) in session, or record the show and delete pair in D4 as a deviation with its own message. |  | severity? should-fix 0.69 |  |
| R-35 | approach | suggestion | in-change | The D3 table says other bytes go to Termina for keys, text and paste. The client never turns bracketed paste on, and a paste start marker ESC [ 200 ~ is a complete CSI to the framer, which Termina decodes alone to no event. The probe gives no paste event. | design.md > D3. The probe frames the input, and Termina decodes each CSI — "A `termina::Parser`, for keys, text and paste." | Drop paste from the row Other bytes, or state that a paste during the probe arrives as keys. |  | severity? should-fix 0.83 |  |
| R-36 | fit | suggestion | in-change | The probe owns a hand framer for ECMA-48. The vte crate, a parser for that state machine, is already in Cargo.lock through vt100. D3 does not name it as an alternative or give a reason against it. | Cargo.lock:4017 — "name = "vte"" | Name vte in the alternatives of D3, with the reason why it does not fit, such as the APC reply, the raw bytes Termina needs for keys, or the crate allowlist. |  |  |  |

## Repeats

None.

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-21 | suggestion | deferred | Test-helper and property-test improvements beyond this change: one private wait for wait_output and wait_screen, a property test of the D2 bound over read schedules, and a property test that every chunking of the probe input gives the same outcome. | https://github.com/seanb4t/mu-portal/issues/126 |
| R-22 | suggestion | deferred | Test-helper and property-test improvements beyond this change: one private wait for wait_output and wait_screen, a property test of the D2 bound over read schedules, and a property test that every chunking of the probe input gives the same outcome. | https://github.com/seanb4t/mu-portal/issues/126 |
| R-26 | suggestion | deferred | Test-helper and property-test improvements beyond this change: one private wait for wait_output and wait_screen, a property test of the D2 bound over read schedules, and a property test that every chunking of the probe input gives the same outcome. | https://github.com/seanb4t/mu-portal/issues/126 |
| R-33 | suggestion | deferred | Design text suggestions from the implementation review of output-pane-design-sync: name the scroll-anchor reset of view in Goals, drop paste from the D3 table row, and name the vte crate among the D3 alternatives with the reason it does not fit. | https://github.com/seanb4t/mu-portal/issues/136 |
| R-35 | suggestion | deferred | Design text suggestions from the implementation review of output-pane-design-sync: name the scroll-anchor reset of view in Goals, drop paste from the D3 table row, and name the vte crate among the D3 alternatives with the reason it does not fit. | https://github.com/seanb4t/mu-portal/issues/136 |
| R-36 | suggestion | deferred | Design text suggestions from the implementation review of output-pane-design-sync: name the scroll-anchor reset of view in Goals, drop paste from the D3 table row, and name the vte crate among the D3 alternatives with the reason it does not fit. | https://github.com/seanb4t/mu-portal/issues/136 |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 372c2e90854597c5c8287f633b24c8ff860a4a85 | e3cdae8b25f34fd3701152b8ff30ec88fbf5b40a3fc5236e05c7be0fd4fbcf06 | changes-requested | 26 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 6fcd861930db5c3d0718d9bdc9a04edafbdaa334 | fabb89e983e8c70dae4bafd96c9685d0b525c3d7604705fd56508f08ce0f49c0 | approve-with-notes | 0 | 0 | 1 | 19 | 0 |  |
| 3 | plan | follow-up | a4921572886382a6b8584a63619f2b63878839e8 | 4f9019e79495a35fd65ccc89c4a1b46e969b0f79e7ec1830b6323731fdca29a9 | approve | 0 | 0 | 0 | 1 | 0 |  |
| 4 | implementation | full | 46b47d6fe3609743a6fa53e066577631893f5000 | df0223b2cb6a6c49eb1870464ad57583059b0cdae090130920e24fb1be9cf243 | approve-with-notes | 10 | 0 | 0 | 0 | 0 |  |

## Judge

judge: 20 questions, 0.001 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|

### Not run

None.

## Verification runs

None.

## Dropped

None.

## Lens notes

- approach: The change has no spec delta (skip_specs: true), so the requirements read were the What Changes items of proposal.md. Each has a decision that produces it: D1 to D7, D8 and the doc tasks.
- approach: Every decision D1 to D10 was read against the code at head. D1 (Msg::Probe first, no frame while an early message waits, crate-private setters), D2 (TICK_TENTHS, now + TICK <= deadline, public read_answers with read and clock), D3 (framer, PENDING_MAX, Skip, Outcome::open, Msg::CutReply, decode_csi, is_set), D4 (screen::finish in probe::run, run::run, screen::open and the example), D5 (Caps from &Outcome, no cell, no window_size_failed), D6 and D7 hold.
- approach: The facts about termina 0.3.3 hold in src/parse.rs: Parser::parse, process_bytes and parse_event exist as cited, R gives the cursor report, ? ... c gives DA1 and ? ... y gives DECRPM. The stdin or /dev/tty choice of R-90 holds in src/terminal/unix.rs line 71. rustix is in the mu-tui row of ALLOWED_CRATES.
- approach: Every test that D4 and D10 name exists in tests/screen.rs, tests/probe.rs and tests/update.rs.
- approach: diff-files.txt lists files outside the Impact list of proposal.md that no decision covers: mu-cli telemetry/pipeline.rs, mu-core examples/transcript_lines.rs, mu-tui layout.rs, store.rs, text.rs, view.rs, wrap.rs, tests/store.rs, tests/data/transcript-lines.txt and deny.toml. They look like changes of other pull requests through a merge or rebase. This lens did not judge them.
- approach: The Impact list names crates/mu-tui/src/lib.rs as changed code, but the diff does not touch it.
- approach: The Context section of design.md describes the code before the change (the hand parser, the calls of set_caps in run.rs). It is the start state, not a claim about head, so it is not raised.
- fit: diff-hunks.txt includes files outside the Impact list of the proposal: crates/mu-cli/src/telemetry/pipeline.rs, crates/mu-core/examples/transcript_lines.rs, mu-tui layout.rs, store.rs, text.rs, wrap.rs, view.rs, tests/store.rs, tests/pane.rs lines 640-776 and deny.toml. They look like other work in the diff base. This lens did not judge them as components of this change.
- fit: D1 (Msg::Probe), D2 (TICK_TENTHS feeds both VTIME and TICK), D5 (From<&Outcome> for Caps, used by update and the example) and D6 (wait_rows calls wait_screen) fit the existing patterns. No copy was found.
- fit: The wait_output copy of the wait_screen block is the deferred row R-21. It is not raised again, because there is no new evidence.
- invariants: I1: The framer holds at most PENDING_MAX bytes after each frame call \| enforced at crates/mu-tui/src/probe.rs:176 \| boundary test none at PENDING_MAX or PENDING_MAX + 1 (inv-1); long_apc_is_dropped uses PENDING_MAX + 2.
- invariants: I2: A held cut reply in Late::Csi is at most PENDING_MAX bytes \| enforced at crates/mu-tui/src/update.rs:240 \| boundary test cut_reply_past_the_cap_is_dropped (held exactly PENDING_MAX, the next byte is refused).
- invariants: I3: The probe starts a read only if it can end by the deadline, so it waits at most WAIT \| enforced at crates/mu-tui/src/probe.rs:468 \| boundary test silent_terminal_keeps_the_bound (1 ms offset, 4 reads, 401 ms). The equality now + TICK == deadline is not tested; both sides keep the bound, so no finding.
- invariants: I4: The first bytes of a UTF-8 character and an ESC with one byte wait for the next read \| enforced at crates/mu-tui/src/probe.rs:226 (unfinished_tail) \| boundary test utf8_split_across_reads (2-byte only) and ss3_split_across_reads; 3- and 4-byte cases untested (inv-2).
- invariants: I5: The rest of a sequence past the cap is dropped up to its end, and an ESC that breaks it starts a new frame \| enforced at crates/mu-tui/src/probe.rs:191 (skipped) \| boundary test long_sequence_tail_is_dropped and dropped_tail_keeps_the_esc_of_the_next_frame.
- invariants: I6: Outcome::open is only an open CSI or a lone ESC, and only when DA1 did not come \| enforced at crates/mu-tui/src/probe.rs:479 and the debug_assert at crates/mu-tui/src/model.rs:138 \| boundary test cut_reply_is_open_in_the_outcome. Msg::CutReply is public, so in release only the producer in run.rs keeps the shape.
- invariants: I7: An empty cut reply is never held \| enforced at crates/mu-tui/src/run.rs:42 and crates/mu-tui/src/update.rs:133 \| boundary test none for Msg::CutReply(vec![]); the debug_assert of hold_cut_reply would panic on it in a debug build.
- invariants: I8: A cut reply is held only while the model waits for late replies \| enforced at crates/mu-tui/src/update.rs:133 \| boundary test early_keys_come_before_a_cut_reply and the cut_* tests (Waiting side only).
- invariants: I9: After a lone ESC, [ continues and _ starts the late graphics reply \| enforced at crates/mu-tui/src/update.rs:236-240 \| boundary test lone_esc_then_bracket_continues_a_cut_reply ([ only; inv-3).
- invariants: I10: The first error wins over the restore error, in four places \| enforced at crates/mu-tui/src/screen.rs:65 (finish), called from probe.rs:434, run.rs:57, screen.rs:102 and examples/image-pane.rs:52 \| boundary test all four cases in crates/mu-tui/tests/screen.rs.
- invariants: I11: Msg::Probe comes first and no frame is drawn while an early message waits \| enforced at crates/mu-tui/src/run.rs:40 and :72 \| boundary test none (the event loop needs a real terminal). Not raised again: R-23 was closed as a duplicate of the resolved R-8.
- invariants: I12: Only update changes the model, apart from view \| enforced at the pub(crate) setters in crates/mu-tui/src/model.rs:130, :206 and :229 \| boundary test the Msg::Probe tests of tests/update.rs; code in the crate is checked by review only, as D1 says.
- invariants: I13: Caps comes from the outcome in one place \| enforced at crates/mu-tui/src/pane.rs:68 (From<&Outcome>) \| boundary test caps_come_from_the_outcome and probe_sets_the_caps_of_every_pane.
- invariants: I14: A setting is set only for Set and PermanentlySet \| enforced at crates/mu-tui/src/probe.rs:393 (is_set) \| boundary test only_set_and_permanently_set_are_set covers all five values.
- invariants: I15: Width profile is measured only with DA1 and exactly one cursor answer per grapheme \| enforced at crates/mu-tui/src/probe.rs:374 \| boundary test N and N+1 answers; N-1 is untested, but the line is unchanged by this change, so no finding.
- invariants: Probe run at head (scratch crate under the handoff directory, inv-probe): 4-byte UTF-8 split 3+1 and 3-byte split 2+1 each give one key; ESC _ plus 4094 bytes stays pending at 4096 bytes; plus 4095 bytes is dropped with skip Apc.
- invariants: Probe run at head: Alt+O, Alt+] or Alt+P directly before a reply in the same read gives the reply and no key; ESC ] abc gives no event. The framer passes these bytes to Termina with no more input, which drops them, as Termina's own reader does. The replies survive, so no finding.
- invariants: diff-files.txt also lists files that origin/main changed after the local main (telemetry/pipeline.rs, deny.toml, text.rs, layout.rs, store.rs, view.rs, wrap.rs: commits #125, #129, #130). They are not part of this change and were not reviewed for invariants.
- invariants: Not raised again: R-22 and R-26 (property tests of the bound and of chunking) are deferred and closed, and there is no new evidence. The repository has proptest, used in crates/mu-tui/tests/store.rs.
- test-coverage: The change has no spec delta (skip_specs: true, no specs directory), so the matrix has no rows.
- test-coverage: No BREAKING requirement exists in proposal.md, so no breaking-unproven finding applies.
- test-coverage: Test-to-task mapping per task was not verified against the test bodies; orphan-test and tests-the-library were not assessed.
- test-honesty: Read the new and changed tests in mu-tui tests probe, update, screen, app, pane, store and the wait_rows helper in mu-cli. No ignored tests, no assertion-free tests, no tautologies found.
- test-honesty: caps_come_from_the_outcome and complete_probe_does_not_wait_for_late_replies are coarse but touch their scenario.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D4. An ESC ends an open sequence"],"If not, the split of D3 cuts the input at that ESC before Termina sees it."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | app-1: Neither branch of D4 makes a reply after Alt+[ count (QUAL-3). termina::Parser holds a bare ESC [ in its buffer, and when the next ESC arrives it drops ESC [ ESC, so the reply loses its ESC. Cutting the input before Termina does not help if the cut-off ESC [ still goes to the same parser.; inv-1: Neither branch of D4 makes an ESC end an open CSI with termina 0.3.3. A cut at the ESC fails too, because the parser keeps a bare ESC [ in its buffer across parse calls even with maybe_more false. A probe gave: ESC [ then ESC [1;1R yields the keys [ 1 ; 1 R, the cursor reply is lost, and a lone ESC [ yields no event, so Alt+[ never reaches the input line. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["path","crates/mu-tui/tests/probe.rs",3] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | app-3: D3 removes the public probe::Reply and the hand parser. But tests/probe.rs imports Reply and parse, and builds Outcome::learn inputs from Reply in 15 tests. No decision or task says where those tests go, or whether the public Outcome::learn becomes private. A pub fn that takes a private type breaks the deny-warnings lints.; inv-6: The edge tests of Outcome::learn and of the kitty reply are in tests/probe.rs: an extra cursor reply, no DA1, mode values 1, 2 and 3, a graphics error, and a graphics reply of another id. They import the public parse and Reply, which task 2.3 removes, and no task moves them. The count rule that the Risks section relies on (Shift+F3 adds a cursor reply) would then have no test. |
| R-3 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D8. The snapshot test also asserts its cells"],"The top row starts with `[alpha/aria]` and then `beta/bryn`, and the rows between show `Welcome, aria.`."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  | severity? should-fix 0.63 | R-86 asks the snapshot test to assert the cells that its scenario names. The scenario The first view also names that the screen shows no line of beta/bryn, and two_worlds sends Welcome, bryn. D8 and task 1.2 assert only the top row and Welcome, aria. |
| R-4 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. An ESC ends an open sequence"],"A reply that follows it starts with ESC, so the reply still counts, and Alt+`[` reaches the input line (QUAL-3)."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | D4 says Alt+[ reaches the input line. Termina gives no event for a bare ESC [: parse_csi returns Ok(None) at length 2 whatever maybe_more is. Under either branch of D4, the Alt+[ key is lost, not delivered. |
| R-5 | 1 | 2 | should-fix | in-change | next-change | ["artifact",["design.md","D6. `Caps` holds what the pane reads"],"Enforced by: the compiler. A field that nothing reads gives a dead code warning, and the lints deny warnings."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | D6 says the compiler enforces it, because an unread field gives a dead code warning. Caps is a pub struct, re-exported from the crate root, with pub fields. rustc gives no dead_code warning for a field like that, so an unread field passes the lints. |
| R-6 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["proposal.md","Impact"],"Tests: `crates/mu-tui/tests/app.rs` and `crates/mu-cli/tests/support/process.rs`."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | Task 4.2 makes the setters that only update calls private to the crate. But tests/update.rs calls await_late_replies and notice from outside the crate, in the late-reply and notice tests. The Impact list names only app.rs and process.rs, and no decision says how those tests reach the late state through Msg::Probe. |
| R-7 | 1 | 2 | should-fix | in-change | next-change | ["path","crates/mu-tui/tests/pane.rs",281] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  | severity? blocker 0.71 | D6 removes Caps.cell, and task 4.3 edits only run.rs. tests/pane.rs builds Caps with cell: None three times, and the example image-pane builds Caps with cell. Neither is in Impact or in task 4.3, so the build of tests and examples fails. |
| R-8 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","Context"],"which feeds it to a `termina::Parser` and gives the events to `update` before the first frame"] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | Context says run.rs gives the early events to update before the first frame. At head, event_loop draws before it takes each early event, so the first frame comes first. D1 wants Msg::Probe, then the early events, then the first frame, but task 4.2 has no step that moves the draw. If Msg::Probe goes into the early queue, the first frame has no caps and no notice, where head sets them before that frame. |
| R-9 | 1 | 2 | should-fix | in-change | next-change | ["artifact",["tasks.md","2. The probe parses with Termina"],"Write the unit tests of the probe input in `crates/mu-tui/src/probe.rs`"] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | Tasks 2.2, 3.1 and 3.3 put the new probe tests in src/probe.rs. Every mu-tui test lives in crates/mu-tui/tests/, and tests/probe.rs already tests the probe through parse and Reply, which D3 removes. No task names tests/probe.rs, so the check of task 2.3 fails on it. |
| R-10 | 1 | 2 | should-fix | in-change | next-change | ["artifact",["tasks.md","4. The probe result is a message"],"Write the unit tests of `update` for `Msg::Probe` in `crates/mu-tui/src/update.rs` or `crates/mu-tui/tests/app.rs`"] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | Task 4.1 puts the Msg::Probe tests in src/update.rs or tests/app.rs, but tests/update.rs is the home of the update tests for terminal replies. Its late-reply and notice tests call await_late_replies and notice, which task 4.2 makes crate-private, and no task rewrites them. |
| R-11 | 1 | 2 | should-fix | in-change | drift | ["path","crates/mu-tui/src/probe.rs",214] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | After D3 the probe gets DECRPM as Termina's DecModeSetting, as update already does, but Outcome keeps mode 2027 as a raw u16 and tests it with Some(1 \| 3). The rule 'mode reported as set' then has two encodings and two copies. |
| R-12 | 1 | 2 | should-fix | in-change | drift | ["artifact",["proposal.md","What Changes"],"`Caps` stops repeating the probe outcome."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | The proposal says Caps stops repeating the probe outcome, but after D6 Caps still holds the profile and the graphics flag of Outcome. The Outcome-to-Caps mapping is copied in run.rs, which D1 moves into update, and in the image-pane example. |
| R-13 | 1 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. The read error wins over the restore error"],"Its message holds both errors."] | resolved | resolved in round 3 from fixed ecacbba87c9a14cce2d2fe93f40cc5a74bc9fe53 |  |  | D5 adds a rule that keeps the restore error in the message of the read error. run.rs and screen.rs handle the same pair, an error and the error of the terminal restore, by dropping the second. D5 names neither, so the client gets two rules for one case. |
| R-14 | 1 | 2 | should-fix | in-change | drift | ["path","crates/mu-tui/src/probe.rs",241] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | D2 needs the tick of 100 milliseconds as a Duration in the loop condition, while the probe sets the same tick as VTIME 1. The design states no single source, so the bound can drift from the read if one of them changes. |
| R-15 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. An ESC ends an open sequence"],"In the input of the probe, an ESC that comes before the end of an open CSI or APC ends that sequence."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | D4 covers an open CSI, but the planned tests pass only a bare ESC [ and an open APC. An open CSI with a ? prefix takes another path in termina: it keeps reading up to the next u, c, n or y. A probe of ESC [ ? before a cursor reply and DA1 gave DA1 alone, so the cursor reply was swallowed. |
| R-16 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Termina parses the CSI replies, and the probe parses only the APC reply"],"An APC, from ESC `_` to ESC `\\\\`, goes to the parser of the kitty graphics reply in the probe."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | The APC ends with ESC \\, so its own terminator starts with the ESC that D4 says ends an open APC. No planned test puts a read boundary between that ESC and the backslash. A probe that reads a trailing ESC as a new sequence loses the kitty reply and turns graphics off. |
| R-17 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["tasks.md","3. The bound of the probe and its errors"],"In it, a fake read adds one tick to a fake clock and returns no bytes."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | As task 3.1 states it, the fake read adds exactly one tick from the deadline base. 500 is a multiple of 100, so the current condition (now < deadline) returns at exactly WAIT, which is not past the bound. The test passes before the fix, and the task's fail-first check cannot hold. A simulation gave 500 for both conditions at offset 0. At offset 1 ms it gave old 501 and new 401. |
| R-18 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. The read error wins over the restore error"],"If only one fails, the probe returns that one."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | D5 has four cases. The planned tests cover both-fail, restore-alone and both-ok, and leave out read-alone, where the read fails and the restore succeeds. A combining function that returns the restore result first passes all three planned tests and drops the read error. |
| R-19 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"The edits are in different lines. The second pull request to merge rebases on the first."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  | severity? should-fix 0.69 | The risk of a conflict with #121 in run.rs has as its mitigation a rebase of the later pull request. No task or check names that step. |
| R-20 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D6. `Caps` holds what the pane reads"],"When #13 sizes an image from the cell, it adds the field back with a reader."] | resolved | resolved in round 2 from fixed 21c94fc351164eaac813eceb3be70eb5c60e0e01 |  |  | D6 removes Caps.cell and the screen::cell call in run.rs, and itself says that #13 adds the field back. #13 must then restore both, and after D1 also carry the cell size into the Msg::Probe arm of update. |
| R-21 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D7. The test helper has one wait on the screen"],"`wait_rows` calls `wait_screen` with a check on the rows of the screen."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/126; reason: Test-helper and property-test improvements beyond this change: one private wait for wait_output and wait_screen, a property test of the D2 bound over read schedules, and a property test that every chunking of the probe input gives the same outcome. |  |  | D7 folds wait_rows into wait_screen, but wait_output keeps the same block of subscribe, ScreenNote, wait_until and wait_for on another watch channel. The two vary only in the channel and the check. |
| R-22 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D2. The probe keeps its bound with a read that cannot end past the deadline"],"So the last read ends by the deadline, and a silent terminal costs at most 500 milliseconds."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/126; reason: Test-helper and property-test improvements beyond this change: one private wait for wait_output and wait_screen, a property test of the D2 bound over read schedules, and a property test that every chunking of the probe input gives the same outcome. |  |  | The D2 bound must hold for any sequence of read durations up to one tick and any start offset, which is an unbounded domain. A single fake-clock example checks one schedule. |
| R-23 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D1. The probe result is a `Msg`"],"`run.rs` gives it to `update` first, then the early input events, then it draws the first frame."] | duplicate | R-8 |  | severity? should-fix 0.74 | D1 has the first frame drawn after the early events, but event_loop draws before it takes each message, and no task changes that order. No test checks the order. |
| R-24 | 1 | 1 | suggestion | in-change |  | ["artifact",["tasks.md","4. The probe result is a message"],"Make each setter that only `update` calls private to the crate."] | duplicate | R-6 |  | severity? blocker 0.63 | Nothing but review enforces the rule that only update changes the model. Task 4.2 makes the setters crate-private, but run.rs is in the same crate and can still call them. tests/update.rs, outside the crate, calls await_late_replies and notice to set up the Late tests, so it no longer builds. Either the setters stay pub, or those tests change. |
| R-25 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D6. `Caps` holds what the pane reads"],"A field that nothing reads gives a dead code warning, and the lints deny warnings."] | duplicate | R-5 |  | severity? should-fix 0.72 | D6 names the compiler as the enforcement, but Caps is pub, lib.rs re-exports it, and its fields are pub. The dead_code lint does not fire on an unread pub field of an exported type. The lint check of task 4.3 would pass with an unread field. |
| R-26 | 1 | 1 | suggestion | in-change |  | ["artifact",["tasks.md","2. The probe parses with Termina"],"`alt_bracket_before_a_reply` (QUAL-3), `open_apc_does_not_hide_da1` (SEC-6) and `split_reply_across_reads`"] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/126; reason: Test-helper and property-test improvements beyond this change: one private wait for wait_output and wait_screen, a property test of the D2 bound over read schedules, and a property test that every chunking of the probe input gives the same outcome. |  |  | The probe's split of its input must not depend on where reads cut it, and the domain of split points and interleaved keys is unbounded. One example test, split_reply_across_reads, checks a single split. A property would check that every chunking of the reply stream, with keys between replies, gives the same Outcome and the same early events as one read. |
| R-27 | 4 | 4 | should-fix | in-change | drift | ["path","crates/mu-tui/src/probe.rs",270] | fixed | 3835890fe9f3b1a18476165d292602381e860926 |  |  | The ECMA-48 CSI byte classes are written out three times: in probe::scan, in probe::skipped and in the cut-reply rebuild of update. D3 wants update to rebuild a cut reply by the framer's rules, but each copy can change apart. |
| R-28 | 4 | 4 | should-fix | in-change | drift | ["path","crates/mu-tui/examples/image-pane.rs",94] | fixed | 07520f8d7e0b4893b5039674648995638b931a04 |  |  | The image-pane session joins the show error and the delete error with its own match. That match repeats screen::finish, which D4 makes the one function for a step and its cleanup. |
| R-29 | 4 | 4 | should-fix | in-change | false-test | ["path","crates/mu-tui/src/probe.rs",176] | fixed | c4d60c4cc1546ba2e79b0b1c216a86e13b9ed749 |  |  | The framer cap is tested only at PENDING_MAX + 2 bytes. No test holds exactly PENDING_MAX bytes or drops at PENDING_MAX + 1, so a guard of > PENDING_MAX + 1 passes every probe test and holds more than 4 KiB. |
| R-30 | 4 | 4 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/probe.rs",230] | fixed | c4d60c4cc1546ba2e79b0b1c216a86e13b9ed749 |  |  | The hold of a split UTF-8 character is tested only for a 2-byte character cut after its first byte. A 4-byte character cut after 3 bytes is the maximum, and a take(1) or take(2) in unfinished_tail passes utf8_split_across_reads. |
| R-31 | 4 | 4 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/update.rs",236] | fixed | c4d60c4cc1546ba2e79b0b1c216a86e13b9ed749 |  |  | After a lone ESC in a cut reply, the rule has two continuations, [ and _. Only [ is tested. With the _ arm removed, the _ and the rest of a cut kitty graphics reply go to the input line, and every update test still passes. |
| R-32 | 4 | 4 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/update.rs",203] | fixed | c4d60c4cc1546ba2e79b0b1c216a86e13b9ed749 |  |  | Late::GraphicsEscape has two sides: a \\ ends the reply, and any other key replays Alt+_ and that key. Only the \\ side is tested, so a GraphicsEscape arm that drops the other key passes every test. |
| R-33 | 4 | 4 | suggestion | in-change |  | ["artifact",["design.md","Goals / Non-Goals"],"The one exception is `view`, which records what each pane drew."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/136; reason: Design text suggestions from the implementation review of output-pane-design-sync: name the scroll-anchor reset of view in Goals, drop paste from the D3 table row, and name the vte crate among the D3 alternatives with the reason it does not fit. |  |  | Goals name one exception to the update-only rule: view records what each pane drew. At head the render of view also clears the anchor of a scrolled pane, so view changes scroll state too, and the model docs say so. |
| R-34 | 4 | 4 | suggestion | in-change |  | ["artifact",["design.md","D4. The read error wins over the restore error"],"One public function in `screen` does this, and the four places call it."] | fixed | 07520f8d7e0b4893b5039674648995638b931a04 |  | severity? should-fix 0.69 | D4 says one public function in screen keeps the first error, and names four places. The example image-pane keeps a fifth, hand-written copy of the same rule for show and the delete of the image, which D4 does not name. |
| R-35 | 4 | 4 | suggestion | in-change |  | ["artifact",["design.md","D3. The probe frames the input, and Termina decodes each CSI"],"A `termina::Parser`, for keys, text and paste."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/136; reason: Design text suggestions from the implementation review of output-pane-design-sync: name the scroll-anchor reset of view in Goals, drop paste from the D3 table row, and name the vte crate among the D3 alternatives with the reason it does not fit. |  | severity? should-fix 0.83 | The D3 table says other bytes go to Termina for keys, text and paste. The client never turns bracketed paste on, and a paste start marker ESC [ 200 ~ is a complete CSI to the framer, which Termina decodes alone to no event. The probe gives no paste event. |
| R-36 | 4 | 4 | suggestion | in-change |  | ["path","Cargo.lock",4017] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/136; reason: Design text suggestions from the implementation review of output-pane-design-sync: name the scroll-anchor reset of view in Goals, drop paste from the D3 table row, and name the vte crate among the D3 alternatives with the reason it does not fit. |  |  | The probe owns a hand framer for ECMA-48. The vte crate, a parser for that state machine, is already in Cargo.lock through vt100. D3 does not name it as an alternative or give a reason against it. |

## Reviewed files

| path | content |
|---|---|
| crates/mu-cli/tests/support/process.rs | c56f1806d6e921f393f16c2ad6b7f3514b3f8c3e |
| crates/mu-tui/examples/image-pane.rs | c872e4507f832504b4ee62fb2dc1c1382d9302c1 |
| crates/mu-tui/src/model.rs | 894e88d99221619f58d27c2fb64d61fc741b7ca3 |
| crates/mu-tui/src/pane.rs | 4c64323057b82d5f4ea54521ea58a9cdcc0b17ff |
| crates/mu-tui/src/probe.rs | acbcf1657254ea725f2e885a316c840c83a33695 |
| crates/mu-tui/src/run.rs | 28e4957958aa67cecf78d3481a7910b8131a2efe |
| crates/mu-tui/src/screen.rs | 15d99c92f0894f95cad1494156b42afc82089b0c |
| crates/mu-tui/src/update.rs | 19302300586acde6212bcb40dca24c978833f1e3 |
| crates/mu-tui/tests/app.rs | 7f5be068529526371454a3f66ac247ba0681bf61 |
| crates/mu-tui/tests/pane.rs | 3581d926d31783f75f54f02f3a0fcf47044dfcd9 |
| crates/mu-tui/tests/probe.rs | 71394809957c04e44692e5ab26488978c1c2009d |
| crates/mu-tui/tests/screen.rs | a11659cf50c65268b4cb97d6476194c8a833b4f7 |
| crates/mu-tui/tests/update.rs | ec9573e86f9394ec64aca72becfb1a4cf91bd3e3 |
| docs/src/content/docs/decisions/character-width.md | e55713ae1814b1347781c6763667f63d884a0ffb |
| docs/src/content/docs/decisions/terminal-client-architecture.md | 960bea0ec167a4df56b9b8f6c2b90b4e1e49c9b8 |
| openspec/changes/output-pane-design-sync/.openspec.yaml | 0b406914846b8938e674ec056eaad38ce274cb5f |
| openspec/changes/output-pane-design-sync/design.md | f6a86dc2b1b440be9a6ae93eb368fb9d07599ebb |
| openspec/changes/output-pane-design-sync/proposal.md | a1df8b9f0f53bf95ff29a5e52eef3f00d885eb08 |
| openspec/changes/output-pane-design-sync/tasks.md | sha256:08442cf1077de194d1d03f6eee77c862086d4258e6b2b78df8831ae06ffff90a |
| openspec/review.json | absent |

## Plan review (2026-11-20)

Verdict approve at head a4921572886382a6b8584a63619f2b63878839e8.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|

### Not run

Plan phase: no verification runs.
