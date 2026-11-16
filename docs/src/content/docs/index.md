---
title: mu-portal
description: A client and server for MU* text games.
---

mu-portal is a client for MU* text games: MUSH, MUX, AresMUSH and Evennia games. It takes its keyboard-first style from TinyFugue, and it takes features from MUSHclient and BeipMU.

## Status

The project is at its skeleton. The repository builds one binary, `mu`, with three verbs. `mu server` holds the game connections, and `mu client` is a terminal client that attaches to it. The only worlds are fake worlds for tests. The [first-run guide](guides/first-run/) shows how to start both.

## Shape

- One program, `mu`, carries every function as a verb.
- One server holds every game connection for one person.
- Zero to many clients attach to that server at the same time, for example a laptop, a phone and a tablet.
- A client reaches the server only through the API.

## Where to read next

- [Decisions](/decisions/server-centered/) records each project-wide decision and its reason.
- [Architecture](/architecture/system-context/) shows the system as diagrams.
- [Contributing](/contributing/build/) explains how to build, test and change the project.
