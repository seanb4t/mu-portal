# Spec Delta

## MODIFIED Requirements

### Requirement: The fake world stands in for a game
The hidden flag `--fake-world <id>=env:<NAME>` of `mu server` SHALL add a fake world. The flag can repeat. The fake world SHALL take the password from the environment variable `<NAME>` and the login name from the character part of the id. It SHALL greet, accept the login line `connect <login> <password>`, and answer each later line `<text>` with `You say, "<text>"`. The line `@drop` SHALL make the fake world close the connection from its side. The fake world SHALL answer `IAC DO TIMING-MARK` with `IAC WILL TIMING-MARK`, after it answers every line that came before the mark, before and after the login. A mark SHALL NOT count as a line that the game got. A fake world SHALL send its login as soon as it connects. It SHALL have no keepalive and SHALL NOT reconnect. `mu server --help` SHALL NOT list the flag.

#### Scenario: The fake world answers
- **GIVEN** a server with `--fake-world alpha/aria=env:ALPHA_PW`
- **WHEN** a client sends `hello` to `alpha/aria`
- **THEN** the stream holds a game event with the text `Welcome, aria.`
- **AND** the stream then holds a game event with the text `You say, "hello"`

#### Scenario: The fake world answers a mark in order
- **GIVEN** a connection to the fake game, logged in
- **WHEN** the connection sends the line `one` and then `IAC DO TIMING-MARK`
- **THEN** the connection reads `You say, "one"` and then `IAC WILL TIMING-MARK`
- **AND** the lines that the game got are the login and `one`

#### Scenario: The flag is hidden
- **WHEN** the user runs `mu server --help`
- **THEN** stdout does not contain `fake-world`

#### Scenario: A missing password variable
- **GIVEN** the environment variable `ALPHA_PW` is not set
- **WHEN** the user runs `mu server --socket <dir>/s.sock --fake-world alpha/aria=env:ALPHA_PW`
- **THEN** stderr contains `ALPHA_PW`
- **AND** the exit status is 1

#### Scenario: A fake world does not reconnect
- **GIVEN** a server with the fake world `alpha/aria`, connected
- **WHEN** a client sends `@drop` to `alpha/aria`
- **THEN** `ListWorlds` returns `alpha/aria` in the state `disconnected`
- **AND** the fake game gets no new connection

### Requirement: A command reaches only its world
The API call `Send` SHALL send one line to the world it names, and no other world. The reply SHALL hold the sequence number of the player event. If a hook gagged the line, the reply SHALL hold no number. An unknown world SHALL fail with `not_found`. A world in the state `connecting`, `reconnecting` or `disconnected` SHALL fail with `failed_precondition`. A text with CR or LF SHALL fail with `invalid_argument`. A call that the server refuses SHALL record no event. If the write of the line to the game fails, the call SHALL fail with `failed_precondition`. The server SHALL then record the disconnect of the world and no player event.

#### Scenario: Two characters on one game
- **GIVEN** a server with the fake worlds `alpha/aria` and `alpha/ash`
- **WHEN** a client sends `one` to `alpha/aria`
- **THEN** the stream holds `You say, "one"` for `alpha/aria`
- **AND** no event of `alpha/ash` contains `one`

#### Scenario: An unknown world
- **WHEN** a client sends `hello` to `nowhere/nobody`
- **THEN** the call fails with the code `not_found`
- **AND** the server records no event

#### Scenario: A world that is not connected
- **GIVEN** the world `alpha/aria` is in the state `disconnected`
- **WHEN** a client sends `hello` to `alpha/aria`
- **THEN** the call fails with the code `failed_precondition`
- **AND** the server records no event

#### Scenario: A write that fails
- **GIVEN** the world `alpha/aria` is connected on a test transport whose writes fail
- **WHEN** a client sends `hello` to `alpha/aria`
- **THEN** the call fails with the code `failed_precondition`
- **AND** the server records one disconnected event for `alpha/aria`
- **AND** the server records no player event

#### Scenario: A world that is still connecting
- **GIVEN** the world `alpha/aria` is in the state `connecting`
- **WHEN** a client sends `hello` to `alpha/aria`
- **THEN** the call fails with the code `failed_precondition`
- **AND** the server records no event

#### Scenario: A world that waits to reconnect
- **GIVEN** the world `alpha/aria` is in the state `reconnecting`
- **WHEN** a client sends `hello` to `alpha/aria`
- **THEN** the call fails with the code `failed_precondition`
- **AND** the server records no event

