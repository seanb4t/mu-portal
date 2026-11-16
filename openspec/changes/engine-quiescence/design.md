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

`Conn::Connecting` keeps a second list, of the frames that come during a connect. When the connect ends and the world is up, those frames move to the list of `Conn::Up`. The session then sends a mark for each one. Each list lives in the state that it belongs to, like the read half and the framer of `Conn::Up`. A frame on a world that is down gets its answer at once. `Session::down` answers each frame in the list of the state that it leaves. Each path that ends a connection or a connect reaches `down`: a failed connect, a failed write, a closed read and a disconnect.

The mark is sent and read outside the state of the telnet options. RFC 860 asks for an answer to each `DO TIMING-MARK`. The Q method of RFC 1143 sends no second `DO` for an option that is already on. So the telnet-engine change keeps `Framer::mark` and `Inbound::Mark` apart from its option table and its accept policy.

Rejected: a marker line in plain text. The game answers it with a line, the journal records that line, and the handlers see it. Several tests count the events after a step, so a visible marker breaks them.

Rejected: a channel to the fake game outside the stream. It cannot order the mark against the bytes that are already in the stream.

### The telnet bytes of the mark live in one place

A new crate-private module, `crates/mu-core/src/telnet.rs`, holds the constants IAC (255), DO (253), WILL (251), WONT (252) and TIMING-MARK (6). The line framer and the fake game both use them. The spike `spikes/telnet-engine/src/lib.rs` defines the same codes. The telnet-engine change grows `telnet.rs` into the engine, so that module stays the one home of the codes.

`LineFramer` does not parse telnet yet. It learns the mark only. A sequence that starts with IAC and is not a mark stays in the line as data, as today. The framer holds up to two bytes of a mark at the end of a read until the next read shows whether a mark follows. Bytes that it gives back to the line go through the same cut as any other byte, so no line is longer than `MAX_LINE`. When a mark comes after a full line, the framer gives the line first. So a cut at `MAX_LINE` never holds a byte of the mark.

### The fake game reads through the line framer

The fake game reads bytes and feeds them to a `LineFramer` for the game side. That framer reads `IAC DO TIMING-MARK` as `Inbound::Mark` and gives the lines as before. `LineFramer::new()` stays the client side, which ends a mark on WILL or WONT. A crate-private named constructor, `LineFramer::game()`, gives the game side, which ends a mark on DO. The crate adds a variant through a named method, not an argument of `new`. So the split of the lines and the search for a mark in any split of reads have one implementation.

The fake game answers each `Inbound::Mark` with `IAC WILL TIMING-MARK`, in the order of the framer output. It answers before and after the login, and it does not check the login state for a mark. A mark does not go into the lines that the game got, so `FakeGame::lines` and `wait_for_lines` count the same lines as before.

### Progress of the server handlers through a watch channel

`Portal::start` makes a `watch::channel(journal.last())` next to the subscription of the server handlers. The subscription starts at `journal.last().next()`, so each event up to `journal.last()` is already handled from the view of the wait. `run_server_handlers` sends `event.seq` after `run_handlers` returns for each event. A wait for the server handlers is `wait_for(|seq| *seq >= mark)`. With no server handlers, there is no task and no channel, and the wait skips this step.

### The order of a round

`Portal::wait_idle` repeats this round:

1. Read `mark = journal.last()`.
2. Wait until the server handlers handled `mark`. Their commands for events up to `mark` are then on the queues.
3. Send a frame to each world, in the order of the ids, and wait for each answer.
4. If `journal.last()` is still `mark`, return. Otherwise start the next round.

Step 2 comes before step 3 so that each frame goes on a queue after the commands of the server handlers. In the other order, a command of a server handler can go on a queue after the frame. The round then sees no new event yet, and the wait returns too early.

A command of a world handler for another world is safe in any order. If it reaches the other queue after the frame, it adds an event later, and the next round sees it.

`Portal::submit` and step 3 share one private method that sends a `Request` to a world and waits for its answer. The kind of request, a command or a frame, is its parameter. `to_every_world` takes a function from `WorldId` to that kind, so `stop`, `connect_all` and step 3 share one loop in the order of the ids.

### The test helper

`settle` goes. `tests/support/mod.rs` gets `idle(&portal)`, which runs `portal.wait_idle()` under `tokio::time::timeout(BOUND, ...)`, like `wait_event` and `wait_state`.

The two refusal tests in `tests/portal.rs` also call `idle` before they assert. The reply of the session shows that it handled the request. It does not show that the fake game read every byte, and it does not show that a connect that a fault started is over. The mark covers the first, and a frame during a connect covers the second.

### Enforcement of each rule

| Rule | Enforced by |
|---|---|
| The answer of a frame comes after each request before it | The tests `wait_covers_game_echo` and `wait_covers_server_handler_loop`, and the loop tests in `tests/scripts.rs`. |
| A frame records no event and sends no line | The test `wait_adds_nothing`. |
| A drop or a failed connect answers the frames that wait | The tests `wait_survives_drop`, `wait_survives_failed_mark_write` and `wait_survives_failed_connect`. |
| Each list keeps more than one frame in order | The test `two_waits_on_one_world`. |
| A mark that no frame waits for changes nothing | The test `stray_mark_is_ignored`. |
| The progress of the server handlers starts at the last event of the journal | The test `wait_on_journal_with_events_at_start`. |
| The fake game answers the mark in order and does not count it | The unit test `fake_game_answers_mark_in_order` in `fake.rs`. |
| The line framer reads a mark in any split and keeps the lines | The unit tests and property tests of task 2.1. |
| No test of `mu-core` sleeps | Task 4.1 searches `crates/mu-core/tests` and `crates/mu-core/src` for `sleep(`. `review-lint`, gate `time-based-test`, sees only the added lines of a diff. The sleeps in `mu-client` and `mu-cli` are outside this change. |

## Risks / Trade-offs

- [A real game ignores `IAC DO TIMING-MARK`, so a wait on it never returns.] → Only tests call `wait_idle`, and its documentation says so. The telnet-engine change decides the behavior with real games.
- [A handler loop with no bound makes the rounds go on forever. The paused clock moves only when no task can run. So the `BOUND` timer cannot fire.] → Each loop of the engine stops at the depth limit or at the rate limit. A test that hangs fails after 60 seconds by `terminate-after` in `.config/nextest.toml`.
- [The fake game changes from `BufReader::lines` to the line framer.] → Task 2.2 runs the `mu-cli` tests of the fake world. They use the fake game through the server.

## Checked facts

Task 1.1 fills this table: each fact about RFC 860, RFC 854, RFC 1143 and `tokio` that the design uses, with its source section.

| Fact | Source |
|---|---|
