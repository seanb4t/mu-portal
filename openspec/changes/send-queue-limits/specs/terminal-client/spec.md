# Spec Delta

## MODIFIED Requirements

### Requirement: Ctrl+C quits the client only
If the user presses Ctrl+C, the client SHALL leave the alternate screen, restore the terminal and exit with status 0. The server and its worlds SHALL keep running. If a text of a world waits for the server, the first Ctrl+C SHALL NOT quit. It SHALL show a line with the prefix `-- ` that holds the count of those texts. A second Ctrl+C SHALL then quit. Any other key between the two SHALL cancel the second one.

#### Scenario: Quit
- **GIVEN** the client is attached to a server
- **WHEN** the user presses Ctrl+C
- **THEN** the client exits with status 0
- **AND** `ListWorlds` on the server still returns each world in the state `connected`

#### Scenario: Quit while lines wait
- **GIVEN** the user pressed Alt+Enter on the lines `one`, `two` and `three`, and the server did not answer `one` yet
- **WHEN** the user presses Ctrl+C
- **THEN** the client does not quit, and the screen shows a line that starts with `-- ` and contains `3 lines not sent yet`
- **WHEN** the user presses Ctrl+C again
- **THEN** the client quits

#### Scenario: Quit while only the line with the server waits
- **GIVEN** the user typed `one` and pressed Enter, and the server did not answer yet
- **WHEN** the user presses Ctrl+C
- **THEN** the client does not quit, and the screen shows a line that contains `1 line not sent yet`

#### Scenario: Another key between the two
- **GIVEN** the user pressed Ctrl+C while lines wait, and the client did not quit
- **WHEN** the user types `x` and presses Ctrl+C
- **THEN** the client does not quit

### Requirement: A refused text stops the sends after it
If the server refuses a text, the client SHALL send none of the texts that wait for the same world. A send with no reply in time counts as a refusal. The requirement "A send with no reply is refused" gives the time. It SHALL put the refused text and each text that waits for that world back in the editor, in order. These texts SHALL go before the text that the user typed since. It SHALL show the reason as a line with the prefix `-- `. A key that sends after that SHALL send at once. If the event stream ends, or the server resets, the client SHALL put each text that waits behind the text with the server back in the editor, in the order of the worlds, before the text that the user typed since. It SHALL show a line with the prefix `-- ` that holds their count. The text with the server SHALL stay, and its reply SHALL act on it alone.

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

#### Scenario: The stream ends while lines wait
- **GIVEN** the user pressed Alt+Enter on the lines `one`, `two` and `three`, and the server did not answer `one` yet
- **WHEN** the user types `later`, and the event stream ends
- **THEN** the editor holds the lines `two`, `three` and `later`
- **AND** the screen shows a line that starts with `-- ` and contains `2 lines that waited`
- **WHEN** the server answers `one`
- **THEN** the client sends no text

#### Scenario: Lines of two worlds wait
- **GIVEN** lines wait behind a send in `beta/bryn`, and after that behind a send in `alpha/aria`
- **WHEN** the event stream ends
- **THEN** the editor holds the lines of `alpha/aria` before the lines of `beta/bryn`

#### Scenario: The server resets while lines wait
- **GIVEN** the user pressed Alt+Enter on the lines `one`, `two` and `three`, and the server did not answer `one` yet
- **WHEN** the server resets
- **THEN** the editor holds the lines `two` and `three`

#### Scenario: A refusal after the stream ended
- **GIVEN** the user pressed Alt+Enter on the lines `one`, `two` and `three`, the server did not answer `one` yet, and the event stream ended
- **WHEN** the server refuses `one`
- **THEN** the editor holds the lines `one`, `two` and `three`

### Requirement: Alt+R joins the lines in place
If the user presses Alt+R, the client SHALL replace each line break of the editor with the `line_break` of the world in focus. It SHALL send nothing. If that world has no `line_break`, or no world is in focus, the client SHALL NOT change the editor. A `line_break` from the server that is empty or holds a control character SHALL count as no `line_break`. It SHALL then show a line with the prefix `-- ` that contains `line break`. If no list of the worlds held the world in focus yet, the client SHALL NOT change the editor. It SHALL show a line with the prefix `-- ` that contains `does not know the line break`, and ask the server for the list of the worlds. The client SHALL also ask for that list after the first event of a world that it did not hold, and after the server resets. It SHALL ask for one list at a time. A list with no reply within 30 seconds SHALL count as failed. A list SHALL set the `line_break` of each world in it. With no world in focus, the client SHALL ask for no list.

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

