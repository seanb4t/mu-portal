---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-16T22:16:57+00:00
head: fd138e482a5bf2715e92afec1547817f29c87ccd
round: 4
next_id: 49
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
| R-14 | approach,invariants | should-fix | in-change | inv-1: The design keeps IAC IAC as an escaped data byte 255 and any non-mark IAC sequence as data, but LineFramer::step gives back the first IAC and re-reads the second as the start of a mark. So the data bytes 255 251 6, sent as IAC IAC WILL TIMING-MARK, give a stray Inbound::Mark on the client side (and a reply on the game side), and the line loses two bytes. No test feeds IAC IAC, and the bound property counts that window as a mark.; app-1: The design keeps each IAC sequence that is not a mark in the line as data, as today. LineFramer::step breaks the escaped data byte IAC IAC: it pushes the first IAC and holds the second as the start of a new sequence. So the data bytes 255 251 6, sent as IAC IAC WILL TM, give one byte 255 and a spurious Inbound::Mark on the client side, and IAC IAC DO TM gives a spurious reply on the game side. The design records no such deviation. | crates/mu-core/src/framer.rs:122 — "(Held::Iac, byte) => {" | inv-1: In LineFramer::step, on Held::Iac followed by IAC push both IAC bytes and return to Held::Nothing. Add IAC IAC WILL TIMING-MARK (client) and IAC IAC DO TIMING-MARK (game) to line_framer_keeps_other_iac_bytes, and count a mark in line_framer_lines_stay_in_the_bound only when its IAC is not escaped.; app-1: In LineFramer::step, read (Held::Iac, IAC) as the two data bytes IAC IAC and go back to Held::Nothing, and add IAC IAC WILL TM (client side) and IAC IAC DO TM (game side) to line_framer_keeps_other_iac_bytes. Or record the deviation in design.md. | regression | severity? blocker 0.89 |
| R-42 | fit | should-fix | in-change | The Reply branch of Session::on_bytes writes the bytes and calls down on an error itself. Session::write_bytes, which write_line and write_mark share, does the same job (and also flushes), so the session has two write-then-down paths. | crates/mu-core/src/session.rs:389 — "&& write.write_all(&bytes).await.is_err()" | In the Inbound::Reply branch of on_bytes, call self.write_bytes(&bytes, 0).await and return on Err, so one path writes to the game and reaches down. |  |  |
| R-43 | fit | should-fix | in-change | The new test helper open_raw in fake.rs repeats the body of the existing helper open line for line; only the BufReader wrap differs. | crates/mu-core/src/fake.rs:232 — "let transport = FakeTransport::new("aria", "s3cret-pw");" | Make open call open_raw and wrap the stream: let (game, stream) = open_raw().await; (game, BufReader::new(stream).lines()). |  |  |
| R-44 | fit | should-fix | in-change | fake_game_answers_mark_in_order and fake_game_answers_mark_before_login have the same loop body (cut at every byte, write_cut, compare the bytes, compare the lines); only the input, the expected bytes and the line list differ. Task 2.2 asks for the before-login case inside the one test. | crates/mu-core/src/fake.rs:282 — "let (game, got) = write_cut(&input, cut, &expected).await;" | Give fake_game_answers_mark_in_order a table of cases (input, expected bytes, lines the game got), with the before-login case as a row, and delete fake_game_answers_mark_before_login. |  | severity? suggestion 0.61 |
| R-45 | fit | should-fix | in-change | The set-up of wait_survives_failed_mark_write (world id, ResolvedWorld with no login, WorldSetup on BrokenTransport, portal, connect_all, wait for Connected) repeats the set-up of failed_write_records_disconnect line for line. | crates/mu-core/tests/portal.rs:449 — "let setup = WorldSetup::new(world, None, Arc::new(BrokenTransport));" | Add one helper in tests/portal.rs, beside BrokenTransport, that returns a connected portal on BrokenTransport with no login and its world id, and call it from both tests. |  | severity? suggestion 0.61 |
| R-46 | fit | should-fix | in-change | The new handler struct Quiet in tests/portal.rs is a second way to build a test handler; tests/scripts.rs already has On and on(), which build a handler from a closure, and on(\|_\| Vec::new()) is Quiet. | crates/mu-core/tests/portal.rs:479 — "impl Handler for Quiet {" | Move On and on() into crates/mu-core/tests/support/mod.rs, delete Quiet, and pass on(\|_\| Vec::new()) in wait_on_journal_with_events_at_start. |  |  |
| R-47 | fit | should-fix | in-change | The fake game now owns a LineFramer but still builds each outgoing line by hand with format!("...\\r\\n"), which LineFramer::encode already does; the design names neither encode nor a reason. | crates/mu-core/src/fake.rs:121 — "write.write_all(format!("{answer}\\r\\n").as_bytes()).await?;" | In serve, write framer.encode(&answer) and framer.encode(&text), so lines in both directions go through the one framer. |  |  |
| R-48 | invariants | should-fix | in-change | The rule that the telnet codes live only in telnet.rs rests on a search for decimal u8 constants. A probe on three edge inputs under the handoff directory found `pub const IAC: u8 = 255;` but missed `const IAC: u8 = 0xFF;`, `const DO: u8 = 253u8;` and `[255, 253, 6]`, so a second home of the codes in those forms passes task 4.1. | tasks.md > 4. No sleep in a test — "Make sure that `rg -n 'const [A-Z_]+: u8 = (6\|25[0-5]);' crates/mu-core/src` finds only lines of `telnet.rs`" | Widen the search of task 4.1 to hex, suffixed and array forms, for example rg -n -i '\\b(25[1-5]\|0xf[b-f])(u8)?\\b' crates/mu-core/src, and list the one known other hit (the non-UTF-8 byte in the secret.rs test) as accepted. |  |  |
| R-22 | fit | suggestion | in-change | The two arms of Portal::ask repeat the same body (make a oneshot, send a Request, map a send error to Stopped, await, map a closed channel to Stopped); they differ only in the Request constructor and the answer type, and the enum Ask mirrors the variants of Request. | crates/mu-core/src/portal.rs:219 — "let (frame, answer) = oneshot::channel();" | Name the parameter: a function from the answer sender to a Request (for example, let Request::Frame carry a oneshot::Sender<Reply> answered with Ok(None)), so ask takes fn(oneshot::Sender<Reply>) -> Request and has one body, and Ask goes. | regression |  |

