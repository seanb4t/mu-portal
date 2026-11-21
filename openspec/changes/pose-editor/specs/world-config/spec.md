# Spec Delta

## MODIFIED Requirements

### Requirement: Built-in types set only quirks
The binary SHALL hold the built-in types `tinymux`, `pennmush`, `rhostmush`, `aresmush` and `evennia`. A built-in type SHALL set only quirk fields: `charset`, `line_break`, `login.template`, `keepalive.command` and `keepalive.seconds`. A built-in type can also name another built-in type with `extends`, and it then gets each quirk that it does not set. The types `pennmush` and `rhostmush` SHALL extend `tinymux`. The built-in type `tinymux` SHALL set `charset = "iso-8859-1"`. The types `tinymux` and `aresmush` SHALL set `line_break = "%r"`, and `evennia` SHALL set `line_break = "|/"`. A name of a character set SHALL match with no regard to case. A user type SHALL NOT have the name of a built-in type.

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

#### Scenario: The line break of each built-in type
- **GIVEN** the games `alpha`, `beta` and `gamma` extend `tinymux`, `aresmush` and `evennia`
- **WHEN** the user runs `mu world show` for a character of each game with `--config <dir>`
- **THEN** stdout shows `line_break = "%r"` for `alpha` and `beta`, and `line_break = "|/"` for `gamma`

#### Scenario: A built-in type that extends another
- **GIVEN** the games `alpha` and `beta` extend `pennmush` and `rhostmush`
- **WHEN** the user runs `mu world show` for a character of each game with `--config <dir>`
- **THEN** for each game, stdout shows `charset = "iso-8859-1"` and `line_break = "%r"`

#### Scenario: A built-in type extends only a built-in type
- **WHEN** a test reads each built-in type
- **THEN** each `extends` names another built-in type, and no chain of types returns to its start

#### Scenario: A game overrides the line break of its type
- **GIVEN** the game `alpha` extends `tinymux` and sets `line_break = "%R"`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows `line_break = "%R"`

### Requirement: Each field is known
Each layer SHALL set only the fields of a world. The connection fields are `host`, `port`, `tls` and `autoconnect`. The display fields are `charset`, `columns` and `rows`. The field `line_break` is the text that the game reads as a line break inside one command. The tables are `login`, `keepalive` and `reconnect`. `login` holds `template`, `name` and `secret`. `keepalive` holds `mode`, `command` and `seconds`. `reconnect` holds `mode`, `initial`, `max` and `attempts`. The arrays are `rules`, `scripts` and `disable`. A layer that allows it also sets `extends`. An unknown field, or a value of the wrong type or out of range, SHALL be an error that names the file and the field.

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
The range of `port`, `columns` and `rows` SHALL be 1 to 65535. The range of `keepalive.seconds` SHALL be 1 to 86400. The range of `reconnect.initial` SHALL be 1 to 3600, of `reconnect.max` 1 to 86400, and of `reconnect.attempts` 0 to 10000. `login.name`, `login.template` and `keepalive.command` SHALL hold no CR and no LF. `line_break` SHALL hold 1 to 16 characters and no control character. `login.template` SHALL hold `{password}`. A value outside its range SHALL be an error that names the file and the field.

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

#### Scenario: The edges of the line break
- **GIVEN** the game `alpha` sets `line_break` to one of these values: an empty text, `%r`, 16 letters `x`, 17 letters `x`, 16 letters `é` and 17 letters `é`
- **AND** the other values are `%` with one of these characters: a tab, an LF, a CR, U+0085 and DEL
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** the exit status is 0 for `%r`, for 16 letters `x` and for 16 letters `é`
- **AND** for each other value, stderr names `games/alpha.toml` and `line_break`, and the exit status is 1
