# Tasks

The change has no spec delta, so no task names a spec scenario. Each task names the test that proves it. Each new test fails before its fix.

## 1. Test helper and snapshot

- [ ] 1.1 Make `wait_rows` in `crates/mu-cli/tests/support/process.rs` call `wait_screen` with a check on the rows of the screen (D7). Make sure that `cargo nextest run -p mu-cli --test client --locked` passes with no edit to a test.
- [ ] 1.2 Add to `first_view_snapshot` in `crates/mu-tui/tests/app.rs` the asserts of the scenario "The first view": the top row starts with `[alpha/aria]` and then `beta/bryn`, and the rows between show `Welcome, aria.` (D8). Make sure that the test fails with a wrong world order and passes at the end of the task.

## 2. The probe parses with Termina

- [ ] 2.1 Read the source and the docs of `termina::Parser` at the version in `Cargo.lock`. Find how it treats an ESC inside an open CSI, and what its flag for more input does with a sequence that a read splits. Write the result in the part "Facts about third-party tools" of design.md, and pick the branch of D4. Make sure that design.md names both facts and the source file that shows each one.
- [ ] 2.2 Write the unit tests of the probe input in `crates/mu-tui/src/probe.rs`: `replies_come_from_termina` (a cursor reply, a DECRPM reply and DA1), `keys_during_the_probe_are_early_events`, `alt_bracket_before_a_reply` (QUAL-3), `open_apc_does_not_hide_da1` (SEC-6) and `split_reply_across_reads`. Make sure that each one fails at the start of the task, or does not build against the new return type.
- [ ] 2.3 Split the input as D3 and D4 say. Give every byte outside an APC to a `termina::Parser`, take the three reply kinds from its events, and return the other events. Remove the hand CSI parser and make the reply type private (QUAL-22). In `run.rs`, use the events from the probe and drop the parser there. Make sure that the tests of 2.2 and `cargo nextest run -p mu-tui -p mu-cli --locked` pass.
- [ ] 2.4 Write D3 and D4 into `docs/src/content/docs/decisions/character-width.md`: the probe parses only the kitty graphics reply, Termina parses the rest, and an ESC ends an open sequence. Make sure that `mise run docs:check` passes and that the page follows the simple-english rules.

## 3. The bound of the probe and its errors

- [ ] 3.1 Make the read loop take the read and the clock as arguments, with the current loop condition. Write `silent_terminal_keeps_the_bound`. In it, a fake read adds one tick to a fake clock and returns no bytes. The test asserts that the fake time at the return is not past `WAIT`. Make sure that it fails with the current condition.
- [ ] 3.2 Change the condition so that a read starts only while the time now plus one tick is not past the deadline (D2). Make sure that `silent_terminal_keeps_the_bound` and `open_apc_does_not_hide_da1` pass.
- [ ] 3.3 Add the function that takes the read result and the restore result and gives one result (D5). Write `read_error_wins_over_restore_error`, `restore_error_alone` and `both_ok`. Make sure that the first one fails before the function keeps the read error.
- [ ] 3.4 Write D2 and D5 into `character-width.md`: the timed read, the bound of 500 milliseconds and why the probe can stop one tick early. Make sure that `mise run docs:check` passes.

## 4. The probe result is a message

- [ ] 4.1 Write the unit tests of `update` for `Msg::Probe` in `crates/mu-tui/src/update.rs` or `crates/mu-tui/tests/app.rs`: `probe_sets_the_caps_of_every_pane`, `incomplete_probe_waits_for_late_replies` and `unmeasured_probe_shows_the_width_notice`. Make sure that they do not build before `Msg::Probe` exists.
- [ ] 4.2 Add `Msg::Probe(probe::Outcome)` and handle it in `update` (D1). In `run.rs`, give it to `update` before the early events, and remove the calls of `set_caps`, `await_late_replies` and `notice` from `run.rs`. Make each setter that only `update` calls private to the crate. Make sure that the tests of 4.1 and `cargo nextest run -p mu-tui -p mu-cli --locked` pass.
- [ ] 4.3 Remove the cell size from `Caps`, and remove the call of `screen::cell` from `run.rs` (D6, QUAL-21). Keep `screen::cell` for the example `image-pane`. Make sure that `mise run lint` passes with no dead code warning.
- [ ] 4.4 Write D1 into `docs/src/content/docs/decisions/terminal-client-architecture.md`: the probe result is a `Msg`, and only `update` changes the model. Make sure that `mise run docs:check` passes.

## 5. Design facts that no code task owns

- [ ] 5.1 Write the rows R-87, R-89, R-90 and R-92 of D9 into `character-width.md`. A kitty reply turns graphics on, with or without DA1. The probe reads stdin or `/dev/tty`. `mu-tui` uses `rustix` with `termios` for the timed read. R-87 names only a type of the code, so it goes into no page. Make sure that `mise run docs:check` passes and that no page states a version literal.

## 6. Integration

- [ ] 6.1 Run `mise run check`, `mise run test` and `mise run deny`. Make sure that each one passes.
- [ ] 6.2 Run `openspec validate output-pane-design-sync --strict`. Make sure that it passes with `skip_specs: true` and no spec file in the change.

## Workflow follow-up

- Run the implementation review with `/sb:openspec-review output-pane-design-sync implementation`.
- Archive with `openspec archive output-pane-design-sync --skip-specs` in its own pull request, after the change merges.
