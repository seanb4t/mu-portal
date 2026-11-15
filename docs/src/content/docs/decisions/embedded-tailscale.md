---
title: Embedded Tailscale
description: The server joins a tailnet by itself with the pure-Rust tailscale crate, behind a small interface.
sidebar:
  order: 5
---

## Decision

The server can join a Tailscale network (a tailnet) by itself, with no Tailscale program on the host. The player turns this on with a flag of the `server` verb, as Rex does with `--tailscale`.

The server uses the `tailscale` crate from Tailscale. This crate is written in Rust and is a preview. The server does not use `libtailscale`, which brings a Go runtime and a Go build step.

One module of the server names the crate. The rest of the server sees this interface:

- `JoinConfig`: the state directory, the hostname, and an auth key that only the first join needs.
- `Node::join`: joins the tailnet. If the control server does not accept the stored keys and no auth key is given, it fails with "needs login".
- `Node::identity`: the node key, the stable node id, the tailnet address and the name.
- `Node::listen`: a TCP listener on the tailnet address that axum serves. The ConnectRPC router of the server runs on it with no change.
- `Node::peer_name`: the tailnet name of the caller at an address. The server can use it to decide who a caller is.
- `Node::leave`: stops the node within a time limit.

The crate writes the key file with the process umask. Thus the interface sets the state directory to mode 0700 before the crate writes private keys into it. Then it sets the key file to mode 0600.

If a task of the crate panics, the server restarts the node. Tokio catches the panic, but the task is gone.

## Reason

The spike ran the four steps of the issue against a real tailnet on macOS (ARM64), and each step passed:

| Step | Result |
|---|---|
| Join with a one-time auth key and an empty state directory | pass: the node joined as `mu-spike` |
| Accept a connection from a tailnet peer | pass: the host "denver" reached the node through the Tailscale app, and `tailscale ping` got a direct path |
| Serve the ConnectRPC API through the node | pass: `buf curl` sent three commands with Connect over HTTP/1.1 and read the stream from sequence number 2 with gRPC over h2c |
| Keep the identity across a restart | pass: three restarts with no auth key kept the node key, the stable node id and the tailnet address |

The server named each caller as "denver" from its tailnet address.

The crate fits the server well. Its listener plugs into `axum::serve`, so the API code of the ConnectRPC spike ran with no change. It needs no Go toolchain and no C library, so the one-binary decision holds.

The crate has defects:

- The key file holds the private keys of the node, and the crate wrote it as mode 0644. Issue 449 of `tailscale/tailscale-rs` reports this. The interface works around it.
- Each stop reported a clean exit, but tasks of the crate panicked during the stop: 5, 1 and 6 panics in three stops. The panics are at `ts_derp/src/dial.rs:172` and `ts_runtime/src/task.rs:132`, on a cancelled `JoinError`. The process ended normally each time.
- If the network goes away, the crate panics in the same way. Issue 439 of `tailscale/tailscale-rs` reports this. There, the node came back without a restart, but only after 42 seconds.

The crate also brings licenses that `deny.toml` does not allow, and `cargo deny check` in `spikes/` rejects them:

| License | Crates | Path |
|---|---|---|
| MPL-2.0 | `dyn-eq` | `ts_transport` |
| 0BSD | `managed`, `smoltcp` | `ts_netstack_smoltcp` |
| CDLA-Permissive-2.0 | `webpki-roots` | `ts_control`, `ts_derp`, `ts_netcheck` |
| Apache-2.0 WITH LLVM-exception | `ar_archive_writer` | a build dependency through `psm` and `stacker` |

MPL-2.0 is a copyleft license at the level of each file. It permits a binary under Apache-2.0, but it is a change to the policy. The production change adds these licenses to `deny.toml`, or it does not use the crate. The crate also brings `paste`, through `stun-rs`, and RustSec advisory RUSTSEC-2024-0436 marks `paste` as not maintained.

If the crate does not hold up, `libtailscale` can go behind the same interface. The interface hides the crate types, so the rest of the server does not change.

## Not tested

- `libtailscale`. The first candidate passed, so the spike did not build the second.
- A host with no Tailscale program. The host of the spike also ran the Tailscale app. A container test is still to do.
- Linux and Windows. The spike ran on macOS only.
- A peer on another host. The only peer was the host itself, through the Tailscale app.
- Loss of the network while the node runs, and the time to come back.
- An expired node key, and a removed node. Both need a new auth key or a browser login.
- A tagged auth key, an ephemeral node, and tailnet access rules (ACLs).
- HTTPS certificates from Tailscale, and Tailscale Funnel.
- A repeatable test without a real tailnet. A control server such as Headscale in a container can give one through the `TS_CONTROL_URL` variable.
- Two processes with the same key file. By mistake the spike ran two at once for a short time, and it measured nothing.

## Source

The spike of issue 6, merged in pull request 47. The code is `spikes/embedded-tailscale`, and the interface is `spikes/embedded-tailscale/src/lib.rs`. The results are in `spikes/embedded-tailscale/results/runs.txt`.

To repeat the runs, create a one-time auth key that is not ephemeral. Then start the server in `spikes/`:

```bash
TS_AUTH_KEY=<key> cargo run -p embedded-tailscale -- <state-dir> mu-spike 8095
```

The server prints its tailnet address. From a second node of the tailnet, call it in `spikes/`:

```bash
buf curl --schema connectrpc-api/proto/mu/spike/v1/portal.proto --protocol connect \
  -d '{"text":"hello"}' http://<address>:8095/mu.spike.v1.PortalService/Send
```

Stop the server with Ctrl-C, start it again with no `TS_AUTH_KEY`, and compare the identity line.