#### Scenario: A world that the first list did not hold
- **GIVEN** the first list of the worlds failed, and the first event of `alpha/aria` came
- **WHEN** the list of the worlds that the client asks for gives `alpha/aria` the `line_break` `%r`, and the user types `one`, Ctrl+J and `two`, and presses Alt+R
- **THEN** the editor holds the one line `one%rtwo`

#### Scenario: No world in focus
- **GIVEN** the first list of the worlds failed, and no world sent an event
- **WHEN** the user types `one`, Ctrl+J and `two`, and presses Alt+R
- **THEN** the editor still holds `one` and `two`
- **AND** the screen shows a line that starts with `-- ` and contains `line break`
- **AND** the client asks the server for no list

#### Scenario: A line break that the client does not know
- **GIVEN** the first list of the worlds failed, the first event of `alpha/aria` came, and the list after it failed too
- **WHEN** the user types `one`, Ctrl+J and `two`, and presses Alt+R
- **THEN** the editor still holds `one` and `two`
- **AND** the screen shows a line that starts with `-- ` and contains `does not know the line break`
- **AND** the client asks the server for the list of the worlds

### Requirement: The client measures character widths at startup
Before its first frame, the client SHALL ask the terminal for the state of mode 2027. It SHALL also ask for the cursor position after one grapheme of each width class. It SHALL end the queries with a device attributes query. If the terminal knows mode 2027 and the mode is off, the client SHALL turn it on. The pane SHALL use the widths that the terminal gave. A bracketed paste that comes during the queries SHALL be one paste, as the requirement "A paste is one edit" of `input-editor` says. If a paste is still open when the client stops the queries, the client SHALL read on to the end of the paste, for at most 500 ms more.

#### Scenario: A terminal that adds code point widths
- **GIVEN** a terminal that answers the cursor position query after `👨‍👩‍👧` at column 9, so the width of that class is 8
- **WHEN** the client starts, and the game sends the line `👨‍👩‍👧x`
- **THEN** the cell of `x` is at column 9

#### Scenario: A terminal that does not answer
- **GIVEN** a terminal that answers no query
- **WHEN** the user starts `mu client`
- **THEN** the top row shows the worlds
- **AND** after the game line `👨‍👩‍👧x`, the cell of `x` is at column 3, because the `unicode-width` value of `👨‍👩‍👧` is 2

#### Scenario: A late answer
- **GIVEN** a terminal that answers the queries only after the client draws its first frame
- **WHEN** the answers arrive
- **THEN** the editor stays empty

#### Scenario: A paste during the queries
- **GIVEN** a terminal that sends a bracketed paste of `one` CR `two` before its answers
- **WHEN** the client starts
- **THEN** the editor holds the lines `one` and `two`
- **AND** the server records no event

#### Scenario: A paste split across reads
- **GIVEN** a terminal that sends a bracketed paste in parts, with a cut at any byte
- **WHEN** the client reads the parts
- **THEN** the client gives one paste with the whole text

## ADDED Requirements

### Requirement: A send with no reply is refused
If the server gives no reply to a text within 30 seconds, the client SHALL count the text as refused. The reason SHALL say that the game can have the line or not.

#### Scenario: A server that does not answer
- **GIVEN** a call that sends `one` and never ends
- **WHEN** 30 seconds pass
- **THEN** the reply refuses `one`, and its reason contains `no answer in 30 s`

#### Scenario: A reply near the limit
- **GIVEN** a call that sends `one`
- **WHEN** the server answers 1 ms before the limit, or 1 ms after it
- **THEN** the reply before the limit takes `one`, and the reply after it refuses `one`

### Requirement: The top row counts the sends that wait
For each world with texts that wait behind the text with the server, the top row SHALL show their count as `(N waiting)` after the world.

#### Scenario: Two lines wait
- **GIVEN** the user pressed Alt+Enter in `alpha/aria` on the lines `one`, `two` and `three`, and the server did not answer `one` yet
- **THEN** the top row shows `[alpha/aria] (2 waiting)`
- **WHEN** the server answers `one` and `two`
- **THEN** the top row shows `[alpha/aria]` with no count