## Repeats

None.

## Out of bound

None.

## Accepted

None.

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | f269a3bfa5f76277baba39eccabf36fe65076958 | d37fc84f5010c6f18872b6a56179c3a647c28ab0c83b5f03469b3eb001e0b747 | changes-requested | 23 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | d28a104623ce37f052a9d455c79e0b571fbf5959 | 4b7f0a5225a2decbbbac73dcf349088cc940277b1203e2e238bb4a5bd1d70481 | approve-with-notes | 10 | 0 | 2 | 0 |  |
| 3 | plan | follow-up | 91366746399a17bc7fafd97b9e41153f2e573d87 | d9512329a6f73b992cedb465144fcaf27f3e82f2d19a1a322d85ddc92e2aa82e | approve-with-notes | 8 | 0 | 2 | 0 |  |
| 4 | implementation | full | fd138e482a5bf2715e92afec1547817f29c87ccd | 3ad7f6b96bbf7a7b8783e61e8746e568d1f262636268ad6a16de36b99a2184e7 | approve-with-notes | 7 | 0 | 2 | 0 |  |

## Judge

judge: 18 questions, 0.001 USD

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
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E test(=wait_survives_drop)`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E test(=wait_survives_failed_connect)`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E test(=wait_adds_nothing)`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E test(=stray_mark_is_ignored)`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E test(=wait_covers_server_handler_loop)`
- exit 0: `cargo nextest run -p mu-core --lib --locked -E test(=framer::tests::line_framer_reads_mark_answer)`
- exit 0: `cargo nextest run -p mu-core --lib --locked -E test(=fake::tests::fake_game_answers_mark_in_order)`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E test(=fake_world_answers)`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E test(=fake_world_flag_is_hidden)`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E test(=missing_password_variable)`

