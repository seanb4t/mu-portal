# Spec Delta

## Purpose

Defines what the plain terminal client shows and sends: the list of worlds, the lines of the world in focus, one input line, and the keys that move the focus.

## ADDED Requirements

### Requirement: The screen has three parts
The client SHALL use the alternate screen. The top row SHALL list the worlds in order by their game part and then by their character part, with the world in focus in square brackets. With no world, the top row SHALL show `no worlds`. The bottom row SHALL hold the input line. The rows between SHALL show the latest lines of the world in focus, the newest at the bottom.

#### Scenario: The first view
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** the top row shows `[alpha/aria]` and then `beta/bryn`
- **AND** the rows between show `Welcome, aria.`
- **AND** the screen shows no line of `beta/bryn`

#### Scenario: The order of the worlds
- **GIVEN** a server with the fake worlds `alpha-2/cy`, `alpha/bea` and `alpha/aria`
- **WHEN** the user starts `mu client`
- **THEN** the top row shows `[alpha/aria]`, then `alpha/bea`, then `alpha-2/cy`

#### Scenario: No worlds
- **GIVEN** a server with no world
- **WHEN** the user starts `mu client`
- **THEN** the top row shows `no worlds`

### Requirement: Lines show their source
The client SHALL show a line from the game as it is. It SHALL show a line from the player with the prefix `> `. It SHALL show an event of the system or a credential with the prefix `-- `.

#### Scenario: Prefixes
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the user types `hello` and presses Enter
- **THEN** the screen shows `> hello`
- **AND** the next row shows `You say, "hello"`
- **AND** the screen shows `-- connect aria ****`

### Requirement: Enter sends the input to the world in focus
If the user presses Enter, the client SHALL send the text of the input line to the world in focus and clear the input line. An empty input line SHALL send an empty line. If the server refuses the line, the client SHALL keep the text in the input line. It SHALL show the reason as a line with the prefix `-- `.

#### Scenario: A line goes to the world in focus
- **GIVEN** the world in focus is `beta/bryn`
- **WHEN** the user types `hi` and presses Enter
- **THEN** the server records a player event `hi` for `beta/bryn`
- **AND** the input line is empty

#### Scenario: An empty line
- **GIVEN** the world in focus is `alpha/aria`, and the input line is empty
- **WHEN** the user presses Enter
- **THEN** the server records a player event with an empty text for `alpha/aria`

#### Scenario: Enter with no world
- **GIVEN** a server with no world
- **WHEN** the user types `hi` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `no world`
- **AND** the input line still holds `hi`

#### Scenario: A refused line
- **GIVEN** the world in focus is `alpha/aria`, and it is disconnected
- **WHEN** the user types `hi` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `not connected`
- **AND** the input line still holds `hi`

### Requirement: Keys move the focus
Ctrl+N SHALL move the focus to the next world and Ctrl+P to the one before. Both SHALL wrap around at the end of the list. A world out of focus can have a line that the user did not see. For such a world, the client SHALL show `*` after its id. If that world gets the focus, the mark SHALL go.

#### Scenario: Next world
- **GIVEN** the world in focus is `alpha/aria`, and `beta/bryn` is the next world
- **WHEN** the user presses Ctrl+N
- **THEN** the top row shows `[beta/bryn]`
- **AND** the rows between show `Welcome, bryn.`

#### Scenario: Wrap around
- **GIVEN** the world in focus is `alpha/aria`, the first world
- **WHEN** the user presses Ctrl+P
- **THEN** the top row shows the last world in square brackets

#### Scenario: Next world wraps at the end
- **GIVEN** the world in focus is `beta/bryn`, the last world
- **WHEN** the user presses Ctrl+N
- **THEN** the top row shows `[alpha/aria]`

#### Scenario: Keys with no world
- **GIVEN** a server with no world
- **WHEN** the user presses Ctrl+N and then Ctrl+P
- **THEN** the top row shows `no worlds`
- **AND** the client is still running

#### Scenario: One world keeps the focus
- **GIVEN** a server with the one fake world `alpha/aria`
- **WHEN** the user presses Ctrl+N and then Ctrl+P
- **THEN** the top row shows `[alpha/aria]`

#### Scenario: Activity mark
- **GIVEN** the world in focus is `alpha/aria`
- **WHEN** a test sends `ping` to `beta/bryn` through the API, and the fake world answers `You say, "ping"`
- **THEN** the top row shows `beta/bryn*`
- **AND** after Ctrl+N the top row shows `[beta/bryn]` with no `*`

### Requirement: The client follows the server
If the event stream ends, the client SHALL show `disconnected` in the top row and resume the stream with no gap. After the resume, the top row SHALL no longer show `disconnected`.

#### Scenario: A lost stream
- **GIVEN** the client shows `alpha/aria`, and reads the stream through a relay
- **WHEN** the relay cuts the connection
- **THEN** the top row shows `disconnected`
- **WHEN** the relay passes again, and a test sends `after the cut` to `alpha/aria` through the API
- **THEN** the screen shows `You say, "after the cut"` on exactly one row
- **AND** the top row no longer shows `disconnected`

#### Scenario: The client resumes before a new event
- **GIVEN** the top row shows `disconnected`
- **WHEN** the stream resumes on the same server, and no world gets a new event
- **THEN** the top row no longer shows `disconnected`

### Requirement: Ctrl+C quits the client only
If the user presses Ctrl+C, the client SHALL leave the alternate screen, restore the terminal and exit with status 0. The server and its worlds SHALL keep running.

#### Scenario: Quit
- **GIVEN** the client is attached to a server
- **WHEN** the user presses Ctrl+C
- **THEN** the client exits with status 0
- **AND** `ListWorlds` on the server still returns each world in the state `connected`
