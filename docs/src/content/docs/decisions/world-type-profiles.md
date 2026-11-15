---
title: World-type profiles
description: One profile for each kind of game server.
sidebar:
  order: 9
---

## Decision

Each world has a type, for example PennMUSH, TinyMUX, RhostMUSH, AresMUSH or Evennia. A profile for each type holds the differences between the servers. Examples are the login command, the keepalive, the text encoding and the supported telnet options.

A player who adds a world picks its type. The player can override each value.

## Reason

The game servers differ in small ways that break a client that treats them alike. A profile puts those differences in data and keeps them out of the connection code. A new server type then needs a new profile and no new code path.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it.
