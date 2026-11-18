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
2. `tracing_opentelemetry::OpenTelemetryLayer` over an `SdkTracerProvider`. It takes the targets of `mu_core`, `mu_server` and `mu_cli`, and their modules. It does not take `mu::metrics`, so a metric event adds no event to a span. The filter is the function `in_crates`, not `tracing_subscriber::filter::Targets`. `Targets` matches a bare prefix, so with it `mu_cli` also takes `mu_client`. The filter therefore needs the `::` boundary.
3. `tracing_opentelemetry::MetricsLayer` over an `SdkMeterProvider`. It takes only the target `mu::metrics`.

If the export is off, the subscriber has only the first layer. `server::run` installs the subscriber with `try_init`, which also sends the records of the `log` crate to it.

A filter on the two OpenTelemetry layers passes only the targets of the workspace: `mu_core`, `mu_server`, `mu_cli` and `mu::metrics`. Thus the events of `connectrpc`, `h2`, `hyper`, `tower`, `reqwest` and the SDK never become span events or metrics. The SDK logs do not loop back into the export. A test sends an event from another target inside a span, and checks that the in-memory exporter gets no span event for it.

`Providers::with_exporters` takes the span exporter and the metric exporter. For `mu server`, `Providers::start` builds the OTLP exporters from the export, and a test passes the in-memory exporters of `opentelemetry_sdk`. `Providers::layers` builds the layers from the providers. It holds only the layers that are present, because an absent layer in the stack still enables its targets. Thus the tests of the engine metrics go through the real layers.

Alternatives: an `opentelemetry` dependency in `mu-core`, with direct calls to a `Meter`. That puts a second seam in the engine and ties `mu-core` to the version of the SDK. A trait of our own for metrics duplicates what `tracing` already gives.

### The names of the spans and the metrics

| Name | Kind | Attributes | Where |
|---|---|---|---|
| `mu.v1.PortalService/<method>` | span | `rpc.system`, `rpc.service`, `rpc.method`, `mu.world` (only in a call that names a world), and `rpc.connect_rpc.error_code` (only in a failed call) | each method of `PortalApi` |
| `command` | span | `mu.world`, `mu.command`, `mu.origin`, `mu.depth` | `Session::handle` |
| `world.connection` | span | `mu.world` | from the start of a connect to the drop of its last clone |
| `mu.journal.events` | sum | `mu.world`, `mu.event.kind` | `Session::record` |
| `mu.subscriber.lag` | histogram | none | `event_stream`, for each event that it sends, as described below |
| `mu.subscriber.lagged` | sum | none | `event_stream`, on a `Lagged` |
| `mu.commands.dropped` | sum | `mu.world`, `mu.drop.reason` | `Session::handle` |
| `mu.script.errors` | sum | `mu.world` | `Session::script_error` |
| `mu.world.state` | gauge | `mu.world`, `mu.world.state.name` | `Session::set_state` |

A new function, `EventKind::name`, in `crates/mu-core/src/event.rs`, gives the snake-case name of each kind: `connected`, `disconnected`, `line`, `prompt`, `message`, `input`, `echo`, `login`, `custom` and `script_error`. The value of `mu.event.kind` is that name. A new function, `Command::name`, gives the lower-case name of each variant: `send`, `connect`, `disconnect`, `login`, `echo` and `emit`. The value of `mu.command` is that name. `Origin::name` gives `player`, `script` and `system` for `mu.origin`. `WorldState::name` gives `connecting`, `connected` and `disconnected` for `mu.world.state.name`. The match of each `name` has no wildcard, so a new variant fails to compile until it has a name. `EventKind`, `Origin` and `WorldState` each have an `ALL` constant, and the unit test of each name reads it. A failed call sets `otel.status_code = "ERROR"` on its span. `tracing-opentelemetry` reads that field as the status. The call also sets `rpc.connect_rpc.error_code` to the name of its Connect error code, for example `not_found`. The name comes from a fixed set, and the message of the error stays out. The span of a world connection lives in the `Conn` state of the session. It opens at the start of a connect. The drop of its last clone closes it: `down` drops the clone in `Conn`, and a step of the session that runs in the span holds a clone until the step ends.

