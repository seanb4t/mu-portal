# Spec Delta

## Purpose

The parser reads the telnet protocol of a game and answers it as a MU client. The telnet framer turns that protocol into lines, prompts, replies, marks and messages for the session.

## ADDED Requirements

### Requirement: The parser reads IAC IAC as one data byte
The parser SHALL read `IAC IAC` in data as one data byte 255. The client SHALL double each byte 255 that it sends in data or in a subnegotiation.

#### Scenario: An escaped byte in data
- **WHEN** the parser reads the bytes `a IAC IAC b`
- **THEN** it gives the data bytes `a`, 255 and `b`, and no command

#### Scenario: The byte 255 in a subnegotiation to the game
- **WHEN** the parser sends a GMCP subnegotiation with the payload `a`, 255 and `b`
- **THEN** the bytes to the game are `IAC SB GMCP a IAC IAC b IAC SE`

#### Scenario: A line to the game
- **WHEN** the telnet framer encodes the line `hi`
- **THEN** the bytes to the game are `hi` and CR LF

### Requirement: Two-byte commands are two bytes
The parser SHALL read `IAC` and any byte other than IAC, SB, WILL, WONT, DO and DONT as a two-byte command. It SHALL give the command as an event, with the byte as its code. The byte after the command SHALL be data.

#### Scenario: AYT before data
- **WHEN** the parser reads `IAC AYT x`
- **THEN** it gives the command AYT and then the data byte `x`

#### Scenario: Each two-byte command
- **WHEN** the parser reads `IAC` and each of NOP, DM, BRK, IP, AO, AYT, EC, EL, GA and EOR, each followed by `x`
- **THEN** it gives that command and then the data byte `x`, for each one

#### Scenario: SE outside a subnegotiation
- **WHEN** the parser reads `IAC SE x` with no subnegotiation open
- **THEN** it gives the command SE and then the data byte `x`

#### Scenario: A byte that is no command
- **WHEN** the parser reads `IAC`, the byte 100 and `x`
- **THEN** it gives a command with the code 100 and then the data byte `x`

### Requirement: Options are negotiated with the Q method
The parser SHALL negotiate each option on each side with the Q method of RFC 1143, section 7. A policy SHALL name the options that the client accepts on each side. The parser SHALL refuse each other option each time the game asks for it. It SHALL NOT answer a request for an option that is already in the asked state.

#### Scenario: An accepted option
- **GIVEN** a policy that accepts GMCP on the side of the game
- **WHEN** the parser reads `IAC WILL GMCP`
- **THEN** it sends `IAC DO GMCP` and gives an event that GMCP is on for the game

#### Scenario: A refused option, twice
- **GIVEN** a policy that does not accept MXP
- **WHEN** the parser reads `IAC WILL MXP` two times
- **THEN** it sends `IAC DONT MXP` two times, gives no event, and MXP stays off

#### Scenario: A repeated request
- **GIVEN** a parser where GMCP is already on for the game
- **WHEN** the parser reads `IAC WILL GMCP` again
- **THEN** it sends nothing and gives no event

#### Scenario: A request of the client that crosses a request of the game
- **GIVEN** a policy that accepts GMCP on the side of the game, and a parser that asked for GMCP with `IAC DO GMCP`
- **WHEN** the parser reads `IAC WILL GMCP`
- **THEN** it sends nothing more, gives an event that GMCP is on for the game, and a second `IAC WILL GMCP` gives no reply

#### Scenario: A refusal of a request of the client
- **GIVEN** a parser that asked for GMCP with `IAC DO GMCP`
- **WHEN** the parser reads `IAC WONT GMCP`
- **THEN** it sends nothing, gives no event, and GMCP stays off

#### Scenario: An answer that the client did not ask for
- **GIVEN** a parser where GMCP is on for the game, and that asked to turn it off with `IAC DONT GMCP`
- **WHEN** the parser reads `IAC WILL GMCP`
- **THEN** it gives an `unrequested-answer` protocol error, as RFC 1143 names a WILL in the state WANTNO

### Requirement: A subnegotiation reaches the caller
The parser SHALL give each subnegotiation as an event with its option and its payload, for any option. It SHALL read `IAC IAC` in the payload as one byte 255, and the bound SHALL count that byte once. It SHALL drop a payload longer than 1 MiB and give a protocol error.

