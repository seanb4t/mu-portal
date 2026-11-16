# Tasks

## 1. Documentation

- [ ] 1.1 Read RFC 860 (TIMING-MARK) and RFC 854 (the codes of IAC, DO, WILL and WONT). Read the part of RFC 1143 on an option that is already on. Read the current documentation of `tokio` with Context7 for `sync::watch` (`channel`, `send_replace`, `wait_for`) and for how `select!` picks a branch. Fill the table "Checked facts" of the design with one row for each fact that the design uses, with its source section. Make sure that the table has a row for each fact. Make sure that each difference from the documentation is an edit of the design that its row names.

## 2. The mark on the wire

Load the `rust-best-practices` skill before this group.

- [ ] 2.1 In `framer.rs`, add the crate-private constants for IAC, DO, WILL, WONT and TIMING-MARK. Add the variant `Inbound::Mark` and the method `Framer::mark`, which gives the bytes of a mark. Give `LineFramer` a side: the client side gives `IAC DO TIMING-MARK` and reads `IAC WILL TIMING-MARK` and `IAC WONT TIMING-MARK` as `Inbound::Mark`. The game side reads `IAC DO TIMING-MARK` as `Inbound::Mark`. Any other IAC sequence stays in the line as data. Add these tests:
  - `line_framer_reads_mark_answer`, for the scenario "The line framer reads the answer to a mark", with both answers
  - `line_framer_keeps_other_iac_bytes`, with `IAC WILL 5` and, on the client side, `IAC DO TIMING-MARK`, each split after every byte
  - `line_framer_mark_at_the_bound`: `MAX_LINE - 1` data bytes, a mark and an LF, fed whole and split after IAC, which give one mark and no line with the byte 255
  - in `line_framer_ignores_read_splits`, a generator that favors the bytes 255, 251, 252, 253 and 6
  - the property test `line_framer_marks_are_transparent`: marks inserted anywhere in a stream with no byte 255 leave the lines unchanged, and the count of marks is the count of inserts

  Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 2.2 In `fake.rs`, make the fake game read bytes and feed them to a game-side `LineFramer`. It answers each `Inbound::Mark` with `IAC WILL TIMING-MARK`, from the constants of task 2.1, before and after the login. A mark does not go into the lines that the game got. Add the unit test `fake_game_answers_mark_in_order` for the scenario "The fake world answers a mark in order": `one`, a mark and `two`, written in one piece and cut at every byte, with a case with a mark before the login. Make sure that `cargo nextest run -p mu-core --locked` passes with the four `fake_game_*` tests from before. Make sure that `cargo nextest run -p mu-cli --test worlds --locked` passes, with `fake_world_answers`, `fake_world_flag_is_hidden` and `missing_password_variable` for the three other scenarios of the fake world.

## 3. The frame and the wait

Load the `rust-best-practices` skill before this group.

- [ ] 3.1 In `session.rs`, add the control frame request beside the command request. A frame on a world that is down gets its answer at once. A frame during a connect waits in the connect list. A frame on a world that is up sends a mark and waits in the mark list. Each `Inbound::Mark` answers the first frame in the mark list, and a mark with an empty list changes nothing. `Session::down` answers each frame in both lists. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 3.2 In `portal.rs`, make `Portal::start` create `watch::channel(journal.last())` next to the subscription of the server handlers. Make `run_server_handlers` send each `event.seq` after it handles the event. Add one private method that sends a `Request` to a world and waits for its answer, and make `Portal::submit` use it. Add `Portal::wait_idle`, the engine wait until no work is left, with the round of the design. Its documentation says that only tests call it. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 3.3 In `tests/support/mod.rs`, add `idle(&portal)`, which runs `wait_idle` under `timeout(BOUND, ...)`. In `tests/portal.rs`, add these tests:
  - `wait_covers_game_echo`, for the scenario "A wait covers the echo of the game"
  - `wait_adds_nothing`, for the scenario "A wait adds nothing"
  - `wait_on_world_that_is_down`, for the scenario "A wait on a world that is down"
  - `wait_outlasts_connect`, for the scenario "A wait outlasts a connect", with a held `FakeTransport`
  - `wait_survives_drop`, for the scenario "A wait survives a drop"
  - `wait_survives_failed_connect`, for the scenario "A wait survives a failed connect", which drops the `ConnectGate`
  - `wait_on_journal_with_events_at_start`: a `MemoryJournal` with one event, a portal with a server handler on it, and a wait that returns

  In `tests/scripts.rs`, add `stray_mark_is_ignored` for the scenario "A mark that no frame waits for". Its test framer is like `Shout` and gives `Inbound::Mark` for the line `mark`. Add `wait_covers_server_handler_loop` for the scenario "A wait covers a loop of server handlers". Make sure that `cargo nextest run -p mu-core --locked` passes.

## 4. No sleep in a test

- [ ] 4.1 Delete `settle` from `tests/support/mod.rs`. Change each call in `tests/portal.rs` and `tests/scripts.rs` to `idle`, also in `refused_line_break` and in `stop_disconnects_every_world_and_refuses_connect`. Make sure that `rg -n 'settle' crates/` and `rg -n 'sleep\(' crates/*/tests crates/*/src` find nothing, and that `cargo nextest run -p mu-core --locked` passes.
- [ ] 4.2 Run `mise run check` and `uv run ~/.agents/plugins/sb/skills/review-lint/scripts/review_lint.py --base origin/main`. Make sure that both pass, and that the gate `time-based-test` reports no finding.

## 5. Review

- [ ] 5.1 Run `/sb:openspec-review engine-quiescence implementation`. Make sure that the verdict in `review.md` is not changes-requested.
