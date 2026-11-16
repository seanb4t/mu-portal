# Tasks

## 1. Crates and dependencies

- [x] 1.1 Read the current documentation of `connectrpc`, `connectrpc-build` and `buffa` with Context7. Read how to serve the router on a tokio `UnixListener`, `Http2Connection::lazy_unix`, server streams, error codes and the `oneof` in generated messages. Note each difference from the design in the design. Make sure that the design names no fact that the documentation contradicts.
- [x] 1.2 Read the current documentation of `tokio` with Context7: `broadcast` and its capacity rules, `signal::unix` and `start_paused`. Read it also for `backon` and the limit of retries of its `ExponentialBuilder`. Read it for `secrecy`, and for `rustix` and its calls for the user id and the file owner. Then read it for `ratatui` and its `TestBackend`, `crossterm` and its `EventStream`, `ratatui-textarea`, `portable-pty` and `vt100`. Make sure that the design names no fact that the documentation contradicts.
- [x] 1.3 Add the crates `crates/mu-server` and `crates/mu-client`, with the workspace package fields and lints. In `crates/mu-cli/tests/workspace_boundary.rs`, change the allowlist and the base fixture to the edges of the design. Remove the edge from `mu-core` to `mu-proto` from both. Add `mu-server` and `mu-client` to the base fixture.
- [x] 1.4 Add a second allowlist to the boundary test: the third-party crates that each member can use, in the form of the list of edges. Add the fixture tests `core_to_proto_fails`, `core_to_buffa_fails`, `tui_to_connectrpc_fails`, `client_to_core_fails` and `tui_to_server_fails`. Add the new failure to the list in `docs/src/content/docs/contributing/workspace.md`, and change the crate table and the text that says four crates. Add a step to its how-to: a new third-party crate also goes in the allowlist of each member that uses it. Change the description and the text of `docs/src/content/docs/architecture/crate-layout.mdx`, which say four crates. Change the crates in `architecture/model.c4`, and the view `crateLayout` and its description in `architecture/views.c4`. Make sure that `cargo nextest run -p mu-cli --test workspace_boundary --locked` and `mise run docs:check` pass.
- [x] 1.5 Add each crate of the design table to `[workspace.dependencies]` at its latest version. Add the `env` feature to `clap`. Pin `protoc` as a tool in `mise.toml`, where Renovate reads it. Turn on the `test-util` feature of `tokio` in the dev-dependencies. Turn on the `event-stream` feature of `crossterm`, at the version that `ratatui` uses. Add the Renovate group `crossterm` for `crossterm` and `ratatui` to `renovate.json`. Name each crate with `workspace = true` in each crate that uses it. Make sure that `mise run deny` passes.

## 2. The schema

- [x] 2.1 Write `crates/mu-proto/proto/mu/v1/portal.proto` with the calls and messages of the design. `Subscribe` streams a `oneof` of `Hello` and `Event`. The other calls are `ListWorlds`, `Send` with an optional sequence number in its reply, `Connect` and `Disconnect`. Enums hold the kind, the source and the state. Generate the code in `build.rs` with `connectrpc-build`. Make sure that `mise exec -- cargo build -p mu-proto --locked` passes, with the `protoc` of `mise.toml`.

## 3. The engine

Load the `rust-best-practices` skill before this group.

