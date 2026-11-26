# Spec Delta

## MODIFIED Requirements

### Requirement: The blocks of the screen
The blocks SHALL be `tabs`, `output`, `editor` and `statusline`. `tabs` shows the row of world tabs. `output` shows lines of text. `editor` takes typed text. A layout SHALL hold `editor` one to four times, `output` one to eight times, and each other block at most once. A block that the layout does not hold SHALL NOT show, and the keys and commands SHALL work the same without it.

#### Scenario: No tabs
- **GIVEN** the layout in use holds the lines `output` and `editor below main`
- **AND** a server has the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client`, presses Ctrl+N, types `hi` and presses Enter
- **THEN** the screen shows no row with `alpha/aria`
- **AND** the server records a player event `hi` for `beta/bryn`

#### Scenario: The editor alone
- **GIVEN** `clients/layouts/mine.toml` holds the line `editor`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `main`

#### Scenario: Nine outputs
- **GIVEN** `clients/layouts/mine.toml` holds `editor` and nine lines that each place an `output` with its own id
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `output`

### Requirement: An output shows the lines of its id
Each line of text SHALL have one or more streams. The client SHALL take the streams of a line of a world from its event. The notices of the client SHALL be in the stream `notices`. An `output` with no `showing` SHALL show the stream with the same name as its id. The output `main` with no `showing` SHALL show the streams `main` and `notices`.

#### Scenario: One output shows the notices too
- **GIVEN** the layout `classic` is in use, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user runs the command `/focus nowhere/nobody`
- **THEN** the pane of `main` shows `Welcome, aria.` and a line that starts with `-- `

#### Scenario: An output for the notices
- **GIVEN** the layout in use holds the lines `output as main showing main`, `editor below main` and `output as notices right of main`
- **AND** the world in focus is the fake world `alpha/aria`
- **WHEN** the user runs the command `/focus nowhere/nobody`
- **THEN** the pane of `notices` shows a line that starts with `-- `
- **AND** the pane of `main` shows `Welcome, aria.` and no line that starts with `-- `

#### Scenario: An output with no stream yet
- **GIVEN** the layout in use holds the lines `output`, `editor below main` and `output as channels right of main`
- **WHEN** the client draws the screen
- **THEN** the pane of `channels` shows its title row and no other line

### Requirement: One output is the main output
Each layout SHALL hold exactly one `output` with the id `main`. A line `output` with no `as` SHALL make the block `main`, so each other `output` SHALL set its own id with `as`. The id `main` SHALL give the block no other role than its default `showing`.

#### Scenario: The main output is not the first output line
- **GIVEN** a layout holds the lines `editor`, `output as notices above editor` and `output as main showing main left of notices`
- **WHEN** a world sends the line `A line.`, and the user runs the command `/focus nowhere/nobody`
- **THEN** the pane of `main` shows `A line.`, and the pane of `notices` shows a line that starts with `-- `

#### Scenario: No main output
- **GIVEN** `clients/layouts/mine.toml` holds the lines `editor` and `output as side above editor`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `main`

### Requirement: A line stays in the output where it arrived
At the arrival of a line, the client SHALL put it into each `output` that shows one of its streams at that time. The client SHALL use the outputs for the world of the line. A later change of the layout SHALL NOT move the line. A notice SHALL be a line of the stream `notices` for the world in focus. The notice of a line that the server refused SHALL be a line of the stream `notices` for the world of that line.

#### Scenario: A refused line goes to the notices
- **GIVEN** the layout in use holds the lines `output as main showing main`, `tabs above main`, `editor below main`, `statusline below editor` and `output as notices right of main, 30%`, and `alpha/aria` has the focus
- **WHEN** the user sends the line `hi`, and the server refuses it with the reason `down`
- **THEN** the last line of the pane of `notices` is `-- down`
- **AND** the pane of `main` shows no line that starts with `-- `

#### Scenario: A refused line of a world that lost the focus
- **GIVEN** the layout in use holds the lines `output as main showing main`, `tabs above main`, `editor below main`, `statusline below editor` and `output as notices right of main, 30%`, and `alpha/aria` has the focus
- **WHEN** the user sends the line `hi`, runs the command `/focus beta/bryn`, and the server then refuses the line with the reason `down`
- **THEN** no pane shows a line that starts with `-- `
- **AND** after the command `/focus alpha/aria`, the last line of the pane of `notices` is `-- down`

#### Scenario: A line with no world goes to the notices
- **GIVEN** the layout in use holds the lines `output as main showing main`, `tabs above main`, `editor below main`, `statusline below editor` and `output as notices right of main, 30%`, and the server has no world
- **WHEN** the user types `hi` and presses Enter
- **THEN** the pane of `notices` shows a line that starts with `-- `, and the pane of `main` shows none

#### Scenario: A notice shows under the world in focus only
- **GIVEN** the layout `classic` is in use, a server has the fake worlds `alpha/aria` and `beta/bryn`, and `alpha/aria` has the focus
- **WHEN** the user runs the command `/focus nowhere/nobody`, and then the entry `focus-next`
- **THEN** the pane of `main` shows `Welcome, bryn.` and no line that contains `nowhere/nobody`

#### Scenario: An output that comes later starts with new lines
- **GIVEN** the layout `classic` is in use, the world in focus is the fake world `alpha/aria`, and the user ran the command `/focus nowhere/nobody`
- **WHEN** the user runs the command `/place output as notices right of main`
- **THEN** the pane of `notices` shows its title row and no other line
- **AND** the pane of `main` shows a line that starts with `-- ` and contains `nowhere/nobody`

### Requirement: A bad sentence makes its layout bad
A layout SHALL be bad in each of these cases, and its notice SHALL name the line number and the word:

- an unknown block, an unknown form, or a bad size
- a block or an id that an earlier line placed
- an anchor that is not on an earlier line
- no `editor`
- no `output` with the id `main`, or no `output` that shows the stream `notices`
- a bad pattern after `showing`
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
- **GIVEN** `clients/layouts/mine.toml` holds the lines `editor` and `tabs above main`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 2` and `output`

