# Spec Delta

## Purpose

Defines the layout of the terminal client: its blocks, panes, splits and stacks, the placement sentences that write it, the built-in layouts, and how the client draws it. Also defines the `[tabs]` label of the world tabs.

## ADDED Requirements

### Requirement: The blocks of the screen
The blocks SHALL be `tabs`, `output`, `editor` and `statusline`. `tabs` shows the row of world tabs. `output` shows lines of text. `editor` takes typed text. A layout SHALL hold `editor` one to four times, `output` at most eight times, and each other block at most once. A block that the layout does not hold SHALL NOT show, and the keys and commands SHALL work the same without it.

#### Scenario: No tabs
- **GIVEN** the layout in use holds the lines `output` and `editor below output`
- **AND** a server has the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client`, presses Ctrl+N, types `hi` and presses Enter
- **THEN** the screen shows no row with `alpha/aria`
- **AND** the server records a player event `hi` for `beta/bryn`

#### Scenario: The editor alone
- **GIVEN** the layout in use holds the line `editor`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `hi` and presses Enter
- **THEN** the server records a player event `hi` for `alpha/aria`

#### Scenario: Nine outputs
- **GIVEN** `clients/layouts/mine.toml` holds `editor` and nine lines that each place an `output` with its own id
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `output`

### Requirement: An output shows the lines of its id
Each line of text SHALL have the name of a stream. The lines of a world that no rule sends elsewhere are in the stream of that world, and the notices of the client are in the stream `notices`. An `output` SHALL show the stream with its id, for the world in focus. The first `output` of the plan SHALL also show the stream of the world, and each stream that no `output` has as its id. Each `output` after the first SHALL set its own id with `as`.

#### Scenario: One output shows the notices too
- **GIVEN** the layout `classic` is in use, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user runs the command `/focus nowhere/nobody`
- **THEN** the pane of `output` shows `Welcome, aria.` and a line that starts with `-- `

#### Scenario: An output for the notices
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `output as notices right of output`
- **AND** the world in focus is the fake world `alpha/aria`
- **WHEN** the user runs the command `/focus nowhere/nobody`
- **THEN** the pane of `notices` shows a line that starts with `-- `
- **AND** the pane of `output` shows `Welcome, aria.` and no line that starts with `-- `

#### Scenario: An output with no stream yet
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `output as channels right of output`
- **WHEN** the client draws the screen
- **THEN** the pane of `channels` shows its title row and no other line

### Requirement: A line stays in the output where it arrived
The client SHALL put each line into one `output` when the line arrives. That SHALL be the `output` that shows the stream of the line at that time, for the world of the line. A later change of the layout SHALL NOT move the line. A notice SHALL be a line of the stream `notices` for the world in focus.

#### Scenario: An output that comes later starts with new lines
- **GIVEN** the layout `classic` is in use, the world in focus is the fake world `alpha/aria`, and the user ran the command `/focus nowhere/nobody`
- **WHEN** the user runs the command `/place output as notices right of output`
- **THEN** the pane of `notices` shows its title row and no other line
- **AND** the pane of `output` shows a line that starts with `-- ` and contains `nowhere/nobody`

#### Scenario: A notice shows under the world in focus only
- **GIVEN** the layout `classic` is in use, a server has the fake worlds `alpha/aria` and `beta/bryn`, and `alpha/aria` has the focus
- **WHEN** the user runs the command `/focus nowhere/nobody`, and then the entry `focus-next`
- **THEN** the pane of `output` shows `Welcome, bryn.` and no line that contains `nowhere/nobody`

### Requirement: A layout can hold several editors
Each `editor` after the first SHALL set its own id with `as`. Each editor SHALL keep its own text for each world, and one more text while no world has the focus. A key that sends SHALL send the text of the editor with the input focus to the world in focus, and SHALL clear only that editor. The editors of a world SHALL share the input history of that world.

#### Scenario: Two editors keep two texts
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `editor as scratch right of editor`
- **AND** the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `one`, presses Alt+E, and types `two`
- **THEN** the left editor holds `one`, and the right editor holds `two`

#### Scenario: Enter sends the editor with the input focus
- **GIVEN** the layout of the scenario "Two editors keep two texts", the editor `editor` holds `one`, and `scratch` holds `two` and has the input focus
- **WHEN** the user presses Enter
- **THEN** the server records a player event `two` for `alpha/aria`
- **AND** the left editor still holds `one`, and the right editor is empty

#### Scenario: One text for each world
- **GIVEN** the layout of the scenario "Two editors keep two texts", the fake worlds `alpha/aria` and `beta/bryn`, and `scratch` holds `two` for `alpha/aria`
- **WHEN** the user presses Ctrl+N
- **THEN** the editor `scratch` is empty
- **WHEN** the user presses Ctrl+P
- **THEN** the editor `scratch` holds `two`

#### Scenario: A second editor with no id
- **GIVEN** `clients/layouts/mine.toml` holds the lines `editor` and `editor above editor`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 2` and `editor`

