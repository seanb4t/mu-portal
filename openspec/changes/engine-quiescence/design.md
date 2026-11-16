# Design

## Context

See proposal.md for the reason. Work of the engine can wait in three places:

- The queue of each world. `Portal::start` gives each world an unbounded `mpsc` channel of `Request`, and one task per world (`session::run`) handles the requests in order. A handler of a world routes its commands into the queues while the session records the event, in `run_handlers`, before `record` returns.
- The stream to the game. The session task reads it in the same `select!` as the queue. That macro picks a ready branch at random, so a request can run while bytes of the game wait unread.
- The task of the server handlers (`run_server_handlers`). It reads the journal through a subscription, so it can lag behind the journal.

The engine starts no timer. Outside the engine, a held `FakeTransport` waits on its gate, and the fake game waits for bytes and for lines from `FakeGame::say`.

Every test on the paused clock runs on a `current_thread` runtime. So `append` and `route` in `Session::record` run with no other task between them.

## Goals / Non-Goals

**Goals:**

- A wait whose end a test can explain with observable state: a full round that adds no event.
- No wait in a test that rests on the clock.

**Non-Goals:**

- A wait that is safe on a real game. A game that does not answer TIMING-MARK makes the wait hang. Only tests call the wait.
- A wait for a gate that the test holds. A wait during a held connect returns only after the test opens the gate or drops it.
- An order between a line from `FakeGame::say` and a mark. The fake game can answer the mark first. No test depends on that order.

## Decisions

### A frame on the queue, not a sleep

The frame is a new variant beside the command request on the queue of each world. The queue is first in, first out, so the answer of a frame proves that the session handled each request before it. A sleep on the paused clock proves only that the runtime went idle.

Rejected: a count of `yield_now` calls. It is the same guess as the sleep, with a number of steps in place of a time.

Rejected: an idle check on the tokio runtime. Tokio has no stable call that reports that every task waits. `RuntimeMetrics::num_alive_tasks` counts tasks that wait as alive.

### TIMING-MARK for the round trip through the game

When the world is up, the session writes `IAC DO TIMING-MARK` (RFC 860) to the game. When the framer reads `IAC WILL TIMING-MARK` or `IAC WONT TIMING-MARK`, the session answers the frame. The game answers in the order it reads. So the session already read and recorded each line that the game sent before the mark. Each handler command of those lines is also already on a queue.

`Conn::Up` keeps the list of the frames that wait for a mark, in order. Each answer of the game takes the first one. A frame goes into the list before the session writes its mark. So a failed write of that mark reaches `down` with the frame already in the list. A mark that no frame waits for changes nothing. The session goes on with other requests while a frame waits, so a frame never blocks the queue.

`Conn::Connecting` keeps a second list, of the frames that come during a connect. When the connect ends and the world is up, those frames move to the list of `Conn::Up`. The session then sends a mark for each one. Each list lives in the state that it belongs to, like the read half and the framer of `Conn::Up`. A frame on a world that is down gets its answer at once. A framer whose `mark` gives no bytes has no mark, so a frame on its world also gets its answer at once, after the requests before it. `Session::down` answers each frame in the list of the state that it leaves. Each path that ends a connection or a connect reaches `down`: a failed connect, a failed write, a closed read and a disconnect.

The mark is sent and read outside the state of the telnet options. RFC 860 asks for an answer to each `DO TIMING-MARK`. The Q method of RFC 1143 sends no second `DO` for an option that is already on. So the telnet-engine change keeps `Framer::mark` and `Inbound::Mark` apart from its option table and its accept policy.

Rejected: a marker line in plain text. The game answers it with a line, the journal records that line, and the handlers see it. Several tests count the events after a step, so a visible marker breaks them.

Rejected: a channel to the fake game outside the stream. It cannot order the mark against the bytes that are already in the stream.

### The telnet bytes of the mark live in one place

A new crate-private module, `crates/mu-core/src/telnet.rs`, holds the codes in the layout of the spike `spikes/telnet-engine/src/lib.rs`. The command codes IAC (255), DO (253), WILL (251) and WONT (252) are at the top of the module, as in the spike. The option code TIMING-MARK (6) is in the inner module `opt`, where the spike keeps its option codes. The spike has no TIMING-MARK, so the telnet-engine change adds nothing for it. It grows `telnet.rs` into the engine, and that module stays the one home of the codes. The line framer and the fake game both use them.

