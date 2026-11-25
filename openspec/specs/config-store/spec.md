# config-store Specification

## Purpose
Defines how the server owns the configuration directory. It covers the check of each file, the client configuration of one client, the report of a bad file, and the reload.

## Requirements

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

#### Scenario: A line break in a layer name
- **GIVEN** `<dir>/clients/devices/` holds a file whose name is `lap`, a line break, and `.toml`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds exactly one line that contains `clients/devices/lap`

#### Scenario: A line break in a binding
- **GIVEN** `<dir>/clients/global.toml` holds `[keys]` with `"alt+\n" = "quit"`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds exactly one line that contains `clients/global.toml`

#### Scenario: A link to an unreadable directory under clients
- **GIVEN** the test runs as a user that is not root, `<other>/locked` is a directory with mode 000 outside `<dir>`, and `<dir>/clients/devices/x` is a symbolic link to `<other>/locked`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds a line that contains `clients/devices/x`
- **AND** an API call on `<dir>/s.sock` succeeds

#### Scenario: A file that is not TOML
- **GIVEN** `<dir>/clients/notes.txt` exists
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/notes.txt`
- **AND** an API call on `<dir>/s.sock` succeeds

#### Scenario: A loop of links under clients
- **GIVEN** `<dir>/clients/devices/a` and `<dir>/clients/devices/b` are symbolic links to `<dir>/clients/devices`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds a line that contains `clients/devices/a`
- **AND** an API call on `<dir>/s.sock` succeeds

#### Scenario: A clients directory in another case
- **GIVEN** the file system of `<dir>` ignores case, and `<dir>/Clients/global.toml` holds `echo = "hide"`
- **WHEN** a server serves `<dir>` on `<dir>/s.sock`, and the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `echo = "hide"`

#### Scenario: A layer for an unknown OS
- **GIVEN** `<dir>/clients/os/windows.toml` holds `echo = "show"`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/os/windows.toml` and `windows`

#### Scenario: A FIFO under clients
- **GIVEN** `<dir>/clients/devices/laptop.toml` is a FIFO with no writer
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/devices/laptop.toml`
- **AND** an API call on `<dir>/s.sock` succeeds

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

#### Scenario: A layer without echo keeps the earlier echo
- **GIVEN** `clients/global.toml` holds `echo = "hide"`, `clients/devices/laptop.toml` holds `[keys]` with `"alt+n" = "focus-next"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `echo = "hide"` with the source `clients/global.toml`

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
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds `echo = "hide"`, and a server serves `<dir>` on `<dir>/s.sock`
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
- **GIVEN** `<dir>/clients/global.toml` holds `[keys]` with `"alt+\u001b" = "quit"`, and a server serves `<dir>` on `<dir>/s.sock`
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
- **WHEN** the user writes `extends = "tinymux"`, `host = "localhost"` and `port = 4201` to `<dir>/games/beta.toml`, writes an empty `<dir>/characters/beta/bryn.toml`, and runs `mu config reload --server unix://<dir>/s.sock`
- **THEN** stdout holds a line that contains `characters/beta/bryn.toml` and `restart`
- **AND** the exit status is 0
- **AND** an API call on `<dir>/s.sock` lists `alpha/aria` and not `beta/bryn`

#### Scenario: A reload with two bad world files
- **GIVEN** a server serves `<dir>` with the worlds `alpha/aria` and `beta/bryn` on `<dir>/s.sock`
- **WHEN** the user writes `port = 0` to `<dir>/characters/alpha/aria.toml` and to `<dir>/characters/beta/bryn.toml`, and runs `mu config reload --server unix://<dir>/s.sock`
- **THEN** stderr holds a line that contains `characters/alpha/aria.toml` and a line that contains `characters/beta/bryn.toml`
- **AND** the exit status is 1

#### Scenario: A reload with a bad game of two worlds
- **GIVEN** a server serves `<dir>` with the worlds `alpha/aria` and `alpha/ann` on `<dir>/s.sock`, both of the game `alpha`
- **WHEN** the user writes `port = 0` to `<dir>/games/alpha.toml` and runs `mu config reload --server unix://<dir>/s.sock`
- **THEN** stderr holds exactly one line that contains `games/alpha.toml`
- **AND** the exit status is 1

