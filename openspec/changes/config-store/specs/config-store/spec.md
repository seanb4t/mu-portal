# Spec Delta

## Purpose

Defines how the server owns the configuration directory. It covers the check of each file, the client configuration of one client, the report of a bad file, and the reload.

## ADDED Requirements

### Requirement: The server checks every client layer at start
At the start, `mu server` SHALL read and check each file under `clients/` in the configuration directory. A bad client layer SHALL NOT stop the start. For each bad layer, the server SHALL write one line to stderr that names the file and the reason. A bad layer is a file that the server cannot read or parse. A file with an unknown field or a bad value is also a bad layer. So is a file under `clients/` that is not one of the layer files.

#### Scenario: A bad device layer at start
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds `echo = "loud"`, and `<dir>` holds the world `alpha/aria`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/devices/laptop.toml` and `echo`
- **AND** an API call on `<dir>/s.sock` lists the world `alpha/aria`

#### Scenario: A file that is not a layer
- **GIVEN** `<dir>/clients/extra.toml` holds `echo = "show"`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/extra.toml`
- **AND** an API call on `<dir>/s.sock` succeeds

#### Scenario: A device file with a bad name
- **GIVEN** `<dir>/clients/devices/Laptop.toml` holds `echo = "show"`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/devices/Laptop.toml`
- **AND** an API call on `<dir>/s.sock` succeeds

#### Scenario: A link to nothing under clients
- **GIVEN** `<dir>/clients/devices/laptop.toml` is a symbolic link to a file that does not exist
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/devices/laptop.toml`
- **AND** an API call on `<dir>/s.sock` succeeds

#### Scenario: A binding of a dot
- **GIVEN** `clients/global.toml` holds `[keys]` with `"alt+." = "quit"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `"alt+period" = "quit"` with the source `clients/global.toml`

#### Scenario: An unlistable clients directory
- **GIVEN** the test runs as a user that is not root, and `<dir>/clients` exists with mode 000
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients`
- **AND** an API call on `<dir>/s.sock` succeeds

#### Scenario: A control character in a layer name
- **GIVEN** `<dir>/clients/devices/` holds a file whose name is `lap`, the character ESC, and `.toml`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds no ESC character, and holds the character U+FFFD

#### Scenario: A layer for an unknown kind
- **GIVEN** `<dir>/clients/kinds/tiu.toml` holds `echo = "show"`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/kinds/tiu.toml` and `tiu`

### Requirement: The layer files of a client
The layer files SHALL be `clients/global.toml`, `clients/kinds/<kind>.toml`, `clients/os/<os>.toml` and `clients/devices/<device>.toml`, and each SHALL be optional. A kind SHALL be `tui`. An OS SHALL be `macos` or `linux`. A device SHALL be a valid device name. A kind, an OS or a device with no file SHALL have no layer and SHALL NOT be an error.

#### Scenario: No clients directory
- **GIVEN** `<dir>` holds the world `alpha/aria` and no `clients` directory, and `mu server --config <dir> --socket <dir>/s.sock` serves
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `echo = "dim"` with the source `default`
- **AND** the exit status is 0

#### Scenario: A device with no file
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds `echo = "hide"`, and `mu server --config <dir> --socket <dir>/s.sock` serves
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device desk`
- **THEN** stdout holds `echo = "dim"` with the source `default`
- **AND** stderr is empty

### Requirement: Client layers merge from global to device
The client configuration of a kind, an OS and a device SHALL merge in this order: the defaults, `global.toml`, the kind, the OS and the device. A field SHALL take its value from the last layer that sets it. A table SHALL merge field by field. The table `[keys]` SHALL merge key by key, so a later layer replaces the binding of each key that it names and keeps the others. Two names of one key in two layers SHALL count as one key.

