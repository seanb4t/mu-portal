# Tasks

## 1. Crates and dependencies

- [ ] 1.1 Read the current documentation of `connectrpc`, `connectrpc-build` and `buffa` with Context7. Read how to serve the router on a tokio `UnixListener`, `Http2Connection::lazy_unix`, server streams, error codes and the `oneof` in generated messages. Note each difference from the design in the design. Make sure that the design names no fact that the documentation contradicts.
- [ ] 1.2 Read the current documentation of `tokio` with Context7: `broadcast` and its capacity rules, `signal::unix` and `start_paused`. Read it also for `tokio-stream` and its `BroadcastStream`, `backon`, `secrecy`, and `rustix` for the user id and the file owner. Then read it for `ratatui` and its `TestBackend`, `crossterm`, `ratatui-textarea`, `portable-pty` and `vt100`. Make sure that the design names no fact that the documentation contradicts.
- [ ] 1.3 Add the crates `crates/mu-server` and `crates/mu-client`, with the workspace package fields and lints. In `crates/mu-cli/tests/workspace_boundary.rs`, change the allowlist and the base fixture to the edges of the design. Remove the edge from `mu-core` to `mu-proto` from both. Add `mu-server` and `mu-client` to the base fixture.
- [ ] 1.4 Add a second list to the boundary test: the third-party crates that `mu-core` and `mu-tui` must not use. Add the fixture tests `core_depends_on_proto_fails`, `core_depends_on_buffa_fails`, `client_depends_on_core_fails` and `tui_depends_on_server_fails`. Add the new failures to the list in `docs/src/content/docs/contributing/workspace.md`. Change the crate layout view in `architecture/model.c4` and the crate table of the same page. Make sure that `cargo nextest run -p mu-cli --test workspace_boundary --locked` and `mise run docs:check` pass.
- [ ] 1.5 Add each crate of the design table to `[workspace.dependencies]` at its latest version. Add the `env` feature to `clap`. Name each crate with `workspace = true` in each crate that uses it. Make sure that `mise run deny` passes.

## 2. The schema

- [ ] 2.1 Write `crates/mu-proto/proto/mu/v1/portal.proto` with the calls and messages of the design. `Subscribe` streams a `oneof` of `Hello` and `Event`. The other calls are `ListWorlds`, `Send` with an optional sequence number in its reply, `Connect` and `Disconnect`. Enums hold the kind, the source and the state. Generate the code in `build.rs` with `connectrpc-build`. Make sure that `cargo build -p mu-proto --locked` passes.

## 3. The engine

Load the `rust-best-practices` skill before this group.

- [ ] 3.1 In `mu-core`, add `WorldId`, which parses an id by the rules of the world-sessions spec. Add the types `Event`, `EventKind`, `Source`, `Seq`, `Command`, `Envelope`, `Origin` and `PortalError`. Add the tests `ids_outside_the_rule` and `ids_inside_the_rule` with the ids of the two scenarios. Add the property test `world_id_round_trip` with `proptest`. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 3.2 Add the `Journal` trait and `MemoryJournal`. Its constructor takes the capacity of the live channel and refuses 0 and a capacity that is not a power of two. Add the tests `sequence_has_no_gap`, `subscribe_from_middle`, `subscribe_from_zero`, `subscribe_from_next`, `subscribe_past_end` and `capacity_must_be_a_power_of_two`. Add `subscribe_takes_backlog_and_live_under_one_lock`, which appends from a second task while it subscribes and checks that each number arrives once. Add the property test `resume_has_no_gap_for_any_cut`. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 3.3 Add `SecretRef`, `Secret` on top of `secrecy`, the `SecretStore` trait and `EnvSecretStore`. Add the tests `env_secret_is_read`, `missing_env_secret_names_the_variable`, `empty_env_secret_is_refused`, `non_unicode_env_secret_is_refused` and `secret_debug_hides_value`. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 3.4 Add `ResolvedWorld` and the `LayerSource` trait. Add the test `fake_layer_source_returns_fields`. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 3.5 Add the `Transport` trait, the fake game and `FakeTransport`. The fake game greets, checks the login line, and answers each line as the world-sessions spec says. A fake transport can hold the connect open, so that a test can see the state `connecting`. Add the tests `fake_game_accepts_login`, `fake_game_refuses_wrong_password` and `fake_game_answers_line`. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 3.6 Add the `Framer` trait and `LineFramer`. Add the tests `line_framer_splits_on_lf`, `line_framer_drops_cr` and `line_framer_keeps_a_partial_line`. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 3.7 Add the session task and `Portal`, with one task for each world. Add the login with redaction at the source, and the mask of the password in each event. Add `Send`, `Connect`, `Disconnect` and `ListWorlds`. Use the paused clock. Make sure that `cargo nextest run -p mu-core --locked` passes, with these tests:
  - `two_characters_on_one_game`, `send_to_unknown_world`, `send_to_disconnected_world`, `send_to_connecting_world`, `send_with_line_break_is_refused`, `one_world_drops` and `world_connects_again`
  - `login_event_is_redacted`: the fake game gets the password, and no event holds it
  - `typed_password_is_masked`: a player line and its echo hold `****` in the journal
