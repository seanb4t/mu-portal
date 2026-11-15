---
title: Character width
description: A startup probe learns one width rule for each class of grapheme, and the pane forces that width on each cell.
sidebar:
  order: 17
---

## Decision

The terminal client measures the width of characters at startup. It does not trust one width table for every terminal.

The probe runs once, on the alternate screen, before the first frame:

1. The client asks for the terminal name with XTVERSION and for the state of mode 2027 with DECRQM. Mode 2027 makes the terminal measure grapheme clusters, not code points. If the terminal knows the mode and it is off, the client turns it on.
2. The client writes one grapheme from each width class and asks for the cursor position after each one. It sends all the queries in one write and ends them with a device attributes query (DA1). Every terminal answers DA1, so the DA1 reply marks the end of the answers.
3. For each class, the client keeps the rule that gives the measured width. The rules are the `unicode-width` value, the sum of the code point widths, or a constant. A zero-width probe sets the width of a joiner (ZWJ) inside a sum.

If the terminal does not answer, the client uses the `unicode-width` value for every class.

The command `cargo run -p char-width -- list` in `spikes/` prints the class of each grapheme.

The client computes the display width of each line with the learned rules. The output pane gives every cell of a risky class the learned width with the ratatui cell option `ForcedWidth`. The buffer and the diff then use the same width as the terminal. After a cell with a width above one, ratatui moves the cursor before it writes the next cell.

## Reason

The spike measured 13 probe graphemes and then 35 other graphemes of the same classes. It also drew each test grapheme in a ratatui row and read the cursor position at the end of the row.

| Terminal | `unicode-width` misses | Profile misses | Rows that drift without the profile | Rows that overlap without the profile |
|---|---|---|---|---|
| Rex | 0 of 35 | 0 of 35 | 0 | 0 |
| Ghostty | 0 of 35 | 0 of 35 | 0 | 0 |
| Terminal.app | 13 of 35 | 0 of 35 | 2 | 9 |

Rex and Ghostty turn on mode 2027 by default, and they agree with `unicode-width`. Terminal.app does not know mode 2027. It adds the width of each code point, and it gives a joiner one cell. VS16 does not widen a character there, and VS15 does not narrow one. A family emoji takes eight cells.

With the profile, no row drifted or overlapped in any terminal. The probe took less than one millisecond.

A table of single graphemes cannot correct a grapheme that the probe did not measure. A rule for each class corrected all 35 test graphemes, and none of them was in the probe.

A fixed reservation of two cells for each risky grapheme, with no probe, stopped the drift. But it left an overlap in 7 rows in Terminal.app, and it wasted a cell for each VS15 character in Rex and Ghostty.

## Not tested

- tmux. tmux answers the cursor position query from its own grid, so the probe measures the widths of tmux. A difference between tmux and the terminal outside it is not visible to the client.
- An East Asian locale where the terminal draws ambiguous characters two cells wide. The ambiguous class has its own probe, so the profile follows it.

## Source

The spike of issue 8. The code is `spikes/char-width`, and the reports of each terminal are in `spikes/char-width/results`. The command `cargo run --release -p char-width -- probe` in `spikes/` runs the probe in the current terminal.
