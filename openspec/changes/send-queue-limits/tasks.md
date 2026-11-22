# Tasks

The code of groups 1 to 3 is on the branch `fix/send-queue-and-paste` in commits `35815e2`, `a183f32`, `d1c98f3` and `a5cf99d`. Each task names the tests that prove it.

## 1. A send with no reply, and the texts that wait (#147)

- [x] 1.1 Read the documentation of `tokio::time::timeout` and of the `test-util` feature at the version in `Cargo.lock`. Make sure that design.md, part "Facts about third-party tools", agrees with it.
- [x] 1.2 Add `run::reply_of` and `SEND_LIMIT`, and call them from the event loop (D1). Write `send_with_no_answer_is_refused` (scenario "A server that does not answer"), `failed_send_is_refused_with_its_reason` and `answered_send_is_sent` in `crates/mu-tui/tests/send.rs`. Make sure that `cargo nextest run -p mu-tui --test send --locked` passes.
- [x] 1.3 Add `return_waiting` and call it on `Lost` and `Reset` (D2). Write these tests in `crates/mu-tui/tests/app.rs`:
  - `lost_server_puts_the_waiting_lines_back` (scenario "The stream ends while lines wait")
  - `reset_puts_the_waiting_lines_back` (scenario "The server resets while lines wait")
  - `refusal_after_a_loss_puts_back_only_its_line` (scenario "A refusal after the stream ended")

  Make sure that they pass.
- [x] 1.4 Add the second Quit (D3). Write `quit_with_waiting_sends_asks_again` (scenario "Quit while lines wait"), `another_key_cancels_the_second_quit` (scenario "Another key between the two") and `quit_with_no_waiting_send_leaves_at_once` (scenario "Quit") in `app.rs`. Make sure that they pass.
- [x] 1.5 Show the count in the top row (D4). Write `top_row_counts_the_sends_that_wait` (scenario "Two lines wait") in `app.rs`. Make sure that it passes.

## 2. The line break of a world that the first list missed (#148)

- [x] 2.1 Add `Effect::ListWorlds`, `Msg::Listed`, `Model::learn`, `World::listed` and `Model::listing`, and handle the effect in the event loop (D5). Write these tests in `app.rs`:
  - `list_takes_the_line_break_of_a_new_world` (scenario "A world that the first list did not hold")
  - `unknown_line_break_asks_the_server` (scenario "A line break that the client does not know")
  - `one_list_at_a_time` and `reset_asks_for_the_list`

  Make sure that they pass. Make sure that `join_with_no_line_break` (scenario "A world with no line break") still passes.

## 3. A paste during the probe (#149)

- [x] 3.1 Make sure that `termina::Event::Paste` holds a `String`, from the source of `termina` at the version in `Cargo.lock`.
- [x] 3.2 Add `Skip::Paste`, `PASTE_START` and `PASTE_END`, and frame a paste as one event (D6). Write `paste_is_one_paste_event`, `paste_holds_no_reply`, `paste_longer_than_the_cap_goes_through`, the property test `paste_split_anywhere_gives_its_text` (scenario "A paste split across reads"), `open_paste_keeps_the_probe_reading` and `ended_paste_does_not_hold_the_probe` in `crates/mu-tui/tests/probe.rs`. Make sure that they pass.
- [x] 3.3 Write `paste_during_the_probe_fills_the_editor` (scenario "A paste during the queries") in `app.rs`: frame the bytes with `probe::frame` and give each event to `update`. Make sure that it fails with the framer of `main` and passes with D6.

## 4. Integration

- [x] 4.1 Run `mise run check`, `mise run test` and `mise run deny`. Make sure that each one passes.
- [x] 4.2 Run `openspec validate send-queue-limits --strict`. Make sure that it passes.

## Workflow follow-up

- Plan review and implementation review with `/sb:openspec-review send-queue-limits`.
- `/sb:suite-review` before the pull request.
- After the merge, archive the change in its own pull request, and close #136 R-35 with a comment.
