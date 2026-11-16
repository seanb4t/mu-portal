## ADDED Requirements

### Requirement: A control frame comes after the work before it
The engine SHALL have a control frame. A frame SHALL go on the queue of one world. It SHALL NOT be a command, and the journal SHALL NOT record it. The session SHALL answer a frame only after it handles each request before the frame. During a connect, the session SHALL answer it only after the connect ends. If the world is up, the session SHALL send `IAC DO TIMING-MARK` to the game. It SHALL then answer the frame only after the framer reads the answer of the game.

#### Scenario: A wait covers the echo of the game
- **GIVEN** an engine with the fake world `alpha/aria`, connected
- **WHEN** a test sends `one` to `alpha/aria` and then waits for the engine
- **THEN** the journal already holds a game event with the text `You say, "one"`, with no wait for that event

#### Scenario: A wait on a world that is down
- **GIVEN** an engine with the fake world `alpha/aria`, never connected
- **WHEN** a test waits for the engine
- **THEN** the wait returns, and the journal holds no event

#### Scenario: A wait outlasts a connect
- **GIVEN** an engine with the fake world `alpha/aria`, whose connect is held
- **WHEN** a test connects the world, starts a wait, and then lets the connect go on
- **THEN** after the wait returns, the journal already holds a game event with the text `Welcome, aria.`

#### Scenario: The line framer reads the answer to a mark
- **WHEN** the line framer reads `IAC WILL TIMING-MARK` or `IAC WONT TIMING-MARK` between two lines
- **THEN** it gives the first line, the end of a mark, and the second line, in that order
- **AND** neither line holds a byte of the mark

### Requirement: A frame that waits gets an answer when the world goes down
When the world goes down, or its connect fails, the session SHALL answer each frame that waits. An answer of the game that no frame waits for SHALL change nothing.

#### Scenario: A wait survives a drop
- **GIVEN** an engine with the fake world `alpha/aria`, connected
- **WHEN** a test sends `@drop` to `alpha/aria` and then waits for the engine
- **THEN** the wait returns, and the journal already holds a disconnect event of `alpha/aria`

#### Scenario: A wait survives a failed connect
- **GIVEN** an engine with the fake world `alpha/aria`, whose connect is held
- **WHEN** a test connects the world, starts a wait, and then drops the gate of the connect
- **THEN** the wait returns, and the journal already holds a disconnect event of `alpha/aria` that names the failed connect

#### Scenario: A mark that no frame waits for
- **GIVEN** a world whose framer gives the end of a mark for the game line `mark`
- **WHEN** the game sends `mark` and then `after`, with no wait
- **THEN** the journal holds a game event with the text `after`, and the world is still connected

### Requirement: The engine can wait until it has no work left
The task of the server handlers SHALL publish the sequence number of the last event that it handled. Before it handles an event, it SHALL publish the last sequence number of the journal at the start of the engine. A wait of the engine SHALL send rounds. Each round SHALL wait until the server handlers handle the last event of the journal, and SHALL then send one frame to each world. The wait SHALL return after the first round that adds no event to the journal.

#### Scenario: A wait covers a loop of server handlers
- **GIVEN** a server handler that returns one custom event for each custom event it sees
- **WHEN** a test emits one custom event with the text `ping` on `alpha/aria` and then waits for the engine
- **THEN** the journal already holds exactly 8 custom events from the handler after the first one, and one script error

#### Scenario: A wait adds nothing
- **GIVEN** an engine with the fake world `alpha/aria`, connected
- **WHEN** a test waits for the engine
- **THEN** the last sequence number of the journal is the same as before the wait
- **AND** the game got no line from the wait

## MODIFIED Requirements

### Requirement: The fake world stands in for a game
The hidden flag `--fake-world <id>=env:<NAME>` of `mu server` SHALL add a fake world. The flag can repeat. The fake world SHALL take the password from the environment variable `<NAME>` and the login name from the character part of the id. It SHALL greet, accept the login line `connect <login> <password>`, and answer each later line `<text>` with `You say, "<text>"`. The line `@drop` SHALL make the fake world close the connection from its side. The fake world SHALL answer `IAC DO TIMING-MARK` with `IAC WILL TIMING-MARK`, after it answers every line that came before the mark, before and after the login. A mark SHALL NOT count as a line that the game got. `mu server --help` SHALL NOT list the flag.

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
