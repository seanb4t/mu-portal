## MODIFIED Requirements

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
- **THEN** the editor holds `hi` alone
- **AND** after Up the editor holds `next`

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

### Requirement: A refused text stops the sends after it
If the server refuses a text, the client SHALL send none of the texts that wait for the same world. A send with no reply in time counts as a refusal. The requirement "A send with no reply is refused" gives the time. The client SHALL put the refused text alone in the editor. It SHALL put each text that waited behind it, and then the text that the editor held, into the input history of that world. Up SHALL then give them in that order. It SHALL show the reason as a line with the prefix `-- `, and the count of the texts in the history. A key that sends after that SHALL send at once. If the event stream ends, or the server resets, the client SHALL take back each text that waits behind the text with the server. The first of them, in the order of the worlds, SHALL go alone in the editor. Each other one SHALL go into the input history of its world. The text that the editor held SHALL go into the history of the world in focus. The client SHALL show a line with the prefix `-- ` that holds their count and the count of each world. The text with the server SHALL stay, and its reply SHALL act on it alone.

#### Scenario: A line that the server refuses
- **GIVEN** the editor holds the lines `one`, `two` and `three`, and a test server that accepts `one` and refuses `two`
- **WHEN** the user presses Alt+Enter
- **THEN** the server gets `one` and `two`, and it does not get `three`
- **AND** the screen shows a line that starts with `-- `
- **AND** the editor holds `two` alone
- **AND** after Up the editor holds `three`

#### Scenario: The last line is refused
- **GIVEN** the user pressed Alt+Enter on the lines `one` and `two`, and the server accepted `one`
- **WHEN** the user types `later`, and the server refuses `two`
- **THEN** the editor holds `two` alone
- **AND** after Up the editor holds `later`

#### Scenario: A refused send of Enter with a send after it
- **GIVEN** the user typed `one` and pressed Enter, typed `two` and pressed Enter, and the server did not answer yet
- **WHEN** the user types `later`, and the server refuses `one`
- **THEN** the server does not get `two`
- **AND** the editor holds `one` alone
- **AND** after Up the editor holds `two`, and after a second Up it holds `later`

#### Scenario: The sends stay apart
- **GIVEN** the user typed `a`, Ctrl+J and `b` and pressed Enter, then typed `c` and pressed Enter, and the server did not answer yet
- **WHEN** the server refuses the first text
- **THEN** the editor holds the lines `a` and `b`
- **AND** after Up the editor holds `c` alone

#### Scenario: The stream ends while lines wait
- **GIVEN** the user pressed Alt+Enter on the lines `one`, `two` and `three`, and the server did not answer `one` yet
- **WHEN** the user types `later`, and the event stream ends
- **THEN** the editor holds `two` alone
- **AND** the screen shows a line that starts with `-- ` and contains `2 lines that waited`
- **AND** after Up the editor holds `three`, and after a second Up it holds `later`
- **WHEN** the server answers `one`
- **THEN** the client sends no text

#### Scenario: Lines of two worlds wait
- **GIVEN** lines wait behind a send in `beta/bryn`, and after that behind a send in `alpha/aria`
- **WHEN** the event stream ends
- **THEN** the editor holds the line of `alpha/aria`
- **AND** the screen shows `1 line of alpha/aria, 1 line of beta/bryn`
- **AND** after Ctrl+N to `beta/bryn` and Up, the editor holds the line of `beta/bryn`

#### Scenario: The server resets while lines wait
- **GIVEN** the user pressed Alt+Enter on the lines `one`, `two` and `three`, and the server did not answer `one` yet
- **WHEN** the server resets
- **THEN** the editor holds `two` alone
- **AND** after Up the editor holds `three`

#### Scenario: A refusal after the stream ended
- **GIVEN** the user pressed Alt+Enter on the lines `one`, `two` and `three`, the server did not answer `one` yet, and the event stream ended
- **WHEN** the server refuses `one`
- **THEN** the editor holds `one` alone
- **AND** after Up the editor holds `two`, and after a second Up it holds `three`

## ADDED Requirements

