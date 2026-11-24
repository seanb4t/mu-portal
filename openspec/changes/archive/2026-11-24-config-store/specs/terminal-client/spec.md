# Spec Delta

## MODIFIED Requirements

### Requirement: The top row shows the first list
The client SHALL open the screen before its first list of the worlds for the screen. `mu client` can list the worlds once before, within the time of `connect_wait` of the server URL, to decide whether to start a server. It also gets its client configuration from the server before it opens the screen. Until that list ends, the top row SHALL show `listing worlds`, and no world SHALL have the focus, also if events of a world arrive. Keys typed in that time SHALL edit an editor of no world, and a key that sends SHALL show that no world is in focus.

#### Scenario: Before the first list
- **GIVEN** the client opened the screen, and its first list of the worlds did not end yet
- **WHEN** the client draws the screen
- **THEN** the top row shows `listing worlds`
- **WHEN** the list ends with the world `alpha/aria`
- **THEN** the top row shows `[alpha/aria]`

#### Scenario: A send before the first list
- **GIVEN** the client opened the screen, and its first list of the worlds did not end yet
- **WHEN** the user types `hi` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `no world`
- **AND** the server records no player event, and the editor holds `hi`

#### Scenario: Text typed before the first list
- **GIVEN** the client opened the screen, and its first list of the worlds did not end yet
- **WHEN** the user types `hi`, and the list ends with the world `alpha/aria`
- **THEN** the editor holds `hi`
- **WHEN** the user presses Enter
- **THEN** the server records a player event `hi` for `alpha/aria`

#### Scenario: An event before the first list
- **GIVEN** the client opened the screen, and its first list of the worlds did not end yet
- **WHEN** an event of `alpha/aria` arrives, and the user types `hi` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `no world`
- **AND** the server records no player event for `alpha/aria`

#### Scenario: A focus key before the first list
- **GIVEN** the client opened the screen, its first list of the worlds did not end yet, and an event of `alpha/aria` arrived
- **WHEN** the user presses Alt+1, types `hi` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `no world`

#### Scenario: A failed first list after an event
- **GIVEN** the client opened the screen, its first list of the worlds did not end yet, and an event of `alpha/aria` arrived
- **WHEN** the reply to the list is the error `boom`
- **THEN** the top row shows `[alpha/aria]`

#### Scenario: A notice before the first list
- **GIVEN** the terminal does not answer the probe of its width, and the first list of the worlds did not end yet
- **WHEN** the list ends with the world `alpha/aria`
- **THEN** the pane of `alpha/aria` shows a line that starts with `-- ` and contains `the terminal gave no character widths`

#### Scenario: The first list fails
- **GIVEN** the client opened the screen, and its first list of the worlds did not end yet
- **WHEN** the reply to the list is the error `boom`
- **THEN** the top row shows `no worlds`, not `listing worlds`
- **AND** the screen shows a line that starts with `-- ` and contains `boom`
