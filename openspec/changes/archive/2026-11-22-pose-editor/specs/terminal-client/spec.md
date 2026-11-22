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

## ADDED Requirements

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
