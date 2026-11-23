## MODIFIED Requirements

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

## ADDED Requirements

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