#### Scenario: A payload with an escaped byte
- **WHEN** the parser reads `IAC SB GMCP a IAC IAC b IAC SE`
- **THEN** it gives a subnegotiation of GMCP with the payload `a`, 255 and `b`

#### Scenario: A payload at the bound
- **WHEN** the parser reads a GMCP subnegotiation with a payload of exactly 1 MiB, in which the last byte is sent as `IAC IAC`
- **THEN** it gives the subnegotiation with the whole payload, and no protocol error

#### Scenario: A payload over the bound
- **WHEN** the parser reads a subnegotiation with a payload of 1 MiB and one byte, followed by `ok`
- **AND** the bytes come in one read, and again in reads of 4096 bytes
- **THEN** each time it gives a protocol error that names the option, no subnegotiation, and then the data `ok`

### Requirement: MCCP2 is inflated inside the parser
When MCCP2 is on for the game, `IAC SB MCCP2 IAC SE` SHALL start compression, and the parser SHALL inflate the bytes that follow. A stream SHALL be able to start inside a read. The bytes after the end of the stream SHALL be plain. When MCCP2 is off, or a stream is already open, that subnegotiation SHALL start nothing.

#### Scenario: A stream that starts inside a read
- **GIVEN** a parser that accepts MCCP2 on the side of the game
- **WHEN** one read holds `IAC WILL MCCP2`, `IAC SB MCCP2 IAC SE`, a zlib stream of `hello` and nothing more
- **THEN** the parser sends `IAC DO MCCP2` and gives that MCCP2 is on, the start of compression and the data `hello`

#### Scenario: Plain bytes after the stream
- **GIVEN** a parser where MCCP2 is on for the game
- **WHEN** one read holds `IAC SB MCCP2 IAC SE`, a complete zlib stream of `in` and the plain bytes `out`
- **THEN** the parser gives the start of compression, the data `in`, the end of compression and the data `out`

#### Scenario: A start inside a stream
- **GIVEN** a parser with an open compressed stream
- **WHEN** the inflated bytes hold `IAC SB MCCP2 IAC SE` and then `x`
- **THEN** it gives a subnegotiation of MCCP2 with an empty payload and then the data `x`, and no second start of compression

#### Scenario: A second stream after the end of the first
- **GIVEN** a parser where MCCP2 is on for the game
- **WHEN** it reads a start, a complete zlib stream of `a`, a second start and a complete zlib stream of `b`
- **THEN** it gives two starts and two ends of compression, and the data `a` and `b`

#### Scenario: A start while MCCP2 is off
- **GIVEN** a parser where MCCP2 is off
- **WHEN** it reads `IAC SB MCCP2 IAC SE` and then `plain`
- **THEN** it gives a subnegotiation of MCCP2 with an empty payload and then the data `plain`, and no start of compression

### Requirement: A bad compressed stream turns MCCP2 off
A compressed stream that is not valid zlib SHALL give one protocol error. The parser SHALL then send `IAC DONT MCCP2`, as the MCCP specification allows. It SHALL drop each byte until the game answers `IAC WONT MCCP2`. It SHALL read the bytes after that answer as plain telnet.

#### Scenario: A stream that is not zlib
- **GIVEN** a parser where MCCP2 is on for the game
- **WHEN** it reads `IAC SB MCCP2 IAC SE`, the bytes `00 01 02`, and in a later read `more\n`
- **THEN** it gives the start of compression and one protocol error, sends `IAC DONT MCCP2`, and gives no data

#### Scenario: Plain bytes after the answer to DONT
- **GIVEN** a parser where MCCP2 is on for the game
- **WHEN** it reads `IAC SB MCCP2 IAC SE`, the bytes `00 01 02`, the bytes `junk`, `IAC WONT MCCP2` and `plain`
- **THEN** for any split it gives the start of compression, one protocol error, that MCCP2 is off and the data `plain`

#### Scenario: A stream that breaks inside a command
- **GIVEN** a parser where MCCP2 is on for the game
- **WHEN** the inflated bytes before the break are `a IAC`, or `a IAC SB GMCP x`, and `IAC WONT MCCP2` and `plain` follow
- **THEN** for any split it gives the start of compression, the data `a`, one protocol error, that MCCP2 is off and the data `plain`

