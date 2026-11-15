---
title: Importers
description: BeipMU configuration first, TinyFugue files later.
sidebar:
  order: 11
---

## Decision

The first importer reads a BeipMU configuration and writes mu-portal worlds and rules. The configuration of the owner is the acceptance test. An importer for TinyFugue `.tfrc` files comes later.

The project does not build an evaluator for the TinyFugue macro language. An importer converts what it can and reports the rest.

## Reason

The owner plays with BeipMU today, so that importer makes the client usable for its first user. The TinyFugue macro language is large, and a faithful evaluator is a project by itself.

## Source

The first exploration. The OpenSpec change `project-foundation` recorded it.
