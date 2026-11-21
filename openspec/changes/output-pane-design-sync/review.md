---
verdict: approve
phase: implementation
reviewed_at: 2026-11-21T00:07:02+00:00
head: 0cd661095cd498badd1f8039b3b16d04ebc97010
round: 5
next_id: 37
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
| 5 | implementation | follow-up | 0cd661095cd498badd1f8039b3b16d04ebc97010 | df0223b2cb6a6c49eb1870464ad57583059b0cdae090130920e24fb1be9cf243 | approve | 0 | 0 | 0 | 7 | 0 |  |

## Judge

judge: 0 questions, 0.000 USD

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

- verify: R-27: the CSI byte classes now live once, in probe::is_csi_final and probe::is_csi_body. skipped, scan and on_cut_reply_key call them, and a search of crates/mu-tui/src finds no other 0x20..=0x3F or 0x40..=0x7E range.
- verify: R-32: escape_after_alt_underscore_keeps_the_next_key checks that the key after Escape is kept. The replayed Alt+_ types nothing in the input line, so the test cannot see whether the replay happens. The claim is only about the dropped key, so this is not a defect of the fix.
- verify: R-31: with the `_` arm of on_cut_reply_key removed, the `_` falls to on_key in the Waiting state and is typed, so lone_esc_then_underscore_starts_a_graphics_reply would see more than `hi` and fail.

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
| R-27 | 4 | 5 | should-fix | in-change | drift | ["path","crates/mu-tui/src/probe.rs",270] | resolved | resolved in round 5 from fixed 3835890fe9f3b1a18476165d292602381e860926 |  |  | The ECMA-48 CSI byte classes are written out three times: in probe::scan, in probe::skipped and in the cut-reply rebuild of update. D3 wants update to rebuild a cut reply by the framer's rules, but each copy can change apart. |
| R-28 | 4 | 5 | should-fix | in-change | drift | ["path","crates/mu-tui/examples/image-pane.rs",94] | resolved | resolved in round 5 from fixed 07520f8d7e0b4893b5039674648995638b931a04 |  |  | The image-pane session joins the show error and the delete error with its own match. That match repeats screen::finish, which D4 makes the one function for a step and its cleanup. |
| R-29 | 4 | 5 | should-fix | in-change | false-test | ["path","crates/mu-tui/src/probe.rs",176] | resolved | resolved in round 5 from fixed c4d60c4cc1546ba2e79b0b1c216a86e13b9ed749 |  |  | The framer cap is tested only at PENDING_MAX + 2 bytes. No test holds exactly PENDING_MAX bytes or drops at PENDING_MAX + 1, so a guard of > PENDING_MAX + 1 passes every probe test and holds more than 4 KiB. |
| R-30 | 4 | 5 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/probe.rs",230] | resolved | resolved in round 5 from fixed c4d60c4cc1546ba2e79b0b1c216a86e13b9ed749 |  |  | The hold of a split UTF-8 character is tested only for a 2-byte character cut after its first byte. A 4-byte character cut after 3 bytes is the maximum, and a take(1) or take(2) in unfinished_tail passes utf8_split_across_reads. |
| R-31 | 4 | 5 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/update.rs",236] | resolved | resolved in round 5 from fixed c4d60c4cc1546ba2e79b0b1c216a86e13b9ed749 |  |  | After a lone ESC in a cut reply, the rule has two continuations, [ and _. Only [ is tested. With the _ arm removed, the _ and the rest of a cut kitty graphics reply go to the input line, and every update test still passes. |
| R-32 | 4 | 5 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/update.rs",203] | resolved | resolved in round 5 from fixed c4d60c4cc1546ba2e79b0b1c216a86e13b9ed749 |  |  | Late::GraphicsEscape has two sides: a \\ ends the reply, and any other key replays Alt+_ and that key. Only the \\ side is tested, so a GraphicsEscape arm that drops the other key passes every test. |
| R-33 | 4 | 4 | suggestion | in-change |  | ["artifact",["design.md","Goals / Non-Goals"],"The one exception is `view`, which records what each pane drew."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/136; reason: Design text suggestions from the implementation review of output-pane-design-sync: name the scroll-anchor reset of view in Goals, drop paste from the D3 table row, and name the vte crate among the D3 alternatives with the reason it does not fit. |  |  | Goals name one exception to the update-only rule: view records what each pane drew. At head the render of view also clears the anchor of a scrolled pane, so view changes scroll state too, and the model docs say so. |
| R-34 | 4 | 5 | suggestion | in-change |  | ["artifact",["design.md","D4. The read error wins over the restore error"],"One public function in `screen` does this, and the four places call it."] | resolved | resolved in round 5 from fixed 07520f8d7e0b4893b5039674648995638b931a04 |  | severity? should-fix 0.69 | D4 says one public function in screen keeps the first error, and names four places. The example image-pane keeps a fifth, hand-written copy of the same rule for show and the delete of the image, which D4 does not name. |
| R-35 | 4 | 4 | suggestion | in-change |  | ["artifact",["design.md","D3. The probe frames the input, and Termina decodes each CSI"],"A `termina::Parser`, for keys, text and paste."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/136; reason: Design text suggestions from the implementation review of output-pane-design-sync: name the scroll-anchor reset of view in Goals, drop paste from the D3 table row, and name the vte crate among the D3 alternatives with the reason it does not fit. |  | severity? should-fix 0.83 | The D3 table says other bytes go to Termina for keys, text and paste. The client never turns bracketed paste on, and a paste start marker ESC [ 200 ~ is a complete CSI to the framer, which Termina decodes alone to no event. The probe gives no paste event. |
| R-36 | 4 | 4 | suggestion | in-change |  | ["path","Cargo.lock",4017] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/136; reason: Design text suggestions from the implementation review of output-pane-design-sync: name the scroll-anchor reset of view in Goals, drop paste from the D3 table row, and name the vte crate among the D3 alternatives with the reason it does not fit. |  |  | The probe owns a hand framer for ECMA-48. The vte crate, a parser for that state machine, is already in Cargo.lock through vt100. D3 does not name it as an alternative or give a reason against it. |

## Reviewed files

| path | content |
|---|---|
| crates/mu-cli/tests/support/process.rs | c56f1806d6e921f393f16c2ad6b7f3514b3f8c3e |
| crates/mu-tui/examples/image-pane.rs | a237c5116282c07fdf342766e48529dbc17393aa |
| crates/mu-tui/src/model.rs | 894e88d99221619f58d27c2fb64d61fc741b7ca3 |
| crates/mu-tui/src/pane.rs | 4c64323057b82d5f4ea54521ea58a9cdcc0b17ff |
| crates/mu-tui/src/probe.rs | 3a0a9440bb869fafb2dada6e848988fd267001ae |
| crates/mu-tui/src/run.rs | 28e4957958aa67cecf78d3481a7910b8131a2efe |
| crates/mu-tui/src/screen.rs | 15d99c92f0894f95cad1494156b42afc82089b0c |
| crates/mu-tui/src/update.rs | 3930ca5e389358bf42f282c70437fd2337f2e45f |
| crates/mu-tui/tests/app.rs | 7f5be068529526371454a3f66ac247ba0681bf61 |
| crates/mu-tui/tests/pane.rs | 3581d926d31783f75f54f02f3a0fcf47044dfcd9 |
| crates/mu-tui/tests/probe.rs | ccec6b6e34f9836257fd138180cdc41996457035 |
| crates/mu-tui/tests/screen.rs | a11659cf50c65268b4cb97d6476194c8a833b4f7 |
| crates/mu-tui/tests/update.rs | 459e38b159d9413a9df481e91b7e32d12969fd6c |
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
