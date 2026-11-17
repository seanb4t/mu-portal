# Design

## Context

See proposal.md, section Why. These are the facts of the code before this change, which shape the design:

- `server::run` in `crates/mu-cli/src/server.rs` installs a `tracing_subscriber::fmt` subscriber on stderr at the level `WARN`. Only `mu server` installs a subscriber, because a log line breaks the screen of the client.
- The engine has one `tracing` call: a `warn!` in `route` of `crates/mu-core/src/session.rs` for a handler that names an unknown world. `mu-core`, `mu-server` and `mu-cli` depend on `tracing`. Only `mu-cli` depends on `tracing-subscriber`.
- `PortalApi` in `crates/mu-server/src/service.rs` implements the five calls of `PortalService`. `event_stream` maps a `Lagged` to `resource_exhausted`.
- `Portal::submit` in `crates/mu-core/src/portal.rs` sends a `Request::Command` over an unbounded channel to the task of the world, and waits for the reply. Thus a command crosses a task boundary, and the span of the caller does not follow it on its own.
- `Session::handle` in `session.rs` drops a command past `DEPTH_LIMIT` with a script error, and a command of a script over the rate with `rate_drop`. `set_state` is the one place that changes the state of a world. `record` is the one place that appends an event.
- `Session::record` masks each text with the password before the journal gets it. The login line goes to the transport with the real password, and its event holds the template.
- `key_of` in `crates/mu-core/src/world/dir.rs` refuses any file that is not a layer. Thus a `server.toml` today stops `mu world` and `mu server` with an error.
- `test_env` in `crates/mu-cli/tests/support/mod.rs` removes `MU_SOCKET` and points `MU_CONFIG` at an empty directory. `ServerBuilder` in `support/process.rs` starts `mu server` with fake worlds and gives a `PortalClient`.
- `ALLOWED_CRATES` in `crates/mu-cli/tests/workspace_boundary.rs` lists the third-party crates of each member.
- `world::parse` in `crates/mu-core/src/world/parse.rs` reads a TOML text into a typed struct through `serde_path_to_error`. Its `ConfigError` holds the place, the field path, the line and the column. It is `pub(crate)`.
- Four places build a `Request::Command`: `Portal::submit`, `Portal::stop`, `Portal::connect_all` and `route`. Each builds it in a closure or inline.
- `Command` has six variants: `Send`, `Connect`, `Disconnect`, `Login`, `Echo` and `Emit`. `EventKind` has no text name. The API names each kind with its own enum, for example `EVENT_KIND_INPUT`.
- `event_stream` gets a `Subscription`, whose `last()` is the last number at the time of the subscribe. `MemoryJournal::append` pushes the event and sends it to the live channel under one lock.
- A session starts in `Disconnected` without a call to `set_state`. A world with no transport never calls it.
- `crates/mu-core/tests/support/mod.rs` builds portals with fake games and test handlers: `world`, `started`, `on`, `idle` and `wait_event`.
- `RATE_PER_SECOND` in `script.rs` is 20, and the limiter allows a burst of 20.

## Goals / Non-Goals

**Goals:**

- One seam for the engine: `tracing`. The engine code makes spans and metric events, and only `mu-cli` knows about OpenTelemetry.
- Tests that read what a real collector gets, over the real OTLP/HTTP path.
- A dead collector costs the engine nothing but the queue.

**Non-Goals:**

- Sampling. Each span goes out. The volume is one person's evening.
- Exemplars, views and custom histogram buckets. The SDK defaults hold.
- A span or a metric for each line from the game.
- A test of the queue size or the export interval of the SDK. Those belong to the SDK.

## Decisions

### The engine speaks tracing, and mu-cli bridges it

`mu-core` and `mu-server` make spans with `tracing::info_span!` and metric events with `tracing::info!`, under the target `mu::metrics`. The field prefixes of `tracing_opentelemetry::MetricsLayer` name the kind of each metric: `monotonic_counter.` for a sum and `gauge.` for a gauge. `histogram.` names a histogram. The other fields of the event become the attributes.

A new module, `crates/mu-cli/src/telemetry.rs`, builds the subscriber of `mu server` from three layers:

1. The `fmt` layer on stderr, with a filter at `WARN`, as today. A metric event is at `INFO`, so it never reaches stderr.
2. `tracing_opentelemetry::OpenTelemetryLayer` over an `SdkTracerProvider`.
3. `tracing_opentelemetry::MetricsLayer` over an `SdkMeterProvider`.

