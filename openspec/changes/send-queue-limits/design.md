# Design

## Context

See proposal.md for the three faults. The send queue is `Model::sends`, a map from a world to its texts in order. The first text of a world is with the server. `update` sends the next one after the reply of the first one (`enqueue`, `on_reply`, `next_send` in `crates/mu-tui/src/update.rs`). The event loop in `run.rs` turns `Effect::Send` into a task that calls `PortalClient::send` and gives the result back as `Msg::Reply`.

The line break of a world comes from `ListWorlds`, and only `Model::from_listing` reads it at startup.

The probe (`crates/mu-tui/src/probe.rs`) reads the terminal itself for at most 500 ms before the Termina event reader starts. Bracketed paste is on before the probe starts, so a paste in that time reaches the probe framer.

## Goals / Non-Goals

**Goals:**

- `update` stays free of I/O. The timeout lives in the event loop, and the model sees only a `Reply`.
- One text per world is with the server at a time, as the requirement "The sends to a world go in order" says.

**Non-Goals:**

- A change to the API or to the server. The server keeps no record of the sends of a client.

## Decisions

### D1. The limit of a send is in the event loop, and it gives a `Reply::Refused`

`run::reply_of` wraps the call in `tokio::time::timeout` with `SEND_LIMIT` (10 s). A call that fails or ends late becomes `Reply::Refused`, so `on_reply` puts the texts back as it does for a refusal. The model gets no new message type.

Alternative: a timer message in the model. It needs a clock in `update`, or a second message for each send. The refusal path already does what the user needs.

Enforced by: `send_with_no_answer_is_refused`, `failed_send_is_refused_with_its_reason` and `answered_send_is_sent` in `crates/mu-tui/tests/send.rs`. They run on a paused tokio clock and do not wait.

### D2. A loss or a reset returns the texts behind the one with the server

`return_waiting` drains each queue from its second text, in the order of the worlds, and puts the texts in the editor before the typed text. The first text stays, because its call is still open. Its reply then acts on the first text alone: `Sent` empties the queue, and `Refused` puts back only that text.

Alternative: clear each whole queue. Then a late `Refused` puts the first text back after the others, out of order. A late `Sent` finds no queue, and that is harmless.

Enforced by: `lost_server_puts_the_waiting_lines_back`, `reset_puts_the_waiting_lines_back` and `refusal_after_a_loss_puts_back_only_its_line` in `crates/mu-tui/tests/app.rs`.

### D3. A second Quit, armed by the first

`Model::quit_armed` turns on at a Quit with texts that wait. A key that does not map to Quit turns it off in `on_input_key`. The count in the notice includes the text with the server, because a quit drops it too.

Enforced by: `quit_with_waiting_sends_asks_again`, `another_key_cancels_the_second_quit` and `quit_with_no_waiting_send_leaves_at_once` in `app.rs`.

### D4. The count in the top row

`view::top_row` reads `Model::sends` and shows `(N waiting)` for each world with N texts behind the one with the server. The statusline of client-shell (#101) can take the count later.

Enforced by: `top_row_counts_the_sends_that_wait` in `app.rs`.

### D5. A list of the worlds as an effect

`Effect::ListWorlds` asks for the list, and `Msg::Listed` brings it back. `Model::learn` sets the line break of each listed world and marks it as listed. `World::listed` tells "no line break" from "not known yet". `Model::listing` (an enum, `Idle` or `Asked`) keeps one list with the server at a time. A failed list sets `Idle` and changes nothing else.

Alternative: put the line break in each event of a world. That changes the API and repeats the value in each event.

Enforced by: `list_takes_the_line_break_of_a_new_world`, `unknown_line_break_asks_the_server`, `one_list_at_a_time` and `reset_asks_for_the_list` in `app.rs`.

### D6. The probe framer holds a whole paste

When the framer finds `ESC [200~`, it sets `Skip::Paste`, and each later read goes into `pending` with no limit. At `ESC [201~` it gives the text as one `Event::Paste`. `update` then cleans it with `clean_paste` as it does each paste. The search for the end mark starts 5 bytes before the new read, so a long paste is not searched again from its start. `read_answers` reads on while a paste is open, for at most one more `wait`. A paste still open after that becomes one paste of its first part.

Alternative: give each read of a paste as its own paste event. Then one paste is several edits, and a CR LF that two reads split becomes two line breaks.

Enforced by: `paste_is_one_paste_event`, `paste_holds_no_reply`, `paste_longer_than_the_cap_goes_through`, the property test `paste_split_anywhere_gives_its_text`, `open_paste_keeps_the_probe_reading` and `ended_paste_does_not_hold_the_probe` in `crates/mu-tui/tests/probe.rs`, and `paste_during_the_probe_fills_the_editor` in `app.rs`.

## Facts about third-party tools

- If the time runs out, `tokio::time::timeout` drops the inner future. The call to the server stops on the client side, but the server can still take the text. The reason of the refusal says so.
- With `start_paused`, if every task waits, a tokio test runtime moves its clock forward. So a timeout of 10 s ends at once. This needs the `test-util` feature.
- `termina::Event` has the variant `Paste(String)`, which `update` already handles.

## Risks / Trade-offs

- [A text that hits the limit can reach the game. A resend repeats it.] → The notice says that the game can have the line or not.
- [A paste during the probe has no size limit.] → The probe stops at most 500 ms after its own bound. The terminal can write only so fast in that time.
- [A quit with texts that wait drops them] → The first Ctrl+C names the count. #156 tracks a queue on disk.
