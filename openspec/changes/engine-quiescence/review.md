---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-16T22:31:08+00:00
head: b8b5da4e7e18c9adf404c0e2787029c1ca5b165c
round: 5
next_id: 53
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: ok
  invariants: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-48 | fit,invariants | should-fix | in-change | inv-1: The new search of task 4.1 still lets a second home of the telnet codes pass. A probe under the handoff directory (probe-inv/src) flagged `const IAC: u8 = 0xFF;` and `const TM: u8 = 0x06;`. It missed `b"\\xff\\xfd\\x06"`, `b'\\xfd'` and `u8::MAX`, which are common Rust forms for the bytes of a mark.; fit-1: The rule that the telnet codes live only in telnet.rs is enforced by a regex run once in task 4.1. The repository's pattern for a source-wide rule is a durable test that reads every Rust file of the members (workspace_boundary.rs), with probe cases for each edge form. The design names neither that pattern nor a reason. | tasks.md > 4. No sleep in a test — "The pattern finds the codes 251 to 255 in decimal, hex, suffixed and array forms, and a `u8` constant of 6." | inv-1: Add byte-escape and MAX forms to the 4.1 pattern, for example `\\\\x0*(f[b-f]\|06)\\b\|u8::MAX`, and name them in the enforcement row. Then run it again on the probe files under probe-inv/src.; fit-1: Enforce the rule with a test in the shape of workspace_boundary.rs: walk crates/mu-core/src with support::files_under, flag a u8 literal token 251 to 255 outside telnet.rs, and add probe cases for the forms of task 4.1. Name that test in the Enforcement row. | regression |  |
| R-49 | fit | should-fix | deviates-design | The rule that no test of mu-core sleeps is enforced by an rg run once in task 4.1, so a sleep added after the change passes every later check. The repository's pattern for a source-wide rule is a durable test over the Rust files (workspace_boundary.rs); the design names neither that pattern nor a reason. | tasks.md > 4. No sleep in a test — "Make sure that `rg -n 'settle' crates/` and `rg -n -i 'sleep' crates/mu-core/tests crates/mu-core/src` find nothing."; scope basis: design.md > Enforcement of each rule — "`review-lint`, gate `time-based-test`, sees only the added lines of a diff." | Add a test beside the one of fit-1 that fails on a sleep token in crates/mu-core/src and crates/mu-core/tests, and name it in the Enforcement row in place of the task-time search. |  | scope? in-change 0.81 |
| R-50 | fit | should-fix | in-change | The answer of a frame (send Ok(None) and ignore a gone caller) is written twice in session.rs: in WaitingFrames::answer_first and in the Down branch of on_frame, with the same comment. | crates/mu-core/src/session.rs:317 — "let _ = frame.send(Ok(None));" | Add one private fn answer(frame) in session.rs that sends Ok(None), and call it from answer_first and from the Down branch of on_frame. |  | severity? suggestion 0.81 |
| R-51 | invariants | should-fix | in-change | The Reply branch of Session::on_bytes now goes through write_bytes, which flushes, calls down on a failed write and stops the rest of the read. No test gives the session an Inbound::Reply, so the success path and the failed-write edge (one disconnect event, waiting frames answered) are not exercised on this path. LineFramer::new never gives Reply. | crates/mu-core/src/session.rs:388 — "if self.write_bytes(&bytes, 0).await.is_err() {" | In tests/scripts.rs, add a test framer like Shout that gives Inbound::Reply for the game line `ask`. Assert that the game gets the reply bytes. Then, on a stream that refuses writes after the read, assert one Disconnected event and an `Ok` from try_idle. |  |  |
| R-52 | invariants | suggestion | in-change | The rule that an escaped IAC is one data byte and never starts a mark holds over any byte stream. The transparency property excludes the byte 255 from its stream, so marks next to escaped 255 bytes at random positions and splits are tried only by the fixed cases of line_framer_reads_escaped_iac. | design.md > The telnet bytes of the mark live in one place — "The escaped byte reaches the line as one byte 255, not as two." | With proptest, in framer.rs line_framer_marks_are_transparent: allow any byte in the stream and write each 255 as IAC IAC in the fed bytes. Build the expected lines by splitting the raw stream on LF, not with LineFramer::new. |  |  |

## Repeats

None.

## Out of bound

| lens | severity | claim | evidence |
|---|---|---|---|
| fit | should-fix | wait_covers_server_handler_loop builds the same echo_custom handler, portal set-up and emit as handler_loop_stops_at_depth_limit, while scripts.rs already shares handler set-ups through helpers such as again_loop and recorder. | crates/mu-core/tests/scripts.rs:475 — "let echo_custom = on(\|event\| {" |

## Accepted

