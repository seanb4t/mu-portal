---
title: ConnectRPC API
description: Unary commands and one server stream, defined in protobuf.
sidebar:
  order: 3
---

## Decision

The API between a client and the server uses ConnectRPC with a protobuf schema. A client sends each command as one request with one reply. A client reads events from one server stream, and it names the sequence number to resume from.

The API has no stream in both directions.

## Reason

A protobuf schema gives every client the same typed contract. ConnectRPC works over plain HTTP, so a browser, a phone and a terminal can all use it. A stream that resumes from a sequence number lets a client reconnect and miss nothing.

The Rust library for ConnectRPC is young. If it does not hold up, gRPC with `tonic` uses the same schema.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it. A spike tests the API with resume before the schema is written.