The span of `Subscribe` ends at the return of its stream, not at the end of the stream. A stream can live for the whole evening, and a span that stays open that long reaches the collector only at its end. The method returns a ready future, so the span covers the check of `from_seq` and the build of the stream. A `Subscribe` from past the end of the journal returns a stream that sends a `Hello` and then `out_of_range`. Its span gets the status `ERROR` before the method returns.

For the gauge, `session::run` emits the value 1 for the state of the watch channel at the start of the session, which is `disconnected`. Then `set_state` emits the value 0 for the old state and 1 for the new state. Thus the last export holds one 1 for each world, a world with no transport too.

### The lag of a subscriber reads the live journal

`event_stream` gains an argument, the journal as `Arc<dyn Journal>`. `PortalApi::subscribe` passes `Portal::journal`. After `event_stream` gets each event, it reads `Journal::last` and records `last - seq` in `mu.subscriber.lag`. `append` pushes the event before it sends it, under one lock, so `last` is never below the number of an event that a subscriber got. The subtraction still saturates at 0, so a later journal cannot make it panic. A test of an event at the head of the journal expects the value 0. The read takes the lock of the journal, so `event_stream` reads `last` only while a layer takes the metrics target.

Alternative: a live `last()` on `Subscription`. That gives the subscription a handle of the journal for one metric.

### The span of a command follows it across the queue

`Request::Command` gains the field `span: tracing::Span`. A new constructor, `Request::command(envelope, reply)`, fills it with `Span::current()`. `Portal::submit`, `Portal::stop`, `Portal::connect_all` and `route` each use it. In `route`, the current span is the span of the code that recorded the event. Inside `Session::handle`, it is the `command` span. For a line of the game, it is the `world.connection` span. In `run_server_handlers`, which runs in its own task, there is no current span, so the `command` span starts a new trace. `Session::handle` makes the `command` span with that span as its parent. `Envelope` stays as it is, because it is public and it is data, not context.

Alternative: put the span in `Envelope`. That makes a public type carry a runtime handle, and each test that builds an `Envelope` must build a span.

### Only ids, names, numbers and states go in a field

No span and no metric event takes a text of a line, a template or a password as a field. The `command` span records `mu.command` as the name of the variant of `Command`, not its text. Each `tracing` call names the world `mu.world`. The `warn!` in `route` changes its field from `world` to `mu.world`. `tracing-opentelemetry` turns each `tracing` event inside a span into a span event, so this rule holds for the `warn!` calls too.

The rule has two checks:

- The test of the scenario "The password stays out" scans each byte that the fake collector got, and stderr, for the password. The test of "The line text stays out" does the same for a marker text.
- Review checks each new `tracing` call.

Alternative: a filter layer that masks each field. A mask needs the password in the subscriber, and it cannot know a line from an id.

### Where the endpoint comes from

`crates/mu-cli/src/server_config.rs` reads `server.toml` from the configuration directory of `ConfigArgs` into a struct with `deny_unknown_fields`, on the outer struct and on `[telemetry]`. A missing `server.toml` gives the default, an empty table. Any other read error stops the start: `ServerFileError::Read` holds the joined path `<directory>/server.toml` and the I/O error, and its message names both. So a `MU_CONFIG` that names a regular file stops the start with that path, because the read of `<file>/server.toml` fails with an error other than not found. It reads the text with `world::parse` of `mu-core`, which this change makes public as `mu_core::world::parse_toml`. Thus the error names `server.toml`, the field path, the line and the column, as for a layer. Its argument `Hidden` says which values an error keeps out of its message: `Hidden::Tables` for the `login` table of a layer, and `Hidden::All` for `server.toml`, so a table added later is hidden too. One function, `check_endpoint`, checks a URL with `http::Uri`. It needs the scheme `http` or `https`, and a host. A base URL, the field or `OTEL_EXPORTER_OTLP_ENDPOINT`, can hold no query, because the join adds the path at its end. A signal variable can hold a query, because the server uses it as it is. No endpoint can hold a fragment, because an HTTP client never sends it, so a fragment is always a mistake. No endpoint can hold a user or a password either. No exporter sends them as authentication, and tonic sends them in the `:authority` of each export, so a token goes in a header. The file and each of the three endpoint variables go through it, and its error names the file and field, or the variable. The function returns the parsed URL, so `https` is found once, in any case. The error holds no more of the value than its scheme, because a URL can hold a password. An error of an exporter build names the setting, not its text, because the text can quote the endpoint. The `Debug` of a signal hides the endpoint.

