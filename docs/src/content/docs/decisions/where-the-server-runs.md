---
title: Where the server runs
description: Linux or macOS, in a container or not, always on.
sidebar:
  order: 6
---

## Decision

The server runs on Linux and on macOS, in a container or as a plain process. It can run on a laptop, on a home machine or on a cloud container service. The cloud service must keep one instance running at all times.

The server reads secrets through one interface. The stores behind it include the macOS Keychain, the Linux Secret Service, an encrypted file and environment variables.

## Reason

A game connection is a long-lived TCP connection. A platform that stops an idle process drops that connection, so a function platform does not fit the server. Linux and macOS are both first-class because the owner uses both.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it.