If the export is off, the subscriber has only the first layer.

A filter on the two OpenTelemetry layers passes only the targets of the workspace: `mu_core`, `mu_server`, `mu_cli` and `mu::metrics`. Thus the events of `connectrpc`, `h2`, `hyper`, `tower`, `reqwest` and the SDK never become span events or metrics. The SDK logs do not loop back into the export. A test sends an event from another target inside a span, and checks that the in-memory exporter gets no span event for it.

The function that builds the layers takes the span exporter and the metric exporter as arguments. `server::run` passes the OTLP exporters, and a test passes the in-memory exporters of `opentelemetry_sdk`. Thus the tests of the engine metrics go through the real layers.

Alternatives: an `opentelemetry` dependency in `mu-core`, with direct calls to a `Meter`. That puts a second seam in the engine and ties `mu-core` to the version of the SDK. A trait of our own for metrics duplicates what `tracing` already gives.

### The names of the spans and the metrics

| Name | Kind | Attributes | Where |
|---|---|---|---|
| `PortalService/<method>` | span | `mu.world` (only in a call that names a world) | each method of `PortalApi` |
| `command` | span | `mu.world`, `mu.command`, `mu.origin`, `mu.depth` | `Session::handle` |
| `world.connection` | span | `mu.world` | from the start of a connect to `down` |
| `mu.journal.events` | sum | `mu.world`, `mu.event.kind` | `Session::record` |
| `mu.subscriber.lag` | histogram | none | `event_stream`, for each event that it sends, as described below |
| `mu.subscriber.lagged` | sum | none | `event_stream`, on a `Lagged` |
| `mu.commands.dropped` | sum | `mu.world`, `mu.drop.reason` | `Session::handle` |
| `mu.script.errors` | sum | `mu.world` | `Session::script_error` |
| `mu.world.state` | gauge | `mu.world`, `mu.world.state.name` | `Session::set_state` |

A new function, `EventKind::name`, in `crates/mu-core/src/event.rs`, gives the snake-case name of each kind: `connected`, `disconnected`, `line`, `prompt`, `message`, `input`, `echo`, `login`, `custom` and `script_error`. The value of `mu.event.kind` is that name. A new function, `Command::name`, gives the lower-case name of each variant: `send`, `connect`, `disconnect`, `login`, `echo` and `emit`. The value of `mu.command` is that name. `Origin::name` gives `player`, `script` and `system` for `mu.origin`. A unit test of each function lists every variant, so a new variant fails to compile until it has a name. A failed call sets `otel.status_code = "ERROR"` on its span. `tracing-opentelemetry` reads that field as the status. The span of a world connection lives in the `Conn` state of the session. It opens at the start of a connect, and it closes in `down`.

For the gauge, `session::run` emits the value 1 for `disconnected` at the start of the session. Then `set_state` emits the value 0 for the old state and 1 for the new state. Thus the last export holds one 1 for each world, a world with no transport too.

### The lag of a subscriber reads the live journal

`event_stream` gains an argument, the journal as `Arc<dyn Journal>`. `PortalApi::subscribe` passes `Portal::journal`. After `event_stream` gets each event, it reads `Journal::last` and records `last - seq` in `mu.subscriber.lag`. `append` pushes the event before it sends it, under one lock, so `last` is never below the number of an event that a subscriber got. The subtraction still saturates at 0, so a later journal cannot make it panic. A test of an event at the head of the journal expects the value 0.

Alternative: a live `last()` on `Subscription`. That gives the subscription a handle of the journal for one metric.

### The span of a command follows it across the queue

`Request::Command` gains the field `span: tracing::Span`. A new constructor, `Request::command(envelope, reply)`, fills it with `Span::current()`. `Portal::submit`, `Portal::stop`, `Portal::connect_all` and `route` each use it. In `route`, the current span is the `command` span of the event that the handler got. `Session::handle` makes the `command` span with that span as its parent. `Envelope` stays as it is, because it is public and it is data, not context.

Alternative: put the span in `Envelope`. That makes a public type carry a runtime handle, and each test that builds an `Envelope` must build a span.

### Only ids, names, numbers and states go in a field

