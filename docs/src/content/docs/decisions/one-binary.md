---
title: One binary with verbs
description: The mu binary carries every function as a verb.
sidebar:
  order: 2
---

## Decision

The product ships as one executable, `mu`. Each function is a verb, in the style of `kubectl` and `docker`. `mu` with no verb starts the client. `mu server` runs the server.

A flag of the client works with the verb and without it. The command definition holds the client arguments once, so the two forms cannot differ.

## Reason

One binary is one thing to install, to update and to put in a container. The server and the client always have the same version. A new function becomes a new verb and needs no new program.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it. The `cli-entrypoint` capability holds the requirements.
