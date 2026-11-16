# Tasks

## 1. Documentation

- [x] 1.1 Read RFC 860 (TIMING-MARK) and RFC 854 (the codes of IAC, DO, WILL and WONT). Read the part of RFC 1143 on an option that is already on. Read the current documentation of `tokio` with Context7 for `sync::watch` (`channel`, `send_replace`, `wait_for`) and for how `select!` picks a branch. Fill the table "Checked facts" of the design with one row for each fact that the design uses, with its source section. Make sure that the table has a row for each fact. Make sure that each difference from the documentation is an edit of the design that its row names.

## 2. The mark on the wire

Load the `rust-best-practices` skill before this group.

- [x] 2.1 Add the crate-private module `telnet.rs`, with IAC, DO, WILL and WONT at the top and TIMING-MARK in the inner module `opt`. In `framer.rs`, add the variant `Inbound::Mark` and the method `Framer::mark`, which gives the bytes of a mark. Its default body gives no bytes, so the test framer `Shout` still compiles. `LineFramer::new()` is the client side: it gives `IAC DO TIMING-MARK` and reads `IAC WILL TIMING-MARK` and `IAC WONT TIMING-MARK` as `Inbound::Mark`. Add the crate-private constructor `LineFramer::game()`. It gives `Inbound::Reply` with `IAC WILL TIMING-MARK` for `IAC DO TIMING-MARK`. On both sides it reads `IAC IAC` as one data byte 255 before it looks for a mark. So `IAC IAC WILL TIMING-MARK` gives the data bytes 255, 251 and 6, and no mark. Any other IAC sequence stays in the line as data. Add these tests:
  - `line_framer_reads_mark_answer`, for the scenario "The line framer reads the answer to a mark", with both answers
  - `line_framer_reads_escaped_iac`, with `IAC IAC`, `IAC IAC WILL TIMING-MARK` on the client side, `IAC IAC DO TIMING-MARK` on the game side, and a mark after an escaped byte on both sides, each split after every byte
  - `line_framer_keeps_other_iac_bytes`, with `IAC WILL 5` on both sides, `IAC DO TIMING-MARK` on the client side, and `IAC WILL TIMING-MARK` and `IAC WONT TIMING-MARK` on the game side, each split after every byte
  - `line_framer_mark_at_the_bound`: `MAX_LINE - 1` data bytes, a mark and an LF, fed whole and split after IAC and after `IAC WILL`, which give one mark and no line with the byte 255
  - `line_framer_near_miss_at_the_bound`: `MAX_LINE - 1` data bytes, `IAC WILL 5` and an LF, fed whole and split after IAC and after WILL, which give lines of at most `MAX_LINE` bytes with the bytes in order
  - the property test `line_framer_lines_stay_in_the_bound`: a prefix of `MAX_LINE - 4` data bytes and a tail of 0 to 12 of the favored bytes, cut at any points, which give lines of at most `MAX_LINE` bytes and no line with the bytes of a whole mark. The test counts a mark only after it reads `IAC IAC` as one data byte
  - in `line_framer_ignores_read_splits`, a generator that favors the bytes 255, 251, 252, 253 and 6
  - the property test `line_framer_marks_are_transparent`: marks inserted anywhere in a stream with no byte 255 leave the lines unchanged, and give one mark for each insert, after the lines that end before it and before the lines that end after it

  Make sure that `cargo nextest run -p mu-core --locked` passes.
- [x] 2.2 In `fake.rs`, make the fake game read bytes and feed them to a game-side `LineFramer`. It writes the bytes of each `Inbound::Reply`, before and after the login. A mark does not go into the lines that the game got. Add the unit test `fake_game_answers_mark_in_order` for the scenario "The fake world answers a mark in order": `one`, a mark and `two`, written in one piece and cut at every byte, with a case with a mark before the login. Make sure that `cargo nextest run -p mu-core --locked` passes with the five `fake_game_*` tests from before. Make sure that `cargo nextest run -p mu-cli --test worlds --locked` passes, with `fake_world_answers`, `fake_world_flag_is_hidden` and `missing_password_variable` for the three other scenarios of the fake world.