No span and no metric event takes a text of a line, a template or a password as a field. The `command` span records `mu.command` as the name of the variant of `Command`, not its text. Each `tracing` call names the world `mu.world`. The `warn!` in `route` changes its field from `world` to `mu.world`. `tracing-opentelemetry` turns each `tracing` event inside a span into a span event, so this rule holds for the `warn!` calls too.

The rule has two checks:

- The test of the scenario "The password stays out" scans each byte that the fake collector got, and stderr, for the password. The test of "The line text stays out" does the same for a marker text.
- Review checks each new `tracing` call.

Alternative: a filter layer that masks each field. A mask needs the password in the subscriber, and it cannot know a line from an id.

### Where the endpoint comes from

`crates/mu-cli/src/server_config.rs` reads `server.toml` from the configuration directory of `ConfigArgs` into a struct with `deny_unknown_fields`, on the outer struct and on `[telemetry]`. It reads the text with `world::parse` of `mu-core`, which this change makes public as `mu_core::world::parse_toml`. Thus the error names `server.toml`, the field path, the line and the column, as for a layer. One function, `check_endpoint`, checks a URL with `http::Uri`: it needs the scheme `http` or `https`, and a host. The file and each of the three endpoint variables go through it, and its error names the file and field, or the variable.

The server then picks the endpoint of each signal in this order:

1. If `OTEL_SDK_DISABLED` is `true` in any case, the export is off.
2. The signal variable, `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` or `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT`, used as it is.
3. `OTEL_EXPORTER_OTLP_ENDPOINT`, with `/v1/traces` or `/v1/metrics` added.
4. The `endpoint` of `server.toml`, with the same path added.
5. None. If neither signal has an endpoint, the export is off.

An empty variable counts as unset. `OTEL_SDK_DISABLED` turns the export off only for `true` in any case. Any other value, `false`, `1` or empty among them, leaves the order as it is.

To add the path to a base URL, the function removes one trailing slash, then adds `/v1/traces` or `/v1/metrics`. Thus `http://h:4318/` gives `http://h:4318/v1/traces`, and `http://h:4318/otel` gives `http://h:4318/otel/v1/traces`.

The server passes the full URL of each signal to the exporter builder with `with_endpoint`. Thus the order is ours, and it does not depend on how the SDK reads the variables. The protocol of each signal follows the same shape: the signal variable, then `OTEL_EXPORTER_OTLP_PROTOCOL`, then the field `protocol`, then `http/protobuf`. The protocol is read only while at least one signal has an endpoint. Any value but `http/protobuf` and `grpc` then stops the start, and the error names the variable or the field. An empty value counts as unset. With the export off, the protocol is not read.

With `grpc`, the base URL takes no path: the endpoint is the address of the gRPC services. A signal variable is used as it is for both protocols.

The certificate file is `OTEL_EXPORTER_OTLP_CERTIFICATE`, then the field `certificate`. The server reads it at the start, only while the export is on. A file that cannot be read stops the start. So does a file that holds no PEM certificate. The error names the variable or the field, and the path. One signal can have an endpoint while the other has none. Then only that signal is exported.

The same function reads `OTEL_SERVICE_NAME`. If it is set and not empty, its value is the service name. Otherwise the name is `mu`. The resource sets `service.name` with that value through `Resource::builder().with_service_name`, so the detectors of the SDK do not decide it.

The walk of the directory in `dir.rs` skips the file `server.toml` at the top level only, before `key_of`. The check compares the whole relative path, so `characters/alpha/server.toml` is still the world `alpha/server`. `mu world` therefore ignores it.

Alternative: a `[telemetry]` table in `global.toml`. The operator chose `server.toml`, because each field of a layer is a field of a world.

### Two transports and one TLS stack