The server then picks the endpoint of each signal in this order:

1. If `OTEL_SDK_DISABLED` is `true` in any case, the export is off.
2. The signal variable, `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` or `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT`, used as it is.
3. `OTEL_EXPORTER_OTLP_ENDPOINT`, with `/v1/traces` or `/v1/metrics` added.
4. The `endpoint` of `server.toml`, with the same path added.
5. None. If neither signal has an endpoint, the export is off.

An empty variable counts as unset. A variable that the export reads must be UTF-8. A value that is not UTF-8 stops the start, and the error names the variable. `OTEL_EXPORTER_OTLP_CERTIFICATE` is a path, so it can hold any bytes. `OTEL_SDK_DISABLED` turns the export off only for `true` in any case. Then the function returns at once, and it reads and checks no other variable and no value of `[telemetry]`. The parse of `server.toml` comes before it and does not read the variables. So an unknown table or field, or a value of the wrong type, stops the start with the export on or off. Any other value, `false`, `1` or empty among them, leaves the order as it is. The OpenTelemetry specification asks for a warning on a value of a boolean variable that is not `true`, `false` or empty, so `server::run` writes one for such a value of `OTEL_SDK_DISABLED`. One function, `telemetry::sdk_disabled`, reads the variable for both `export` and that warning. A value that is not UTF-8 is an error there, so it stops the start and gets no warning. The warning does not quote the value.

To add the path to a base URL, the function removes each trailing slash, then adds `/v1/traces` or `/v1/metrics`. Thus `http://h:4318/` gives `http://h:4318/v1/traces`, and `http://h:4318/otel` gives `http://h:4318/otel/v1/traces`.

The server passes the full URL of each signal to the exporter builder with `with_endpoint`. Thus the order is ours, and it does not depend on how the SDK reads the variables. The protocol of each signal follows the same shape: the signal variable, then `OTEL_EXPORTER_OTLP_PROTOCOL`, then the field `protocol`, then `http/protobuf`. The protocol is read only while at least one signal has an endpoint. Any value but `http/protobuf` and `grpc` then stops the start. The error names the variable or the field, but not the value, because a URL in the wrong place can hold a password. An empty value counts as unset. With the export off, the protocol is not read.

With `grpc`, the endpoint is the address of the gRPC services, and the server adds no path. tonic sends only the scheme and the host of the endpoint. So a gRPC endpoint with a path other than `/`, or with a query, stops the start, and the error names its source. The rule applies only to the endpoint that a signal uses. A field or a base variable that a higher source wins over gets only the check of `check_endpoint`. With `http/protobuf`, a signal variable is used as it is.

The certificate file is `OTEL_EXPORTER_OTLP_CERTIFICATE`, then the field `certificate`. The server reads it at the start, only while the export is on. A file that cannot be read stops the start. So does a file that holds no PEM certificate, and a certificate that rustls cannot parse: `read_authorities` adds each one to a `RootCertStore`. Without that check, tonic drops a certificate that does not parse and the server starts, and reqwest fails with only `builder error`. The error names the variable or the field, and the path. A relative path in the field starts at the configuration directory: `server_config::read` joins it to that directory. One signal can have an endpoint while the other has none. Then only that signal is exported.

The same function reads `OTEL_SERVICE_NAME`. If it is set and not empty, its value is the service name. Otherwise the name is `mu`. The resource sets `service.name` with that value through `Resource::builder().with_service_name`, so the detectors of the SDK do not decide it.

