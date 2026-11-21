# Spec Delta

## MODIFIED Requirements

### Requirement: The screen has three parts
The client SHALL use the alternate screen. The top row SHALL list the worlds in order by their game part and then by their character part. The world in focus SHALL be in square brackets. With no world, the top row SHALL show `no worlds`. The bottom rows SHALL hold the pose editor, which this spec also calls the input line. With no text, the editor SHALL take one row. The rows between SHALL show the output pane of the world in focus. At the start, the pane is at the bottom, so it shows the latest lines, the newest at the bottom.

#### Scenario: The first view
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** the top row shows `[alpha/aria]` and then `beta/bryn`
- **AND** the rows between show `Welcome, aria.`
- **AND** the screen shows no line of `beta/bryn`
- **AND** the editor takes only the bottom row

#### Scenario: The order of the worlds
- **GIVEN** a server with the fake worlds `alpha-2/cy`, `alpha/bea` and `alpha/aria`
- **WHEN** the user starts `mu client`
- **THEN** the top row shows `[alpha/aria]`, then `alpha/bea`, then `alpha-2/cy`

#### Scenario: No worlds
- **GIVEN** a server with no world
- **WHEN** the user starts `mu client`
- **THEN** the top row shows `no worlds`

### Requirement: Enter sends the input to the world in focus
If the user presses Enter, the client SHALL send the whole text of the editor to the world in focus and clear the editor. A line break of the text SHALL go to the server as LF. An empty editor SHALL send an empty line. If the server refuses the text, the client SHALL keep the text in the editor. It SHALL show the reason as a line with the prefix `-- `.

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

#### Scenario: A pose of two lines
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `one`, presses Ctrl+J, types `two` and presses Enter
- **THEN** the screen shows `You say, "one%rtwo"`
- **AND** the editor is empty and takes one row

#### Scenario: A line break that the world refuses
- **GIVEN** the world in focus is a configured world with no `line_break`, on a scripted game
- **WHEN** the user types `one`, presses Ctrl+J, types `two` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `line break`
- **AND** the editor still holds `one` and `two` on 2 rows

### Requirement: The client restores the terminal
The client SHALL turn on mouse capture and bracketed paste at the start. When it exits, it SHALL turn off mouse capture, bracketed paste and each mode that it turned on, mode 2027 included. If it pushed kitty keyboard flags, it SHALL pop them. Then it SHALL leave the alternate screen.

#### Scenario: The modes at exit
- **GIVEN** a terminal that reports mode 2027 as known and off, so the client turned on mouse capture and mode 2027
- **WHEN** the user presses Ctrl+C
- **THEN** the client writes the sequences that turn off mouse capture and mode 2027 before it exits

#### Scenario: Mode 2027 was already on
- **GIVEN** a terminal that reports mode 2027 as on
- **WHEN** the user presses Ctrl+C
- **THEN** the client writes the sequence that turns off mouse capture
- **AND** the client writes no sequence that turns off mode 2027

#### Scenario: Bracketed paste at the start and at exit
- **WHEN** the user starts `mu client` and presses Ctrl+C
- **THEN** the client writes ESC `[?2004h` before its first frame
- **AND** the client writes ESC `[?2004l` before it exits

## ADDED Requirements

### Requirement: The client turns on the kitty keyboard protocol
The startup probe SHALL ask the terminal for its kitty keyboard flags, before the device attributes query. If the terminal answers, the client SHALL push the flag that disambiguates escape codes before its first frame. If the terminal does not answer before the probe stops, the client SHALL push no flag. A late answer SHALL NOT reach the editor.

#### Scenario: A terminal with the protocol
- **GIVEN** a terminal that answers the kitty keyboard query with ESC `[?0u`
- **WHEN** the user starts `mu client` and presses Ctrl+C
- **THEN** the client writes ESC `[>1u` before its first frame
- **AND** the client writes ESC `[<1u` before it exits

#### Scenario: A terminal without the protocol
- **GIVEN** a terminal that answers the device attributes query and not the kitty keyboard query
- **WHEN** the user starts `mu client` and presses Ctrl+C
- **THEN** the client writes no ESC `[>1u` and no ESC `[<1u`

#### Scenario: Shift+Enter with the protocol
- **GIVEN** the client pushed the kitty keyboard flag, and the editor holds `a`
- **WHEN** the terminal sends ESC `[13;2u` and the user types `b`
- **THEN** the editor holds `a` and `b` on 2 rows
- **AND** the server records no event

#### Scenario: A late answer of the kitty keyboard query
- **GIVEN** a terminal that answers the queries only after the client draws its first frame
- **WHEN** the answers arrive, the kitty keyboard answer among them
- **THEN** the input line stays empty
- **AND** the client writes no ESC `[>1u`
