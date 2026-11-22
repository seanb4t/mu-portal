# Design

## Context

See proposal.md for the three faults. The send queue is `Model::sends`, a map from a world to its texts in order. The first text of a world is with the server. `update` sends the next one after the reply of the first one (`enqueue`, `on_reply`, `next_send` in `crates/mu-tui/src/update.rs`). The event loop in `run.rs` turns `Effect::Send` into a task that calls `PortalClient::send` and gives the result back as `Msg::Reply`.

The server bounds its own work on a send. `crates/mu-core/src/session.rs` gives a write to the game `WRITE_TIMEOUT`, 10 seconds. A session does one command at a time, so a send can wait behind one other write. A send during a connect or a login is refused at once.

The line break of a world comes from `ListWorlds`, and only `Model::from_listing` reads it at startup.

The probe (`crates/mu-tui/src/probe.rs`) reads the terminal itself for at most 500 ms before the Termina event reader starts. Bracketed paste is on before the probe starts, so a paste in that time reaches the probe framer.

## Goals / Non-Goals

**Goals:**

- `update` stays free of I/O. The limits live in the event loop, and the model sees only a `Reply` or a `Msg::Listed`.
- One text per world is with the server at a time, as the requirement "The sends to a world go in order" says.

**Non-Goals:**

- A change to the API or to the server. The server keeps no record of the sends of a client.

## Decisions

### D1. One limit for each call, in the event loop

`run::within` wraps a call in `tokio::time::timeout` with `CALL_LIMIT` (30 s). A call with no answer by then becomes an error with the reason `the server gave no answer in 30 s`. `run::reply_of` turns the result of a send into `Reply::Sent` or `Reply::Refused`, and its reason adds that the game can have the line or not. So `on_reply` puts the texts back as it does for a refusal. A list of the worlds goes through `within` too, so a list with no answer becomes `Msg::Listed(Err)`. The first list at startup also goes through `within`, and a list with no answer becomes the failed first list that `Model::from_listing` shows as a notice. One helper, `spawn_reply`, starts both calls on a task and gives their message to the loop.

The limit is above the 10 seconds of the server. When a game is slow, the server's own refusal, with its exact reason, comes first. A send behind one other write can wait 20 seconds on the server, and that is still inside the limit.

Alternative: the per-call timeout of connectrpc (`CallOptions::with_timeout` through the generated `send_with_options`). It sends the deadline to the server too. The handler of the server can then stop in the middle of a write to the game. The limit of the client is a limit on the wait of the client only. `tokio::time::timeout` keeps it so, and a paused tokio clock tests it with no server. The default timeout of `ClientConfig` is not an option either: it also ends the event stream.

Alternative: a timer message in the model. It needs a clock in `update`, or a second message for each send. The refusal path already does what the user needs.

Enforced by these tests in `crates/mu-tui/tests/send.rs`, on a paused tokio clock:

- `send_with_no_answer_is_refused`
- `failed_send_is_refused_with_its_reason`
- `answered_send_is_sent`
- `answer_just_inside_the_limit_is_sent` and `answer_just_past_the_limit_is_refused`
- `list_with_no_answer_fails`

### D2. A loss or a reset returns the texts behind the one with the server

`return_waiting` drains each queue from its second text, in the order of the worlds, and puts the texts in the editor before the typed text. The first text stays, because its call is still open. Its reply then acts on the first text alone: `Sent` empties the queue, and `Refused` puts back only that text. The line about the returned texts is a hint, so a later reset does not show it again. It names the count of each world, because the texts of all worlds go to the one editor.

Alternative: clear each whole queue. Then the next send of a world goes while its first call is still open. Two texts of one world are then with the server at once. That breaks the order of the sends.

Enforced by these tests in `crates/mu-tui/tests/app.rs`:

- `lost_server_puts_the_waiting_lines_back`
- `reset_puts_the_waiting_lines_back`
- `refusal_after_a_loss_puts_back_only_its_line`
- `lost_server_puts_back_the_lines_in_the_order_of_the_worlds`
- `notice_of_the_returned_lines_does_not_come_back_after_a_reset`

### D3. A second Quit, armed by the first

`Model::quit_armed` turns on at a Quit with texts that wait. A key that does not map to Quit turns it off in `on_input_key`. The count in the notice includes the text with the server, because a quit drops it too.