The walk of the directory in `dir.rs` skips the file `server.toml` at the top level only, before `key_of`. The walk skips a top-level entry that meets two rules. Its name is `server.toml` in any case, and `same_file::is_same_file` finds that it is the file `server.toml`. On a file system that ignores case, `server_config` reads `Server.toml` as that file, so the walk skips it too. On another file system, `Server.toml` is a stray file and an error. The walk skips the exact name `server.toml` with no compare, so an unreadable `server.toml` is left to `mu server`, which reports the read error. For another case of the name, a compare that cannot open the files is an error that names the entry and the cause. A missing `server.toml` is the exception: then the entry is a stray file. A link with another name to `server.toml` is still an error, and a `server.toml` that links to `global.toml` does not hide the global layer. `characters/alpha/server.toml` is still the world `alpha/server`. `mu world` therefore ignores the server file.

Alternative: a `[telemetry]` table in `global.toml`. The operator chose `server.toml`, because each field of a layer is a field of a world.

### Two transports and one TLS stack

The HTTP exporter uses `reqwest-blocking-client`. `server::run` builds the `reqwest::blocking::Client` itself, with the feature `rustls-no-provider` of `reqwest`. Its verifier is `rustls-platform-verifier`, which trusts the roots of the system. `tls_certs_merge` adds the certificates of the file. The client follows no redirect. It uses the proxy variables `HTTP_PROXY`, `HTTPS_PROXY` and `ALL_PROXY`, as `reqwest` does by default, and the gRPC exporter does not. It passes the client with `with_http_client`. The SDK reads the header variables, for example `OTEL_EXPORTER_OTLP_HEADERS`, itself, and it sends the headers with each export. Over `http`, they travel in clear text. The guide says so.

The gRPC exporter uses `grpc-tonic` and the feature `tls-ring` of `opentelemetry-otlp`. Its `ClientTlsConfig` calls `with_native_roots`, a feature of `tonic`, and `ca_certificate` for each certificate of the file.

