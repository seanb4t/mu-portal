# Spec Delta

## Purpose

The telemetry of the server. It covers where the server sends traces and metrics over OTLP, which spans and metrics it makes, and what never goes into them.

## ADDED Requirements

### Requirement: No endpoint means no export
With no endpoint from `server.toml` and no endpoint variable, the server SHALL export nothing. It SHALL serve and stop as it does without this capability, and its stderr SHALL hold no line about telemetry.

#### Scenario: A server with no endpoint
- **GIVEN** an empty configuration directory, and no `OTEL_*` variable in the environment
- **WHEN** a server with the fake world `alpha/aria` starts, serves one `Send`, and gets SIGTERM
- **THEN** the exit status is 0
- **AND** stderr holds only the line that starts with `mu server: serving the API on`

### Requirement: The endpoint of server.toml
The table `[telemetry]` of `server.toml` SHALL accept the field `endpoint`, the base URL of an OTLP/HTTP collector. The server SHALL send traces with `POST` to the path `v1/traces` under that URL, and metrics with `POST` to the path `v1/metrics`, as protobuf. One trailing slash of the URL SHALL NOT double the slash before the path. At the stop, the server SHALL send the data that it holds before it exits.

#### Scenario: Traces and metrics reach the collector
- **GIVEN** a fake collector at `http://127.0.0.1:<port>`, and `server.toml` sets `[telemetry] endpoint = "http://127.0.0.1:<port>"`
- **WHEN** a server with the fake world `alpha/aria` starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got at least one `POST /v1/traces` and at least one `POST /v1/metrics`, each with the content type `application/x-protobuf`
- **AND** the exit status is 0

#### Scenario: A base URL with a trailing slash
- **GIVEN** a fake collector, and `server.toml` sets `[telemetry] endpoint = "http://127.0.0.1:<port>/"`
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got `POST /v1/traces` and `POST /v1/metrics`

#### Scenario: The log lines stay
- **GIVEN** a fake collector, and `server.toml` names it
- **WHEN** a server starts and gets SIGTERM
- **THEN** stderr holds the line that starts with `mu server: serving the API on`

### Requirement: Each field of server.toml is known
`server.toml` SHALL hold only the table `telemetry`, and that table SHALL hold only `endpoint`. An unknown table or field SHALL stop the start. So SHALL a value of the wrong type, an endpoint that is not a URL, and a scheme other than `http`. The server SHALL write an error that names `server.toml` and the field, and exit with status 1.

#### Scenario: An unknown field
- **GIVEN** `server.toml` sets `[telemetry] endpont = "http://127.0.0.1:4318"`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml` and `telemetry.endpont`
- **AND** the exit status is 1

#### Scenario: An unknown table
- **GIVEN** `server.toml` holds the table `[logging]`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml` and `logging`
- **AND** the exit status is 1

#### Scenario: A value of the wrong type
- **GIVEN** `server.toml` sets `[telemetry] endpoint = 4318`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml` and `telemetry.endpoint`
- **AND** the exit status is 1

#### Scenario: An endpoint that is not a URL
- **GIVEN** `server.toml` sets `[telemetry] endpoint = "not a url"`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml` and `telemetry.endpoint`
- **AND** the exit status is 1

#### Scenario: An https endpoint
- **GIVEN** `server.toml` sets `[telemetry] endpoint = "https://collector.example:4318"`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml`, `telemetry.endpoint` and `https`
- **AND** the exit status is 1

### Requirement: The standard variables
The server SHALL read `OTEL_EXPORTER_OTLP_ENDPOINT` as a base URL, the same as the field `endpoint`. A value of that variable SHALL win over `server.toml`. `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` and `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` SHALL each be the full URL of one signal. The server SHALL add no path to them, and each SHALL win over the base URL for its signal. An empty variable SHALL count as unset.

#### Scenario: The variable wins over the file
- **GIVEN** the fake collectors A and B, `server.toml` names A, and `OTEL_EXPORTER_OTLP_ENDPOINT` names B
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** B got `POST /v1/traces` and `POST /v1/metrics`
- **AND** A got no request

#### Scenario: The endpoint of the traces
- **GIVEN** a fake collector, and `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` is `http://127.0.0.1:<port>/custom/traces`
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got `POST /custom/traces`
- **AND** it got no request to `/v1/metrics`

#### Scenario: The endpoint of the metrics
- **GIVEN** a fake collector, and `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` is `http://127.0.0.1:<port>/custom/metrics`
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got `POST /custom/metrics`
- **AND** it got no request to `/v1/traces`

#### Scenario: An empty variable
- **GIVEN** a fake collector, `server.toml` names it, and `OTEL_EXPORTER_OTLP_ENDPOINT` is empty
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got `POST /v1/traces`

