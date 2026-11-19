# world-sessions Specification

## Purpose
Defines how the server keeps several worlds connected at once and how a command reaches one world. It also defines the fake world that stands in for a game.

## Requirements

### Requirement: A world id names one character on one game
Each world SHALL have an id of the form `<game>/<character>`. Each id SHALL have exactly one slash. Each part SHALL hold only the ASCII letters `a` to `z`, the digits and hyphens, and SHALL start with a letter. Each command to a world SHALL name its id, and each event SHALL carry one.

#### Scenario: A bad world id is refused
- **WHEN** the user runs `mu server --socket <dir>/s.sock --fake-world Alpha_1=env:PW`
- **THEN** stderr contains `Alpha_1`
- **AND** the exit status is 2

#### Scenario: Ids outside the rule
- **WHEN** the engine parses `-alpha/aria`, `alpha`, `alpha/b/c`, `alpha/`, `1alpha/aria`, `alpha/arïa`, `Alpha/aria` and `alpha/a_b`
- **THEN** each parse fails

#### Scenario: Ids inside the rule
- **WHEN** the engine parses `alpha/aria`, `alpha-2/bea` and `a/b-9`
- **THEN** each parse succeeds, and the display of each id is the text it came from

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

### Requirement: A world connects and disconnects alone
The API calls `Connect` and `Disconnect` SHALL change the state of the world they name only. A world that disconnects SHALL leave every other world in its state.

#### Scenario: One world drops
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, both connected
- **WHEN** a client calls `Disconnect` for `alpha/aria`
- **THEN** the stream holds a disconnect event for `alpha/aria`
- **AND** a client that sends `still here` to `beta/bryn` gets a game event with the text `You say, "still here"`

#### Scenario: A world connects again
- **GIVEN** the world `alpha/aria` is in the state `disconnected`
- **WHEN** a client calls `Connect` for `alpha/aria`
- **THEN** the stream holds a connect event and a login event for `alpha/aria`
- **AND** `ListWorlds` returns `alpha/aria` in the state `connected`

### Requirement: A hook can change a line before the journal
The engine SHALL run each hook of a world on each line from the game and from the player. The hooks run before the event reaches the journal. A hook SHALL pass the line, replace its text, or gag it. A gagged line SHALL make no event. A gagged player line SHALL not reach the game. A hook can replace a player line with a text that holds CR or LF. Then the line SHALL not reach the game and SHALL make no player event, and the engine SHALL record a script error.

#### Scenario: A hook gags a game line
- **GIVEN** a world with a test hook that gags each game line that contains `spam`
- **WHEN** the game sends `buy spam now` and then `hello`
- **THEN** the journal holds a game event for `hello`
- **AND** no event contains `spam`

#### Scenario: A hook replaces a player line
- **GIVEN** a world with a test hook that replaces the player line `hi` with `say hi`
- **WHEN** a client sends `hi`
- **THEN** the game gets the line `say hi`
- **AND** the journal holds a player event with the text `say hi`

#### Scenario: A hook puts a line break in a player line
- **GIVEN** a world with a test hook that replaces the player line `hi` with a text that holds LF
- **WHEN** a client sends `hi`
- **THEN** the reply holds no sequence number
- **AND** the game gets no line
- **AND** the journal holds one script error and no player event

#### Scenario: A hook gags a player line
- **GIVEN** a world with a test hook that gags the player line `secret plan`
- **WHEN** a client sends `secret plan`
- **THEN** the reply holds no sequence number
- **AND** the game gets no line
- **AND** no event contains `secret plan`

### Requirement: A handler acts through commands
The engine SHALL run each handler of a world on each event of that world after the journal records it. A server handler SHALL run on the events of every world. A handler SHALL return commands. The engine SHALL run each command and SHALL record its result as a new event. A command from a handler SHALL NOT run the same handler again for its own event in the same step. No handler SHALL run on an event of the kind script error.

