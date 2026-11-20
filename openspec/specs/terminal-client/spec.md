# terminal-client Specification

## Purpose
Defines what the plain terminal client shows and sends: the list of worlds, the lines of the world in focus, one input line, and the keys that move the focus.

## Requirements

### Requirement: The screen has three parts
The client SHALL use the alternate screen. The top row SHALL list the worlds in order by their game part and then by their character part. The world in focus SHALL be in square brackets. With no world, the top row SHALL show `no worlds`. The bottom row SHALL hold the input line. The rows between SHALL show the output pane of the world in focus. At the start, the pane is at the bottom, so it shows the latest lines, the newest at the bottom.

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
The client SHALL show a line from the game with the styles of its SGR sequences, and drop each other escape sequence. It SHALL show a line from the player with the prefix `> `. It SHALL show an event of the system or a credential with the prefix `-- `. After that, the client SHALL show a tab as a space and each other control character as U+FFFD. Thus no text sends a sequence to the terminal.

#### Scenario: Prefixes
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the user types `hello` and presses Enter
- **THEN** the screen shows `> hello`
- **AND** the next row shows `You say, "hello"`
- **AND** the screen shows `-- connect aria ****`

#### Scenario: Control characters
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the game sends a line that holds BEL, DEL, a tab and the C1 character U+009B
- **THEN** the screen shows U+FFFD for each of BEL, DEL and U+009B, and a space for the tab

#### Scenario: Colors
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the game sends the line ESC `[1;31mred` ESC `[0m plain`
- **THEN** the screen shows `red plain`
- **AND** the cells of `red` are bold and red, and the cells of `plain` have the default style

#### Scenario: Other escape sequences
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the game sends the line `a` ESC `]0;title` BEL `b` ESC `[2Jc`
- **THEN** the screen shows `abc`
- **AND** the rows of the screen above that line do not change

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
- **WHEN** the relay cuts the connection and holds
- **THEN** the top row shows `disconnected`
- **WHEN** the relay releases, and a test sends `after the cut` to `alpha/aria` through the API
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

### Requirement: A style lasts to its reset
An SGR style of a game line SHALL last into the next game lines of the same world, until an SGR sequence resets it. The client SHALL support the 16 named colors, the 256 indexed colors and 24-bit colors.

#### Scenario: A color across two lines
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the game sends the line ESC `[32mgreen` and then the line `still` ESC `[0m done`
- **THEN** the cells of `green` and `still` are green
- **AND** the cells of `done` have the default style

#### Scenario: Indexed and 24-bit colors
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the game sends the line ESC `[38;5;208mA` ESC `[38;2;10;20;30mB`
- **THEN** the cell of `A` has the indexed color 208
- **AND** the cell of `B` has the color with red 10, green 20 and blue 30

### Requirement: An unfinished line shows once
A prompt event of a world SHALL show as the last row of its pane. A later prompt or line of the same world SHALL replace that row. Each event SHALL start the escape parser in its ground state. Only a line event SHALL change the SGR style that carries to the next line.

#### Scenario: A line in two reads
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the game sends `hel` in one read and `lo` with a line end in a later read
- **THEN** the screen shows `hello` on exactly one row
- **AND** no row shows `hel` on its own

#### Scenario: A prompt with no line end
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the game sends `Name: ` with no line end
- **THEN** the last row of the pane shows `Name: `

#### Scenario: A read that ends inside an escape sequence
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the game sends `ab` ESC `[3` in one read, and `1mred` with a line end in a later read
- **THEN** the screen shows `abred` on exactly one row
- **AND** the cells of `red` are red

#### Scenario: A prompt does not carry its style
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the game sends ESC `[31mName: ` with no line end, then the line `plain`
- **THEN** the cells of `plain` have the default style

#### Scenario: An open OSC
- **GIVEN** the client shows `alpha/aria`
- **WHEN** the game sends the line `a` ESC `]0;title` and then the line `next`
- **THEN** the screen shows `a` and `next` on two rows

### Requirement: A reset clears each pane
At a Reset of the server, the client SHALL clear the lines, the unfinished line, the scroll position and the selection of each world. It SHALL also reset the SGR style of each world to the default.

#### Scenario: A reset
- **GIVEN** the pane of `alpha/aria` shows the unfinished line `Name: ` and a selection, and the last game line set the color red
- **WHEN** the server sends a Reset, and then the game sends the line `after`
- **THEN** the pane shows only `after`, at the bottom
- **AND** the cells of `after` have the default style

