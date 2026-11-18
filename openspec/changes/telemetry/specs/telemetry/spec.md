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
`server.toml` SHALL hold only the table `telemetry`, and that table SHALL hold only `endpoint`, `protocol` and `certificate`. An unknown table or field SHALL stop the start. So SHALL a value of the wrong type, an endpoint that is not a URL, and a scheme other than `http` and `https`. So SHALL an endpoint with a query or a fragment. The server SHALL write an error that names `server.toml` and the field, and exit with status 1. A `server.toml` that cannot be read SHALL stop the start with an error that names its path.

#### Scenario: An unknown field
- **GIVEN** `server.toml` sets `[telemetry] endpont = "http://127.0.0.1:4318"`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml` and `telemetry.endpont`
- **AND** the exit status is 1

#### Scenario: A configuration directory that is a file
- **GIVEN** `MU_CONFIG` names a regular file
- **WHEN** the user runs `mu server`
- **THEN** stderr names the path of that file
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

#### Scenario: An endpoint with a query
- **GIVEN** `server.toml` sets `[telemetry] endpoint = "http://127.0.0.1:4318?tenant=a"`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml` and `telemetry.endpoint`
- **AND** the exit status is 1

#### Scenario: An endpoint with another scheme
- **GIVEN** `server.toml` sets `[telemetry] endpoint = "ftp://collector.example:4318"`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml`, `telemetry.endpoint` and `ftp`
- **AND** the exit status is 1

### Requirement: The standard variables
The server SHALL read `OTEL_EXPORTER_OTLP_ENDPOINT` as a base URL, the same as the field `endpoint`. A value of that variable SHALL win over `server.toml`. `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` and `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` SHALL each be the full URL of one signal. The server SHALL add no path to them, and each SHALL win over the base URL for its signal. An empty variable SHALL count as unset. When the OpenTelemetry library ignores the value of a variable that it reads at the start, the server SHALL write a line to stderr that names the variable.

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

#### Scenario: A value that the library ignores
- **GIVEN** a fake collector, `server.toml` names it, and `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE` is `deltaa`
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** stderr names `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE`

### Requirement: Each endpoint variable is checked
Each of the three endpoint variables SHALL pass the check of the field `endpoint`: a URL with the scheme `http` or `https`, and a host. A bad value SHALL stop the start with an error that names the variable, and the exit status SHALL be 1. Each telemetry variable that the server reads SHALL be UTF-8, except `OTEL_EXPORTER_OTLP_CERTIFICATE`, which is a path. A value that is not UTF-8 SHALL stop the start with an error that names the variable.

#### Scenario: A bad variable
- **GIVEN** `OTEL_EXPORTER_OTLP_ENDPOINT` is `ftp://collector.example:4318`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `OTEL_EXPORTER_OTLP_ENDPOINT` and `ftp`
- **AND** the exit status is 1

#### Scenario: A bad variable of one signal
- **GIVEN** one of `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` and `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` is `ftp://collector.example:4318/v1/x`, or is `not a url`
- **WHEN** the user runs `mu server`
- **THEN** stderr names that variable
- **AND** the exit status is 1

#### Scenario: A variable that is not UTF-8
- **GIVEN** `OTEL_EXPORTER_OTLP_ENDPOINT` holds the byte `0xff`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `OTEL_EXPORTER_OTLP_ENDPOINT` and `UTF-8`
- **AND** the exit status is 1

### Requirement: The variables that turn the export off and name the service
`OTEL_SDK_DISABLED` with the value `true`, in any case, SHALL turn the export off, whatever the endpoint. The server SHALL then read and check no other telemetry variable and no value of `[telemetry]`. An unknown table or field, or a value of the wrong type, in `server.toml` SHALL still stop the start. A value other than `true`, `false` or empty, in any case, SHALL leave the export as the other settings make it, and the server SHALL write a warning that names `OTEL_SDK_DISABLED`. The resource of each export SHALL carry `service.name`. If `OTEL_SERVICE_NAME` is set and not empty, its value SHALL be that variable. Otherwise, its value SHALL be `mu`.

#### Scenario: The export is off
- **GIVEN** a fake collector, `server.toml` names it, and `OTEL_SDK_DISABLED` is `TRUE`
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got no request
- **AND** the exit status is 0

#### Scenario: A value that is not true or false
- **GIVEN** a fake collector, `server.toml` names it, and `OTEL_SDK_DISABLED` is `1`
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** stderr names `OTEL_SDK_DISABLED`
- **AND** the collector got requests

#### Scenario: The export is off with a bad variable
- **GIVEN** `OTEL_SDK_DISABLED` is `true`, and `OTEL_EXPORTER_OTLP_ENDPOINT` is `ftp://collector.example:4318`
- **WHEN** a server starts and gets SIGTERM
- **THEN** the exit status is 0

