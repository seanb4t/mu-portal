# Tasks

## 1. Sources and dependencies

- [ ] 1.1 Read the current documentation of `opentelemetry-otlp`, `opentelemetry_sdk` and `tracing-opentelemetry` for each fact that the design uses:
  - the field prefixes of `MetricsLayer`, the `gauge.` prefix among them, and how its other fields become attributes
  - how `OpenTelemetryLayer` reads `otel.status_code`, and how it turns an event in a span into a span event
  - the thread of the default `BatchSpanProcessor` and of the `PeriodicReader`, the default queue size, and what `on_end` does with a full queue
  - the `reqwest-blocking-client` feature: where the client must be built, and `with_http_client`
  - the `grpc-tonic` feature: that the build needs a tokio runtime, and `with_tls_config` with `ClientTlsConfig`
  - the TLS features of `opentelemetry-otlp`, `reqwest` and `tonic`: the roots of the system, a PEM file, and one rustls crypto provider that all of them support
  - whether `opentelemetry-otlp` re-exports the `tonic` and `reqwest` types that the build needs
  - `with_endpoint` of the HTTP exporter: whether it adds a path to the URL, and whether it wins over the signal variables
  - `Resource::builder().with_service_name`: whether it wins over `OTEL_SERVICE_NAME`
  - `shutdown` of each provider: whether it flushes the spans and collects the metrics one last time
  - the in-memory exporters of `opentelemetry_sdk` for the tests, and their feature

  Add a table "Checked facts" to the design with one row for each fact and its source. Make sure that each difference from the sources is an edit of the design that its row names.
- [ ] 1.2 Add `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `tracing-opentelemetry` and `opentelemetry-proto` to `[workspace.dependencies]`. Give `opentelemetry-otlp` the features `http-proto`, `reqwest-blocking-client`, `grpc-tonic`, `trace` and `metrics`, and the TLS features that task 1.1 picks. Add the first four, `serde` and `http` to the dependencies of `mu-cli`. If task 1.1 finds no re-export of the `reqwest` and `tonic` types, add those two crates too. Add `opentelemetry-proto` with its gRPC services, `tonic`, `axum`, `axum-server` and `rcgen` to its dev-dependencies, and the testing feature of `opentelemetry_sdk`. Add the new crates of `mu-cli` to `ALLOWED_CRATES` in `crates/mu-cli/tests/workspace_boundary.rs`. Make sure that `cargo deny check` and `cargo nextest run -p mu-cli --test workspace_boundary --locked` pass, and that Renovate reads each new pin from `Cargo.toml`.

## 2. server.toml

Load the `rust-best-practices` skill before this group.

- [ ] 2.1 Make the walk of `TomlDirSource` in `crates/mu-core/src/world/dir.rs` skip the relative path `server.toml`, at the top level only. Add `server_file_makes_no_world` and `character_named_server_is_a_world` to `crates/mu-cli/tests/world_config.rs`, for the scenarios "The file of the server" and "A character named server". Make sure that `stray_file` still passes, and that `cargo nextest run -p mu-cli --test world_config --locked` passes.
- [ ] 2.2 Make `world::parse` public as `mu_core::world::parse_toml`. Add `crates/mu-cli/src/server_config.rs`, which reads `server.toml` through it into a struct with `deny_unknown_fields` on the outer struct and on `[telemetry]`. Add `check_endpoint`, and the read of the certificate file, as the design describes. A missing file gives no telemetry table. Make `server::run` read the file before the socket opens and stop on an error. Add `crates/mu-cli/tests/telemetry.rs` with these tests, for the five scenarios of "Each field of server.toml is known":
  - `unknown_field_in_server_toml` and `unknown_table_in_server_toml`
  - `value_of_the_wrong_type`, `endpoint_that_is_not_a_url` and `endpoint_with_another_scheme`

  Make sure that the five tests pass.

## 3. The export

Load the `rust-best-practices` skill before this group.

- [ ] 3.1 Add to `crates/mu-cli/src/telemetry.rs` the function that picks the endpoint of each signal and the service name. It follows the order of the design. It takes the values of the variables and the endpoint of `server.toml` as arguments. It checks the three endpoint variables with `check_endpoint`, and the three protocol variables only while a signal has an endpoint. Add unit tests:
  - each step of the order, an empty variable, and one signal with no endpoint
  - `OTEL_SDK_DISABLED` with `true` and `TrUe` (off), and with `false`, `1` and the empty value (the order holds)
  - the order of the protocol, and `grpc` and `http/protobuf` from each source
  - `http/json` in each of the three protocol variables and in the field, with an endpoint: refused, and the error names the source
  - `http/json` with no endpoint, and with the export off: accepted
  - `ftp://` and `not a url` in each of the three endpoint variables: refused, and the error names the variable
  - the join of `http://h:4318/` and of `http://h:4318/otel` with each signal path, and no join with `grpc`
  - the order of the certificate file, a missing file, and a file with no PEM certificate
  - the service name with the variable unset, empty and set
  - a `proptest` block over bases with and without a trailing slash and a path, and over random sets of the variables. It checks that no URL holds `//` before `v1/`, and that a set step always wins over each lower step

  Make sure that `cargo nextest run -p mu-cli --lib --locked` passes.