The HTTP exporter uses `reqwest-blocking-client`. `server::run` builds the `reqwest::blocking::Client` itself, with rustls, the root certificates of the system and the certificates of the file. It passes the client with `with_http_client`. The gRPC exporter uses `grpc-tonic`, with a `ClientTlsConfig` that holds the roots of the system and the same certificates. Both use rustls with one crypto provider. Task 1.1 picks the provider that both crates support, and the decision page names it, so that the tls change (#58) uses the same one.

The scheme decides TLS. `https` uses TLS, and `http` does not. A collector whose certificate the server does not trust gets no export. The SDK logs the failure at `WARN`, and the engine goes on.

Alternative: the root store that the crates embed, `tls-webpki-roots`. It cannot trust the authority of a private network, and the certificate file covers that case only together with the system roots.

### The exporter has its own threads and a bounded stop

The default `BatchSpanProcessor` and the `PeriodicReader` each run on a dedicated thread. A full queue drops the span, and `on_end` never waits. So a slow collector does not touch a task of tokio.

The blocking HTTP client is built before the runtime. The build of the gRPC exporter needs a tokio runtime. Thus `server::run` builds it inside `Runtime::enter`. Its calls run on the runtime of the engine as I/O tasks, and the batch thread waits for them, not a task of the engine.

The stop comes after `serve` returns and before the runtime drops. Then `server::run` calls `shutdown` of both providers on a helper thread, and waits at most five seconds for it. Then the process exits, whether the flush finished or not. The shutdown flushes each span and collects the metrics one last time. The tests read the collector after the exit, so no test waits for an export interval.

Alternative: the async exporters with `rt-tokio`. They share the runtime of the engine, and the SDK documents the threaded processor as the default.

### The fake collector of the tests

`crates/mu-cli/tests/support/collector.rs` serves OTLP on `127.0.0.1:0` in four modes: HTTP and HTTPS with `axum` and `axum-server`, and gRPC with and without TLS with the `tonic` services of `opentelemetry-proto`. It records the method or the service, the path, the content type and the body of each request. It decodes the bodies into `ExportTraceServiceRequest` and `ExportMetricsServiceRequest`. `rcgen` makes a test authority and a certificate for `127.0.0.1` in each test. A fifth mode accepts each connection, and it never answers, for the scenario of the slow collector.

The engine tests of the metrics, in `crates/mu-cli/tests/telemetry_engine.rs`, include `crates/mu-core/tests/support/mod.rs` with a `#[path]` module. Thus they reuse `world`, `started`, `on`, `idle` and `wait_event`, and add no copy of them.

### Which test decides each rule

| Rule | What enforces it |
|---|---|
| No endpoint means no export | `server_without_endpoint_writes_only_the_serving_line` |
| The endpoint order and the empty variable | the tests of "The standard variables", plus unit tests of the order function |
| The schemes `http` and `https`, and the protocols `http/protobuf` and `grpc` | the tests of each protocol over each scheme, and of `ftp`, `http/json` and `thrift` |
| TLS trusts only the system and the file | `collector_that_the_server_does_not_trust` |
| No password and no line text | the two scanning tests, the target filter and its test, and review of each `tracing` call |
| A slow collector does not slow a world | `silent_collector_does_not_delay_a_send`, which also measures the stop |
| The span and metric names | the tests of each scenario, through the fake collector or the in-memory exporters |
| `mu-core` and `mu-server` take no OpenTelemetry crate | `ALLOWED_CRATES` in the boundary test |

## Risks / Trade-offs

- [A blocking reqwest client that is built inside a tokio runtime can panic.] → `server::run` builds the exporters before it builds the runtime. Task 1.1 reads the current documentation of this point.
- [`deny.toml` can refuse the TLS stack of `reqwest`, `tonic` and `rustls`.] → The change turns off the default features where it can. Task 1.2 runs `cargo deny check`.
- [The gRPC exporter shares the runtime of the engine.] → Its calls are I/O tasks. The silent-collector test runs with each protocol.
- [A dead collector can write warning lines on stderr.] → The SDK logs through `tracing`. The `fmt` layer keeps its lines at `WARN`. The SDK logs the first drop only once. A warning about a dead collector is useful to the owner. The OpenTelemetry layers filter out the targets of the SDK, so its logs do not loop back into the export.
- [A later change can add a field with a text.] → The scanning tests run on each change. The decision page states the rule.
- [The full-queue test depends on the default queue size of the SDK.] → Task 1.1 records the size. The test sends 2100 calls, which make two spans each, about twice the default size of 2048.
- [Up to five seconds more at the stop with a dead collector.] → It is the bound of the spec. `mu server` stops at the end of an evening, not in a loop.

## Migration Plan

With no endpoint, the server behaves as before. A `server.toml` that an earlier `mu world` refused now works.

Rollback: an earlier binary refuses `server.toml` as a stray file, and stops `mu world` and `mu server`. Before a downgrade, remove or rename `server.toml`. The user page of task 7.1 says so.

## Open Questions

- The default queue size of the `BatchSpanProcessor` holds for now. If an evening with a busy world drops spans, a later change can set it.
