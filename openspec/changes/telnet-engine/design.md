# Design

## Context

See proposal.md for the reason. The specs are in `specs/telnet-protocol/spec.md`.

The session reads a game through the `Framer` seam of `crates/mu-core/src/framer.rs`. `Framer::feed` takes the bytes of one read and gives `Inbound` items: a line, a prompt, reply bytes, the end of a mark, or a message of a side protocol. `Framer::encode` gives the bytes of one line, and `Framer::mark` gives the bytes of a mark. The session handles each item in `Session::on_bytes` and needs no change for a new framer.

`LineFramer` is the only framer today. It splits on LF, drops CR, reads `IAC IAC` as the byte 255 and reads the answer to a mark. It cuts a run with no LF at `MAX_LINE`. The fake game uses its game side, `LineFramer::game()`.

`crates/mu-core/src/telnet.rs` is a private module with the codes IAC, DO, WILL, WONT and TIMING-MARK. The test `source_of_mu_core_keeps_its_rules` in `crates/mu-cli/tests/source_rules.rs` scans each file of `mu-core` other than `src/telnet.rs`. Its list `CODES` holds 251 to 255 and 6, and it fails for those codes in the forms that it finds.

The spike `spikes/telnet-engine` built an engine with the same goals. Its decision page is `docs/src/content/docs/decisions/telnet-engine.md`, and pull request #32 merged it.

## Goals / Non-Goals

**Goals:**

- An engine that a test can drive with bytes alone, with no socket and no clock.
- A framer that a world can use in place of `LineFramer`, with no change to the session.
- Tests that hold for any split of the bytes into reads.

**Non-Goals:**

- A change to the `Framer` trait or to `Inbound`. The framer reports protocol errors as messages, so the seam stays as it is.
- A second NAWS message after a change of the client window. The framer sends the size of its settings.
- The effect of an option on the client, for example ECHO on input. The framer accepts ECHO, as the spike client did, and does nothing more with it.
- A disconnect after a bad MCCP2 stream. The engine goes quiet. World-sessions decides the time to close the connection.
- The issue #73 on the source test. This change adds codes to its list and changes nothing else in it.

## Decisions

### The engine lives under `telnet.rs`, and the codes stay in it

`src/telnet.rs` becomes a public module. It keeps every code of telnet: the commands from EOR (239) to IAC (255), and the option codes in `telnet::opt`. It also keeps two small functions on the codes. `telnet::escape` doubles each byte 255 of a byte slice. `telnet::MARK` is the bytes `IAC DO TIMING-MARK`. Its submodules hold the logic: `src/telnet/q.rs` for the Q method and `src/telnet/engine.rs` for the engine. Each submodule names a code by its constant.

The source test adds the commands from 239 to 250 to its list `CODES`. No file of `mu-core` holds such a literal today. Its test `bytes_that_are_not_codes_pass` holds the case `let x = 250;`, which becomes a code. That case moves to 238, the first number below the range. `telnet_code_is_found_in_each_form` gains a case for 239, the lowest code. The tests in `crates/mu-core/tests/` use the constants, because the module is public. A test that writes `255` or `250` fails the source test.

An option code such as GMCP (201) or NAWS (31) is a common number in other code. So a scan of every such number gives false hits. The source test finds an option code by its place instead. A literal right after the name `WILL`, `WONT`, `DO`, `DONT` or `SB` in a token list is an option code, as in `[IAC, WILL, 201]`. A case for each of the five names proves the rule, and `[IAC, WILL, opt::GMCP]` passes it. An option code in any other place passes the test, and review covers it.

Alternative: one file `telnet.rs` with all the logic. It holds the codes and the engine in more than 1000 lines.

Alternative: widen `TELNET_HOME` to the directory `src/telnet/`. That makes the rule weaker, and the submodules do not need it.

### The engine API

`Engine::new(policy)` makes an engine. `Engine::receive(bytes)` reads one read and queues events and output. `Engine::poll_event` gives the next event, and `Engine::take_output` gives the bytes to send. `Engine::request` asks to turn an option on or off. `Engine::send_subnegotiation` queues a subnegotiation and doubles each byte 255 of its payload with `telnet::escape`. The engine has no step that sends data. `Framer::encode` takes `&self`, so `TelnetFramer::encode` calls `telnet::escape` itself. So the escape of the byte 255 has one home, and each path to the game uses it.

