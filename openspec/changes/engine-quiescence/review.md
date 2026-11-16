---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-16T20:52:51+00:00
head: f269a3bfa5f76277baba39eccabf36fe65076958
round: 1
next_id: 24
lenses:
  spec-lint: ok
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-1 | approach | blocker | in-change | The fake-world requirement says the fake game answers a mark before the login and never counts a mark as a received line; no decision in design.md produces either rule. Only a risk bullet says the fake game splits lines itself. | specs/world-sessions/spec.md > Requirement: The fake world stands in for a game — "before and after the login. A mark SHALL NOT count as a line that the game got." | Add a decision for the fake game: it answers IAC DO TIMING-MARK with IAC WILL TIMING-MARK in read order in both login states, and does not push the mark into the received lines. |  | severity? should-fix 0.71 |
| R-2 | invariants | blocker | deviates-design | Asserting at once in refused_line_break makes its game-side check vacuous: the fake game records a received line in its own task, so a regression that writes the refused line still passes. | tasks.md > 4. No sleep in a test — "In `tests/portal.rs`, remove the call from `refused_line_break`"; scope basis: design.md > The test helper — "the session sends the refusal as its reply, after it handled the request, so the next line can assert at once" | Replace the settle call in refused_line_break with idle(&portal), so the mark round trip proves the game read every byte before it, and keep the game.lines() assertion after it. |  | scope? in-change 0.98; severity? should-fix 0.71 |
| R-3 | invariants | blocker | deviates-design | Asserting at once in stop_disconnects_every_world_and_refuses_connect makes its no-event check vacuous: a Connected event comes only after the session polls the finished connect, after the reply. | tasks.md > 4. No sleep in a test — "from `stop_disconnects_every_world_and_refuses_connect`, so that each asserts right after the refusal"; scope basis: design.md > The test helper — "The two refusal tests in `tests/portal.rs` lose the call" | Call idle(&portal) between the refused connect and the journal assertion. A frame during a connect waits for its end, so the wait then covers a connect that a regression starts. |  | severity? should-fix 0.65 |
| R-4 | approach | should-fix | in-change | The proposal says the time-based-test gate reports the sleep of settle on every run, but the gate reads only the added lines of a diff, so it reports settle only on a diff that adds it. | proposal.md > Why — "The `review-lint` gate `time-based-test` reports the sleep on every run." | Restate the reason: the gate reports a sleep that a diff adds, so each change that adds a call to a sleeping helper trips it; or name the run that reported it. |  |  |
| R-5 | approach | should-fix | in-change | The enforcement table says the time-based-test gate enforces that no test in the workspace sleeps, but the gate sees only added lines of the diff, so it cannot see a sleep that the diff does not add. | design.md > Enforcement of each rule — "\| No test in the workspace sleeps \| `review-lint`, gate `time-based-test`, with no accepted finding. \|" | Narrow the row to 'this change adds no sleep to a test', or name a check that reads the whole test tree, for example an rg search for sleep( under the crates' tests in task 4.1. |  |  |
| R-6 | approach | should-fix | in-change | The mitigation of the endless-loop risk says the BOUND timeout then fails the test, but on the paused clock the timer fires only when the runtime has no work, and an endless loop always has work, so the test hangs. | design.md > Risks / Trade-offs — "That bound passes only when nothing can run on the paused clock. The test then fails with a clear panic." | State the real mitigation: every handler loop stops at the depth limit or at the rate limit (whose clock does not advance while work runs), or bound the number of rounds in wait_idle and panic past it. |  | severity? blocker 0.75 |
| R-7 | approach | should-fix | in-change | The context says the select! of session::run and the fake game are the only places that wait, but run_server_handlers waits on its subscription and a held FakeTransport connect waits on its gate. | design.md > Context — "The `select!` in `session::run` and the fake game in `fake.rs` are the only places that wait." | Name every place that waits, or restrict the sentence to the places that race a read of the game against the queue. |  |  |
| R-8 | fit | should-fix | in-change | The fake game gets its own byte-to-line splitter that must also find a mark in any split, which LineFramer::feed already does for lines and which task 2.1 teaches it for marks. The design names neither LineFramer as the alternative nor a reason. | tasks.md > 2. The mark on the wire — "In `fake.rs`, make the fake game read bytes and split the lines itself." | Let the fake game feed its bytes through LineFramer, with the verbs that end a mark as a parameter (WILL and WONT for the client side, DO for the game side), or name in the design why the fake game needs a second splitter. |  |  |
| R-9 | fit | should-fix | in-change | Tasks 2.1 and 2.2 both write and read the same telnet bytes, IAC DO TIMING-MARK and IAC WILL TIMING-MARK, in framer.rs and in fake.rs, and neither the design nor the tasks name one place for the codes. | tasks.md > 2. The mark on the wire — "`LineFramer` gives `IAC DO TIMING-MARK`. It reads `IAC WILL TIMING-MARK` and `IAC WONT TIMING-MARK` as `Inbound::Mark`, " | Name in task 2.1 one set of crate-private constants for IAC, DO, WILL, WONT and TIMING-MARK in framer.rs, and have task 2.2 use them in fake.rs. |  |  |
| R-10 | invariants | should-fix | in-change | Frames that wait for a mark must be answered when the connection goes down; no task names a test of that case, and four paths reach Session::down. | design.md > TIMING-MARK for the round trip through the game — "If the connection goes down, the session answers every frame that waits." | In task 3.1 answer the waiting frames inside Session::down, the one place every drop path reaches. In task 3.3 add a test that sends `@drop` and then waits: the fake game closes before it reads the mark, and the wait must return with a Disconnected event. |  |  |
| R-11 | invariants | should-fix | in-change | Frames in the connect list must be answered when the connect fails; only the success path has a test (wait_outlasts_connect). | design.md > TIMING-MARK for the round trip through the game — "If the connect failed, it answers them at once." | Add a test in task 3.3: a held FakeTransport, connect, start a wait, drop the ConnectGate so the connect fails, and assert that the wait returns and the journal holds the failed-connect Disconnected event. |  |  |
| R-12 | invariants | should-fix | in-change | An Inbound::Mark with no waiting frame (the empty-list case) has no stated behavior and no test; a Vec::remove(0) or unwrap there panics the session task. | tasks.md > 3. The frame and the wait — "Each `Inbound::Mark` answers the first frame in the list." | State in task 3.1 that a mark with an empty list is ignored, and add a test with a test framer (like Shout in tests/scripts.rs) that maps a game line to Inbound::Mark, so game.say gives a mark that no frame waits for. |  |  |
| R-13 | invariants | should-fix | in-change | The rule that no line holds a byte of the mark has no test at the MAX_LINE cut; the property test stops at 200 bytes, so a held IAC that fills the line to MAX_LINE is never tried. | specs/world-sessions/spec.md > Requirement: A control frame comes after the work before it — "**AND** neither line holds a byte of the mark" | Add a unit test in task 2.1: MAX_LINE - 1 data bytes, then IAC WILL TIMING-MARK, then LF, fed whole and split after IAC. Assert one Mark and that no line holds 0xFF. |  |  |
| R-14 | invariants | should-fix | in-change | The framer must hold IAC across reads and give the bytes back when they do not form a mark; no rule or test covers a near miss such as IAC WILL ECHO, IAC IAC or IAC then LF. | design.md > TIMING-MARK for the round trip through the game — "It learns these two answers only." | State in design.md what the LineFramer gives for an IAC sequence that is not a mark (the bytes stay in the line, as today), and add a test of IAC WILL 0x05 and IAC DO TIMING-MARK, split after each byte. Bias the property-test generator to 0xFF, 0xFB, 0xFC and 0x06. |  |  |
| R-15 | invariants | should-fix | in-change | The wait needs the published seq to start at or above every event before the subscription, but the design names no initial value; with Seq::default() a journal that holds events at start makes wait_for never return. | design.md > Progress of the server handlers through a watch channel — "A wait for the server handlers is `wait_for(\|seq\| *seq >= mark)`." | In task 3.2 create the channel as watch::channel(journal.last()) next to the subscription in Portal::start, and add a test that appends an event to a MemoryJournal, starts a portal with a server handler on it, and waits. |  | severity? blocker 0.87 |
| R-16 | invariants | should-fix | in-change | The fake game moves from BufReader::lines to its own byte splitter, but no task tests a mark or a line split across reads, the edge that a reader of bytes must handle. | design.md > Risks / Trade-offs — "The fake game reads bytes and splits the lines itself." | In task 2.2 put the splitter in a pure type with a feed method, and test it on `one\\r\\n`, IAC DO TIMING-MARK and `two\\r\\n` cut at every byte, asserting the lines and one mark in order. |  |  |
| R-17 | spec-lint | should-fix | in-change | The requirement 'A control frame comes after the work before it' shares only the word 'frame' with its task lines, so no task line matches its subject. | specs/world-sessions/spec.md > Requirement: A control frame comes after the work before it — "The engine SHALL have a control frame. A frame SHALL go on the queue of one world." | Name the control frame in a task line, for example 'add the control frame request beside the command request', so the requirement subject appears with a second distinctive word. |  |  |
| R-18 | spec-lint | should-fix | in-change | The requirement 'The engine can wait until it has no work left' has no task line with two of its distinctive words; task 3.2 names only 'wait' in 'Portal::wait_idle'. | specs/world-sessions/spec.md > Requirement: The engine can wait until it has no work left — "A wait of the engine SHALL send rounds. Each round SHALL wait until the server handlers handle the last event" | Name the engine wait and the no-work end in one task line, for example 'add Portal::wait_idle, the engine wait until no work is left, with the round of the design'. |  |  |
| R-19 | spec-lint | should-fix | in-change | Task 1.1 asks that the design name no fact the documentation contradicts, but names no command, test, or concrete observable result to check that against. | tasks.md > 1. Documentation — "Make sure that the design names no fact that the documentation contradicts." | State the observable: for example, list each RFC 860 and tokio fact used in design.md with its source section, and note each difference in design.md as a named edit. |  |  |
| R-20 | test-coverage | should-fix | in-change | Task 2.2 rewrites the fake game but only runs mu-core tests. The server-level fake-world scenarios live in mu-cli tests that no task runs. | tasks.md > 2. The mark on the wire — "make the fake game read bytes and split the lines itself." | Add `cargo nextest run -p mu-cli --locked` to task 2.2 or 4.2. It must cover the scenarios 'The fake world answers', 'The flag is hidden' and 'A missing password variable'. Name the mu-cli tests that prove them. |  |  |
| R-21 | fit | suggestion | in-change | Each wait sends a new IAC DO TIMING-MARK, but the telnet-engine decision keeps one RFC 1143 Q state per option, so after the first WILL TM the engine sends no second DO and a policy that does not accept TM answers with DONT. The design says the telnet engine keeps the mark but does not name that TM must stay outside the Q method. | design.md > TIMING-MARK for the round trip through the game — "The current `LineFramer` does not parse telnet yet. It learns these two answers only. The telnet-engine change replaces " | In the TIMING-MARK decision, state that the mark is sent and read outside the per-option Q state and the accept policy, so that the telnet-engine change keeps Framer mark and Inbound::Mark without a wrap. |  | severity? should-fix 0.75 |
| R-22 | fit | suggestion | in-change | Step 3 of the round repeats the body of Portal::submit (look up the sender, make a oneshot, send a Request, await the answer) and the id-order loop of to_every_world; only the request variant differs. | design.md > The order of a round — "Send a frame to each world, in the order of the ids, and wait for each answer." | Name the parameter in the design: one private send-and-await of a Request, with the variant (command or frame) as the parameter, that submit and wait_idle both call. |  |  |
| R-23 | invariants | suggestion | in-change | Mark transparency holds over unbounded input: inserting marks at any point of a byte stream without 0xFF must leave the lines unchanged and give one Mark for each insert. | tasks.md > 2. The mark on the wire — "Extend the property test `line_framer_ignores_read_splits` with marks." | Add a proptest (proptest 1.12.0, as in crates/mu-core/src/framer.rs) line_framer_marks_are_transparent: lines of the stream with marks inserted equal lines of the stream alone, and the Mark count equals the insert count. |  |  |

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

## Judge

judge: 46 questions, 0.002 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A wait covers the echo of the game | 3.3 | covered |  |
| A wait on a world that is down | 3.3 | covered |  |
| A wait outlasts a connect | 3.3 | covered |  |
| The line framer reads the answer to a mark | 2.1 | covered |  |
| A wait covers a loop of server handlers | 3.3 | covered |  |
| A wait adds nothing | 3.3 | covered |  |
| The fake world answers | 2.2 | partial |  |
| The fake world answers a mark in order | 2.2 | covered |  |
| The flag is hidden |  | partial |  |
| A missing password variable |  | partial |  |

### Not run

Plan phase: no verification runs.

## Verification runs

None.

## Dropped

None.

## Lens notes

- approach: Every scenario THEN is producible from the design: Inbound::Mark, FakeGame::lines, the held FakeTransport and DEPTH_LIMIT 8 exist or are created.
- approach: Cited facts that hold at head: select! picks a branch at random (tokio 1.54.1 select.rs:63), watch::Receiver::wait_for and RuntimeMetrics::num_alive_tasks exist, Seq is Ord, record routes handler commands before it returns, the two refusal tests reply from the session.
- approach: For the fit or spec-lint lens: the non-goal 'A wait for work outside the engine, for example a connect that a gate holds' reads against the scenario 'A wait outlasts a connect', which waits through a held connect.
- approach: The claim that a cross-world handler command is safe in any order holds only on a current_thread runtime, where append and route in Session::record run with no interleaving; the design does not state that assumption. All paused-clock tests are current_thread.
- approach: The design does not say whether the frames of the second list (during a connect) are answered when a Disconnect cancels the connect; 'If the connection goes down, the session answers every frame that waits' may or may not cover them.
- approach: A line queued with FakeGame::say before a wait is not ordered against the mark: the fake game's select! can answer the mark first. No current test depends on it.
- approach: The design does not give the initial value of the watch channel of the server handlers; it must be the journal's last sequence number at start, or a wait on a journal with earlier events never ends.
- approach: The new Framer method of task 2.1 needs a default, or the Shout framer in crates/mu-core/tests/scripts.rs:328 no longer compiles; the design names no such method.
- fit: The watch channel for the progress of the server handlers fits the existing pattern: WorldState and FakeGame already publish through watch senders and wait with wait_for.
- fit: The test helper idle under timeout(BOUND, ...) fits wait_event and wait_state in crates/mu-core/tests/support/mod.rs.
- fit: The only telnet byte constants in the repository are in spikes/telnet-engine/src/lib.rs, which the workspace excludes, so they are no collapse target.
- fit: tests/scripts.rs implements Framer for Shout (line 330). A new required method on Framer touches it, and the tasks do not name it.
- fit: openspec/config.yaml has no library row for telnet, and the workspace has no telnet crate, so no bespoke-over-library case applies.
- invariants: I1: A frame is answered only after each request before it on its queue. \| enforced at session::run FIFO queue and the mark list (task 3.1) \| boundary test wait_covers_game_echo (task 3.3)
- invariants: I2: A frame records no event and sends no line to the game. \| enforced at the frame branch of session (task 3.1) and the fake game splitter (task 2.2) \| boundary test wait_adds_nothing (task 3.3)
- invariants: I3: Every frame is answered: at once on a down world, at once after a failed connect, and at a drop. \| enforced at session (task 3.1) \| boundary test wait_on_world_that_is_down only; none for a drop (inv-3) or a failed connect (inv-4)
- invariants: I4: Each Inbound::Mark answers the oldest waiting frame, and a mark with no waiting frame is harmless. \| enforced at session (task 3.1) \| boundary test none for the empty list (inv-5)
- invariants: I5: LineFramer gives Inbound::Mark for IAC WILL/WONT TIMING-MARK in any split, and no line holds a mark byte. \| enforced at LineFramer::feed (task 2.1) \| boundary test line_framer_reads_mark_answer and the split proptest; none at MAX_LINE (inv-6) or for a near miss (inv-7)
- invariants: I6: The fake game answers a mark after every line before it, and a mark is not a received line. \| enforced at fake::serve (task 2.2) \| boundary test fake_game_answers_mark_in_order and a mark before login; none for a split read (inv-9)
- invariants: I7: The published seq of the server handlers is monotonic and starts at or above each event before the subscription. \| enforced at run_server_handlers (task 3.2), initial value none \| boundary test none (inv-8)
- invariants: I8: wait_idle returns only after a round with no new event, and waits for the server handlers before it sends frames. \| enforced at Portal::wait_idle (task 3.2) \| boundary test handler_loop_stops_at_depth_limit with idle; a skipped or reversed step 2 leaves the Emit on the queue and fails it
- invariants: I9: Handler loops end at depth 8 (DEPTH_LIMIT) or 20 script commands a second (RATE_PER_SECOND). \| enforced at Session::handle (session.rs:319, 326) \| boundary test handler_loop_stops_at_depth_limit (1 + 8 custom, one error, both sides of the limit), handler_loop_through_game_is_rate_limited
- invariants: I10: A refused call records no event and sends no line. \| enforced at Session::handle (session.rs:332) \| boundary test refused_line_break, weakened by the plan (inv-1)
- invariants: I11: A connect after stop records no event. \| enforced at Session::handle (session.rs:354) \| boundary test stop_disconnects_every_world_and_refuses_connect, weakened by the plan (inv-2)
- invariants: I12: An empty journal has last() == Seq(0) and the first event is Seq(1). \| enforced at journal.rs last_of (line 143) \| boundary test journal unit tests and wait_on_world_that_is_down
- invariants: Plan phase: no enforcement point is a script yet, so no edge probe ran.
- invariants: A frame that went through the depth and rate checks of Session::handle would take a rate token; the spec's 'SHALL NOT be a command' covers it, but no test checks the limiter count after a wait.
- invariants: Task 2.2 says the four fake_game_* tests from before; fake.rs holds five (accepts_login, refuses_wrong_password, answers_line, drops_on_request, says_a_line).
- spec-lint: prior.json is an empty list: first round.
- spec-lint: All scenarios in specs/world-sessions/spec.md have WHEN or GIVEN and THEN lines; no malformed-scenario finding.
- spec-lint: Build targets in tasks.md resolve: mise run check exists in mise.toml, and the mu-core package exists in crates/mu-core/Cargo.toml.
- spec-lint: No BREAKING marker in proposal.md or in the delta spec, so no breaking-mismatch finding.
- spec-lint: design.md has no open-questions section, so no assumed-answer finding.
- spec-lint: Scenarios with a THEN that names a status, a value, a journal state or a log line were judged observable; no unverifiable finding was raised.
- test-coverage: The change has no BREAKING requirement.
- test-coverage gap: design.md not read

## Ledger

| id | first | last | severity | scope | location | disposition | basis | override | claim |
|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 1 | blocker | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: The fake world stands in for a game"],"before and after the login. A mark SHALL NOT count as a line that the game got."] |  |  |  | The fake-world requirement says the fake game answers a mark before the login and never counts a mark as a received line; no decision in design.md produces either rule. Only a risk bullet says the fake game splits lines itself. |
| R-2 | 1 | 1 | blocker | deviates-design | ["artifact",["tasks.md","4. No sleep in a test"],"In `tests/portal.rs`, remove the call from `refused_line_break`"] |  |  |  | Asserting at once in refused_line_break makes its game-side check vacuous: the fake game records a received line in its own task, so a regression that writes the refused line still passes. |
| R-3 | 1 | 1 | blocker | deviates-design | ["artifact",["tasks.md","4. No sleep in a test"],"from `stop_disconnects_every_world_and_refuses_connect`, so that each asserts right after the refusal"] |  |  |  | Asserting at once in stop_disconnects_every_world_and_refuses_connect makes its no-event check vacuous: a Connected event comes only after the session polls the finished connect, after the reply. |
| R-4 | 1 | 1 | should-fix | in-change | ["artifact",["proposal.md","Why"],"The `review-lint` gate `time-based-test` reports the sleep on every run."] |  |  |  | The proposal says the time-based-test gate reports the sleep of settle on every run, but the gate reads only the added lines of a diff, so it reports settle only on a diff that adds it. |
| R-5 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"\| No test in the workspace sleeps \| `review-lint`, gate `time-based-test`, with no accepted finding. \|"] |  |  |  | The enforcement table says the time-based-test gate enforces that no test in the workspace sleeps, but the gate sees only added lines of the diff, so it cannot see a sleep that the diff does not add. |
| R-6 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Risks / Trade-offs"],"That bound passes only when nothing can run on the paused clock. The test then fails with a clear panic."] |  |  |  | The mitigation of the endless-loop risk says the BOUND timeout then fails the test, but on the paused clock the timer fires only when the runtime has no work, and an endless loop always has work, so the test hangs. |
| R-7 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Context"],"The `select!` in `session::run` and the fake game in `fake.rs` are the only places that wait."] |  |  |  | The context says the select! of session::run and the fake game are the only places that wait, but run_server_handlers waits on its subscription and a held FakeTransport connect waits on its gate. |
| R-8 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"In `fake.rs`, make the fake game read bytes and split the lines itself."] |  |  |  | The fake game gets its own byte-to-line splitter that must also find a mark in any split, which LineFramer::feed already does for lines and which task 2.1 teaches it for marks. The design names neither LineFramer as the alternative nor a reason. |
| R-9 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"`LineFramer` gives `IAC DO TIMING-MARK`. It reads `IAC WILL TIMING-MARK` and `IAC WONT TIMING-MARK` as `Inbound::Mark`, in any split of reads."] |  |  |  | Tasks 2.1 and 2.2 both write and read the same telnet bytes, IAC DO TIMING-MARK and IAC WILL TIMING-MARK, in framer.rs and in fake.rs, and neither the design nor the tasks name one place for the codes. |
| R-10 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"If the connection goes down, the session answers every frame that waits."] |  |  |  | Frames that wait for a mark must be answered when the connection goes down; no task names a test of that case, and four paths reach Session::down. |
| R-11 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"If the connect failed, it answers them at once."] |  |  |  | Frames in the connect list must be answered when the connect fails; only the success path has a test (wait_outlasts_connect). |
| R-12 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","3. The frame and the wait"],"Each `Inbound::Mark` answers the first frame in the list."] |  |  |  | An Inbound::Mark with no waiting frame (the empty-list case) has no stated behavior and no test; a Vec::remove(0) or unwrap there panics the session task. |
| R-13 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A control frame comes after the work before it"],"**AND** neither line holds a byte of the mark"] |  |  |  | The rule that no line holds a byte of the mark has no test at the MAX_LINE cut; the property test stops at 200 bytes, so a held IAC that fills the line to MAX_LINE is never tried. |
| R-14 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"It learns these two answers only."] |  |  |  | The framer must hold IAC across reads and give the bytes back when they do not form a mark; no rule or test covers a near miss such as IAC WILL ECHO, IAC IAC or IAC then LF. |
| R-15 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Progress of the server handlers through a watch channel"],"A wait for the server handlers is `wait_for(\|seq\| *seq >= mark)`."] |  |  |  | The wait needs the published seq to start at or above every event before the subscription, but the design names no initial value; with Seq::default() a journal that holds events at start makes wait_for never return. |
| R-16 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Risks / Trade-offs"],"The fake game reads bytes and splits the lines itself."] |  |  |  | The fake game moves from BufReader::lines to its own byte splitter, but no task tests a mark or a line split across reads, the edge that a reader of bytes must handle. |
| R-17 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A control frame comes after the work before it"],"The engine SHALL have a control frame. A frame SHALL go on the queue of one world."] |  |  |  | The requirement 'A control frame comes after the work before it' shares only the word 'frame' with its task lines, so no task line matches its subject. |
| R-18 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: The engine can wait until it has no work left"],"A wait of the engine SHALL send rounds. Each round SHALL wait until the server handlers handle the last event"] |  |  |  | The requirement 'The engine can wait until it has no work left' has no task line with two of its distinctive words; task 3.2 names only 'wait' in 'Portal::wait_idle'. |
| R-19 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","1. Documentation"],"Make sure that the design names no fact that the documentation contradicts."] |  |  |  | Task 1.1 asks that the design name no fact the documentation contradicts, but names no command, test, or concrete observable result to check that against. |
| R-20 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"make the fake game read bytes and split the lines itself."] |  |  |  | Task 2.2 rewrites the fake game but only runs mu-core tests. The server-level fake-world scenarios live in mu-cli tests that no task runs. |
| R-21 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"The current `LineFramer` does not parse telnet yet. It learns these two answers only. The telnet-engine change replaces it and keeps the mark."] |  |  |  | Each wait sends a new IAC DO TIMING-MARK, but the telnet-engine decision keeps one RFC 1143 Q state per option, so after the first WILL TM the engine sends no second DO and a policy that does not accept TM answers with DONT. The design says the telnet engine keeps the mark but does not name that TM must stay outside the Q method. |
| R-22 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","The order of a round"],"Send a frame to each world, in the order of the ids, and wait for each answer."] |  |  |  | Step 3 of the round repeats the body of Portal::submit (look up the sender, make a oneshot, send a Request, await the answer) and the id-order loop of to_every_world; only the request variant differs. |
| R-23 | 1 | 1 | suggestion | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"Extend the property test `line_framer_ignores_read_splits` with marks."] |  |  |  | Mark transparency holds over unbounded input: inserting marks at any point of a byte stream without 0xFF must leave the lines unchanged and give one Mark for each insert. |
