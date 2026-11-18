---
title: Telemetry
description: The engine speaks tracing, and mu server exports its spans and metrics over OTLP on threads of its own.
sidebar:
  order: 21
---

## Decision

The engine makes its spans and metric events with `tracing` alone. `mu-core` and `mu-server` take no OpenTelemetry crate. `mu server` adds the layers of `tracing-opentelemetry`, which turn the spans and the events into OpenTelemetry data.

A filter on the two OpenTelemetry layers passes only the targets `mu_core`, `mu_server`, `mu_cli` and `mu::metrics`. Thus the events of the libraries, for example `h2`, `hyper` and `reqwest`, never become span events or metrics. The metric events use the target `mu::metrics`.

### Transports and TLS

The server exports over OTLP with the protocol `http/protobuf` or `grpc`. The HTTP exporter uses a blocking `reqwest` client that the server builds itself. The gRPC exporter uses `tonic`.

Both transports use rustls with the crypto provider `ring`. The server installs `ring` as the default provider of the process before it builds a client. Each transport trusts the roots of the system and the authorities of the certificate file. The scheme of the endpoint decides TLS.

### Threads

The batch processor of the spans and the reader of the metrics each run on a thread of their own, with a bounded queue. A full queue drops new data. The gRPC exporter needs a tokio runtime, so it gets its own current-thread runtime on the thread `mu-telemetry`. No telemetry task runs on a worker of the engine.

At the stop, each provider shuts down on a helper thread of its own, so a slow trace export does not delay the last metrics. The server waits at most five seconds for both. A failed or late stop writes a line to stderr. Then the server stops the `mu-telemetry` runtime and exits.

### Names

| Name | Kind | Attributes |
|---|---|---|
| `mu.v1.PortalService/<method>` | span | `rpc.system`, `rpc.service`, `rpc.method`, `mu.world` |
| `command` | span | `mu.world`, `mu.command`, `mu.origin`, `mu.depth` |
| `world.connection` | span | `mu.world` |
| `mu.journal.events` | sum | `mu.world`, `mu.event.kind` |
| `mu.subscriber.lag` | histogram | none |
| `mu.subscriber.lagged` | sum | none |
| `mu.commands.dropped` | sum | `mu.world`, `mu.drop.reason` |
| `mu.script.errors` | sum | `mu.world` |
| `mu.world.state` | gauge | `mu.world`, `mu.world.state.name` |

An API span has `mu.world` only for a call that names a world. The span of a command is a child of the span of its caller. A request to a world carries the span of its caller across the queue of the world. A failed API call sets the status `ERROR` on its span.

### No text in a field

A field of a span or a metric event holds only ids, names, numbers and states. It never holds the text of a line, a template or a password. `mu.command` is the name of the variant of the command, not its text.

## Reason

One seam keeps the engine free of the OpenTelemetry crates. The tests of the engine run the real layers with the in-memory exporters of the SDK.

The target filter keeps the noise of the libraries out of the export. It also stops a loop of the SDK logs back into the export.

`ring` is the provider because the license of `aws-lc-sys` is outside the list of `deny.toml`. Later TLS work uses the same provider. The roots of the system cover an authority that the owner already installed. The certificate file covers a private authority.

The export stays off the runtime of the engine, so a slow collector does not slow a world. The bounded stop keeps a dead collector from holding the exit.

No text in a field keeps the password and the content of the game out of the collector. A test reads each byte that the collector gets.

## Source

The OpenSpec change `telemetry` recorded it. The `telemetry` capability holds the requirements. The guide [The server file and telemetry](/guides/telemetry/) describes the configuration.