### Requirement: The split of reads changes no event
Join adjacent data events. Then the parser SHALL give the same events and send the same bytes for any split of the same input into reads. One exception holds: a compressed stream that fails after its first byte can give different events before its error. Its error and the events after it SHALL be the same for any split.

#### Scenario: Random streams in random reads
- **WHEN** a property test splits random telnet streams into random reads
- **AND** the streams hold compression that starts inside a read, streams with a bad zlib header, and malformed framing
- **THEN** the joined events and the sent bytes match those of the whole stream in one read

#### Scenario: The recorded transcripts in other reads
- **WHEN** each of the six transcripts of the telnet spike runs as recorded and in reads of 1, 2, 3, 7, 64 and 1500 bytes
- **THEN** each run gives no protocol error and the same joined events as the recorded reads

### Requirement: The telnet framer gives lines
The telnet framer SHALL end a line at LF and SHALL drop each CR, so a bare LF ends a line. It SHALL drop a NUL right after a CR, in the same read or the next one. It SHALL end each line to the game with CR LF, and SHALL send each CR in that line as CR NUL, as RFC 854 asks. It SHALL decode a line in its character set. It SHALL cut a run of bytes with no LF at the line bound. A CR or a dropped NUL SHALL NOT start a cut.

#### Scenario: Line ends
- **WHEN** the telnet framer reads `one\r\ntwo\nthree\n`
- **THEN** it gives the lines `one`, `two` and `three`

#### Scenario: A bare CR in a line to the game
- **WHEN** the session sends the line `a\rb`, or `a\r`, through the telnet framer
- **THEN** the bytes are `a\r\0b\r\n`, or `a\r\0\r\n`

#### Scenario: Lines in a compressed stream
- **GIVEN** a telnet framer where MCCP2 is on for the game
- **WHEN** it reads `IAC SB MCCP2 IAC SE` and a zlib stream of `one\ntwo\n`
- **THEN** it gives the lines `one` and `two`

#### Scenario: CR NUL across reads
- **WHEN** one read ends with `a\r` and the next read starts with the byte 0 and `b\n`
- **THEN** the telnet framer gives the line `ab`

#### Scenario: A long run with no line end
- **WHEN** the telnet framer reads the line bound plus three bytes of `a` and then LF
- **THEN** it gives one line at the bound and then the line `aaa`

#### Scenario: A line of exactly the bound
- **WHEN** the telnet framer or the line framer reads the line bound in bytes of `a` and then LF, or then CR LF
- **THEN** each time it gives one line at the bound and no empty line

### Requirement: The telnet framer uses its character set
The telnet framer SHALL have one character set, UTF-8 or ISO-8859-1. It SHALL decode lines and prompts in that set, and encode each line to the game in it. In UTF-8, each invalid sequence SHALL become a replacement character. In ISO-8859-1, each character with no byte in the set SHALL become `?`. A CHARSET answer SHALL NOT change the set.

#### Scenario: A Latin-1 line from the game
- **GIVEN** a telnet framer with the character set ISO-8859-1
- **WHEN** it reads the bytes `caf`, 233 and LF
- **THEN** it gives the line `café`

#### Scenario: A UTF-8 line from the game
- **GIVEN** a telnet framer with the character set UTF-8
- **WHEN** it reads the bytes `caf`, 195, 169, LF, then the byte 233 and LF
- **THEN** it gives the line `café` and then a line with one replacement character

#### Scenario: Every Latin-1 byte
- **WHEN** a test encodes each of the 256 characters below 256 as Latin-1 lines, and the framer decodes the bytes again
- **THEN** each character other than CR and LF comes back unchanged, 128 to 159 included

#### Scenario: A Latin-1 line to the game
- **GIVEN** a telnet framer with the character set ISO-8859-1
- **WHEN** it encodes the line `café ÿ ✓`
- **THEN** the bytes to the game are `caf`, 233, a space, `IAC IAC`, a space, `?`, CR and LF

### Requirement: The telnet framer gives prompts
A read can add a data byte to an unfinished line. At the end of such a read, the telnet framer SHALL give the unfinished text as a prompt, if that text is not empty. GA and EOR SHALL also end a non-empty prompt, and SHALL clear the text. If a later read ends the text with LF, the framer SHALL give the whole text as one line. The lines SHALL be the same for any split of the bytes into reads.

