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

The client reads the replies itself, before its event reader starts. It waits at most `probe::WAIT` for DA1. The event reader gives the kitty graphics reply as key presses, and its parser loses a reply that follows a stray escape byte. So the replies must not reach it. A key that the user presses during the probe still reaches the client.

The probe frames its input. It cuts the bytes it reads into frames by the byte classes of ECMA-48. A CSI (control sequence introducer) starts with the escape byte and `[`. It holds parameter bytes and ends with one final byte. An APC (application program command) starts with the escape byte and `_`. It ends with the escape byte and `\`.

Termina decodes each CSI. The probe gives each complete CSI to a Termina parser alone. The cursor, mode and DA1 replies come out of the parser as events. The probe decodes the kitty graphics reply itself, because Termina gives an APC as key presses. A complete CSI that Termina decodes to no event gives no event and no reply. Termina's own reader does the same.

An escape byte ends an open CSI or an open APC. The one exception is the escape byte of the `\` that ends an APC. A byte that cannot be part of a CSI also ends an open CSI. These rules for a broken sequence are the probe's own. The probe turns the bytes of the broken sequence into key presses for the client, and a new frame starts. A stray Alt+`[` or an open APC cannot hide a reply that follows it. If DA1 came, a sequence that is still open when the probe stops becomes key presses, because no reply follows DA1.

The probe holds some bytes until its next read. It holds an open CSI or APC. It also holds an escape byte with at most one byte after it, and the first bytes of a UTF-8 character. So a key such as F1 reaches the client in one piece when its bytes come in two reads. F1 is the escape byte, `O` and `P`. So does a character such as `é`.

The probe holds at most `probe::PENDING_MAX` bytes. If an open CSI or APC grows past that limit, the probe drops its bytes. It then drops each later byte up to the end of that sequence. The end is the final byte of a CSI, a byte that breaks a CSI, or the escape byte and `\` of an APC.

The wait can end inside a reply. Then the probe keeps the open CSI, or a lone escape byte, in its outcome. It does not turn those bytes into key presses. The client queues them as one message after the keys that came during the probe. Those keys came before the open bytes, so they stay keys. The client holds the bytes if it still waits for late replies.

While the client holds the bytes, each key that is a CSI byte as text adds its byte. A key is a CSI byte as text if it is a character from space to `~` with no modifier but Shift. At the final byte, the client decodes the bytes with the decoder of the probe and handles the result as a late reply. So a cut DA1 reply still ends the wait, and a cut cursor reply never reaches the input line. After a lone escape byte, `[` continues the CSI, and `_` starts a late kitty graphics reply.

Any other key drops the held bytes. A byte past `probe::PENDING_MAX` does the same. The client then handles that key as it does while it waits for late replies. So Alt+`_` there starts a late kitty graphics reply. The client takes each key that the user types at the deadline, up to a final byte from `@` to `~`.

The kitty graphics reply ends with the escape byte and `\`. If the wait ends between them, the client sees the key Escape. In the late kitty graphics path, Escape waits for the `\` key that ends the reply.

The probe reads from stdin if stdin is a terminal. If it is not, the probe opens `/dev/tty`. Termina makes the same choice. The `mu-tui` crate uses `rustix` with its `termios` feature for the timed read.

A kitty graphics reply of `OK` with the id of the probe query turns graphics on, with or without DA1. The reply proves that the terminal supports images. An error reply, or a reply with another id, does not turn graphics on. Only the width profile needs DA1.

The probe reads with a timed read. It sets the terminal to return from a read after at most one tick (`probe::TICK`), even with no input. The terminal setting is VMIN at zero and VTIME at the tick. A read starts only if it can end before `probe::WAIT` is over. So a terminal that stays silent costs the client at most `probe::WAIT`. The probe can stop up to one tick early. A DA1 reply in that last tick counts as a late reply.

The probe sets the terminal back after its reads. If the read fails and the restore fails too, the client reports the read error. The message of that error also holds the restore error. The same rule holds when the client leaves, and when it enters the alternate screen. If only the restore fails, the client reports the restore error.

If the terminal does not answer before `probe::WAIT` ends, the client uses the `unicode-width` value for every class. When the client exits, it turns off mode 2027, unless the terminal gave the mode as on before the probe. A mode 2027 reply or a DA1 reply that comes after the wait still counts, if it comes before the client exits. The client drops a kitty graphics reply and a cursor reply that come after the wait.

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

- tmux. tmux answers the cursor position query from its own grid, so the probe measures the widths of tmux. A difference between tmux and the terminal outside it is not visible to the client.
- An East Asian locale where the terminal draws ambiguous characters two cells wide. The ambiguous class has its own probe, so the profile follows it.

## Source

The spike of issue 8, merged in pull request 30. The code is `spikes/char-width`, and the reports of each terminal are in `spikes/char-width/results`. The command `cargo run --release -p char-width -- probe` in `spikes/` runs the probe in the current terminal.