### Requirement: Keys and the mouse scroll the pane
PageUp SHALL scroll the pane back by its height less one row, and PageDown SHALL scroll it forward by the same amount. The step SHALL be at least one row. Each step of the mouse wheel SHALL scroll it by 3 rows. If the user presses Enter, the pane of the world in focus SHALL return to the bottom.

#### Scenario: Back and forward
- **GIVEN** the client shows `alpha/aria`, and the world has more lines than the pane shows
- **WHEN** the user presses PageUp
- **THEN** the pane shows older lines, and the newest line is not on the screen
- **WHEN** the user presses PageDown
- **THEN** the pane shows the newest line on its last row

#### Scenario: A new line while scrolled back
- **GIVEN** the pane of `alpha/aria` is scrolled back
- **WHEN** a test sends `ping` to `alpha/aria` through the API
- **THEN** the rows of the pane do not change

#### Scenario: Enter returns to the bottom
- **GIVEN** the pane of `alpha/aria` is scrolled back
- **WHEN** the user types `hi` and presses Enter
- **THEN** the pane shows `You say, "hi"` on its last row

#### Scenario: The mouse wheel
- **GIVEN** the client shows `alpha/aria`, and the world has more lines than the pane shows
- **WHEN** the user turns the mouse wheel up by one step
- **THEN** the pane shows the rows 3 rows older

#### Scenario: A pane of one row
- **GIVEN** a terminal of 3 rows, so the pane of `alpha/aria` is 1 row high, and the world has 5 lines
- **WHEN** the user presses PageUp
- **THEN** the pane shows the line before the newest line

#### Scenario: A pane of no rows
- **GIVEN** a terminal of 2 rows, so the pane of `alpha/aria` has no row
- **WHEN** the user presses PageUp and then PageDown
- **THEN** the client shows the top row and the input line, and does not stop

### Requirement: Each world keeps its own scroll position
Each world SHALL have its own pane, with its own scroll position and selection. A change of focus SHALL keep the position of each pane.

#### Scenario: Back to a scrolled world
- **GIVEN** the pane of `alpha/aria` is scrolled back
- **WHEN** the user presses Ctrl+N and then Ctrl+P
- **THEN** the pane of `alpha/aria` shows the same rows as before

### Requirement: A drag copies through OSC 52
A drag of the left mouse button in the pane SHALL select the text from the press to the release. At the release, the client SHALL write the copy text of the selection to the terminal as an OSC 52 sequence for the clipboard. A click with no drag SHALL clear the selection.

#### Scenario: Copy by drag
- **GIVEN** the pane shows the row `You say, "hello"`
- **WHEN** the user drags from the `h` of `hello` to its `o` and releases
- **THEN** the client writes ESC `]52;c;aGVsbG8=` and then ESC `\`
- **AND** the cells of `hello` are in reverse

#### Scenario: A click clears
- **GIVEN** the pane shows a selection
- **WHEN** the user clicks once in the pane
- **THEN** no cell of the pane is in reverse

### Requirement: The client measures character widths at startup
Before its first frame, the client SHALL ask the terminal for the state of mode 2027. It SHALL also ask for the cursor position after one grapheme of each width class. It SHALL end the queries with a device attributes query. If the terminal knows mode 2027 and the mode is off, the client SHALL turn it on. The pane SHALL use the widths that the terminal gave.

#### Scenario: A terminal that adds code point widths
- **GIVEN** a terminal that answers the cursor position query after `👨‍👩‍👧` at column 9, so the width of that class is 8
- **WHEN** the client starts, and the game sends the line `👨‍👩‍👧x`
- **THEN** the cell of `x` is at column 9

#### Scenario: A terminal that does not answer
- **GIVEN** a terminal that answers no query
- **WHEN** the user starts `mu client`
- **THEN** the top row shows the worlds
- **AND** the client uses the `unicode-width` value of each grapheme

#### Scenario: A late answer
- **GIVEN** a terminal that answers the queries only after the client draws its first frame
- **WHEN** the answers arrive
- **THEN** the input line stays empty

### Requirement: The client restores the terminal
The client SHALL turn on mouse capture at the start. When it exits, it SHALL turn off mouse capture and each mode that it turned on, mode 2027 included, and leave the alternate screen.

#### Scenario: The modes at exit
- **GIVEN** a terminal that reports mode 2027 as known and off, so the client turned on mouse capture and mode 2027
- **WHEN** the user presses Ctrl+C
- **THEN** the client writes the sequences that turn off mouse capture and mode 2027 before it exits

#### Scenario: Mode 2027 was already on
- **GIVEN** a terminal that reports mode 2027 as on
- **WHEN** the user presses Ctrl+C
- **THEN** the client writes the sequence that turns off mouse capture
- **AND** the client writes no sequence that turns off mode 2027