The engine names the two ends of an option with `telnet::Side`. `Local` is an option that the client performs, and `Remote` one that the game performs. The private enum `Side` of `framer.rs` becomes `Role`, with `Client` and `Game`, because it names the end that a `LineFramer` reads for.

These are the events:

- data
- a two-byte command, with its code
- an option that turns on or off on one side
- a subnegotiation
- the end of a mark
- the start and the end of compression
- a protocol error

`IAC` and any byte other than IAC, SB, WILL, WONT, DO and DONT give a two-byte command. So `IAC SE` outside a subnegotiation and `IAC` before an unknown byte are commands with that code, and the next byte is data.

### The bound of a subnegotiation

`telnet::MAX_SUBNEGOTIATION` is 1 MiB. The engine counts the bytes of a payload after it reads `IAC IAC` as one byte, so an escaped byte counts once. When a payload reaches the bound, the engine keeps no more bytes and marks the payload as too long. It reads on to `IAC SE` in every case, so the framing stays in step. At `IAC SE`, a payload over the bound gives `ProtocolError::SubnegotiationTooLong` with the option and no subnegotiation event, and the bytes after it are data again. This is the rule of the spike, and the state of the bound holds across reads.

### The source of the engine

This is the shape of the spike, which the decision page measured. The code is new and written under the tests of this change.

Alternative: libmudtelnet. The decision page measured it wrong on `IAC IAC` in data and in a subnegotiation, and on the two-byte commands. It also reports a subnegotiation only for an option that the client performs. So the rule to reuse open source first does not apply to it.

### The Q method in its own module

`telnet::q` holds the state of one option on one side, and the steps of RFC 1143, section 7. The states are `No`, `Yes`, `WantNo` and `WantYes`. Each want state holds its queued opposite. A step takes a state and a message and gives the next state, the reply and the report. It is a pure function, so a unit test can walk each row of the table.

A policy names the options that the client accepts on each side. Take an option in the state `No` that the policy refuses. Each request for it gets the negative reply, and the state stays `No`. This is the rule of RFC 1143, section 7, "Upon receipt of WILL", and the same rule for DO. A request for an option that is already in the asked state gets no reply. So a refused option never loops: each request gets one reply, and the engine sends no request of its own for it.

### The mark is outside the option table

RFC 860 asks for one answer to each `DO TIMING-MARK`. The Q method sends no second `DO` for an option that is already on. So the engine reads `IAC WILL TIMING-MARK` and `IAC WONT TIMING-MARK` as the end of a mark and changes no state. `IAC DO TIMING-MARK` from the game goes through the table like any other option that the policy refuses. So the engine answers `IAC WONT TIMING-MARK` each time.

`TelnetFramer::mark` and `LineFramer::mark` both give `telnet::MARK`. The tests of the fake game use the same constant.

### Two parsers of the mark, on purpose

`LineFramer` keeps its own small parser of `IAC IAC` and the mark. The fake game and every test of the fake world use it, and it keeps each other IAC sequence in the line as data. The engine reads those sequences as telnet. A `LineFramer` on the engine changes the lines of the fake game for any test that sends an IAC byte. Its game side also answers `DO TIMING-MARK` with `WILL`, which the client engine refuses.

So the two parsers stay. They share the codes, `telnet::MARK` and the line buffer. World-sessions moves real worlds to `TelnetFramer`. When no world of the server uses `LineFramer`, a later change can retire it.

### The line buffer is shared by both framers

`LineFramer` and `TelnetFramer` split lines the same way. The partial line, the LF split, the drop of CR and the cut at `MAX_LINE` move into one crate-private type, `LineBuffer`, in `framer.rs`. Each framer feeds it data bytes.

The cut moves to the byte after the bound. Today the partial line is cut as soon as it holds `MAX_LINE` bytes. So a line of exactly `MAX_LINE` bytes and then LF gives that line and an extra empty line. In `LineBuffer`, a full partial line followed by LF is one line. A CR and a NUL after a CR are dropped before the check, so they never start a cut. A full partial line followed by any other byte is cut. So a line of exactly `MAX_LINE` bytes that ends with CR LF is one line too. The existing tests of `LineFramer` keep their results.