`LineFramer` does not parse telnet yet. It learns the mark and the escape `IAC IAC` only. It reads `IAC IAC` as one data byte 255, as RFC 854 says, and it looks for a mark only after that. So `IAC IAC WILL TIMING-MARK` gives the data bytes 255, 251 and 6 and no mark, on both sides. The escaped byte reaches the line as one byte 255, not as two. Any other sequence that starts with IAC and is not a mark stays in the line as data, as today. The framer holds up to two bytes of a mark, or one IAC, at the end of a read until the next read shows what follows. Bytes that it gives back to the line go through the same cut as any other byte, so no line is longer than `MAX_LINE`. When a mark comes after a full line, the framer gives the line first. So a cut at `MAX_LINE` never holds a byte of the mark.

### The fake game reads through the line framer

The fake game reads bytes and feeds them to a `LineFramer` for the game side, which gives the lines as before. `LineFramer::new()` stays the client side. It gives `Inbound::Mark` for `IAC WILL TIMING-MARK` and `IAC WONT TIMING-MARK`. A crate-private named constructor, `LineFramer::game()`, gives the game side. It gives `Inbound::Reply` with `IAC WILL TIMING-MARK` for `IAC DO TIMING-MARK`, like any other negotiation reply. So `Inbound::Mark` keeps one meaning: the game answered a mark of the client. The game side reads `IAC WILL TIMING-MARK` and `IAC WONT TIMING-MARK` as data, like any other IAC sequence. The crate adds a variant through a named method, not an argument of `new`. So the split of the lines and the search for a mark in any split of reads have one implementation.

The fake game writes the bytes of each `Inbound::Reply`, in the order of the framer output. It answers before and after the login, and it does not check the login state for a mark. A mark does not go into the lines that the game got, so `FakeGame::lines` and `wait_for_lines` count the same lines as before.

### Progress of the server handlers through a watch channel

`Portal::start` makes a `watch::channel(journal.last())` next to the subscription of the server handlers. The subscription starts at `journal.last().next()`, so each event up to `journal.last()` is already handled from the view of the wait. `run_server_handlers` stores `event.seq` with `send_replace` after `run_handlers` returns for each event. The method `send` drops the value when no receiver exists, and `send_replace` does not. A wait for the server handlers is `wait_for(|seq| *seq >= mark)`. With no server handlers, there is no task and no channel, and the wait skips this step.

### The order of a round

`Portal::wait_idle` repeats this round:

1. Read `mark = journal.last()`.
2. Wait until the server handlers handled `mark`. Their commands for events up to `mark` are then on the queues.
3. Send a frame to each world, in the order of the ids, and wait for each answer.
4. If `journal.last()` is still `mark`, return. Otherwise start the next round.

Step 2 comes before step 3 so that each frame goes on a queue after the commands of the server handlers. In the other order, a command of a server handler can go on a queue after the frame. The round then sees no new event yet, and the wait returns too early.

A command of a world handler for another world is safe in any order. If it reaches the other queue after the frame, it adds an event later, and the next round sees it.

`Portal::submit` and step 3 share one private method that sends a `Request` to a world and waits for its answer. Its parameter is a function from the answer sender to the `Request`. A frame carries a sender of the same `Reply` as a command and gets `Ok(None)`. `to_every_world` takes a function from `WorldId` and the answer sender to the `Request`. So `stop`, `connect_all` and step 3 share one loop in the order of the ids. It returns the result of each world. `stop` and `connect_all` ignore the results, as today. `wait_idle` returns `Result<(), PortalError>`, and a frame whose answer channel closed with no answer makes it fail with `PortalError::Stopped`. So a session that drops a waiting frame, in place of an answer, fails the test that waits.

The list of waiting frames is a small type in `session.rs` over a `VecDeque`. A unit test of that type checks that the answers come in the order of the frames.

### The test helper

`settle` goes. `tests/support/mod.rs` gets `idle(&portal)`, which runs `portal.wait_idle()` under `tokio::time::timeout(BOUND, ...)`, like `wait_event` and `wait_state`.

The two refusal tests in `tests/portal.rs` also call `idle` before they assert. The reply of the session shows that it handled the request. It does not show that the fake game read every byte, and it does not show that a connect that a fault started is over. The mark covers the first, and a frame during a connect covers the second.

### Enforcement of each rule

