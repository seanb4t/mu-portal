# Proposal

## Why

The pose-editor change (#56) left three faults in the terminal client. If a send gets no reply, every later send of its world stops for the rest of the session. Nobody can see the lines that wait (#147). A world that the first list of the worlds did not hold has no line break, and Alt+R blames the world (#148). A paste in the first 500 ms arrives as keys, so each line break of it sends a line (#149, and row R-35 of #136).

The change is tracked by issue #147. It also fixes #148 and #149.

## What Changes

- A send waits at most 30 seconds for its reply. After that, the client counts it as refused: the notice says that the game can have the line or not, and the line and each line behind it go back to the editor.
- A lost event stream and a reset of the server put the lines that wait back in the editor, with a notice. The line that is with the server stays, so its reply still decides what happens to it.
- The top row shows how many lines of each world wait behind the line that is with the server.
- If lines wait, Ctrl+C shows a notice and does not quit. A second Ctrl+C quits. Any other key cancels the second one.
- The client asks for the list of the worlds again in three cases. The first event of a new world comes. The server resets. Alt+R finds a world with an unknown line break. One list goes at a time.
- Alt+R on such a world says that the client does not know its line break yet, and does not say that the world has none.
- The startup probe reads a bracketed paste as one paste. If a paste is still open at the end of the probe, the probe reads for at most 500 ms more.

## Choices the owner will notice

- The limit of a send is 30 seconds, fixed. No configuration changes it. The server gives up on a connect or a write to a game after 10 seconds, so its own answer comes first when a game is slow.
- A list of the worlds has the same limit of 30 seconds. A list with no answer counts as failed.
- A send that hits the limit goes back to the editor although the game can have it. A second send of it can repeat it in the game. The notice says so.
- The count shows as `(2 waiting)` after the world in the top row. The line that is with the server is not in the count, so a single send shows no count.
- The second Ctrl+C has no time limit. Only another key cancels it.
- The notice of Ctrl+C counts the line that is with the server too: `3 lines not sent yet, quit again to leave and drop them`.
- A failed list of the worlds shows no notice. The next new world, reset or Alt+R asks again.
- Alt+R with no world in focus still shows a line about the line break, and asks for no list.
- Alt+R on a world with an unknown line break shows `the client does not know the line break of the world in focus yet, try again`. The user presses Alt+R again after the list comes.
- During the probe, a paste has no size limit. A paste that is still open after the extra 500 ms becomes one edit of its first part, and its rest arrives as keys.
- No test runs a paste in a real terminal during the probe, and no test runs a server that never answers. The tests drive `update`, `view`, the probe framer and the reply function.

## Not in this change

- A send queue on the local disk that survives a quit or a crash (#156).
- A key that cancels the sends that wait, and a view of their text. When the lines go back to the editor, the user sees them.
- A limit that the user configures.
- A paste that arrives after the probe and is cut by the probe's end. Termina reads the rest.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-client`: four requirements change.
  - "Ctrl+C quits the client only"
  - "A refused text stops the sends after it"
  - "Alt+R joins the lines in place"
  - "The client measures character widths at startup"

  Two requirements are new: "A send with no reply is refused" and "The top row counts the sends that wait".

## Impact

- Code: `crates/mu-tui/src/update.rs`, `model.rs`, `view.rs`, `run.rs`, `probe.rs` and `lib.rs`.
- Tests: `crates/mu-tui/tests/app.rs`, `probe.rs` and the new `send.rs`.
- Dependencies: the `test-util` feature of `tokio` in the dev-dependencies of `mu-tui`, for a paused clock in `send.rs`.
- The convention of the field: TinyFugue and BeipMU send each line at once and do not wait for a reply. This client waits for each reply, so it needs the limit. That is a choice of the pose-editor change, and this change keeps it.