- [x] 3.1 In `mu-core`, add `WorldId`, which parses an id by the rules of the world-sessions spec. Add the types `Event`, `EventKind`, `Source`, `Seq`, `Command`, `Envelope`, `Origin` and `PortalError`. Add the tests `ids_outside_the_rule` and `ids_inside_the_rule` with the ids of the two scenarios. Add `event_source_follows_kind`, which checks the source of each kind, and of an input and an echo from each origin. Add the property test `world_id_round_trip` with `proptest`. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [x] 3.2 Add the `Journal` trait and `MemoryJournal`. Its constructor takes the capacity of the live channel. It refuses 0, a capacity that is not a power of two, and a capacity above 65536. Add the tests `sequence_has_no_gap`, `subscribe_from_middle`, `subscribe_from_zero` and `subscribe_from_next`. Add `subscribe_past_end`, with the number 8 for a journal of 6. Add `capacity_rules`, with 0, 3, 4, 65536 and 131072. Add `receiver_at_capacity_is_not_cut` and `receiver_past_capacity_is_cut`, which hold an unread receiver with the capacity 4 through 4 and 5 events. Add `subscribe_takes_backlog_and_live_under_one_lock`, which appends from a second task while it subscribes and checks that each number arrives once. Add the property tests `resume_has_no_gap_for_any_cut` and `interleaved_appends_and_subscribes_keep_order`. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [x] 3.3 Add `SecretRef`, `Secret` on top of `secrecy`, the `SecretStore` trait and `EnvSecretStore`. Add the refusal of a password that `****` holds, and of a password with CR or LF. Add the tests `env_secret_is_read`, `missing_env_secret_names_the_variable`, `empty_env_secret_is_refused`, `non_unicode_env_secret_is_refused` and `secret_debug_hides_value`. Add `star_passwords_at_the_edge`, which refuses `*` and `****` and accepts `*****`. Add `password_with_cr_is_refused` and `password_with_lf_is_refused`. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [x] 3.4 Add `ResolvedWorld`, the output of the merge that #53 adds with `figment`. Add no trait of its own for the layers. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [x] 3.5 Add the `Transport` trait, the fake game and `FakeTransport`. The fake game greets, checks the login line, and answers each line as the world-sessions spec says. A fake transport can hold the connect open, so that a test can see the state `connecting`. The line `@drop` makes the fake game close the connection. Add the tests `fake_game_accepts_login`, `fake_game_refuses_wrong_password`, `fake_game_answers_line` and `fake_game_drops_on_request`. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [x] 3.6 Add the `Framer` trait and `LineFramer`. `Inbound` has a variant for a message of a side protocol, which `LineFramer` never gives. Add the tests `line_framer_splits_on_lf`, `line_framer_drops_cr` and `line_framer_keeps_a_partial_line`, and the property test `line_framer_ignores_read_splits`. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [x] 3.7 Add the session task and `Portal`, with one task for each world. Add the login with redaction at the source, and the mask of the password in each event. Add `Send`, `Connect`, `Disconnect` and `ListWorlds`. Use the paused clock. Make sure that `cargo nextest run -p mu-core --locked` passes, with these tests:
  - `two_characters_on_one_game`, `send_to_unknown_world`, `send_to_disconnected_world`, `send_to_connecting_world`, `empty_send_reaches_game`, `send_with_lf_is_refused`, `send_with_cr_is_refused`, `one_world_drops` and `world_connects_again`
  - `connect_from_api_has_source_system`
  - `login_event_is_redacted`: the fake game gets the password, and no event holds it
  - `typed_password_is_masked`: a player line and its echo hold `****` in the journal
  - `password_with_star_masks_whole_text`: the password `a*` and the text `aa*` give `****`
  - the property test `mask_never_leaves_the_password`, for any text and any password that the secret store accepts
- [x] 3.8 Add the `Hook` and `Handler` traits, the depth of each event, the depth limit and the rate limit. Add the server-handler task, which reads the journal. Add these tests, each with a test hook or handler:
  - `hook_gags_game_line`, `hook_replaces_player_line` and `hook_gags_player_line`
  - `handler_answers_line`, which checks that the event of the send has the source script
  - `handler_loop_stops_at_depth_limit`, which counts exactly 8 custom events after the first one
  - `handler_on_every_event_stops`, which counts exactly one script error and no event after it
  - `handler_loop_through_game_is_rate_limited`, which counts exactly 20 sent lines and one script error in the first second, with the paused clock
  - `rate_limit_refills`, which checks that a command right after the burst is dropped. It then moves the paused clock by 50 milliseconds, and checks that the next `start` gives `again`
  - `two_drops_give_one_script_error`, with a handler that returns 22 sends at once
  - `world_handler_skips_script_error`, with a world handler that counts the events it sees
  - `handler_connect_has_source_system`
  - `server_handler_sees_every_world`, and `server_handler_resumes_after_lag`, which checks that the handler sees each event once

  Make sure that `cargo nextest run -p mu-core --locked` passes.

## 4. The server adapter

- [x] 4.1 In `mu-server`, add the conversion between engine types and protobuf types, and the map from `PortalError` to codes. Add the test `portal_errors_map_to_codes`, with one case for each arm. Make sure that `cargo nextest run -p mu-server --locked` passes.
- [x] 4.2 Add the `PortalService` implementation over `Portal`, with the `Hello` message first on each stream. Turn the subscription into a stream with `Subscription::into_stream`, and end a lagging stream with `resource_exhausted`. Add the in-process test `lagged_stream_ends_with_resource_exhausted`, with a second subscriber that reads every event in order. The edge of 4 and 5 events is in the tests of task 3.2. Make sure that `cargo nextest run -p mu-server --locked` passes.
- [x] 4.3 Add the steps of the process that `mu-server` owns, in the order of the design: the lock and the check of the file at the socket path. Then bind, write the line on stderr, build the `Portal`, connect each world, and stop on a signal. Make sure that `cargo nextest run -p mu-server --locked` passes. The tests of the built binary are in group 6.