## Dropped

None.

## Lens notes

- approach: Every prior row with disposition fixed was checked at head where it touches a design citation or a decision; none of their defects came back in the approach view.
- approach: The tokio 1.54.1 rows of Checked facts were checked against the vendored source in the cargo registry (watch::Receiver::wait_for, Sender::send and send_replace, RuntimeMetrics stable methods). The RFC 860, 854 and 1143 rows were checked against the known text of the RFCs, not against a local copy.
- approach: Portal::wait_idle also fails with PortalError::Stopped when the server handler task ends; the round in design.md names only the failure of a frame. This adds behavior and departs from no decision.
- approach: No Open Questions section and no migration plan exist, so answered-open-question and no-rollback do not apply.
- fit: Library rows come from openspec/config.yaml context. The test-helper row names crates/mu-cli/tests/support/mod.rs; mu-core has its own tests/support/mod.rs, which fit-5 uses as the collapse location.
- fit: telnet.rs follows the layout of spikes/telnet-engine/src/lib.rs (commands at the top, options in opt), and no other u8 telnet code constant exists under crates/mu-core/src; no finding.
- fit: The progress watch channel of run_server_handlers follows the watch::Sender pattern of FakeGame (received, connects) and of the world states; no finding.
- fit: I could not run git, so for fit-1 and fit-6 I could not confirm which of the cited lines this change wrote. Both sit in code that the change rewrote: on_bytes and write_bytes, and serve.
- invariants: I1: No line of LineFramer is longer than MAX_LINE \| enforced at framer.rs push, the == MAX_LINE check after each byte, which given-back held bytes also pass \| boundary test line_framer_near_miss_at_the_bound, line_framer_mark_at_the_bound, property line_framer_lines_stay_in_the_bound (prefix MAX_LINE - 4, so it crosses the cut)
- invariants: I2: A mark never lands in a line, in any split of reads \| enforced at framer.rs step, the Held state \| boundary test line_framer_keeps_other_iac_bytes (split after every byte), property line_framer_ignores_read_splits, property line_framer_marks_are_transparent (checks the position of each mark among the lines)
- invariants: I3: An IAC sequence that is not a mark stays as data, IAC IAC included \| enforced at framer.rs step for IAC WILL 5 and IAC then LF, not for IAC IAC \| boundary test none for IAC IAC (inv-1)
- invariants: I4: The client side ends a mark only on WILL or WONT, the game side only on DO \| enforced at framer.rs Side::reads_verb \| boundary test line_framer_keeps_other_iac_bytes, both sides
- invariants: I5: Waiting frames are answered first in, first out, and a mark with an empty list changes nothing \| enforced at session.rs WaitingFrames::answer_first (pop_front, if let) \| boundary test waiting_frames_answer_in_order (three frames and an answer on an empty list), stray_mark_is_ignored
- invariants: I6: Session::down answers every frame of the state it leaves \| enforced at session.rs down, mem::replace then answer_all \| boundary test wait_survives_drop, wait_survives_failed_connect, wait_survives_failed_mark_write, each asserting Ok
- invariants: I7: A frame is in the list before its mark is written \| enforced at session.rs on_frame, push then write_mark \| boundary test wait_survives_failed_mark_write (BrokenTransport, login None, so the mark is the first write)
- invariants: I8: Frames of a connect move to Conn::Up and get one mark each \| enforced at session.rs on_connect, mem::take and a loop of waiting marks that stops on the first failed write \| boundary test wait_outlasts_connect, two_waits_on_one_world (two frames in each list)
- invariants: I9: The progress of the server handlers starts at journal.last() and only grows \| enforced at portal.rs Portal::start watch::channel(last), send_replace after run_handlers \| boundary test wait_on_journal_with_events_at_start (seq == mark), wait_covers_server_handler_loop (seq < mark)
- invariants: I10: wait_idle returns only after a round with no new event, and a frame with no answer fails it \| enforced at portal.rs wait_idle and ask (answer.await maps a dropped sender to Stopped) \| boundary test wait_adds_nothing, two_waits_on_one_world
- invariants: I11: The fake game answers each mark in order, before and after the login, and never counts it as a line \| enforced at fake.rs serve, Inbound::Reply written and skipped \| boundary test fake_game_answers_mark_in_order and fake_game_answers_mark_before_login, cut at every byte
- invariants: I12: No test of mu-core sleeps \| enforced at task 4.1 rg -n -i 'sleep' (one-time search) \| boundary test none; the search finds nothing at head and catches sleep_until and 'sleep (' by its form
- invariants: I13: The telnet codes live only in telnet.rs \| enforced at task 4.1 const search \| boundary test none; probe in probe-inv/src under the handoff directory misses hex, suffixed and array forms (inv-2)
- invariants: The fake game moved from BufReader::lines to the line framer: a last line with no LF at EOF is now dropped, invalid UTF-8 is now lossy, and a CR inside a line is dropped. No test or caller found that relies on the old behavior, so no weakened finding.
- test-coverage: Every scenario has a test that asserts its THEN. No BREAKING requirement. The wait-fails-on-unanswered-frame rule has no scenario, so it is not in the matrix.
- test-honesty: Read every added or changed test in tests/portal.rs, tests/scripts.rs, framer.rs, fake.rs and session.rs. No disabled test, no test without assertion, no sleep.
- test-honesty: line_framer_marks_are_transparent builds its expected lines with LineFramer::new().feed on the stream without marks. That is a differential check on a stream with no byte 255, and other tests pin the plain line behavior, so it is not reported.

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
| R-14 | 1 | 4 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"It learns these two answers only."] |  | regressed from fixed d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The framer must hold IAC across reads and give the bytes back when they do not form a mark; no rule or test covers a near miss such as IAC WILL ECHO, IAC IAC or IAC then LF. |
| R-15 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Progress of the server handlers through a watch channel"],"A wait for the server handlers is `wait_for(\|seq\| *seq >= mark)`."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The wait needs the published seq to start at or above every event before the subscription, but the design names no initial value; with Seq::default() a journal that holds events at start makes wait_for never return. |
| R-16 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Risks / Trade-offs"],"The fake game reads bytes and splits the lines itself."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The fake game moves from BufReader::lines to its own byte splitter, but no task tests a mark or a line split across reads, the edge that a reader of bytes must handle. |
| R-17 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A control frame comes after the work before it"],"The engine SHALL have a control frame. A frame SHALL go on the queue of one world."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The requirement 'A control frame comes after the work before it' shares only the word 'frame' with its task lines, so no task line matches its subject. |
| R-18 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: The engine can wait until it has no work left"],"A wait of the engine SHALL send rounds. Each round SHALL wait until the server handlers handle the last event"] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The requirement 'The engine can wait until it has no work left' has no task line with two of its distinctive words; task 3.2 names only 'wait' in 'Portal::wait_idle'. |
| R-19 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","1. Documentation"],"Make sure that the design names no fact that the documentation contradicts."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Task 1.1 asks that the design name no fact the documentation contradicts, but names no command, test, or concrete observable result to check that against. |
| R-20 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"make the fake game read bytes and split the lines itself."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Task 2.2 rewrites the fake game but only runs mu-core tests. The server-level fake-world scenarios live in mu-cli tests that no task runs. |
| R-21 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"The current `LineFramer` does not parse telnet yet. It learns these two answers only. The telnet-engine change replaces it and keeps the mark."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Each wait sends a new IAC DO TIMING-MARK, but the telnet-engine decision keeps one RFC 1143 Q state per option, so after the first WILL TM the engine sends no second DO and a policy that does not accept TM answers with DONT. The design says the telnet engine keeps the mark but does not name that TM must stay outside the Q method. |
| R-22 | 1 | 4 | suggestion | in-change | ["artifact",["design.md","The order of a round"],"Send a frame to each world, in the order of the ids, and wait for each answer."] |  | regressed from fixed 91366746399a17bc7fafd97b9e41153f2e573d87 |  | Step 3 of the round repeats the body of Portal::submit (look up the sender, make a oneshot, send a Request, await the answer) and the id-order loop of to_every_world; only the request variant differs. |
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
| R-42 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",389] |  |  |  | The Reply branch of Session::on_bytes writes the bytes and calls down on an error itself. Session::write_bytes, which write_line and write_mark share, does the same job (and also flushes), so the session has two write-then-down paths. |
| R-43 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/fake.rs",232] |  |  |  | The new test helper open_raw in fake.rs repeats the body of the existing helper open line for line; only the BufReader wrap differs. |
| R-44 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/fake.rs",282] |  |  |  | fake_game_answers_mark_in_order and fake_game_answers_mark_before_login have the same loop body (cut at every byte, write_cut, compare the bytes, compare the lines); only the input, the expected bytes and the line list differ. Task 2.2 asks for the before-login case inside the one test. |
| R-45 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/tests/portal.rs",449] |  |  |  | The set-up of wait_survives_failed_mark_write (world id, ResolvedWorld with no login, WorldSetup on BrokenTransport, portal, connect_all, wait for Connected) repeats the set-up of failed_write_records_disconnect line for line. |
| R-46 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/tests/portal.rs",479] |  |  |  | The new handler struct Quiet in tests/portal.rs is a second way to build a test handler; tests/scripts.rs already has On and on(), which build a handler from a closure, and on(\|_\| Vec::new()) is Quiet. |
| R-47 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/fake.rs",121] |  |  |  | The fake game now owns a LineFramer but still builds each outgoing line by hand with format!("...\\r\\n"), which LineFramer::encode already does; the design names neither encode nor a reason. |
| R-48 | 4 | 4 | should-fix | in-change | ["artifact",["tasks.md","4. No sleep in a test"],"Make sure that `rg -n 'const [A-Z_]+: u8 = (6\|25[0-5]);' crates/mu-core/src` finds only lines of `telnet.rs`"] |  |  |  | The rule that the telnet codes live only in telnet.rs rests on a search for decimal u8 constants. A probe on three edge inputs under the handoff directory found `pub const IAC: u8 = 255;` but missed `const IAC: u8 = 0xFF;`, `const DO: u8 = 253u8;` and `[255, 253, 6]`, so a second home of the codes in those forms passes task 4.1. |

