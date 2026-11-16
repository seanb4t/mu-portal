# Spec Delta

## Purpose

Defines how the server keeps several worlds connected at once and how a command reaches one world. It also defines the fake world that stands in for a game.

## ADDED Requirements

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
The hidden flag `--fake-world <id>=env:<NAME>` of `mu server` SHALL add a fake world. The flag can repeat. The fake world SHALL take the password from the environment variable `<NAME>` and the login name from the character part of the id. It SHALL greet, accept the login line `connect <login> <password>`, and answer each later line `<text>` with `You say, "<text>"`. The line `@drop` SHALL make the fake world close the connection from its side. `mu server --help` SHALL NOT list the flag.

#### Scenario: The fake world answers
- **GIVEN** a server with `--fake-world alpha/aria=env:ALPHA_PW`
- **WHEN** a client sends `hello` to `alpha/aria`
- **THEN** the stream holds a game event with the text `Welcome, aria.`
- **AND** the stream then holds a game event with the text `You say, "hello"`

#### Scenario: The flag is hidden
- **WHEN** the user runs `mu server --help`
- **THEN** stdout does not contain `fake-world`

#### Scenario: A missing password variable
- **GIVEN** the environment variable `ALPHA_PW` is not set
- **WHEN** the user runs `mu server --socket <dir>/s.sock --fake-world alpha/aria=env:ALPHA_PW`
- **THEN** stderr contains `ALPHA_PW`
- **AND** the exit status is 1

### Requirement: Each world connects and logs in at the start
At the start of the server, each world SHALL connect. Each world with a login SHALL send it as soon as it connects. The API call `ListWorlds` SHALL return each world with its id and its state, one of `connecting`, `connected` and `disconnected`.

#### Scenario: Two worlds connect
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** a client calls `ListWorlds` after both connect events
- **THEN** the reply holds `alpha/aria` and `beta/bryn`, each in the state `connected`

### Requirement: A command reaches only its world
The API call `Send` SHALL send one line to the world it names, and no other world. The reply SHALL hold the sequence number of the player event. If a hook gagged the line, the reply SHALL hold no number. An unknown world SHALL fail with `not_found`. A world in the state `connecting` or `disconnected` SHALL fail with `failed_precondition`. A text with CR or LF SHALL fail with `invalid_argument`. A failed call SHALL record no event.

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

#### Scenario: A world that is still connecting
- **GIVEN** the world `alpha/aria` is in the state `connecting`
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
The engine SHALL run each hook of a world on each line from the game and from the player. The hooks run before the event reaches the journal. A hook SHALL pass the line, replace its text, or gag it. A gagged line SHALL make no event. A gagged player line SHALL not reach the game.

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