#### Scenario: A reload with two stray files
- **GIVEN** a server serves `<dir>` with the world `alpha/aria` on `<dir>/s.sock`
- **WHEN** the user creates `<dir>/worlds.toml` and `<dir>/games/Beta.toml`, and runs `mu config reload --server unix://<dir>/s.sock`
- **THEN** stderr holds a line that contains `worlds.toml` and a line that contains `games/Beta.toml`
- **AND** the exit status is 1

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

#### Scenario: A reload lists a changed server file
- **GIVEN** a server serves `<dir>` on `<dir>/s.sock`, and `<dir>/server.toml` is empty
- **WHEN** the user writes `[telemetry]` to `<dir>/server.toml` and runs `mu config reload --server unix://<dir>/s.sock`
- **THEN** stdout holds a line that contains `server.toml` and `restart`
- **AND** the exit status is 0

#### Scenario: A second reload lists the world change again
- **GIVEN** a server serves `<dir>` with the world `alpha/aria` on `<dir>/s.sock`
- **AND** after the server started, `<dir>/games/beta.toml` got `extends = "tinymux"`, `host = "localhost"` and `port = 4201`, `<dir>/characters/beta/bryn.toml` was written empty, and `mu config reload --server unix://<dir>/s.sock` ran once
- **WHEN** the user runs `mu config reload --server unix://<dir>/s.sock` again
- **THEN** stdout holds a line that contains `characters/beta/bryn.toml` and `restart`

#### Scenario: A reload with no server
- **GIVEN** nothing serves `<dir>/s.sock`
- **WHEN** the user runs `mu config reload --server unix://<dir>/s.sock`
- **THEN** the exit status is 1, and stderr contains `<dir>/s.sock`
- **AND** no file `<dir>/server.log` exists

### Requirement: The server keeps and writes the result of a reload
If the configuration directory does not exist at a reload, the server SHALL keep the current client layers, and `mu config reload` SHALL print a line to stderr that names the directory and exit with status 1. The server SHALL write each notice of a reload to its own stderr.

#### Scenario: A reload of a removed directory
- **GIVEN** `<dir>/clients/global.toml` holds `echo = "show"`, and a server serves `<dir>`
- **WHEN** the user moves `<dir>` away and runs `mu config reload`, and then runs `mu config show --device laptop`
- **THEN** the exit status of the reload is 1, and stderr contains `the configuration directory does not exist`
- **AND** stdout of the show holds `echo = "show" # clients/global.toml`

#### Scenario: The server writes the notices of a reload
- **GIVEN** a server serves `<dir>`
- **WHEN** the user writes `echo = "loud"` to `<dir>/clients/devices/laptop.toml` and runs `mu config reload`
- **THEN** stderr of the server gets a line with `clients/devices/laptop.toml` and `echo`

### Requirement: mu config reaches only a private socket
`mu config show` and `mu config reload` SHALL refuse a socket directory that exists and that another user owns, or whose mode gives access to the group or to others. The command SHALL write an error to stderr that names the directory and the mode, print nothing to stdout, and exit with status 1. Each command SHALL write the text that the server sends to stdout with each control character, other than the line break and the tab, as U+FFFD.

#### Scenario: An open socket directory
- **GIVEN** a server serves `<dir>`, and the directory of its socket has mode 0755
- **WHEN** the user runs `mu config show --device laptop` on that socket
- **THEN** the exit status is 1, stdout is empty, and stderr names the directory of the socket and the mode 0755

#### Scenario: A control character in the reply
- **GIVEN** a reply of `GetClientConfig` whose revision holds ESC, and a reload reply whose restart path holds ESC
- **WHEN** `mu config show` and `mu config reload` render them
- **THEN** neither output holds ESC, and the restart output is one line

### Requirement: An attached client gets a new configuration
After a reload that changes `clients/`, the server SHALL send a new store hash to each watching client. The server SHALL keep no state about a client. A client SHALL fetch its configuration when the hash differs, and SHALL apply a new revision without a restart. After a gap in its watch stream, the client SHALL compare the hash again. If a layer that applies to the client is bad after the reload, the client SHALL show one notice for it.