## Reviewed files

| path | content |
|---|---|
| crates/mu-core/src/fake.rs | 66b117ed6a10838812d17adf7a0b55269b04dc9b |
| crates/mu-core/src/framer.rs | 8e79ef53af528d1eb7bf5f48ccfd452a5d9f7743 |
| crates/mu-core/src/lib.rs | c8f9731010211e46355962fe97a02f1fcc5f5e39 |
| crates/mu-core/src/portal.rs | b3128400f7c1ebcdad3dd53a890eb2b2e324e889 |
| crates/mu-core/src/session.rs | cbbde6ba128158e9f306a3266709b39ad567bf0e |
| crates/mu-core/src/telnet.rs | 09b4d1e77c63b06931fc0de8f09aff211eec7514 |
| crates/mu-core/tests/portal.rs | 522b877dfba73116da65622a4a182eef9071ed30 |
| crates/mu-core/tests/scripts.rs | 6bbae7aaed71d5171bed230ffc78276d771c13da |
| crates/mu-core/tests/support/mod.rs | 89db7fe982726b71a970f66a0384ef5498933dff |
| openspec/changes/engine-quiescence/.openspec.yaml | cfbed6e8fdb007b75eb7f423f77f14d16ff7b4b4 |
| openspec/changes/engine-quiescence/design.md | a78370942e4d7bca9edfd39a66e0912ef2ac2c02 |
| openspec/changes/engine-quiescence/proposal.md | 8030d37ddde8211bcf9620b54049abd24ecf68d8 |
| openspec/changes/engine-quiescence/specs/world-sessions/spec.md | e979d8acec8e6b70e598819344f8cfed7979ebdc |
| openspec/changes/engine-quiescence/tasks.md | sha256:2413aa263a1bc556a1a7df20267665f3f8480f23d31ef55dc45313208b298f33 |

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