#### Scenario: The export is off with an unknown field
- **GIVEN** `OTEL_SDK_DISABLED` is `true`, and `server.toml` sets `[telemetry] endpont = "http://127.0.0.1:4318"`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml` and `telemetry.endpont`
- **AND** the exit status is 1

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

### Requirement: The protocol of the export
The protocol SHALL be `http/protobuf` or `grpc`, and `http/protobuf` by default. The field `protocol` of `[telemetry]` SHALL set it. `OTEL_EXPORTER_OTLP_PROTOCOL` SHALL win over the field, and `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL` and `OTEL_EXPORTER_OTLP_METRICS_PROTOCOL` SHALL each win for its signal. With `grpc`, the server SHALL call the OTLP gRPC services at the endpoint, with no path added.

#### Scenario: gRPC from the file
- **GIVEN** a fake gRPC collector at `http://127.0.0.1:<port>`, and `server.toml` sets that endpoint and `protocol = "grpc"`
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got at least one call of `TraceService/Export` and one of `MetricsService/Export`

#### Scenario: The variable wins over the protocol of the file
- **GIVEN** a fake gRPC collector that `server.toml` names with `protocol = "http/protobuf"`, and `OTEL_EXPORTER_OTLP_PROTOCOL` is `grpc`
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got at least one call of `TraceService/Export`

#### Scenario: The protocol of one signal
- **GIVEN** a fake HTTP collector that `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` names, a fake gRPC collector that `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` names, and `OTEL_EXPORTER_OTLP_METRICS_PROTOCOL` is `grpc`
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the HTTP collector got a `POST` with traces at the path of its variable
- **AND** the gRPC collector got a call of `MetricsService/Export`

### Requirement: An unknown protocol is refused
While a signal has an endpoint, a protocol other than `http/protobuf` and `grpc` SHALL stop the start. The error SHALL name its source, the field or the variable, and the exit status SHALL be 1. The error SHALL NOT hold the value, because a URL in the wrong place can hold a password. With the export off, the server SHALL NOT read the protocol.

#### Scenario: The protocol http/json
- **GIVEN** a fake collector that `server.toml` names, and `OTEL_EXPORTER_OTLP_PROTOCOL` is `http/json`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `OTEL_EXPORTER_OTLP_PROTOCOL`
- **AND** stderr does not hold `http/json`
- **AND** the exit status is 1

#### Scenario: An unknown protocol in the file
- **GIVEN** `server.toml` sets `endpoint = "http://127.0.0.1:4318"` and `protocol = "thrift"`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml` and `telemetry.protocol`
- **AND** the exit status is 1

#### Scenario: An unknown protocol with the export off
- **GIVEN** an empty configuration directory, no endpoint variable, and `OTEL_EXPORTER_OTLP_PROTOCOL` is `http/json`
- **WHEN** a server starts and gets SIGTERM
- **THEN** the exit status is 0

### Requirement: A gRPC endpoint holds no path
With the protocol `grpc`, the endpoint that a signal uses SHALL hold no path other than `/`, and no query. The gRPC client sends only the scheme and the host. Such an endpoint SHALL stop the start with an error that names its source, and the exit status SHALL be 1.

#### Scenario: A gRPC endpoint with a path
- **GIVEN** `server.toml` sets `endpoint = "http://127.0.0.1:4317/otel"` and `protocol = "grpc"`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml` and `telemetry.endpoint`
- **AND** the exit status is 1

### Requirement: TLS for an https endpoint
An endpoint with the scheme `https` SHALL use TLS, for both protocols. The server SHALL trust the root certificates of the system. It SHALL also trust the PEM certificates of the file that the field `certificate` names. `OTEL_EXPORTER_OTLP_CERTIFICATE` SHALL win over the field. A relative path in the field SHALL start at the configuration directory. A certificate of the file that does not parse SHALL stop the start, for both protocols. The server SHALL NOT export to a collector whose certificate it does not trust.

#### Scenario: HTTPS with a certificate file
- **GIVEN** a fake HTTPS collector with a certificate from a test authority
- **AND** `server.toml` sets its `https://` endpoint, and `certificate` names the file of that authority
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got `POST /v1/traces` and `POST /v1/metrics`

#### Scenario: A relative certificate path
- **GIVEN** a fake HTTPS collector with a certificate from a test authority
- **AND** `server.toml` sets its `https://` endpoint and `certificate = "certs/ca.pem"`, and `certs/ca.pem` in the configuration directory is the file of that authority
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got `POST /v1/traces`

#### Scenario: gRPC with TLS
- **GIVEN** a fake gRPC collector over TLS with a certificate from a test authority
- **AND** `server.toml` sets its `https://` endpoint and `protocol = "grpc"`
- **AND** `OTEL_EXPORTER_OTLP_CERTIFICATE` names the file of that authority
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got a call of `TraceService/Export`