#### Scenario: Five editors
- **GIVEN** `clients/layouts/mine.toml` holds `editor` and four more lines that each place an editor with its own id
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `editor`

### Requirement: One editor has the input focus
Typed text and each entry that edits or sends SHALL go to the editor with the input focus. At the start, and after a new layout, the first editor of the plan SHALL have it. The entry `editor-next`, with the default key Alt+E, SHALL give it to the next editor of the plan, and after the last the first. A click in an editor SHALL give it to that editor. The terminal cursor SHALL be in the editor with the input focus.

#### Scenario: The key moves the input focus
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `editor as scratch right of editor`
- **WHEN** the user presses Alt+E and types `x`
- **THEN** the editor `scratch` holds `x`, and the terminal cursor is in it

#### Scenario: A click moves the input focus
- **GIVEN** the layout of the scenario "The key moves the input focus"
- **WHEN** the user clicks in the editor `scratch` and types `x`
- **THEN** the editor `scratch` holds `x`

#### Scenario: One editor
- **GIVEN** the layout `classic` is in use
- **WHEN** the user presses Alt+E and types `x`
- **THEN** the editor holds `x`

### Requirement: A layout is a plan of numbered lines
A layout SHALL be a plan: a list of 1 to 32 lines in order. mu SHALL show each line with its number, from 1, and each error about a line SHALL name its number. The first line SHALL be one block, which takes the whole screen. Each later line SHALL place one new block next to an anchor, a block of an earlier line: `<block> above <anchor>`, `below`, `left of` or `right of`. The new block SHALL take a part of the pane of the anchor, on that side.

#### Scenario: The output above the tabs
- **GIVEN** the layout in use holds the lines `output`, `tabs below output` and `editor below tabs`
- **AND** a server has the fake world `alpha/aria`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** row 23 shows `[alpha/aria]`, and `Welcome, aria.` shows above it

#### Scenario: A block beside another
- **GIVEN** the layout in use holds the lines `output`, `tabs above output`, `editor below output` and `output as notices right of output`
- **AND** a server has the fake world `alpha/aria`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** row 1 shows `[alpha/aria]` over the whole width
- **AND** `Welcome, aria.` shows in the left half of the rows between

#### Scenario: The numbers of the lines
- **GIVEN** a server serves `<dir>`
- **WHEN** the user runs `mu layout show classic`
- **THEN** stdout holds `1. output`, `2. tabs above output` and `3. editor below output`

### Requirement: The order of the lines decides the shape
A line SHALL split only the pane that its anchor has after the lines before it. So the same lines in another order can give another screen.

#### Scenario: The editor after the notices
- **GIVEN** the layout in use holds the lines `output`, `output as notices right of output` and `editor below output`
- **WHEN** the client draws the screen
- **THEN** the editor is below the output only, and the notices take the whole height

#### Scenario: The editor before the notices
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `output as notices right of output`
- **WHEN** the client draws the screen
- **THEN** the editor takes the whole width, and the notices are beside the output only

### Requirement: A line can name several anchors
A line SHALL take two or more anchors joined with `and`, such as `output as notices right of output and editor`. The anchors SHALL be neighbors in one split. The new block SHALL stand beside all of them, and the blocks outside the anchors SHALL keep their places.

#### Scenario: A block beside two rows
- **GIVEN** the layout in use holds the lines `output`, `tabs above output`, `editor below output`, `statusline below editor` and `output as notices right of output and editor, 33%`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** rows 1 and 24 take the whole width
- **AND** the pane of `notices` takes columns 55 to 80 of rows 2 to 23, and a divider shows in column 54
- **AND** the editor takes columns 1 to 53 of row 23

#### Scenario: Anchors that are not neighbors
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output`, `tabs above output`, `editor below output` and `output as notices right of tabs and editor`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 4`, `tabs` and `editor`

