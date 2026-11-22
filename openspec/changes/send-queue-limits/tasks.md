# Tasks

The code is on the branch `fix/send-queue-and-paste`. Each task names the tests that prove it, and the command that runs them. Group 4 names the tests of the scenarios that the modified requirements keep.

## 1. A send with no reply, and the texts that wait (#147)

- [x] 1.1 Read the documentation of `tokio::time::timeout` and of the `test-util` feature at the version in `Cargo.lock`, and the `CallOptions` of `connectrpc`. Make sure that design.md, part "Facts about third-party tools", agrees with them.
- [x] 1.2 Add `run::within`, `run::reply_of`, `run::spawn_reply` and `CALL_LIMIT`, and call them from the event loop (D1). Write these tests in `crates/mu-tui/tests/send.rs`:
  - `send_with_no_answer_is_refused` (scenario "A server that does not answer")
  - `answer_just_inside_the_limit_is_sent` and `answer_just_past_the_limit_is_refused` (scenario "A reply near the limit")
  - `failed_send_is_refused_with_its_reason` and `answered_send_is_sent`

  Make sure that `cargo nextest run -p mu-tui --test send --locked` passes.
- [x] 1.3 Add `return_waiting` and call it on `Lost` and `Reset` (D2). Write these tests in `crates/mu-tui/tests/app.rs`:
  - `lost_server_puts_the_waiting_lines_back` (scenario "The stream ends while lines wait")
  - `lost_server_puts_back_the_lines_in_the_order_of_the_worlds` (scenario "Lines of two worlds wait")
  - `reset_puts_the_waiting_lines_back` (scenario "The server resets while lines wait")
  - `refusal_after_a_loss_puts_back_only_its_line` (scenario "A refusal after the stream ended")
  - `notice_of_the_returned_lines_does_not_come_back_after_a_reset`

  Make sure that `cargo nextest run -p mu-tui --test app --locked` passes.
- [x] 1.4 Add the second Quit (D3). Write these tests in `app.rs`:
  - `quit_with_waiting_sends_asks_again` (scenario "Quit while lines wait")
  - `quit_with_only_the_line_with_the_server_asks_again` (scenario "Quit while only the line with the server waits")
  - `another_key_cancels_the_second_quit` (scenario "Another key between the two")
  - `quit_with_no_waiting_send_leaves_at_once`

  Make sure that `cargo nextest run -p mu-tui --test app --locked` passes.
- [x] 1.5 Show the count in the top row (D4). Write `top_row_counts_the_sends_that_wait` (scenario "Two lines wait") in `app.rs`. Make sure that `cargo nextest run -p mu-tui --test app --locked -E 'test(=top_row_counts_the_sends_that_wait)'` passes.

## 2. The line break of a world that the first list missed (#148)

- [x] 2.1 Add `Effect::ListWorlds`, `Msg::Listed`, `Model::learn`, `World::listed` and `Model::listing`, and handle the effect in the event loop through `within` (D5). Write these tests in `app.rs`:
  - `list_takes_the_line_break_of_a_new_world` (scenario "A world that the first list did not hold")
  - `unknown_line_break_asks_the_server` (scenario "A line break that the client does not know")
  - `alt_r_with_no_world_names_the_line_break` (scenario "No world in focus")
  - `one_list_at_a_time` and `reset_asks_for_the_list`

  Write `list_with_no_answer_fails` in `send.rs`. Make sure that `cargo nextest run -p mu-tui --test app --test send --locked` passes.

## 3. A paste during the probe (#149)

- [x] 3.1 Make sure that `termina::Event::Paste` holds a `String`, from the source of `termina` at the version in `Cargo.lock`.
- [x] 3.2 Add `Skip::Paste`, `PASTE_START` and `PASTE_END`, and frame a paste as one event (D6). Write these tests in `crates/mu-tui/tests/probe.rs`:
  - `paste_is_one_paste_event` and `paste_holds_no_reply`
  - `paste_longer_than_the_cap_goes_through`
  - the property test `paste_split_anywhere_gives_its_text` (scenario "A paste split across reads")
  - `open_paste_keeps_the_probe_reading` and `ended_paste_does_not_hold_the_probe`
  - `cut_end_mark_is_not_text_of_the_paste`

  Make sure that `cargo nextest run -p mu-tui --test probe --locked` passes.
- [x] 3.3 Write `paste_during_the_probe_fills_the_editor` (scenario "A paste during the queries") in `app.rs`. It frames the bytes with `probe::frame` and gives each event to `update`. Make sure that `cargo nextest run -p mu-tui --test app --locked -E 'test(=paste_during_the_probe_fills_the_editor)'` fails with the framer of `main` and passes with D6.

## 4. The scenarios that the modified requirements keep

- [x] 4.1 Make sure that `ctrl_c_quits_client_only` (scenario "Quit") passes with `cargo nextest run -p mu-cli --test client --locked -E 'test(=ctrl_c_quits_client_only)'`.
- [x] 4.2 Make sure that the tests of "A refused text stops the sends after it" pass with `cargo nextest run -p mu-tui --test app --locked`:
  - `refused_line_stops_the_rest` (scenario "A line that the server refuses")
  - `refused_last_line_goes_before_the_typed_text` (scenario "The last line is refused")
  - `refused_send_stops_the_sends_after_it` (scenario "A refused send of Enter with a send after it")
- [x] 4.3 Make sure that the tests of "Alt+R joins the lines in place" pass with `cargo nextest run -p mu-tui --test app --locked`:
  - `lines_become_one_line` (scenario "The lines become one line")
  - `join_with_no_line_break` (scenario "A world with no line break")
  - `line_break_that_the_editor_cannot_hold_is_no_line_break` (scenario "A line break that the editor cannot hold")
- [x] 4.4 Make sure that the tests of "The client measures character widths at startup" pass with `cargo nextest run -p mu-cli --test client --locked`:
  - `probe_adds_code_point_widths` (scenario "A terminal that adds code point widths")
  - `probe_with_no_answer` (scenario "A terminal that does not answer". It checks the column of `x`.)
  - `late_answer` (scenario "A late answer")

## 5. Integration

- [x] 5.1 Run `mise run check`, `mise run test` and `mise run deny`. Make sure that each one passes.
- [x] 5.2 Run `openspec validate send-queue-limits --strict`. Make sure that it passes.

## Workflow follow-up

- Implementation review with `/sb:openspec-review send-queue-limits implementation`.
- `/sb:suite-review` before the pull request.
- After the merge, archive the change in its own pull request, and close #136 R-35 with a comment.
