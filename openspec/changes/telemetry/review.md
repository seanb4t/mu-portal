---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-18T02:56:57+00:00
head: 8a22dd1a0635fa61dd433559500c058ab11b6043
round: 5
next_id: 80
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: ok
  invariants: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-66 | approach,invariants | should-fix | in-change | app-1: The walk skips each top-level entry that is the same file (device and inode) as server.toml, so another case of the name, a hard link or a symlink to it is skipped. design.md still says the check compares the whole relative path, and records no deviation.; inv-1: The walk now skips any top-level file with the inode of server.toml. A hard link worlds.toml lists worlds with exit 0, and a link server.toml to global.toml drops the global layer with no error. | crates/mu-core/src/world/dir.rs:42 — "([name], Some(server)) => !is_server_file(&self.root.join(name), server)," | app-1: Rewrite the dir.rs sentence of 'Where the endpoint comes from' to say that the skip compares file identity at the top level, and give the reason: server_config reads the same file on a file system that ignores case. Also state what happens to a top-level link to server.toml.; inv-1: Skip a top-level entry only when its name equals server.toml ignoring ASCII case and its identity matches. Add a test with a hard link worlds.toml to server.toml (exit 1, names worlds.toml) and one with server.toml linked to global.toml (the global field stays in mu world show). |  |  |
| R-67 | approach | should-fix | in-change | The guide and the Risks section say that each failed export gives one stderr line. At head a failed export gives two: the WARN of the OTLP retry (Export.Failed.Exhausted) and the ERROR of the batch processor or the reader. The design's TLS decision and collector_that_the_server_does_not_trust already expect both. | docs/src/content/docs/guides/telemetry.md:68 — "The server writes a line to stderr for each failed export, and one line at the first drop." | In the Risks bullet and in the guide (lines 64 and 68), say that each failed export gives two lines: a WARN from the exporter and an ERROR from the processor or the reader. |  |  |
| R-68 | approach | should-fix | in-change | The design now stops each provider on a helper thread of its own, in parallel, so a slow trace export does not delay the last metrics, and the code does the same. The decision page of task 7.2 still says one helper thread shuts the providers down. | design.md > The exporter has its own threads and a bounded stop — "`server::run` calls `shutdown` of each provider on a helper thread of its own, so a slow trace export does not delay the" | Change line 24 of decisions/telemetry.md to say that each provider stops on its own helper thread, and that the server waits at most five seconds for both. |  |  |
| R-69 | fit | should-fix | in-change | dir.rs hand-writes a file-identity compare (dev and ino, with a name fallback off unix) to skip server.toml. same_file::is_same_file does exactly this, and same-file is already in Cargo.lock through walkdir. The design names neither the library nor the identity check: it still says the skip compares the relative path. | crates/mu-core/src/world/dir.rs:105 — ".is_ok_and(\|file\| (file.dev(), file.ino()) == (server.dev(), server.ino()))" | Call same_file::is_same_file from is_server_file, with same-file added through [workspace.dependencies] and to ALLOWED_CRATES of mu-core. Or state in 'Where the endpoint comes from' that the skip compares file identity, and why a hand compare wins over same-file. |  |  |
| R-70 | fit | should-fix | in-change | The span layer now filters with a hand matcher, in_crates, where tracing_subscriber::filter::Targets is the library filter for targets. The reason is real: Targets matches a bare prefix, so mu_cli would also take mu_client, a crate of the workspace. But only a test holds that reason. The design names neither Targets nor why it does not fit. | crates/mu-cli/src/telemetry/pipeline.rs:274 — ".is_some_and(\|rest\| rest.is_empty() \|\| rest.starts_with("::"))" | Keep in_crates. Add one sentence to the decision 'The engine speaks tracing, and mu-cli bridges it': Targets matches a bare prefix, so `mu_cli` would take `mu_client`, and the filter therefore needs the `::` boundary. |  |  |
| R-71 | invariants | should-fix | in-change | A late or failed stop of a provider must write a WARN line, but no test reads stderr after a stop past the bound; the silent-collector test asserts only the status and the elapsed time. | design.md > The exporter has its own threads and a bounded stop — "A failed shutdown, or a wait that ends first, writes a line at `WARN` to stderr." | In silent_collector_with, also assert that stderr holds a WARN line with 'did not finish within'. A stop that ends in time must not write it: assert its absence in traces_and_metrics_reach_the_collector. |  |  |
| R-72 | invariants | should-fix | in-change | The HTTP client must follow no redirect, but no test makes the collector answer with a redirect, so a removal of Policy::none passes every test. | design.md > Two transports and one TLS stack — "The client follows no redirect." | Add a collector mode or route that answers POST /v1/traces with 307 to /elsewhere, and assert that the collector got no request at /elsewhere. |  |  |
| R-73 | invariants | should-fix | in-change | The metrics layer must take only the target mu::metrics, but no test emits a metric-prefixed field under another target such as mu_core, the side the filter must refuse. | design.md > The engine speaks tracing, and mu-cli bridges it — "It takes only the target `mu::metrics`." | Add to telemetry_engine.rs a test that emits monotonic_counter.mu.x with the target mu_core under the in-memory exporters and expects no metric, beside one with METRICS_TARGET that expects the value. |  |  |
| R-74 | invariants | should-fix | in-change | check_endpoint now states that no URL can hold a fragment, the signal variables too, but the only fragment test passes # in OTEL_EXPORTER_OTLP_ENDPOINT, not in a signal variable or the field. | crates/mu-cli/src/telemetry.rs:249 — "/// Checks that `value` is a URL with the scheme `http` or `https` and a host. No URL can hold a" | Add http://t:1/x#top in OTEL_EXPORTER_OTLP_TRACES_ENDPOINT and OTEL_EXPORTER_OTLP_METRICS_ENDPOINT to a refusal test that expects the variable as the source. http::Uri drops a fragment, so only the raw check refuses it. |  |  |
| R-76 | test-coverage | should-fix | in-change | covered row 'Worlds from character files' has no verification run | review > not-run — "Worlds from character files" | run the covering test and record it in verification-runs.txt |  |  |
| R-77 | test-coverage | should-fix | in-change | covered row 'A character with no game' has no verification run | review > not-run — "A character with no game" | run the covering test and record it in verification-runs.txt |  |  |
| R-78 | test-coverage | should-fix | in-change | covered row 'A file name outside the id rule' has no verification run | review > not-run — "A file name outside the id rule" | run the covering test and record it in verification-runs.txt |  |  |
| R-79 | test-coverage | should-fix | in-change | covered row 'A link to nothing' has no verification run | review > not-run — "A link to nothing" | run the covering test and record it in verification-runs.txt |  |  |
| R-75 | fit | suggestion | in-change | The shared TOML parse of mu-core hard-codes the hidden tables of both callers: login, a table of a world layer, and telemetry, a table of server.toml, which mu-cli owns. Which tables to hide differs by caller on purpose. Each later table of server.toml that holds a secret must edit mu-core, and each layer file gets telemetry hidden too. | crates/mu-core/src/world/parse.rs:37 — "const HIDDEN: [&str; 2] = ["login", "telemetry"];" | Give parse (parse_toml) a parameter `hidden: &[&str]`. dir.rs passes ["login"], and server_config.rs passes ["telemetry"]. |  |  |

## Repeats

None.

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-64 | suggestion | accepted | Only the metric name and the attributes vary between the six events, so a macro adds a layer of indirection for little gain. |  |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 7a1e4068f5853f5661404f4a44c8436091ef9fb0 | cf9aee9b950be1f2ac52bd6cfe2c38ec60aa9fb624779f098f36c8e0e31e4ddd | changes-requested | 29 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 54b5a2a6975e8d1dce256af0a1aba44e2097b395 | d83e961fa6800173c9ff0af3121c363a4fdae0af6584080f2ba292996bf2de13 | approve-with-notes | 15 | 0 | 0 | 0 |  |
| 3 | implementation | full | 74942b0c11812f2f3e2184e0705c6a5df52a7991 | edba5d78053bcfb78a8ed98bf6877143508a1c966814b7c77e0b7d83656e8e52 | approve-with-notes | 17 | 0 | 1 | 0 |  |
| 4 | implementation | follow-up | bd78e021d40ef560f025ad64dd9e082ccc6d8964 | 6c7fee037f8473d7861ff8a1d21785330070bc82764e55b1ec4bb7655fef01d0 | approve-with-notes | 4 | 0 | 0 | 1 |  |
| 5 | implementation | follow-up | 8a22dd1a0635fa61dd433559500c058ab11b6043 | c02a832ad60eed7a2f566cbb7b23144c8efda88813a3ece30b7a631f4bc21700 | approve-with-notes | 14 | 0 | 0 | 0 |  |

## Judge