### Requirement: Each block has a stable id
Each block of a layout SHALL have an id that is unique in the layout. The default id SHALL be the name of the block. A line SHALL set another id with `as`, such as `output as side right of output`. An id SHALL be 1 to 32 characters from `a` to `z`, `0` to `9` and `-`. It SHALL start with a letter, and SHALL NOT be a word of the sentences. An anchor, a command and a call SHALL name a block by its id. No change to another line SHALL change an id.

#### Scenario: An id in a later line
- **GIVEN** the layout in use holds the lines `output`, `editor below output`, `output as side right of output` and `tabs above side`
- **WHEN** the client draws the screen
- **THEN** row 1 shows no tab in the columns of the pane of `output`, and a tab in the columns of the pane of `side`

#### Scenario: An id twice
- **GIVEN** `clients/layouts/mine.toml` holds the lines `editor`, `output as main above editor` and `output as main at right`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 3` and `main`

#### Scenario: An id that is a word of the sentences
- **GIVEN** `clients/layouts/mine.toml` holds the lines `editor` and `output as above above editor`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 2` and `above`

### Requirement: A sentence can place a block at an edge of the screen
The forms `<block> at top`, `<block> at bottom`, `<block> at left` and `<block> at right` SHALL place the new block along that whole edge of the screen. The blocks of the earlier lines SHALL take the rest.

#### Scenario: The statusline at the bottom
- **GIVEN** the layout in use holds the lines `output`, `editor below output`, `output as notices right of output` and `statusline at bottom`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** row 24 is the statusline over the whole width

### Requirement: Blocks can share a pane as a stack
The form `<block> with <block>` SHALL put the new block in the pane of the earlier block. The pane SHALL show one block of the stack at a time, and a title row with the name of each. The name of the block that shows SHALL be in square brackets. Only `output` blocks SHALL stack.

#### Scenario: Notices behind the output
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `output as notices with output`
- **AND** a server has the fake world `alpha/aria`
- **WHEN** the user starts `mu client`
- **THEN** row 1 shows `[output] notices`
- **AND** the screen shows `Welcome, aria.`

#### Scenario: The editor does not stack
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output` and `editor with output`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml`, `line 2` and `editor`

### Requirement: The size of a block
A line that places a block SHALL take a size after a comma: a percent from `10%` to `90%` of the split, or a whole count of cells from 1. A placed block with no size SHALL share the rest of its split in equal parts with its anchor side and the other such blocks. The first line SHALL take no size. `tabs` and `statusline` SHALL take 1 row, and SHALL refuse a size.

#### Scenario: A percent
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `output as notices right of output, 25%`
- **AND** a server has the fake world `alpha/aria`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** the divider is in column 60

#### Scenario: A count of cells
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `output as notices below output, 5`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** the pane of `notices` takes 5 rows, its title row included

#### Scenario: A size on the tabs
- **GIVEN** `clients/layouts/mine.toml` holds the lines `editor` and `tabs above editor, 3`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml`, `line 2` and `tabs`

#### Scenario: A size on the first line
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output, 50%` and `editor below output`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 1`

### Requirement: The editor grows with its text
The block `editor` SHALL take one row for each wrapped row of its text. That count SHALL be at least 1, at most its `max`, and at most half the rows of its split. A sentence SHALL set the most rows with `max N` after a comma, from 1 to 100. The default SHALL be 8. The editor SHALL refuse any other size. Beside another block in one row, the editor SHALL share the width as a block with no size.

#### Scenario: The editor stops at its max
- **GIVEN** the layout in use holds the lines `output` and `editor below output, max 3`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types five lines with Alt+Enter between them, in a terminal of 80 columns and 24 rows
- **THEN** the editor takes the bottom 3 rows

#### Scenario: The default max
- **GIVEN** the layout `classic` is in use, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types twelve lines with Alt+Enter between them, in a terminal of 80 columns and 24 rows
- **THEN** the editor takes the bottom 8 rows

### Requirement: Editors side by side take the same rows
A part of the screen that holds only editors side by side SHALL take the rows of its tallest editor. Each editor SHALL count its rows at its own width, and up to its own `max`.

#### Scenario: Two editors side by side
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `editor as scratch right of editor`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types three lines with Alt+Enter between them, in a terminal of 80 columns and 24 rows
- **THEN** the pane of `editor` and the pane of `scratch` each take rows 22 to 24

#### Scenario: The shorter max does not stop the taller editor
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `editor as scratch right of editor, max 2`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types three lines with Alt+Enter between them, in a terminal of 80 columns and 24 rows
- **THEN** the pane of `editor` and the pane of `scratch` each take rows 22 to 24

### Requirement: A bad sentence makes its layout bad
A layout SHALL be bad in each of these cases, and its notice SHALL name the line number and the word:

- an unknown block, an unknown form, or a bad size
- a block or an id that an earlier line placed
- an anchor that is not on an earlier line
- no `editor`
- more than 32 lines, or more than 8 levels of splits

#### Scenario: An unknown block
- **GIVEN** `clients/layouts/mine.toml` holds the lines `editor` and `spawns above editor`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml`, `line 2` and `spawns`

