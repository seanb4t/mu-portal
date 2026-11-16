---
verdict: approve-with-notes
phase: plan
reviewed_at: 2026-11-16T21:10:44+00:00
head: d28a104623ce37f052a9d455c79e0b571fbf5959
round: 2
next_id: 34
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
| R-24 | approach | should-fix | in-change | The enforcement row says task 4.1's search of crates/*/tests and crates/*/src for sleep( enforces that no test in the workspace sleeps, but at head that search hits a sleeping unit test in mu-client and two production sleeps that the change does not touch, so the rule is false and the check of task 4.1 cannot pass. | design.md > Enforcement of each rule — "\| No test in the workspace sleeps \| Task 4.1 searches `crates/*/tests` and `crates/*/src` for `sleep(`." | Narrow the rule and the search to the tests of mu-core (crates/mu-core/tests and the test modules of crates/mu-core/src), or name the mu-client test sleep as out of scope with its issue. |  |  |
| R-25 | approach | should-fix | in-change | Task 2.2 says four fake_game_* tests exist before the change; fake.rs at head has five. | tasks.md > 2. The mark on the wire — "passes with the four `fake_game_*` tests from before" | Say five, or name the tests: fake_game_accepts_login, fake_game_refuses_wrong_password, fake_game_answers_line, fake_game_drops_on_request, fake_game_says_a_line. |  |  |
| R-26 | fit | should-fix | in-change | The side goes into the constructor of the public LineFramer for the in-crate fake game; the crate keeps new() free of arguments and adds a variant through a named method, and the design names neither. | design.md > The fake game reads through the line framer — "The side is a parameter of the constructor: the client side ends a mark on WILL or WONT, and the game side ends it on DO" | Keep LineFramer::new() as the client side and add a crate-private named constructor for the game side, such as LineFramer::game(); state that in this decision and in task 2.1. |  |  |
| R-27 | fit | should-fix | in-change | The two lists of waiting frames sit beside the connection state, but the session keeps the data of each connection state inside its Conn variant, and the design names neither that pattern nor a reason. | design.md > TIMING-MARK for the round trip through the game — "A frame that comes during a connect waits in a second list." | Put the connect list in Conn::Connecting and the mark list in Conn::Up, and let down() answer the list of the state it leaves; or name in this decision why the lists stay outside Conn. |  |  |
| R-10 | invariants | should-fix | in-change | A failed write of the frame's own mark must still answer that frame, so the frame must enter the mark list before the write reaches down. No test forces this path. wait_survives_drop reaches the failed write or the closed read at random, by the select! order. | design.md > TIMING-MARK for the round trip through the game — "Each path that ends a connection or a connect reaches `down`: a failed connect, a failed write, a closed read and a disc" | In task 3.1, state that the frame enters the mark list before the mark is written, or that a failed mark write answers it. In task 3.3, add a test where the mark write fails for certain, for example a transport whose stream refuses writes after the login. Assert that the wait returns and that the journal holds a disconnect event. | regression |  |
| R-28 | invariants | should-fix | in-change | The plan adds a path that gives held IAC bytes back to the line, but the buffer bound is checked with == after each single push. Bytes given back at MAX_LINE - 1 skip the cut, so the line can grow without bound. No planned test feeds a non-mark IAC sequence at the bound. | design.md > The telnet bytes of the mark live in one place — "A sequence that starts with IAC and is not a mark stays in the line as data, as today." | In task 2.1, add `line_framer_near_miss_at_the_bound`: `MAX_LINE - 1` data bytes, then `IAC WILL 5` and an LF, fed whole and split after IAC and after WILL. Assert that every line is at most MAX_LINE bytes and that the bytes come out in order. Also split the mark test after `IAC WILL`, where the framer holds two bytes. |  | severity? blocker 0.79 |
| R-29 | invariants | should-fix | in-change | The named enforcement of 'No test in the workspace sleeps' cannot pass at head. The search of crates/*/src finds production backoff sleeps in mu-client and mu-cli, and a mu-client unit test that sleeps, all outside the change. So the rule has no working enforcement point. | design.md > Enforcement of each rule — "Task 4.1 searches `crates/*/tests` and `crates/*/src` for `sleep(`." | Narrow the rule to the tests of mu-core, and bound the search of task 4.1 to `crates/mu-core/tests` and `crates/mu-core/src` so that it can pass. Update the row of the enforcement table to match. |  |  |
| R-30 | invariants | should-fix | in-change | The mark list and the connect list are FIFO lists, but every planned test has at most one waiting frame, because a round waits for each answer in turn. A single slot that overwrites an earlier frame passes every planned test. | design.md > TIMING-MARK for the round trip through the game — "The session keeps a list of the frames that wait for a mark, in order. Each answer of the game takes the first one." | In task 3.3, add `two_waits_on_one_world`: two `wait_idle` calls joined with `tokio::join!`, once on a connected world and once during a held connect that then opens. Assert that both return. |  |  |
| R-31 | spec-lint | should-fix | in-change | The THEN of the scenario 'A wait survives a failed connect' names the disconnect event by a content ('names the failed connect') with no value or field to check. | specs/world-sessions/spec.md > Scenario: A wait survives a failed connect — "the journal already holds a disconnect event of `alpha/aria` that names the failed connect" | Name the observable of the event: the exact text or field that shows the connect failed, for example the event text. |  |  |
| R-22 | fit | suggestion | in-change | Step 3 repeats the id-order loop of to_every_world, which takes only a Command; the design shares the body of submit but not that loop. | design.md > The order of a round — "3. Send a frame to each world, in the order of the ids, and wait for each answer." | Make the kind of request the parameter of to_every_world, a function from WorldId to the request kind, so that stop, connect_all and step 3 share one loop. | regression |  |
| R-32 | fit | suggestion | in-change | framer.rs gets crate-private IAC, DO, WILL and WONT constants, but the telnet engine of the next change already defines the same codes as public constants, so that change must move one set. | design.md > The telnet bytes of the mark live in one place — "`framer.rs` holds crate-private constants for IAC (255), DO (253), WILL (251), WONT (252) and TIMING-MARK (6)." | In this decision, name the codes of the telnet-engine spike and state which module the telnet-engine change keeps as the one home of the codes, so that the next change imports them instead of moving them. |  | severity? should-fix 0.74 |
| R-33 | invariants | suggestion | in-change | No line is longer than MAX_LINE, and the data bytes keep their order, over any stream and any split. This size bound holds over an unbounded domain, but the planned property tests stop at 200 bytes, far below the bound. | tasks.md > 2. The mark on the wire — "in `line_framer_ignores_read_splits`, a generator that favors the bytes 255, 251, 252, 253 and 6" | With `proptest`, add a property in the style of `line_framer_ignores_read_splits` in crates/mu-core/src/framer.rs. Use a prefix of `MAX_LINE - 4` data bytes and a tail of 0 to 12 favored bytes, cut at any points. Assert that each line is at most MAX_LINE bytes and that no line holds the bytes of a whole mark. |  |  |

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

## Judge

judge: 24 questions, 0.001 USD

## Coverage matrix

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

## Verification runs

None.

## Dropped

None.

## Lens notes

- approach: Every requirement of the delta spec has a decision in design.md that produces it; every scenario THEN traced to a step the design creates.
- approach: A wait outlasts a connect holds whatever the order of the connect-list marks and the login line: the Connected event always forces a second round, whose mark goes after the login line.
- approach: Checked at head: run_handlers routes inside Session::record; the server-handler subscription starts at journal.last().next(); every path that ends a connection reaches Session::down; DEPTH_LIMIT is 8; terminate-after 3 x 20s in .config/nextest.toml; tokio 1.54.1 num_alive_tasks counts tasks until they exit and global_queue_depth counts only the injection queue, so the rejection of a runtime idle check holds.
- approach: The decision on the telnet bytes says the framer holds an IAC at the end of a read; a mark split after IAC WILL or IAC DO needs two held bytes. The split tests of task 2.1 force the right behavior, so this is not raised as a finding.
- approach: All prior rows R-1 to R-23 marked fixed were checked; none of their defects is still present at head.
- fit: The library rows of openspec/config.yaml name no row for telnet codes or for test waits; the watch channel of the progress and the idle helper follow the existing watch and BOUND patterns of fake.rs and tests/support/mod.rs.
- fit: Outside this lens: the check of task 4.1, rg for sleep( in crates/*/src, matches production backoff sleeps in crates/mu-client/src/lib.rs lines 242 and 286 and crates/mu-cli/src/client.rs line 92.
- invariants: I1: A frame is answered only after each request before it on the queue \| enforced at the FIFO mpsc queue of each world (portal.rs:94) \| boundary test wait_covers_game_echo, wait_covers_server_handler_loop
- invariants: I2: A frame records no event and sends no line \| enforced at none yet (task 3.1) \| boundary test wait_adds_nothing
- invariants: I3: A mark with an empty mark list changes nothing \| enforced at task 3.1 \| boundary test stray_mark_is_ignored (empty case)
- invariants: I4: Each frame that waits is answered on down, for each of four paths \| enforced at Session::down (task 3.1) \| boundary test wait_survives_drop (closed read, or a failed write at random), wait_survives_failed_connect. No test for a certain failed mark write or a Disconnect command (inv-2)
- invariants: I5: The mark list and the connect list keep FIFO order with more than one frame \| enforced at task 3.1 \| boundary test none (inv-4)
- invariants: I6: Published progress starts at journal.last() at start and never decreases \| enforced at watch::channel(journal.last()) and the send after each event (task 3.2), with seq monotonic across the Lagged resubscribe from `next` (portal.rs:272) \| boundary test wait_on_journal_with_events_at_start (equality side), wait_covers_server_handler_loop (one past)
- invariants: I7: The seq is published for every event, a ScriptError included, though run_handlers returns early for it (session.rs:72) \| enforced at task 3.2 \| boundary test wait_covers_server_handler_loop, whose last event is a script error
- invariants: I8: The depth limit stops a handler loop: depth 8 is accepted and depth 9 is refused \| enforced at session.rs:319 \| boundary test wait_covers_server_handler_loop asserts 8 events and one script error (both sides)
- invariants: I9: No line holds a byte of a mark, at any split and at the MAX_LINE cut \| enforced at LineFramer (task 2.1) \| boundary test line_framer_mark_at_the_bound (split after IAC only), property tests of task 2.1
- invariants: I10: The line buffer never exceeds MAX_LINE \| enforced at framer.rs:68 with == per pushed byte \| boundary test none for bytes given back from a held IAC (inv-1)
- invariants: I11: The fake game answers each DO TIMING-MARK once, in order after the lines before it, and never counts a mark as a line \| enforced at fake.rs serve (task 2.2) \| boundary test fake_game_answers_mark_in_order, cut at every byte, with a mark before the login
- invariants: I12: No test in the workspace sleeps \| enforced at the task 4.1 search, which cannot pass at head \| boundary test none (inv-3)
- invariants: No probe ran: every enforcement point of this change is Rust code that the plan has not written yet, and the only script, the task 4.1 search, was read against the matches at head.
- invariants: The design gives no behavior for a wait whose progress sender is gone. run_server_handlers returns when a resubscribe fails (portal.rs:272-273), and watch::Receiver::wait_for then returns an error. A wait_idle that unwraps it panics. In practice the resubscribe from `next` cannot fail, so this is a note only.
- invariants: Task 2.2 names 'the four fake_game_* tests from before', but fake.rs holds five: accepts_login, refuses_wrong_password, answers_line, drops_on_request and says_a_line. This is left to the coverage lens.
- invariants: Follow-up bound: the base f269a3b is origin/main, and head d28a104 adds every artifact of the change, so all artifact text is inside the bound.
- spec-lint: Follow-up round: diff-files.txt lists all six change artifacts, so the whole change is inside the bound.
- spec-lint: All 23 prior.json rows (R-1 to R-23) were checked against the text at head. None is still present in the form class of this lens, so none is raised again. R-19 is reworded but still names a file and a table, so it is not raised.
- spec-lint: Scenario shape is complete: every scenario has a GIVEN or WHEN and a THEN, and every requirement has at least one scenario.
- spec-lint: No BREAKING marker in proposal.md or in the delta spec, so no breaking-mismatch.
- spec-lint: Every requirement heading has a task line with two distinctive words: 'control frame', 'frame ... waits ... down', 'engine wait until no work is left', 'fake game'.
- spec-lint: Build targets exist: mise run check (mise.toml), cargo nextest run -p mu-core, -p mu-cli --test worlds (crates/mu-cli/tests/worlds.rs with the three named tests), and review_lint.py under ~/.agents/plugins/sb/skills/review-lint/scripts/.
- spec-lint: design.md has no open-questions section, so no assumed-answer.
- test-coverage: Follow-up round: every prior row is fixed; no raised-again defect found. No BREAKING requirement in the delta.

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
| R-10 | 1 | 2 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"If the connection goes down, the session answers every frame that waits."] |  | regressed from fixed d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Frames that wait for a mark must be answered when the connection goes down; no task names a test of that case, and four paths reach Session::down. |
| R-11 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"If the connect failed, it answers them at once."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Frames in the connect list must be answered when the connect fails; only the success path has a test (wait_outlasts_connect). |
| R-12 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","3. The frame and the wait"],"Each `Inbound::Mark` answers the first frame in the list."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | An Inbound::Mark with no waiting frame (the empty-list case) has no stated behavior and no test; a Vec::remove(0) or unwrap there panics the session task. |
| R-13 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A control frame comes after the work before it"],"**AND** neither line holds a byte of the mark"] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The rule that no line holds a byte of the mark has no test at the MAX_LINE cut; the property test stops at 200 bytes, so a held IAC that fills the line to MAX_LINE is never tried. |
| R-14 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"It learns these two answers only."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The framer must hold IAC across reads and give the bytes back when they do not form a mark; no rule or test covers a near miss such as IAC WILL ECHO, IAC IAC or IAC then LF. |
| R-15 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Progress of the server handlers through a watch channel"],"A wait for the server handlers is `wait_for(\|seq\| *seq >= mark)`."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The wait needs the published seq to start at or above every event before the subscription, but the design names no initial value; with Seq::default() a journal that holds events at start makes wait_for never return. |
| R-16 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Risks / Trade-offs"],"The fake game reads bytes and splits the lines itself."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The fake game moves from BufReader::lines to its own byte splitter, but no task tests a mark or a line split across reads, the edge that a reader of bytes must handle. |
| R-17 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A control frame comes after the work before it"],"The engine SHALL have a control frame. A frame SHALL go on the queue of one world."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The requirement 'A control frame comes after the work before it' shares only the word 'frame' with its task lines, so no task line matches its subject. |
| R-18 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: The engine can wait until it has no work left"],"A wait of the engine SHALL send rounds. Each round SHALL wait until the server handlers handle the last event"] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | The requirement 'The engine can wait until it has no work left' has no task line with two of its distinctive words; task 3.2 names only 'wait' in 'Portal::wait_idle'. |
| R-19 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","1. Documentation"],"Make sure that the design names no fact that the documentation contradicts."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Task 1.1 asks that the design name no fact the documentation contradicts, but names no command, test, or concrete observable result to check that against. |
| R-20 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"make the fake game read bytes and split the lines itself."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Task 2.2 rewrites the fake game but only runs mu-core tests. The server-level fake-world scenarios live in mu-cli tests that no task runs. |
| R-21 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"The current `LineFramer` does not parse telnet yet. It learns these two answers only. The telnet-engine change replaces it and keeps the mark."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Each wait sends a new IAC DO TIMING-MARK, but the telnet-engine decision keeps one RFC 1143 Q state per option, so after the first WILL TM the engine sends no second DO and a policy that does not accept TM answers with DONT. The design says the telnet engine keeps the mark but does not name that TM must stay outside the Q method. |
| R-22 | 1 | 2 | suggestion | in-change | ["artifact",["design.md","The order of a round"],"Send a frame to each world, in the order of the ids, and wait for each answer."] |  | regressed from fixed d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Step 3 of the round repeats the body of Portal::submit (look up the sender, make a oneshot, send a Request, await the answer) and the id-order loop of to_every_world; only the request variant differs. |
| R-23 | 1 | 1 | suggestion | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"Extend the property test `line_framer_ignores_read_splits` with marks."] | fixed | d28a104623ce37f052a9d455c79e0b571fbf5959 |  | Mark transparency holds over unbounded input: inserting marks at any point of a byte stream without 0xFF must leave the lines unchanged and give one Mark for each insert. |
| R-24 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"\| No test in the workspace sleeps \| Task 4.1 searches `crates/*/tests` and `crates/*/src` for `sleep(`."] |  |  |  | The enforcement row says task 4.1's search of crates/*/tests and crates/*/src for sleep( enforces that no test in the workspace sleeps, but at head that search hits a sleeping unit test in mu-client and two production sleeps that the change does not touch, so the rule is false and the check of task 4.1 cannot pass. |
| R-25 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"passes with the four `fake_game_*` tests from before"] |  |  |  | Task 2.2 says four fake_game_* tests exist before the change; fake.rs at head has five. |
| R-26 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The fake game reads through the line framer"],"The side is a parameter of the constructor: the client side ends a mark on WILL or WONT, and the game side ends it on DO."] |  |  |  | The side goes into the constructor of the public LineFramer for the in-crate fake game; the crate keeps new() free of arguments and adds a variant through a named method, and the design names neither. |
| R-27 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"A frame that comes during a connect waits in a second list."] |  |  |  | The two lists of waiting frames sit beside the connection state, but the session keeps the data of each connection state inside its Conn variant, and the design names neither that pattern nor a reason. |
| R-28 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The telnet bytes of the mark live in one place"],"A sequence that starts with IAC and is not a mark stays in the line as data, as today."] |  |  |  | The plan adds a path that gives held IAC bytes back to the line, but the buffer bound is checked with == after each single push. Bytes given back at MAX_LINE - 1 skip the cut, so the line can grow without bound. No planned test feeds a non-mark IAC sequence at the bound. |
| R-29 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"Task 4.1 searches `crates/*/tests` and `crates/*/src` for `sleep(`."] |  |  |  | The named enforcement of 'No test in the workspace sleeps' cannot pass at head. The search of crates/*/src finds production backoff sleeps in mu-client and mu-cli, and a mu-client unit test that sleeps, all outside the change. So the rule has no working enforcement point. |
| R-30 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","TIMING-MARK for the round trip through the game"],"The session keeps a list of the frames that wait for a mark, in order. Each answer of the game takes the first one."] |  |  |  | The mark list and the connect list are FIFO lists, but every planned test has at most one waiting frame, because a round waits for each answer in turn. A single slot that overwrites an earlier frame passes every planned test. |
| R-31 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: A wait survives a failed connect"],"the journal already holds a disconnect event of `alpha/aria` that names the failed connect"] |  |  |  | The THEN of the scenario 'A wait survives a failed connect' names the disconnect event by a content ('names the failed connect') with no value or field to check. |
| R-32 | 2 | 2 | suggestion | in-change | ["artifact",["design.md","The telnet bytes of the mark live in one place"],"`framer.rs` holds crate-private constants for IAC (255), DO (253), WILL (251), WONT (252) and TIMING-MARK (6)."] |  |  |  | framer.rs gets crate-private IAC, DO, WILL and WONT constants, but the telnet engine of the next change already defines the same codes as public constants, so that change must move one set. |
| R-33 | 2 | 2 | suggestion | in-change | ["artifact",["tasks.md","2. The mark on the wire"],"in `line_framer_ignores_read_splits`, a generator that favors the bytes 255, 251, 252, 253 and 6"] |  |  |  | No line is longer than MAX_LINE, and the data bytes keep their order, over any stream and any split. This size bound holds over an unbounded domain, but the planned property tests stop at 200 bytes, far below the bound. |