#### Scenario: An empty line
- **GIVEN** the world `alpha/aria` is connected
- **WHEN** a client sends an empty text to `alpha/aria`
- **THEN** the fake world gets an empty line
- **AND** the stream holds a player event with an empty text for `alpha/aria`

#### Scenario: A carriage return in the text
- **WHEN** a client sends `one` followed by CR and `two` to `alpha/aria`
- **THEN** the call fails with the code `invalid_argument`
- **AND** the server records no event

#### Scenario: A line break in the text
- **WHEN** a client sends `one` followed by LF and `two` to `alpha/aria`
- **THEN** the call fails with the code `invalid_argument`
- **AND** the server records no event
- **AND** the fake world gets no line

## ADDED Requirements

### Requirement: Worlds connect at the start or on request
At the start of the server, each fake world and each configured world with `autoconnect = true` SHALL connect. Each other configured world SHALL start in the state `disconnected`, and `Connect` SHALL connect it. A world with a login SHALL wait for its login moment before it sends the login.

#### Scenario: Two worlds connect
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** a client calls `ListWorlds` after both connect events
- **THEN** the reply holds `alpha/aria` and `beta/bryn`, each in the state `connected`

#### Scenario: A configured world waits
- **GIVEN** a server with the fake world `beta/bryn` and a configuration that holds the world `alpha/aria`, with no `autoconnect`
- **WHEN** a client calls `ListWorlds` after the connect event of `beta/bryn`
- **THEN** the reply holds `alpha/aria` in the state `disconnected` and `beta/bryn` in the state `connected`

#### Scenario: A configured world that connects at the start
- **GIVEN** a fake game on TCP, and a configuration that holds the world `alpha/aria` with its host and port and `autoconnect = true`
- **WHEN** the server starts
- **THEN** the stream holds a connect event for `alpha/aria` with no call of `Connect`

#### Scenario: A configured world connects on request
- **GIVEN** a fake game on TCP, and a configuration that holds the world `alpha/aria` with its host and port
- **WHEN** a client calls `Connect` for `alpha/aria`
- **THEN** the call succeeds, and the stream holds a connect event for `alpha/aria`

#### Scenario: A failed connect at the start
- **GIVEN** a server with the fake world `beta/bryn`, and a configuration that holds the world `alpha/aria` with `tls = true` and `autoconnect = true`
- **WHEN** a client calls `ListWorlds` after the connect event of `beta/bryn`
- **THEN** the server started
- **AND** the reply holds `alpha/aria` in the state `disconnected` and `beta/bryn` in the state `connected`

### Requirement: ListWorlds gives the state of each world
The API call `ListWorlds` SHALL return each world with its id and its state, one of `connecting`, `connected`, `reconnecting` and `disconnected`. A world in the state `reconnecting` SHALL also carry the time of its next attempt. **BREAKING**: the state `reconnecting` is new, so a client that refuses an unknown state fails on it.

#### Scenario: A world that waits to reconnect
- **GIVEN** a configured world `alpha/aria` in the state `reconnecting`
- **WHEN** a client calls `ListWorlds`
- **THEN** the reply holds `alpha/aria` in the state `reconnecting`, with a time of the next attempt that is after the call


### Requirement: A wait of the engine covers a pending login
The session SHALL answer a control frame only after the login of its world went out. A world that goes down SHALL also answer it. A world in the state `reconnecting` SHALL answer a frame at once, as a world that is down does.

#### Scenario: A wait outlasts the quiet before the login
- **GIVEN** an engine on a paused clock with a world that waits for a quiet second before its login
- **WHEN** a test connects the world and waits for the engine
- **THEN** after the wait returns, the journal already holds the login event of the world

#### Scenario: A wait on a world that waits to reconnect
- **GIVEN** an engine with a world in the state `reconnecting`
- **WHEN** a test waits for the engine
- **THEN** the wait returns before the next attempt of the world

## REMOVED Requirements

### Requirement: Each world connects and logs in at the start
**Reason**: A configured world can now connect, so `Connect` on it no longer fails. The world can also connect at the start, and a new state `reconnecting` exists.
**Migration**: The requirement "Worlds connect at the start or on request" replaces it. A client reads the state `reconnecting` from `ListWorlds`.
