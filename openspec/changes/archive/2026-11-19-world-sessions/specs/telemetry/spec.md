# Spec Delta

## MODIFIED Requirements

### Requirement: A span for each command of a world
The server SHALL make one span named `command` for each command that a world handles. It SHALL carry `mu.world`, `mu.command`, `mu.origin` and `mu.depth`. The values of `mu.command` SHALL be `send`, `connect`, `disconnect`, `login`, `echo` and `emit`. The values of `mu.origin` SHALL be `player`, `script` and `system`. The span of a command from the API SHALL be a child of the span of that call. The span of a command that fails SHALL have the status `error`.

#### Scenario: A command under its call
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** a client calls `Send` for `alpha/aria`, and the server gets SIGTERM
- **THEN** the collector got a span named `command` with `mu.world` equal to `alpha/aria`, `mu.command` equal to `send` and `mu.origin` equal to `player`
- **AND** its parent is the span named `mu.v1.PortalService/Send`

#### Scenario: A command of the server
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** the server connects the world at the start, and gets SIGTERM
- **THEN** the collector got a span named `command` with `mu.command` equal to `connect` and `mu.origin` equal to `system`

#### Scenario: A command that fails
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector, and `alpha/aria` is disconnected
- **WHEN** a client calls `Send` for `alpha/aria`, and the server gets SIGTERM
- **THEN** the collector got a span named `command` with `mu.command` equal to `send` and the status `error`

### Requirement: A span for each connection of a world
The server SHALL make one span named `world.connection` for each connection and for each attempt of a world. It SHALL cover the time from the open of the transport to the disconnect of the world. It SHALL carry `mu.world`, and at its end `mu.connection.end` with one of `connect_failed`, `timeout`, `lost`, `closed`, `player` and `stopped`. A span that ends with `connect_failed`, `timeout` or `lost` SHALL have the status `error`.

#### Scenario: The connection of a fake world
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** the world connects at the start, and the server gets SIGTERM
- **THEN** the collector got one span named `world.connection` with `mu.world` equal to `alpha/aria` and `mu.connection.end` equal to `stopped`

#### Scenario: A connect that fails
- **GIVEN** a server that exports to a fake collector, and a configured world `alpha/aria` with `reconnect.mode = "off"` whose port has no listener
- **WHEN** a client calls `Connect` for `alpha/aria`, and the server gets SIGTERM
- **THEN** the collector got a span named `world.connection` with `mu.connection.end` equal to `connect_failed` and the status `error`

#### Scenario: The game closes the connection
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** a client sends `@drop` to `alpha/aria`, and the server gets SIGTERM
- **THEN** the collector got a span named `world.connection` with `mu.connection.end` equal to `closed`, and its status is not `error`

#### Scenario: A connection that is lost
- **GIVEN** a world on a test transport whose writes fail, connected, with a fake collector
- **WHEN** a client sends `hello` to the world
- **THEN** the collector got a span named `world.connection` with `mu.connection.end` equal to `lost` and the status `error`

### Requirement: Metrics of the worlds
The server SHALL count each dropped command in the sum `mu.commands.dropped`. Its attributes SHALL be `mu.world` and `mu.drop.reason`, `depth` or `rate`. It SHALL count each script error in the sum `mu.script.errors`, with `mu.world`. It SHALL count each keepalive line in the sum `mu.keepalive.sent`, with `mu.world`. The gauge `mu.world.state` SHALL carry `mu.world` and `mu.world.state.name`, one of `connecting`, `connected`, `reconnecting` and `disconnected`. Its value SHALL be 1 for the current state of the world, and 0 for each state that it left. A world SHALL report `disconnected` from the start of the server.

#### Scenario: A command past the depth limit
- **GIVEN** a world with the depth limit of 8, and a handler that answers each event of the world with one `Echo`
- **WHEN** a client sends one line, so that the chain of `Echo` commands passes the depth 8
- **THEN** the server records 1 in `mu.commands.dropped` with `mu.drop.reason` equal to `depth`
- **AND** it records 1 in `mu.script.errors` for that world

#### Scenario: A command over the rate
- **GIVEN** a world with a paused clock and the rate limit of 20 commands in each second
- **AND** a handler that sends 25 commands for one event
- **WHEN** the engine handles the 25 commands
- **THEN** the server records 5 in `mu.commands.dropped` for that world, with `mu.drop.reason` equal to `rate`

#### Scenario: Commands at the rate
- **GIVEN** a world with a paused clock and the rate limit of 20 commands in each second
- **AND** a handler that sends 20 commands for one event
- **WHEN** the engine handles the 20 commands
- **THEN** the server records no value of `mu.commands.dropped` for that world

#### Scenario: A world that never connects
- **GIVEN** a server that exports to a fake collector, and a configuration that holds the world `alpha/aria`
- **WHEN** the server starts and gets SIGTERM
- **THEN** the last export of `mu.world.state` for `alpha/aria` has the value 1 for `disconnected`

#### Scenario: The state at the stop
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** the world connects at the start, and the server gets SIGTERM
- **THEN** the last export of `mu.world.state` for `alpha/aria` has the value 1 for `disconnected` and 0 for `connected`

#### Scenario: The keepalive lines
- **GIVEN** a world on a paused clock with `keepalive.mode = "interval"` and `keepalive.seconds = 60`, connected at the time 0
- **WHEN** the clock reaches 130
- **THEN** the server records 2 in `mu.keepalive.sent` for that world

#### Scenario: A world that waits to reconnect
- **GIVEN** a world that was connected and is now in the state `reconnecting`
- **WHEN** a test reads the gauge `mu.world.state` of the world
- **THEN** it has the value 1 for `reconnecting` and 0 for `connected`