#### Scenario: A collector that the server does not trust
- **GIVEN** a fake HTTPS collector with a certificate from a test authority, and `server.toml` sets its `https://` endpoint and no `certificate`
- **WHEN** a server starts, serves one `Send`, and gets SIGTERM
- **THEN** the collector got no complete request
- **AND** the exit status is 0

#### Scenario: A missing certificate file
- **GIVEN** `server.toml` sets an endpoint and `certificate = "/nonexistent/ca.pem"`
- **WHEN** the user runs `mu server`
- **THEN** stderr names `server.toml`, `telemetry.certificate` and `/nonexistent/ca.pem`
- **AND** the exit status is 1

#### Scenario: A file that holds no certificate
- **GIVEN** an endpoint in `server.toml`, and `OTEL_EXPORTER_OTLP_CERTIFICATE` names a file that holds no PEM certificate
- **WHEN** the user runs `mu server`
- **THEN** stderr names `OTEL_EXPORTER_OTLP_CERTIFICATE`
- **AND** the exit status is 1

#### Scenario: A certificate that does not parse
- **GIVEN** an `https` endpoint, with `http/protobuf` and then with `grpc`
- **AND** `OTEL_EXPORTER_OTLP_CERTIFICATE` names a file with a `CERTIFICATE` section whose bytes are not a certificate
- **WHEN** the user runs `mu server`
- **THEN** stderr names `OTEL_EXPORTER_OTLP_CERTIFICATE` and the path of the file
- **AND** the exit status is 1

### Requirement: A span for each API call
The server SHALL make one span for each call of the API, named `mu.v1.PortalService/<method>`. It SHALL carry `rpc.system` (`connect_rpc`), `rpc.service` and `rpc.method`. A call that names a world SHALL carry `mu.world` with the world id. A call that fails SHALL have the status `error`, and SHALL carry `rpc.connect_rpc.error_code` with the name of its Connect error code. The span of `Subscribe` SHALL cover the call up to the return of its stream. It SHALL NOT cover the life of the stream.

#### Scenario: The span of a Send
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** a client calls `Send` for `alpha/aria`, and the server gets SIGTERM
- **THEN** the collector got a span named `mu.v1.PortalService/Send` with `mu.world` equal to `alpha/aria`
- **AND** that span has `rpc.system` equal to `connect_rpc`, `rpc.service` equal to `mu.v1.PortalService` and `rpc.method` equal to `Send`

#### Scenario: The span of a Subscribe
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** a client subscribes, then calls `Send` for `alpha/aria`, reads the event of that `Send` on the stream, and the server gets SIGTERM
- **THEN** the span named `mu.v1.PortalService/Subscribe` ends before the span named `mu.v1.PortalService/Send` starts

#### Scenario: A failed call
- **GIVEN** a server that exports to a fake collector
- **WHEN** a client calls `Connect` for the unknown world `zeta/zed`, and the server gets SIGTERM
- **THEN** the collector got a span named `mu.v1.PortalService/Connect` with the status `error`
- **AND** that span has `rpc.connect_rpc.error_code` equal to `not_found`

### Requirement: A span for each command of a world
The server SHALL make one span named `command` for each command that a world handles. It SHALL carry `mu.world`, `mu.command`, `mu.origin` and `mu.depth`. The values of `mu.command` SHALL be `send`, `connect`, `disconnect`, `login`, `echo` and `emit`. The values of `mu.origin` SHALL be `player`, `script` and `system`. The span of a command from the API SHALL be a child of the span of that call.

#### Scenario: A command under its call
- **GIVEN** a server with the fake world `alpha/aria` that exports to a fake collector
- **WHEN** a client calls `Send` for `alpha/aria`, and the server gets SIGTERM
- **THEN** the collector got a span named `command` with `mu.world` equal to `alpha/aria`, `mu.command` equal to `send` and `mu.origin` equal to `player`
- **AND** its parent is the span named `mu.v1.PortalService/Send`

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
- **GIVEN** a journal whose live channel holds four events, and a stream of the API that read its `Hello` and no event
- **WHEN** five events are appended before the stream reads again
- **THEN** the stream ends with `resource_exhausted`
- **AND** the server records 1 in `mu.subscriber.lagged`

### Requirement: Metrics of the worlds
The server SHALL count each dropped command in the sum `mu.commands.dropped`. Its attributes SHALL be `mu.world` and `mu.drop.reason`, `depth` or `rate`. It SHALL count each script error in the sum `mu.script.errors`, with `mu.world`. The gauge `mu.world.state` SHALL carry `mu.world` and `mu.world.state.name`. Its value SHALL be 1 for the current state of the world, and 0 for each state that it left. A world SHALL report `disconnected` from the start of the server.

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
