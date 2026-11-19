# Tasks

## 1. Sources

- [x] 1.1 Read the current documentation for each third-party fact that the design uses:
  - `tokio::net::TcpStream::connect` with a `(host, port)` pair, and whether it resolves the name on each call
  - `TcpStream::set_nodelay`
  - `tokio::io::duplex`, and how a duplex stream behaves on the paused clock
  - the exponential builder of `backon`: its jitter, and whether a jitter can grow with each attempt up to a cap
  - how `tracing-opentelemetry` reads `otel.status_code` on a span that is already open
  - whether Cargo accepts a dev-dependency cycle between a crate and a member that depends on it, for integration tests

  Add a table "Checked facts" to the design with one row for each fact and its source. Make sure that each difference from the sources is an edit of the design that its row names.

## 2. The crate mu-fake-server

Load the `rust-best-practices` skill before this group.

- [x] 2.1 Add `ALLOWED_DEV_EDGES` to `crates/mu-cli/tests/workspace_boundary.rs`, with the rule of design decision 13. Add `dev_edge_allows_dev_dependency` and `dev_edge_refuses_normal_dependency`, on the fixture of that file. Make sure that both tests pass, and that the second test fails while the rule reads `ALLOWED_DEV_EDGES` as normal edges.
- [x] 2.2 Create `crates/mu-fake-server` and move `crates/mu-core/src/fake.rs` into it with no change in behavior. Move the tests of the fake with it, the test of "The fake world answers a mark in order" among them. Point `mu-cli`, `crates/mu-core/tests` and `crates/mu-cli/tests/telemetry_engine.rs` at the new crate. Add the edges of design decision 13, the crate to the LikeC4 model in `architecture/`, and the crate to `docs/src/content/docs/architecture/crate-layout.mdx`. Make sure that `cargo nextest run --workspace --locked` passes with no other change to a test, and that `mise run docs:check` passes.
- [x] 2.3 Add the scripted fake game to `mu-fake-server`. Its steps are send bytes, wait for bytes, go silent and close. It runs over TCP on `127.0.0.1:0` or over a duplex stream, and it records every byte that the client wrote. It also records each connect and refuses a connect on request. Add unit tests in the crate for each step, for a refused connect, and for the record of the bytes. Make sure that the tests pass, and that the crate does not use `TelnetFramer` or `LineFramer` for the scripted game.
- [x] 2.4 Add transcript replay to `mu-fake-server`. It takes the bytes of one transcript from the caller. Add `replay_reads_each_transcript` to `crates/mu-core/tests/telnet_transcripts.rs`. It replays each transcript of `crates/mu-core/tests/fixtures/telnet` into a `TelnetFramer` over the duplex stream. Make sure that the test passes for all six transcripts, and that no file in `crates/` reads a path in `spikes/`.

## 3. The configuration fields

Load the `rust-best-practices` skill before this group.

- [x] 3.1 Add `Keepalive` and `Reconnect` to `mu-core`, as design decision 4 describes, with no behavior yet. Add `autoconnect`, `keepalive.mode` and the table `reconnect` to the layers, and resolve them into `ResolvedWorld` (design decision 10). This extends the field list of the requirement "Each field is known" and the ranges of the requirement "Each value is in its range". Add the defaults to `WorldDefaults`, and the rule that `reconnect.initial` is not greater than `reconnect.max`. Add these tests to `crates/mu-cli/tests/world_config.rs`:
  - `unknown_mode`, for "An unknown mode"
  - `edges_of_the_attempts`, for "The edges of the attempts"
  - `connection_defaults`, for "The defaults"
  - `first_wait_over_the_cap`, for "A first wait over the cap"
  - `first_wait_equal_to_the_cap`, for "A first wait equal to the cap"

  Extend the test of "The edges of each range" with `reconnect.initial` and `reconnect.max`, and pass `reconnect.max = 86400` beside each edge of `reconnect.initial`. Make sure that the five new tests and the extended test pass. Also make sure that the tests of "An unknown field" and "A port out of range" still pass.
- [x] 3.2 Change the keepalive rule of the requirement "A world has a place to connect". `keepalive.mode = "off"` lifts the rule that a keepalive has both halves, and a world with neither half resolves to no keepalive. Add `half_keepalive_that_is_off` and `mode_with_no_keepalive` to `crates/mu-cli/tests/world_config.rs`, for "Half a keepalive that is off" and "A mode with no keepalive". Make sure that both pass, and that the tests of "Half a keepalive" and "A world with no host or port" still pass.
- [x] 3.3 Give the quirk schema of the built-in types its own keepalive table without `mode`, as design decision 10 describes. Add `quirks_refuse_connection_fields` to `crates/mu-core/src/world/builtin.rs`. It calls `parse_quirks` with `keepalive.mode`, with `autoconnect` and with a `[reconnect]` table, and expects each call to fail and name the field. Add `autoconnect` to the fields that a shared plugin layer cannot set. Extend the test of "A plugin sets a field of the connection" with it. Make sure that the new test, the extended test and the test of "Each built-in type reads as quirks" pass. Also make sure that the new test fails while the schema reuses `KeepaliveFields`.
- [x] 3.4 Describe the new fields, their defaults and their ranges in `docs/src/content/docs/guides/configuration.md`. Make sure that each example in the page resolves with `mu world show --config <dir>`, and that the page names the source `default` for each default.