## 5. The client library

- [x] 5.1 In `mu-client`, add `PortalClient` with the calls and `events()`. It resumes after an error and after a clean end of the stream, with the backoff of `backon` and no limit of tries. It yields `Resumed` after each `Hello` with the same instance id, and `Reset` on a new one. Add the unit test `backoff_schedule`, which checks the waits 100, 200, 400, 800, 1600, 3200 and then 5000 milliseconds. Add `backoff_runs_past_four_tries`. Both use the paused clock. Make sure that `cargo nextest run -p mu-client --locked` passes. The resume tests are in task 6.6.

## 6. The verbs

- [x] 6.1 In `mu-cli`, add `SocketArgs` with `--socket` and `MU_SOCKET`, the default path and `private_dir`. Flatten `SocketArgs` into `ClientArgs` and into the arguments of `mu server`. Wire `mu server` with the hidden `--fake-world`, which a `clap` value parser reads. Read the secrets and call `private_dir` before `mu-server` runs. Remove `not_implemented` and the tests `server_not_implemented`, `client_not_implemented` and `stub_verbs_leave_files_alone` from `crates/mu-cli/tests/cli.rs`. Add the unit tests `default_socket_under_runtime_dir_path`, `default_socket_without_runtime_dir` and `empty_runtime_dir_counts_as_unset`. Add `relative_runtime_dir_counts_as_unset` and `empty_socket_env_counts_as_unset`. Add `private_dir_refuses_another_owner`, which passes the user id of the owner plus 1. Make sure that `cargo nextest run -p mu-cli --locked` passes, with `help_lists_verbs` and `no_verb_equals_client`.
- [x] 6.2 Move the helper `mu` from `cli.rs` to `crates/mu-cli/tests/support/mod.rs`, and add `mu_with` with an environment. Add `ServerProcess` with a builder for the arguments and the environment, `Relay` with its cut and hold, `mu_on_pty` and `wait_until` there, as the design says. `ServerProcess` starts the built `mu server` on a socket in a temporary directory. It waits for the line on stderr through `wait_until` and returns a `PortalClient`. It kills the process on drop. Make sure that `cargo nextest run -p mu-cli --locked` passes.
- [x] 6.3 Add `crates/mu-cli/tests/server.rs` with one test for each scenario of the server-process spec that needs no terminal. Make sure that `cargo nextest run -p mu-cli --test server --locked` passes on Linux and on macOS in CI, with these tests:
  - `serves_on_named_socket`, which also checks that stderr does not contain `not implemented`, `env_names_socket`, `flag_wins_over_env`, `default_socket_under_runtime_dir`, `empty_runtime_dir_is_ignored` and `empty_socket_env_is_ignored`
  - `new_socket_dir_is_private`, which checks the mode 0700 of the directory and the mode 0600 of the socket
  - `open_socket_dir_is_refused` with the modes 0740, 0720, 0710, 0704, 0702 and 0701, one for each access bit, and 0755
  - `second_server_is_refused`, `stale_socket_is_replaced` and `regular_file_at_socket_path_is_kept`
  - `sigterm_stops_server` and `sigint_stops_server`
  - `client_needs_a_terminal`, which uses `mu_on_pty` to give the client a terminal on stdin and a pipe on stdout. It checks that the directory is still empty, and that stderr does not contain `not implemented`
- [x] 6.4 Add `crates/mu-cli/tests/worlds.rs` with one test for each scenario of the world-sessions spec on the binary. Make sure that `cargo nextest run -p mu-cli --test worlds --locked` passes, with these tests:
  - `bad_world_id_is_refused`
  - `fake_world_answers`
  - `fake_world_flag_is_hidden`
  - `missing_password_variable`
  - `two_worlds_connect`
  - `two_characters_on_one_game`
  - `send_to_unknown_world`
  - `send_to_disconnected_world`
  - `send_with_lf_is_refused`
  - `send_with_cr_is_refused`
  - `empty_send_reaches_game`
  - `one_world_drops`
  - `world_connects_again`