#### Scenario: A block twice
- **GIVEN** `clients/layouts/mine.toml` holds the lines `editor`, `output above editor` and `output at top`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 3` and `output`

#### Scenario: A block that is not placed yet
- **GIVEN** `clients/layouts/mine.toml` holds the lines `editor` and `tabs above output`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 2` and `output`

#### Scenario: No editor
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output` and `tabs above output`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `editor`

#### Scenario: A percent out of range
- **GIVEN** `clients/layouts/mine.toml` holds the lines `editor` and `output above editor, 95%`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 2` and `95%`

#### Scenario: Too many lines
- **GIVEN** `clients/layouts/mine.toml` holds 33 lines
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `32`

### Requirement: A plan that mu writes again gives the same screen
When mu writes a plan again from a screen, the new plan SHALL give the same screen at each terminal size. It SHALL keep the id of each block.

#### Scenario: Any layout reads back
- **GIVEN** any valid layout
- **WHEN** mu writes a plan from it and reads that plan
- **THEN** the second layout equals the first, and each block has the same id

### Requirement: The built-in layouts
mu SHALL have these built-in layouts, and the default layout SHALL be `classic`:

- `classic`: `output`, `tabs above output`, `editor below output`
- `status`: the lines of `classic`, then `statusline below editor`
- `sidebar`: the lines of `status`, then `output as notices right of output, 30%`
- `focus`: `output`, `editor below output`

#### Scenario: The default layout
- **GIVEN** the configuration directory of the server holds no `clients` directory
- **WHEN** the user runs `mu config show --device laptop`
- **THEN** stdout holds `layout = "classic"` with the source `default`

#### Scenario: The sidebar layout
- **GIVEN** `clients/global.toml` holds `layout = "sidebar"`, and a server has the fake world `alpha/aria`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** row 1 shows `[alpha/aria]`, row 24 is the statusline, and a divider shows in column 56

### Requirement: The client draws dividers and title rows
Between two panes side by side, the client SHALL draw a divider of one column. A pane of `output` SHALL show a title row in two cases: its pane is a stack, or the layout holds more than one `output`. The title row SHALL show the id of each block of the pane. The title row SHALL be the first row of the pane.

#### Scenario: Two text panes
- **GIVEN** the layout `sidebar` is in use
- **WHEN** the client draws the screen
- **THEN** the first row of the left pane shows `[output]`, and the first row of the right pane shows `[notices]`

#### Scenario: One text pane has no title row
- **GIVEN** the layout `classic` is in use, and a server has the fake world `alpha/aria`
- **WHEN** the client draws the screen
- **THEN** no row shows `[output]`



### Requirement: A small terminal keeps each pane usable
A pane of `output` SHALL keep at least 2 rows and 10 columns while the terminal has room for each pane at that size. The client SHALL NOT stop in a terminal with less room.

#### Scenario: A percent in a narrow terminal
- **GIVEN** the layout in use holds the lines `output`, `editor below output` and `output as notices right of output, 10%`
- **WHEN** the user starts `mu client` in a terminal of 40 columns
- **THEN** the pane of `notices` is 10 columns wide

### Requirement: The client applies a new layout live
When an attached client gets a configuration with another layout, it SHALL draw the new layout at the next draw. The scroll position of each pane and the text of each editor SHALL stay.

#### Scenario: A reload changes the layout
- **GIVEN** a client shows the fake world `alpha/aria` with the layout `classic`, and the user typed `draft`
- **WHEN** the user writes `layout = "status"` to `clients/global.toml`, and runs `mu config reload`
- **THEN** the bottom row is the statusline
- **AND** the editor holds `draft`

### Requirement: The client checks the layout that it gets
The client SHALL check the lines of each layout that it gets with the same rules as the server. If the check fails, the client SHALL keep its layout and show a notice that names `layout`.