- [ ] 3.2 Add the function that builds the three layers from a span exporter and a metric exporter, with the resource of the design. Give the two OpenTelemetry layers the filter that passes only the targets `mu_core`, `mu_server`, `mu_cli` and `mu::metrics`. Make `server::run` build the HTTP exporters with their own `reqwest` client before the runtime. It builds the gRPC exporters inside `Runtime::enter`. Each exporter gets the TLS of the design. Install the subscriber. After `serve` returns, it shuts the providers down on a helper thread, and waits at most five seconds. Without an endpoint, install only the `fmt` layer. Add `other_targets_make_no_span_event` to `crates/mu-cli/tests/telemetry_engine.rs`. It sends an event with the target `h2` inside a span, under the in-memory exporters, and checks that the span has no event. Make sure that it passes, and that `cargo nextest run -p mu-cli --locked` passes.
- [ ] 3.3 Make `test_env` remove each variable of the environment whose name starts with `OTEL_`. Add the test `helper_removes_the_otel_variables`. It sets `OTEL_EXPORTER_OTLP_COMPRESSION` in the test process, and checks that a `mu` child does not see it. Make sure that it passes.
- [ ] 3.4 Add `crates/mu-cli/tests/support/collector.rs`, the fake collector of the design, with its five modes and the test authority of `rcgen`. Make sure that the tests of 3.5 use it and pass.
- [ ] 3.5 Add these tests to `crates/mu-cli/tests/telemetry.rs`:
  - `server_without_endpoint_writes_only_the_serving_line`, for "A server with no endpoint"
  - `traces_and_metrics_reach_the_collector`, `base_url_with_a_trailing_slash` and `log_lines_stay`, for the three scenarios of "The endpoint of server.toml"
  - `variable_wins_over_the_file`, `endpoint_of_the_traces`, `endpoint_of_the_metrics` and `empty_variable_counts_as_unset`, for the four scenarios of "The standard variables"
  - `bad_variable_is_refused` and `bad_variable_of_one_signal_is_refused`, for the two scenarios of "Each endpoint variable is checked". Make the second a table test over the two variables and the two values
  - `sdk_disabled_turns_the_export_off`, `default_service_name`, `empty_service_name` and `service_name_from_the_variable`, for the four scenarios of the variables that turn the export off and name the service
  - `grpc_from_the_file`, `variable_wins_over_the_protocol_of_the_file` and `protocol_of_one_signal`, for the three scenarios of "The protocol of the export"
  - `http_json_is_refused`, `unknown_protocol_in_the_file` and `unknown_protocol_with_the_export_off`, for the three scenarios of "An unknown protocol is refused"
  - `https_with_a_certificate_file`, `grpc_with_tls`, `collector_that_the_server_does_not_trust`, `missing_certificate_file` and `file_with_no_certificate`, for the five scenarios of "TLS for an https endpoint"

  Make sure that each test passes. Break each rule by hand, and make sure that its test then fails.

## 4. The spans

Load the `rust-best-practices` skill before this group.