#### Scenario: A new binding works without a restart
- **GIVEN** `mu client` runs with the device `laptop` on a server with the fake worlds `alpha/aria` and `beta/bryn`, and the top row shows `[alpha/aria]`
- **WHEN** the user writes `[keys]` with `"alt+n" = "focus-next"` to `clients/devices/laptop.toml`, runs `mu config reload`, and presses Alt+N in the client
- **THEN** the top row shows `[beta/bryn]`

#### Scenario: A reload while the stream is down
- **GIVEN** `mu client` runs with the device `laptop` on a server with the fake worlds `alpha/aria` and `beta/bryn`, connects through a relay, and the top row shows `[alpha/aria]`
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

#### Scenario: A clients directory that turns unlistable
- **GIVEN** the test runs as a user that is not root, and `mu client` runs with the device `laptop` on a server that serves `<dir>`
- **WHEN** the user sets the mode of `<dir>/clients` to 000 and runs `mu config reload`
- **THEN** the screen shows a notice that contains `clients`

### Requirement: A failed fetch is tried again
If the fetch of the configuration fails while the store hash still differs, the client SHALL show one notice and SHALL fetch again after a wait that grows with each failure.

#### Scenario: A failed fetch is tried again
- **GIVEN** an attached client got a store hash that differs from the hash of its configuration
- **WHEN** the fetch of the configuration fails
- **THEN** the client shows one notice, waits 100 ms, and fetches again
- **AND** each later failure waits longer, and an applied configuration starts the waits again

### Requirement: A client gets the notices of its bad layers at attach
When `mu client` attaches, it SHALL show one notice for each bad layer that applies to it, and SHALL run with the good layers.

#### Scenario: A bad device layer at attach
- **GIVEN** `clients/global.toml` holds `echo = "show"`, `clients/devices/laptop.toml` holds `echo = "loud"`, and a server serves that directory with the fake world `alpha/aria`
- **WHEN** the user starts `mu client --device laptop`, types `hello` and presses Enter
- **THEN** the screen shows a notice with `clients/devices/laptop.toml` and `echo`
- **AND** the screen shows `> hello`, and its cells are not dim

### Requirement: The layout files
A layout file SHALL be `clients/layouts/<name>.toml`, where the name follows the rule of a device name. It SHALL hold only the field `place`, a list of placement sentences. The server SHALL check each layout file at start and at reload. A bad layout file SHALL be skipped, with one notice that names the file and the reason. A file with the name of a built-in layout SHALL be bad.

#### Scenario: A layout file in use
- **GIVEN** `clients/layouts/mine.toml` holds `place` with the lines `output` and `editor below main`
- **AND** `clients/global.toml` holds `layout = "mine"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `layout = "mine"` with the source `clients/global.toml`
- **AND** stderr is empty

#### Scenario: A bad layout file at start
- **GIVEN** `<dir>/clients/layouts/mine.toml` holds `place` with the one line `output`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `editor`
- **AND** the server serves

#### Scenario: A file with a built-in name
- **GIVEN** `<dir>/clients/layouts/classic.toml` holds `place` with the one line `editor`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/layouts/classic.toml` and `built-in`

#### Scenario: An unknown field of a layout file
- **GIVEN** `<dir>/clients/layouts/mine.toml` holds `place = ["editor"]` and `colour = "red"`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `colour`

### Requirement: The layout field names the layout in use
The field `layout` of a client layer SHALL name a built-in layout or a layout file. It SHALL merge as any other field, and its default SHALL be `classic`. `GetClientConfig` SHALL return the name, the lines of that layout, and the source of the field. If the name is of no good layout, the reply SHALL hold `classic` and a notice that names the layer and the layout.

#### Scenario: The device names another layout
- **GIVEN** `clients/global.toml` holds `layout = "status"`, and `clients/devices/laptop.toml` holds `layout = "focus"`
- **AND** a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `layout = "focus"` with the source `clients/devices/laptop.toml`