#### Scenario: The device wins
- **GIVEN** `clients/global.toml` holds `echo = "show"` and `clients/devices/laptop.toml` holds `echo = "hide"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `echo = "hide"` with the source `clients/devices/laptop.toml`

#### Scenario: Keys merge key by key
- **GIVEN** `clients/global.toml` holds `[keys]` with `"alt+n" = "focus-next"` and `"alt+p" = "focus-prev"`
- **AND** `clients/os/<os>.toml` for the OS of the test holds `[keys]` with `"alt+p" = "none"`
- **AND** a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout binds `alt+n` to `focus-next` with the source `clients/global.toml`
- **AND** stdout binds `alt+p` to `none` with the source `clients/os/<os>.toml`

#### Scenario: Two names of one key in two layers
- **GIVEN** `clients/global.toml` holds `[keys]` with `"ctrl+shift+z" = "redo"`, `clients/devices/laptop.toml` holds `[keys]` with `"shift+ctrl+z" = "none"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout binds one key, Ctrl+Shift+Z, to `none` with the source `clients/devices/laptop.toml`
- **AND** stderr is empty

### Requirement: A bad layer is skipped as a whole
The resolution SHALL skip each bad layer, as for a layer with no file. It SHALL use each good layer. No field or binding of a bad layer SHALL apply.

#### Scenario: One bad binding skips its whole layer
- **GIVEN** `clients/global.toml` holds `echo = "show"`, `clients/devices/laptop.toml` holds `echo = "hide"` and `[keys]` with `"alt+n" = "fly"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `echo = "show"` with the source `clients/global.toml`
- **AND** stdout binds no key `alt+n`

### Requirement: GetClientConfig resolves one client
The call `GetClientConfig` SHALL take a kind, an OS and a device name. It SHALL return the resolved client configuration and its revision. It SHALL return the source of each field and of each binding, and one notice for each bad layer that applies. A source SHALL be a file path relative to the configuration directory, or `default`. A notice SHALL name the file and the reason. A bad name SHALL be an error that names it.

#### Scenario: A notice for a bad layer that applies
- **GIVEN** `clients/devices/laptop.toml` holds `echo = "loud"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/devices/laptop.toml` and `echo`
- **AND** the exit status is 0

#### Scenario: A bad device name in a call
- **GIVEN** a server serves `<dir>` on `<dir>/s.sock`
- **WHEN** a client calls `GetClientConfig` with the kind `tui`, the OS `linux` and the device `My Laptop`
- **THEN** the call fails, and its error contains `My Laptop`

#### Scenario: A bad kind in a call
- **GIVEN** a server serves `<dir>` on `<dir>/s.sock`
- **WHEN** a client calls `GetClientConfig` with the kind `gui`, the OS `linux` and the device `laptop`
- **THEN** the call fails, and its error contains `gui`

#### Scenario: A bad OS in a call
- **GIVEN** a server serves `<dir>` on `<dir>/s.sock`
- **WHEN** a client calls `GetClientConfig` with the kind `tui`, the OS `windows` and the device `laptop`
- **THEN** the call fails, and its error contains `windows`

#### Scenario: No notice for a bad layer of another device
- **GIVEN** `clients/devices/laptop.toml` holds `echo = "loud"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device desk`
- **THEN** stderr is empty

### Requirement: The revision of a client configuration
The revision SHALL be a text. A change to the content of a layer that applies to the client SHALL change it. So SHALL an added or a removed layer of that client. Without such a change, the revision SHALL stay the same, also across a restart of the server. `mu config show` SHALL print the revision on its first line, as `# revision <text>`.

#### Scenario: The same files give the same revision
- **GIVEN** a server serves `<dir>` on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop` twice, with a restart of the server between the two runs
- **THEN** both first lines are the same

#### Scenario: A changed layer gives a new revision
- **GIVEN** a server serves `<dir>` on `<dir>/s.sock`, and the user ran `mu config show --server unix://<dir>/s.sock --device laptop`
- **WHEN** the user writes `echo = "hide"` to `<dir>/clients/devices/laptop.toml`, runs `mu config reload --server unix://<dir>/s.sock`, and runs the same `mu config show` again
- **THEN** the two first lines differ