#### Scenario: No editor
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output` and `tabs above main`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `editor`

#### Scenario: No output for the notices
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output as main showing main, chan-*` and `editor below main`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `notices`

#### Scenario: A percent out of range
- **GIVEN** `clients/layouts/mine.toml` holds the lines `editor` and `output above editor, 95%`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 2` and `95%`

#### Scenario: Too many lines
- **GIVEN** `clients/layouts/mine.toml` holds 33 lines
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `32`

### Requirement: The built-in layouts
mu SHALL have these built-in layouts, and the default layout SHALL be `classic`:

- `classic`: `output`, `tabs above main`, `editor below main`
- `status`: the lines of `classic`, then `statusline below editor`
- `sidebar`: `output as main showing main`, `tabs above main`, `editor below main`, `statusline below editor`, `output as notices right of main, 30%`
- `focus`: `output`, `editor below main`

#### Scenario: The default layout
- **GIVEN** the configuration directory of the server holds no `clients` directory
- **WHEN** the user runs `mu config show --device laptop`
- **THEN** stdout holds `layout = "classic"` with the source `default`

#### Scenario: The sidebar layout
- **GIVEN** `clients/global.toml` holds `layout = "sidebar"`, and a server has the fake world `alpha/aria`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** row 1 shows `[alpha/aria]`, row 24 is the statusline, and a divider shows in column 56

#### Scenario: The sidebar keeps notices out of main
- **GIVEN** `clients/global.toml` holds `layout = "sidebar"`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user runs the command `/focus nowhere/nobody`
- **THEN** the pane of `notices` shows a line that starts with `-- `, and the pane of `main` shows none

## REMOVED Requirements

### Requirement: The lines of a world stay with the main output
**Reason**: Each output shows the streams that its line names. No block takes the role of another, and the layout must hold `main`, so the lines of a world never need a new home.
**Migration**: None. A layout with no `main` is now bad, and `/pane hide main` is refused.

### Requirement: A notice waits for an output
**Reason**: Each layout must hold `main` and an output that shows `notices`, so a notice always has an output.
**Migration**: None. A layout with no output is now bad.

## ADDED Requirements

### Requirement: An output shows the streams that its patterns match
The form `output as <id> showing <pattern>, ...` SHALL set the streams of an output. A pattern SHALL be a glob with `*` and `?`. A pattern with `!` in front SHALL exclude. An output SHALL show a stream that matches at least one pattern without `!` and no pattern with `!`. The order of the patterns SHALL NOT matter. A list of only exclusions SHALL be bad, and so SHALL an empty list or a lone `!`. A pattern that starts with `re:` or `/` SHALL be bad.

#### Scenario: One output for all channels
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^You say, "\[(\w+)\] '` and `stream = "chan-$1"`
- **AND** the layout in use holds the lines `output`, `editor below main` and `output as comms showing chan-*, pages right of main`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user sends `[Public] one` and `[Ooc] two`
- **THEN** the pane of `comms` shows `You say, "[Public] one"` and `You say, "[Ooc] two"`
- **AND** the pane of `main` shows neither

