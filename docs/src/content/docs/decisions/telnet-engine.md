---
title: Telnet engine
description: The client parses telnet with its own engine that does no I/O, and each world type holds the quirks of its server family.
sidebar:
  order: 18
---

## Decision

The client uses its own telnet engine. It does not use libmudtelnet.

The engine does no I/O. The game session gives it the bytes of each read, and the engine gives back events and the bytes to send:

- Data, with IAC IAC read as one byte 255. The engine flushes the data at the end of each read, so a prompt with no line end reaches the client at once.
- Two-byte commands, for example GA, EOR, NOP and AYT.
- An option that turns on or off, for the client side or for the server side. The engine negotiates with the Q method of RFC 1143, and a policy names the options that the client accepts. The Q method keeps a state for each option on each side, so the two ends never loop.
- Each subnegotiation, for any option. The client decides what to do with it. The engine drops a subnegotiation longer than 1 MiB.
- The start and the end of MCCP2 compression. The engine inflates the compressed stream itself. A stream can start in the middle of a read, and the bytes after the end of a stream are plain again.
- Protocol errors, for example a stream that is not valid zlib.

The engine gives the same events for any split of the same bytes into reads.

A world-type profile holds the quirks of each server family. The quirks that this spike found are in the table below. The recorded transcripts are test data for the production engine.

## Reason

### The engine

The spike crate `spikes/telnet-engine` has these tests:

| Test | Cases | Result |
|---|---|---|
| Random streams through the engine and through libmudtelnet, split into random reads, with compression that starts inside a read | 2000 | Both agree, and both match the expected events |
| The same bytes split into other reads | 3000 | Same events and the same replies |
| Option negotiation (RFC 1143) | 7 | Pass |
| MCCP2: a start inside a read, plain bytes after the end of the stream, a bad stream | 4 | Pass |
| The six transcripts, as recorded and split into reads of 1 to 1500 bytes | 6 | No protocol error, same events |

The generator of the random streams also writes the events that a correct parser must report. The test compares each parser with that list, so a match with libmudtelnet alone proves nothing.

### libmudtelnet

The command `cargo run -p telnet-engine -- diverge` in `spikes/` counts the streams where each parser is wrong. The command adds each kind of input below to 3000 streams that both parsers read correctly.

| Input | Engine wrong | libmudtelnet wrong | What libmudtelnet does | Who breaks the standard |
|---|---|---|---|---|
| IAC IAC in data | 0 | 3000 | Reports command 255 and loses the data byte | libmudtelnet. RFC 854 makes IAC IAC the data byte 255. |
| IAC IAC in a subnegotiation | 0 | 3000 | Keeps both bytes in the payload | libmudtelnet. RFC 855 doubles IAC in a subnegotiation, and the receiver reads one byte. |
| AYT, DM, BRK and the other two-byte commands | 0 | 2661 | Reads one more byte as part of the command | libmudtelnet. RFC 854 defines these commands as two bytes. Only GA, EOR and NOP are two bytes in libmudtelnet. |
| None of the above | 0 | 0 | | |

libmudtelnet also reports a subnegotiation only for an option that the client performs. GMCP, MSSP and MCCP2 are options that the server performs, so the client must mark them as its own before libmudtelnet reports them. No RFC covers this. It is the design of the library.

libmudtelnet does not inflate MCCP2. It gives the rest of the read back, and the caller inflates it and parses the result. The engine does this inside, so the session code never handles a compressed byte.

### The server families

Each family ran in Docker with its default configuration. The version of each family is the pin in its Dockerfile. The recorder answered as a MU client: terminal type with MTTS, a window of 120 by 40, UTF-8, GMCP, MSSP, EOR and MCCP2. It sent a login, `look`, `WHO`, a line with UTF-8 text, a line with color and a quit.