| Rule | Enforced by |
|---|---|
| The answer of a frame comes after each request before it | The tests `wait_covers_game_echo` and `wait_covers_server_handler_loop`, and the loop tests in `tests/scripts.rs`. |
| A frame records no event and sends no line | The test `wait_adds_nothing`. |
| A drop or a failed connect answers the frames that wait | The tests `wait_survives_drop`, `wait_survives_failed_mark_write` and `wait_survives_failed_connect`. Each asserts that the wait returns `Ok`. A dropped frame gives `Err`. |
| Each list keeps more than one frame, in order | The test `two_waits_on_one_world`, which asserts `Ok` for both waits, and the unit test `waiting_frames_answer_in_order` in `session.rs`. |
| The codes of telnet live only in `telnet.rs` | The test `source_of_mu_core_keeps_its_rules` in `crates/mu-cli/tests/source_rules.rs`, beside `workspace_boundary.rs`. A search that runs once at task time passes a code that a later change adds, and this test runs with each `cargo nextest run`. It reads each Rust file of `src` and `tests` of `mu-core` as tokens, with `support::files_under` of `mu-cli`. It uses `proc-macro2` with the feature `span-locations`, which gives the line of a hit, and `syn`. `mu-cli` already has both as dev-dependencies, so `mu-core` gets no new dependency. The test file sits outside the directories that it scans, so only `src/telnet.rs` is exempt from the rule. It fails with the file and the line for an integer from 251 to 255. The base and the suffix do not matter. It also fails for a byte or a byte string with such a code, and for `u8::MAX`. It finds a 6 in two forms: an integer with the suffix `u8`, as in `6u8`, and an integer right after `u8 =`, as in `const TM: u8 = 6;`. It misses each other 6, such as `[u8; 1] = [6]`, `6 as u8`, `-> u8 { 6 }` and `Some(6)`. An allow list names each accepted literal with its count and a reason. Its one entry is the byte `0xff` in a test of `secret.rs`. The tests of the same file try each form of a code. The list below the table names the forms that the test misses. |
| A mark that no frame waits for changes nothing | The test `stray_mark_is_ignored`. |
| The progress of the server handlers starts at the last event of the journal | The test `wait_on_journal_with_events_at_start`. |
| The fake game answers the mark in order and does not count it | The unit test `fake_game_answers_mark_in_order` in `fake.rs`. |
| The line framer reads a mark in any split and keeps the lines | The unit tests and property tests of task 2.1. |
| The line framer reads `IAC IAC` as one data byte, before it looks for a mark | The unit test `line_framer_reads_escaped_iac`. The property test `line_framer_lines_stay_in_the_bound` counts a mark only after it reads `IAC IAC` as data. |
| No test of `mu-core` sleeps | The same test. It fails with the file and the line for the name `sleep` or `sleep_until` in any file of `src` or `tests` of `mu-core`. `review-lint`, gate `time-based-test`, sees only the added lines of a diff. The sleeps in `mu-client` and `mu-cli` are outside this change. The list below the table names the waits that the test misses. |

The source test is a best-effort check. It does not find these forms:

- A wait by an interval or a timeout, such as `interval(d).tick()` or `timeout(d, pending())`.
- A sleep in a doc test. The scanner reads a doc comment as a string.
- A code of telnet written as `!0u8`, `<u8>::MAX`, `u8::max_value()`, a cast such as `(-1i8) as u8`, or a C string such as `c"\xff"`.

## Risks / Trade-offs

- [A real game ignores `IAC DO TIMING-MARK`, so a wait on it never returns.] → Only tests call `wait_idle`, and its documentation says so. The telnet-engine change decides the behavior with real games.
- [A handler loop with no bound makes the rounds go on forever. The paused clock moves only when no task can run. So the `BOUND` timer cannot fire.] → Each loop of the engine stops at the depth limit or at the rate limit. A test that hangs fails after 60 seconds by `terminate-after` in `.config/nextest.toml`.
- [The fake game changes from `BufReader::lines` to the line framer.] → Task 2.2 runs the `mu-cli` tests of the fake world. They use the fake game through the server.

## Checked facts

Task 1.1 fills this table: each fact about RFC 860, RFC 854, RFC 1143 and `tokio` that the design uses, with its source section.

