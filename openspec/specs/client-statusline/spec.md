# client-statusline Specification

## Purpose

Defines the `statusline` block of the layout, the segments that it shows about the world in focus, and the `[statusline]` table that picks them.

## Requirements

### Requirement: The statusline shows two lists of segments
The block `statusline` SHALL show one row. The segments of `left` SHALL start at the left edge. The segments of `right` SHALL end at the right edge. The segments of one side SHALL have ` | ` between them. A segment with no value SHALL not show, and SHALL take no separator. If the two sides do not fit, the right side SHALL show in full and the left side SHALL be cut at its end.

#### Scenario: The default segments
- **GIVEN** `clients/global.toml` holds `layout = "status"`, and a server has the fake world `alpha/aria`, which is connected
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** row 24 starts with `alpha/aria | connected`

#### Scenario: A narrow terminal
- **GIVEN** the layout `status` is in use, `[statusline]` with `left = ["world"]` and `right = ["state"]`, and the connected fake world `alpha/aria`
- **WHEN** the user starts `mu client` in a terminal of 12 columns
- **THEN** the row of the statusline ends with `connected`

### Requirement: The segments of the statusline
The table `[statusline]` SHALL hold `left` and `right`, two lists of segment names. Their defaults SHALL be `["world", "state"]` and `["idle"]`. Each segment SHALL be about the world in focus:

- `world`: the world id
- `state`: `connecting`, `connected`, `disconnected` or `reconnecting`
- `connected`: the time since the world connected, while it is connected
- `idle`: the time since the last send to the world from any client
- `typed`: the count of characters in the editor with the input focus

#### Scenario: The typed count
- **GIVEN** the layout `status` is in use, `[statusline]` with `left = ["typed"]` and `right = []`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `héllo`
- **THEN** the row of the statusline starts with `5`

#### Scenario: The state after a disconnect
- **GIVEN** the layout `status` is in use, the default segments, and the connected fake world `alpha/aria` in focus
- **WHEN** the user runs the command `/disconnect`
- **THEN** the row of the statusline starts with `alpha/aria | disconnected`

#### Scenario: The idle time after a send
- **GIVEN** the layout `status` is in use, `[statusline]` with `left = []` and `right = ["idle"]`, and the connected fake world `alpha/aria` in focus
- **WHEN** the user types `hi` and presses Enter
- **THEN** the row of the statusline ends with a time from `0:00` to `0:09`

#### Scenario: The connected time
- **GIVEN** the layout `status` is in use, `[statusline]` with `left = ["connected"]` and `right = []`, and the fake world `alpha/aria` connected at the start of the test
- **WHEN** the user starts `mu client`
- **THEN** the row of the statusline starts with a time from `0:00` to `0:09`

#### Scenario: No connected time while disconnected
- **GIVEN** the layout `status` is in use, `[statusline]` with `left = ["world", "connected"]` and `right = []`, and the disconnected fake world `alpha/aria` in focus
- **WHEN** the client draws the screen
- **THEN** the row of the statusline holds `alpha/aria` and no `|`

### Requirement: No world in focus
With no world in focus, the statusline SHALL show `no world` at the left edge and no other segment.

#### Scenario: No worlds
- **GIVEN** the layout `status` is in use, and a server with no world
- **WHEN** the user starts `mu client`
- **THEN** the row of the statusline starts with `no world`

### Requirement: The format of a time
A time SHALL show as `M:SS` under one hour and as `H:MM:SS` from one hour. A time before the clock of the client, from a clock that went back, SHALL show as `0:00`.

#### Scenario: Under an hour
- **GIVEN** a world that connected 75 seconds before the draw
- **WHEN** the client draws a statusline with `connected`
- **THEN** the segment shows `1:15`

#### Scenario: Over an hour
- **GIVEN** a world that connected 3725 seconds before the draw
- **WHEN** the client draws a statusline with `connected`
- **THEN** the segment shows `1:02:05`

### Requirement: The times count each second
While the layout holds `statusline`, and its segments hold `connected` or `idle`, the client SHALL draw the screen again once a second. Without such a segment, the client SHALL NOT draw for the clock alone.

#### Scenario: A tick moves the idle time
- **GIVEN** a model with the layout `status`, and the last send to the world in focus at time T
- **WHEN** the model gets a tick of the clock at T plus 3 seconds
- **THEN** the segment shows `0:03`

#### Scenario: No tick without a time segment
- **GIVEN** a model with the layout `classic`
- **WHEN** the client asks for its effects
- **THEN** the effects hold no tick of the clock

### Requirement: A bad statusline makes its layer bad
The layer SHALL be bad, and its notice SHALL name the file and the field, in each of these cases:

- an unknown segment
- a segment twice in one list
- a value that is not a list of texts

#### Scenario: An unknown segment
- **GIVEN** `clients/global.toml` holds `[statusline]` with `left = ["mood"]`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml`, `statusline.left` and `mood`

#### Scenario: A segment twice
- **GIVEN** `clients/global.toml` holds `[statusline]` with `right = ["idle", "idle"]`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml`, `statusline.right` and `idle`

### Requirement: A list of segments replaces the earlier list
A layer that sets `left` or `right` SHALL replace the whole list of an earlier layer. A layer that sets one list SHALL keep the other list of the earlier layer.

#### Scenario: A device sets the left list
- **GIVEN** `clients/global.toml` holds `[statusline]` with `left = ["world", "state"]` and `right = ["typed"]`
- **AND** `clients/devices/laptop.toml` holds `[statusline]` with `left = ["idle"]`
- **AND** a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `left = ["idle"]` with the source `clients/devices/laptop.toml`
- **AND** stdout holds `right = ["typed"]` with the source `clients/global.toml`
