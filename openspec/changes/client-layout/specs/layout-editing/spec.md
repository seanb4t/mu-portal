# Spec Delta

## Purpose

Defines how a player changes the layout inside the terminal client: the commands, the mouse actions and arrange mode, and when the client saves a change to the server.

## ADDED Requirements

### Requirement: The place command puts a block on the screen
The command `/place <sentence>` SHALL take one placement sentence that is not a first line. If the block is on the screen, the command SHALL move it. If not, the command SHALL add it as the last line of the plan. The screen SHALL show the result at once. A bad sentence SHALL change nothing, and SHALL show a notice with the reason.

#### Scenario: Add the notices
- **GIVEN** the layout `classic` is in use
- **WHEN** the user runs the command `/place output as notices right of output, 30%`
- **THEN** the screen shows a divider, and the right pane shows `[notices]`

#### Scenario: Move the tabs
- **GIVEN** the layout `classic` is in use, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user runs the command `/place tabs below output`
- **THEN** the row above the editor shows `[alpha/aria]`

#### Scenario: A bad sentence
- **GIVEN** the layout `classic` is in use
- **WHEN** the user runs the command `/place spawns right of output`
- **THEN** the screen shows a line that starts with `-- ` and contains `spawns`
- **AND** the layout does not change

### Requirement: The pane commands hide and resize a block
`/pane hide <id>` SHALL take the block off the screen, and the other blocks SHALL take its room. It SHALL refuse the last editor of the layout with a notice. If the hidden editor had the input focus, the first editor of the plan SHALL get it. `/pane size <id> <size>` SHALL set the size on the line of the block, and a size of `fill` SHALL remove it. For the block of the first line, it SHALL change nothing, and SHALL show a notice that names the blocks beside it that take a size.

#### Scenario: Hide the tabs
- **GIVEN** the layout `classic` is in use
- **WHEN** the user runs the command `/pane hide tabs`
- **THEN** row 1 shows the output

#### Scenario: The editor stays
- **GIVEN** the layout `classic` is in use
- **WHEN** the user runs the command `/pane hide editor`
- **THEN** the screen shows a line that starts with `-- ` and contains `editor`
- **AND** the editor still shows

#### Scenario: Hide a second editor
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `editor as scratch right of editor`, and `scratch` has the input focus
- **WHEN** the user runs the command `/pane hide scratch` and types `x`
- **THEN** one editor shows, and it holds `x`

#### Scenario: Resize the notices
- **GIVEN** the layout `sidebar` is in use, in a terminal of 80 columns
- **WHEN** the user runs the command `/pane size notices 50%`
- **THEN** the divider is in column 40

#### Scenario: A size of the first block
- **GIVEN** the layout `sidebar` is in use
- **WHEN** the user runs the command `/pane size output 60%`
- **THEN** the layout does not change
- **AND** the screen shows a line that starts with `-- ` and contains `notices`

### Requirement: The plan commands list and edit the lines
`/layout plan` SHALL show each line of the plan with its number and the id of its block. `/layout insert <number> <sentence>` SHALL put a new line at that number, and the lines from there SHALL move down. `/layout delete <line>` SHALL remove a line. `/layout edit <line> <sentence>` SHALL replace a line. A line SHALL be named by its number or by the id of its block. If the new plan is bad, the command SHALL change nothing, and SHALL show the reason with the number of the bad line.

#### Scenario: List the plan
- **GIVEN** the layout `classic` is in use
- **WHEN** the user runs the command `/layout plan`
- **THEN** the screen shows `1. output`, `2. tabs above output` and `3. editor below output`

#### Scenario: Insert in the middle
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `statusline below editor`
- **WHEN** the user runs the command `/layout insert 2 output as notices right of output`
- **THEN** the plan holds `output as notices right of output` as line 2
- **AND** the notices are beside the output only, and the editor takes the whole width

#### Scenario: Edit a line by its id
- **GIVEN** the layout `sidebar` is in use
- **WHEN** the user runs the command `/layout edit notices output as notices left of output, 20%`
- **THEN** the notices are left of the output
- **AND** each other line of the plan is as before

#### Scenario: Delete a line that others need
- **GIVEN** the layout `classic` is in use
- **WHEN** the user runs the command `/layout delete 1`
- **THEN** the layout does not change
- **AND** the screen shows a line that starts with `-- ` and contains `line 2` and `output`

### Requirement: A change keeps the other lines of the plan
A change from `/place`, `/pane`, the mouse or arrange mode asks for one screen. It SHALL change only the line of its block, if that plan gives the screen. A line of a moved block SHALL go to the end of the plan if its new anchor comes later. If no such plan exists, mu SHALL write the whole plan again from the screen, and the client SHALL show a notice that the lines changed.

