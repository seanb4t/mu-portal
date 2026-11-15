---
title: Lua 5.4 scripting
description: Scripts use stock Lua 5.4, with limits on shared plugins.
sidebar:
  order: 8
---

## Decision

Scripts use stock Lua 5.4. The scripts of the owner are trusted. A shared plugin runs with limits: no file access, no process access, no module loading, no bytecode, a memory limit and an instruction limit.

Simple triggers and highlights do not need a script. A rule file in TOML states them. The rules use the priority and fall-through model of TinyFugue.

Scripts run on the server and never block a client.

## Reason

Lua is the scripting language that MU* players already know from MUSHclient. Stock Lua keeps existing habits and libraries. The limits exist because a plugin from another person is not trusted code.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it.
