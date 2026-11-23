# Spec Delta

## MODIFIED Requirements

### Requirement: The client starts a server
If nothing listens on the socket of the server URL of `mu client`, the client SHALL start `mu server` for that socket and then attach to it. Nothing listens if no file is at the socket path, or if the connection is refused. Any other error of the first list of the worlds SHALL stop the client with exit status 1, and stderr SHALL name the socket and the error. The client SHALL wait for the answer to its first list at most the time of `connect_wait` of the URL, and SHALL wait for a server that it started at most the same time. A message about a server that did not answer SHALL name that time. The client SHALL NOT pass `--config` to the new server, which finds its directory from the environment that it inherits. The new server SHALL run in the background, in its own process group. The background server SHALL keep running after the client exits. Its stderr SHALL go to the file `server.log` in the socket directory.

#### Scenario: A client with no server starts one
- **GIVEN** the directory `<dir>/run` does not exist
- **WHEN** the user runs `mu client --server unix://<dir>/run/s.sock` in a terminal and then quits the client
- **THEN** an API call on `<dir>/run/s.sock` succeeds after the client exits
- **AND** the mode of `<dir>/run` is 0700
- **AND** the file `<dir>/run/server.log` exists

#### Scenario: No server answers in time
- **GIVEN** a test holds the lock `<dir>/s.sock.lock`, and nothing serves `<dir>/s.sock`
- **WHEN** the user runs `mu client --server unix://<dir>/s.sock` in a terminal
- **THEN** after at most 5 seconds stderr contains `<dir>/s.sock` and `server.log`
- **AND** the exit status is 1

#### Scenario: Two clients start at the same time
- **GIVEN** no server serves `<dir>/s.sock`
- **WHEN** two clients start with `--server unix://<dir>/s.sock` at the same time
- **THEN** the screen of each client shows `no worlds` in the top row
- **AND** `<dir>/server.log` holds exactly one line that names `<dir>/s.sock` as the socket it serves

#### Scenario: A server that closes the connection
- **GIVEN** a test listens on `<dir>/s.sock`, and closes each connection as soon as it accepts it
- **WHEN** the user runs `mu client --server unix://<dir>/s.sock` in a terminal
- **THEN** the exit status is 1, and stderr contains `<dir>/s.sock`
- **AND** no file `<dir>/server.log` exists

#### Scenario: A server that does not answer
- **GIVEN** a test listens on `<dir>/s.sock`, accepts each connection and never answers
- **WHEN** the user runs `mu client --server "unix://<dir>/s.sock?connect_wait=0.2"` in a terminal
- **THEN** the exit status is 1, and stderr contains `<dir>/s.sock` and `0.2`

#### Scenario: The started server finds the default directory
- **GIVEN** `XDG_CONFIG_HOME` is `<dir>/xdg`, `<dir>/xdg/mu` does not exist, `MU_CONFIG` is not set, and nothing serves `<dir>/run/s.sock`
- **WHEN** the user runs `mu client --server unix://<dir>/run/s.sock` in a terminal and then quits the client
- **THEN** `<dir>/run/server.log` holds no line that contains `<dir>/xdg`

#### Scenario: The started server inherits MU_CONFIG
- **GIVEN** `MU_CONFIG` is `<dir>/conf`, `<dir>/conf` holds the world `alpha/aria`, and nothing serves `<dir>/run/s.sock`
- **WHEN** the user runs `mu client --server unix://<dir>/run/s.sock` in a terminal
- **THEN** the top row shows `alpha/aria`

### Requirement: The client needs a terminal
If stdin or stdout of `mu client` is not a terminal, the client SHALL start no server. It SHALL write an error to stderr that contains the text `terminal`, and exit with status 1.

#### Scenario: No terminal
- **GIVEN** `<dir>` is an empty directory, and no server serves `<dir>/s.sock`
- **WHEN** the user runs `mu client --server unix://<dir>/s.sock` with stdin on a terminal and stdout as a pipe
- **THEN** stderr contains the text `terminal`
- **AND** stderr does not contain `not implemented`
- **AND** the exit status is 1
- **AND** the directory `<dir>` is still empty

#### Scenario: No terminal on stdin
- **GIVEN** `<dir>` is an empty directory
- **WHEN** the user runs `mu client --server unix://<dir>/s.sock` with stdout on a terminal and stdin as a pipe
- **THEN** stderr contains the text `terminal`
- **AND** the exit status is 1
- **AND** the directory `<dir>` is still empty

## REMOVED Requirements

### Requirement: The client notes a server with another directory
**Reason**: The client has no configuration directory, so it has nothing to compare with the directory of the server (#169).
**Migration**: None. `mu config show` names the source file of each field of the client configuration.