### Requirement: Each endpoint variable is checked
Each of the three endpoint variables SHALL pass the check of the field `endpoint`: a URL with the scheme `http` and a host. A bad value SHALL stop the start with an error that names the variable, and the exit status SHALL be 1.

#### Scenario: A bad variable
- **GIVEN** `OTEL_EXPORTER_OTLP_ENDPOINT` is `https://collector.example:4318`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `OTEL_EXPORTER_OTLP_ENDPOINT` and `https`
- **AND** the exit status is 1

#### Scenario: A bad variable of one signal
- **GIVEN** one of `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` and `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` is `https://collector.example:4318/v1/x`, or is `not a url`
- **WHEN** the user runs `mu server`
- **THEN** stderr names that variable
- **AND** the exit status is 1

### Requirement: The variables that turn the export off and name the service
`OTEL_SDK_DISABLED` with the value `true`, in any case, SHALL turn the export off, whatever the endpoint. The resource of each export SHALL carry `service.name`. If `OTEL_SERVICE_NAME` is set and not empty, its value SHALL be that variable. Otherwise, its value SHALL be `mu`.

#### Scenario: The export is off
- **GIVEN** a fake collector, `server.toml` names it, and `OTEL_SDK_DISABLED` is `TRUE`
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got no request
- **AND** the exit status is 0

#### Scenario: The default service name
- **GIVEN** a fake collector that `server.toml` names, and no `OTEL_SERVICE_NAME`
- **WHEN** a server starts and gets SIGTERM
- **THEN** each export that the collector got has the resource attribute `service.name` with the value `mu`

#### Scenario: An empty service name
- **GIVEN** a fake collector that `server.toml` names, and `OTEL_SERVICE_NAME` is empty
- **WHEN** a server starts and gets SIGTERM
- **THEN** each export that the collector got has the resource attribute `service.name` with the value `mu`

#### Scenario: A service name from the variable
- **GIVEN** a fake collector that `server.toml` names, and `OTEL_SERVICE_NAME` is `mu-desk`
- **WHEN** a server starts and gets SIGTERM
- **THEN** each export that the collector got has the resource attribute `service.name` with the value `mu-desk`

### Requirement: Only OTLP over HTTP with protobuf
The server SHALL refuse at the start an `OTEL_EXPORTER_OTLP_PROTOCOL`, `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL` or `OTEL_EXPORTER_OTLP_METRICS_PROTOCOL` whose value is not `http/protobuf` while the export is on. It SHALL write an error that names the variable, and exit with status 1.

#### Scenario: The protocol grpc
- **GIVEN** a fake collector that `server.toml` names, and `OTEL_EXPORTER_OTLP_PROTOCOL` is `grpc`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `OTEL_EXPORTER_OTLP_PROTOCOL` and `grpc`
- **AND** the exit status is 1

#### Scenario: The protocol grpc with the export off
- **GIVEN** an empty configuration directory, no endpoint variable, and `OTEL_EXPORTER_OTLP_PROTOCOL` is `grpc`
- **WHEN** a server starts and gets SIGTERM
- **THEN** the exit status is 0

### Requirement: A span for each API call
The server SHALL make one span for each call of the API, named `PortalService/<method>`, for example `PortalService/Send`. A call that names a world SHALL carry the attribute `mu.world` with the world id. A call that fails SHALL have the status `error`. The span of `Subscribe` SHALL cover the call up to the return of its stream. It SHALL NOT cover the life of the stream.

#### Scenario: The span of a Send
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** a client calls `Send` for `alpha/aria`, and the server gets SIGTERM
- **THEN** the collector got a span named `PortalService/Send` with `mu.world` equal to `alpha/aria`

#### Scenario: The span of a Subscribe
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** a client subscribes, then calls `Send` for `alpha/aria`, reads the event of that `Send` on the stream, and the server gets SIGTERM
- **THEN** the span named `PortalService/Subscribe` ends before the span named `PortalService/Send` starts

#### Scenario: A failed call
- **GIVEN** a server that exports to a fake collector
- **WHEN** a client calls `Connect` for the unknown world `zeta/zed`, and the server gets SIGTERM
- **THEN** the collector got a span named `PortalService/Connect` with the status `error`

### Requirement: A span for each command of a world
The server SHALL make one span named `command` for each command that a world handles. It SHALL carry `mu.world`, `mu.command`, `mu.origin` and `mu.depth`. The values of `mu.command` SHALL be `send`, `connect`, `disconnect`, `login`, `echo` and `emit`. The values of `mu.origin` SHALL be `player`, `script` and `system`. The span of a command from the API SHALL be a child of the span of that call.

#### Scenario: A command under its call
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** a client calls `Send` for `alpha/aria`, and the server gets SIGTERM
- **THEN** the collector got a span named `command` with `mu.world` equal to `alpha/aria`, `mu.command` equal to `send` and `mu.origin` equal to `player`
- **AND** its parent is the span named `PortalService/Send`