#### Scenario: A layout that does not exist
- **GIVEN** `clients/global.toml` holds `layout = "gone"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `gone`
- **AND** stdout holds the lines of `classic`

#### Scenario: A change to a layout file changes the revision
- **GIVEN** `clients/global.toml` holds `layout = "mine"`, and the user ran `mu config show --device laptop`
- **WHEN** the user adds a line to `clients/layouts/mine.toml`, runs `mu config reload`, and runs the same `mu config show` again
- **THEN** the two first lines differ

### Requirement: ListLayouts gives every layout
The call `ListLayouts` SHALL return each built-in layout and each good layout file: its name, its lines, and whether it is built in. The order SHALL be by name. The reply SHALL also hold the notice of each bad layout file.

#### Scenario: The list with one player layout
- **GIVEN** `clients/layouts/mine.toml` is a good layout file, and a server serves that directory
- **WHEN** the user runs `mu layout list`
- **THEN** stdout names `classic`, `focus`, `mine`, `sidebar` and `status` in that order
- **AND** each built-in name has the mark `built-in`

#### Scenario: The list with a bad layout file
- **GIVEN** `clients/layouts/mine.toml` holds `place` with the one line `output`, and a server serves that directory
- **WHEN** the user runs `mu layout list`
- **THEN** stdout does not name `mine`
- **AND** stderr holds one line that contains `clients/layouts/mine.toml` and `editor`

### Requirement: SaveLayout writes a layout file
The call `SaveLayout` SHALL take a name, the lines of a layout, and an optional device name. The server SHALL check the name and the lines with the rules of a layout file. If the check passes, it SHALL write `clients/layouts/<name>.toml` as a whole, so that a reader never sees a part of it. It SHALL then read the directory again, as a reload does. A failed check SHALL be an error that names the reason, and SHALL write nothing.

#### Scenario: A save writes the file
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory
- **WHEN** a client calls `SaveLayout` with the name `mine` and the lines `output` and `editor below main`
- **THEN** `<dir>/clients/layouts/mine.toml` holds those two lines in `place`
- **AND** `mu layout list` names `mine`

#### Scenario: A save of a bad layout
- **GIVEN** a server serves `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `mine` and the one line `output`
- **THEN** the call fails, and its error contains `editor`
- **AND** no file `<dir>/clients/layouts/mine.toml` exists

#### Scenario: A save with a built-in name
- **GIVEN** a server serves `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `classic` and the line `editor`
- **THEN** the call fails, and its error contains `classic`

#### Scenario: A save with a bad name
- **GIVEN** a server serves `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `../x` and the line `editor`
- **THEN** the call fails, and no file is written outside `<dir>/clients/layouts`

#### Scenario: A save with the name of two dots
- **GIVEN** a server serves `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `..` and the lines `output` and `editor below main`
- **THEN** the call fails, and its error contains `..`
- **AND** `<dir>/clients` holds no file

### Requirement: A write makes the directories that it needs
Before a write, the server SHALL make `clients`, `clients/layouts` and `clients/devices` if they are missing. A link to a directory SHALL count as a directory, and the write SHALL go through the link. If one of these paths exists and is not a directory, the call SHALL fail with an error that names the path, and SHALL write nothing.

#### Scenario: A save through a linked directory
- **GIVEN** `<dir>/clients` is a symbolic link to a directory that holds `layouts/mine.toml` with the lines `output` and `editor below main`
- **WHEN** a client calls `EditLayout` on `mine` with the place `tabs above main`
- **THEN** the call succeeds, and the file `layouts/mine.toml` of the linked directory holds the lines `output`, `editor below main` and `tabs above main`
- **AND** `<dir>/clients` is still a symbolic link

#### Scenario: A clients path that is a file
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory, and the user then makes the empty file `<dir>/clients`
- **WHEN** a client calls `SaveLayout` with the name `mine` and the lines `output` and `editor below main`
- **THEN** the call fails, and its error contains `clients`
- **AND** `<dir>/clients` is still an empty file

### Requirement: A write checks first, and keeps links and permissions
The server SHALL make each check of a write call before its first write, so a refused call writes no file and makes no directory. It SHALL store the lines of a layout as mu writes them. A write to a symbolic link to a regular file SHALL replace that file and keep the link. A link to anything else SHALL be an error. A write SHALL keep the permissions of a file that it replaces. An error of a write SHALL hold no line of a file.

#### Scenario: A refused save writes no layout file
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds `echo = `, and a server serves `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `mine`, the line `editor` and the device `laptop`
- **THEN** the call fails, and `<dir>/clients/layouts/mine.toml` does not exist