#### Scenario: A prompt with no GA
- **WHEN** one read ends with `Name: ` and no line end
- **THEN** the telnet framer gives the prompt `Name: `

#### Scenario: A prompt with GA
- **WHEN** one read holds `HP 10> IAC GA more`
- **THEN** the telnet framer gives the prompt `HP 10> ` and then, at the end of the read, the prompt `more`

#### Scenario: A prompt with EOR
- **WHEN** one read holds `HP 10> IAC EOR`
- **THEN** the telnet framer gives the prompt `HP 10> ` once

#### Scenario: A CR or a NUL alone
- **WHEN** one read ends with `Name: `, the next read holds only CR, and the read after it only NUL
- **THEN** the telnet framer gives the prompt `Name: ` once

#### Scenario: A line split across reads
- **WHEN** one read ends with `hel` and the next read starts with `lo\n`
- **THEN** the telnet framer gives the prompt `hel` and then the line `hello`

#### Scenario: A read that ends at a line end
- **WHEN** one read holds `hel`, and the next read holds `lo\n`
- **THEN** the telnet framer gives the prompt `hel` and the line `hello`, and no empty prompt

#### Scenario: GA after a line end
- **WHEN** one read holds `ok\n` and `IAC GA`
- **THEN** the telnet framer gives the line `ok` and no prompt

#### Scenario: Negotiation alone
- **WHEN** one read ends with `Name: `, and the next read holds only `IAC WILL GMCP`
- **THEN** the telnet framer gives the prompt `Name: ` once

#### Scenario: The same prompt after a line
- **WHEN** one read holds `> `, the next read holds `ok\n> `
- **THEN** the telnet framer gives the prompt `> `, the line `> ok` and the prompt `> `

#### Scenario: Lines in any split
- **WHEN** a property test splits random streams into random reads
- **THEN** the telnet framer gives the same lines, in the same order, as for the whole stream in one read

### Requirement: The telnet framer reads the mark apart from the options
The telnet framer SHALL send `IAC DO TIMING-MARK` as its mark each time, whatever the state of the options. It SHALL give `IAC WILL TIMING-MARK` and `IAC WONT TIMING-MARK` as the end of a mark. A mark SHALL NOT change the state of any option. The parser SHALL refuse `IAC DO TIMING-MARK` from the game each time, whatever its policy.

#### Scenario: Two marks in a row
- **WHEN** the session asks the telnet framer for a mark twice
- **THEN** both times the bytes are `IAC DO TIMING-MARK`

#### Scenario: The answer to a mark
- **WHEN** the telnet framer reads `one\n`, `IAC WILL TIMING-MARK` or `IAC WONT TIMING-MARK`, and `two\n`
- **THEN** it gives the line `one`, the end of a mark and the line `two`, and no reply

#### Scenario: A mark from the game
- **WHEN** the parser reads `IAC DO TIMING-MARK` two times
- **THEN** it sends `IAC WONT TIMING-MARK` two times, and TIMING-MARK stays off on both sides

#### Scenario: A mark from the game with a policy that accepts it
- **GIVEN** a policy that accepts TIMING-MARK on the side of the client
- **WHEN** the parser reads `IAC DO TIMING-MARK` two times, then `IAC DONT TIMING-MARK`
- **THEN** it sends `IAC WONT TIMING-MARK` two times and nothing for `DONT`

### Requirement: GMCP from the game is a message
The telnet framer SHALL give each GMCP subnegotiation from the game as a message with the name `GMCP` and the payload as its body.

#### Scenario: A GMCP message
- **GIVEN** a telnet framer where GMCP is on for the game
- **WHEN** it reads `IAC SB GMCP Logged.In IAC SE`
- **THEN** it gives a message with the name `GMCP` and the body `Logged.In`

### Requirement: The telnet framer answers the terminal type
While TTYPE is on for the client at the request, the telnet framer SHALL answer each request of the game for the terminal type. Each reply of the framer SHALL reach the game in the order of the events of the read. It SHALL answer `MU-PORTAL`, then `XTERM-256COLOR`, then `MTTS 269`. It SHALL answer `MTTS 269` to each later request.