judge: 20 questions, 0.001 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A server with no endpoint | crates/mu-cli/tests/telemetry.rs: server_without_endpoint_writes_only_the_serving_line | covered | pass |
| Traces and metrics reach the collector | crates/mu-cli/tests/telemetry.rs: traces_and_metrics_reach_the_collector | covered | pass |
| A base URL with a trailing slash | crates/mu-cli/tests/telemetry.rs: base_url_with_a_trailing_slash | covered | pass |
| The log lines stay | crates/mu-cli/tests/telemetry.rs: log_lines_stay | covered | pass |
| An unknown field | crates/mu-cli/tests/telemetry.rs: unknown_field_in_server_toml | covered | pass |
| An unknown table | crates/mu-cli/tests/telemetry.rs: unknown_table_in_server_toml | covered | pass |
| A value of the wrong type | crates/mu-cli/tests/telemetry.rs: value_of_the_wrong_type | covered | pass |
| An endpoint that is not a URL | crates/mu-cli/tests/telemetry.rs: endpoint_that_is_not_a_url | covered | pass |
| An endpoint with a query | crates/mu-cli/tests/telemetry.rs: endpoint_with_a_query | covered | pass |
| An endpoint with another scheme | crates/mu-cli/tests/telemetry.rs: endpoint_with_another_scheme | covered | pass |
| The variable wins over the file | crates/mu-cli/tests/telemetry.rs: variable_wins_over_the_file | covered | pass |
| The endpoint of the traces | crates/mu-cli/tests/telemetry.rs: endpoint_of_the_traces | covered | pass |
| The endpoint of the metrics | crates/mu-cli/tests/telemetry.rs: endpoint_of_the_metrics | covered | pass |
| An empty variable | crates/mu-cli/tests/telemetry.rs: empty_variable_counts_as_unset | covered | pass |
| A bad variable | crates/mu-cli/tests/telemetry.rs: bad_variable_is_refused | covered | pass |
| A bad variable of one signal | crates/mu-cli/tests/telemetry.rs: bad_variable_of_one_signal_is_refused | covered | pass |
| The export is off | crates/mu-cli/tests/telemetry.rs: sdk_disabled_turns_the_export_off | covered | pass |
| The export is off with a bad variable | crates/mu-cli/tests/telemetry.rs: sdk_disabled_with_a_bad_variable | covered | pass |
| The default service name | crates/mu-cli/tests/telemetry.rs: default_service_name | covered | pass |
| An empty service name | crates/mu-cli/tests/telemetry.rs: empty_service_name | covered | pass |
| A service name from the variable | crates/mu-cli/tests/telemetry.rs: service_name_from_the_variable | covered | pass |
| gRPC from the file | crates/mu-cli/tests/telemetry.rs: grpc_from_the_file | covered | pass |
| The variable wins over the protocol of the file | crates/mu-cli/tests/telemetry.rs: variable_wins_over_the_protocol_of_the_file | covered | pass |
| The protocol of one signal | crates/mu-cli/tests/telemetry.rs: protocol_of_one_signal | covered | pass |
| The protocol http/json | crates/mu-cli/tests/telemetry.rs: http_json_is_refused | covered | pass |
| An unknown protocol in the file | crates/mu-cli/tests/telemetry.rs: unknown_protocol_in_the_file | covered | pass |
| An unknown protocol with the export off | crates/mu-cli/tests/telemetry.rs: unknown_protocol_with_the_export_off | covered | pass |
| HTTPS with a certificate file | crates/mu-cli/tests/telemetry.rs: https_with_a_certificate_file | covered | pass |
| gRPC with TLS | crates/mu-cli/tests/telemetry.rs: grpc_with_tls | covered | pass |
| A collector that the server does not trust | crates/mu-cli/tests/telemetry.rs: collector_that_the_server_does_not_trust | covered | pass |
| A missing certificate file | crates/mu-cli/tests/telemetry.rs: missing_certificate_file | covered | pass |
| A file that holds no certificate | crates/mu-cli/tests/telemetry.rs: file_with_no_certificate | covered | pass |
| The span of a Send | crates/mu-cli/tests/telemetry.rs: span_of_a_send | covered | pass |
| The span of a Subscribe | crates/mu-cli/tests/telemetry.rs: span_of_a_subscribe | covered | pass |
| A failed call | crates/mu-cli/tests/telemetry.rs: span_of_a_failed_call | covered | pass |
| A command under its call | crates/mu-cli/tests/telemetry.rs: command_under_its_call | covered | pass |
| A command of the server | crates/mu-cli/tests/telemetry.rs: command_of_the_server | covered | pass |
| The connection of a fake world | crates/mu-cli/tests/telemetry.rs: connection_of_a_fake_world | covered | pass |
| The events of a world | crates/mu-cli/tests/telemetry.rs: events_of_a_world | covered | pass |
| The lag of a subscriber | crates/mu-cli/tests/telemetry.rs: lag_of_a_subscriber | covered | pass |
| A stream that a lag ended | crates/mu-cli/tests/telemetry_engine.rs: lagged_stream_is_counted | covered | pass |
| A command past the depth limit | crates/mu-cli/tests/telemetry_engine.rs: command_past_the_depth_limit | covered | pass |
| A command over the rate | crates/mu-cli/tests/telemetry_engine.rs: command_over_the_rate | covered | pass |
| Commands at the rate | crates/mu-cli/tests/telemetry_engine.rs: commands_at_the_rate | covered | pass |
| A world that never connects | crates/mu-cli/tests/telemetry.rs: world_that_never_connects | covered | pass |
| The state at the stop | crates/mu-cli/tests/telemetry.rs: state_at_the_stop | covered | pass |
| The password stays out | crates/mu-cli/tests/telemetry.rs: password_stays_out | covered | pass |
| The line text stays out | crates/mu-cli/tests/telemetry.rs: line_text_stays_out | covered | pass |
| A collector that never answers | crates/mu-cli/tests/telemetry.rs: silent_collector_does_not_delay_a_send | covered | pass |
| Worlds from character files | existing world_config.rs tests (not in this change's diff) | covered | not-run |
| A character with no game | existing world_config.rs tests (not in this change's diff) | covered | not-run |
| A file name outside the id rule | existing world_config.rs tests (not in this change's diff) | covered | not-run |
| A stray file | crates/mu-cli/tests/world_config.rs: stray_file | covered | pass |
| A link to nothing | existing world_config.rs tests (not in this change's diff) | covered | not-run |
| The file of the server | crates/mu-cli/tests/world_config.rs: server_file_makes_no_world | covered | pass |
| A character named server | crates/mu-cli/tests/world_config.rs: character_named_server_is_a_world | covered | pass |

### Not run

- Worlds from character files: no verification run names this row
- A character with no game: no verification run names this row
- A file name outside the id rule: no verification run names this row
- A link to nothing: no verification run names this row

## Verification runs

- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=bad_variable_is_refused)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=bad_variable_of_one_signal_is_refused)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=base_url_with_a_trailing_slash)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=collector_that_the_server_does_not_trust)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=command_of_the_server)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=command_under_its_call)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=connection_of_a_fake_world)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=default_service_name)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=empty_service_name)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=empty_variable_counts_as_unset)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=endpoint_of_the_metrics)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=endpoint_of_the_traces)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=endpoint_that_is_not_a_url)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=endpoint_with_a_query)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=endpoint_with_another_scheme)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=events_of_a_world)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=file_with_no_certificate)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=grpc_from_the_file)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=grpc_with_tls)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=http_json_is_refused)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=https_with_a_certificate_file)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=lag_of_a_subscriber)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=line_text_stays_out)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=log_lines_stay)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=missing_certificate_file)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=password_stays_out)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=protocol_of_one_signal)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=sdk_disabled_turns_the_export_off)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=sdk_disabled_with_a_bad_variable)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=server_without_endpoint_writes_only_the_serving_line)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=service_name_from_the_variable)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=silent_collector_does_not_delay_a_send)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=span_of_a_failed_call)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=span_of_a_send)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=span_of_a_subscribe)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=state_at_the_stop)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=traces_and_metrics_reach_the_collector)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=unknown_field_in_server_toml)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=unknown_protocol_in_the_file)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=unknown_protocol_with_the_export_off)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=unknown_table_in_server_toml)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=value_of_the_wrong_type)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=variable_wins_over_the_file)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=variable_wins_over_the_protocol_of_the_file)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=world_that_never_connects)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry_engine --locked -E 'test(=command_over_the_rate)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry_engine --locked -E 'test(=command_past_the_depth_limit)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry_engine --locked -E 'test(=commands_at_the_rate)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry_engine --locked -E 'test(=lagged_stream_is_counted)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=character_named_server_is_a_world)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=server_file_makes_no_world)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=stray_file)'`

## Dropped

None.

## Lens notes