#### Scenario: A resize changes one line
- **GIVEN** the layout `sidebar` is in use
- **WHEN** the user runs the command `/pane size notices 40%`
- **THEN** line 5 of the plan is `output as notices right of output, 40%`, and lines 1 to 4 are as before

#### Scenario: A hide of an anchor writes the plan again
- **GIVEN** the layout in use holds the lines `output`, `output as notices right of output`, `tabs above notices` and `editor at bottom`
- **WHEN** the user runs the command `/pane hide notices`
- **THEN** the screen shows the tabs, the output and the editor, and no notices
- **AND** the screen shows a line that starts with `-- ` and contains `plan`

### Requirement: A key or a click picks the block of a stack
The entry `pane-next` SHALL show the next block of the stack of `output`, and after the last block the first. A click on a name in a title row SHALL show that block.

#### Scenario: A click on a title
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `output as notices with output`
- **WHEN** the user clicks `notices` in the title row
- **THEN** the title row shows `output [notices]`

### Requirement: A drag of a divider resizes two panes
A press of the left button on a divider, then a drag left or right, SHALL move the divider with the mouse. The two panes SHALL change at each report of the mouse. Neither pane SHALL get smaller than 10 columns. At the release, the size of the placed block beside the divider SHALL be a whole percent of its split. A drag that starts on a divider SHALL select no text.

#### Scenario: A drag to the left
- **GIVEN** the layout `sidebar` is in use, in a terminal of 80 columns and 24 rows
- **WHEN** the user presses the left button on the divider, drags to column 40 and releases
- **THEN** the divider is in column 40
- **AND** `mu layout show` of the layout in use holds `output as notices right of output, 50%`

#### Scenario: A drag past the smallest pane
- **GIVEN** the layout `sidebar` is in use, in a terminal of 80 columns
- **WHEN** the user drags the divider to column 78 and releases
- **THEN** the pane of `notices` is 10 columns wide

### Requirement: A drag of a title row resizes two panes
A pane with a title row can be below another pane of the same split. Then a drag of its title row up or down SHALL move the border between the two. Neither pane SHALL get smaller than 2 rows. A press and a release on one name with no move between SHALL be a click.

#### Scenario: A drag down
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `output as notices below output, 50%`, in a terminal of 80 columns and 24 rows
- **WHEN** the user drags the title row of `notices` down by 4 rows and releases
- **THEN** the pane of `notices` is 4 rows smaller

### Requirement: A drag inside a pane still selects text
Outside arrange mode, a drag that starts inside a pane of `output`, below its title row, SHALL select text as before.

#### Scenario: A selection beside a divider
- **GIVEN** the layout `sidebar` is in use, and the fake world `alpha/aria` sent `Welcome, aria.`
- **WHEN** the user drags over `Welcome` in the pane of `output` and releases
- **THEN** the clipboard write holds `Welcome`
- **AND** the divider did not move

### Requirement: Arrange mode marks each block
The command `/arrange` and the entry `arrange`, with the default key Alt+L, SHALL start arrange mode. In arrange mode, each block on the screen SHALL show a letter at its top left corner, from `A` in the order of the screen. The last row of the screen SHALL show the keys of the mode. Typed text SHALL NOT reach the editor, and the editor SHALL keep its text. Events of the worlds SHALL still show.

#### Scenario: The letters
- **GIVEN** the layout `classic` is in use
- **WHEN** the user presses Alt+L
- **THEN** the tabs show `A`, the output shows `B`, and the editor shows `C`
- **AND** the last row contains `Enter` and `Esc`

#### Scenario: Typing in arrange mode
- **GIVEN** the user typed `draft` and then pressed Alt+L
- **WHEN** the user presses `z`, and then Esc
- **THEN** the editor holds `draft`

### Requirement: Arrange mode moves a block with three keys
The letter of a block SHALL pick it. Then the letter of a second block and an arrow key SHALL place the first block on that side of the second. In place of the arrow, `=` SHALL stack the first block with the second. The keys of the mode that are not letters leave each letter free for a block. The screen SHALL show the result at once. A move that the layout rules refuse SHALL change nothing, and the last row SHALL show the reason.

#### Scenario: The tabs go below the output
- **GIVEN** the layout `classic` is in use, the world in focus is `alpha/aria`, and arrange mode is on
- **WHEN** the user presses `a`, `b` and the Down key
- **THEN** the row above the editor shows `[alpha/aria]`