#### Scenario: A layout with no editor from the server
- **GIVEN** a client with the layout `classic`
- **WHEN** it gets a configuration whose layout holds the one line `output`
- **THEN** the client keeps the layout `classic`
- **AND** the screen shows a line that starts with `-- ` and contains `layout`

### Requirement: The tab label
The table `[tabs]` SHALL hold `label`, a template of 1 to 64 characters. Its default SHALL be `{world}`. The tab SHALL show the label in place of the world id. The marks of the focus, of unseen lines and of the sends that wait SHALL stay around it.

#### Scenario: A label with the number
- **GIVEN** `clients/global.toml` holds `[tabs]` with `label = "{number}:{character}"`, and a server has the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client`
- **THEN** the top row shows `[1:aria] 2:bryn`

#### Scenario: A label keeps the unseen mark
- **GIVEN** `clients/global.toml` holds `[tabs]` with `label = "{character}"`, the focus is on `alpha/aria`, and the fake world `beta/bryn` sent a line
- **WHEN** the client draws the screen
- **THEN** the top row shows `bryn*`

### Requirement: The fields of a tab label
The label SHALL replace each field with its value. `{{` SHALL give `{`, and `}}` SHALL give `}`. The fields SHALL be these:

- `{world}`: the world id
- `{game}`: the part of the id before the first `/`
- `{character}`: the part of the id after the first `/`
- `{number}`: the place of the world in the tabs, from 1

#### Scenario: The game and the character
- **GIVEN** `clients/global.toml` holds `[tabs]` with `label = "{character}@{game}"`, and a server has the fake world `alpha/aria`
- **WHEN** the user starts `mu client`
- **THEN** the top row shows `[aria@alpha]`

#### Scenario: Braces
- **GIVEN** `clients/global.toml` holds `[tabs]` with `label = "{{{character}}}"`, and a server has the fake world `alpha/aria`
- **WHEN** the user starts `mu client`
- **THEN** the top row shows `[{aria}]`

### Requirement: A bad tab label makes its layer bad
The layer SHALL be bad, and its notice SHALL name the file and `tabs.label`, in each of these cases:

- an empty label, or a label of more than 64 characters
- an unknown field
- a `{` with no closing `}`, or a lone `}`

#### Scenario: An unknown field
- **GIVEN** `clients/global.toml` holds `[tabs]` with `label = "{name}"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml`, `tabs.label` and `name`

#### Scenario: An open brace
- **GIVEN** `clients/global.toml` holds `[tabs]` with `label = "{world"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `tabs.label`

#### Scenario: A label too long
- **GIVEN** `clients/global.toml` holds `[tabs]` with a `label` of 65 characters, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `tabs.label`

### Requirement: The client draws the layout of the world in focus
The client SHALL draw the layout of the world in focus. That SHALL be the layout of the `[world-layouts]` entry of its world id. With no such entry, it SHALL be the layout of the entry of its game. With neither, and with no world in focus, it SHALL be the layout of the field `layout`. The entry or the field that gives the layout on the screen is the choice in use.

#### Scenario: A world with its own layout
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `"alpha/aria" = "status"`, and a server has the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** the tab of `alpha/aria` has the focus, and row 24 is the statusline

#### Scenario: The focus moves to a world with no entry
- **GIVEN** the client of the scenario "A world with its own layout"
- **WHEN** the user runs the entry `focus-next`
- **THEN** the tab of `beta/bryn` has the focus, and row 24 is the editor

#### Scenario: A layout for a game
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `alpha = "focus"`, and a server has the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client`
- **THEN** row 1 shows no tab

#### Scenario: The world entry comes before the game entry
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `alpha = "focus"` and `"alpha/aria" = "classic"`, and a server has the fake world `alpha/aria`
- **WHEN** the user starts `mu client`
- **THEN** row 1 shows `[alpha/aria]`

#### Scenario: No world in focus
- **GIVEN** `clients/global.toml` holds `layout = "status"` and `[world-layouts]` with `alpha = "focus"`, and a server with no world
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** row 24 is the statusline

#### Scenario: A world keeps its text across a change of layout
- **GIVEN** the client of the scenario "A world with its own layout", and the user typed `draft`
- **WHEN** the user runs the entry `focus-next`, and then the entry `focus-prev`
- **THEN** the editor holds `draft`, and row 24 is the statusline
