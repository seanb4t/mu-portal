# Spec Delta

## MODIFIED Requirements

### Requirement: Connect and disconnect a world
`/connect` SHALL ask the server to connect the world that it names, and `/disconnect` to disconnect it. With no world, each SHALL act on the world in focus. With no world and no world in focus, it SHALL be a bad command. If the server refuses, the client SHALL show the reason as a line with the prefix `-- `.

#### Scenario: Connect the world in focus
- **GIVEN** the world in focus is the fake world `alpha/aria`, and a test disconnected it through the API
- **WHEN** the user types `/connect` and presses Enter
- **THEN** the server connects `alpha/aria`
- **AND** the screen shows `Welcome, aria.` after the disconnect

#### Scenario: Disconnect a named world
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, and the world in focus is `alpha/aria`
- **WHEN** the user types `/disconnect beta/bryn` and presses Enter
- **THEN** the server reports `beta/bryn` as disconnected
- **AND** `alpha/aria` is still connected

#### Scenario: A key bound to connect with no world in focus
- **GIVEN** `clients/global.toml` holds `[keys]` with `"alt+c" = "connect"`, and a server serves that directory with no world
- **WHEN** the user starts `mu client` and presses Alt+C
- **THEN** the screen shows a line that starts with `-- ` and contains `no world`
- **AND** the client is still running

#### Scenario: Connect with no world in focus
- **GIVEN** a server with no world
- **WHEN** the user types `/connect` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `no world`
- **AND** the editor holds `/connect`

#### Scenario: A trailing space is no argument
- **GIVEN** the world in focus is the fake world `alpha/aria`, and a test disconnected it through the API
- **WHEN** the user types `/connect ` with a trailing space and presses Enter
- **THEN** the server connects `alpha/aria`

#### Scenario: Disconnect an unknown world
- **GIVEN** a server with the fake world `alpha/aria`
- **WHEN** the user types `/disconnect gamma/gil` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `gamma/gil`
- **AND** `alpha/aria` is still connected