Both use rustls with the crypto provider `ring`. `server::run` installs `ring` as the default provider of the process before it builds a client. `aws-lc-rs` is not used, because the license of `aws-lc-sys` is outside the list of `deny.toml`. The decision page names the provider, so that the tls change (#58) uses the same one.

The scheme decides TLS. `https` uses TLS, and `http` does not. A collector whose certificate the server does not trust gets no export. The exporter logs the failure at `WARN`, the batch processor or the reader logs it at `ERROR`, and the engine goes on. The build turns on the feature `internal-logs` of the OpenTelemetry crates, so these lines go through `tracing` to stderr. The SDK also writes a line when it ignores the value of a variable that it reads at the build of an exporter. That build comes before `server::run` installs the subscriber, so `Providers::start` builds under a temporary subscriber with the same stderr layer. Such a line quotes the value. The SDK writes it only for the compression, the temporality and the protocol, and `export` refuses each protocol value that it does not take before the build.

Alternative: the root store that the crates embed, `tls-webpki-roots`, with the certificate file. It covers a private authority too. But it ignores an authority that the owner already installed in the system, and its list changes only with a new build of `mu`.

### The exporter has its own threads and a bounded stop

The default `BatchSpanProcessor` and the `PeriodicReader` each run on a dedicated thread. Each export request has a limit of ten seconds, `EXPORT_TIMEOUT` in `pipeline.rs`. The limit replaces the timeout variables of OpenTelemetry on both protocols, and the guide lists those variables as not supported. Without `with_timeout`, the HTTP exporter takes those variables as the deadline of its retries. A full queue drops the span, and `on_end` never waits. So a slow collector does not touch a task of tokio.

The blocking HTTP client is built before the runtime of the engine. The build of the gRPC exporter needs a tokio runtime, and its channel runs its tasks on that runtime. The gRPC export therefore gets a runtime of its own. When the first signal uses `grpc`, `telemetry/pipeline.rs` starts one thread, `mu-telemetry`, with a current-thread tokio runtime. The thread runs `block_on` of a future that ends at the stop, so the runtime drives the tasks of the gRPC channel. `Providers::start` builds the gRPC exporters inside the `enter` of that runtime. An export over `http/protobuf` alone starts no such thread, and it runs on the threads of the SDK. Thus no telemetry task runs on a worker of the engine, for both protocols.

The engine keeps the multi-thread runtime of `Runtime::new` in `server::run`. The sessions are I/O tasks, and that runtime is the usual one for them. The rule "a slow collector does not slow a world" depends only on the telemetry staying off it.

Alternative: the gRPC exporter on the runtime of the engine. Its calls are I/O, so a silent collector holds no worker. But the export then shares the queue of the scheduler with each world, and the stop must flush before the runtime of the engine drops.

The stop comes after `serve` returns. Then `server::run` calls `shutdown` of each provider on a helper thread of its own, so a slow trace export does not delay the last metrics. It waits at most five seconds for both. A failed shutdown, a helper thread that cannot start or panics, or a wait that ends first, writes a line at `WARN` to stderr. The line of a late stop names its signals. A guard in each helper thread sends `Panicked` when its stop unwinds, so a panic is named even while the other stop is late. The thread makes its guard when it starts, so a thread that cannot start sends no `Panicked`. Each helper thread gets a clone of its provider. A provider whose thread cannot start is never dropped, because the drop of the last reference runs the shutdown on the calling thread, with the fixed wait of the SDK and not within the bound. A drop of `Providers` runs the same stop, so an early return of `server::run` does not skip it. The batch span processor of the SDK drops the result of its last export, so only the metrics reader can report a failed shutdown. After the shutdown, or after the wait, the runtime of `mu-telemetry` stops. The runtime of the engine drops independently of it. Then the process exits, whether the flush finished or not. The shutdown flushes each span and collects the metrics one last time. The tests read the collector after the exit, so no test waits for an export interval.

Alternative: the async exporters with `rt-tokio`. They share the runtime of the engine, and the SDK documents the threaded processor as the default.

### The fake collector of the tests

`crates/mu-cli/tests/support/collector.rs` serves OTLP on `127.0.0.1:0` in four modes: HTTP and HTTPS with `axum` and `axum-server`, and gRPC with and without TLS with the `tonic` services of `opentelemetry-proto`. It records the method or the service, the path, the content type and the body of each request. It decodes the bodies into `ExportTraceServiceRequest` and `ExportMetricsServiceRequest`. `rcgen` makes a test authority and a certificate for `127.0.0.1` in each test. A fifth mode accepts each connection, and it never answers, for the scenario of the slow collector. The refusal tests of tasks 2.2 and 3.5 run a server to its exit with `ServerBuilder::output`, so that helper now waits at most `BOUND` for the exit. A server that starts when it must refuse then fails the test, and does not hang it. Every other caller of `output` gets the same bound.

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

## Checked facts

Task 1.1 read each fact in the source of the released crates: `opentelemetry-otlp` 0.33.1, `opentelemetry_sdk` 0.33.1, `tracing-opentelemetry` 0.34.0, `reqwest` 0.13.5 and `tonic` 0.14.9.

| Fact | Source | Edit of the design |
|---|---|---|
| `MetricsLayer` reads the prefixes `monotonic_counter.`, `counter.`, `histogram.` and `gauge.`. The other fields of the event become attributes. | `tracing-opentelemetry` `src/metrics.rs` | none |
| `OpenTelemetryLayer` reads `otel.status_code` as the status, and turns an event inside a span into a span event. | `tracing-opentelemetry` `src/layer.rs` | none |
| `BatchSpanProcessor` runs on the thread `OpenTelemetry.Traces.BatchProcessor`. Its queue holds 2048 spans. `on_end` calls `try_send`, and drops the span on a full queue. | `opentelemetry_sdk` `src/trace/span_processor.rs` | none |
| `PeriodicReader` runs on the thread `OpenTelemetry.Metrics.PeriodicReader`, every 60 seconds by default. | `opentelemetry_sdk` `src/metrics/periodic_reader.rs` | none |
| The HTTP exporter builds its own blocking client on a new thread, unless `with_http_client` gives one. | `opentelemetry-otlp` `src/exporter/http/mod.rs` | none |
| `with_endpoint` wins over each endpoint variable, and the exporter adds no path to it. | `resolve_http_endpoint` and `resolve_endpoint` of the two exporters | none |
| `with_protocol` wins over the protocol variables. The HTTP exporter refuses `grpc`. | `resolve_protocol` in `src/exporter/mod.rs` | none |
| The gRPC exporter calls `connect_lazy`, which spawns its task on the current tokio runtime. An endpoint with a scheme ignores `OTEL_EXPORTER_OTLP_INSECURE`. | `src/exporter/tonic/mod.rs` | none |
| An `https` gRPC endpoint needs `tls-ring`, `tls-aws-lc` or `tls-provider-agnostic` of `opentelemetry-otlp`. It re-exports `ClientTlsConfig` and `Certificate` in `tonic_types::transport`. It has no feature for the roots of the system. | `opentelemetry-otlp` `Cargo.toml` and `src/lib.rs` | the TLS section: `tonic` with `tls-native-roots` is a dependency of `mu-cli` |
| `reqwest` 0.13 has `rustls`, which brings `aws-lc-rs`, and `rustls-no-provider`, which takes the default provider of the process. Both use the verifier `rustls-platform-verifier`. `tls_certs_merge` adds roots to it. `opentelemetry-otlp` does not re-export `reqwest`. | `reqwest` `Cargo.toml` and `src/async_impl/client.rs` | the TLS section: `reqwest` with `blocking` and `rustls-no-provider` is a dependency of `mu-cli` |
| `Resource::builder` runs the detectors first, `OTEL_SERVICE_NAME` among them. `with_service_name` merges after them, so it wins. | `opentelemetry_sdk` `src/resource/mod.rs` and `src/resource/env.rs` | none |
| `shutdown` of the tracer provider exports the spans in the queue. `shutdown` of the meter provider collects and exports one last time. | `opentelemetry_sdk` `src/trace/span_processor.rs` and `src/metrics/periodic_reader.rs` | none |
| `InMemorySpanExporter` and `InMemoryMetricExporter` need the feature `testing` of `opentelemetry_sdk`. | `opentelemetry_sdk` `src/trace/mod.rs` and `src/metrics/mod.rs` | none |
| With the feature `internal-logs`, `otel_warn!` and `otel_error!` emit a `tracing` event with the crate name as target. Without it, they compile to nothing. | `opentelemetry` `src/global/internal_logging.rs` | the TLS section and the risks: the three OpenTelemetry crates turn on `internal-logs` |

## Risks / Trade-offs

- [A blocking reqwest client that is built inside a tokio runtime can panic.] → `server::run` builds the exporters before it builds the runtime. Task 1.1 reads the current documentation of this point.
- [`deny.toml` can refuse the TLS stack of `reqwest`, `tonic` and `rustls`.] → The change turns off the default features where it can. Task 1.2 runs `mise run deny`.
- [A later change can build a gRPC client on the runtime of the engine by mistake.] → The decision page names the `mu-telemetry` runtime. The silent-collector test runs with each protocol.
- [A dead collector can write warning lines on stderr.] → The SDK logs through `tracing`, because the build turns on its feature `internal-logs`. The `fmt` layer keeps the lines at `WARN` and above. Each failed export gives two lines: a `WARN` from the exporter and an `ERROR` from the batch processor or the reader. The first drop of a full queue gives one line, and the stop gives the count of the drops. A warning about a dead collector is useful to the owner. The OpenTelemetry layers filter out the targets of the SDK, so its logs do not loop back into the export.
- [A later change can add a field with a text.] → The scanning tests run on each change. The decision page states the rule.
- [The full-queue test depends on the default queue size of the SDK.] → Task 1.1 records the size. The test sends 2100 calls, which make two spans each, about twice the default size of 2048.
- [Up to five seconds more at the stop with a dead collector.] → It is the bound of the spec. `mu server` stops at the end of an evening, not in a loop.

## Migration Plan

With no endpoint, the server behaves as before. A `server.toml` that an earlier `mu world` refused now works.

Rollback: an earlier binary refuses `server.toml` as a stray file, and stops `mu world` and `mu server`. Before a downgrade, remove or rename `server.toml`. The user page of task 7.1 says so.

## Open Questions

None. The default queue size of the `BatchSpanProcessor` holds, as the risks and task 6.2 record. If a busy evening drops spans, a later change can set it.
