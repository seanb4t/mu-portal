# Spec Delta

## MODIFIED Requirements

### Requirement: Built-in types set only quirks
The binary SHALL hold the built-in types `tinymux`, `aresmush` and `evennia`. A built-in type SHALL set only quirk fields: `charset`, `login.template`, `keepalive.command` and `keepalive.seconds`. The built-in type `tinymux` SHALL set `charset = "iso-8859-1"`. A name of a character set SHALL match with no regard to case. A user type SHALL NOT have the name of a built-in type.

#### Scenario: Each built-in type reads as quirks
- **WHEN** a test reads each built-in type with the schema of the quirk fields, which refuses every other field
- **THEN** each read succeeds

#### Scenario: A global preference is not hidden by a type
- **GIVEN** `global.toml` sets `columns = 100`, and the game `alpha` extends `tinymux`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows `columns = 100`

#### Scenario: A user type with a built-in name
- **GIVEN** a directory with `types/tinymux.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `types/tinymux.toml` and `tinymux`
- **AND** the exit status is 1

### Requirement: Each field is known
Each layer SHALL set only the fields of a world. The connection fields are `host`, `port`, `tls` and `autoconnect`. The display fields are `charset`, `columns` and `rows`. The tables are `login`, `keepalive` and `reconnect`. `login` holds `template`, `name` and `secret`. `keepalive` holds `mode`, `command` and `seconds`. `reconnect` holds `mode`, `initial`, `max` and `attempts`. The arrays are `rules`, `scripts` and `disable`. A layer that allows it also sets `extends`. An unknown field, or a value of the wrong type or out of range, SHALL be an error that names the file and the field.

#### Scenario: An unknown field
- **GIVEN** the game `alpha` sets `hots = "mux.example.org"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `hots`
- **AND** the exit status is 1

#### Scenario: A port out of range
- **GIVEN** the game `alpha` sets `port = 70000`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `port`
- **AND** the exit status is 1

#### Scenario: A template with no password
- **GIVEN** the game `alpha` sets `login.template = "connect {login}"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `login.template`
- **AND** the exit status is 1

#### Scenario: The case of a character set
- **GIVEN** the games `alpha`, `beta` and `gamma` set `charset` to `ISO-8859-1`, `iso-8859-1` and `utf-8`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** the exit status is 0

#### Scenario: A character set that the engine cannot read
- **GIVEN** the game `alpha` sets `charset = "koi8-r"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `charset`
- **AND** the exit status is 1

#### Scenario: An unknown mode
- **GIVEN** the game `alpha` sets `keepalive.mode = "often"` or `reconnect.mode = "always"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and the field
- **AND** the exit status is 1

### Requirement: Each value is in its range
The range of `port`, `columns` and `rows` SHALL be 1 to 65535. The range of `keepalive.seconds` SHALL be 1 to 86400. The range of `reconnect.initial` SHALL be 1 to 3600, of `reconnect.max` 1 to 86400, and of `reconnect.attempts` 0 to 10000. `login.name`, `login.template` and `keepalive.command` SHALL hold no CR and no LF. `login.template` SHALL hold `{password}`. A value outside its range SHALL be an error that names the file and the field.

#### Scenario: The edges of each range
- **GIVEN** a field of `port`, `columns`, `rows`, `keepalive.seconds`, `reconnect.initial` and `reconnect.max`, and for each field a value at its edges: 0, 1 and the top of its range, and one more than the top
- **AND** the game sets `reconnect.max = 86400` beside each value of `reconnect.initial`
- **WHEN** the user runs `mu world list --config <dir>` with the game `alpha` that sets that field to that value
- **THEN** the exit status is 0 for 1 and the top of the range
- **AND** for 0 and one more than the top, stderr names `games/alpha.toml` and the field, and the exit status is 1

#### Scenario: The edges of the attempts
- **GIVEN** the game `alpha` sets `reconnect.attempts` to 0, 10000 or 10001
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** the exit status is 0 for 0 and 10000
- **AND** for 10001, stderr names `games/alpha.toml` and `reconnect.attempts`, and the exit status is 1

#### Scenario: A line break in a line field
- **GIVEN** the game `alpha` sets one of `login.name`, `login.template` and `keepalive.command` to a value with a CR or an LF
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and the field
- **AND** the exit status is 1

### Requirement: A shared plugin layer cannot set the connection
A layer from a shared plugin source SHALL NOT set `host`, `port`, `tls`, `autoconnect`, `login.name`, `login.secret` or `login.template`. Such a layer SHALL be an error that names the source and the field.

#### Scenario: A plugin sets a field of the connection
- **GIVEN** a layer from a shared plugin source named `colors` that sets one field of this list: `host = "evil.example.org"`, `port = 4201`, `tls = true`, `autoconnect = true`, `login.name = "Eve"`, `login.secret = "env:EVE_PW"` or `login.template = "page eve={password}"`
- **WHEN** the configuration resolves the world `alpha/aria` with that layer
- **THEN** the resolution fails with an error that names `colors` and the field

#### Scenario: A plugin sets a preference
- **GIVEN** a layer from a shared plugin source named `colors` that sets `columns = 72`
- **WHEN** the configuration resolves the world `alpha/aria` with that layer after the game
- **THEN** the resolved world has `columns = 72`

### Requirement: A world has a place to connect
After the merge, each world SHALL have `host` and `port`, and `host` SHALL NOT be empty. A world with `login.name` or `login.secret` SHALL have both. A world with neither SHALL log in by hand. Without `login.name` and `login.secret`, a `login.template` SHALL NOT count. Unless `keepalive.mode` is `off`, a world with `keepalive.command` or `keepalive.seconds` SHALL have both. A world with neither half SHALL have no keepalive, whatever its mode. A missing field SHALL be an error that names the world and the field.

#### Scenario: A world with no host or port
- **GIVEN** no layer of `alpha/aria` sets `host`, or the last layer that sets it sets `host = ""`, or no layer sets `port`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `alpha/aria` and the missing field
- **AND** the exit status is 1

#### Scenario: A login name with no secret
- **GIVEN** the character sets `login.name = "Aria"` and no layer sets `login.secret`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `alpha/aria` and `login.secret`
- **AND** the exit status is 1

#### Scenario: A login secret with no name
- **GIVEN** the character sets `login.secret = "env:ARIA_PW"` and no layer sets `login.name`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `alpha/aria` and `login.name`
- **AND** the exit status is 1

#### Scenario: Half a keepalive
- **GIVEN** the game `alpha` sets `extends = "aresmush"`, which sets no keepalive
- **AND** the game sets only `keepalive.command = "idle"`, or only `keepalive.seconds = 60`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `alpha/aria` and the half that is missing, `keepalive.seconds` or `keepalive.command`
- **AND** the exit status is 1

#### Scenario: Half a keepalive that is off
- **GIVEN** the game `alpha` sets `extends = "aresmush"`, `keepalive.mode = "off"` and only `keepalive.command = "idle"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** the exit status is 0