## 4. The strategies and policies

Load the `rust-best-practices` skill before this group.

- [x] 4.1 Add the trait `LoginMoment` with `Immediate` and `Quiet` to `mu-core`, as pure methods over the tokio `Instant` (design decision 3). Add unit tests for a banner and then quiet, for a silent game, for a game that is never quiet, and for the cap. Make sure that the tests pass, and that `Quiet` holds no timer.
- [x] 4.2 Add the deadline methods of `Keepalive` for `Idle` and `Interval`. Add unit tests for the next deadline after the move to `connected`, after a write and after a keepalive, for each mode. Make sure that the tests pass.
- [x] 4.3 Add the wait of design decision 5 to `Reconnect`, with the saturation of the doubling. If task 1.1 found that `backon` fits, use it. Otherwise the policy takes the random draw as an argument. Add `extra_time_reaches_its_cap`, a property test for "The extra time at its cap", up to the largest attempt number. Add unit tests for the reset after `max` seconds and for no reset after `max - 1` seconds. Make sure that the tests pass.
- [x] 4.4 Add the cause of design decision 2 to `mu-core`. Add `cause_map`, a unit test of each row of the table of design decision 2: the reconnect choice, the value of `mu.connection.end` and the span status. Make sure that the test passes.

## 5. The session

Load the `rust-best-practices` skill before this group. The tests of this group are engine tests on the paused clock in `crates/mu-core/tests/connection.rs`. They use the scripted fake game over a duplex stream.

- [x] 5.1 Make `SessionParts` hold a `SecretSource` that reads through `load_password`, as design decision 7 describes. Replace `PortalError::NoPassword` with `PortalError::NoSecret`, and `NoTransport` with `TlsNotSupported`, in `mu-core` and in `crates/mu-server/src/convert.rs`. Make sure that every current test still passes, and that the fake world still reads its password at the start.
- [x] 5.2 Add the connect function of design decision 1, with the 10 second timeout and the span. Make the `Connect` handler use it. Give `Session::down` the cause of task 4.4. Add `connect_that_hangs`, for "A connect that hangs". Make sure that it passes, and that the current tests of `crates/mu-core/tests/portal.rs` still pass.
- [x] 5.3 Make the session send the login at the moment that its `LoginMoment` gives. Keep the world in `connecting` until then (design decision 3). Control frames wait in the queue, and `Send` fails. Add these tests:
  - `login_after_banner_and_quiet`, for "A banner and then quiet"
  - `login_after_a_silent_second`, for "A silent game"
  - `login_cap_for_a_noisy_game`, for "A game that never goes quiet"
  - `send_during_quiet_second_is_refused`, for "A send during the quiet second"
  - `no_login_sends_no_line`, for "A world that logs in by hand" of `world-connection`
  - `wait_covers_pending_login`, for "A wait outlasts the quiet before the login"

  Make sure that the six tests pass, and that the tests of the fake world in `crates/mu-cli/tests/worlds.rs` still pass.
- [x] 5.4 Make the session send the keepalive from the move to `connected`, with no journal event. Add `keepalive_idle`, `keepalive_interval`, `keepalive_off`, `journal_holds_no_keepalive` and `no_keepalive_before_login`, for the five scenarios of "The keepalive keeps an idle world connected". Make sure that the five tests pass.
- [x] 5.5 Add `Conn::Waiting` and `WorldState::Reconnecting`, and the reconnect after a drop through `Reconnect` (design decisions 1 and 5). Each attempt calls the connect function of task 5.2. Add these tests:
  - `waits_grow_until_the_last_attempt`, for "The waits of a game that refuses every connect"
  - `world_stops_after_last_attempt`, for "The last attempt"
  - `short_connections_count_as_attempts`, for "A game that closes each connection after the login"
  - `attempts_zero_never_stops`, for "No limit"
  - `stable_connection_resets_the_count` and `short_connection_keeps_the_count`, for the two scenarios of the reset
  - `secret_gone_at_reconnect_stops`, for "A secret that is gone at a reconnect"
  - `connect_during_a_wait` and `disconnect_during_a_wait`, for the two scenarios of "Connect and Disconnect act on a world that waits"
  - `wait_returns_during_reconnect_wait`, for "A wait on a world that waits to reconnect"
  - `send_to_reconnecting_world`, for "A world that waits to reconnect" of "A command reaches only its world"

  Make sure that the eleven tests pass.