#### Scenario: Four requests
- **GIVEN** a telnet framer where TTYPE is on for the client
- **WHEN** the game sends `IAC SB TTYPE SEND IAC SE` four times
- **THEN** the framer replies `IS MU-PORTAL`, `IS XTERM-256COLOR`, `IS MTTS 269` and `IS MTTS 269`, in that order

#### Scenario: A request while TTYPE is off
- **WHEN** the game sends `IAC SB TTYPE SEND IAC SE` before `IAC DO TTYPE`, alone or in the same read
- **THEN** the framer sends no `IS` answer

#### Scenario: An answer in the order of the read
- **WHEN** one read holds `IAC DO TTYPE`, `IAC SB TTYPE SEND IAC SE` and `IAC DONT TTYPE`
- **THEN** the framer sends `IAC WILL TTYPE`, `IS MU-PORTAL` and `IAC WONT TTYPE`, in that order

### Requirement: The telnet framer sends the window size
When NAWS turns on for the client, the telnet framer SHALL send its window size. The size is two 16-bit numbers in network order, with each byte 255 doubled.

#### Scenario: NAWS turns on
- **GIVEN** a telnet framer with a window of 120 columns and 40 rows
- **WHEN** it reads `IAC DO NAWS`
- **THEN** it replies `IAC WILL NAWS` and `IAC SB NAWS 0 120 0 40 IAC SE`

#### Scenario: A size with the byte 255
- **GIVEN** a telnet framer with a window of 255 columns and 40 rows
- **WHEN** it reads `IAC DO NAWS`
- **THEN** it replies `IAC WILL NAWS` and `IAC SB NAWS 0 IAC IAC 0 40 IAC SE`

### Requirement: The telnet framer answers a CHARSET request
If a request offers the character set of the telnet framer, the framer SHALL accept it, with no regard to case. Otherwise it SHALL reject the request. It SHALL read an optional `[TTABLE ]` prefix and its version byte before the separator. No payload SHALL make the framer fail.

#### Scenario: The character set is offered
- **GIVEN** a telnet framer with the character set `UTF-8`
- **WHEN** it reads a CHARSET request with the list `;ISO-8859-1;utf-8`
- **THEN** it replies `ACCEPTED utf-8`

#### Scenario: A request with the TTABLE prefix
- **GIVEN** a telnet framer with the character set `UTF-8`
- **WHEN** it reads a CHARSET request that starts with `[TTABLE ]`, the version byte 1 and the list `;UTF-8`
- **THEN** it replies `ACCEPTED UTF-8`

#### Scenario: The character set is not offered
- **GIVEN** a telnet framer with the character set `UTF-8`
- **WHEN** it reads a CHARSET request with the list `;ISO-8859-1`
- **THEN** it replies `REJECTED`

#### Scenario: A TTABLE prefix with no version
- **GIVEN** a telnet framer with the character set `UTF-8`
- **WHEN** it reads a CHARSET request that holds only `[TTABLE ]`
- **THEN** it replies `REJECTED`

#### Scenario: An empty list
- **GIVEN** a telnet framer with the character set `UTF-8`
- **WHEN** it reads a CHARSET request with no list, or with `[TTABLE ]`, the version byte 1 and no list
- **THEN** it replies `REJECTED`

#### Scenario: Any payload
- **WHEN** a property test gives the telnet framer CHARSET, TTYPE and GMCP subnegotiations with random payloads
- **THEN** the framer returns from each read

### Requirement: The telnet framer reports a protocol error
The telnet framer SHALL give each protocol error of the parser as a message with the name `telnet`. The body SHALL start with a stable code of the error, then a space and a text.

#### Scenario: A subnegotiation over the bound
- **WHEN** the telnet framer reads a GMCP subnegotiation longer than 1 MiB
- **THEN** it gives a message with the name `telnet` whose body starts with `subnegotiation-too-long` and names the option, and no GMCP message

#### Scenario: A bad compressed stream
- **GIVEN** a telnet framer where MCCP2 is on for the game
- **WHEN** it reads `IAC SB MCCP2 IAC SE`, the bytes `00 01 02`, and in a later read `more\n`
- **THEN** it gives one message with the name `telnet` whose body starts with `bad-compressed-stream`, and no line or prompt
