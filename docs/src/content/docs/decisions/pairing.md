---
title: Device pairing
description: A device must pair with the server before it can attach.
sidebar:
  order: 4
---

## Decision

A client on another device must pair with the server before it can attach. The server can list its paired devices and can revoke one. A client on the same machine uses a local socket and needs no pairing.

The exact design is open. The proposal is one key for each device, a one-time code to pair, and a scope for each pairing. A scope lets an agent pair with read-only access.

## Reason

The server holds game passwords and the full history of play. A network address alone must not be enough to reach it. Revoking one device must not disturb the others.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it. A later change designs the pairing protocol.
