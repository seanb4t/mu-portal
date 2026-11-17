# Proposal

## Why

The engine reads games through `LineFramer`, which splits on LF and knows only the escape `IAC IAC` and the timing mark. A real game negotiates options, sends subnegotiations and can compress its output with MCCP2. So world-sessions (#54) cannot connect to TinyMUX, AresMUSH or Evennia until the engine parses telnet. This change builds that parser. Issue #52 tracks it, and it is step 2 of the Phase 1 plan in #61.

## What Changes

- A telnet engine in `mu-core` that does no I/O. The caller gives it the bytes of each read, and it gives back events and the bytes to send. It reads data, with `IAC IAC` as the byte 255, and two-byte commands. It negotiates options with the Q method of RFC 1143 under an accept policy. It reads subnegotiations for any option up to 1 MiB, and MCCP2 compression that can start inside a read. It reports protocol errors. It gives the same events for any split of the same bytes into reads.
- `TelnetFramer`, a framer behind the existing `Framer` seam that uses the engine. It gives lines, prompts, replies, marks and GMCP messages as `Inbound` items. The session does not change.
- A prompt is the text with no line end at the end of a read, or the text before GA or EOR. If a later read finishes that text with a line end, the framer gives the whole line. The lines are the same for any split of the bytes.
- The answers of a MU client: the terminal type with MTTS, the window size (NAWS), and the character set (CHARSET, RFC 2066, with the `[TTABLE ]` prefix and its version byte). The character set to accept is a setting of the framer.
- The text of a world in its character set: UTF-8 or ISO-8859-1. The framer decodes lines and prompts, and encodes input, in that character set. TinyMUX needs ISO-8859-1, because it takes no input after the client accepts UTF-8.
- The six recorded transcripts of the telnet spike, copied into `mu-core` as test data.

## Not in this change

- Connections to real games: TCP, login, keepalive and reconnect. These belong to world-sessions (#54). No world uses `TelnetFramer` until then, and the fake world keeps `LineFramer`.
- The quirks of each server family. Examples are the ISO-8859-1 character set of TinyMUX and the data after GMCP `Core.Goodbye`. world-config (#53) holds them, and world-sessions passes them to the framer.
- Character sets other than UTF-8 and ISO-8859-1.
- The behavior of real games with `IAC DO TIMING-MARK`. `Portal::wait_idle` stays a test helper, and world-sessions decides the behavior with real games.
- GMCP messages from the client, such as `Core.Hello`. The framer reports GMCP from the game. A later change decides what the client sends.
- MCCP3, MXP, MSDP, TLS and the GMCP, MSSP and MSDP payload formats.
- The deletion of `spikes/telnet-engine`. It stays as evidence until the last change that cites its decision page is archived.

## Capabilities

### New Capabilities

- `telnet-protocol`: how the engine reads and answers the telnet protocol of a game. It also covers how the telnet framer gives lines, prompts, replies, marks and messages.

### Modified Capabilities

None. The `Framer` seam, the session and the world-sessions requirements stay as they are.

## Impact

- Code: `crates/mu-core/src/telnet.rs` grows from the codes into the engine, with submodules under `crates/mu-core/src/telnet/`. `framer.rs` gains `TelnetFramer`, and `lib.rs` exports it.
- Test data: `crates/mu-core/tests/fixtures/telnet/` holds the six transcripts.
- `LineFramer`: its line split moves into a buffer that both framers share. A line of exactly the line bound no longer gives an extra empty line.
- Dependencies: one new crate for the inflate of MCCP2, through `[workspace.dependencies]`.
- Rules: the codes of telnet still live only in `src/telnet.rs`. The test `source_of_mu_core_keeps_its_rules` adds the commands from 239 to 250 to the codes that it finds.
- Boundary test: `ALLOWED_CRATES` of `mu-core` gains `miniz_oxide` and `hex`.
