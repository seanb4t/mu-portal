# Spec Delta

## MODIFIED Requirements

### Requirement: The fields of a client layer
A client layer SHALL hold only the field `echo` and the field `layout`, and the tables `[keys]`, `[tabs]`, `[statusline]` and `[world-layouts]`. Each SHALL be optional. An unknown field or table SHALL make the layer bad. So SHALL a value of the wrong type. The notice SHALL name the file and the field.

#### Scenario: A server table
- **GIVEN** `clients/global.toml` holds `[server]` with `connect_wait_seconds = 2`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `server`

#### Scenario: An unknown field
- **GIVEN** `clients/global.toml` holds `colour = "red"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `colour`

#### Scenario: Not TOML
- **GIVEN** `clients/global.toml` holds `echo = `, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and line 1

#### Scenario: A file that cannot be read
- **GIVEN** the test runs as a user that is not root, `<dir>/clients/global.toml` exists with mode 000, and a server serves `<dir>` on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml`
- **AND** the exit status is 0

#### Scenario: The three new fields
- **GIVEN** `clients/global.toml` holds `layout = "focus"`, `[tabs]` with `label = "{character}"`, and `[statusline]` with `left = ["world"]`
- **AND** a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr is empty
- **AND** stdout holds `layout = "focus"`, `label = "{character}"` and `left = ["world"]`, each with the source `clients/global.toml`

#### Scenario: A layout table
- **GIVEN** `clients/global.toml` holds `[layout]` with `shape = "wide"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `layout`

#### Scenario: An unknown field of tabs
- **GIVEN** `clients/global.toml` holds `[tabs]` with `colour = "red"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `colour`

#### Scenario: A world layouts table
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `"alpha/aria" = "focus"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr is empty
- **AND** stdout holds `world-layouts."alpha/aria" = "focus"` with the source `clients/global.toml`
