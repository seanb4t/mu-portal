## MODIFIED Requirements

### Requirement: Enter sends the input to the world in focus
If the user presses Enter, the client SHALL send the whole text of the editor to the world in focus and clear the editor. Ctrl+M SHALL do the same as Enter. A line break of the text SHALL go to the server as LF. An empty editor SHALL send an empty line. If the server refuses the text, the client SHALL put the text back in the editor of that world. The requirement "A refused text stops the sends after it" gives the rule. It SHALL show the reason as a line with the prefix `-- `.

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
If the server refuses a text, the client SHALL send none of the texts that wait for the same world. A send with no reply in time counts as a refusal. The requirement "A send with no reply is refused" gives the time. The client SHALL put the refused text alone in the editor of the refused world, with the cursor at its start. It SHALL put each text that waited behind it, and then the text that this editor held, into the input history of that world. Up in that world SHALL then give them in that order. The client SHALL NOT change the editor or the history of another world. It SHALL show the reason as a line with the prefix `-- ` in the pane of the refused world. If texts went into the history, the line SHALL also hold their count, as `(<n> in history)`. A key that sends after that SHALL send at once. If the event stream ends, or the server resets, the client SHALL take back each text that waits behind the text with the server. For each world, the first of them SHALL go alone in the editor of that world, with the cursor at its start, and the others and the text that this editor held SHALL go into the input history of that world. During a walk of the history, the text that an editor held is the draft that the walk saved, not the entry on screen. The client SHALL show a line with the prefix `-- ` that holds their count and the count of each world. The text with the server SHALL stay, and its reply SHALL act on it alone.

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
- **THEN** the server records no player event after `one`

#### Scenario: Lines of two worlds wait
- **GIVEN** the line `b2` waits behind the send `b1` in `beta/bryn`, and after that the line `a2` waits behind the send `a1` in `alpha/aria`
- **WHEN** the event stream ends
- **THEN** the editor holds `a2`
- **AND** the screen shows `1 line of alpha/aria, 1 line of beta/bryn`
- **AND** after Ctrl+N to `beta/bryn`, the editor holds `b2`

#### Scenario: A refusal out of focus
- **GIVEN** the world in focus is `alpha/aria`, and the line `b2` waits behind the send `b1` in `beta/bryn`
- **WHEN** the user types `typed`, and the server refuses `b1`
- **THEN** the editor holds `typed`
- **AND** after Ctrl+N to `beta/bryn`, the editor holds `b1` alone
- **AND** after Up the editor holds `b2`

#### Scenario: The server resets while lines wait
- **GIVEN** the user pressed Alt+Enter on the lines `one`, `two` and `three`, and the server did not answer `one` yet
- **WHEN** the server resets
- **THEN** the editor holds `two` alone
- **AND** after Up the editor holds `three`

#### Scenario: A refusal during a walk of the history
- **GIVEN** the user sent `old` to `alpha/aria`, then typed `one` and pressed Enter, and the server did not answer yet
- **WHEN** the user types `draft`, presses Up, and the server refuses `one`
- **THEN** the editor holds `one` alone
- **AND** the screen shows a line that starts with `-- ` and contains `(1 in history)`
- **AND** after Up the editor holds `draft`

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

#### Scenario: No empty entry
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user sends `one` and then an empty line, and presses Up
- **THEN** the editor holds `one`

#### Scenario: Each world has its own history
- **GIVEN** the user sent `one` to `alpha/aria`
- **WHEN** the user presses Ctrl+N to `beta/bryn` and presses Up
- **THEN** the editor is empty

#### Scenario: No repeat
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user sends `one`, `one` and `two`, then presses Up twice
- **THEN** the editor holds `one`
- **AND** after a third Up the editor still holds `one`

#### Scenario: An escaped slash in the history
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user sends `//me waves`, then presses Up and Enter
- **THEN** the server records the player event `/me waves` twice for `alpha/aria`

#### Scenario: Down with no walk
- **GIVEN** the world in focus is the fake world `alpha/aria`, and its history holds `one`
- **WHEN** the user types `draft` and presses Down
- **THEN** the editor holds `draft`

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

### Requirement: Each world keeps its own editor
The client SHALL keep a pose editor for each world, with its text, its cursor and its walk of the history. The editor on screen SHALL be the editor of the world in focus. A change of the focus SHALL NOT change the text of an editor, and SHALL NOT end a walk. A send or a command during a walk SHALL end it, and the draft SHALL go into the history before the sent text. The text typed while no world has the focus SHALL go into the editor of the first world that gets the focus.

#### Scenario: A send during a walk
- **GIVEN** the user sent `one` to `alpha/aria`, typed `draft` and pressed Up
- **WHEN** the user presses Enter, and then presses Up twice
- **THEN** the server records the player event `one` twice for `alpha/aria`
- **AND** the editor holds `draft`

#### Scenario: A focus change keeps each editor
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, and the world in focus is `alpha/aria`
- **WHEN** the user types `draft` and presses Ctrl+N to `beta/bryn`
- **THEN** the editor is empty
- **WHEN** the user types `other` and presses Ctrl+P to `alpha/aria`
- **THEN** the editor holds `draft`
- **AND** after Ctrl+N the editor holds `other`

#### Scenario: A walk stays with its world
- **GIVEN** the user sent `one` to `alpha/aria`, typed `draft` and pressed Up
- **WHEN** the user presses Ctrl+N to `beta/bryn` and Ctrl+P to `alpha/aria`
- **THEN** the editor holds `one`
- **AND** after Down the editor holds `draft`

### Requirement: Alt and a digit give the focus to a world
Alt+1 to Alt+9 SHALL give the focus to the world at that place in the top row. If no world is at that place, the key SHALL do nothing.

#### Scenario: The second world
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, and the world in focus is `alpha/aria`
- **WHEN** the user presses Alt+2
- **THEN** the top row shows `[beta/bryn]`

#### Scenario: No world at that place
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, and the world in focus is `alpha/aria`
- **WHEN** the user presses Alt+3
- **THEN** the top row shows `[alpha/aria]`

#### Scenario: Back to the first world
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, and the world in focus is `beta/bryn`
- **WHEN** the user presses Alt+1
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

#### Scenario: The echo of a pose of two lines
- **GIVEN** the world in focus is the fake world `alpha/aria`, whose `line_break` is `%r`
- **WHEN** the user types `one`, presses Ctrl+J, types `two` and presses Enter
- **THEN** the screen shows a row `> one` and the next row `  two`
- **AND** the screen shows no row that holds `one%rtwo` and starts with `> `

### Requirement: The top row shows the first list
The client SHALL open the screen before its first list of the worlds for the screen. `mu client` can list the worlds once before, within the time of `connect_wait_seconds` in `client.toml`, to decide whether to start a server. Until that list ends, the top row SHALL show `listing worlds`, and no world SHALL have the focus, also if events of a world arrive. Keys typed in that time SHALL edit an editor of no world, and a key that sends SHALL show that no world is in focus.

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