#### Scenario: A refused stack
- **GIVEN** the layout `classic` is in use, and arrange mode is on
- **WHEN** the user presses `c`, `b` and `=`
- **THEN** the layout does not change
- **AND** the last row contains `editor`

### Requirement: Arrange mode adds, hides and resizes
With no block picked, `+` SHALL list the blocks that are not on the screen, each with a letter, and that letter SHALL pick the block. With a block picked, `>` SHALL make it 5 percent of its split larger, and `<` SHALL make it 5 percent smaller. `-` SHALL hide it. The key of the entry `undo` SHALL undo the last change of this mode.

#### Scenario: Add the statusline
- **GIVEN** the layout `classic` is in use, and arrange mode is on
- **WHEN** the user presses `+`, the letter of `statusline`, `c` and the Down key
- **THEN** the bottom row above the keys of the mode is the statusline

#### Scenario: Undo
- **GIVEN** the layout `classic` is in use, arrange mode is on, and the user hid the tabs with `a` and `-`
- **WHEN** the user presses Ctrl+Z
- **THEN** the tabs show again

### Requirement: Arrange mode moves a block with a drag
In arrange mode, a drag that starts on a block and ends on another block SHALL move the first block. A release in the outer third of the second block on one side SHALL place it on that side. A release in the middle SHALL stack it. During the drag, the client SHALL mark the part of the second block where the first block lands.

#### Scenario: A drag to the right edge
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `output as notices below output`, and arrange mode is on
- **WHEN** the user drags from the pane of `notices` to the right edge of the pane of `output` and releases
- **THEN** the pane of `notices` is right of the pane of `output`

### Requirement: Enter saves and Esc restores
In arrange mode, Enter SHALL leave the mode and save the layout. Esc with no block picked SHALL leave the mode and restore the layout from before the mode. Esc with a block picked SHALL only drop the pick.

#### Scenario: Esc restores
- **GIVEN** the layout `classic` is in use, arrange mode is on, and the user hid the tabs
- **WHEN** the user presses Esc
- **THEN** the tabs show again, and the server got no save

### Requirement: Each finished change saves the layout
The client SHALL save the layout after each of these: a command that changed it, the release of a drag that resized a pane, and Enter in arrange mode. If the layout on the screen is a player layout, the save SHALL replace it. If it is built in, the save SHALL write a new layout, and SHALL set the choice in use to that layout in the device layer.

#### Scenario: A save from a built-in layout
- **GIVEN** `clients/` holds no layout, and `mu client --device laptop` shows the layout `classic`
- **WHEN** the user runs the command `/pane hide tabs`
- **THEN** `clients/layouts/laptop.toml` holds the lines `output` and `editor below output`
- **AND** `mu config show --device laptop` holds `layout = "laptop"` with the source `clients/devices/laptop.toml`

#### Scenario: A save to a player layout
- **GIVEN** `clients/global.toml` holds `layout = "mine"`, and `clients/layouts/mine.toml` holds the lines `output` and `editor below output`
- **WHEN** the user of `mu client --device laptop` runs the command `/place tabs at top`
- **THEN** `clients/layouts/mine.toml` holds three lines, and the last is `tabs at top`
- **AND** no file `clients/devices/laptop.toml` exists

#### Scenario: A second client gets the change
- **GIVEN** two clients show the player layout `mine`
- **WHEN** the user of the first client runs the command `/pane hide tabs`
- **THEN** the second client shows no tabs

### Requirement: A failed save keeps the screen
If the server refuses a save or does not answer, the client SHALL keep the changed layout on the screen. It SHALL show a notice with the reason.

#### Scenario: The server refuses
- **GIVEN** `clients/devices/laptop.toml` holds `echo = `, and `mu client --device laptop` shows the layout `classic`
- **WHEN** the user runs the command `/pane hide tabs`
- **THEN** the screen shows no tabs
- **AND** the screen shows a line that starts with `-- ` and contains `clients/devices/laptop.toml`

### Requirement: The layout commands choose and name a layout
`/layout use <name>` SHALL set the field `layout` of the device layer. `/layout save <name>` SHALL save the layout on the screen under that name, and SHALL set the choice in use to it. `/layout list` SHALL show the name of each layout. The entry `layout-next` SHALL set the choice in use to the next layout in the order of the names, and after the last the first. An unknown name and the name of a built-in layout in a save SHALL give a notice and change nothing.

#### Scenario: Use a built-in layout
- **GIVEN** `mu client --device laptop` shows the layout `classic`
- **WHEN** the user runs the command `/layout use status`
- **THEN** the bottom row is the statusline
- **AND** `mu config show --device laptop` holds `layout = "status"`