`TelnetFramer` also drops a NUL right after a CR, because RFC 854 sends a bare CR as `CR NUL`. The buffer remembers a CR at the end of a read, so the rule holds across reads.

Alternative: a second copy of the split in `TelnetFramer`. Two copies can disagree on the bound.

### Prompts

Take a read that added a data byte to the partial line. At the end of its `feed`, `TelnetFramer` gives the partial line as a prompt, if it is not empty. A read that ends at LF leaves an empty partial line, so it gives no prompt. A read of negotiation alone adds no data byte, so it gives no prompt. A prompt such as `> ` after a line is new data, so it shows each time. GA and EOR give the partial line as a prompt and clear it. A later LF ends the line, and the framer gives the whole line, with the text of the prompt in it.

The lines do not depend on the split, because only GA, EOR, LF and the cut clear the partial line. The prompts depend on the split, as the spec says.

Alternative: prompts on GA and EOR alone. No family of the spike sent either. So a prompt shows only after a line end.

### The answers of a MU client

`TelnetFramer::new` takes a `TelnetSettings` value: the character set to accept and the window size. `TelnetSettings::default()` gives `UTF-8` and 120 by 40. World-sessions adds the quirks of each world type to this struct, so the signature of `new` does not change. `ResolvedWorld` stays the resolved configuration of a world. World-config (#53) adds the character set to it, and world-sessions builds `TelnetSettings` from it for each connection. So `TelnetSettings` is the input of the framer, not a second home of the configuration.

The framer answers these, as the spike client did:

- TTYPE SEND: `MU-PORTAL`, `XTERM-256COLOR`, then `MTTS 269` for each later request. The MTTS bits are ANSI (1), UTF-8 (4), 256 colors (8) and truecolor (256).
- NAWS on for the client: the window size, through `send_subnegotiation`, which doubles a byte 255.
- CHARSET REQUEST: `ACCEPTED` with the name as the game wrote it, or `REJECTED`. The parse reads the prefix `[TTABLE ]` with its space, then skips the version byte. A prefix with no version byte and an empty list give `REJECTED`.

The review of pull request #32 found that the spike stripped the prefix with no space. The commit `fix(spikes): read the RFC 2066 TTABLE prefix with its space` (#35) fixed the spike. This change keeps the fixed parse and tests it.

The policy accepts TTYPE, NAWS and CHARSET for the client, and ECHO, SGA, EOR, CHARSET, MSSP, MCCP2 and GMCP for the game. This is the policy that recorded the transcripts.

`TelnetFramer::encode` gives `telnet::escape` of the line in the character set of the framer, then CR LF. A UTF-8 line never holds the byte 255, but a Latin-1 line can.

### The character set of the text

`TelnetSettings` holds the character set as an enum, `Charset::Utf8` or `Charset::Latin1`. The name that the CHARSET answer uses comes from the enum: `UTF-8` or `ISO-8859-1`. `LineBuffer` keeps bytes, and the framer decodes a line or a prompt when it gives it. UTF-8 decodes with replacement characters, as `LineFramer` does today. ISO-8859-1 maps each byte to the character with the same number, which `char::from(u8)` does in the standard library. `encode` maps each character below 256 to its byte and each other character to `?`, then runs `telnet::escape`. So `ÿ` (255) goes to the game as `IAC IAC`.

The character set is a setting, not a result of the CHARSET answer. TinyMUX reads Latin-1 when the client gives no answer, so world-sessions sets ISO-8859-1 for TinyMUX either way.

The property test `latin1_round_trip` encodes each character below 256 and decodes the bytes again, 128 to 159 included.

Alternative: `encoding_rs`. Its label `ISO-8859-1` maps to windows-1252, which gives other characters for the bytes 128 to 159. The standard library gives the true ISO-8859-1 map, so no crate is needed.

### Messages and errors

A GMCP subnegotiation from the game becomes `Inbound::Message` with the name `GMCP`. The framer drops each other subnegotiation that it does not answer, for example MSSP.

A protocol error becomes `Inbound::Message` with the name `telnet` and a text that names the error. The session records it as a message event, so the player sees it.

Alternative: a new `Inbound::Error`. It changes the seam and the session for a case that world-sessions handles with a disconnect.

### MCCP2 with `miniz_oxide`

MCCP2 starts on `IAC SB MCCP2 IAC SE` while MCCP2 is on for the game. While it is off, that subnegotiation is an ordinary event. The engine inflates with the streaming API of `miniz_oxide`, which tells how many input bytes a call used. So the engine knows where a stream ends inside a read, and it reads the bytes after it as plain telnet.

A stream that is not valid zlib gives one protocol error. After it, the engine drops each later byte and gives no event, because no byte after it can be read. So the error and the silence after it are the same for any split.

The data before the error can differ by split. Inflate gives out what it can decode from the bytes of each call. A stream that breaks in its middle can break at a byte that one split reaches first. To hold the data back until the stream ends needs a buffer with no bound. So the spec states this case as the one exception to the split rule. The split property test uses bad streams that fail in their header, where no data comes before the error. A unit test, `bad_stream_after_data`, checks a stream that breaks after some data: one error and silence for any split.

The engine opens a stream only when none is open. The spike guards the same case. An `IAC SB MCCP2 IAC SE` inside an open stream is an ordinary subnegotiation event.

The crate is pure Rust under MIT, Zlib or Apache-2.0, and `deny.toml` allows all three. The spike used it, and `Cargo.lock` does not hold it yet. The boundary test lists the third-party crates of each member in `ALLOWED_CRATES`. This change adds `miniz_oxide` and `hex` to the list of `mu-core`.

Alternative: `flate2`. It wraps `miniz_oxide` by default and adds a layer that the engine does not need.

### Test data and test helpers

The six transcripts go to `crates/mu-core/tests/fixtures/telnet/`, beside the use of `tests/fixtures` in `mu-cli`. A parser of the transcript format goes into `crates/mu-core/tests/support/mod.rs`. It decodes the hex of each record with the `hex` crate, a dev-dependency. `Cargo.lock` already holds `hex` 0.4.3.

The transcript test reads its files at run time, from a path under `CARGO_MANIFEST_DIR`. The boundary test of `crates/mu-cli/tests/workspace_boundary.rs` holds every rule that keeps the crates away from `spikes/`. So it gains one more: a new `Violation::SpikeLink` for any symbolic link under `crates/` whose target resolves into `spikes/`. A probe case makes a link in a `TreeCopy` of the fixture and makes sure that the boundary test reports it. This covers the transcripts and each other file of each crate.

`tests/support/mod.rs` also gets `reads(bytes, points)`, which splits a byte slice at the given points. The engine, transcript and framer tests use it. The unit tests in `framer.rs` keep their own `feed_split`, because a unit test cannot reach `tests/support`.

The property test of the engine builds random streams from parts: data, two-byte commands, negotiations, subnegotiations, marks, compressed runs and, as the last part, a bad stream. Malformed framing is a part too: `IAC SB IAC SE`, a negotiation verb inside a subnegotiation, and an `IAC` as the last byte of the stream. Each of these gives the events that the engine rules state. Each part also gives the events that a correct parser must report. So the test compares the engine with this oracle, not with another parser. The compressed runs use the deflate side of `miniz_oxide`. A second property test, `engine_round_trip`, sends random data through `telnet::escape` and random payloads through `send_subnegotiation` of one engine. A second engine reads them back.

### Using the spike

This change follows "Using a spike" on `docs/src/content/docs/contributing/spikes.md`:

1. The decision page is `docs/src/content/docs/decisions/telnet-engine.md`, and the spike merged in pull request #32.
2. Pull request #32 has one review finding, the TTABLE prefix. Commit #35 fixed it in the spike, and the section "The answers of a MU client" keeps the fix and tests it.
3. The items under Not tested map as follows:
   - The cause of the TinyMUX stall after `ACCEPTED UTF-8` is a non-goal. World-config gives TinyMUX ISO-8859-1.
   - The cause of `?` for UTF-8 in PennMUSH is a non-goal. PennMUSH is not a Phase 1 target.
   - Prompts with GA or EOR: this change reads them, and tests use bytes that the tests write. Real games with GA or EOR are a non-goal.
   - MCCP3, TLS, MXP and MSDP are non-goals. TLS is #58. The policy refuses MXP and MSDP.
   - Servers with other configurations and long sessions are a non-goal of this change. World-sessions connects to real games.
4. The code is new. The tests of this change come first.
5. The transcripts are copied to `crates/mu-core/tests/fixtures/telnet/`, a path with no directory named `spikes`.

### Enforcement of each rule

| Rule | Enforced by |
|---|---|
| The commands of telnet live only in `src/telnet.rs` | The test `source_of_mu_core_keeps_its_rules`, with 239 to 255 and 6 in its list `CODES`. |
| The option codes live only in `telnet::opt` | The source test, for a literal after `WILL`, `WONT`, `DO`, `DONT` or `SB`. Review covers each other place. |
| No crate file includes a spike file | The boundary test in `crates/mu-cli/tests/workspace_boundary.rs`. |
| No crate reads a spike file through a symbolic link | `Violation::SpikeLink` of the boundary test, and its probe case `spike_link_is_found`. |
| Each third-party crate of `mu-core` is allowed | `ALLOWED_CRATES` in the boundary test, and `cargo deny check` in `mise run check`. |
| The engine gives the same events for any split | The property test `engine_ignores_read_splits`, with bad headers and malformed framing, and the transcript test `transcripts_in_other_reads`. |
| A bad stream gives one error and silence for any split | The test `bad_stream_after_data`. |
| The bound counts an escaped byte once and holds across reads | The tests `payload_at_the_bound` and `payload_over_the_bound`. |
| A stream opens only when none is open | The test `start_inside_a_stream`. |
| The engine matches the oracle | The property test `engine_matches_the_oracle`. |
| Data and payloads survive the escape | The property test `engine_round_trip`. |
| The Q method follows RFC 1143 | The unit tests of `telnet::q`, one for each row of the table in section 7. |
| The mark changes no option | The tests `answer_to_a_mark` and `mark_from_the_game`. |
| One copy of the mark bytes | `telnet::MARK`, used by both framers and the fake game tests. Review checks that no other copy appears. |
| The two framers split lines the same way | The unit tests of `LineBuffer`, and the existing tests of `LineFramer`, which keep passing. |
| A line of exactly the bound gives no empty line | The tests `line_of_exactly_the_bound` and `line_framer_line_of_exactly_the_bound`, each with LF and with CR LF. |
| No empty prompt | The test `read_that_ends_at_a_line_end`. |
| The lines of the telnet framer do not depend on the split | The property test `telnet_framer_lines_ignore_read_splits`. |
| The framer decodes and encodes in its character set | The tests `latin1_line_from_the_game`, `utf8_line_from_the_game` and `latin1_line_to_the_game`, and the property test `latin1_round_trip`. |
| No payload makes the framer fail | The property test `framer_survives_any_payload`. |
| Each fact about RFCs and `miniz_oxide` holds | Task 1.1 reads the sources and fills the table of checked facts. |

## Risks / Trade-offs

- [A line split across reads shows a false prompt.] → At the end of the line, the framer gives the whole line. Output-pane (#55) decides how to show a prompt that a line replaced. Until then, the plain view of the client shows both.
- [A game sends a prompt, waits, and adds text to the line.] → The framer gives a second, longer prompt, then the line. No text is lost.
- [The framer drops a subnegotiation other than GMCP.] → MSSP and the others have no reader yet. A later change can turn one into a message.
- [The game ignores `IAC DO TIMING-MARK`.] → `Portal::wait_idle` stays a test helper. World-sessions decides the behavior with real games.
- [The engine goes quiet after a bad MCCP2 stream, with the connection still open.] → The error message tells the player. World-sessions adds the disconnect.
- [A bad stream that fails after its header can give a different amount of data for different splits.] → The spec states the exception. The error and the silence after it are the same for any split, and `bad_stream_after_data` tests that.

## Checked facts

Task 1.1 fills this table with each fact that the design uses, with its source. The sources are RFC 854, 855, 860, 1073, 1091, 1143 and 2066, MTTS, MCCP2 and `miniz_oxide`.

| Fact | Source |
|---|---|