### Requirement: Each world keeps an input history
The client SHALL keep an input history for each world, in memory. Each key that sends SHALL add its whole text to the history of the world that it goes to. The client SHALL NOT add an empty text, or a text equal to the newest entry. A history SHALL keep at most 1000 entries, and drop the oldest first. A command of the client SHALL go into the history of the world in focus.

#### Scenario: A send enters the history
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `one` and presses Enter, then presses Up
- **THEN** the editor holds `one`

#### Scenario: A pose is one entry
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `a`, Ctrl+J and `b`, presses Alt+Enter, then presses Up
- **THEN** the editor holds the lines `a` and `b`

#### Scenario: Each world has its own history
- **GIVEN** the user sent `one` to `alpha/aria`
- **WHEN** the user presses Ctrl+N to `beta/bryn` and presses Up
- **THEN** the editor is empty

#### Scenario: No repeat
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user sends `one`, `one` and `two`, then presses Up twice
- **THEN** the editor holds `one`
- **AND** after a third Up the editor still holds `one`

### Requirement: Up and Down walk the history
A line here is a line between line breaks, not a wrapped row. If the cursor is on the first line of the editor, Up SHALL replace the text with the next older entry of the world in focus. If the cursor is on the last line, Down SHALL replace it with the next newer entry. Past the newest entry, Down SHALL bring back the text that the editor held before the first Up. On other lines, Up and Down SHALL move the cursor. With no older entry, Up SHALL NOT change the text.

#### Scenario: Back to the draft
- **GIVEN** the user sent `one` to `alpha/aria`, and typed `draft`
- **WHEN** the user presses Up and then Down
- **THEN** the editor holds `draft`

#### Scenario: Up inside a pose moves the cursor
- **GIVEN** the editor holds the lines `a` and `b`, the cursor on `b`, and the history holds `one`
- **WHEN** the user presses Up
- **THEN** the editor still holds `a` and `b`, and the cursor is on `a`

### Requirement: Alt and a digit give the focus to a world
Alt+1 to Alt+9 SHALL give the focus to the world at that place in the top row. If no world is at that place, the key SHALL do nothing.

#### Scenario: The second world
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, and the world in focus is `alpha/aria`
- **WHEN** the user presses Alt+2
- **THEN** the top row shows `[beta/bryn]`

#### Scenario: No world at that place
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, and the world in focus is `alpha/aria`
- **WHEN** the user presses Alt+5
- **THEN** the top row shows `[alpha/aria]`

### Requirement: Keys scroll the pane to its ends
Alt+Home SHALL scroll the pane in focus to its oldest line, and Alt+End SHALL return it to the bottom.

#### Scenario: Top and bottom
- **GIVEN** the client shows `alpha/aria`, and the world has more lines than the pane shows
- **WHEN** the user presses Alt+Home
- **THEN** the pane shows the oldest line of the world on its first row
- **WHEN** the user presses Alt+End
- **THEN** the pane shows the newest line on its last row

### Requirement: The echo line shows each line of a pose
The client SHALL split the echo line of a send at each `line_break` of its world, and show each part on its own row. The first row SHALL have the prefix `> ` and each other row two spaces. A world with no known `line_break` SHALL show the text in one row.

#### Scenario: A pose of two lines
- **GIVEN** the world in focus is the fake world `alpha/aria`, whose `line_break` is `%r`
- **WHEN** the user types `one`, presses Ctrl+J, types `two` and presses Enter
- **THEN** the screen shows a row `> one` and the next row `  two`
- **AND** the screen shows no row that holds `one%rtwo` and starts with `> `

### Requirement: The top row shows the first list
The client SHALL open the screen before its first list of the worlds. Until that list ends, the top row SHALL show `listing worlds`. Keys typed in that time SHALL edit the editor, and a key that sends SHALL show that no world is in focus.

#### Scenario: Before the first list
- **GIVEN** the client opened the screen, and its first list of the worlds did not end yet
- **WHEN** the client draws the screen
- **THEN** the top row shows `listing worlds`
- **WHEN** the list ends with the world `alpha/aria`
- **THEN** the top row shows `[alpha/aria]`