#### Scenario: A command of the server
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** the server connects the world at the start, and gets SIGTERM
- **THEN** the collector got a span named `command` with `mu.command` equal to `connect` and `mu.origin` equal to `system`

### Requirement: A span for each connection of a world
The server SHALL make one span named `world.connection` for each connection of a world. It SHALL cover the time from the open of the transport to the disconnect of the world. It SHALL carry `mu.world`.

#### Scenario: The connection of a fake world
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** the world connects at the start, and the server gets SIGTERM
- **THEN** the collector got one span named `world.connection` with `mu.world` equal to `alpha/aria`

### Requirement: Metrics of the journal
The server SHALL count each event of the journal in the sum `mu.journal.events`, with the attributes `mu.world` and `mu.event.kind`. For each event that the API sends, the histogram `mu.subscriber.lag` SHALL get the gap from that event to the last event of the journal. It SHALL count in the sum `mu.subscriber.lagged` each stream that a lag ended.

#### Scenario: The events of a world
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** a client calls `Send` for `alpha/aria` twice, and the server gets SIGTERM
- **THEN** the last export of `mu.journal.events` for `mu.world` `alpha/aria` and `mu.event.kind` `input` has the value 2

#### Scenario: The lag of a subscriber
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** a client subscribes, reads three events, and the server gets SIGTERM
- **THEN** the last export of `mu.subscriber.lag` has a count of at least 3

#### Scenario: A stream that a lag ended
- **GIVEN** a journal that holds four events, and a stream of the API that falls more than four events behind
- **WHEN** the stream ends with `resource_exhausted`
- **THEN** the server records 1 in `mu.subscriber.lagged`

### Requirement: Metrics of the worlds
The server SHALL count each dropped command in the sum `mu.commands.dropped`. Its attributes SHALL be `mu.world` and `mu.drop.reason`, `depth` or `rate`. It SHALL count each script error in the sum `mu.script.errors`, with `mu.world`. The gauge `mu.world.state` SHALL carry `mu.world` and `mu.world.state.name`. Its value SHALL be 1 for the current state of the world, and 0 for each state that it left. A world SHALL report `disconnected` from the start of the server.

#### Scenario: A command past the depth limit
- **GIVEN** a world with a handler that sends a command for each event, past the depth limit
- **WHEN** the engine drops the command
- **THEN** the server records 1 in `mu.commands.dropped` with `mu.drop.reason` equal to `depth`
- **AND** it records 1 in `mu.script.errors` for that world

#### Scenario: A command over the rate
- **GIVEN** a world with a paused clock and the rate limit of 20 commands in each second
- **AND** a handler that sends 25 commands for one event
- **WHEN** the engine handles the 25 commands
- **THEN** the server records 5 in `mu.commands.dropped` for that world, with `mu.drop.reason` equal to `rate`

#### Scenario: A world that never connects
- **GIVEN** a server that exports to a fake collector, and a configuration that holds the world `alpha/aria`
- **WHEN** the server starts and gets SIGTERM
- **THEN** the last export of `mu.world.state` for `alpha/aria` has the value 1 for `disconnected`

#### Scenario: The state at the stop
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** the world connects at the start, and the server gets SIGTERM
- **THEN** the last export of `mu.world.state` for `alpha/aria` has the value 1 for `disconnected` and 0 for `connected`

### Requirement: No password and no line text in the telemetry
No span, attribute, event, metric, resource or log line SHALL hold the password of a world or the text of a line. The server SHALL put only ids, names, numbers and states in the telemetry.

#### Scenario: The password stays out
- **GIVEN** a fake collector that `server.toml` names, and the fake world `alpha/aria` reads its password `s3cret-pw` from `ALPHA_PW`
- **WHEN** the world logs in, a client calls `Send` for `alpha/aria` with the text `say s3cret-pw`, and the server gets SIGTERM
- **THEN** no byte sequence of any request that the collector got holds `s3cret-pw`
- **AND** no line of stderr holds `s3cret-pw`

#### Scenario: The line text stays out
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** a client calls `Send` for `alpha/aria` with the text `say marker-7f3a`, and the server gets SIGTERM
- **THEN** no byte sequence of any request that the collector got holds `marker-7f3a`

### Requirement: A slow collector does not slow a world
The export SHALL run apart from the engine, with a bounded queue. When the queue is full, the server SHALL drop the new data. A collector that does not answer SHALL NOT delay a command. At the stop, the server SHALL wait for the export at most five seconds, then exit.

#### Scenario: A collector that never answers
- **GIVEN** a fake collector that accepts each connection and never answers, and `server.toml` names it
- **WHEN** a client calls `Send` for the fake world `alpha/aria` 2100 times, so that the spans fill the queue
- **THEN** each call returns a sequence number
- **AND** after SIGTERM, the server exits with status 0 within seven seconds of the signal