#### Scenario: An exclusion
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^You say, "\[(\w+)\] '` and `stream = "chan-$1"`
- **AND** the layout in use holds the lines `output as main showing main, notices, chan-*, !chan-ooc` and `editor below main`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user sends `[Public] one` and `[Ooc] two`
- **THEN** the pane of `main` shows `You say, "[Public] one"` and not `You say, "[Ooc] two"`

#### Scenario: Only exclusions
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output`, `editor below main` and `output as side showing !chan-* right of main`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 3` and `showing`

#### Scenario: An empty list of patterns
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output`, `editor below main` and `output as side showing right of main`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 3` and `showing`

#### Scenario: A reserved pattern
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output`, `editor below main` and `output as side showing re:chan right of main`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 3` and `re:chan`

#### Scenario: A plan keeps the patterns
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output`, `editor below main` and `output as comms showing chan-*, pages right of main, 30%`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stdout holds the line `output as comms showing chan-*, pages right of main, 30%`

### Requirement: A line shows in each output that matches it
A line SHALL show in each output that shows one of its streams. One output SHALL show a line one time. This SHALL also hold if the output shows more than one of the streams of the line.

#### Scenario: Two outputs show main
- **GIVEN** the layout in use holds the lines `output`, `editor below main` and `output as log showing main right of main`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the client draws the screen
- **THEN** the panes of `main` and `log` each show `Welcome, aria.`

#### Scenario: A line in two streams of one output
- **GIVEN** `global.toml` has the rule `page` with `pattern = '^You say, "page'` and `stream = ["pages", "main"]`
- **AND** the layout `classic` is in use with the line `output` replaced by `output as main showing main, notices, pages`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user sends `page hi`
- **THEN** the pane of `main` shows `You say, "page hi"` one time

### Requirement: A stream with no output gets one notice
A line can have a stream that no output of the layout of its world shows. Then the client SHALL show one notice that names the stream and the world. It SHALL show no other such notice for that stream and world until the client starts again. A Reset SHALL NOT bring the notice back. The line SHALL stay in the journal.

#### Scenario: A channel with no output
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^You say, "\[(\w+)\] '` and `stream = "chan-$1"`
- **AND** the layout `classic` is in use, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user sends `[Public] one` and `[Public] two`
- **THEN** the pane of `main` shows one line that starts with `-- ` and contains `chan-public` and `alpha/aria`
- **AND** the pane of `main` shows neither `You say, "[Public] one"` nor `You say, "[Public] two"`

#### Scenario: No notice of a stream after a Reset
- **GIVEN** the layout `classic` is in use, and a line of the stream `chan-public` of `alpha/aria` gave the notice of a stream with no output
- **WHEN** the server sends a Reset
- **THEN** the pane of `main` shows no line that contains `chan-public`

### Requirement: The client remembers a limited number of streams with no output
The client SHALL remember at most 256 pairs of a stream and a world that got the notice of a stream with no output. If a new pair comes past that limit, the client SHALL forget the oldest pair. A forgotten pair SHALL get the notice again.

#### Scenario: The oldest stream gets its notice again
- **GIVEN** the layout `classic` is in use, and lines of `alpha/aria` came in the streams `chan-0` to `chan-256`, one stream for each line
- **WHEN** a line of `chan-1` comes, and then a line of `chan-0`
- **THEN** the line of `chan-1` gives no notice
- **AND** the line of `chan-0` gives the notice that names `chan-0`

### Requirement: A layout always keeps main and the notices
A change can leave the layout with no `main`, or with no output that shows `notices`. Then the client SHALL refuse the change, also from a command, the mouse or arrange mode. The refusal SHALL change nothing, and SHALL show a notice with the reason.

#### Scenario: Hide the main output
- **GIVEN** the layout `classic` is in use
- **WHEN** the user runs the command `/pane hide main`
- **THEN** the pane of `main` still shows
- **AND** the pane of `main` shows a line that starts with `-- ` and contains `main`

#### Scenario: Hide the only output of the notices
- **GIVEN** the layout `sidebar` is in use
- **WHEN** the user runs the command `/pane hide notices`
- **THEN** the pane of `notices` still shows, and its last line starts with `-- ` and contains `notices`