None.

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | f269a3bfa5f76277baba39eccabf36fe65076958 | d37fc84f5010c6f18872b6a56179c3a647c28ab0c83b5f03469b3eb001e0b747 | changes-requested | 23 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | d28a104623ce37f052a9d455c79e0b571fbf5959 | 4b7f0a5225a2decbbbac73dcf349088cc940277b1203e2e238bb4a5bd1d70481 | approve-with-notes | 10 | 0 | 2 | 0 |  |
| 3 | plan | follow-up | 91366746399a17bc7fafd97b9e41153f2e573d87 | d9512329a6f73b992cedb465144fcaf27f3e82f2d19a1a322d85ddc92e2aa82e | approve-with-notes | 8 | 0 | 2 | 0 |  |
| 4 | implementation | full | fd138e482a5bf2715e92afec1547817f29c87ccd | 3ad7f6b96bbf7a7b8783e61e8746e568d1f262636268ad6a16de36b99a2184e7 | approve-with-notes | 7 | 0 | 2 | 0 |  |
| 5 | implementation | follow-up | b8b5da4e7e18c9adf404c0e2787029c1ca5b165c | 02662d01b9ba559f746008959f07dac3dfc3010e27ff46eb94f4fa55c63fe01c | approve-with-notes | 4 | 0 | 1 | 1 |  |

## Judge

judge: 10 questions, 0.000 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A wait covers the echo of the game | crates/mu-core/tests/portal.rs: wait_covers_game_echo | covered | pass |
| A wait on a world that is down | crates/mu-core/tests/portal.rs: wait_on_world_that_is_down | covered | pass |
| A wait outlasts a connect | crates/mu-core/tests/portal.rs: wait_outlasts_connect | covered | pass |
| The line framer reads the answer to a mark | crates/mu-core/src/framer.rs: line_framer_reads_mark_answer | covered | pass |
| A wait survives a drop | crates/mu-core/tests/portal.rs: wait_survives_drop | covered | pass |
| A wait survives a failed connect | crates/mu-core/tests/portal.rs: wait_survives_failed_connect | covered | pass |
| A mark that no frame waits for | crates/mu-core/tests/scripts.rs: stray_mark_is_ignored | covered | pass |
| A wait covers a loop of server handlers | crates/mu-core/tests/scripts.rs: wait_covers_server_handler_loop | covered | pass |
| A wait adds nothing | crates/mu-core/tests/portal.rs: wait_adds_nothing | covered | pass |
| The fake world answers | crates/mu-cli/tests/worlds.rs: fake_world_answers | covered | pass |
| The fake world answers a mark in order | crates/mu-core/src/fake.rs: fake_game_answers_mark_in_order | covered | pass |
| The flag is hidden | crates/mu-cli/tests/worlds.rs: fake_world_flag_is_hidden | covered | pass |
| A missing password variable | crates/mu-cli/tests/worlds.rs: missing_password_variable | covered | pass |

### Not run

None.

## Verification runs

- exit 0: `cargo nextest run -p mu-core --test portal --locked -E test(=wait_covers_game_echo)`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E test(=wait_on_world_that_is_down)`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E test(=wait_outlasts_connect)`
- exit 0: `cargo nextest run -p mu-core --lib --locked -E test(=framer::tests::line_framer_reads_mark_answer)`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E test(=wait_survives_drop)`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E test(=wait_survives_failed_connect)`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E test(=stray_mark_is_ignored)`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E test(=wait_covers_server_handler_loop)`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E test(=wait_adds_nothing)`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E test(=fake_world_answers)`
- exit 0: `cargo nextest run -p mu-core --lib --locked -E test(=fake::tests::fake_game_answers_mark_in_order)`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E test(=fake_world_flag_is_hidden)`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E test(=missing_password_variable)`

## Dropped

None.

## Lens notes