- [x] 5.6 Put each write to the game and its flush under the write timeout of design decision 1. Add `write_that_hangs`, for "A write that hangs". Make sure that it passes.

## 6. The TCP connection of a configured world

Load the `rust-best-practices` skill before this group. The tests of this group are binary tests in `crates/mu-cli/tests/connection.rs`. They use the scripted fake game over TCP.

- [x] 6.1 Add `TcpTransport` to `mu-core`, as design decision 6 describes. Make `mu-cli` give each configured world a `TcpTransport`, a `TelnetFramer` with its character set and window size (design decision 8), and a `SecretSource::Store`. Add these tests:
  - `world_reaches_its_game` and `nothing_listens_on_the_port`, for the two binary scenarios of "A configured world connects over TCP"
  - `tls_world_is_refused`, for "A TLS world"
  - `latin1_game` and `window_size_of_the_world`, for the two scenarios of "A configured world speaks telnet in its character set and window size"
  - `missing_secret_variable_at_connect` and `login_line_hides_the_password`, for the two binary scenarios of "The secret is read at each connect"

  Make sure that the seven tests pass.
- [x] 6.2 Add these tests for "A world reconnects after a drop":
  - `game_close_reconnects`, for "The game closes the connection"
  - `player_disconnect_does_not_reconnect`, for "The player disconnects"
  - `reconnect_off_stays_disconnected`, for "Reconnect off"

  Add `fake_world_does_not_reconnect` to `crates/mu-cli/tests/worlds.rs`, for "A fake world does not reconnect". Make sure that the four tests pass.

## 7. The start and the API

Load the `rust-best-practices` skill before this group.

- [x] 7.1 Replace `Portal::connect_all` with `Portal::connect_at_start`, as design decision 9 describes. In `crates/mu-cli/tests/worlds.rs`, add these tests:
  - `autoconnect_world_connects_at_start`, for "A configured world that connects at the start"
  - `failed_autoconnect_does_not_stop_start`, for "A failed connect at the start"

  Replace `configured_world_cannot_connect_yet` with `configured_world_connects_on_request`, for "A configured world connects on request". Make `configured_world_waits` set no `autoconnect`. Make sure that the four tests and `two_worlds_connect` pass.
- [x] 7.2 Add `WORLD_STATE_RECONNECTING` and `next_attempt_ms` to `portal.proto`, as design decision 11 describes, and map them in `mu-server`. Add `list_worlds_shows_reconnecting` to `crates/mu-cli/tests/worlds.rs`, for "A world that waits to reconnect" of "ListWorlds gives the state of each world". Make sure that the test passes, and that the existing enum values and field numbers of `World` do not change.

## 8. Telemetry

Load the `rust-best-practices` skill before this group.

- [x] 8.1 Record `mu.connection.end` and the span status of design decision 2 on the `world.connection` span. Record the status `error` on a failed `command` span (design decision 12). Add these tests to `crates/mu-cli/tests/telemetry.rs`:
  - `failed_command_span_is_error`, for "A command that fails"
  - `failed_connect_span_is_error`, for "A connect that fails"
  - `closed_connection_span_is_not_error`, for "The game closes the connection" of the telemetry delta

  Add `lost_connection_span_is_error` to `crates/mu-cli/tests/telemetry_engine.rs`, for "A connection that is lost". Extend the test of "The connection of a fake world" with `mu.connection.end` equal to `stopped`. Make sure that the four new tests and the extended test pass.
- [x] 8.2 Count each keepalive in `mu.keepalive.sent`, and add `reconnecting` to the gauge `mu.world.state`. Add `keepalive_lines_are_counted` and `gauge_shows_reconnecting` to `crates/mu-cli/tests/telemetry_engine.rs`, for "The keepalive lines" and "A world that waits to reconnect" of the telemetry delta. Describe the new metric and attribute in `docs/src/content/docs/guides/telemetry.md`. Make sure that both tests pass, and that `state_at_the_stop` and the test of "A world that never connects" still pass.

## 9. Integration

- [x] 9.1 Run `mise run check` and `mise run docs:check` on the branch. Make sure that both pass, and that `openspec validate world-sessions --type change --strict` passes.
- [ ] 9.2 Run `/sb:review-lint` and `/sb:suite-review` on the branch before the pull request. Make sure that each finding is fixed or answered in the pull request.

## Workflow follow-up

- Run the implementation review with `/sb:openspec-review world-sessions implementation` before the merge.
- After the merge, archive the change with `/sb:post-merge`, and close #54 and #95.