- [ ] 4.1 Add a span named `PortalService/<method>` to each method of `PortalApi`. Give it `mu.world` for a call that names a world, and `otel.status_code` for a failure. The span of `Subscribe` closes before the method returns its stream. Add `span_of_a_send`, `span_of_a_subscribe` and `span_of_a_failed_call` to `crates/mu-cli/tests/telemetry.rs`, for the three scenarios of "A span for each API call". Make sure that the three pass.
- [ ] 4.2 Add `Command::name` and `Origin::name`, each with a unit test that lists every variant. Add the field `span` and the constructor `Request::command` to `session.rs`. Use the constructor in `Portal::submit`, `Portal::stop`, `Portal::connect_all` and `route`. Make the `command` span in `Session::handle` with that span as the parent. Rename the field of the `warn!` in `route` to `mu.world`. Add `command_under_its_call` and `command_of_the_server`, for the two scenarios of "A span for each command of a world". Make sure that both pass, and that `cargo nextest run -p mu-core --locked` passes.
- [ ] 4.3 Add the `world.connection` span to the `Conn` state of the session, from the start of a connect to `down`. Add `connection_of_a_fake_world`, for its scenario. Make sure that it passes.

## 5. The metrics

Load the `rust-best-practices` skill before this group.

- [ ] 5.1 Add `EventKind::name`, with a unit test that lists every kind. Emit `mu.journal.events` in `Session::record`, with the target `mu::metrics`. Give `event_stream` the journal as an argument, and pass it from `PortalApi::subscribe` and the tests of `service.rs`. Emit `mu.subscriber.lag` and `mu.subscriber.lagged` there, as the design describes. Add `events_of_a_world` and `lag_of_a_subscriber` to `crates/mu-cli/tests/telemetry.rs`. Add `lagged_stream_is_counted` and `lag_at_the_head_is_zero` to `crates/mu-cli/tests/telemetry_engine.rs`, which run `event_stream` under the layers of 3.2 with the in-memory exporters. Make sure that the four tests pass.
- [ ] 5.2 Emit `mu.commands.dropped` and `mu.script.errors` in `Session::handle` and `Session::script_error`. Emit `mu.world.state` at the start of `session::run` and in `Session::set_state`. Include `crates/mu-core/tests/support/mod.rs` in `telemetry_engine.rs` with a `#[path]` module, and add these tests there, under the in-memory exporters:
  - `command_past_the_depth_limit`, with a handler from `on` that sends a command for each event
  - `command_over_the_rate`, with a paused clock and a handler that sends 25 commands for one event. It expects 5 in `mu.commands.dropped`
  - `world_without_transport_is_disconnected`, with `WorldSetup::without_transport`

  Add `world_that_never_connects` and `state_at_the_stop` to `telemetry.rs`. Make sure that the five tests pass.

## 6. Secrets and a slow collector

- [ ] 6.1 Add `password_stays_out` and `line_text_stays_out` to `crates/mu-cli/tests/telemetry.rs`. Each one scans each body that the collector got, and stderr. Make sure that both pass. Put the text of the `Send` in a span field by hand, and make sure that both then fail.
- [ ] 6.2 Add `silent_collector_does_not_delay_a_send` for the scenario "A collector that never answers", with the silent mode of the collector and 2100 calls. Run it for each protocol. It measures the time from SIGTERM to the exit, and expects at most seven seconds. Make sure that it passes. Remove the bound of the stop by hand, and make sure that the test then fails.

## 7. Docs and follow-ups

- [ ] 7.1 Add a page for `server.toml` and the telemetry to `docs/src/content/docs/guides/`, and link it from `configuration.md`. It names the field, the variables that work, and the variables that this change does not claim. It also says to remove or rename `server.toml` before a downgrade to an earlier binary. Make sure that `mise run docs:check` passes.
- [ ] 7.2 Add the decision page `docs/src/content/docs/decisions/telemetry.md`. It states the `tracing` seam and the target filter. It states the two transports, the trust roots and the crypto provider. It also states the names of the spans and the metrics, and the rule that a field holds no text. Make sure that `mise run docs:check` passes.
- [ ] 7.3 Open a Parked issue for the OTLP logs signal, and one for client certificates of the exporter. Each issue describes the gap only. Make sure that each issue exists in the Project with the status Parked.

## 8. Integration

- [ ] 8.1 Run `/sb:openspec-review telemetry implementation`. Make sure that the verdict is not changes-requested.
- [ ] 8.2 Run `/sb:suite-review` on the branch. Fix or triage each blocker and major finding. Make sure that `mise run check` passes.

## Workflow follow-up

- Open the pull request, and link issue #63.
- After the merge, archive the change with `/sb:post-merge`.
