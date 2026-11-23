# Spec Delta

## Purpose

Defines the values that a client needs before a server answers: the URL of the server with its connection options, and the name of the device. A client reads them from flags and the environment only.

## ADDED Requirements

### Requirement: The server URL
`mu client` and `mu config` SHALL take the URL of the server from the flag `--server`, then from `MU_SERVER`. An empty `MU_SERVER` SHALL count as unset. With neither, the URL SHALL be `unix://` and the default socket path of `mu server`. `MU_SOCKET` SHALL NOT change the URL of a client.

#### Scenario: The flag wins
- **GIVEN** a server serves `<dir>/flag.sock`, nothing serves `<dir>/env.sock`, and `MU_SERVER` is `unix://<dir>/env.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/flag.sock --device laptop`
- **THEN** the exit status is 0

#### Scenario: The variable
- **GIVEN** a server serves `<dir>/env.sock`, and `MU_SERVER` is `unix://<dir>/env.sock`
- **WHEN** the user runs `mu config show --device laptop`
- **THEN** the exit status is 0

#### Scenario: The default
- **GIVEN** `XDG_RUNTIME_DIR` is `<dir>`, `mu server` with no `--socket` serves, and neither `MU_SERVER` nor `--server` is given
- **WHEN** the user runs `mu config show --device laptop`
- **THEN** the exit status is 0

#### Scenario: MU_SOCKET does not count
- **GIVEN** `XDG_RUNTIME_DIR` is `<dir>/run`, a server serves `<dir>/other.sock`, nothing serves the default socket, and `MU_SOCKET` is `<dir>/other.sock`
- **WHEN** the user runs `mu config show --device laptop`
- **THEN** the exit status is 1, and stderr names the default socket path

### Requirement: A URL names a Unix socket
A server URL SHALL be `unix://`, then an absolute path, then an optional query. The path SHALL be decoded from percent escapes. A URL with another scheme, a relative path, a host, a user, a password or a fragment SHALL stop the command before it connects. The command SHALL exit with status 1, and stderr SHALL name the URL and the reason.

#### Scenario: A path with a space
- **GIVEN** a server serves `<dir>/a b/s.sock`
- **WHEN** the user runs `mu config show --server "unix://<dir>/a%20b/s.sock" --device laptop`
- **THEN** the exit status is 0

#### Scenario: Another scheme
- **WHEN** the user runs `mu config show --server https://example.com --device laptop`
- **THEN** the exit status is 1, and stderr contains `https` and `unix`

#### Scenario: A host
- **WHEN** the user runs `mu config show --server unix://host/tmp/s.sock --device laptop`
- **THEN** the exit status is 1, and stderr contains `unix://host/tmp/s.sock`

#### Scenario: No double slash
- **WHEN** the user runs `mu config show --server unix:/tmp/s.sock --device laptop`
- **THEN** the exit status is 1, and stderr contains `unix:/tmp/s.sock`

#### Scenario: A relative path
- **WHEN** the user runs `mu config show --server unix:s.sock --device laptop`
- **THEN** the exit status is 1, and stderr contains `unix:s.sock`

### Requirement: The connect_wait option
The query option `connect_wait` SHALL be the longest time, in seconds, that a client waits for a server to answer. It SHALL allow a fraction, and its default SHALL be 5. A value that is 0 or less, not finite, or less than 1 ns SHALL stop the command. So SHALL a query option with any other name, and an option that is there twice. The command SHALL exit with status 1, and stderr SHALL name the option.

#### Scenario: A wait of zero
- **WHEN** the user runs `mu config show --server "unix://<dir>/s.sock?connect_wait=0" --device laptop`
- **THEN** the exit status is 1, and stderr contains `connect_wait`

#### Scenario: An unknown option
- **WHEN** the user runs `mu config show --server "unix://<dir>/s.sock?colour=red" --device laptop`
- **THEN** the exit status is 1, and stderr contains `colour`

#### Scenario: An option twice
- **WHEN** the user runs `mu config show --server "unix://<dir>/s.sock?connect_wait=1&connect_wait=2" --device laptop`
- **THEN** the exit status is 1, and stderr contains `connect_wait`

#### Scenario: A short wait
- **GIVEN** a test listens on `<dir>/s.sock`, accepts each connection and never answers
- **WHEN** the user runs `mu config show --server "unix://<dir>/s.sock?connect_wait=0.2" --device laptop`
- **THEN** the exit status is 1, and stderr contains `<dir>/s.sock` and `0.2`

### Requirement: The device name
`mu client` and `mu config` SHALL take the device name from the flag `--device`, then from `MU_DEVICE`. An empty `MU_DEVICE` SHALL count as unset. With neither, the device name SHALL be the host name of the machine in lowercase. A device name SHALL have 1 to 64 characters from `a` to `z`, `0` to `9`, `-`, `_` and `.`. Another name SHALL stop the command with exit status 1, and stderr SHALL name it.

#### Scenario: The variable
- **GIVEN** `clients/devices/laptop.toml` holds `echo = "hide"`, a server serves that directory on `<dir>/s.sock`, and `MU_DEVICE` is `laptop`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock`
- **THEN** stdout holds `echo = "hide"` with the source `clients/devices/laptop.toml`

#### Scenario: The flag wins
- **GIVEN** `clients/devices/laptop.toml` holds `echo = "hide"`, a server serves that directory on `<dir>/s.sock`, and `MU_DEVICE` is `desk`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `echo = "hide"` with the source `clients/devices/laptop.toml`

#### Scenario: The host name
- **GIVEN** `<host>` is the host name of the test machine in lowercase
- **AND** `clients/devices/<host>.toml` holds `echo = "hide"`, and a server serves that directory on `<dir>/s.sock`
- **AND** neither `MU_DEVICE` nor `--device` is given
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock`
- **THEN** stdout holds `echo = "hide"` with the source `clients/devices/<host>.toml`

#### Scenario: A bad name
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device "my laptop"`
- **THEN** the exit status is 1, and stderr contains `my laptop`

#### Scenario: A name of 64 characters
- **GIVEN** a server serves `<dir>` on `<dir>/s.sock`, and `<name>` is 64 times the letter `a`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device <name>`
- **THEN** the exit status is 0

#### Scenario: A name of 65 characters
- **GIVEN** `<name>` is 65 times the letter `a`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device <name>`
- **THEN** the exit status is 1, and stderr contains `<name>`

#### Scenario: An empty device flag
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device ""`
- **THEN** the exit status is 1, and stderr contains `device`

### Requirement: The client names itself to the server
`mu client` SHALL ask the server for the configuration of the kind `tui`, the OS of the machine, and its device name. The OS SHALL be `macos` on macOS and `linux` on Linux.

#### Scenario: The OS layer applies
- **GIVEN** `clients/os/<os>.toml` for the OS of the test holds `echo = "hide"`, and a server serves that directory with the fake world `alpha/aria`
- **WHEN** the user starts `mu client --device laptop`, types `hello` and presses Enter
- **THEN** the screen shows no row that starts with `> `

### Requirement: mu client refuses the old socket flag
**BREAKING** `mu client` SHALL NOT take `--socket`. The flag SHALL be a usage error with exit status 2, and stderr SHALL name `--socket`.

#### Scenario: The socket flag
- **WHEN** the user runs `mu client --socket <dir>/s.sock` in a terminal
- **THEN** the exit status is 2, and stderr contains `--socket`
- **AND** no file `<dir>/s.sock` exists
