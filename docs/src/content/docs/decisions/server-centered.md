---
title: Server-centered shape
description: One server for one person, with zero to many clients attached.
sidebar:
  order: 1
---

## Decision

The product is one server and zero to many clients. One person owns the server. A laptop, a phone and a tablet can all attach at the same time.

The server holds every game connection, the scrollback and the scripts. A client shows the output and sends the input. A client reaches the server only through the API.

The terminal client starts a local server by itself, as `tmux` does. The player does not have to start one first.

## Reason

A game connection must stay open while the player moves between devices. A server that outlives each client keeps the connection and the history. A strict boundary between the client and the server also lets a later client use the same API.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it.