#### Scenario: Hide a second output of the notices
- **GIVEN** the layout in use holds the lines `output`, `editor below main` and `output as notices right of main`
- **WHEN** the user runs the command `/pane hide notices`
- **THEN** the screen shows no pane of `notices`

### Requirement: An output can show the streams of all worlds
The form `output as <id> of all worlds` SHALL place an output that shows its streams for every world. The words `of all worlds` SHALL come right after the id, and before `showing`. Such an output SHALL be one pane for all worlds. If the focus moves, it SHALL keep its lines. The output `main` SHALL NOT take this form. The words `all`, `worlds` and `showing` SHALL be words of the sentences.

#### Scenario: A page from another world
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^You say, "\['` and `stream = "chat"`
- **AND** the layout in use holds the lines `output`, `editor below main` and `output as chat of all worlds right of main`, and the server has the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user runs `/focus beta/bryn`, types `[Public] hi`, presses Enter, and runs `/focus alpha/aria`
- **THEN** the pane of `chat` shows `beta/bryn: You say, "[Public] hi"`
- **AND** the pane of `main` shows `Welcome, aria.`

#### Scenario: Patterns of all worlds
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output`, `editor below main` and `output as comms of all worlds showing chan-*, pages right of main, 30%`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stdout holds the line `output as comms of all worlds showing chan-*, pages right of main, 30%`

#### Scenario: The main output of all worlds
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output as main of all worlds` and `editor below main`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 1` and `main`

#### Scenario: The id all
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output`, `editor below main` and `output as all right of main`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 3` and `all`

#### Scenario: The scope after the place
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output`, `editor below main` and `output as chat right of main of all worlds`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stderr holds one line that contains `line 3`

#### Scenario: A place command with the scope
- **GIVEN** the layout `classic` is in use, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user runs the command `/place output as notices of all worlds right of main` and then `/focus nowhere/nobody`
- **THEN** the pane of `notices` shows a line that starts with `alpha/aria: -- ` and contains `nowhere/nobody`

### Requirement: A line in an output of all worlds names its owner
In an output of all worlds, each line SHALL start with its owner and `: `. The owner of a line of a world SHALL be the id of the world. The owner of a notice of the client SHALL be the world that the notice belongs to. A notice that belongs to no world SHALL have the owner `client`. The prefix SHALL be dim.

#### Scenario: Two worlds in one pane
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^You say, "\['` and `stream = "chat"`
- **AND** the layout in use holds the lines `output`, `editor below main` and `output as chat of all worlds right of main`, and the server has the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user sends `[Public] one` to `alpha/aria`, runs `/focus beta/bryn`, and sends `[Public] two`
- **THEN** the pane of `chat` shows `alpha/aria: You say, "[Public] one"` above `beta/bryn: You say, "[Public] two"`

#### Scenario: A notice in an output of all worlds
- **GIVEN** the layout in use holds the lines `output as main showing main`, `editor below main` and `output as notices of all worlds right of main`, and the server has no world
- **WHEN** the user types `hi` and presses Enter
- **THEN** the pane of `notices` shows a line that starts with `client: -- `

### Requirement: A world sends a line by its own layout
A line of a world SHALL go to the panes that the layout of that world names for its streams. A pane of all worlds SHALL show the lines of each world whose layout has an output of all worlds with that id. If the focus moves to a world whose layout has no such output, that pane SHALL NOT show.

#### Scenario: A world with another layout
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^You say, "\['` and `stream = "chat"`
- **AND** the world `alpha/aria` has a layout with the lines `output`, `editor below main` and `output as chat of all worlds right of main`, and the world `beta/bryn` has the layout `classic`
- **WHEN** the user sends `[Public] two` to `beta/bryn`, and runs `/focus alpha/aria`
- **THEN** the pane of `chat` does not show `beta/bryn: You say, "[Public] two"`

### Requirement: Ids and stream names are lower case
The id of a block and each pattern after `showing` SHALL be lower case. Before mu checks one of them, it SHALL turn each upper-case letter to lower case. A plan SHALL show the lower-case form.

#### Scenario: An id in upper case
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output`, `editor below main` and `output as Comms showing Chan-*, Pages right of main`
- **WHEN** the user runs `mu layout show mine`
- **THEN** stdout holds the line `output as comms showing chan-*, pages right of main`