- [ ] 3.8 Add the `Hook` and `Handler` traits, the handlers of the server, and the depth limit. Add the tests `hook_gags_game_line`, `hook_replaces_player_line`, `hook_gags_player_line`, `handler_answers_line`, `handler_loop_stops_at_depth_limit` and `handler_on_every_event_stops`, each with a test hook or handler. `handler_loop_stops_at_depth_limit` counts exactly 8 custom events after the first one. `handler_answers_line` checks that the event of the send has the source script. Make sure that `cargo nextest run -p mu-core --locked` passes.

## 4. The server adapter

- [ ] 4.1 In `mu-server`, add the conversion between engine types and protobuf types, and the map from `PortalError` to codes. Add the test `portal_errors_map_to_codes`, with one case for each arm. Make sure that `cargo nextest run -p mu-server --locked` passes.
- [ ] 4.2 Add the `PortalService` implementation over `Portal`, with the `Hello` message first on each stream. Turn the live receiver into a stream with `BroadcastStream`, and end a lagging stream with `resource_exhausted`. Add the in-process test `slow_subscriber_is_cut` (scenario: a subscriber that does not read). It uses the capacity 4, and checks that the stream is open after 4 events and ends after the fifth. Make sure that `cargo nextest run -p mu-server --locked` passes.
- [ ] 4.3 Add the life of the process, in the order of the design: read the secrets, take the private directory, the lock and the check of the file at the socket path. Then bind, write the line on stderr, build the `Portal`, connect each world, and stop on a signal. Make sure that `cargo nextest run -p mu-server --locked` passes. The tests of the built binary are in group 6.

## 5. The client library

- [ ] 5.1 In `mu-client`, add `PortalClient` with the calls and `events()`. It resumes after an error and after a clean end of the stream, with the backoff of `backon`. It resets on a new instance id. Add the unit test `backoff_schedule` with the paused clock. Make sure that `cargo nextest run -p mu-client --locked` passes. The resume tests are in task 6.6.

## 6. The verbs

- [ ] 6.1 In `mu-cli`, add `SocketArgs` with `--socket` and `MU_SOCKET`, the default path and `private_dir`. Flatten `SocketArgs` into `ClientArgs` and into the arguments of `mu server`. Wire `mu server` with the hidden `--fake-world`. Remove `not_implemented` and the tests `server_not_implemented`, `client_not_implemented` and `stub_verbs_leave_files_alone` from `crates/mu-cli/tests/cli.rs`. Add the unit tests `default_socket_under_runtime_dir_path` and `default_socket_without_runtime_dir`. Make sure that `cargo nextest run -p mu-cli --locked` passes, with `help_lists_verbs` and `no_verb_equals_client`.
- [ ] 6.2 Move the helper `mu` from `cli.rs` to `crates/mu-cli/tests/support/mod.rs`. Add `ServerProcess`, `Relay` and `wait_until` there, as the design says. `ServerProcess` starts the built `mu server` on a socket in a temporary directory. It waits for the line on stderr through `wait_until` and returns a `PortalClient`. It kills the process on drop. Make sure that `cargo nextest run -p mu-cli --locked` passes.
- [ ] 6.3 Add `crates/mu-cli/tests/server.rs` with one test for each scenario of the server-process spec that needs no terminal. Make sure that `cargo nextest run -p mu-cli --test server --locked` passes on Linux and on macOS in CI, with these tests:
  - `serves_on_named_socket`, `env_names_socket`, `flag_wins_over_env` and `default_socket_under_runtime_dir`
  - `new_socket_dir_is_private`, and `open_socket_dir_is_refused` with the modes 0755, 0750, 0705 and 0701
  - `second_server_is_refused`, `stale_socket_is_replaced` and `regular_file_at_socket_path_is_kept`
  - `sigterm_stops_server` and `sigint_stops_server`
  - `client_needs_a_terminal`, which pipes stdout and checks that the directory is still empty