#### Scenario: A refused edit of a built-in layout writes no layout file
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds `echo = `, and a server serves `<dir>`
- **WHEN** a client calls `EditLayout` on `classic` with the hide of `tabs` and the device `laptop`
- **THEN** the call fails, and `<dir>/clients/layouts/laptop.toml` does not exist

#### Scenario: A write to a linked device file
- **GIVEN** `<dir>/clients/devices/laptop.toml` is a symbolic link to a file that holds `# shared` and `echo = "show"`
- **WHEN** a client calls `UseLayout` with the device `laptop` and the layout `status`
- **THEN** the path is still a symbolic link, and the linked file holds `# shared` and `layout = "status"`

#### Scenario: A link to a directory
- **GIVEN** `<dir>/clients/layouts/mine.toml` is a symbolic link to a directory outside `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `mine` and the line `editor`
- **THEN** the call fails, its error contains `clients/layouts/mine.toml` and `no regular file`, and holds no path outside `<dir>`
- **AND** the path is still a symbolic link, and the directory holds what it held

#### Scenario: A link to nothing
- **GIVEN** `<dir>/clients/layouts/mine.toml` is a symbolic link to a file that does not exist
- **WHEN** a client calls `SaveLayout` with the name `mine` and the line `editor`
- **THEN** the call fails, its error contains `a link to nothing`, and the file still does not exist

#### Scenario: A refused call makes no directory
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory
- **WHEN** a client calls `UseLayout` with the device `laptop`, an empty layout name and the key `alpha/ari`
- **THEN** the call fails, and `<dir>/clients` does not exist

#### Scenario: A save that makes a good layer bad
- **GIVEN** the text of `<dir>/clients/devices/laptop.toml` changed after the server read it, so that the edit of the field `layout` gives a bad layer
- **WHEN** a client calls `SaveLayout` with the name `mine`, the line `editor` and the device `laptop`
- **THEN** the call fails, and `<dir>/clients/layouts/mine.toml` does not exist

#### Scenario: A write keeps the permissions
- **GIVEN** `<dir>/clients/devices/laptop.toml` has the mode 0640
- **WHEN** a client calls `UseLayout` with the device `laptop` and the layout `status`
- **THEN** the file has the mode 0640

#### Scenario: An error holds no line of the file
- **GIVEN** the text of a layer is `echo = "show"` and then the line `password hunter2`
- **WHEN** the server edits the field `layout` of that text
- **THEN** the error holds `line 2`, and does not hold `hunter2`

#### Scenario: A save stores the lines as mu writes them
- **GIVEN** a server serves an empty configuration directory
- **WHEN** a client calls `SaveLayout` with the lines `  output  ` and `editor   below   main`
- **THEN** the layout file holds the lines `output` and `editor below main`

### Requirement: A write ends with a reload if its caller goes away
A write call SHALL finish its files and the reload after it, also if its caller goes away in the middle, as a call with a deadline that ends does. So the server never serves an older state than its files hold after a write.

#### Scenario: The caller goes away
- **GIVEN** a server serves `<dir>`, and a client calls `SaveLayout` with the name `mine` and the line `editor`
- **WHEN** the client drops the call after the server started it
- **THEN** `<dir>/clients/layouts/mine.toml` exists, and the store hash of the server changed

### Requirement: SaveLayout with a key sets the entry of a world
The call `SaveLayout` SHALL also take an optional key, which is a world id or the name of a game. With a device name and a key, the server SHALL set the entry of that key in `[world-layouts]` of the device layer, in the same write as the layout file, and SHALL NOT set the field `layout`. A key with no device name SHALL be an error, and SHALL write nothing.

#### Scenario: A save with a key
- **GIVEN** a server serves an empty configuration directory `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `mine`, the lines `output` and `editor below main`, the device `laptop` and the key `alpha/aria`
- **THEN** `<dir>/clients/layouts/mine.toml` holds the two lines
- **AND** `<dir>/clients/devices/laptop.toml` holds the entry `"alpha/aria" = "mine"` in `[world-layouts]`, and no field `layout`