#### Scenario: A mode with no keepalive
- **GIVEN** the game `alpha` sets `extends = "aresmush"` and `keepalive.mode = "interval"`, and no layer sets `keepalive.command` or `keepalive.seconds`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows `mode = "interval"` under `keepalive`, and no `command` and no `seconds`
- **AND** the exit status is 0

#### Scenario: A world that logs in by hand
- **GIVEN** no layer of `alpha/aria` sets `login.name` or `login.secret`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows no login
- **AND** the exit status is 0

### Requirement: The server holds the configured worlds
At the start, `mu server` SHALL resolve the configuration directory. A bad configuration SHALL stop the start with an error that names the file, and the exit status SHALL be 1. A world id that is both a configured world and a `--fake-world` SHALL be an error that names the id. The server SHALL NOT read the secret of a configured world at the start, unless the world sets `autoconnect = true`. Such a world SHALL read its secret at its connect at the start, and a failed read SHALL NOT stop the start.

#### Scenario: A bad configuration stops the start
- **GIVEN** the game `alpha` sets `port = 70000`
- **WHEN** the user runs `mu server --socket <dir>/s.sock --config <dir>/conf`
- **THEN** stderr names `games/alpha.toml`
- **AND** no file `<dir>/s.sock` exists
- **AND** the exit status is 1

#### Scenario: A fake world with a configured id
- **GIVEN** the configuration holds the world `alpha/aria`
- **WHEN** the user runs `mu server --socket <dir>/s.sock --config <dir>/conf --fake-world alpha/aria=env:PW`
- **THEN** stderr names `alpha/aria`
- **AND** the exit status is 1

#### Scenario: The secret is not read at the start
- **GIVEN** the character sets `login.name = "Aria"` and `login.secret = "env:ARIA_PW"`, with no `autoconnect`, and `ARIA_PW` is not set
- **WHEN** the user runs `mu server --socket <dir>/s.sock --config <dir>/conf`
- **THEN** an API call to list the worlds on `<dir>/s.sock` succeeds

#### Scenario: An autoconnect world whose secret is missing
- **GIVEN** a fake game on TCP, and the character sets its host and port, `autoconnect = true`, `login.name = "Aria"` and `login.secret = "env:ARIA_PW"`, and `ARIA_PW` is not set
- **WHEN** the user runs `mu server --socket <dir>/s.sock --config <dir>/conf`
- **THEN** an API call to list the worlds on `<dir>/s.sock` returns `alpha/aria` in the state `disconnected`
- **AND** the fake game gets no connection

## ADDED Requirements

### Requirement: The connection fields have defaults
A world that no layer gives the field SHALL have `autoconnect = false`, `keepalive.mode = "idle"`, `reconnect.mode = "backoff"`, `reconnect.initial = 1`, `reconnect.max = 60` and `reconnect.attempts = 30`. `mu world show` SHALL name the source `default` for each of them.

#### Scenario: The defaults
- **GIVEN** a world `alpha/aria` whose layers set none of these fields
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows `autoconnect = false`, `mode = "idle"` under `keepalive`, and `mode = "backoff"`, `initial = 1`, `max = 60` and `attempts = 30` under `reconnect`
- **AND** each of these lines names the source `default`

### Requirement: The first wait is not longer than the last
After the merge, `reconnect.initial` SHALL NOT be greater than `reconnect.max`. Such a world SHALL be an error that names the world and both fields.

#### Scenario: A first wait over the cap
- **GIVEN** the game `alpha` sets `reconnect.initial = 120` and `reconnect.max = 60`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `alpha/aria`, `reconnect.initial` and `reconnect.max`
- **AND** the exit status is 1

#### Scenario: A first wait equal to the cap
- **GIVEN** the game `alpha` sets `reconnect.initial = 60` and `reconnect.max = 60`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** the exit status is 0
