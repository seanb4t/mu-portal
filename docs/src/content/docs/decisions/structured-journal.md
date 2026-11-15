---
title: Structured journal
description: An ordered record of events is the source of truth.
sidebar:
  order: 10
---

## Decision

The server writes every event of a world to a journal: each line from the game, each line the player sends and each change of state. Each event has a sequence number. The journal is the source of truth.

A text log and an HTML log are renderings of the journal. A search index is built from it.

## Reason

A client that reconnects asks for the events after a sequence number. That request needs one ordered record. Logs that are renderings can change format later, and the old play stays readable. A structured record also lets a tool or an agent read play history safely.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it.
