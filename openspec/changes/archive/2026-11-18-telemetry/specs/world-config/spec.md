# Spec Delta

## MODIFIED Requirements

### Requirement: Each character file makes one world
The directory SHALL hold `global.toml`, `types/<name>.toml`, `games/<game>.toml` and `characters/<game>/<character>.toml`, and each file SHALL be optional. It can also hold `server.toml` at its top level, the configuration of the server. That file SHALL make no world and no layer. Each character file SHALL make the world `<game>/<character>`, and its game SHALL be `games/<game>.toml`. A file name that is not a part of a world id SHALL be an error that names the file. So SHALL a character with no game file, and any other file that ends in `.toml`. So SHALL an entry that cannot be read, such as a link to nothing.

#### Scenario: Worlds from character files
- **GIVEN** a directory with `games/alpha.toml`, `games/beta.toml`, `characters/alpha/aria.toml`, `characters/alpha/ann.toml` and `characters/beta/bryn.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stdout holds the lines `alpha/ann`, `alpha/aria` and `beta/bryn`, in that order

#### Scenario: A character with no game
- **GIVEN** a directory with `characters/gamma/gil.toml` and no `games/gamma.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `characters/gamma/gil.toml` and `games/gamma.toml`
- **AND** the exit status is 1

#### Scenario: A file name outside the id rule
- **GIVEN** a directory with `games/Beta.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/Beta.toml`
- **AND** the exit status is 1

#### Scenario: A stray file
- **GIVEN** a directory with one of `worlds.toml`, `characters/alpha.toml`, `games/old/alpha.toml` and `characters/alpha/aria/x.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names that path
- **AND** the exit status is 1

#### Scenario: A link to nothing
- **GIVEN** `games/beta` is a symbolic link to a path that does not exist
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/beta`
- **AND** the exit status is 1

#### Scenario: The file of the server
- **GIVEN** a directory with `games/alpha.toml`, `characters/alpha/aria.toml` and a `server.toml` that sets `[telemetry] endpoint = "http://127.0.0.1:4318"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stdout holds the line `alpha/aria`
- **AND** the exit status is 0

#### Scenario: A character named server
- **GIVEN** a directory with `games/alpha.toml` and `characters/alpha/server.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stdout holds the line `alpha/server`
- **AND** the exit status is 0