- approach: The approach lens has no Bash tool, so it could not diff against since bd78e021. It placed the changed text by the commit subjects after the base: identity skip of server.toml, hidden telemetry values, request limit, parallel stop and layer filters. The merge should check the bound of each finding against the real diff.
- approach: server_config.rs resolves a relative certificate field against the configuration directory, and the guide documents it. design.md names no rule for a relative path, and OTEL_EXPORTER_OTLP_CERTIFICATE is resolved against the working directory. Not raised: no decision is departed from.
- approach: parse.rs now drops the serde message under the telemetry table, and check_endpoint keeps the URL out of its message. design.md records neither rule. Both go beyond the spec, which names only the password of a world. Not raised.
- approach: The other decisions checked against head hold: the per-layer target filters, the ten-second EXPORT_TIMEOUT (with_timeout wins over the variables in resolve_timeout of opentelemetry-otlp 0.33.1), the ring install in server::run, internal-logs on the three OpenTelemetry crates, Request::command at the four sites, the command and world.connection spans, the lag from Journal::last, the gauge at the start of session::run, and the Subscribe past-the-end status.
- fit: Follow-up round from bd78e021. Every file of diff-files.txt was read or searched. The two docs guides were only searched for server.toml and OTEL_ names. They hold no component that the fit lens judges.
- fit: Fixed rows checked at head and still fixed: R-50 (one METRICS_TARGET in mu-core), R-51 (one SERVER_FILE, re-exported by mu-cli), R-52 (PORTAL_SERVICE_SERVICE_NAME), R-53 (shared grpc and http helpers), R-54 (is_https parses with http::Uri), R-55 (fails_naming in tests/support), R-60 (ring installed in server::run), R-61 (traced helper), R-65 (subscriber setup helper).
- fit: R-64 (accepted) still holds in service.rs and session.rs. It is not raised again, because there is no new evidence.
- fit: The OTEL_* name constants at the top of tests/telemetry.rs repeat the public constants of mu_cli::telemetry. This is not reported: test literals that restate spec names on purpose are excluded.
- fit: The library rows came from openspec/config.yaml context. The row for test helpers (tests/support/mod.rs) is followed: fails_naming, ConfigDir and stdout come from there.
- invariants: I1: The walk skips only the top-level server.toml, and each other top-level .toml file is an error \| enforced at crates/mu-core/src/world/dir.rs:40 by inode identity \| boundary test server_file_makes_no_world, server_file_in_another_case, character_named_server_is_a_world; none for an alias (see inv-1).
- invariants: I2: An error under [telemetry] of server.toml never quotes a value \| enforced at crates/mu-core/src/world/parse.rs HIDDEN and under \| boundary test error_under_telemetry_drops_the_message (path telemetry); the dot and bracket branches of under are shared with login and its test.
- invariants: I3: A relative certificate of server.toml starts at the configuration directory, and an absolute one is kept \| enforced at crates/mu-cli/src/server_config.rs read \| boundary test relative_certificate_starts_at_the_directory and absolute_certificate_is_kept.
- invariants: I4: The span layer takes a target only when it is a crate of CRATES or starts with that crate and :: \| enforced at pipeline.rs in_crates \| boundary test crate_with_a_longer_name_makes_no_span (mu_client refused, mu_cli::server taken) and metric_event_makes_no_span_event.
- invariants: I5: The metrics layer takes only mu::metrics \| enforced at pipeline.rs:191 \| boundary test none for the refused side (see inv-4).
- invariants: I6: The stop waits at most the bound for both providers together, each on its own thread \| enforced at pipeline.rs shutdown with one deadline and a break on the first timeout \| boundary test silent_collector_does_not_delay_a_send (elapsed within seven seconds) and silent_traces_collector_does_not_stop_the_metrics (a late traces stop does not cost the metrics).
- invariants: I7: A late or failed stop writes a WARN line \| enforced at pipeline.rs:249 and :254 \| boundary test none (see inv-2).
- invariants: I8: The HTTP client follows no redirect \| enforced at pipeline.rs:294 \| boundary test none (see inv-3).
- invariants: I9: No URL of the export holds a fragment, and a base holds no query \| enforced at telemetry.rs:257 and :269 \| boundary test query_or_fragment_in_the_base_is_refused for the base only (see inv-5).
- invariants: I10: A gRPC signal always has the telemetry runtime \| enforced at pipeline.rs Providers::start, which starts it when any signal is gRPC; NoRuntime is unreachable by construction \| boundary test the gRPC tests of telemetry.rs.
- invariants: I11: A failed export logs at WARN, so the fmt filter must pass WARN \| enforced at the fmt filter and the internal-logs feature \| boundary test collector_that_the_server_does_not_trust now asserts a WARN line with Export.Failed.Exhausted (R-63 stays fixed).
- invariants: I12: The fake collector serves until the test reads it \| enforced at support/collector.rs requests() with assert of !task.is_finished() \| test-support invariant, no product boundary.
- invariants: I13: Each Command variant has a name \| enforced at compile time by the exhaustive match of Command::name \| boundary test each_command_has_a_name lists six variants; the changed comment at command.rs:158 says the list itself is kept by hand, so a seventh variant compiles with no asserted name.
- invariants: Probe: target/debug/mu, built at 21:47 before commit 1fa4e80 at 21:48:58 (the dir.rs code matches), ran mu world list and show on three directories under probe/ of the handoff directory. A separate worlds.toml exits 1; a hard link worlds.toml to server.toml exits 0 with alpha/aria; server.toml linked to global.toml shows columns = 120 # default in place of columns = 99 # global.toml.
- test-coverage: Round 5 diff text checked: tests in telemetry.rs and telemetry_engine.rs assert each THEN condition. Prior rows R-56, R-57, R-58 are closed by lag_behind_the_head, subscribe_past_the_end and the ResourceExhausted assertion.
- test-honesty: Follow-up round: read telemetry_engine.rs in full for the changed tests (lag_behind_the_head, commands_at_the_rate, lagged_stream_is_counted) and scanned telemetry.rs assertions. Each asserts a specific value tied to its scenario; no ignored tests, no tautologies found.
- test-honesty: The test lag_of_a_subscriber in telemetry.rs asserts count >= 3 only, but the exact lag value is covered by lag_behind_the_head in telemetry_engine.rs.
- test-coverage gap: Bodies of world_config.rs tests and the unmodified world-config scenarios were matched by name only.

## Ledger

