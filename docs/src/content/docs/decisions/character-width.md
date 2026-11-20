---
title: Character width
description: A startup probe learns one width rule for each class of grapheme, and the pane forces that width on each cell.
sidebar:
  order: 17
---

## Decision

The terminal client measures the width of characters at startup. It does not trust one width table for every terminal.

The probe runs once, on the alternate screen, before the first frame:

1. The client asks for the state of mode 2027 with DECRQM. Mode 2027 makes the terminal measure grapheme clusters, not code points. Then the client turns the mode on. A terminal that does not know the mode ignores it.
2. The client writes one grapheme from each width class and asks for the cursor position after each one. It also asks whether the terminal draws kitty images. It sends all the queries in one write and ends them with a device attributes query (DA1). Every terminal answers DA1, so the DA1 reply marks the end of the answers.
3. For each class, the client keeps the rule that gives the measured width. The rules are the `unicode-width` value, the sum of the code point widths, or a constant. A zero-width probe sets the width of a joiner (ZWJ) inside a sum.

The client reads the replies itself, before its event reader starts, and waits at most 500 milliseconds for DA1. The event reader drops some replies and gives the kitty graphics reply as key presses, so the replies must not reach it. A key that the user presses during the probe still reaches the client.

If the terminal does not answer in that time, the client uses the `unicode-width` value for every class. When the client exits, it turns off mode 2027, unless the terminal gave the mode as on before the probe.

The command `cargo run -p char-width -- list` in `spikes/` prints the class of each grapheme.

The client computes the display width of each line with the learned rules. The output pane gives the learned width to every cell of a risky class, and to every cell whose learned width differs from the `unicode-width` value. It uses the ratatui cell option `ForcedWidth`. The buffer and the diff then use the same width as the terminal. After a cell with a width above one, ratatui moves the cursor before it writes the next cell.

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

The tests of the client cover a terminal that adds the widths of code points, a terminal that does not answer, and a terminal that answers after the first frame. A pseudo-terminal plays each terminal. In Rex, the client starts with the probe, and no reply reaches the input line.

These cases have no test:

- A kitty graphics reply that comes after the wait of 500 milliseconds. The event reader gives it to the input line as key presses.
- tmux. tmux answers the cursor position query from its own grid, so the probe measures the widths of tmux. A difference between tmux and the terminal outside it is not visible to the client.
- An East Asian locale where the terminal draws ambiguous characters two cells wide. The ambiguous class has its own probe, so the profile follows it.

## Source

The spike of issue 8, merged in pull request 30. The code is `spikes/char-width`, and the reports of each terminal are in `spikes/char-width/results`. The command `cargo run --release -p char-width -- probe` in `spikes/` runs the probe in the current terminal.
