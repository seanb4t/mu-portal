---
title: ConnectRPC API
description: Unary commands and one server stream that a client resumes from a sequence number, served with the connectrpc crate.
sidebar:
  order: 3
---

## Decision

The API between a client and the server uses the `connectrpc` crate with a protobuf schema. The server does not use `tonic`.

A client sends each command as one unary request with one reply. A client reads events from one server stream, `Subscribe`, and names the first sequence number it wants. Sequence numbers start at 1 and have no gaps. The API has no stream in both directions.

The server takes the backlog and the live receiver under one lock. Thus a stream from a sequence number has no gap and no duplicate. A subscriber that falls too far behind the live channel gets an error and resumes from its last sequence number.

A client treats every error in the middle of the stream as a reason to resume. It does not choose by error code, because a dropped connection arrives as `internal`, not as `unavailable`. A resume is safe to repeat, so a resume after a real server error costs one extra request and nothing more.

The server serves one axum router on a Unix socket and on a TCP listener. On TCP it accepts HTTP/1.1 and HTTP/2 without TLS (h2c) on the same port.

## Reason

The spike ran one scenario on six combinations of transport and protocol. The client sends three commands, reads them from the stream, and loses the connection. A relay in front of the server closes the connection, as a network drop does. Two events arrive while the client is away. The client resumes from the next sequence number, and then sends one more command and reads it live.

| Transport | HTTP | Protocol | Result |
|---|---|---|---|
| Unix socket | HTTP/2 (h2c) | Connect | pass |
| Unix socket | HTTP/2 (h2c) | gRPC | pass |
| TCP | HTTP/1.1 | Connect | pass |
| TCP | HTTP/1.1 | gRPC-Web | pass |
| TCP | HTTP/2 (h2c) | Connect | pass |
| TCP | HTTP/2 (h2c) | gRPC | pass |

In every combination the client read sequence numbers 1 to 6 once each, in order. The client reconnected by itself for the next call, over HTTP/1.1 and over HTTP/2. The server closed the stream of the lost client each time, so a dropped client leaves no work on the server. The server recorded the HTTP version of each request, and each combination used the version that the table names.

The cut always reached the client as an error and never as a clean end of the stream. The code was `internal` with the message "failed to read response body", in all six combinations.

`buf curl` is a client in Go. Over TCP, it called the server with the Connect, gRPC and gRPC-Web protocols. Over the Unix socket, it called the server with Connect. It read a stream that started from sequence number 2. So a client that does not use Rust can use the API.

The server speaks gRPC too, so a gRPC client works against it without a change. If the crate does not hold up, `tonic` can serve the same schema.

## Not tested

- A connection that stops without a close, for example a phone that loses its network. Only an HTTP/2 keep-alive or a deadline finds such a connection. The spike cut connections with a close only.
- A connection between two hosts. Every TCP test used the loopback address.
- TLS, and the Tailscale transport of issue 6.
- A restart of the server. The log is in memory, so a restart loses it. The journal decision covers a log that lasts.
- A subscriber that falls behind the live channel. The code ends its stream with `resource_exhausted`, but no test fills the channel.
- A browser client, and a client in TypeScript or Swift.
- Load: many subscribers, a long stream, or large messages.

## Source

The spike of issue 5, merged in pull request 42. The code is `spikes/connectrpc-api`, and the schema is `spikes/connectrpc-api/proto/mu/spike/v1/portal.proto`. The results are in `spikes/connectrpc-api/results`.

The command `cargo test -p connectrpc-api -- --nocapture --test-threads 1` in `spikes/` runs the six combinations and prints the line of each one in `results/matrix.txt`.

To repeat the `buf curl` calls of `results/buf-curl.txt`, start the server in `spikes/` and call it from a second shell in the same directory:

```bash
cargo run -p connectrpc-api -- 127.0.0.1:8095 mu.sock
buf curl --schema connectrpc-api/proto/mu/spike/v1/portal.proto --protocol connect \
  -d '{"text":"hello"}' http://127.0.0.1:8095/mu.spike.v1.PortalService/Send
```
