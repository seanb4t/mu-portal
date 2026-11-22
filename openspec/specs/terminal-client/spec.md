# terminal-client Specification

## Purpose
Defines what the plain terminal client shows and sends: the list of worlds, the lines of the world in focus, one input line, and the keys that move the focus.

## Requirements

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
If the user presses Enter, the client SHALL send the whole text of the editor to the world in focus and clear the editor. Ctrl+M SHALL do the same as Enter. A line break of the text SHALL go to the server as LF. An empty editor SHALL send an empty line. If the server refuses the text, the client SHALL put the text back in the editor. The requirement "A refused text stops the sends after it" gives the rule. It SHALL show the reason as a line with the prefix `-- `.

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

#### Scenario: A refused line after more text
- **GIVEN** the user typed `hi` and pressed Enter, and the server did not answer yet
- **WHEN** the user types `next`, and the server refuses `hi`
- **THEN** the editor holds the lines `hi` and `next`

#### Scenario: Ctrl+M sends
- **GIVEN** the world in focus is `beta/bryn`
- **WHEN** the user types `hi`, and the client gets the key event of Ctrl+M
- **THEN** the server records a player event `hi` for `beta/bryn`
- **AND** the editor is empty

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
- **THEN** the client shows the top row and the editor, and does not stop

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
- **THEN** the editor stays empty

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

### Requirement: Alt+Enter sends each line as a command
If the user presses Alt+Enter, the client SHALL send each line of the editor to the world in focus as its own text. It SHALL send the lines in order and clear the editor. An empty line SHALL go as an empty text. If the terminal reports the Ctrl modifier with Enter, Ctrl+Enter SHALL do the same. The client SHALL send a line only after the server answered the line before it. Each line SHALL go to the world that had the focus at the Alt+Enter. A later move of the focus SHALL NOT change that world.

#### Scenario: Each line is a command
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `one`, Ctrl+J and `two`, and the client gets the key event of Alt+Enter or of Ctrl+Enter
- **THEN** the screen shows `You say, "one"` and then `You say, "two"`
- **AND** the editor is empty and takes one row

#### Scenario: The focus moves between two lines
- **GIVEN** the worlds `alpha/aria` and `beta/bryn`, the focus on `alpha/aria`, and the editor with the lines `one` and `two`
- **WHEN** the user presses Alt+Enter, and presses Ctrl+N before the server answers `one`
- **THEN** the server gets `one` and `two` for `alpha/aria`
- **AND** the server gets no line for `beta/bryn`

### Requirement: The sends to a world go in order
The client SHALL send the texts of a world to the server in the order of the keys, from Enter, Alt+Enter and Ctrl+Enter alike. It SHALL send a text of a world only after the server answered the text of that world before it. A key that sends SHALL clear the editor also while an earlier text waits. A text that waits for one world SHALL NOT delay a text for another world.

#### Scenario: Two sends of Enter
- **GIVEN** the user typed `one` and pressed Enter in `alpha/aria`, and the server did not answer yet
- **WHEN** the user types `two` and presses Enter
- **THEN** the editor is empty, and the client did not send `two` yet
- **AND** after the server answers `one`, the client sends `two`

#### Scenario: A send while lines wait
- **GIVEN** the user pressed Alt+Enter on the lines `one` and `two`, and the server did not answer `one` yet
- **WHEN** the user types `next` and presses Enter or Alt+Enter
- **THEN** the editor is empty
- **AND** the server gets `one`, `two` and `next` in this order, each after the answer for the one before

#### Scenario: A send of each line while a send waits
- **GIVEN** the user typed `one` and pressed Enter in `alpha/aria`, and the server did not answer yet
- **WHEN** the user types `one`, Ctrl+J and `two`, and presses Alt+Enter
- **THEN** the server gets `one`, `one` and `two` in this order, each after the answer for the one before

#### Scenario: A send to another world
- **GIVEN** the user typed `before` and pressed Enter in `alpha/aria`, and the server did not answer yet
- **WHEN** the user presses Ctrl+N, types `one` and presses Enter
- **THEN** the client sends `one` to `beta/bryn` at once

### Requirement: A refused text stops the sends after it
If the server refuses a text, the client SHALL send none of the texts that wait for the same world. It SHALL put the refused text and each text that waits for that world back in the editor, in order. These texts SHALL go before the text that the user typed since. It SHALL show the reason as a line with the prefix `-- `. A key that sends after that SHALL send at once.

#### Scenario: A line that the server refuses
- **GIVEN** the editor holds the lines `one`, `two` and `three`, and a test server that accepts `one` and refuses `two`
- **WHEN** the user presses Alt+Enter
- **THEN** the server gets `one` and `two`, and it does not get `three`
- **AND** the screen shows a line that starts with `-- `
- **AND** the editor holds the lines `two` and `three`

#### Scenario: The last line is refused
- **GIVEN** the user pressed Alt+Enter on the lines `one` and `two`, and the server accepted `one`
- **WHEN** the user types `later`, and the server refuses `two`
- **THEN** the editor holds the lines `two` and `later`

#### Scenario: A refused send of Enter with a send after it
- **GIVEN** the user typed `one` and pressed Enter, typed `two` and pressed Enter, and the server did not answer yet
- **WHEN** the user types `later`, and the server refuses `one`
- **THEN** the server does not get `two`
- **AND** the editor holds the lines `one`, `two` and `later`

### Requirement: Alt+R joins the lines in place
If the user presses Alt+R, the client SHALL replace each line break of the editor with the `line_break` of the world in focus. It SHALL send nothing. If that world has no `line_break`, or no world is in focus, the client SHALL NOT change the editor. A `line_break` from the server that is empty or holds a control character SHALL count as no `line_break`. It SHALL then show a line with the prefix `-- ` that contains `line break`.

#### Scenario: The lines become one line
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `one`, Ctrl+J and `two`, and presses Alt+R
- **THEN** the editor holds the one line `one%rtwo`
- **AND** the server records no event

#### Scenario: A world with no line break
- **GIVEN** the world in focus is a configured world with no `line_break`
- **WHEN** the user types `one`, Ctrl+J and `two`, and presses Alt+R
- **THEN** the editor still holds `one` and `two` on 2 rows
- **AND** the screen shows a line that starts with `-- ` and contains `line break`

#### Scenario: A line break that the editor cannot hold
- **GIVEN** the list of the worlds gives `alpha/aria` an empty `line_break`, or one with ESC or LF
- **WHEN** the user types `one`, Ctrl+J and `two`, and presses Alt+R
- **THEN** the editor still holds `one` and `two`

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