#### Scenario: Save under a name
- **GIVEN** `mu client --device laptop` shows the layout `classic`
- **WHEN** the user runs the command `/layout save evening`
- **THEN** `clients/layouts/evening.toml` holds the lines of `classic`

#### Scenario: A save over a built-in name
- **GIVEN** `mu client --device laptop` shows the layout `classic`
- **WHEN** the user runs the command `/layout save focus`
- **THEN** the screen shows a line that starts with `-- ` and contains `focus`
- **AND** no file `clients/layouts/focus.toml` exists

### Requirement: The name of a layout saved from a built-in layout
If the field `layout` is the choice in use, the new layout SHALL have the name of the device. If an entry of `[world-layouts]` is the choice in use, the name SHALL have three parts: the device name, then `.`, then the key of the entry. Each `/` of the key SHALL become `.`. If that name has more than 64 characters, the client SHALL NOT save. It SHALL show a notice that names `/layout save`.

#### Scenario: A save from the built-in layout of one world
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `"alpha/aria" = "classic"`, and a server has the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user of `mu client --device laptop` runs the command `/pane hide tabs`
- **THEN** `clients/layouts/laptop.alpha.aria.toml` holds the lines `output` and `editor below output`
- **AND** `mu config show --device laptop` holds `world-layouts."alpha/aria" = "laptop.alpha.aria"` with the source `clients/devices/laptop.toml`
- **AND** it holds `layout = "classic"` with the source `default`

#### Scenario: The other world keeps its layout
- **GIVEN** the client of the scenario "A save from the built-in layout of one world", after the command
- **WHEN** the user runs the entry `focus-next`
- **THEN** row 1 shows the tab `[beta/bryn]`

#### Scenario: A name that is too long
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `"alpha/aria" = "classic"`, and the device name of the client has 60 characters
- **WHEN** the user runs the command `/pane hide tabs`
- **THEN** the screen shows no tabs, and a line that starts with `-- ` and contains `/layout save`
- **AND** the directory `clients/layouts` holds no file

### Requirement: The layout here command sets the layout of one world
`/layout here <name>` SHALL set the entry of the world in focus in `[world-layouts]` of the device layer, and the client SHALL draw that layout. `/layout here` with no name SHALL remove that entry. With no world in focus, and for an unknown name, the command SHALL give a notice and change nothing. If an entry is the choice in use, `/layout use` SHALL also show a notice that names `/layout here`.

#### Scenario: A layout for this world
- **GIVEN** `mu client --device laptop` shows the fake worlds `alpha/aria` and `beta/bryn` with the layout `classic`, and `alpha/aria` has the focus
- **WHEN** the user runs the command `/layout here focus`
- **THEN** row 1 shows no tab
- **AND** `mu config show --device laptop` holds `world-layouts."alpha/aria" = "focus"` with the source `clients/devices/laptop.toml`

#### Scenario: Remove the layout of this world
- **GIVEN** the client of the scenario "A layout for this world", after the command
- **WHEN** the user runs the command `/layout here`
- **THEN** row 1 shows `[alpha/aria]`
- **AND** `mu config show --device laptop` holds no line that starts with `world-layouts`

#### Scenario: No world in focus
- **GIVEN** `mu client --device laptop` on a server with no world
- **WHEN** the user runs the command `/layout here focus`
- **THEN** the screen shows a line that starts with `-- ` and contains `world`
- **AND** no file `clients/devices/laptop.toml` exists

#### Scenario: An unknown layout for this world
- **GIVEN** `mu client --device laptop` shows the fake world `alpha/aria`
- **WHEN** the user runs the command `/layout here gone`
- **THEN** the screen shows a line that starts with `-- ` and contains `gone`
- **AND** no file `clients/devices/laptop.toml` exists

#### Scenario: A use behind an entry
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `"alpha/aria" = "focus"`, and `mu client --device laptop` shows the fake world `alpha/aria`
- **WHEN** the user runs the command `/layout use status`
- **THEN** `mu config show --device laptop` holds `layout = "status"`
- **AND** the screen shows a line that starts with `-- ` and contains `/layout here`
- **AND** row 1 shows no tab

#### Scenario: The next layout of a world entry
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `"alpha/aria" = "classic"`, the server holds no layout file, and `mu client --device laptop` shows the fake world `alpha/aria`
- **WHEN** the user runs the entry `layout-next`
- **THEN** `mu config show --device laptop` holds `world-layouts."alpha/aria" = "focus"` and `layout = "classic"`
