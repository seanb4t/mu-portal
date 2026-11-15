---
title: Embedded Tailscale
description: The server can join a tailnet by itself.
sidebar:
  order: 5
---

## Decision

The server can join a Tailscale network (a tailnet) by itself, with no Tailscale program on the host. The player turns this on with a flag of the `server` verb. Rex does the same with its server.

## Reason

A remote client needs a private and encrypted path to the server. Tailscale gives that path with no open port on the public internet. An embedded node also works in a container, where a host program is not available.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it. A spike compares a Rust library with the official C library before the work starts.