#### Scenario: A save with a key and no device
- **GIVEN** a server serves an empty configuration directory `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `mine`, the line `editor`, no device and the key `alpha/aria`
- **THEN** the call fails, its error contains `device`, and `<dir>/clients/layouts/mine.toml` does not exist

### Requirement: UseLayout sets the layout of a device
The call `UseLayout` SHALL take a device name and the name of a layout. `SaveLayout` with a device name SHALL do the same in the same write as the layout file. The server SHALL set the field `layout` in `clients/devices/<device>.toml`, and SHALL keep each other line of the file, its comments included. With no such file, it SHALL make one. It SHALL then read the directory again.

#### Scenario: A device file with a comment
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds the comment `# my laptop` and `echo = "show"`, and a server serves `<dir>`
- **WHEN** a client calls `UseLayout` with the device `laptop` and the layout `status`
- **THEN** the file holds `# my laptop`, `echo = "show"` and `layout = "status"`

#### Scenario: No device file
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory
- **WHEN** the user runs `mu layout use focus --device laptop`
- **THEN** `<dir>/clients/devices/laptop.toml` holds `layout = "focus"`

### Requirement: UseLayout refuses what it cannot apply
An unknown layout SHALL be an error that names it, and SHALL write nothing. So SHALL a bad device name, and a device file that is a bad layer.

#### Scenario: An unknown layout
- **GIVEN** a server serves `<dir>`
- **WHEN** the user runs `mu layout use gone --device laptop`
- **THEN** the exit status is 1, and stderr contains `gone`
- **AND** no file `<dir>/clients/devices/laptop.toml` exists

#### Scenario: A bad device layer
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds `echo = `, and a server serves `<dir>`
- **WHEN** a client calls `UseLayout` with the device `laptop` and the layout `status`
- **THEN** the call fails, and its error contains `clients/devices/laptop.toml`
- **AND** the file still holds `echo = `

### Requirement: An attached client gets a saved layout
After a `SaveLayout` or a `UseLayout`, the watch of the configuration SHALL send a new hash, as after a reload. Each attached client whose layout changed SHALL draw the new layout.

#### Scenario: A use from the command line
- **GIVEN** `mu client --device laptop` shows the layout `classic`
- **WHEN** the user runs `mu layout use status --device laptop` in another terminal
- **THEN** the bottom row of the client is the statusline

### Requirement: mu layout prints layouts
`mu layout list` SHALL print each layout of `ListLayouts` with a picture of its panes, each pane with the name of its block. `mu layout show <name>` SHALL print the numbered lines of one layout and its picture. If the name is of a bad layout file, it SHALL write the notice of the file to stderr and exit with status 1.

#### Scenario: The picture of classic
- **GIVEN** a server serves `<dir>`
- **WHEN** the user runs `mu layout show classic`
- **THEN** stdout holds the three lines of `classic`
- **AND** stdout holds a picture with the words `tabs`, `main` and `editor` from top to bottom

### Requirement: mu layout reaches only a private socket
`mu layout use <name>` SHALL call `UseLayout` with the device name of the requirement "The device name". Each `mu layout` command SHALL follow the requirement "mu config reaches only a private socket". No `mu layout` command SHALL start a server.

#### Scenario: An open socket directory
- **GIVEN** a server serves `<dir>`, and the directory of its socket has mode 0755
- **WHEN** the user runs `mu layout list` on that socket
- **THEN** the exit status is 1, and stdout is empty

#### Scenario: No server
- **GIVEN** nothing serves `<dir>/s.sock`
- **WHEN** the user runs `mu layout list --server unix://<dir>/s.sock`
- **THEN** the exit status is 1, and no file `<dir>/s.sock` exists

### Requirement: EditLayout changes one line of a layout
The call `EditLayout` SHALL take the name of a layout, one change, and an optional device name. A change SHALL be one of these: place a block, hide a block, set the size of a block, insert a line, replace a line, or delete a line. The server SHALL apply the change with the rules of the client commands of the same names, and save the result as `SaveLayout` does. The reply SHALL hold the new plan, and SHALL say whether mu wrote the whole plan again.

