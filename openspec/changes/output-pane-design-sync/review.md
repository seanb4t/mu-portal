---
verdict: approve-with-notes
phase: plan
reviewed_at: 2026-11-20T14:28:59+00:00
head: 6fcd861930db5c3d0718d9bdc9a04edafbdaa334
round: 2
next_id: 27
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
| R-13 | verify | should-fix | in-change | PARTIAL: the fix 21c94fc351164eaac813eceb3be70eb5c60e0e01 of R-13 did not hold. D5 adds a rule that keeps the restore error in the message of the read error. run.rs and screen.rs handle the same pair, an error and the error of the terminal restore, by dropping the second. D5 names neither, so the client gets two rules for one case. | crates/mu-tui/src/screen.rs:80 — "let _ = leave(out, false);" | fix it again | regression |  | wrong-behaviour |

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

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 372c2e90854597c5c8287f633b24c8ff860a4a85 | e3cdae8b25f34fd3701152b8ff30ec88fbf5b40a3fc5236e05c7be0fd4fbcf06 | changes-requested | 26 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 6fcd861930db5c3d0718d9bdc9a04edafbdaa334 | fabb89e983e8c70dae4bafd96c9685d0b525c3d7604705fd56508f08ce0f49c0 | approve-with-notes | 0 | 0 | 1 | 19 | 0 |  |

## Judge

judge: 2 questions, 0.000 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|

### Not run

Plan phase: no verification runs.

## Verification runs

None.

## Dropped

None.

## Lens notes

- verify: Task 2.2 adds open_apc_does_not_hide_da1, which D10 says runs with the fake clock, and task 2.3 requires the tests of 2.2 to pass. But the read loop that takes a clock comes only in task 3.1. Task 3.2 checks the test again, so the plan may mean the clock part to wait until then. Saying so in task 2.2 would make the order clear.
- verify: D3 says an ESC or an open sequence at the end of a read waits for the next read, but it gives no rule for bytes still pending when the probe ends at DA1 or at the deadline, now that run.rs makes no parser. The code at head passes the pending bytes to the run.rs parser, and a bare ESC [ is lost there too, so this is no regression.
- verify: tests/update.rs sets the notice to the text note and counts rows that read -- note on a 20-column screen (lines 141-179). Through Msg::Probe the notice becomes WIDTH_NOTICE, so task 4.1 has to change the assertion and the screen width, not just the setup.
- verify: Task 3.3 names no file for the four tests of the D4 function. If they live in crates/mu-tui/tests/, as every other mu-tui test does, the function in screen must be public.

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
| R-13 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. The read error wins over the restore error"],"Its message holds both errors."] | fixed | ecacbba87c9a14cce2d2fe93f40cc5a74bc9fe53 |  |  | D5 adds a rule that keeps the restore error in the message of the read error. run.rs and screen.rs handle the same pair, an error and the error of the terminal restore, by dropping the second. D5 names neither, so the client gets two rules for one case. |
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
