---
title: Output pane
description: The output pane is a ratatui widget over a chunked line store, and it wraps only the lines on screen.
sidebar:
  order: 16
---

## Decision

The output pane of the terminal client is a ratatui widget. It does not write to the terminal directly.

The pane keeps its scroll position as an anchor: a line and a byte in that line. The top row of the screen is the row that holds the anchor byte. A resize wraps only the lines on the screen again. The pane keeps no index of rows for the whole store. A scroll bar shows the position as a fraction of the lines, not of the rows.

The line store has this shape:

- The store keeps lines in chunks of a fixed number of lines. A line id is absolute and never used again, so a position stays valid after the store drops its oldest chunks.
- A chunk holds the text of its lines in one string, the style spans in one list, and one small record for each line. The record holds the end of the text, the end of the spans, the display width and flags.
- A style span is a byte offset and an id in a table of styles. At the arrival of a line, the client converts its game colors to spans. The text holds no control characters.
- An image is one line of its own. It takes a fixed number of rows at every width, and it is never wrapped.
- A selection is a pair of logical positions, a line and a byte. A copy gives the lines of the game, not the wrapped rows. The copy goes to the clipboard through OSC 52.

The pane shows an image with kitty unicode placeholders and a virtual placement. Each placeholder cell carries its row mark and its column mark, and the image id is the foreground color.

## Reason

The spike measured each part with one million lines. The store held them in less memory than twice the text. A wrap of all lines took more than one frame, so a row index for the whole store must not be on the resize path. A wrap of only the lines on the screen took a small part of one frame.

The ratatui widget used more processor time than direct writes, but both used a small part of one frame. If nothing on the screen changes, ratatui writes nothing. ratatui also lets the pane share the screen with the input line, the status line and other panes. Placeholder cells pass through the ratatui buffer without a change. A cell holds a string, and the diff writes each changed cell with both marks, so each cell finds its place in the image.

Mouse capture is on, so the pane does its own selection. The terminal selection is still available with the modifier key of the terminal.

## Source

The spike of issue 3, merged in pull request 29. The code is `spikes/output-pane`. The command `cargo run --release -p output-pane -- bench` in `spikes/` gives the current measurements.