#### Scenario: A layer of another device leaves the revision
- **GIVEN** a server serves `<dir>` on `<dir>/s.sock`, and the user ran `mu config show --server unix://<dir>/s.sock --device laptop`
- **WHEN** the user writes `echo = "hide"` to `<dir>/clients/devices/desk.toml`, runs `mu config reload --server unix://<dir>/s.sock`, and runs the same `mu config show` again
- **THEN** the two first lines are the same

### Requirement: mu config show prints one client configuration
`mu config show` SHALL print the configuration that `GetClientConfig` returns as TOML, with a comment on each field and each binding that names its source. The default of `--kind` SHALL be `tui`. The default of `--os` SHALL be the OS of the machine. The default of `--device` SHALL follow the requirement "The device name". It SHALL write each notice to stderr. It SHALL NOT start a server. If no server answers, it SHALL exit with status 1, and stderr SHALL name the server URL.

#### Scenario: No server
- **GIVEN** nothing serves `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock`
- **THEN** the exit status is 1, and stderr contains `<dir>/s.sock`
- **AND** no file `<dir>/s.sock` exists

#### Scenario: Another OS
- **GIVEN** `clients/os/linux.toml` holds `echo = "hide"` and `clients/os/macos.toml` holds `echo = "show"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --os linux --device laptop`
- **THEN** stdout holds `echo = "hide"` with the source `clients/os/linux.toml`

#### Scenario: An unknown OS flag
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop --os windows`
- **THEN** the exit status is 1, and stderr contains `windows`

#### Scenario: Show filters a control character in a notice
- **GIVEN** `<dir>/clients/` holds a file whose name is `glo`, the character ESC, and `bal.toml`, and a server serves `<dir>` on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds no ESC character, and holds the character U+FFFD

### Requirement: mu config reload reads the directory again
`mu config reload` SHALL ask the server to read and check the configuration directory again. The server SHALL apply the new client layers. It SHALL NOT apply a change to a world or to `server.toml`. For each such changed file, the command SHALL print one line that says that the change applies at restart. The command SHALL print one line for each bad file to stderr. If every file is good, the exit status SHALL be 0. If a file is bad, it SHALL be 1. The good layers SHALL apply in both cases.

#### Scenario: A reload applies a client layer
- **GIVEN** a server serves `<dir>` on `<dir>/s.sock`, and `<dir>` holds no `clients` directory
- **WHEN** the user writes `echo = "hide"` to `<dir>/clients/global.toml`, runs `mu config reload --server unix://<dir>/s.sock`, and then runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** the exit status of the reload is 0
- **AND** stdout of the show holds `echo = "hide"` with the source `clients/global.toml`

#### Scenario: A reload with a bad layer
- **GIVEN** a server serves `<dir>` on `<dir>/s.sock`
- **WHEN** the user writes `echo = "show"` to `<dir>/clients/global.toml` and `echo = "loud"` to `<dir>/clients/devices/laptop.toml`, and runs `mu config reload --server unix://<dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/devices/laptop.toml` and `echo`
- **AND** the exit status is 1
- **AND** `mu config show --server unix://<dir>/s.sock --device laptop` prints `echo = "show"` with the source `clients/global.toml`

#### Scenario: A reload does not apply a world change
- **GIVEN** a server serves `<dir>` with the world `alpha/aria` on `<dir>/s.sock`
- **WHEN** the user adds the world `beta/bryn` to `<dir>` and runs `mu config reload --server unix://<dir>/s.sock`
- **THEN** stdout holds a line that contains `characters/beta/bryn.toml` and `restart`
- **AND** an API call on `<dir>/s.sock` lists `alpha/aria` and not `beta/bryn`

