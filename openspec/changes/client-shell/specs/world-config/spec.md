## MODIFIED Requirements

### Requirement: The configuration directory
The flag `--config <dir>` of `mu server`, `mu world` and `mu client` SHALL name the configuration directory, then `MU_CONFIG`. An empty `MU_CONFIG` SHALL count as unset. Next comes `mu` under `XDG_CONFIG_HOME`, then `.config/mu` under `HOME`. Each of these two SHALL count only with an absolute path. With none, there SHALL be no directory. A missing directory SHALL hold no worlds and SHALL NOT be an error. If `--config` or `MU_CONFIG` names a missing directory, stderr SHALL name it.

#### Scenario: The flag wins over the environment
- **GIVEN** `MU_CONFIG` is `<dir>/env` with the world `alpha/aria`, and `<dir>/flag` holds the world `beta/bryn`
- **WHEN** the user runs `mu world list --config <dir>/flag`
- **THEN** stdout is `beta/bryn` and a line end
- **AND** the exit status is 0

#### Scenario: The default directory
- **GIVEN** `XDG_CONFIG_HOME` is `<dir>`, `<dir>/mu` holds the world `alpha/aria`, and neither `MU_CONFIG` nor `--config` is given
- **WHEN** the user runs `mu world list`
- **THEN** stdout is `alpha/aria` and a line end

#### Scenario: The directory under the home directory
- **GIVEN** `XDG_CONFIG_HOME` is empty, `HOME` is `<dir>`, and `<dir>/.config/mu` holds the world `alpha/aria`
- **WHEN** the user runs `mu world list`
- **THEN** stdout is `alpha/aria` and a line end

#### Scenario: An empty MU_CONFIG
- **GIVEN** `MU_CONFIG` is empty, `XDG_CONFIG_HOME` is `<dir>`, and `<dir>/mu` holds the world `alpha/aria`
- **WHEN** the user runs `mu world list`
- **THEN** stdout is `alpha/aria` and a line end

#### Scenario: A relative XDG_CONFIG_HOME
- **GIVEN** the current directory is `<dir>`, `XDG_CONFIG_HOME` is `xdg`, and `<dir>/xdg/mu` holds the world `beta/bryn`
- **AND** `HOME` is `<dir>/home`, and `<dir>/home/.config/mu` holds the world `alpha/aria`
- **WHEN** the user runs `mu world list`
- **THEN** stdout is `alpha/aria` and a line end

#### Scenario: No home directory
- **GIVEN** `MU_CONFIG` and `XDG_CONFIG_HOME` are not set, `HOME` is empty, and the current directory `<dir>` holds `.config/mu` with the world `alpha/aria`
- **WHEN** the user runs `mu world list`
- **THEN** stdout is empty
- **AND** the exit status is 0

#### Scenario: A missing directory
- **WHEN** the user runs `mu world list --config <dir>/none`
- **THEN** stdout is empty
- **AND** stderr names `<dir>/none`
- **AND** the exit status is 0

#### Scenario: A missing default directory
- **GIVEN** `XDG_CONFIG_HOME` is `<dir>`, `<dir>/mu` does not exist, and neither `MU_CONFIG` nor `--config` is given
- **WHEN** the user runs `mu world list`
- **THEN** stdout and stderr are empty
- **AND** the exit status is 0

#### Scenario: The client takes the flag
- **GIVEN** `MU_CONFIG` is `<dir>/env` with a `client.toml` that holds `echo = "loud"`, and `<dir>/flag` holds no `client.toml`
- **WHEN** the user starts `mu client --config <dir>/flag`
- **THEN** the client opens the screen

### Requirement: Each character file makes one world
The directory SHALL hold `global.toml`, `types/<name>.toml`, `games/<game>.toml` and `characters/<game>/<character>.toml`, and each file SHALL be optional. It can also hold `server.toml` at its top level, the configuration of the server, and `client.toml`, the configuration of the terminal client. Neither file SHALL make a world or a layer. Each character file SHALL make the world `<game>/<character>`, and its game SHALL be `games/<game>.toml`. A file name that is not a part of a world id SHALL be an error that names the file. So SHALL a character with no game file, and any other file that ends in `.toml`. So SHALL an entry that cannot be read, such as a link to nothing.

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

#### Scenario: The file of the client
- **GIVEN** a directory with `games/alpha.toml`, `characters/alpha/aria.toml` and a `client.toml` that holds `echo = "hide"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stdout holds the line `alpha/aria`
- **AND** the exit status is 0