- approach: Follow-up round since fd138e4. Every file of diff-files.txt was read: framer.rs, fake.rs, session.rs, portal.rs, tests/portal.rs, tests/scripts.rs, tests/support/mod.rs, design.md and tasks.md. review.md was not read.
- approach: Without the Bash tool, the changed lines since the base cannot be listed exactly. The bound was judged per file and per section, with the fix rows R-42 to R-48 and the IAC IAC commit as the guide to the changed text.
- approach: The IAC IAC paragraph of 'The telnet bytes of the mark live in one place' matches framer.rs: IAC IAC gives one data byte 255 on both sides, a mark is read only after that, and given-back bytes go through push and its MAX_LINE cut.
- approach: The RFC 854 row for the doubled IAC holds: the first paragraph of TELNET COMMAND STRUCTURE says only the IAC need be doubled to be sent as data.
- approach: The task 4.1 search for the codes, run with the same pattern and the -i flag, finds only crates/mu-core/src/secret.rs:227, which is inside the tests module of secret.rs. The searches for settle in crates/ and for sleep in mu-core find nothing.
- approach: The shared ask method, to_every_world over fn(WorldId, Sender) and the Reply branch through write_bytes match the decision 'The order of a round' and the R-42 fix. wait_idle also fails with Stopped when the server handler task ends. Its doc comment says so, and the design does not forbid it.
- approach: Out of bound, not reported: the requirement 'A control frame comes after the work before it' says the session SHALL send IAC DO TIMING-MARK when the world is up. The design and write_mark answer at once for a framer whose mark gives no bytes (session.rs:306), and the spec has no such exception. spec.md did not change since the base.
- approach: Out of bound for this lens: the doc comment of LineFramer (framer.rs:88-90) says it gives the mark as Inbound::Mark and gives no other message. The game side gives Inbound::Reply, as the design decides. The doc text lags the decision. Code behavior does not.
- fit: The library rows were read from openspec/config.yaml.
- fit: This lens has no git access, so it cannot list the lines changed since fd138e4. The bound placement of fit-3 and fit-4 is a best guess, and the merge decides it.
- fit: The fixes of R-42 to R-47 hold at head: on_bytes writes a Reply through write_bytes, open calls open_raw, one cases loop serves fake_game_answers_mark_in_order, broken_world shares the set-up, on() lives in tests/support, and serve encodes through framer.encode.
- invariants: Follow-up round since fd138e48: read the diffs of framer.rs, session.rs, portal.rs, fake.rs, tests/portal.rs, tests/scripts.rs, tests/support/mod.rs, design.md and tasks.md.
- invariants: I1: IAC IAC is one data byte 255, read before any mark \| enforced at crates/mu-core/src/framer.rs:117 (Held::Iac, IAC) \| boundary test line_framer_reads_escaped_iac (split after every byte, odd run IAC IAC IAC)
- invariants: I2: No line is longer than MAX_LINE, also when an escaped IAC lands on the cut \| enforced at framer.rs:146 push \| boundary test property line_framer_lines_stay_in_the_bound (prefix MAX_LINE-4, favored tail crossing the cut)
- invariants: I3: The framer holds at most two bytes of a mark or one IAC across reads \| enforced by the Held enum, framer.rs:81 \| boundary test line_framer_ignores_read_splits and every-byte splits
- invariants: I4: The count_marks model matches the framer: verb patterns tried before IAC IAC, and IAC is never a verb \| enforced at framer.rs:268-279 \| checked by hand on IAC WILL IAC IAC, IAC IAC IAC WILL TM, IAC DO TM (client): data counts agree
- invariants: I5: A frame is answered only with Ok(None) \| enforced at session.rs:70 and session.rs:317 \| boundary test waiting_frames_answer_in_order (empty-list answer_first too)
- invariants: I6: A frame whose channel closes with no answer makes wait_idle fail with Stopped \| enforced at portal.rs ask, answer.await.map_err \| boundary test none that drops a frame (the wait_survives_* tests assert Ok)
- invariants: I7: A failed write of a reply ends the connection and answers waiting frames \| enforced at session.rs:280 write_bytes \| boundary test none on the Reply path (inv-2)
- invariants: I8: The telnet codes live only in telnet.rs \| enforced at task 4.1 rg search \| probe: misses byte escapes and u8::MAX (inv-1)
- invariants: I9: The 4.1 search accepts every hit in secret.rs because rg -l names files only, so a code added to secret.rs would pass; listing line hits with an exact expected set would close that.
- invariants: I10: Encoded lines never hold a lone 255 \| enforced by the &str type of Framer::encode (UTF-8 has no 0xFF) \| boundary test not needed
- invariants: No test or caller in crates relied on IAC IAC staying two bytes; search found 0xff only in secret.rs and mu-cli/tests/cli.rs, both unrelated, so the escape change weakens no check.
- test-coverage: Round 5 follow-up: the changed tests in the diff assert each THEN condition; no new gap found.
- test-honesty: Read every added or changed test in tests/portal.rs, tests/scripts.rs (the two new tests), framer.rs, fake.rs and session.rs. None is disabled, assertion-free, a mock echo or a constant compare.
- test-honesty: line_framer_marks_are_transparent takes its expected lines from LineFramer::new().feed on the mark-free stream. That is a differential oracle on a different input, and the mark positions are computed by the test itself, so it is not a tautology.
- test-honesty: wait_on_journal_with_events_at_start asserts only Ok from a bounded wait. It fails by the BOUND panic if the wait never returns, which is the defect it guards.
- test-coverage gap: The test bodies of fake_world_answers, fake_world_flag_is_hidden and missing_password_variable are unchanged by this change and not read; graded by name only.

## Ledger

