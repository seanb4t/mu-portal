# Proposal

## Why

The server writes only warning lines to stderr. It has no spans across the API, no metrics and no exporter. Thus a slow command, a world that drops, or a handler that floods the rate limit leaves no trace. world-sessions (#54) connects real games for the first time, and the first evening on real games needs traces to explain what went wrong. This change adds traces and metrics over OTLP. Issue #63 tracks it, and it is step 4 of the Phase 1 plan in #61.

## What Changes

- The server exports traces and metrics with OpenTelemetry over OTLP. The protocol is `http/protobuf` or `grpc`, and the endpoint is `http://` or `https://`. TLS trusts the root certificates of the system and an optional certificate file. With no endpoint, the server exports nothing and behaves as it does today.
- A new optional file, `server.toml`, in the configuration directory. It holds the configuration of the server, not of a world. Its first table is `[telemetry]`, with the fields `endpoint`, `protocol` and `certificate`. The walk of the worlds skips this file, and `mu world` ignores it.
- These standard variables also work: the three endpoint variables, the three protocol variables, `OTEL_EXPORTER_OTLP_CERTIFICATE`, `OTEL_SERVICE_NAME` and `OTEL_SDK_DISABLED`. A variable wins over the file.
- Spans: one for each API call, one for each connection of a world, and one for each command that a world runs. Each span of a world carries the world id. The span of a command is a child of the span of the API call that sent it.
- Metrics of the journal: its events, the lag of the subscribers, and the streams that a lag ended.
- Metrics of the worlds: the dropped commands of the depth limit and the rate limit, the script errors, and the state of each world.
- The engine emits spans and metrics through `tracing` alone. The `tracing-opentelemetry` layers turn them into OpenTelemetry data. `mu-core` and `mu-server` take no OpenTelemetry crate.
- The stderr log lines stay as they are, at the level `WARN`.
- No password, and no text of a line, goes into a span, an attribute, a metric or a log line. A test reads each byte that the collector receives.
- The exporter runs on its own thread with a bounded queue. A slow or absent collector drops data. It does not slow a world. At the stop, the server flushes for a bounded time, then exits.

## Not in this change

- The OTLP logs signal. The log lines stay on stderr.
- The protocol `http/json`. The server refuses it at the start.
- Client certificates (mutual TLS), and the certificate variables of one signal.
- `OTEL_EXPORTER_OTLP_INSECURE`. The scheme of the endpoint decides TLS, and an endpoint with no scheme is refused.
- The other `OTEL_*` variables: headers, timeouts, compression, samplers, `OTEL_TRACES_EXPORTER`, `OTEL_METRICS_EXPORTER` and the batch variables. The SDK can read some of them, but this change claims none of them and tests none of them.
- Trace context across the API. The client sends no `traceparent`, and the server reads none. Each API call starts a new trace.
- Telemetry from the client, `mu world` or `mu version`.
- Spans or metrics for lines from the game. A span for each line costs more than it tells.
- A reload of `server.toml` while the server runs.

## Capabilities

### New Capabilities

- `telemetry`: what the server exports, where it sends it, which spans and metrics it makes, and what never goes into them.

### Modified Capabilities

- `world-config`: the configuration directory can also hold `server.toml`. It is not a layer of a world, and it is not a stray file.

## Impact

- Code: a new module `telemetry` in `mu-cli` builds the subscriber and the providers. The function `server::run` calls it in place of the `fmt` subscriber of today. `mu-cli` reads `server.toml`. `mu-core` and `mu-server` emit spans and metric events with `tracing`. The command envelope carries the span of its caller across the queue of a world. The walk of the directory in `mu-core` skips `server.toml`, and `mu-core` makes its TOML parse public, so `mu-cli` reads `server.toml` with it.
- API: no change to `portal.proto`.
- Dependencies: `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp` and `tracing-opentelemetry`, through `[workspace.dependencies]`, for `mu-cli` only. The OTLP exporter brings `reqwest`, `tonic` and `rustls`. The dev-dependencies of `mu-cli` gain `opentelemetry-proto` with its gRPC services, `tonic` and `rcgen`, for the fake collectors and their test certificates. `mu-cli` takes `serde`, which is already in `[workspace.dependencies]`.
- Tests: the shared test environment clears each `OTEL_*` variable, so that the environment of the developer never reaches a test. A fake OTLP collector in the test support records each request.
- Docs: a user page for `server.toml` and the telemetry, and a decision page for the shape of the telemetry.