- [ ] 6.4 Add `crates/mu-cli/tests/worlds.rs` with one test for each scenario of the world-sessions spec on the binary. The tests are `bad_world_id_is_refused`, `fake_world_answers`, `fake_world_flag_is_hidden`, `missing_password_variable`, `two_worlds_connect`, `two_characters_on_one_game`, `send_to_unknown_world`, `send_to_disconnected_world`, `send_with_line_break_is_refused`, `one_world_drops` and `world_connects_again`. Make sure that `cargo nextest run -p mu-cli --test worlds --locked` passes.
- [ ] 6.5 Add `crates/mu-cli/tests/journal.rs` with one test for each scenario of the event-journal spec on the binary. The tests are `events_of_two_worlds_share_one_sequence`, `sources_of_a_session`, `password_is_replaced`, `typed_password_is_masked`, `stream_from_the_middle`, `stream_from_zero`, `stream_from_the_next_number` and `stream_past_the_end`. `password_is_replaced` also reads the stderr of the server. Make sure that `cargo nextest run -p mu-cli --test journal --locked` passes.
- [ ] 6.6 Add `crates/mu-cli/tests/resume.rs` with `PortalClient` and the relay. Add the tests `resume_after_cut_connection` (scenario: resume after a cut connection) and `resume_after_server_restart` (scenario: resume after a restart of the server). Add `resume_after_clean_end`, which stops the server with SIGTERM, starts it again, and checks that the client resets and reads the new events. Make sure that `cargo nextest run -p mu-cli --test resume --locked` passes.

## 7. The terminal client

Load the `rust-best-practices` skill before this group.

- [ ] 7.1 In `mu-tui`, add `App` with `on_event`, `on_key` and `draw`, and the input line on `ratatui-textarea`. Add `crates/mu-tui/tests/app.rs` with `TestBackend`, and one test for each scenario. Make sure that `cargo nextest run -p mu-tui --locked` passes, with these tests:
  - `first_view`, `world_order`, `no_worlds` and `line_prefixes`
  - `enter_sends_to_world_in_focus`, `empty_enter_sends_empty_line`, `enter_with_no_world` and `refused_line_keeps_input`
  - `ctrl_n_moves_focus`, `ctrl_p_wraps_around`, `one_world_keeps_focus` and `activity_mark`
  - `lost_stream_resumes`, which feeds `Lost`, then `after the cut` once, and checks the top row and the rows
- [ ] 7.2 Add the loop that joins `crossterm` keys, `PortalClient::events()` and the replies. The loop uses the alternate screen. It restores the terminal on exit and on a panic. Wire `mu client` to it, with the steps of the design: the check for a terminal first, then `private_dir`, then the start of a server. Make sure that `cargo build -p mu-cli --locked` passes.
- [ ] 7.3 Add `crates/mu-cli/tests/client.rs`. It runs the built `mu client` with `portable-pty`, in a pseudo-terminal of 80 by 24. It reads the screen with `vt100` and waits through `wait_until`. A failure prints the screen. Make sure that `cargo nextest run -p mu-cli --test client --locked` passes on Linux and on macOS in CI, with these tests:
  - `client_starts_a_server`, which also checks the mode 0700 of the new directory
  - `two_clients_start_one_server`, which checks that `server.log` holds exactly one line that names the socket as served
  - `client_needs_a_terminal_on_stdin`, which gives the client a terminal on stdout and a pipe on stdin
  - `ctrl_c_quits_client_only`

## 8. Documentation

- [ ] 8.1 Remove the rule for a verb without an implementation from `docs/src/content/docs/contributing/openspec.md`. Change the text that says the verbs print `not implemented` in `README.md` and `docs/src/content/docs/index.md`. Add the page `docs/src/content/docs/guides/first-run.md`. It shows how to start a server with two fake worlds and attach a client. Add a Guides group for the directory `guides` to the sidebar in `docs/astro.config.mjs`. Make sure that `mise run docs:check` passes, that the page is in the sidebar of the built site, and that `rg -n 'not implemented' README.md docs/src` prints nothing.

## 9. Review and checks

- [ ] 9.1 Run `/sb:openspec-review portal-skeleton implementation` and treat changes-requested as a blocker. Make sure that `review.md` has a verdict that is not changes-requested.
- [ ] 9.2 Run `mise run check` and `uv run ~/.agents/plugins/sb/skills/review-lint/scripts/review_lint.py --base origin/main`. Make sure that both pass.

## Workflow follow-up

- After the merge, archive the change with the post-merge skill. The acceptance command is `mise run check`.