| id | first | last | severity | scope | location | disposition | basis | override | claim |
|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 1 | blocker | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: The fake world stands in for a game"],"before and after the login. A mark SHALL NOT count as a line that the game got."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The fake-world requirement says the fake game answers a mark before the login and never counts a mark as a received line; no decision in design.md produces either rule. Only a risk bullet says the fake game splits lines itself. |
| R-2 | 1 | 1 | blocker | deviates-design | ["artifact",["tasks.md","4. No sleep in a test"],"In `tests/portal.rs`, remove the call from `refused_line_break`"] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Asserting at once in refused_line_break makes its game-side check vacuous: the fake game records a received line in its own task, so a regression that writes the refused line still passes. |
| R-3 | 1 | 1 | blocker | deviates-design | ["artifact",["tasks.md","4. No sleep in a test"],"from `stop_disconnects_every_world_and_refuses_connect`, so that each asserts right after the refusal"] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Asserting at once in stop_disconnects_every_world_and_refuses_connect makes its no-event check vacuous: a Connected event comes only after the session polls the finished connect, after the reply. |
| R-4 | 1 | 1 | should-fix | in-change | ["artifact",["proposal.md","Why"],"The `review-lint` gate `time-based-test` reports the sleep on every run."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The proposal says the time-based-test gate reports the sleep of settle on every run, but the gate reads only the added lines of a diff, so it reports settle only on a diff that adds it. |
| R-5 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"\| No test in the workspace sleeps \| `review-lint`, gate `time-based-test`, with no accepted finding. \|"] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The enforcement table says the time-based-test gate enforces that no test in the workspace sleeps, but the gate sees only added lines of the diff, so it cannot see a sleep that the diff does not add. |
| R-6 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Risks / Trade-offs"],"That bound passes only when nothing can run on the paused clock. The test then fails with a clear panic."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The mitigation of the endless-loop risk says the BOUND timeout then fails the test, but on the paused clock the timer fires only when the runtime has no work, and an endless loop always has work, so the test hangs. |
| R-7 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Context"],"The `select!` in `session::run` and the fake game in `fake.rs` are the only places that wait."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The context says the select! of session::run and the fake game are the only places that wait, but run_server_handlers waits on its subscription and a held FakeTransport connect waits on its gate. |
| R-8 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"In `fake.rs`, make the fake game read bytes and split the lines itself."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The fake game gets its own byte-to-line splitter that must also find a mark in any split, which LineFramer::feed already does for lines and which task 2.1 teaches it for marks. The design names neither LineFramer as the alternative nor a reason. |
| R-9 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"`LineFramer` gives `IAC DO TIMING-MARK`. It reads `IAC WILL TIMING-MARK` and `IAC WONT TIMING-MARK` as `Inbound::Mark`, in any split of reads."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Tasks 2.1 and 2.2 both write and read the same telnet bytes, IAC DO TIMING-MARK and IAC WILL TIMING-MARK, in framer.rs and in fake.rs, and neither the design nor the tasks name one place for the codes. |
| R-10 | 1 | 2 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"If the connection goes down, the session answers every frame that waits."] | fixed | 91366746399a17bc7fafd97b9e41153f2e573d87 |  | Frames that wait for a mark must be answered when the connection goes down; no task names a test of that case, and four paths reach Session::down. |
| R-11 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"If the connect failed, it answers them at once."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Frames in the connect list must be answered when the connect fails; only the success path has a test (wait_outlasts_connect). |
| R-12 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","3. The frame and the wait"],"Each `Inbound::Mark` answers the first frame in the list."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | An Inbound::Mark with no waiting frame (the empty-list case) has no stated behavior and no test; a Vec::remove(0) or unwrap there panics the session task. |
| R-13 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A control frame comes after the work before it"],"**AND** neither line holds a byte of the mark"] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The rule that no line holds a byte of the mark has no test at the MAX_LINE cut; the property test stops at 200 bytes, so a held IAC that fills the line to MAX_LINE is never tried. |
| R-14 | 1 | 4 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"It learns these two answers only."] | fixed | a59890617ab6f9cc7f559a8fcbc7f4d8a920a6cc |  | The framer must hold IAC across reads and give the bytes back when they do not form a mark; no rule or test covers a near miss such as IAC WILL ECHO, IAC IAC or IAC then LF. |
| R-15 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Progress of the server handlers through a watch channel"],"A wait for the server handlers is `wait_for(\|seq\| *seq >= mark)`."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The wait needs the published seq to start at or above every event before the subscription, but the design names no initial value; with Seq::default() a journal that holds events at start makes wait_for never return. |
| R-16 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Risks / Trade-offs"],"The fake game reads bytes and splits the lines itself."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The fake game moves from BufReader::lines to its own byte splitter, but no task tests a mark or a line split across reads, the edge that a reader of bytes must handle. |
| R-17 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A control frame comes after the work before it"],"The engine SHALL have a control frame. A frame SHALL go on the queue of one world."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The requirement 'A control frame comes after the work before it' shares only the word 'frame' with its task lines, so no task line matches its subject. |
| R-18 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: The engine can wait until it has no work left"],"A wait of the engine SHALL send rounds. Each round SHALL wait until the server handlers handle the last event"] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The requirement 'The engine can wait until it has no work left' has no task line with two of its distinctive words; task 3.2 names only 'wait' in 'Portal::wait_idle'. |
| R-19 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","1. Documentation"],"Make sure that the design names no fact that the documentation contradicts."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Task 1.1 asks that the design name no fact the documentation contradicts, but names no command, test, or concrete observable result to check that against. |
| R-20 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"make the fake game read bytes and split the lines itself."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Task 2.2 rewrites the fake game but only runs mu-core tests. The server-level fake-world scenarios live in mu-cli tests that no task runs. |
| R-21 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"The current `LineFramer` does not parse telnet yet. It learns these two answers only. The telnet-engine change replaces it and keeps the mark."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Each wait sends a new IAC DO TIMING-MARK, but the telnet-engine decision keeps one RFC 1143 Q state per option, so after the first WILL TM the engine sends no second DO and a policy that does not accept TM answers with DONT. The design says the telnet engine keeps the mark but does not name that TM must stay outside the Q method. |
| R-22 | 1 | 4 | suggestion | in-change | ["artifact",["design.md","The order of a round"],"Send a frame to each world, in the order of the ids, and wait for each answer."] | fixed | 7d66513b65b074b6d6ce523c86e54f05a227b07a |  | Step 3 of the round repeats the body of Portal::submit (look up the sender, make a oneshot, send a Request, await the answer) and the id-order loop of to_every_world; only the request variant differs. |
| R-23 | 1 | 1 | suggestion | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"Extend the property test `line_framer_ignores_read_splits` with marks."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Mark transparency holds over unbounded input: inserting marks at any point of a byte stream without 0xFF must leave the lines unchanged and give one Mark for each insert. |
| R-24 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"\| No test in the workspace sleeps \| Task 4.1 searches `crates/*/tests` and `crates/*/src` for `sleep(`."] | fixed | 91366746399a17bc7fafd97b9e41153f2e573d87 |  | The enforcement row says task 4.1's search of crates/*/tests and crates/*/src for sleep( enforces that no test in the workspace sleeps, but at head that search hits a sleeping unit test in mu-client and two production sleeps that the change does not touch, so the rule is false and the check of task 4.1 cannot pass. |
| R-25 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"passes with the four `fake_game_*` tests from before"] | fixed | 91366746399a17bc7fafd97b9e41153f2e573d87 |  | Task 2.2 says four fake_game_* tests exist before the change; fake.rs at head has five. |
| R-26 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The fake game reads through the line framer"],"The side is a parameter of the constructor: the client side ends a mark on WILL or WONT, and the game side ends it on DO."] | fixed | 91366746399a17bc7fafd97b9e41153f2e573d87 |  | The side goes into the constructor of the public LineFramer for the in-crate fake game; the crate keeps new() free of arguments and adds a variant through a named method, and the design names neither. |
| R-27 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"A frame that comes during a connect waits in a second list."] | fixed | 91366746399a17bc7fafd97b9e41153f2e573d87 |  | The two lists of waiting frames sit beside the connection state, but the session keeps the data of each connection state inside its Conn variant, and the design names neither that pattern nor a reason. |
| R-28 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The telnet bytes of the mark live in one place"],"A sequence that starts with IAC and is not a mark stays in the line as data, as today."] | fixed | 91366746399a17bc7fafd97b9e41153f2e573d87 |  | The plan adds a path that gives held IAC bytes back to the line, but the buffer bound is checked with == after each single push. Bytes given back at MAX_LINE - 1 skip the cut, so the line can grow without bound. No planned test feeds a non-mark IAC sequence at the bound. |
| R-29 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"Task 4.1 searches `crates/*/tests` and `crates/*/src` for `sleep(`."] | duplicate | R-24 |  | The named enforcement of 'No test in the workspace sleeps' cannot pass at head. The search of crates/*/src finds production backoff sleeps in mu-client and mu-cli, and a mu-client unit test that sleeps, all outside the change. So the rule has no working enforcement point. |
| R-30 | 2 | 3 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"The session keeps a list of the frames that wait for a mark, in order. Each answer of the game takes the first one."] | fixed | 9d36ded2c4efbb5a722b89f49454a87515d0ddf3 |  | The mark list and the connect list are FIFO lists, but every planned test has at most one waiting frame, because a round waits for each answer in turn. A single slot that overwrites an earlier frame passes every planned test. |
| R-31 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: A wait survives a failed connect"],"the journal already holds a disconnect event of `alpha/aria` that names the failed connect"] | fixed | 91366746399a17bc7fafd97b9e41153f2e573d87 |  | The THEN of the scenario 'A wait survives a failed connect' names the disconnect event by a content ('names the failed connect') with no value or field to check. |
| R-32 | 2 | 3 | suggestion | in-change | ["artifact",["design.md","The telnet bytes of the mark live in one place"],"`framer.rs` holds crate-private constants for IAC (255), DO (253), WILL (251), WONT (252) and TIMING-MARK (6)."] | fixed | 9d36ded2c4efbb5a722b89f49454a87515d0ddf3 |  | framer.rs gets crate-private IAC, DO, WILL and WONT constants, but the telnet engine of the next change already defines the same codes as public constants, so that change must move one set. |
| R-33 | 2 | 2 | suggestion | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"in `line_framer_ignores_read_splits`, a generator that favors the bytes 255, 251, 252, 253 and 6"] | fixed | 91366746399a17bc7fafd97b9e41153f2e573d87 |  | No line is longer than MAX_LINE, and the data bytes keep their order, over any stream and any split. This size bound holds over an unbounded domain, but the planned property tests stop at 200 bytes, far below the bound. |
| R-34 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The telnet bytes of the mark live in one place"],"The spike `spikes/telnet-engine/src/lib.rs` defines the same codes."] | fixed | 9d36ded2c4efbb5a722b89f49454a87515d0ddf3 |  | The design says the spike defines the same codes as telnet.rs, which include TIMING-MARK (6). At head the spike defines IAC, DO, WONT and WILL, but no TIMING-MARK constant. Its option list stops at the side protocols. |
| R-35 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The fake game reads through the line framer"],"The fake game answers each `Inbound::Mark` with `IAC WILL TIMING-MARK`, in the order of the framer output."] | fixed | 9d36ded2c4efbb5a722b89f49454a87515d0ddf3 |  | The game-side framer gives Inbound::Mark for IAC DO TIMING-MARK and the fake game builds the answer bytes itself, but the framer already has Inbound::Reply for negotiation replies that the reader writes in order. The design names neither that variant nor a reason, and Inbound::Mark gets two meanings, one per side. |
| R-36 | 3 | 3 | should-fix | in-change | ["artifact",["tasks.md","3. The frame and the wait"],"`wait_survives_failed_mark_write`: a test transport whose stream refuses each write after the login"] | fixed | 9d36ded2c4efbb5a722b89f49454a87515d0ddf3 |  | Task 3.3 asks for a new test transport whose stream refuses writes, but tests/portal.rs already has BrokenTransport, whose every write fails. With login None, as failed_write_records_disconnect uses it, the first write is the mark. |
| R-37 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"\| A drop or a failed connect answers the frames that wait \| The tests `wait_survives_drop`, `wait_survives_failed_mark_write` and `wait_survives_failed_connect`. \|"] | fixed | 9d36ded2c4efbb5a722b89f49454a87515d0ddf3 |  | The three tests named for 'a drop or a failed connect answers the frames that wait' cannot fail when down drops the list instead of answering it. Step 3 shares to_every_world, which discards each Err, and a dropped oneshot is an Err(Stopped), so the wait goes on as if the frame had been answered. |
| R-38 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The telnet bytes of the mark live in one place"],"The telnet-engine change grows `telnet.rs` into the engine, so that module stays the one home of the codes."] | fixed | 9d36ded2c4efbb5a722b89f49454a87515d0ddf3 |  | The design says telnet.rs stays the one home of the codes IAC, DO, WILL, WONT and TIMING-MARK. Nothing checks it: the enforcement table has no row for it, and no task searches for the codes outside telnet.rs. |
| R-39 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"\| No test of `mu-core` sleeps \| Task 4.1 searches `crates/mu-core/tests` and `crates/mu-core/src` for `sleep(`."] | fixed | 9d36ded2c4efbb5a722b89f49454a87515d0ddf3 |  | The rule 'No test of mu-core sleeps' rests on rg 'sleep\\(' only. A probe on three edge inputs found tokio::time::sleep(..) but missed tokio::time::sleep_until(..) and 'std::thread::sleep (d)', so a clock wait of those forms passes task 4.1. |
| R-40 | 3 | 3 | should-fix | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"`line_framer_keeps_other_iac_bytes`, with `IAC WILL 5` and, on the client side, `IAC DO TIMING-MARK`, each split after every byte"] | fixed | 9d36ded2c4efbb5a722b89f49454a87515d0ddf3 |  | The game side must end a mark on DO only, but the near-miss test checks the other side's answer only on the client side. A game-side framer that also ends a mark on IAC WILL or WONT TIMING-MARK passes every planned test. |
| R-41 | 3 | 3 | suggestion | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"marks inserted anywhere in a stream with no byte 255 leave the lines unchanged, and the count of marks is the count of inserts"] | fixed | 9d36ded2c4efbb5a722b89f49454a87515d0ddf3 |  | The design depends on the position of each Mark among the lines (every line the game sent before the mark comes first). This holds over any stream, but line_framer_marks_are_transparent checks only the lines and the count of marks, so an order that moves a mark past the next line passes it. |
| R-42 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",389] | fixed | 7d66513b65b074b6d6ce523c86e54f05a227b07a |  | The Reply branch of Session::on_bytes writes the bytes and calls down on an error itself. Session::write_bytes, which write_line and write_mark share, does the same job (and also flushes), so the session has two write-then-down paths. |
| R-43 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/fake.rs",232] | fixed | 419d4a703d138175a91293bc275bf1c6d4cb1778 |  | The new test helper open_raw in fake.rs repeats the body of the existing helper open line for line; only the BufReader wrap differs. |
| R-44 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/fake.rs",282] | fixed | 419d4a703d138175a91293bc275bf1c6d4cb1778 |  | fake_game_answers_mark_in_order and fake_game_answers_mark_before_login have the same loop body (cut at every byte, write_cut, compare the bytes, compare the lines); only the input, the expected bytes and the line list differ. Task 2.2 asks for the before-login case inside the one test. |
| R-45 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/tests/portal.rs",449] | fixed | 419d4a703d138175a91293bc275bf1c6d4cb1778 |  | The set-up of wait_survives_failed_mark_write (world id, ResolvedWorld with no login, WorldSetup on BrokenTransport, portal, connect_all, wait for Connected) repeats the set-up of failed_write_records_disconnect line for line. |
| R-46 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/tests/portal.rs",479] | fixed | 419d4a703d138175a91293bc275bf1c6d4cb1778 |  | The new handler struct Quiet in tests/portal.rs is a second way to build a test handler; tests/scripts.rs already has On and on(), which build a handler from a closure, and on(\|_\| Vec::new()) is Quiet. |
| R-47 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/fake.rs",121] | fixed | 7d66513b65b074b6d6ce523c86e54f05a227b07a |  | The fake game now owns a LineFramer but still builds each outgoing line by hand with format!("...\\r\\n"), which LineFramer::encode already does; the design names neither encode nor a reason. |
| R-48 | 4 | 5 | should-fix | in-change | ["artifact",["tasks.md","4. No sleep in a test"],"Make sure that `rg -n 'const [A-Z_]+: u8 = (6\|25[0-5]);' crates/mu-core/src` finds only lines of `telnet.rs`"] |  | regressed from fixed 1ff19b0c3486947b99db2ac88e8c6e0061cc80aa |  | The rule that the telnet codes live only in telnet.rs rests on a search for decimal u8 constants. A probe on three edge inputs under the handoff directory found `pub const IAC: u8 = 255;` but missed `const IAC: u8 = 0xFF;`, `const DO: u8 = 253u8;` and `[255, 253, 6]`, so a second home of the codes in those forms passes task 4.1. |
| R-49 | 5 | 5 | should-fix | deviates-design | ["artifact",["tasks.md","4. No sleep in a test"],"Make sure that `rg -n 'settle' crates/` and `rg -n -i 'sleep' crates/mu-core/tests crates/mu-core/src` find nothing."] |  |  |  | The rule that no test of mu-core sleeps is enforced by an rg run once in task 4.1, so a sleep added after the change passes every later check. The repository's pattern for a source-wide rule is a durable test over the Rust files (workspace_boundary.rs); the design names neither that pattern nor a reason. |
| R-50 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",317] |  |  |  | The answer of a frame (send Ok(None) and ignore a gone caller) is written twice in session.rs: in WaitingFrames::answer_first and in the Down branch of on_frame, with the same comment. |
| R-51 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",388] |  |  |  | The Reply branch of Session::on_bytes now goes through write_bytes, which flushes, calls down on a failed write and stops the rest of the read. No test gives the session an Inbound::Reply, so the success path and the failed-write edge (one disconnect event, waiting frames answered) are not exercised on this path. LineFramer::new never gives Reply. |
| R-52 | 5 | 5 | suggestion | in-change | ["artifact",["design.md","The telnet bytes of the mark live in one place"],"The escaped byte reaches the line as one byte 255, not as two."] |  |  |  | The rule that an escaped IAC is one data byte and never starts a mark holds over any byte stream. The transparency property excludes the byte 255 from its stream, so marks next to escaped 255 bytes at random positions and splits are tried only by the fixed cases of line_framer_reads_escaped_iac. |

## Reviewed files

| path | content |
|---|---|
| crates/mu-core/src/fake.rs | 474a1f022ac34cf15e9ee7645fcf645da25be6e8 |
| crates/mu-core/src/framer.rs | f8a385712e5115a2fc24e39f246b2178551c26c5 |
| crates/mu-core/src/lib.rs | c8f9731010211e46355962fe97a02f1fcc5f5e39 |
| crates/mu-core/src/portal.rs | c351d686da51b3954dac00341022e8398b233f5b |
| crates/mu-core/src/session.rs | 40fd3d3dc49bea25143505ec447618151e77e004 |
| crates/mu-core/src/telnet.rs | 09b4d1e77c63b06931fc0de8f09aff211eec7514 |
| crates/mu-core/tests/portal.rs | d60be6a42ce528513ea5309601e39cc2ce106788 |
| crates/mu-core/tests/scripts.rs | 58dad932a3caa4478af1812e091efb968b6dcd1c |
| crates/mu-core/tests/support/mod.rs | f441f0fed8f91078efa67e647bcaeed29f1594ce |
| openspec/changes/engine-quiescence/.openspec.yaml | cfbed6e8fdb007b75eb7f423f77f14d16ff7b4b4 |
| openspec/changes/engine-quiescence/design.md | d70fe779676eea1a6eadf4316e7bbcebfb5ff1b4 |
| openspec/changes/engine-quiescence/proposal.md | 8030d37ddde8211bcf9620b54049abd24ecf68d8 |
| openspec/changes/engine-quiescence/specs/world-sessions/spec.md | e979d8acec8e6b70e598819344f8cfed7979ebdc |
| openspec/changes/engine-quiescence/tasks.md | sha256:e22f24894def147d28b24c82be0da2c4e6ae4646818d48acbec0aad85cae95f1 |

## Plan review (2026-11-16)

Verdict approve-with-notes at head 91366746399a17bc7fafd97b9e41153f2e573d87.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-34 | approach | should-fix | in-change | The design says the spike defines the same codes as telnet.rs, which include TIMING-MARK (6). At head the spike defines IAC, DO, WONT and WILL, but no TIMING-MARK constant. Its option list stops at the side protocols. | design.md > The telnet bytes of the mark live in one place — "The spike `spikes/telnet-engine/src/lib.rs` defines the same codes." | Say that the spike defines IAC, DO, WILL and WONT but not TIMING-MARK, or drop the sentence. |  |  |
| R-35 | fit | should-fix | in-change | The game-side framer gives Inbound::Mark for IAC DO TIMING-MARK and the fake game builds the answer bytes itself, but the framer already has Inbound::Reply for negotiation replies that the reader writes in order. The design names neither that variant nor a reason, and Inbound::Mark gets two meanings, one per side. | design.md > The fake game reads through the line framer — "The fake game answers each `Inbound::Mark` with `IAC WILL TIMING-MARK`, in the order of the framer output." | Make LineFramer::game() give Inbound::Reply with IAC WILL TIMING-MARK for each IAC DO TIMING-MARK, and have the fake game write each Reply in framer order; or name Inbound::Reply in this decision with the reason it is not used. |  |  |
| R-36 | fit | should-fix | in-change | Task 3.3 asks for a new test transport whose stream refuses writes, but tests/portal.rs already has BrokenTransport, whose every write fails. With login None, as failed_write_records_disconnect uses it, the first write is the mark. | tasks.md > 3. The frame and the wait — "`wait_survives_failed_mark_write`: a test transport whose stream refuses each write after the login" | In task 3.3, build wait_survives_failed_mark_write on BrokenTransport with a world that has no login, as failed_write_records_disconnect does, instead of a new transport. |  | severity? suggestion 0.65 |
| R-30 | invariants | should-fix | in-change | two_waits_on_one_world checks only that both waits return. A single slot that overwrites the first frame, or a LIFO list, passes it: the overwritten frame's dropped oneshot is discarded by to_every_world, so the first wait returns early, without its mark. | design.md > Enforcement of each rule — "\| Each list keeps more than one frame in order \| The test `two_waits_on_one_world`. \|" | Assert that each frame gets an Ok answer from its own mark. For example, use a test game that answers the first mark only, check that only the first wait returned, then release the second mark. Or count two answered marks and fail on a dropped answer as in inv-1. | regression | severity? blocker 0.63 |
| R-37 | invariants | should-fix | in-change | The three tests named for 'a drop or a failed connect answers the frames that wait' cannot fail when down drops the list instead of answering it. Step 3 shares to_every_world, which discards each Err, and a dropped oneshot is an Err(Stopped), so the wait goes on as if the frame had been answered. | design.md > Enforcement of each rule — "\| A drop or a failed connect answers the frames that wait \| The tests `wait_survives_drop`, `wait_survives_failed_mark_w" | Make a dropped frame answer visible: have the frame step of wait_idle fail (or panic in tests) on a closed oneshot instead of discarding it in to_every_world, or have the three tests check that each frame got an Ok answer. Then a down that only replaces conn fails them. |  |  |
| R-38 | invariants | should-fix | in-change | The design says telnet.rs stays the one home of the codes IAC, DO, WILL, WONT and TIMING-MARK. Nothing checks it: the enforcement table has no row for it, and no task searches for the codes outside telnet.rs. | design.md > The telnet bytes of the mark live in one place — "The telnet-engine change grows `telnet.rs` into the engine, so that module stays the one home of the codes." | Add a row to the enforcement table and a check to task 4.1, for example an rg for the literal codes (255, 251, 252, 253 as u8, 0xff, b"\\xff") in crates/mu-core/src outside telnet.rs that must find nothing. |  |  |
| R-39 | invariants | should-fix | in-change | The rule 'No test of mu-core sleeps' rests on rg 'sleep\\(' only. A probe on three edge inputs found tokio::time::sleep(..) but missed tokio::time::sleep_until(..) and 'std::thread::sleep (d)', so a clock wait of those forms passes task 4.1. | design.md > Enforcement of each rule — "\| No test of `mu-core` sleeps \| Task 4.1 searches `crates/mu-core/tests` and `crates/mu-core/src` for `sleep(`." | Widen the search of task 4.1 and the table row, for example rg -n '\\bsleep(_until)?\\s*\\(' crates/mu-core/tests crates/mu-core/src. |  |  |
| R-40 | invariants | should-fix | in-change | The game side must end a mark on DO only, but the near-miss test checks the other side's answer only on the client side. A game-side framer that also ends a mark on IAC WILL or WONT TIMING-MARK passes every planned test. | tasks.md > 2. The mark on the wire — "`line_framer_keeps_other_iac_bytes`, with `IAC WILL 5` and, on the client side, `IAC DO TIMING-MARK`, each split after e" | Add game-side cases to line_framer_keeps_other_iac_bytes: IAC WILL TIMING-MARK and IAC WONT TIMING-MARK fed to LineFramer::game() stay in the line as data, split after every byte. |  |  |
| R-32 | fit | suggestion | in-change | telnet.rs is to grow into the telnet engine, whose spike keeps command codes at module level and option codes in `opt`, with no TIMING-MARK. The design lists TIMING-MARK beside IAC and DO with no layout, so the telnet-engine change must move it. | design.md > The telnet bytes of the mark live in one place — "holds the constants IAC (255), DO (253), WILL (251), WONT (252) and TIMING-MARK (6)" | State in this decision that telnet.rs follows the spike layout: IAC, DO, WILL and WONT at module level and TIMING-MARK as `opt::TIMING_MARK`; and say the spike defines the command codes, not TIMING-MARK. | regression | severity? should-fix 0.66 |
| R-41 | invariants | suggestion | in-change | The design depends on the position of each Mark among the lines (every line the game sent before the mark comes first). This holds over any stream, but line_framer_marks_are_transparent checks only the lines and the count of marks, so an order that moves a mark past the next line passes it. | tasks.md > 2. The mark on the wire — "marks inserted anywhere in a stream with no byte 255 leave the lines unchanged, and the count of marks is the count of i" | With proptest, as in line_framer_ignores_read_splits in crates/mu-core/src/framer.rs, extend line_framer_marks_are_transparent to assert that the k-th Mark comes after exactly the lines whose LF lies before the k-th insert point. |  |  |

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A wait covers the echo of the game | 3.3 | covered |  |
| A wait on a world that is down | 3.3 | covered |  |
| A wait outlasts a connect | 3.3 | covered |  |
| The line framer reads the answer to a mark | 2.1 | covered |  |
| A wait survives a drop | 3.3 | covered |  |
| A wait survives a failed connect | 3.3 | covered |  |
| A mark that no frame waits for | 3.3 | covered |  |
| A wait covers a loop of server handlers | 3.3 | covered |  |
| A wait adds nothing | 3.3 | covered |  |
| The fake world answers | 2.2 | covered |  |
| The fake world answers a mark in order | 2.2 | covered |  |
| The flag is hidden | 2.2 | covered |  |
| A missing password variable | 2.2 | covered |  |

### Not run

Plan phase: no verification runs.