#### Scenario: A handler answers a line
- **GIVEN** a world with a test handler that returns a send of `wave` for each game event with the text `Hello?`
- **WHEN** the game sends `Hello?`
- **THEN** the game gets the line `wave`
- **AND** the journal holds an event with the text `wave` and the source script

#### Scenario: A loop of handlers stops
- **GIVEN** a server handler that returns one custom event for each custom event it sees
- **WHEN** a test emits one custom event with the text `ping` on `alpha/aria`
- **THEN** the journal holds exactly 8 custom events from the handler after the first one
- **AND** the journal holds one event of the kind script error that names the depth limit

#### Scenario: A handler that answers every event
- **GIVEN** a server handler that returns one custom event for each event of any kind
- **WHEN** a test emits one custom event with the text `ping` on `alpha/aria`
- **THEN** the journal holds exactly one event of the kind script error
- **AND** the journal records no event after that script error

### Requirement: A loop through the game has a bound
A line from the game SHALL start at the depth 0. Each world SHALL accept commands from handlers at a rate of at most 20 in one second, with a burst of at most 20. The engine SHALL drop each command over the limit. It SHALL record at most one script error for each second in which it dropped a command. After a drop, the world SHALL accept commands from handlers again as the rate allows.

#### Scenario: A handler that answers its own echo
- **GIVEN** a world with a test handler that sends `again` for each game event, and a fake world that echoes each line
- **WHEN** a client sends `start`
- **THEN** the game gets exactly 20 lines `again` in the first second
- **AND** the journal holds exactly one script error for that second

#### Scenario: The limit opens again
- **GIVEN** the loop of `again` stopped at the limit in the first second
- **WHEN** one second passes, and a client sends `start` again
- **THEN** the game gets the line `again` again

### Requirement: The events of handler commands name their source
An input from a handler SHALL have the source script. A connect or a disconnect from a handler SHALL have the source system.

#### Scenario: A handler connects
- **GIVEN** a world with a test handler that returns a connect for each disconnect of its world
- **WHEN** a client sends `@drop` to the world
- **THEN** the journal holds a connect event with the source system after the disconnect

### Requirement: A control frame comes after the work before it
The engine SHALL have a control frame. A frame SHALL go on the queue of one world. It SHALL NOT be a command, and the journal SHALL NOT record it. The session SHALL answer a frame only after it handles each request before the frame. During a connect, the session SHALL answer it only after the connect ends. If the world is up, the session SHALL send the mark of the framer and answer after the framer reads the answer. A framer with no mark gets an answer at once.

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
- **THEN** the wait returns, and the journal already holds a disconnect event of `alpha/aria` with the text `the connect failed: the connect gate closed`

#### Scenario: A mark that no frame waits for
- **GIVEN** a world whose framer gives the end of a mark for the game line `mark`
- **WHEN** the game sends `mark` and then `after`, with no wait
- **THEN** the journal holds a game event with the text `after`, and the world is still connected

### Requirement: The engine can wait until it has no work left
The server handlers SHALL publish the number of the last event they handled, from the last number of the journal at the start. A wait of the engine SHALL send rounds. Each round SHALL wait until the server handlers handle the last event of the journal, and SHALL then send one frame to each world. The wait SHALL return after the first round that adds no event to the journal. If a world ends a frame with no answer, the wait SHALL fail.

#### Scenario: A wait covers a loop of server handlers
- **GIVEN** a server handler that returns one custom event for each custom event it sees
- **WHEN** a test emits one custom event with the text `ping` on `alpha/aria` and then waits for the engine
- **THEN** the journal already holds exactly 8 custom events from the handler after the first one, and one script error

#### Scenario: A wait adds nothing
- **GIVEN** an engine with the fake world `alpha/aria`, connected
- **WHEN** a test waits for the engine
- **THEN** the last sequence number of the journal is the same as before the wait
- **AND** the game got no line from the wait

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