| Fact | Source |
|---|---|
| TIMING-MARK has the option code 6. | RFC 860, section 1. |
| `IAC DO TIMING-MARK` asks the receiver to send `WILL TIMING-MARK` in the data stream at the appropriate place. | RFC 860, section 2. |
| When the sender of the DO reads `WILL TIMING-MARK`, all data that the other side sent before the mark is delivered to it. The session reads one stream in order, so it has handled each earlier line when it reads the mark. | RFC 860, section 5, the paragraph that starts "When A receives the WILL TIMING-MARK". |
| `IAC WONT TIMING-MARK` refuses to synchronize. The requester still learns that the other side received all earlier data, though not that it processed them. | RFC 860, sections 2 and 4. |
| The default of the option is WONT. RFC 1143 asks each implementation to refuse an option it does not support. So only a silent game makes the wait hang. | RFC 860, section 3, and RFC 1143, section 2, the last rule. |
| RFC 860 describes one answer for one `DO TIMING-MARK`. It has no rule against a second DO, so the design sends one DO for each frame. | RFC 860, sections 2 and 5. |
| The command codes are IAC 255, WILL 251, WONT 252, DO 253 and DONT 254. | RFC 854, "TELNET COMMAND STRUCTURE", the table of commands. |
| A negotiation is three bytes: IAC, the command and the option code. So a mark has three bytes, and a read can end after one or two of them. | RFC 854, "TELNET COMMAND STRUCTURE", first paragraph. |
| The data byte 255 is sent as `IAC IAC`. No other code needs doubling. The framer reads `IAC IAC` as one data byte 255, so an escaped 255 never starts a mark. It keeps each other IAC sequence that is not a mark in the line as data, as it does today. | RFC 854, "TELNET COMMAND STRUCTURE", first paragraph. |
| An implementation must not start a `DO` for an option that is already on. The state machine calls that case an error. | RFC 1143, section 2, and section 7, "If we decide to ask him to enable", state YES. |
| On `WILL` for an option that is already on, the Q method ignores it and sends no answer. The same holds on our side with DO and WILL swapped. | RFC 1143, section 7, "Upon receipt of WILL", and the last paragraph of the example. |
| So one `DO TIMING-MARK` for each frame does not fit the option table of the Q method. A game that treats the mark as an ordinary option ignores a second DO. Real games are left to the telnet-engine change. | RFC 1143, sections 2 and 7. |
| Cargo.lock pins tokio 1.54.1. The workspace enables the features `sync`, `macros` and `time`, and `mu-core` adds `test-util` for tests. | `Cargo.lock`, `Cargo.toml` and `crates/mu-core/Cargo.toml`. The docs and the source of 1.54.1 were read. |
| `watch::channel(init)` makes a sender and a receiver, and `init` is the current value. | tokio 1.54.1 docs, `sync::watch::channel`. |
| `Sender::send_replace` always stores the value and returns the old one, even with no receiver. `Sender::send` stores nothing and returns an error when no receiver exists. | tokio 1.54.1 docs, `sync::watch::Sender::send` and `Sender::send_replace`. Edit: the design now names `send_replace`. |
| `Receiver::wait_for` first calls the closure on the current value, even a value that is already seen. It returns at once when the closure gives true. | tokio 1.54.1 docs, `sync::watch::Receiver::wait_for`. |
| A watch channel keeps only the latest value, so `wait_for` can skip updates. The test `seq >= mark` is safe because `seq` only grows. | tokio 1.54.1 docs, `sync::watch::Receiver::wait_for`. |
| `wait_for` takes `&mut self`, so each wait needs its own receiver. It returns `Err(RecvError)` when the sender is gone and the closure is false for the last value. | tokio 1.54.1 docs, `sync::watch::Receiver::wait_for`, and `watch.rs` in the source. |
| `select!` starts at a pseudo-random branch and takes the first branch that is ready. So with two ready branches, either one can run first. The mode `biased` polls from top to bottom. | tokio 1.54.1 docs, `select!`, the section "Fairness". |
| `UnboundedReceiver::recv` is cancel safe. If the other branch of `select!` wins, no request is lost. | tokio 1.54.1 docs, `sync::mpsc::UnboundedReceiver::recv`, "Cancel safety". |
| An unbounded `mpsc` channel is a queue, and `send` never waits. The source gives each message the next slot with an atomic counter, so the order of receive follows the order of send. | tokio 1.54.1 docs, `sync::mpsc` module, and `sync/mpsc/list.rs` in the source. |
| Pausing the clock needs the `current_thread` runtime. Tasks are scheduled cooperatively and switch only at an `.await`. In `Session::record`, no `.await` sits between `append` and `run_handlers`. | tokio 1.54.1 docs, `time::pause` and the `tokio::task` module, "Tasks are scheduled cooperatively". Repository: `crates/mu-core/src/session.rs`, `record`. |
| A paused clock advances to the next timer only when the runtime has no work to do. A chain of ready tasks never lets the `BOUND` timer fire. | tokio 1.54.1 docs, `time::pause`, the section "Auto-advance". |
| `RuntimeMetrics::num_alive_tasks` counts a task from its spawn to its exit, so a waiting task counts. No stable call of `RuntimeMetrics` reports that every task waits. | tokio 1.54.1 docs, `runtime::RuntimeMetrics::num_alive_tasks`, and `runtime/metrics/runtime.rs` in the source. |
| The runtime promises no order among ready tasks, and `yield_now` does not change that. So a count of `yield_now` calls proves nothing. | tokio 1.54.1 docs, `tokio::runtime`, "Detailed runtime behavior", the paragraph on scheduling order. |
