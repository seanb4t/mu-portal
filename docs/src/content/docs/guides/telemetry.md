---
title: The server file and telemetry
description: Send the traces and metrics of mu server to an OpenTelemetry collector with server.toml or the standard OTEL variables.
sidebar:
  order: 3
---

This guide shows how to send the traces and metrics of `mu server` to a collector. A collector is a service that receives OpenTelemetry data over OTLP, the OpenTelemetry protocol. Without a collector, the server sends nothing and writes no line about telemetry.

## The server file

The file `server.toml` holds the configuration of the server. It goes at the top level of the configuration directory, next to `global.toml`. The guide [The configuration files](/guides/configuration/) tells where that directory is. `server.toml` makes no world, and `mu world` ignores it.

The file holds one table, `[telemetry]`:

```toml
# server.toml
[telemetry]
endpoint = "http://127.0.0.1:4318"
protocol = "http/protobuf"
certificate = "/etc/mu/collector-ca.pem"
```

| Field | Meaning |
|---|---|
| `endpoint` | The base URL of the collector, with the scheme `http` or `https` |
| `protocol` | `http/protobuf` or `grpc`. The default is `http/protobuf` |
| `certificate` | A PEM file of the authorities to trust beside the roots of the system |

Each field is optional. The server reads the file once, at the start, before it opens its socket. An unknown table, an unknown field, or a value of the wrong type stops the start. So does an endpoint that is not a URL, or a scheme other than `http` and `https`. So does a base URL with a query or a fragment. The error names `server.toml` and the field, and the exit status is 1.

With `http/protobuf`, the server sends the traces to `v1/traces` and the metrics to `v1/metrics` under the base URL. One trailing slash does not change the result. With `grpc`, the base URL is the address of the gRPC services, and the server adds no path.

## The variables

The server also reads these standard variables. A variable that is set wins over the field of `server.toml`. An empty variable counts as unset.

| Variable | Meaning |
|---|---|
| `OTEL_SDK_DISABLED` | `true`, in any case, turns the export off. Then the server reads no other telemetry variable and no field |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | The base URL of both signals, the same as `endpoint` |
| `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` | The full URL of the traces. The server adds no path |
| `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` | The full URL of the metrics. The server adds no path |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | The protocol of both signals, the same as `protocol` |
| `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL` | The protocol of the traces |
| `OTEL_EXPORTER_OTLP_METRICS_PROTOCOL` | The protocol of the metrics |
| `OTEL_EXPORTER_OTLP_CERTIFICATE` | The certificate file, the same as `certificate` |
| `OTEL_SERVICE_NAME` | The value of `service.name`. The default is `mu` |

For each signal, the server picks the endpoint in this order:

1. The variable of the signal, as it is.
2. `OTEL_EXPORTER_OTLP_ENDPOINT`, with the path of the signal.
3. The field `endpoint`, with the path of the signal.

If a signal has no endpoint, the server does not export it. The protocol follows the same order: the variable of the signal, then `OTEL_EXPORTER_OTLP_PROTOCOL`, then the field `protocol`.

Each endpoint variable follows the rules of the field. A bad value stops the start, and the error names the variable. The server reads the protocol and the certificate file only while the export is on.

## TLS

The scheme decides TLS. `https` uses TLS, and `http` does not. The server trusts the roots of the system and the authorities of the certificate file. A certificate file that cannot be read, or that holds no PEM certificate, stops the start.

If the server does not trust the certificate of the collector, it exports nothing and serves as usual.

## A slow collector

The export runs apart from the worlds, with a bounded queue. If the collector is slow or absent, the server drops data, and the worlds do not wait. At the stop, the server waits at most five seconds for the export, then exits.

## What the server does not support

The server does not support these features. Some of them can have an effect through the OpenTelemetry library, but `mu` does not test them:

- The protocol `http/json`. The server refuses it at the start.
- `OTEL_EXPORTER_OTLP_INSECURE`. The scheme decides TLS.
- Client certificates, and the certificate variables of one signal.
- Headers, timeouts, compression, samplers, the batch variables, `OTEL_TRACES_EXPORTER` and `OTEL_METRICS_EXPORTER`.
- The OTLP logs signal. The log lines stay on stderr, at the level `WARN`.
- A reload of `server.toml` while the server runs.

## Before a downgrade

An earlier `mu` does not know `server.toml`. It reads the file as a stray file, and `mu world` and `mu server` stop with an error. Before you install an earlier binary, remove or rename `server.toml`.
