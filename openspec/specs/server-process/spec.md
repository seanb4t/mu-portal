# server-process Specification

## Purpose
Defines the life of the `mu` server process: where it listens, how one server owns a socket, how it stops, and how a client starts one.

## Requirements

### Requirement: The server listens on a Unix socket
`mu server` SHALL serve the API on one Unix socket. The flag `--socket <path>` and the environment variable `MU_SOCKET` name the socket, and the flag wins. An empty value of `MU_SOCKET` SHALL count as unset. Without either, the socket is `server.sock` in the directory `mu` under `XDG_RUNTIME_DIR`. If `XDG_RUNTIME_DIR` is not set, empty or relative, the directory is `mu-<uid>` under the temporary directory of the user. A relative socket path SHALL be relative to the working directory. The server SHALL write one line to stderr that names the absolute socket path.

#### Scenario: The server serves the API on the named socket
- **WHEN** the user runs `mu server --socket <dir>/s.sock`
- **THEN** stderr gets a line that contains `<dir>/s.sock`
- **AND** stderr does not contain `not implemented`
- **AND** an API call to list the worlds on `<dir>/s.sock` succeeds

#### Scenario: The environment names the socket
- **GIVEN** `MU_SOCKET` is `<dir>/env.sock` and no `--socket` flag is given
- **WHEN** the user runs `mu server`
- **THEN** an API call to list the worlds on `<dir>/env.sock` succeeds

#### Scenario: The flag wins over the environment
- **GIVEN** `MU_SOCKET` is `<dir>/env.sock`
- **WHEN** the user runs `mu server --socket <dir>/flag.sock`
- **THEN** an API call on `<dir>/flag.sock` succeeds
- **AND** no file `<dir>/env.sock` exists

#### Scenario: The default socket under the runtime directory
- **GIVEN** `XDG_RUNTIME_DIR` is `<dir>`, and neither `MU_SOCKET` nor `--socket` is given
- **WHEN** the user runs `mu server`
- **THEN** an API call on `<dir>/mu/server.sock` succeeds

#### Scenario: An empty socket variable
- **GIVEN** `MU_SOCKET` is set to an empty value, and `XDG_RUNTIME_DIR` is `<dir>`
- **WHEN** the user runs `mu server`
- **THEN** an API call on `<dir>/mu/server.sock` succeeds

#### Scenario: An empty runtime directory
- **GIVEN** `XDG_RUNTIME_DIR` is set to an empty value, and the working directory is `<dir>`
- **WHEN** the user runs `mu server`
- **THEN** stderr gets a line that names an absolute socket path under the temporary directory of the user
- **AND** no directory `<dir>/mu` exists

#### Scenario: A bare socket name
- **GIVEN** the working directory is `<dir>`, a private directory
- **WHEN** the user runs `mu server --socket s.sock`
- **THEN** stderr gets a line that names `<dir>/s.sock`
- **AND** an API call on `<dir>/s.sock` succeeds

### Requirement: Only the owner can reach the socket
The server and the client SHALL create a missing socket directory with mode 0700. The server SHALL create the socket with mode 0600. The server SHALL refuse a socket directory that another user owns, or whose mode gives access to the group or to others. For such a directory, the server SHALL write an error to stderr that names the directory. The exit status SHALL be 1.

#### Scenario: New directory and socket are private
- **GIVEN** the directory `<dir>/run` does not exist
- **WHEN** the user runs `mu server --socket <dir>/run/s.sock`
- **THEN** the mode of `<dir>/run` is 0700
- **AND** the mode of `<dir>/run/s.sock` is 0600

#### Scenario: A directory of another user is refused
- **GIVEN** the directory `<dir>/run` has mode 0700, and the check expects the user id of its owner plus 1
- **WHEN** the private directory check runs for `<dir>/run`
- **THEN** the check fails with an error that names `<dir>/run`

#### Scenario: An open directory is refused
- **GIVEN** the directory `<dir>/run` exists with mode 0755
- **WHEN** the user runs `mu server --socket <dir>/run/s.sock`
- **THEN** stderr contains `<dir>/run`
- **AND** the exit status is 1
- **AND** no file `<dir>/run/s.sock` exists

### Requirement: One server for each socket
Only one server SHALL serve a socket at a time. A second server for a socket that a running server serves SHALL write an error to stderr that names the socket. The exit status of the second server SHALL be 1. The first server SHALL keep serving. If a socket file exists at the path and no server serves it, the new server SHALL replace it and serve. A file at the path can be something other than a socket. Then the server SHALL keep it, write an error that names the path, and exit with status 1.