#### Scenario: An edit places a block
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output` and `editor below main`, and a server serves that directory
- **WHEN** the user runs `mu layout place mine "output as notices right of main, 30%"`
- **THEN** `mu layout show mine` holds three lines, and the last is `3. output as notices right of main, 30%`

#### Scenario: An edit gives the same file as the command
- **GIVEN** two servers, each with the layout file `mine` of the lines `output`, `tabs above main` and `editor below main`
- **WHEN** a client of the first server runs the command `/pane hide tabs`, and the user runs `mu layout hide mine tabs` on the second server
- **THEN** the two files `clients/layouts/mine.toml` hold the same lines

#### Scenario: An insert from the command line
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output` and `editor below main`, and a server serves that directory
- **WHEN** the user runs `mu layout insert mine 2 "output as notices right of main"`
- **THEN** `mu layout show mine` holds `2. output as notices right of main` and `3. editor below main`

#### Scenario: A delete by id
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output`, `editor below main` and `output as side right of main`
- **WHEN** the user runs `mu layout delete mine side`
- **THEN** `mu layout show mine` holds two lines

### Requirement: EditLayout refuses what it cannot save
For a built-in layout, `EditLayout` SHALL need the device name, and SHALL save under that name. A change that the layout rules refuse SHALL be an error with the reason, and SHALL write nothing.

#### Scenario: An edit of a built-in layout
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory
- **WHEN** the user runs `mu layout hide classic tabs --device laptop`
- **THEN** `<dir>/clients/layouts/laptop.toml` holds the lines `output` and `editor below main`
- **AND** `mu config show --device laptop` holds `layout = "laptop"`

#### Scenario: A bad edit
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output` and `editor below main`, and a server serves that directory
- **WHEN** the user runs `mu layout hide mine editor`
- **THEN** the exit status is 1, and stderr contains `editor`
- **AND** the file still holds two lines

### Requirement: DescribeLayout gives the panes of a layout
The call `DescribeLayout` SHALL take the name of a layout, a width and a height. It SHALL return one entry for each block of the layout: its id, its name, the number of its line, the rectangle that it takes on a screen of that size, whether it is in front of its stack, and a text that names its data source. `mu layout describe <name>` SHALL print the entries, and with `--json` it SHALL print them as one JSON document.

#### Scenario: Describe gives each block
- **GIVEN** a server serves `<dir>`
- **WHEN** the user runs `mu layout describe sidebar --width 80 --height 24 --json`
- **THEN** stdout is JSON with five entries, one for each of `tabs`, `output`, `notices`, `editor` and `statusline`
- **AND** each entry has an id and a line number
- **AND** the entry of `notices` has the width 24, and the entry of `tabs` has the width 80 and the height 1

#### Scenario: Describe of an unknown layout
- **GIVEN** a server serves `<dir>`
- **WHEN** the user runs `mu layout describe gone`
- **THEN** the exit status is 1, and stderr contains `gone`

### Requirement: The world layouts table names a layout for a world
The table `[world-layouts]` of a client layer SHALL map a world id or the name of a game to the name of a layout. An entry SHALL replace the entry of an earlier layer with the same key, and the other entries SHALL stay. `GetClientConfig` SHALL return each entry with its key, the name and the lines of its layout, and its source. An entry for a world that the server does not have SHALL give no notice.

#### Scenario: A device adds an entry
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `alpha = "focus"`
- **AND** `clients/devices/laptop.toml` holds `[world-layouts]` with `"alpha/aria" = "status"`
- **AND** a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `world-layouts.alpha = "focus"` with the source `clients/global.toml`
- **AND** stdout holds `world-layouts."alpha/aria" = "status"` with the source `clients/devices/laptop.toml`

#### Scenario: A device replaces an entry
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `alpha = "focus"`, and `clients/devices/laptop.toml` holds `[world-layouts]` with `alpha = "status"`
- **AND** a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `world-layouts.alpha = "status"` with the source `clients/devices/laptop.toml`
- **AND** stdout does not hold `world-layouts.alpha = "focus"`

#### Scenario: An entry for a world that the server does not have
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `"zeta/zed" = "focus"`, and a server with no world serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr is empty

