---
title: Alternate-screen terminal client
description: The terminal client owns the screen, the scrollback and the wrapping.
sidebar:
  order: 7
---

## Decision

The terminal client uses the alternate screen of the terminal. It keeps its own scrollback, selection and search. It wraps text to its own width.

The server reports one fixed width to each game. The default is 120 columns, and the player can set it for each world. A client that changes size does not change that width.

Inline images and hyperlinks are required features. The first targets are Ghostty and Rex.

## Reason

Several clients with different widths attach to one game connection. The game can only know one width, so each client must wrap for itself. A client that owns its scrollback can also search it and split it into panes.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it.
