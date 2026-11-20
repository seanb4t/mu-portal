# Tasks

The change has no spec delta, so no task names a spec scenario. Each task names the test that proves it. Each new test fails before its fix.

## 1. Test helper and snapshot

- [x] 1.1 Make `wait_rows` in `crates/mu-cli/tests/support/process.rs` call `wait_screen` with a check on the rows of the screen (D6). Make sure that `cargo nextest run -p mu-cli --test client --locked` passes with no edit to a test.
- [x] 1.2 Add to `first_view_snapshot` in `crates/mu-tui/tests/app.rs` the asserts of D7: the top row starts with `[alpha/aria]` and then `beta/bryn`, and the rows between show `Welcome, aria.` and no `Welcome, bryn.`. Make sure that the test fails with a wrong world order. Make sure that it fails with the pane of `beta/bryn` in focus. Make sure that it passes at the end of the task.

## 2. The probe frames its input, and Termina decodes each CSI

- [x] 2.1 Give one complete CSI alone to a new `termina::Parser`, with no more input to come. Check that it gives exactly one event, for a cursor reply, a DECRPM reply and DA1. Read the source of `termina` at the version in `Cargo.lock` and run the check in a test of task 2.2. Write the result in the part "Facts about third-party tools" of design.md. Make sure that design.md names the source file that shows it.
- [x] 2.2 Move the 14 tests of `crates/mu-tui/tests/probe.rs` from `parse` and `Reply` to the framer and `probe::Answer`, with the same inputs and outcomes. Add the framer tests of D10: `replies_come_from_termina`, `keys_during_the_probe_are_early_events`, `alt_bracket_before_a_reply`, `question_csi_before_a_reply`, `open_apc_does_not_hide_da1`, `apc_end_split_across_reads` and `split_reply_across_reads`. Make sure that each one fails or does not build at the start of the task.
- [x] 2.3 Write the framer of D3 in `crates/mu-tui/src/probe.rs`. Give each complete CSI alone to a `termina::Parser`. Give the APC to the graphics parser. Make the keys of a broken sequence. Give the other bytes to a `termina::Parser`. Remove the hand CSI decoder, rename the reply type `probe::Answer` (QUAL-22), and make `probe::run` return the early events. In `run.rs`, drop the parser there. Make sure that the tests of 2.2 pass, except `open_apc_does_not_hide_da1`, which needs the fake clock of task 3.1. Make sure that `cargo nextest run -p mu-tui -p mu-cli --locked` passes with that one test failing.
- [x] 2.4 Keep the DECRPM value in `Outcome` as `DecModeSetting`, and add the one function that says whether a setting is set. Call it from `Outcome::resets_grapheme_mode` and from the late DECRPM arm of `update`. Make sure that the probe tests and `cargo nextest run -p mu-tui --test update --locked` pass.
- [x] 2.5 Write D3 into `docs/src/content/docs/decisions/character-width.md`. The probe frames its input. Termina decodes each CSI. An ESC ends an open sequence. Make sure that `mise run docs:check` passes and that the page follows the simple-english rules.

## 3. The bound of the probe and its errors

- [x] 3.1 Add the constant of the tick in tenths of a second, and set VTIME and the `Duration` of the tick from it. Make the read loop a public function that takes the read and the clock. Keep the current loop condition. Write `silent_terminal_keeps_the_bound` in `tests/probe.rs`. Its fake read returns no bytes and moves the fake clock one tick, plus 1 millisecond on the first read. Make sure that it fails with the current condition.
- [x] 3.2 Change the condition so that a read starts only while the time now plus one tick is not past the deadline (D2). Make sure that `silent_terminal_keeps_the_bound` and `open_apc_does_not_hide_da1` pass.
- [x] 3.3 Add the public function of D4 to `screen`, and call it from `probe::run`, `run::run` and `screen::open`. Write `both_errors_keep_the_first`, `read_error_alone`, `restore_error_alone` and `both_ok` in `crates/mu-tui/tests/screen.rs`. Make sure that `both_errors_keep_the_first` and `read_error_alone` fail before the function keeps the first error.
- [x] 3.4 Write D2 and D4 into `character-width.md`: the timed read, the bound of 500 milliseconds, why the probe can stop one tick early, and which error the client reports. Make sure that `mise run docs:check` passes.

## 4. The probe result is a message

- [x] 4.1 Write the tests of `Msg::Probe` in `crates/mu-tui/tests/update.rs`: `probe_sets_the_caps_of_every_pane`, `incomplete_probe_waits_for_late_replies` and `unmeasured_probe_shows_the_width_notice`. Move the tests there that call `await_late_replies` or `notice` to set their state through `Msg::Probe`. A notice test then asserts the width notice, at a screen width that shows it whole. Make sure that the tests do not build before `Msg::Probe` exists.
- [ ] 4.2 Add `Msg::Probe(probe::Outcome)` and handle it in `update` (D1). In `run.rs`, put it first in the queue of early messages, and remove the calls of `set_caps`, `await_late_replies` and `notice`. While an early message waits, make the event loop draw no frame. Make the three setters private to the crate. Make sure that the tests of 4.1 and `cargo nextest run -p mu-tui -p mu-cli --locked` pass.
- [ ] 4.3 Remove the cell size from `Caps`, and add the one conversion from `probe::Outcome` to `Caps` (D5). Use it in `update` and in `crates/mu-tui/examples/image-pane.rs`. Build `Caps` without the cell size in `crates/mu-tui/tests/pane.rs`. Remove the call of `screen::cell` from `run.rs`, and keep `screen::cell` for the example. Make sure that `mise run lint` passes and that `cargo build -p mu-tui --examples --locked` builds.
- [ ] 4.4 Write D1 into `docs/src/content/docs/decisions/terminal-client-architecture.md`. The probe result is a `Msg`, and only `update` changes the model. Make sure that `mise run docs:check` passes.

## 5. Design facts that no code task owns

- [ ] 5.1 Write the rows R-89, R-90 and R-92 of D8 into `character-width.md`. A kitty reply turns graphics on, with or without DA1. The probe reads stdin or `/dev/tty`. `mu-tui` uses `rustix` with `termios` for the timed read. R-87 names only a type of the code, so it goes into no page. Make sure that `mise run docs:check` passes and that no page states a version literal.

## 6. Integration

- [ ] 6.1 Check the state of #121 with `gh issue view 121`. If its pull request merged, rebase the branch on `main` and run the tests again. Then run `mise run check`, `mise run test` and `mise run deny`. Make sure that each one passes.
- [ ] 6.2 Run `openspec validate output-pane-design-sync --strict`. Make sure that it passes with `skip_specs: true` and no spec file in the change.

## Workflow follow-up

- Run the implementation review with `/sb:openspec-review output-pane-design-sync implementation`.
- Archive with `openspec archive output-pane-design-sync --skip-specs` in its own pull request, after the change merges.