#### Scenario: A second server is refused
- **GIVEN** a server serves `<dir>/s.sock`
- **WHEN** the user runs `mu server --socket <dir>/s.sock` again
- **THEN** the second process writes a line to stderr that contains `<dir>/s.sock` and the text `already`
- **AND** the second process exits with status 1
- **AND** an API call on `<dir>/s.sock` still succeeds

#### Scenario: A stale socket file is replaced
- **GIVEN** `<dir>/s.sock` is a socket file that no process serves
- **WHEN** the user runs `mu server --socket <dir>/s.sock`
- **THEN** an API call on `<dir>/s.sock` succeeds

#### Scenario: A file that is not a socket is kept
- **GIVEN** `<dir>/s.sock` is a regular file that holds the text `keep`
- **WHEN** the user runs `mu server --socket <dir>/s.sock`
- **THEN** stderr contains `<dir>/s.sock`
- **AND** the exit status is 1
- **AND** `<dir>/s.sock` still holds the text `keep`

### Requirement: The server stops on a signal
If the server gets SIGTERM or SIGINT, it SHALL close its connections to the worlds first, so each open event stream gets the disconnected event of each world. From then on it SHALL refuse each connect, from the API and from a handler. Then it SHALL end each open event stream. Then it SHALL remove its socket file and exit with status 0.

#### Scenario: SIGTERM stops the server
- **GIVEN** a server serves `<dir>/s.sock`, and a client reads its event stream
- **WHEN** the server process gets SIGTERM
- **THEN** the event stream of the client ends
- **AND** the server exits with status 0
- **AND** no file `<dir>/s.sock` exists

#### Scenario: The streams get the disconnects
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, and a client reads its event stream
- **WHEN** the server process gets SIGTERM
- **THEN** the stream holds a disconnected event for `alpha/aria` and one for `beta/bryn` before it ends

#### Scenario: No world connects during the stop
- **GIVEN** a server with the fake world `alpha/aria` and a handler that connects its world on each disconnect
- **WHEN** the server stops
- **THEN** `alpha/aria` stays disconnected, and the journal holds one connected event for it
- **AND** a `Connect` call for `alpha/aria` fails, and the journal gets no event

#### Scenario: SIGINT stops the server
- **GIVEN** a server serves `<dir>/s.sock`
- **WHEN** the server process gets SIGINT
- **THEN** the server exits with status 0
- **AND** no file `<dir>/s.sock` exists

### Requirement: The client starts a server
If nothing listens on the socket of `mu client`, the client SHALL start `mu server` for that socket and then attach to it. Nothing listens if no file is at the socket path, or if the connection is refused. Any other error of the first list of the worlds SHALL stop the client with exit status 1, and stderr SHALL name the socket and the error. The client SHALL wait for the answer to its first list at most the time of `connect_wait_seconds` in the `[server]` table of `client.toml`, and SHALL wait for a server that it started at most the same time. A message about a server that did not answer SHALL name that time. The client SHALL pass `--config` to the new server only if the user named a directory with `--config` or `MU_CONFIG`. The new server SHALL run in the background, in its own process group. The background server SHALL keep running after the client exits. Its stderr SHALL go to the file `server.log` in the socket directory.

#### Scenario: A client with no server starts one
- **GIVEN** the directory `<dir>/run` does not exist
- **WHEN** the user runs `mu client --socket <dir>/run/s.sock` in a terminal and then quits the client
- **THEN** an API call on `<dir>/run/s.sock` succeeds after the client exits
- **AND** the mode of `<dir>/run` is 0700
- **AND** the file `<dir>/run/server.log` exists

#### Scenario: No server answers in time
- **GIVEN** a test holds the lock `<dir>/s.sock.lock`, and nothing serves `<dir>/s.sock`
- **WHEN** the user runs `mu client --socket <dir>/s.sock` in a terminal
- **THEN** after at most 5 seconds stderr contains `<dir>/s.sock` and `server.log`
- **AND** the exit status is 1

#### Scenario: Two clients start at the same time
- **GIVEN** no server serves `<dir>/s.sock`
- **WHEN** two clients start with `--socket <dir>/s.sock` at the same time
- **THEN** the screen of each client shows `no worlds` in the top row
- **AND** `<dir>/server.log` holds exactly one line that names `<dir>/s.sock` as the socket it serves