#### Scenario: A reload with a bad world file
- **GIVEN** a server serves `<dir>` with the world `alpha/aria` on `<dir>/s.sock`
- **WHEN** the user writes `port = 0` to `<dir>/characters/alpha/aria.toml` and runs `mu config reload --server unix://<dir>/s.sock`
- **THEN** stderr holds a line that contains `characters/alpha/aria.toml` and `port`
- **AND** the exit status is 1
- **AND** an API call on `<dir>/s.sock` still lists `alpha/aria`

#### Scenario: A reload with a link to nothing in games
- **GIVEN** a server serves `<dir>` on `<dir>/s.sock`, and then the user adds `<dir>/games/ghost.toml` as a symbolic link to a file that does not exist and writes `echo = "hide"` to `clients/global.toml`
- **WHEN** the user runs `mu config reload --server unix://<dir>/s.sock`
- **THEN** stderr contains `games/ghost.toml`, and the exit status is 1
- **AND** `mu config show --server unix://<dir>/s.sock --device laptop` prints `echo = "hide"`

### Requirement: An attached client gets a new configuration
After a reload, the server SHALL tell each attached client whose configuration has a new revision. The client SHALL then apply the new configuration without a restart. After a gap in its stream, the client SHALL also get its configuration again. If a layer that applies to the client is bad after the reload, the client SHALL show one notice for it.

#### Scenario: A new binding works without a restart
- **GIVEN** `mu client` runs with the device `laptop` on a server with the fake worlds `alpha/aria` and `beta/bryn`, and the top row shows `[alpha/aria]`
- **WHEN** the user writes `[keys]` with `"alt+n" = "focus-next"` to `clients/devices/laptop.toml`, runs `mu config reload`, and presses Alt+N in the client
- **THEN** the top row shows `[beta/bryn]`

#### Scenario: A reload while the stream is down
- **GIVEN** `mu client` runs with the device `laptop` on a server with the fake worlds `alpha/aria` and `beta/bryn`, reads the stream through a relay, and the top row shows `[alpha/aria]`
- **WHEN** the relay cuts the connection and holds, the user writes `[keys]` with `"alt+n" = "focus-next"` to `clients/devices/laptop.toml` and runs `mu config reload`, and then the relay releases
- **AND** the top row no longer shows `disconnected`, and the user presses Alt+N
- **THEN** the top row shows `[beta/bryn]`

#### Scenario: A reload across a server restart
- **GIVEN** `mu client` runs with the device `laptop` on a server with the fake worlds `alpha/aria` and `beta/bryn`, and the top row shows `[alpha/aria]`
- **WHEN** the server stops, the user writes `[keys]` with `"alt+n" = "focus-next"` to `clients/devices/laptop.toml`, and a new server starts on the same socket with the same directory
- **AND** the top row no longer shows `disconnected`, and the user presses Alt+N
- **THEN** the top row shows `[beta/bryn]`

#### Scenario: A bad layer after a reload
- **GIVEN** `mu client` runs with the device `laptop`, and `clients/devices/laptop.toml` holds `echo = "hide"`
- **WHEN** the user writes `echo = "loud"` to that file and runs `mu config reload`
- **THEN** the screen shows a notice with `clients/devices/laptop.toml` and `echo`
- **AND** after the user types `hello` and presses Enter, the cells of `> hello` are dim

### Requirement: A client gets the notices of its bad layers at attach
When `mu client` attaches, it SHALL show one notice for each bad layer that applies to it, and SHALL run with the good layers.

#### Scenario: A bad device layer at attach
- **GIVEN** `clients/global.toml` holds `echo = "show"`, `clients/devices/laptop.toml` holds `echo = "loud"`, and a server serves that directory with the fake world `alpha/aria`
- **WHEN** the user starts `mu client --device laptop`, types `hello` and presses Enter
- **THEN** the screen shows a notice with `clients/devices/laptop.toml` and `echo`
- **AND** the screen shows `> hello`, and its cells are not dim
