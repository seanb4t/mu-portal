---
title: Concurrent worlds
description: Several worlds stay connected at once, each in its own session, with one event stream for all of them.
sidebar:
  order: 20
---

## Decision

The server keeps several worlds connected at the same time, on one server or on several. Two characters on the same game are two worlds.

Each world has a `WorldId`. The id is the name of the world in the configuration, for example `example-mux/alt-two`. Each command names a `WorldId`, and each event carries one.

Each world runs in its own session, as one task with a handle. Commands go in through the handle, and events come out. A world connects, drops and reconnects without an effect on the other worlds.

The journal has one sequence for the events of all worlds. A client reads all of them from one `Subscribe` stream and resumes with one sequence number. A client that shows one world filters the events by `WorldId`. The journal keeps an index by world for the scrollback of one world.

## Reason

The owner plays several characters at once, on one server or on several. A stalled connection to one server must not block another, and one task for each world keeps them apart.

One sequence keeps the stream of the [ConnectRPC decision](/decisions/connectrpc-api/) as the spike tested it. That stream is one stream, with no gaps in its numbers, and a resume from one number. A sequence for each world needs one number for each world on resume. Activity marks for other worlds also come for free, because every event already reaches each client.

A name from the configuration is readable in logs, in scripts and in the API. A number is not.

## Source

The Phase 1 exploration. Issue 61 tracks the changes that build it.