| id | first | last | severity | scope | location | disposition | basis | override | claim |
|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 1 | blocker | in-change | ["artifact",["specs/telemetry/spec.md","Scenario: A bad variable"],"**THEN** stderr names `OTEL_EXPORTER_OTLP_ENDPOINT` and `https`"] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The scenario 'A bad variable' needs an https value of OTEL_EXPORTER_OTLP_ENDPOINT refused with an error that names the variable, but the design checks the scheme only for the endpoint of server.toml and uses the variables as they are. |
| R-2 | 1 | 1 | blocker | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: The variables that turn the export off and name the service"],"If `OTEL_SERVICE_NAME` is set and not empty, its value SHALL be that variable. Otherwise, its value SHALL be `mu`."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | app-2: No decision produces service.name: design.md never names OTEL_SERVICE_NAME, the default mu, or the empty variable, although the SDK resource builder reads that variable on its own and an explicit name can override it.; inv-12: OTEL_SERVICE_NAME must count as unset when empty and give mu, but the planned tests pass only an absent variable and mu-desk. The SDK's own resource detector reads the variable, so the empty case is where the code and the SDK can disagree. |
| R-3 | 1 | 1 | blocker | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: Metrics of the journal"],"the histogram `mu.subscriber.lag` SHALL get the gap from that event to the last event of the journal"] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The lag histogram needs the current last event of the journal, but event_stream gets only a Subscription, whose last() is a snapshot from the subscribe; no decision says where event_stream reads the live last number. |
| R-4 | 1 | 1 | blocker | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: A span for each command of a world"],"It SHALL carry `mu.world`, `mu.command` (`send`, `connect` or `disconnect`), `mu.origin` (`player`, `script` or `system`) and `mu.depth`."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The requirement closes mu.command to send, connect or disconnect, but the design records the name of the Command variant, and Command also has Login, Echo and Emit; the design states no lower-case rule for the scenario value send either. |
| R-5 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The names of the spans and the metrics"],"The kind of an event uses the snake-case name of `EventKind`, the same as the API."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The design cites a snake-case name of EventKind that the API shares, but no such name exists at head: the API names the kinds EVENT_KIND_INPUT and so on, and EventKind has no name method. |
| R-6 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","7. Docs and follow-ups"],"Make sure that `mise run docs:build` passes, or the docs check that `mise.toml` names."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | app-7: Task 7.1 cites the mise task docs:build, which mise.toml does not define; the docs task there is docs:check.; fit-5: Task 7.1 names mise run docs:build, which mise.toml does not define; the docs check of the repository is docs:check. |
| R-7 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Migration Plan"],"None. With no endpoint, the server behaves as before. A `server.toml` that an earlier `mu world` refused now works."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The migration plan says none, but a server.toml that an operator writes is deployed configuration: a rollback to an earlier binary makes key_of refuse it and stops mu world and mu server, and no step says to remove the file. |
| R-8 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Where the endpoint comes from"],"`serde_path_to_error` gives the path of a bad field, as `mu-core` does for a layer."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | server_config.rs re-implements the TOML parse of mu-core (toml deserializer, serde_path_to_error, line and column, field path), which already exists as world::parse with ConfigError. |
| R-9 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The span of a command follows it across the queue"],"`Portal::ask` fills it with `Span::current()`, and `route` fills it with the span of the handler run."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | Portal::ask does not build the request: a closure does, so the new span field must be filled at each of the four sites that build Request::Command. |
| R-10 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","5. The metrics"],"Each one builds a `Portal` with a test handler through `WorldSetup::with_handler`, and runs under the in-memory exporters with a paused clock."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The depth and rate tests of telemetry_engine.rs need a portal with a handler and a fake game, which the mu-core test support already builds; the plan names no reuse, so mu-cli gets a copy. |
| R-11 | 1 | 1 | should-fix | deviates-design | ["artifact",["design.md","Only ids, names, numbers and states go in a field"],"`tracing-opentelemetry` turns each `tracing` event inside a span into a span event, so this rule holds for the `warn!` calls too."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The world id gets two attribute names in the export: mu.world on the new spans and metrics, and world on the existing warn, which becomes a span event of the command span. |
| R-12 | 1 | 1 | should-fix | in-change | ["artifact",["proposal.md","Impact"],"the shared test environment clears each `OTEL_*` variable, so that the environment of the developer never reaches a test"] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The proposal says the test environment clears each OTEL_* variable, but task 3.3 removes nine named ones; test_env removes by name, so OTEL_TRACES_SAMPLER, OTEL_EXPORTER_OTLP_COMPRESSION, OTEL_EXPORTER_OTLP_HEADERS and OTEL_RESOURCE_ATTRIBUTES of the developer still reach each test. Compression would also make the byte scans for the password pass vacuously. |
| R-13 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: The variables that turn the export off and name the service"],"`OTEL_SDK_DISABLED` with the value `true`, in any case, SHALL turn the export off, whatever the endpoint."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | OTEL_SDK_DISABLED turns the export off only for true in any case, but the one test passes TRUE. No test passes the other side, such as false or 1, where the export must stay on. |
| R-14 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: Only OTLP over HTTP with protobuf"],"whose value is not `http/protobuf` while the export is on"] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The protocol check covers three variables and applies only while the export is on, but the one test sets OTEL_EXPORTER_OTLP_PROTOCOL=grpc with an endpoint. No test passes grpc in the TRACES or METRICS variable, and none passes grpc with the export off, where the start must succeed. |
| R-15 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: A slow collector does not slow a world"],"At the stop, the server SHALL wait for the export at most five seconds, then exit."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The stop must wait for the export at most five seconds, but the silent-collector scenario asserts only exit status 0 and no elapsed time. Task 6.2 expects the test to fail without the bound, but nothing in the scenario fails on a slow exit. |
| R-16 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: A slow collector does not slow a world"],"When the queue is full, the server SHALL drop the new data."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The spec says the server drops new data when the queue is full, but the test sends 200 calls, which make about 400 spans plus events. That is below the default BatchSpanProcessor queue of 2048, so the full-queue edge is never reached. |
| R-17 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: A span for each API call"],"The span of `Subscribe` SHALL cover the call up to the return of its stream. It SHALL NOT cover the life of the stream."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The Subscribe span must end when the call returns its stream, not with the stream, but no task tests the Subscribe span. An instrumented stream would keep it open until the stream ends. |
| R-18 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Only ids, names, numbers and states go in a field"],"`tracing-opentelemetry` turns each `tracing` event inside a span into a span event"] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The rule that no span event holds line text depends on what reaches the OpenTelemetry layers, but the design filters only the SDK targets there. The engine's dependencies connectrpc, h2, hyper-util and tower emit tracing events, and inside an API span each becomes a span event. Review of each new tracing call does not cover them. |
| R-19 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The names of the spans and the metrics"],"\| `mu.subscriber.lag` \| histogram \| none \| `event_stream`, for each event that it sends \|"] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The lag invariant (lag = last seq of the journal minus the seq of the sent event, never negative) has no source: event_stream gets only a Subscription, whose last() is the snapshot at the start, and no task gives it the journal. Snapshot minus a live event's seq underflows the u64 of Seq. |
| R-20 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: Metrics of the worlds"],"Its value SHALL be 1 for the current state of the world, and 0 for each state that it left."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The gauge mu.world.state must hold 1 for the current state of each world, but the design emits it only in set_state. Each world starts as Disconnected without set_state, and a world without a transport or a password never calls set_state, so it never gets a 1. |
| R-21 | 1 | 1 | should-fix | in-change | ["artifact",["proposal.md","Not in this change"],"The server refuses an `https://` endpoint at the start with an error that names its source."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The proposal says the server refuses any https:// endpoint at the start, naming its source, but the design uses the signal variables 'as it is' and puts the http::Uri check only in server_config.rs. No task checks OTEL_EXPORTER_OTLP_TRACES_ENDPOINT or OTEL_EXPORTER_OTLP_METRICS_ENDPOINT, so an https value there starts the server, and the exporter without TLS then fails each export. |
| R-22 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Where the endpoint comes from"],"`OTEL_EXPORTER_OTLP_ENDPOINT`, with `/v1/traces` or `/v1/metrics` added."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The base URL gets /v1/traces or /v1/metrics added, but no planned test passes a base with a trailing slash or with a path. A naive join turns http://h:4318/ into http://h:4318//v1/traces, which an axum route for /v1/traces does not match. |
| R-23 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Where the endpoint comes from"],"The walk of the directory in `dir.rs` skips the file `server.toml` at the top level, before `key_of`."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The skip of server.toml must hold only at the top level, but the planned test covers only the top-level file. characters/alpha/server.toml is the world alpha/server, and a skip by file name instead of by path would drop that world with no error. |
| R-24 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: Each field of server.toml is known"],"An unknown table or field SHALL stop the start."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The spec says an unknown table of server.toml stops the start, but the three planned tests cover only an unknown field inside [telemetry]. deny_unknown_fields on the outer struct is a separate attribute that no test checks. |
| R-25 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: Each field of server.toml is known"],"So SHALL a value of the wrong type, an endpoint that is not a URL, and a scheme other than `http`."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The spec says a value of the wrong type stops the start and names the field, but no scenario or task passes one, such as endpoint = 4318 or telemetry = "x". |
| R-26 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","7. Docs and follow-ups"],"Make sure that `mise run docs:build` passes, or the docs check that `mise.toml` names, passes."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | Task 7.1 names the build target `mise run docs:build`, which mise.toml does not define. Its docs target is `docs:check`. |
| R-27 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Metrics of the worlds"],"a world with a handler that sends more commands in one second than the rate limit allows"] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The GIVEN step of the scenario 'A command over the rate' names no number of commands and no rate limit value, so the input is not concrete. |
| R-28 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","Risks / Trade-offs"],"The OpenTelemetry layers filter out the targets of the SDK, so its logs do not loop back into the export."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The mitigation that the OpenTelemetry layers filter out the SDK targets names nothing in the tasks: task 3.2 builds the three layers with no filter of those targets. |
| R-29 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","Where the endpoint comes from"],"An empty variable counts as unset. The server passes the full URL of each signal to the exporter builder with `with_endpoint`."] | fixed | e7228de7f89fa7062244a90610b2c9ea5fc899a1 |  | The join of a base URL and a signal path and the precedence of the endpoint order hold over unbounded strings: the result never holds // before v1/, and a non-empty higher step always wins over each lower one. |
| R-30 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The fake collector of the tests"],"include `crates/mu-core/tests/support/mod.rs` with a `#[path]` module. Thus they reuse `world`, `started`, `on`, `idle` and `wait_event`"] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | app-1: The design reuses crates/mu-core/tests/support/mod.rs in mu-cli through a #[path] include. At head that file calls hex::decode, and its paused-clock tests need the tokio feature test-util. mu-cli has neither, and task 1.2 adds neither, so the include does not compile in telemetry_engine.rs.; fit-2: The #[path] include of the mu-core test support into a mu-cli test binary has no precedent in the repository; the existing pattern for test doubles shared across crates is the public mu_core::fake module. The design names neither, and the included file needs hex and the tokio test-util feature, which mu-cli does not have and task 1.2 does not add (nor hex to ALLOWED_CRATES of mu-cli). |
| R-31 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The span of a command follows it across the queue"],"In `route`, the current span is the `command` span of the event that the handler got."] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | The design says the current span in route is the command span of the event the handler got. At head, route also runs from run_server_handlers in its own task, and from record calls outside Session::handle, such as a game line in on_bytes. In those cases no command span is current, so the claim does not hold. |
| R-32 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Two transports and one TLS stack"],"It cannot trust the authority of a private network, and the certificate file covers that case only together with the system roots."] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | The rejection of tls-webpki-roots says the certificate file covers a private authority only together with the system roots. The reqwest builder (add_root_certificate) and tonic ClientTlsConfig (ca_certificate) add a PEM authority on top of either root store, so the file covers that case with the embedded roots too. |
| R-33 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The names of the spans and the metrics"],"\| `mu.world.state` \| gauge \| `mu.world`, `mu.world.state.name` \| `Session::set_state` \|"] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | The gauge attribute mu.world.state.name has no named source, while the design gives each of the other three enums that the telemetry names (EventKind, Command, Origin) a name function with a variant-listing test. WorldState has no text name at head; only convert::state maps it to the schema enum. |
| R-34 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The names of the spans and the metrics"],"\| `PortalService/<method>` \| span \| `mu.world` (only in a call that names a world) \| each method of `PortalApi` \|"] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | The API span name PortalService/<method> drops the package of the schema service, mu.v1.PortalService, which is the procedure name on the wire and the service name that the OpenTelemetry RPC conventions use for span names and rpc.* attributes. The design names neither that convention nor a reason for the short name. |
| R-35 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","1. Sources and dependencies"],"Make sure that `cargo deny check` and `cargo nextest run -p mu-cli --test workspace_boundary --locked` pass"] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | Task 1.2 and the design check the dependency policy with a raw `cargo deny check`, while mise.toml, the source of the check commands, defines the task deny as `cargo deny --locked check`. The raw command drops --locked. |
| R-36 | 2 | 2 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: Each endpoint variable is checked"],"A bad value SHALL stop the start with an error that names the variable, and the exit status SHALL be 1."] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | Two universal rules meet when OTEL_SDK_DISABLED is true and an endpoint variable is bad: one says the bad value stops the start, the other says the export is off whatever the endpoint. No scenario, decision or test passes that combination, so either outcome ships unchecked. |
| R-37 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Where the endpoint comes from"],"One function, `check_endpoint`, checks a URL with `http::Uri`: it needs the scheme `http` or `https`, and a host."] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | check_endpoint has three refusals (scheme other than http/https, no scheme, no host), but the planned tests pass only ftp:// and 'not a url'. 'not a url' fails in the Uri parse on its spaces, so no test reaches the no-scheme branch (collector:4318) or the empty-host branch (http://:4318). |
| R-38 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","3. The export"],"It checks that no URL holds `//` before `v1/`, and that a set step always wins over each lower step"] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | The planned property says that no URL has // before v1/, but the join removes only one trailing slash. check_endpoint accepts the base http://h:4318//, and the join turns it into http://h:4318//v1/traces. A model of the join confirmed this. Either the proptest fails on that base, or its generator must leave the base out. |
| R-39 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Where the endpoint comes from"],"An empty value counts as unset. With the export off, the protocol is not read."] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | An empty protocol value must count as unset and fall through to the next source, but no planned test passes an empty OTEL_EXPORTER_OTLP_PROTOCOL or signal protocol variable while an endpoint is set. An implementation that refuses the empty string as an unknown protocol passes every listed test. |
| R-40 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","5. The metrics"],"`command_over_the_rate`, with a paused clock and a handler that sends 25 commands for one event. It expects 5 in `mu.commands.dropped`"] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | mu.commands.dropped with reason rate must count only the commands that the limiter refuses, but the one test sends 25 commands, past the burst of 20. No test sends exactly 20, where the count must stay 0, so a counter emitted before or beside the limiter check is not caught at the equality. |
| R-41 | 2 | 2 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: The endpoint of server.toml"],"The server SHALL send traces with `POST` to the path `v1/traces` under that URL, and metrics with `POST` to the path `v1/metrics`, as protobuf."] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | The spec sends traces to the path v1/traces under the base URL, but check_endpoint accepts a base with a query, and the join appends to the end of the string. http://h:4318?tenant=a gives http://h:4318?tenant=a/v1/traces, a POST to / with a changed query. A model of the join confirmed this. |
| R-42 | 2 | 2 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Scenario: A command past the depth limit"],"a world with a handler that sends a command for each event, past the depth limit"] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | The GIVEN of the scenario 'A command past the depth limit' names no event count and no depth value, and its WHEN names an outcome, not an input. |
| R-43 | 2 | 2 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Scenario: A stream that a lag ended"],"the stream ends with `resource_exhausted`"] | fixed | 734060751bbc58634d932b35cddabcc56aa1bac3 |  | The WHEN of the scenario 'A stream that a lag ended' names the output of the server, not an input that causes the lag. |
| R-44 | 2 | 3 | suggestion | in-change | ["artifact",["design.md","Open Questions"],"The default queue size of the `BatchSpanProcessor` holds for now."] | fixed | f1d6fa8df9736d9a101fd674c232bb36d85e113c |  | Task 6.2 sizes its load for the default BatchSpanProcessor queue, which the open question of design.md leaves open for now. |
| R-45 | 3 | 3 | should-fix | in-change | ["path","crates/mu-cli/src/telemetry.rs",186] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | The design sends the endpoint of server.toml through check_endpoint, and the spec stops the start on a bad endpoint field. The code checks the field only when OTEL_EXPORTER_OTLP_ENDPOINT is unset, so a bad field under a set variable starts the server. |
| R-46 | 3 | 3 | should-fix | in-change | ["path","crates/mu-server/src/service.rs",171] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | The design marks the span of a failed call with otel.status_code ERROR. A Subscribe from past the end fails with out_of_range, but the code wraps that failure in stream_ok, so ended() sees Ok and the Subscribe span keeps no error status. |
| R-47 | 3 | 3 | should-fix | in-change | ["artifact",["specs/telemetry/spec.md","Requirement: A span for each API call"],"The span of `Subscribe` SHALL cover the call up to the return of its stream. It SHALL NOT cover the life of the stream."] | fixed | f1d6fa8df9736d9a101fd674c232bb36d85e113c |  | The requirement ends the Subscribe span at the return of its stream, not with the stream. No decision in design.md produces this: the span table names only the span and its attributes. Only task 4.1 and a code comment hold the rule. |
| R-48 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Two transports and one TLS stack"],"A collector whose certificate the server does not trust gets no export. The SDK logs the failure at `WARN`, and the engine goes on."] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | The design says the SDK logs an export failure at WARN. At head, every opentelemetry crate builds without the internal-logs feature: the workspace sets default-features = false and no dependency turns it on. So otel_warn! compiles to nothing, and an untrusted or dead collector leaves no line. |
| R-49 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Risks / Trade-offs"],"The SDK logs through `tracing`. The `fmt` layer keeps its lines at `WARN`. The SDK logs the first drop only once."] | fixed | f1d6fa8df9736d9a101fd674c232bb36d85e113c |  | The risk of warning lines from a dead collector rests on the SDK logging through tracing, and logging the first drop once. With internal-logs off at head, the SDK emits no tracing event, so the risk and its mitigation describe behavior that the build does not have. |
| R-50 | 3 | 3 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",52] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | The metric target mu::metrics is written three times: a private METRICS in session.rs, a private METRICS in service.rs, and a literal in TARGETS of pipeline.rs. A change to one copy makes the target filter drop that crate's metrics with no error. |
| R-51 | 3 | 3 | should-fix | in-change | ["path","crates/mu-core/src/world/dir.rs",92] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | The file name server.toml is defined twice: a private SERVER_FILE in dir.rs, which the walk skips, and a public SERVER_FILE in server_config.rs, which mu-cli reads. mu-cli already depends on mu-core. |
| R-52 | 3 | 3 | should-fix | in-change | ["path","crates/mu-server/src/service.rs",113] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | service.rs writes the service name mu.v1.PortalService by hand, while connectrpc-build already generates it as PORTAL_SERVICE_SERVICE_NAME, which mu-proto re-exports with the rest of mu::v1. |
| R-53 | 3 | 3 | should-fix | in-change | ["path","crates/mu-cli/src/telemetry/pipeline.rs",266] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | span_exporter and metric_exporter are the same 27 lines with another builder type: the same tonic branch (enter, protocol, timeout, endpoint, TLS) and the same HTTP branch. Both builders implement WithExportConfig, WithTonicConfig and WithHttpConfig through blanket impls. |
| R-54 | 3 | 3 | should-fix | in-change | ["path","crates/mu-cli/src/telemetry/pipeline.rs",234] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | fit-5: Two mechanisms decide the scheme: check_endpoint reads it with http::Uri, which accepts HTTPS:// in any case, and is_https tests the raw prefix https://. An endpoint HTTPS://host passes the check as https but gets a gRPC exporter with no TLS config.; inv-1: check_endpoint accepts an https scheme in any case (http::Uri reads HTTPS:// as https), but is_https tests the raw text for lower-case https://. A gRPC endpoint HTTPS://collector:4317 thus gets no with_tls_config: opentelemetry-otlp falls back to ClientTlsConfig::new(), whose root store is empty, so neither the system roots nor the certificate file are trusted and each export fails. A probe of both checks on HTTPS:// and Https:// confirmed the split (check Ok, is_https false). No test passes an upper-case scheme. |
| R-55 | 3 | 3 | should-fix | in-change | ["path","crates/mu-cli/tests/telemetry.rs",234] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | telemetry.rs adds fails_naming, a verbatim copy of the helper in world_config.rs. The repository's shared test helpers live in tests/support/mod.rs, which telemetry.rs already uses for stderr. |
| R-56 | 3 | 3 | should-fix | in-change | ["path","crates/mu-server/src/service.rs",75] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | mu.subscriber.lag must be last - seq for each sent event, but the tests pass only the zero edge: lag_at_the_head_is_zero expects sum 0, and lag_of_a_subscriber checks only a count of at least 3. No test sends an event behind the head, so a reversed subtraction (seq.saturating_sub(last), always 0) or a constant 0 passes every test. |
| R-57 | 3 | 3 | should-fix | in-change | ["path","crates/mu-server/src/service.rs",176] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | A call that fails must have the status error, but a Subscribe past the end of the journal returns stream_ok with a Hello and then out_of_range. ended() marks only an outer Err, so that failed Subscribe gets a span with no error status. No test subscribes past the end. |
| R-58 | 3 | 3 | should-fix | in-change | ["path","crates/mu-cli/tests/telemetry_engine.rs",143] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | lagged_stream_is_counted asserts only is_err, not the resource_exhausted code the scenario's THEN names. |
| R-59 | 3 | 3 | should-fix | in-change | ["path","crates/mu-cli/src/telemetry/tests.rs",426] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | The precedence proptest computes its expected URL with the production join helper, so a fault in join passes the property. |
| R-60 | 3 | 3 | suggestion | in-change | ["path","crates/mu-cli/src/telemetry/pipeline.rs",134] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | The design says server::run installs ring as the process provider so that the tls change (#58) uses the same one, but the install sits in Providers::start, which runs only while the export is on. #58 must move it out of the telemetry pipeline. |
| R-61 | 3 | 3 | suggestion | in-change | ["path","crates/mu-server/src/service.rs",224] | fixed | 240f37fb6518ba3a7ff1b3bb8a0cdd9d53039624 |  | Send, Connect and Disconnect each repeat the same span wrap: call_span, an async block under instrument(span.clone()), then ended(&span, result). Only the method name and the inner future vary. |
| R-62 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","Two transports and one TLS stack"],"The exporter logs the failure at `WARN`, the batch processor or the reader logs it at `ERROR`, and the engine goes on."] | refuted | opentelemetry-otlp src/retry.rs:217 logs Export.Failed.Exhausted at WARN after the retries, and the untrusted-collector run shows that WARN line on stderr. Only the per-client ExportFailed line is at DEBUG. |  | app-1: The design says the exporter logs a failed export at WARN. In opentelemetry-otlp 0.33.1 both transports log the failure at DEBUG, on purpose, because the batch processor and the reader already log it at ERROR. The WARN filter of the fmt layer drops the exporter line.; inv-1: The design says the exporter logs a failed export at WARN. In opentelemetry-otlp 0.33.1 both exporters log the failure only at DEBUG, which the WARN filter drops; only the processor's ERROR line reaches stderr. |
| R-63 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","Risks / Trade-offs"],"The `fmt` layer keeps the lines at `WARN` and above. Each failed export gives one line. The first drop of a full queue gives one line"] | fixed | 1fabd2108de7440bddd490cc7c2f72e70481c7ef |  | The fmt layer must pass SDK lines at WARN and above, and the first drop of a full queue gives one WARN line. The only stderr test passes an ERROR line, so a filter raised to ERROR drops the promised drop line and every test still passes. |
| R-64 | 4 | 4 | suggestion | in-change | ["path","crates/mu-core/src/session.rs",312] | accepted | ticket: none; reason: Only the metric name and the attributes vary between the six events, so a macro adds a layer of indirection for little gain. |  | Six metric events repeat the same shape: tracing::event! with target METRICS_TARGET and Level::INFO, then the metric field and its attributes. Four are in session.rs and two in service.rs. Two use a local helper (state_gauge, dropped), and four are inline. Only the metric field and the attributes vary, and they vary on purpose. |
| R-65 | 4 | 4 | suggestion | in-change | ["path","crates/mu-cli/tests/telemetry_engine.rs",167] | fixed | 1fabd2108de7440bddd490cc7c2f72e70481c7ef |  | The new test lag_behind_the_head repeats the setup of lagged_stream_is_counted and lag_at_the_head_is_zero line for line: the in-memory subscriber, a MemoryJournal, a watch channel, a subscription from Seq(1), event_stream, and the read of the Hello. Only the capacity of the journal varies (4 or 8). |
| R-66 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/world/dir.rs",42] | fixed | 932cf8e9ed3f9d99bf0b00294c1f85cbacb4d586 |  | app-1: The walk skips each top-level entry that is the same file (device and inode) as server.toml, so another case of the name, a hard link or a symlink to it is skipped. design.md still says the check compares the whole relative path, and records no deviation.; inv-1: The walk now skips any top-level file with the inode of server.toml. A hard link worlds.toml lists worlds with exit 0, and a link server.toml to global.toml drops the global layer with no error. |
| R-67 | 5 | 5 | should-fix | in-change | ["path","docs/src/content/docs/guides/telemetry.md",68] | fixed | 32ce627e788bbd7450f95a46f0cc98ac8e274b2f |  | The guide and the Risks section say that each failed export gives one stderr line. At head a failed export gives two: the WARN of the OTLP retry (Export.Failed.Exhausted) and the ERROR of the batch processor or the reader. The design's TLS decision and collector_that_the_server_does_not_trust already expect both. |
| R-68 | 5 | 5 | should-fix | in-change | ["artifact",["design.md","The exporter has its own threads and a bounded stop"],"`server::run` calls `shutdown` of each provider on a helper thread of its own, so a slow trace export does not delay the last metrics."] | fixed | 32ce627e788bbd7450f95a46f0cc98ac8e274b2f |  | The design now stops each provider on a helper thread of its own, in parallel, so a slow trace export does not delay the last metrics, and the code does the same. The decision page of task 7.2 still says one helper thread shuts the providers down. |
| R-69 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/world/dir.rs",105] | fixed | 932cf8e9ed3f9d99bf0b00294c1f85cbacb4d586 |  | dir.rs hand-writes a file-identity compare (dev and ino, with a name fallback off unix) to skip server.toml. same_file::is_same_file does exactly this, and same-file is already in Cargo.lock through walkdir. The design names neither the library nor the identity check: it still says the skip compares the relative path. |
| R-70 | 5 | 5 | should-fix | in-change | ["path","crates/mu-cli/src/telemetry/pipeline.rs",274] | fixed | 32ce627e788bbd7450f95a46f0cc98ac8e274b2f |  | The span layer now filters with a hand matcher, in_crates, where tracing_subscriber::filter::Targets is the library filter for targets. The reason is real: Targets matches a bare prefix, so mu_cli would also take mu_client, a crate of the workspace. But only a test holds that reason. The design names neither Targets nor why it does not fit. |
| R-71 | 5 | 5 | should-fix | in-change | ["artifact",["design.md","The exporter has its own threads and a bounded stop"],"A failed shutdown, or a wait that ends first, writes a line at `WARN` to stderr."] | fixed | c22866f902161138d61acf6e770782924c1b5e6e |  | A late or failed stop of a provider must write a WARN line, but no test reads stderr after a stop past the bound; the silent-collector test asserts only the status and the elapsed time. |
| R-72 | 5 | 5 | should-fix | in-change | ["artifact",["design.md","Two transports and one TLS stack"],"The client follows no redirect."] | fixed | c22866f902161138d61acf6e770782924c1b5e6e |  | The HTTP client must follow no redirect, but no test makes the collector answer with a redirect, so a removal of Policy::none passes every test. |
| R-73 | 5 | 5 | should-fix | in-change | ["artifact",["design.md","The engine speaks tracing, and mu-cli bridges it"],"It takes only the target `mu::metrics`."] | fixed | c22866f902161138d61acf6e770782924c1b5e6e |  | The metrics layer must take only the target mu::metrics, but no test emits a metric-prefixed field under another target such as mu_core, the side the filter must refuse. |
| R-74 | 5 | 5 | should-fix | in-change | ["path","crates/mu-cli/src/telemetry.rs",249] | fixed | c22866f902161138d61acf6e770782924c1b5e6e |  | check_endpoint now states that no URL can hold a fragment, the signal variables too, but the only fragment test passes # in OTEL_EXPORTER_OTLP_ENDPOINT, not in a signal variable or the field. |
| R-75 | 5 | 5 | suggestion | in-change | ["path","crates/mu-core/src/world/parse.rs",37] | fixed | 932cf8e9ed3f9d99bf0b00294c1f85cbacb4d586 |  | The shared TOML parse of mu-core hard-codes the hidden tables of both callers: login, a table of a world layer, and telemetry, a table of server.toml, which mu-cli owns. Which tables to hide differs by caller on purpose. Each later table of server.toml that holds a secret must edit mu-core, and each layer file gets telemetry hidden too. |
| R-76 | 5 | 5 | should-fix | in-change | ["artifact",["review","not-run"],"Worlds from character files"] | refuted | The matrix row names the unchanged world-config tests as a group, not one test id, so no one-test run can match it. Those tests pass in the full workspace run of 541 tests at this head. |  | covered row 'Worlds from character files' has no verification run |
| R-77 | 5 | 5 | should-fix | in-change | ["artifact",["review","not-run"],"A character with no game"] | refuted | The matrix row names the unchanged world-config tests as a group, not one test id, so no one-test run can match it. Those tests pass in the full workspace run of 541 tests at this head. |  | covered row 'A character with no game' has no verification run |
| R-78 | 5 | 5 | should-fix | in-change | ["artifact",["review","not-run"],"A file name outside the id rule"] | refuted | The matrix row names the unchanged world-config tests as a group, not one test id, so no one-test run can match it. Those tests pass in the full workspace run of 541 tests at this head. |  | covered row 'A file name outside the id rule' has no verification run |
| R-79 | 5 | 5 | should-fix | in-change | ["artifact",["review","not-run"],"A link to nothing"] | refuted | The matrix row names the unchanged world-config tests as a group, not one test id, so no one-test run can match it. Those tests pass in the full workspace run of 541 tests at this head. |  | covered row 'A link to nothing' has no verification run |

## Reviewed files

| path | content |
|---|---|
| Cargo.lock | e4520903809380362411400c36e678a02626bf9d |
| Cargo.toml | 404041d7e218b90a45a449b133962b622730620e |
| crates/mu-cli/Cargo.toml | 46dc4d3d4bf57e9dea427aa7571b430a897ba200 |
| crates/mu-cli/src/config.rs | e3e19c68331635350b58bc047e585effe7066532 |
| crates/mu-cli/src/lib.rs | 7184de55452e03f00b1d4b6a3d4d7ef638800c0d |
| crates/mu-cli/src/server.rs | 15bc40e8a2cc2f6b482f8c1a4aa17b577e9c9de6 |
| crates/mu-cli/src/server_config.rs | 19bcdf04a0a0b8fd5bcb233dbe72727894bc1290 |
| crates/mu-cli/src/telemetry.rs | 6eb92aaa455f7773e16ba81cf45efbfbb56f0323 |
| crates/mu-cli/src/telemetry/pipeline.rs | 5c604cb106d3c5a4ef2dc4a492a98abf29988fa3 |
| crates/mu-cli/src/telemetry/tests.rs | a661258c2c9ad75bc1c7d7d82c309efdea7ce7f1 |
| crates/mu-cli/tests/support/collector.rs | 259c007c8030fbd9403da947efd93cdcba20ad3f |
| crates/mu-cli/tests/support/mod.rs | 25e00f84aca56cf2686f4c0f88f55c51a01c47fb |
| crates/mu-cli/tests/support/process.rs | f6657e1fe453b23671b07de4842f1eccfc64ef86 |
| crates/mu-cli/tests/telemetry.rs | f845de4a1a579e51973a79b13329ec724321a5b1 |
| crates/mu-cli/tests/telemetry_engine.rs | 6947254d12341662742bde4183cd3b27fcdff574 |
| crates/mu-cli/tests/workspace_boundary.rs | 33b366a4d09f3fdd5829bfe1bcd37436b2a39494 |
| crates/mu-cli/tests/world_config.rs | 2ff361a7f34d3e22629884240fe6b76cf5f3a385 |
| crates/mu-core/src/command.rs | 7f54da87e5740d44fb453487dcc54394896a00ed |
| crates/mu-core/src/event.rs | 1de0c7515daf1c69c9e3038c89de64e2875b013e |
| crates/mu-core/src/lib.rs | 8c507cc99ac198430e949d8366bd4f27405dde83 |
| crates/mu-core/src/portal.rs | 8ac1ed6006a7b2095b07e17fbd31eaa26b95be64 |
| crates/mu-core/src/session.rs | 3674a062429699f28c9d6fdb8492ebd99a701ed6 |
| crates/mu-core/src/world/dir.rs | aeefea1cd1e9b641c9c70e716b4279f428af2aa9 |
| crates/mu-core/src/world/mod.rs | d63b19cc922fc0d0b1b45c4cfa2d0e6fb64af247 |
| crates/mu-core/src/world/parse.rs | 5fe1be09434b05c6e2a0fce1f1b13b170b5bd24a |
| crates/mu-server/src/service.rs | b8f9efa9cf89e4699f799a6b8eee8adc85022db8 |
| deny.toml | 6d67b5264909c485fbe050fe1d657258be4747de |
| docs/src/content/docs/decisions/telemetry.md | 7d7f9b233354714a84eba262e8a7fb0b92cef4c6 |
| docs/src/content/docs/guides/configuration.md | 2acfbf5eb084781003533329a8fcbcccc8056604 |
| docs/src/content/docs/guides/telemetry.md | 2d86d78e3a26db3dbcc32afeda5fbf742832c1a5 |
| openspec/changes/telemetry/.openspec.yaml | 578c89ac91eda0589930783c384ddbe6201a0568 |
| openspec/changes/telemetry/design.md | 55e11fb2b02784a7aad659b5c750c241b1c267aa |
| openspec/changes/telemetry/proposal.md | 09698870137709423e65935a81326ce42d8f4ac0 |
| openspec/changes/telemetry/specs/telemetry/spec.md | 6d47d6345ea97917b6f06dedc5d4e52aa7807f85 |
| openspec/changes/telemetry/specs/world-config/spec.md | ae472d6ae06864b5462314cc111c3028330c76f2 |
| openspec/changes/telemetry/tasks.md | sha256:6f67e3f6a75d46c4f715775025e540034487fb63bc52ea1de8ac605d81bee060 |

## Plan review (2026-11-17)

Verdict approve-with-notes at head 54b5a2a6975e8d1dce256af0a1aba44e2097b395.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-30 | approach,fit | should-fix | in-change | app-1: The design reuses crates/mu-core/tests/support/mod.rs in mu-cli through a #[path] include. At head that file calls hex::decode, and its paused-clock tests need the tokio feature test-util. mu-cli has neither, and task 1.2 adds neither, so the include does not compile in telemetry_engine.rs.; fit-2: The #[path] include of the mu-core test support into a mu-cli test binary has no precedent in the repository; the existing pattern for test doubles shared across crates is the public mu_core::fake module. The design names neither, and the included file needs hex and the tokio test-util feature, which mu-cli does not have and task 1.2 does not add (nor hex to ALLOWED_CRATES of mu-cli). | design.md > The fake collector of the tests — "include `crates/mu-core/tests/support/mod.rs` with a `#[path]` module. Thus they reuse `world`, `started`, `on`, `idle` " | app-1: In task 1.2, add hex and tokio with the feature test-util to the dev-dependencies of mu-cli, and hex to the mu-cli row of ALLOWED_CRATES. Name both in the design section of the fake collector.; fit-2: Name the #[path] include as a deliberate choice over a shared module, and add hex and the tokio test-util feature to the mu-cli dev-dependencies and hex to ALLOWED_CRATES of mu-cli in task 1.2; or move world, started, on, idle and wait_event beside mu_core::fake. |  | severity? blocker 0.87 |
| R-31 | approach | should-fix | in-change | The design says the current span in route is the command span of the event the handler got. At head, route also runs from run_server_handlers in its own task, and from record calls outside Session::handle, such as a game line in on_bytes. In those cases no command span is current, so the claim does not hold. | design.md > The span of a command follows it across the queue — "In `route`, the current span is the `command` span of the event that the handler got." | Limit the sentence to events that Session::handle records. State what parents a command of a server handler or of a game line: a new trace, or the world.connection span entered around on_bytes. |  |  |
| R-32 | approach | should-fix | in-change | The rejection of tls-webpki-roots says the certificate file covers a private authority only together with the system roots. The reqwest builder (add_root_certificate) and tonic ClientTlsConfig (ca_certificate) add a PEM authority on top of either root store, so the file covers that case with the embedded roots too. | design.md > Two transports and one TLS stack — "It cannot trust the authority of a private network, and the certificate file covers that case only together with the sys" | Give the real reason for the system roots: an authority that the operator installed in the system store works without a certificate file. Drop the claim that the file needs the system roots. |  |  |
| R-33 | fit | should-fix | in-change | The gauge attribute mu.world.state.name has no named source, while the design gives each of the other three enums that the telemetry names (EventKind, Command, Origin) a name function with a variant-listing test. WorldState has no text name at head; only convert::state maps it to the schema enum. | design.md > The names of the spans and the metrics — "\| `mu.world.state` \| gauge \| `mu.world`, `mu.world.state.name` \| `Session::set_state` \|" | Add WorldState::name (connecting, connected, disconnected) in the same decision, name it as the value of mu.world.state.name, and add it with its variant-listing test to task 5.2. |  |  |
| R-34 | fit | should-fix | in-change | The API span name PortalService/<method> drops the package of the schema service, mu.v1.PortalService, which is the procedure name on the wire and the service name that the OpenTelemetry RPC conventions use for span names and rpc.* attributes. The design names neither that convention nor a reason for the short name. | design.md > The names of the spans and the metrics — "\| `PortalService/<method>` \| span \| `mu.world` (only in a call that names a world) \| each method of `PortalApi` \|" | Name the span mu.v1.PortalService/<method> and add rpc.service and rpc.method, or state in the decision why the short name is kept; task 1.1 can read the current RPC conventions. |  |  |
| R-35 | fit | should-fix | in-change | Task 1.2 and the design check the dependency policy with a raw `cargo deny check`, while mise.toml, the source of the check commands, defines the task deny as `cargo deny --locked check`. The raw command drops --locked. | tasks.md > 1. Sources and dependencies — "Make sure that `cargo deny check` and `cargo nextest run -p mu-cli --test workspace_boundary --locked` pass" | Write `mise run deny` in task 1.2 and in the risk line of the design. |  |  |
| R-36 | invariants | should-fix | in-change | Two universal rules meet when OTEL_SDK_DISABLED is true and an endpoint variable is bad: one says the bad value stops the start, the other says the export is off whatever the endpoint. No scenario, decision or test passes that combination, so either outcome ships unchecked. | specs/telemetry/spec.md > Requirement: Each endpoint variable is checked — "A bad value SHALL stop the start with an error that names the variable, and the exit status SHALL be 1." | State in 'Where the endpoint comes from' whether check_endpoint runs before step 1 of the order, and add a unit test in task 3.1 with OTEL_SDK_DISABLED=true and OTEL_EXPORTER_OTLP_ENDPOINT=ftp://collector.example:4318 that asserts that outcome. |  | severity? blocker 0.73 |
| R-37 | invariants | should-fix | in-change | check_endpoint has three refusals (scheme other than http/https, no scheme, no host), but the planned tests pass only ftp:// and 'not a url'. 'not a url' fails in the Uri parse on its spaces, so no test reaches the no-scheme branch (collector:4318) or the empty-host branch (http://:4318). | design.md > Where the endpoint comes from — "One function, `check_endpoint`, checks a URL with `http::Uri`: it needs the scheme `http` or `https`, and a host." | Add to the unit tests of task 3.1 the values collector.example:4318 (no scheme) and http://:4318 (empty host), each refused with an error that names its source. |  |  |
| R-38 | invariants | should-fix | in-change | The planned property says that no URL has // before v1/, but the join removes only one trailing slash. check_endpoint accepts the base http://h:4318//, and the join turns it into http://h:4318//v1/traces. A model of the join confirmed this. Either the proptest fails on that base, or its generator must leave the base out. | tasks.md > 3. The export — "It checks that no URL holds `//` before `v1/`, and that a set step always wins over each lower step" | Make the join remove every trailing slash (trim_end_matches('/')) and add http://h:4318// to the join unit tests. Or narrow the property in task 3.1 to bases that end in at most one slash, and say so in the design. |  |  |
| R-39 | invariants | should-fix | in-change | An empty protocol value must count as unset and fall through to the next source, but no planned test passes an empty OTEL_EXPORTER_OTLP_PROTOCOL or signal protocol variable while an endpoint is set. An implementation that refuses the empty string as an unknown protocol passes every listed test. | design.md > Where the endpoint comes from — "An empty value counts as unset. With the export off, the protocol is not read." | Add a unit test in task 3.1: endpoint set, OTEL_EXPORTER_OTLP_PROTOCOL empty and field protocol = "grpc", expect grpc; and the empty signal variable over a set general variable. |  |  |
| R-40 | invariants | should-fix | in-change | mu.commands.dropped with reason rate must count only the commands that the limiter refuses, but the one test sends 25 commands, past the burst of 20. No test sends exactly 20, where the count must stay 0, so a counter emitted before or beside the limiter check is not caught at the equality. | tasks.md > 5. The metrics — "`command_over_the_rate`, with a paused clock and a handler that sends 25 commands for one event. It expects 5 in `mu.com" | Make command_over_the_rate a table over 20 commands (expect no rate point, or 0) and 21 commands (expect 1), next to the 25 case. |  |  |
| R-41 | invariants | should-fix | in-change | The spec sends traces to the path v1/traces under the base URL, but check_endpoint accepts a base with a query, and the join appends to the end of the string. http://h:4318?tenant=a gives http://h:4318?tenant=a/v1/traces, a POST to / with a changed query. A model of the join confirmed this. | specs/telemetry/spec.md > Requirement: The endpoint of server.toml — "The server SHALL send traces with `POST` to the path `v1/traces` under that URL, and metrics with `POST` to the path `v1" | Make check_endpoint refuse a base URL with a query or a fragment, or join on the path of the parsed http::Uri. Add the base http://h:4318?tenant=a to the unit tests of task 3.1. |  |  |
| R-42 | spec-lint | should-fix | in-change | The GIVEN of the scenario 'A command past the depth limit' names no event count and no depth value, and its WHEN names an outcome, not an input. | specs/telemetry/spec.md > Scenario: A command past the depth limit — "a world with a handler that sends a command for each event, past the depth limit" | State the nesting or event count that reaches DEPTH_LIMIT in the GIVEN, and name the input event that the WHEN sends. |  |  |
| R-43 | spec-lint | should-fix | in-change | The WHEN of the scenario 'A stream that a lag ended' names the output of the server, not an input that causes the lag. | specs/telemetry/spec.md > Scenario: A stream that a lag ended — "the stream ends with `resource_exhausted`" | Name the input in the WHEN, such as the subscriber that stops reading while the journal gets more events, and keep the resource_exhausted result in the THEN. |  |  |
| R-44 | spec-lint | suggestion | in-change | Task 6.2 sizes its load for the default BatchSpanProcessor queue, which the open question of design.md leaves open for now. | design.md > Open Questions — "The default queue size of the `BatchSpanProcessor` holds for now." | Tie the 2100-call count in 6.2 to the queue size that task 1.1 records, or state in the task that the count assumes the default queue of 2048. |  | severity? should-fix 0.72 |

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A server with no endpoint | 3.5 | covered |  |
| Traces and metrics reach the collector | 3.5 | covered |  |
| A base URL with a trailing slash | 3.5 | covered |  |
| The log lines stay | 3.5 | covered |  |
| An unknown field | 2.2 | covered |  |
| An unknown table | 2.2 | covered |  |
| A value of the wrong type | 2.2 | covered |  |
| An endpoint that is not a URL | 2.2 | covered |  |
| An endpoint with another scheme | 2.2 | covered |  |
| The variable wins over the file | 3.5 | covered |  |
| The endpoint of the traces | 3.5 | covered |  |
| The endpoint of the metrics | 3.5 | covered |  |
| An empty variable | 3.5 | covered |  |
| A bad variable | 3.5 | covered |  |
| A bad variable of one signal | 3.5 | covered |  |
| The export is off | 3.5 | covered |  |
| The default service name | 3.5 | covered |  |
| An empty service name | 3.5 | covered |  |
| A service name from the variable | 3.5 | covered |  |
| gRPC from the file | 3.5 | covered |  |
| The variable wins over the protocol of the file | 3.5 | covered |  |
| The protocol of one signal | 3.5 | covered |  |
| The protocol http/json | 3.5 | covered |  |
| An unknown protocol in the file | 3.5 | covered |  |
| An unknown protocol with the export off | 3.5 | covered |  |
| HTTPS with a certificate file | 3.5 | covered |  |
| gRPC with TLS | 3.5 | covered |  |
| A collector that the server does not trust | 3.5 | covered |  |
| A missing certificate file | 3.5 | covered |  |
| A file that holds no certificate | 3.5 | covered |  |
| The span of a Send | 4.1 | covered |  |
| The span of a Subscribe | 4.1 | covered |  |
| A failed call | 4.1 | covered |  |
| A command under its call | 4.2 | covered |  |
| A command of the server | 4.2 | covered |  |
| The connection of a fake world | 4.3 | covered |  |
| The events of a world | 5.1 | covered |  |
| The lag of a subscriber | 5.1 | covered |  |
| A stream that a lag ended | 5.1 | covered |  |
| A command past the depth limit | 5.2 | covered |  |
| A command over the rate | 5.2 | covered |  |
| A world that never connects | 5.2 | covered |  |
| The state at the stop | 5.2 | covered |  |
| The password stays out | 6.1 | covered |  |
| The line text stays out | 6.1 | covered |  |
| A collector that never answers | 6.2 | covered |  |
| Worlds from character files | crates/mu-cli/tests/world_config.rs: worlds_from_character_files | covered |  |
| A character with no game | crates/mu-cli/tests/world_config.rs: character_with_no_game | covered |  |
| A file name outside the id rule | crates/mu-cli/tests/world_config.rs: file_name_outside_the_id_rule | covered |  |
| A stray file | 2.1 | covered |  |
| A link to nothing | crates/mu-cli/tests/world_config.rs: link_to_nothing_is_an_error | covered |  |
| The file of the server | 2.1 | covered |  |
| A character named server | 2.1 | covered |  |

### Not run

Plan phase: no verification runs.