#### Scenario: A change to the layout file of an entry changes the revision
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `alpha = "mine"`, and the user ran `mu config show --device laptop`
- **WHEN** the user adds a line to `clients/layouts/mine.toml`, runs `mu config reload`, and runs the same `mu config show` again
- **THEN** the two first lines differ

### Requirement: A bad world layouts entry
A key of `[world-layouts]` SHALL be a world id or the name of a game. Any other key SHALL make its layer bad, and the notice SHALL name the file and the key. An entry whose name is of no good layout SHALL NOT be in the reply of `GetClientConfig`. The reply SHALL hold a notice that names the layer, the key and the layout.

#### Scenario: A key with two slashes
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `"a/b/c" = "focus"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `a/b/c`

#### Scenario: An entry with a layout that does not exist
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `alpha = "gone"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml`, `alpha` and `gone`
- **AND** stdout does not hold `world-layouts.alpha`

### Requirement: UseLayout can set the layout of one world
`UseLayout` SHALL also take a world id or the name of a game. The server SHALL then set that entry of `[world-layouts]` in the device file, and SHALL NOT change the field `layout`. With an empty layout name, it SHALL remove the entry. `mu layout use <name>` with `--world <world>` or `--game <game>` SHALL set the entry. `mu layout forget` with one of the two flags SHALL remove it. A bad key, and a removal of an entry that the device file does not hold, SHALL be an error, and SHALL write nothing.

#### Scenario: A use for one world
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds the comment `# my laptop` and `layout = "status"`, and a server serves `<dir>`
- **WHEN** the user runs `mu layout use focus --world alpha/aria --device laptop`
- **THEN** the file holds `# my laptop`, `layout = "status"`, and `"alpha/aria" = "focus"` in the table `[world-layouts]`

#### Scenario: A use for a game
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory
- **WHEN** the user runs `mu layout use focus --game alpha --device laptop`
- **THEN** `mu config show --device laptop` holds `world-layouts.alpha = "focus"` with the source `clients/devices/laptop.toml`
- **AND** it holds `layout = "classic"` with the source `default`

#### Scenario: Forget the layout of a world
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds `[world-layouts]` with `"alpha/aria" = "focus"` and `alpha = "status"`, and a server serves `<dir>`
- **WHEN** the user runs `mu layout forget --world alpha/aria --device laptop`
- **THEN** the file holds `alpha = "status"`, and does not hold `alpha/aria`

#### Scenario: Forget an entry that is not there
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory
- **WHEN** the user runs `mu layout forget --world alpha/ari --device laptop`
- **THEN** the exit status is 1, stderr holds `alpha/ari`, and `<dir>/clients/devices/laptop.toml` does not exist

#### Scenario: Forget an entry that is not there, in each form of the file
- **GIVEN** the device file has no final newline, or CRLF line ends, or an empty `[world-layouts]` table, and holds no entry `alpha/aria`
- **WHEN** a client calls `UseLayout` with an empty layout name and the key `alpha/aria`
- **THEN** the call fails, and the text of the file is the same as before

#### Scenario: A use with a bad world id
- **GIVEN** a server serves `<dir>`
- **WHEN** the user runs `mu layout use focus --world a/b/c --device laptop`
- **THEN** the exit status is 1, and stderr contains `a/b/c`
- **AND** no file `<dir>/clients/devices/laptop.toml` exists

### Requirement: EditLayout of a built-in layout can name a world
`EditLayout` SHALL also take a world id. For a built-in layout, the server SHALL then save by the rules of the client for the world in focus: the same name, and the same entry or field of the device file.

#### Scenario: An edit of the built-in layout of one world
- **GIVEN** `<dir>/clients/global.toml` holds `[world-layouts]` with `"alpha/aria" = "focus"`, and a server serves `<dir>`
- **WHEN** the user runs `mu layout place focus "tabs at top" --device laptop --world alpha/aria`
- **THEN** `<dir>/clients/layouts/laptop.alpha.aria.toml` holds three lines
- **AND** `mu config show --device laptop` holds `world-layouts."alpha/aria" = "laptop.alpha.aria"` and `layout = "classic"`