| Family | At connect | Compression | Character set | Other quirks |
|---|---|---|---|---|
| PennMUSH | DO LINEMODE only, as a test for a telnet client. The offers come after the answer: DO TTYPE, DO NAWS, WILL CHARSET, WILL MSSP and WILL GMCP. | None | Offers UTF-8 in a CHARSET request. After the client accepted UTF-8, it sent each character outside ASCII as `?`. | Sends GMCP `Core.Goodbye` at quit, then the two bytes `c3 b0` (`ð`) after IAC SE. |
| TinyMUX | WILL EOR, DO EOR, DO SGA, DO TTYPE, DO NAWS, DO NEW-ENVIRON, WILL CHARSET, DO CHARSET, WILL MSSP and WILL GMCP | None | After the client accepted UTF-8, the server took no more input. It behaved the same way in 3 tries. With ISO-8859-1 or no answer, it reads input as Latin-1, and characters outside Latin-1 come back as `?`. | Puts each GMCP message from the client in its command queue, except Core.Hello, Core.Ping and Core.KeepAlive. Before login, such a message counts as a failed connect. Sends WILL BINARY and DO BINARY after a CHARSET answer. |
| RhostMUSH | Nothing until the first input from the client. Then DO NAWS, DO TTYPE and a CHARSET request with no WILL or DO CHARSET before it. | None | After the client accepted UTF-8, it sent `café ✓` as `cafe ?`. | Asks for the terminal type 4 times. Has no GMCP and no MSSP. |
| AresMUSH | DO NAWS, DO CHARSET and WILL MSSP | None | Asks for `utf-8` in lowercase. UTF-8 passes. | No terminal type. The banner ends 70 of its 81 lines with a bare LF. The first login waits for `tos/agree`. |
| Evennia | All at once: DO LINEMODE, WILL SGA, DO NAWS, DO TTYPE, WILL MCCP2, WILL MSSP, WILL MSDP, WILL GMCP and WILL MXP | MCCP2 starts 6 bytes into a read, and 498 compressed bytes follow in the same read. The stream stays open to the end. | No CHARSET. UTF-8 passes. | Asks for the terminal type 3 times. Sends GMCP `Logged.In` after login. |

No family sent GA or EOR after a line in these sessions. A profile cannot use them to find the prompt.

These quirks go into the world-type profiles:

- The character set that the client accepts. It is UTF-8 for every family except TinyMUX. For TinyMUX it is ISO-8859-1.
- The time to send GMCP. For TinyMUX the client sends GMCP only after the login.
- A line end can be a bare LF. The client reads LF alone as the end of a line for every family.
- The client ignores data after GMCP `Core.Goodbye`.

## Not tested

- The cause of the TinyMUX stall after `ACCEPTED UTF-8`. The source handles the answer, so a report to TinyMUX needs a build outside Docker first.
- The cause of `?` for UTF-8 in PennMUSH. The image has no ICU, because the configure script of PennMUSH looks for `icu-config`, and Debian no longer ships it.
- Prompts with GA or EOR. A game must turn them on, for example with a prompt command.
- MCCP3 (compression from the client), TLS, MXP and MSDP. The client refused MXP and MSDP.
- Servers with a configuration other than the default, and long sessions.

## Source

The spike of issue 7, merged in pull request 32. Its review found that the spike read the TTABLE prefix of RFC 2066 with no space. The commit `fix(spikes): read the RFC 2066 TTABLE prefix with its space` fixed it. The code is `spikes/telnet-engine`, and the transcripts are in `spikes/telnet-engine/transcripts`. The Dockerfiles of the servers are in `spikes/telnet-engine/servers`.

To record a family again, build and run its image, then run the recorder in `spikes/`:

```bash
docker build -t mu-spike/evennia telnet-engine/servers/evennia
docker run --rm -d --name mu-evennia -p 4205:4201 mu-spike/evennia
cargo run --release -p telnet-engine -- record --port 4205 --family evennia \
  --script telnet-engine/scripts/evennia.txt --out telnet-engine/transcripts/evennia.txt
```

The command `cargo run -p telnet-engine -- replay telnet-engine/transcripts/*.txt` prints the events of each transcript and a summary.