Enforced by these tests in `app.rs`:

- `quit_with_waiting_sends_asks_again`
- `quit_with_only_the_line_with_the_server_asks_again`
- `another_key_cancels_the_second_quit`
- `quit_with_no_waiting_send_leaves_at_once`

### D4. The count in the top row

`view::top_row` reads `Model::sends` and shows `(N waiting)` for each world with N texts behind the one with the server. #157 tracks a method of the model that the top row, `return_waiting` and the statusline of client-shell (#101) can share.

Enforced by: `top_row_counts_the_sends_that_wait` in `app.rs`.

### D5. A list of the worlds as an effect

`Effect::ListWorlds` asks for the list, and `Msg::Listed` brings it back. `Model::learn` sets the line break of each listed world and marks it as listed. A listed world that the model does not hold comes in, with its own pane. `Model::list_failure` keeps the reason of the last failed list, and the hint of Alt+R on an unlisted world names it. `World::listed` tells "no line break" from "not known yet". `Model::listing` (an enum: `Idle`, `Asked` or `AskedBeforeReset`) keeps one list with the server at a time. A failed list, or one with no answer within the limit of D1, sets `Idle`, keeps its reason, and changes no world. A reset while a list is with the server sets `AskedBeforeReset`, because the reply can be of the server before the reset. That reply then asks for one more list.

Alt+R has three outcomes:

- No world in focus: a hint that names the line break, and no list.
- A world that no list held yet: a hint that the client does not know its line break yet, and a list.
- A listed world: the join, or the hint that the world sets no line break.

Alternative: put the line break in each event of a world. That changes the API and repeats the value in each event.

Enforced by these tests in `app.rs`:

- `list_takes_the_line_break_of_a_new_world`
- `unknown_line_break_asks_the_server`
- `alt_r_with_no_world_names_the_line_break`
- `one_list_at_a_time`
- `reset_asks_for_the_list`
- `reset_during_a_list_asks_again_after_its_reply`
- `failed_list_names_its_reason_at_alt_r`
- `list_adds_a_world_that_the_client_did_not_hold`

### D6. The probe framer holds a whole paste

When the framer finds `ESC [200~`, it sets `Skip::Paste`, and each later read goes into `pending` with no limit. At `ESC [201~` it gives the text as one `Event::Paste`. `update` then cleans it with `clean_paste` as it does each paste. The search for the end mark starts 5 bytes before the new read, so a long paste is not searched again from its start. `read_answers` reads on while a paste is open, for at most one more `wait`. A paste still open after that becomes one paste of its first part. A cut end mark or a cut UTF-8 character at its end is not part of that text.

Alternative: give each read of a paste as its own paste event. Then one paste is several edits, and a CR LF that two reads split becomes two line breaks.

Enforced by these tests in `crates/mu-tui/tests/probe.rs`:

- `paste_is_one_paste_event`
- `paste_holds_no_reply`
- `paste_longer_than_the_cap_goes_through`
- the property test `paste_split_anywhere_gives_its_text`
- `open_paste_keeps_the_probe_reading` and `ended_paste_does_not_hold_the_probe`
- `cut_end_mark_is_not_text_of_the_paste` and `cut_character_is_not_text_of_the_paste`

`paste_during_the_probe_fills_the_editor` in `app.rs` also enforces it.

## Facts about third-party tools

- If the time runs out, `tokio::time::timeout` drops the inner future. The call to the server stops on the client side, but the server can still take the text. The reason of the refusal says so.
- If every task waits, a tokio test runtime with `start_paused` moves its clock forward. So a timeout of 30 s ends at once. This needs the `test-util` feature.
- Each generated method of `connectrpc` has a `_with_options` form that takes `CallOptions`. Its timeout also goes to the server (`connectrpc` source, `src/client/mod.rs`, `CallOptions::with_timeout`).
- `termina::Event` has the variant `Paste(String)`, which `update` already handles.

## Risks / Trade-offs

- [A text that hits the limit can reach the game. A resend repeats it.] → The notice says that the game can have the line or not. The limit is above the limits of the server, so this needs a server that hangs.
- [A paste during the probe has no size limit.] → The probe stops at most 500 ms after its own bound. The terminal can write only so fast in that time.
- [A quit with texts that wait drops them] → The first Ctrl+C names the count. #156 tracks a queue on disk.
