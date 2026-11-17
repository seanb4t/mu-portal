# Spec Delta

## MODIFIED Requirements

### Requirement: Each world connects and logs in at the start
At the start of the server, each fake world SHALL connect. Each world with a login SHALL send it as soon as it connects. A world from the configuration SHALL start in the state `disconnected`, and `Connect` on it SHALL fail with `failed-precondition` and a message that names the world. The API call `ListWorlds` SHALL return each world with its id and its state, one of `connecting`, `connected` and `disconnected`.

#### Scenario: Two worlds connect
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** a client calls `ListWorlds` after both connect events
- **THEN** the reply holds `alpha/aria` and `beta/bryn`, each in the state `connected`

#### Scenario: A configured world waits
- **GIVEN** a server with the fake world `beta/bryn` and a configuration that holds the world `alpha/aria`
- **WHEN** a client calls `ListWorlds` after the connect event of `beta/bryn`
- **THEN** the reply holds `alpha/aria` in the state `disconnected` and `beta/bryn` in the state `connected`

#### Scenario: A configured world cannot connect yet
- **GIVEN** a server with a configuration that holds the world `alpha/aria`
- **WHEN** a client calls `Connect` for `alpha/aria`
- **THEN** the call fails with `failed-precondition`
- **AND** the message names `alpha/aria`