## 3. The frame and the wait

Load the `rust-best-practices` skill before this group.

- [x] 3.1 In `session.rs`, add the control frame request beside the command request. A frame on a world that is down gets its answer at once. A frame during a connect waits in the list of `Conn::Connecting`. A frame on a world that is up goes into the list of `Conn::Up`, and then the session writes its mark. Each `Inbound::Mark` answers the first frame in that list, and a mark with an empty list changes nothing. Put the list in a small type over a `VecDeque`, with the unit test `waiting_frames_answer_in_order`. `Session::down` answers each frame in the list of the state that it leaves. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [x] 3.2 In `portal.rs`, make `Portal::start` create `watch::channel(journal.last())` next to the subscription of the server handlers. Make `run_server_handlers` send each `event.seq` after it handles the event. Add one private method that sends a `Request` to a world and waits for its answer, and make `Portal::submit` use it. Its parameter is a function from the answer sender to the `Request`, and a frame carries a sender of the same `Reply` as a command. Make `to_every_world` take a function from `WorldId` and the answer sender to the `Request`, and return the result of each world. Use it for `stop`, `connect_all` and the frames of a round. Add `Portal::wait_idle`, the engine wait until no work is left, with the round of the design. It returns `Err(PortalError::Stopped)` when a frame gets no answer. Its documentation says that only tests call it. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [x] 3.3 In `tests/support/mod.rs`, add `idle(&portal)`, which runs `wait_idle` under `timeout(BOUND, ...)`. In `tests/portal.rs`, add these tests:
  - `wait_covers_game_echo`, for the scenario "A wait covers the echo of the game"
  - `wait_adds_nothing`, for the scenario "A wait adds nothing"
  - `wait_on_world_that_is_down`, for the scenario "A wait on a world that is down"
  - `wait_outlasts_connect`, for the scenario "A wait outlasts a connect", with a held `FakeTransport`
  - `wait_survives_drop`, for the scenario "A wait survives a drop"
  - `wait_survives_failed_connect`, for the scenario "A wait survives a failed connect", which drops the `ConnectGate`
  - `wait_survives_failed_mark_write`: a world on `BrokenTransport` with no login, so that the mark is its first write, and a wait that returns `Ok` while the journal holds a disconnect event
  - `two_waits_on_one_world`: two waits joined with `tokio::join!`, once on a connected world and once during a held connect that then opens, which both return `Ok`

  The tests of the three scenarios of a drop or a failed connect assert that the wait returns `Ok`.  - `wait_on_journal_with_events_at_start`: a `MemoryJournal` with one event, a portal with a server handler on it, and a wait that returns

  In `tests/scripts.rs`, add `stray_mark_is_ignored` for the scenario "A mark that no frame waits for". Its test framer is like `Shout` and gives `Inbound::Mark` for the line `mark`. Add `wait_covers_server_handler_loop` for the scenario "A wait covers a loop of server handlers". Make sure that `cargo nextest run -p mu-core --locked` passes.

## 4. No sleep in a test

- [x] 4.1 Delete `settle` from `tests/support/mod.rs`. Change each call in `tests/portal.rs` and `tests/scripts.rs` to `idle`, also in `refused_line_break` and in `stop_disconnects_every_world_and_refuses_connect`. Make sure that `rg -n 'settle' crates/` and `rg -n -i 'sleep' crates/mu-core/tests crates/mu-core/src` find nothing. Make sure that `rg -n 'const [A-Z_]+: u8 = (6|25[0-5]);' crates/mu-core/src` finds only lines of `telnet.rs`, and that `cargo nextest run -p mu-core --locked` passes.
- [x] 4.2 Run `mise run check` and `uv run ~/.agents/plugins/sb/skills/review-lint/scripts/review_lint.py --base origin/main`. Make sure that both pass, and that the gate `time-based-test` reports no finding.

## 5. Review

- [ ] 5.1 Run `/sb:openspec-review engine-quiescence implementation`. Make sure that the verdict in `review.md` is not changes-requested.
