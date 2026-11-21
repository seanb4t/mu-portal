# Spec Delta

## MODIFIED Requirements

### Requirement: The fake world stands in for a game
The hidden flag `--fake-world <id>=env:<NAME>` of `mu server` SHALL add a fake world. The flag can repeat. The fake world SHALL take the password from the environment variable `<NAME>` and the login name from the character part of the id. It SHALL greet, accept the login line `connect <login> <password>`, and answer each later line `<text>` with `You say, "<text>"`. The line `@drop` SHALL make the fake world close the connection from its side. The fake world SHALL answer `IAC DO TIMING-MARK` with `IAC WILL TIMING-MARK`, after it answers every line that came before the mark, before and after the login. A mark SHALL NOT count as a line that the game got. A fake world SHALL send its login as soon as it connects. It SHALL have no keepalive, SHALL NOT reconnect, and SHALL have `line_break = "%r"`. `mu server --help` SHALL NOT list the flag.

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

#### Scenario: The fake world gets a joined line
- **GIVEN** a server with `--fake-world alpha/aria=env:ALPHA_PW`
- **WHEN** a client sends `one` followed by LF and `two` to `alpha/aria`
- **THEN** the stream holds a game event with the text `You say, "one%rtwo"`

### Requirement: A command reaches only its world
The API call `Send` SHALL send one line to the world it names, and no other world. The reply SHALL hold the sequence number of the player event. If a hook gagged the line, the reply SHALL hold no number. An unknown world SHALL fail with `not_found`. A world in the state `connecting`, `reconnecting` or `disconnected` SHALL fail with `failed_precondition`. A text with CR SHALL fail with `invalid_argument`. If the world has a `line_break`, the server SHALL replace each LF of the text with it. The replace SHALL come before the hooks, the journal and the write to the game. A text with LF to a world with no `line_break` SHALL fail with `invalid_argument`. A call that the server refuses SHALL record no event. If the write of the line to the game fails, the call SHALL fail with `failed_precondition`. The server SHALL then record the disconnect of the world and no player event.

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
- **GIVEN** the world `alpha/aria` has no `line_break`
- **WHEN** a client sends `one` followed by LF and `two` to `alpha/aria`
- **THEN** the call fails with the code `invalid_argument`
- **AND** the server records no event
- **AND** the fake world gets no line

#### Scenario: A line break that the world joins
- **GIVEN** the world `alpha/aria` has `line_break = "|/"`, and is connected
- **WHEN** a client sends `one`, LF, `two`, LF, LF and `three` to `alpha/aria`
- **THEN** the game gets the one line `one|/two|/|/three`
- **AND** the stream holds a player event with the text `one|/two|/|/three`

#### Scenario: A carriage return to a world that joins
- **GIVEN** the world `alpha/aria` has `line_break = "%r"`
- **WHEN** a client sends `one` followed by CR, LF and `two` to `alpha/aria`
- **THEN** the call fails with the code `invalid_argument`
- **AND** the server records no event

#### Scenario: A hook sees the joined line
- **GIVEN** the world `alpha/aria` has `line_break = "%r"`, and a hook that records each player line it gets
- **WHEN** a client sends `one` followed by LF and `two` to `alpha/aria`
- **THEN** the hook gets `one%rtwo`