- [x] 6.5 Add `crates/mu-cli/tests/journal.rs` with one test for each scenario of the event-journal spec on the binary. The tests are `events_of_two_worlds_share_one_sequence`, `sources_of_a_session`, `password_is_replaced`, `typed_password_is_masked`, `password_with_star_masks_whole_text`, `password_of_stars_is_refused`, `stream_from_the_middle`, `stream_from_zero`, `stream_from_the_next_number` and `stream_past_the_end`. `password_is_replaced` also reads the stderr of the server. Make sure that `cargo nextest run -p mu-cli --test journal --locked` passes.
- [ ] 6.6 Add `crates/mu-cli/tests/resume.rs` with `PortalClient` and the relay. Add the tests `resume_after_cut_connection` (scenario: resume after a cut connection) and `resume_after_server_restart` (scenario: resume after a restart of the server). Add `resume_after_clean_end`, which stops the server with SIGTERM, starts it again, and checks that the client resets and reads the new events. Make sure that `cargo nextest run -p mu-cli --test resume --locked` passes.

## 7. The terminal client

Load the `rust-best-practices` skill before this group.

- [ ] 7.1 In `mu-tui`, add `App` with `on_event`, `on_reply`, `on_key` and `draw`, and the input line on `ratatui-textarea`. Add `crates/mu-tui/tests/app.rs` with `TestBackend`, and one test for each scenario. Make sure that `cargo nextest run -p mu-tui --locked` passes, with these tests:
  - `first_view`, `world_order` and `no_worlds`
  - `line_prefixes` (requirement: lines show their source), which checks the prefixes `> ` and `-- `, and a game line with no prefix
  - `enter_sends_to_world_in_focus`, `empty_enter_sends_empty_line`, `enter_with_no_world` and `refused_line_keeps_input`
  - `ctrl_n_moves_focus`, `ctrl_p_wraps_around`, `ctrl_n_wraps_at_the_end`, `keys_with_no_world`, `one_world_keeps_focus` and `activity_mark`
  - `refused_line_keeps_input` feeds a refused reply to `on_reply`
  - `lost_stream_resumes`, which feeds `Lost`, then `Resumed` and the line once, and checks the top row and the rows
  - `resumed_clears_disconnected`, which feeds `Lost` and then `Resumed` with no event
  - the property test `focus_is_always_valid`, for any sequence of keys and events
- [ ] 7.2 Add the loop that joins `crossterm` keys, `PortalClient::events()` and the replies. The loop uses the alternate screen. It restores the terminal on exit and on a panic. Wire `mu client` to it, with the steps of the design: the check for a terminal first, then `private_dir`, then the start of a server. Make sure that `cargo build -p mu-cli --locked` passes.
- [ ] 7.3 Add `crates/mu-cli/tests/client.rs`. It runs the built `mu client` with `portable-pty`, in a pseudo-terminal of 80 by 24. It reads the screen with `vt100` and waits through `wait_until`. A failure prints the screen. Make sure that `cargo nextest run -p mu-cli --test client --locked` passes on Linux and on macOS in CI, with these tests:
  - `client_starts_a_server`, which also checks the mode 0700 of the new directory
  - `two_clients_start_one_server`, which checks that each screen shows `no worlds`, and that `server.log` holds exactly one line that names the socket as served
  - `client_gives_up_after_five_seconds` (scenario: no server answers in time), which also checks that the client still runs at 4 seconds
  - `client_needs_a_terminal_on_stdin`, which uses `mu_on_pty` to give the client a terminal on stdout and a pipe on stdin
  - `lost_stream_resumes_through_relay` (scenario: a lost stream)
  - `ctrl_c_quits_client_only`, which checks with `ListWorlds` that each world is still `connected`

## 8. Documentation

- [ ] 8.1 Remove the rule for a verb without an implementation from `docs/src/content/docs/contributing/openspec.md`. Change the text that says the verbs print `not implemented` in `README.md` and `docs/src/content/docs/index.md`. Add the page `docs/src/content/docs/guides/first-run.md`. It shows how to start a server with two fake worlds and attach a client. Add a Guides group for the directory `guides` to the sidebar in `docs/astro.config.mjs`. Make sure that `mise run docs:check` passes, that the page is in the sidebar of the built site, and that `rg -n 'not implemented' README.md docs/src` prints nothing.

## 9. Review and checks

- [ ] 9.1 Run `/sb:openspec-review portal-skeleton implementation` and treat changes-requested as a blocker. Make sure that `review.md` has a verdict that is not changes-requested.
- [ ] 9.2 Run `mise run check` and `uv run ~/.agents/plugins/sb/skills/review-lint/scripts/review_lint.py --base origin/main`. Make sure that both pass.

## Workflow follow-up

- After the merge, archive the change with the post-merge skill. The acceptance command is `mise run check`.