#### Scenario: A server that closes the connection
- **GIVEN** a test listens on `<dir>/s.sock`, and closes each connection as soon as it accepts it
- **WHEN** the user runs `mu client --socket <dir>/s.sock` in a terminal
- **THEN** the exit status is 1, and stderr contains `<dir>/s.sock`
- **AND** no file `<dir>/server.log` exists

#### Scenario: A server that does not answer
- **GIVEN** `<dir>/conf/client.toml` holds `[server]` with `connect_wait_seconds = 0.2`, and a test listens on `<dir>/s.sock`, accepts each connection and never answers
- **WHEN** the user runs `mu client --config <dir>/conf --socket <dir>/s.sock` in a terminal
- **THEN** the exit status is 1, and stderr contains `<dir>/s.sock` and `0.2`

#### Scenario: The started server finds the default directory
- **GIVEN** `XDG_CONFIG_HOME` is `<dir>/xdg`, `<dir>/xdg/mu` does not exist, neither `MU_CONFIG` nor `--config` is given, and nothing serves `<dir>/run/s.sock`
- **WHEN** the user runs `mu client --socket <dir>/run/s.sock` in a terminal and then quits the client
- **THEN** `<dir>/run/server.log` holds no line that contains `<dir>/xdg`

### Requirement: The client needs a terminal
If stdin or stdout of `mu client` is not a terminal, the client SHALL start no server. It SHALL write an error to stderr that contains the text `terminal`, and exit with status 1.

#### Scenario: No terminal
- **GIVEN** `<dir>` is an empty directory, and no server serves `<dir>/s.sock`
- **WHEN** the user runs `mu client --socket <dir>/s.sock` with stdin on a terminal and stdout as a pipe
- **THEN** stderr contains the text `terminal`
- **AND** stderr does not contain `not implemented`
- **AND** the exit status is 1
- **AND** the directory `<dir>` is still empty

#### Scenario: No terminal on stdin
- **GIVEN** `<dir>` is an empty directory
- **WHEN** the user runs `mu client --socket <dir>/s.sock` with stdout on a terminal and stdin as a pipe
- **THEN** stderr contains the text `terminal`
- **AND** the exit status is 1
- **AND** the directory `<dir>` is still empty

### Requirement: The server reports its configuration directory
`ListWorlds` SHALL return `config_dir`: the configuration directory that the server read its worlds from, as an absolute path with each symbolic link resolved. A server with no directory SHALL return no value.

#### Scenario: The server names its directory
- **GIVEN** `<dir>/link` is a symbolic link to `<dir>/a`, and `mu server --config <dir>/link --socket <dir>/s.sock` serves the worlds of `<dir>/a`
- **WHEN** a client calls `ListWorlds`
- **THEN** `config_dir` is the absolute path of `<dir>/a`

### Requirement: The client notes a server with another directory
If `mu client` attaches to a server that it did not start, and the `config_dir` of that server differs from the directory of the client, resolved the same way, the client SHALL still attach and SHALL show a notice line on the screen. The notice SHALL name the socket and both directories, or `no directory` for a side that has none.

#### Scenario: A server with another directory
- **GIVEN** `mu server --config <dir>/a --socket <dir>/s.sock --fake-world alpha/aria=env:PW` serves
- **WHEN** the user runs `mu client --config <dir>/b --socket <dir>/s.sock` in a terminal
- **THEN** the top row shows `alpha/aria`
- **AND** the screen shows a notice with `reads its worlds from`, the resolved path of `<dir>/a` and `<dir>/b`

#### Scenario: A server with the same directory
- **GIVEN** `mu server --config <dir>/a --socket <dir>/s.sock --fake-world alpha/aria=env:PW` serves, and `<dir>/link` is a symbolic link to `<dir>/a`
- **WHEN** the user runs `mu client --config <dir>/link --socket <dir>/s.sock` in a terminal
- **THEN** the top row shows `alpha/aria`
- **AND** the screen shows no notice with `reads its worlds from`

#### Scenario: Two clients with other directories start at once
- **GIVEN** nothing serves `<dir>/run/mu.sock`
- **WHEN** the user runs `mu client --config <dir>/a --socket <dir>/run/mu.sock` and `mu client --config <dir>/b --socket <dir>/run/mu.sock` at the same time, each in its own terminal
- **THEN** each top row shows `no worlds`
- **AND** exactly one of the two screens shows a notice with `reads its worlds from`
