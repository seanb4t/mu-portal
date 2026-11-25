# Spec Delta

## ADDED Requirements

### Requirement: The layout files
A layout file SHALL be `clients/layouts/<name>.toml`, where the name follows the rule of a device name. It SHALL hold only the field `place`, a list of placement sentences. The server SHALL check each layout file at start and at reload. A bad layout file SHALL be skipped, with one notice that names the file and the reason. A file with the name of a built-in layout SHALL be bad.

#### Scenario: A layout file in use
- **GIVEN** `clients/layouts/mine.toml` holds `place` with the lines `output` and `editor below main`
- **AND** `clients/global.toml` holds `layout = "mine"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `layout = "mine"` with the source `clients/global.toml`
- **AND** stderr is empty

#### Scenario: A bad layout file at start
- **GIVEN** `<dir>/clients/layouts/mine.toml` holds `place` with the one line `output`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `editor`
- **AND** the server serves

#### Scenario: A file with a built-in name
- **GIVEN** `<dir>/clients/layouts/classic.toml` holds `place` with the one line `editor`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/layouts/classic.toml` and `built-in`

#### Scenario: An unknown field of a layout file
- **GIVEN** `<dir>/clients/layouts/mine.toml` holds `place = ["editor"]` and `colour = "red"`
- **WHEN** the user runs `mu server --config <dir> --socket <dir>/s.sock`
- **THEN** stderr holds one line that contains `clients/layouts/mine.toml` and `colour`

### Requirement: The layout field names the layout in use
The field `layout` of a client layer SHALL name a built-in layout or a layout file. It SHALL merge as any other field, and its default SHALL be `classic`. `GetClientConfig` SHALL return the name, the lines of that layout, and the source of the field. If the name is of no good layout, the reply SHALL hold `classic` and a notice that names the layer and the layout.

#### Scenario: The device names another layout
- **GIVEN** `clients/global.toml` holds `layout = "status"`, and `clients/devices/laptop.toml` holds `layout = "focus"`
- **AND** a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `layout = "focus"` with the source `clients/devices/laptop.toml`

#### Scenario: A layout that does not exist
- **GIVEN** `clients/global.toml` holds `layout = "gone"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `gone`
- **AND** stdout holds the lines of `classic`

#### Scenario: A change to a layout file changes the revision
- **GIVEN** `clients/global.toml` holds `layout = "mine"`, and the user ran `mu config show --device laptop`
- **WHEN** the user adds a line to `clients/layouts/mine.toml`, runs `mu config reload`, and runs the same `mu config show` again
- **THEN** the two first lines differ

### Requirement: ListLayouts gives every layout
The call `ListLayouts` SHALL return each built-in layout and each good layout file: its name, its lines, and whether it is built in. The order SHALL be by name. The reply SHALL also hold the notice of each bad layout file.

#### Scenario: The list with one player layout
- **GIVEN** `clients/layouts/mine.toml` is a good layout file, and a server serves that directory
- **WHEN** the user runs `mu layout list`
- **THEN** stdout names `classic`, `focus`, `mine`, `sidebar` and `status` in that order
- **AND** each built-in name has the mark `built-in`

#### Scenario: The list with a bad layout file
- **GIVEN** `clients/layouts/mine.toml` holds `place` with the one line `output`, and a server serves that directory
- **WHEN** the user runs `mu layout list`
- **THEN** stdout does not name `mine`
- **AND** stderr holds one line that contains `clients/layouts/mine.toml` and `editor`

### Requirement: SaveLayout writes a layout file
The call `SaveLayout` SHALL take a name, the lines of a layout, and an optional device name. The server SHALL check the name and the lines with the rules of a layout file. If the check passes, it SHALL write `clients/layouts/<name>.toml` as a whole, so that a reader never sees a part of it. It SHALL then read the directory again, as a reload does. A failed check SHALL be an error that names the reason, and SHALL write nothing.

#### Scenario: A save writes the file
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory
- **WHEN** a client calls `SaveLayout` with the name `mine` and the lines `output` and `editor below main`
- **THEN** `<dir>/clients/layouts/mine.toml` holds those two lines in `place`
- **AND** `mu layout list` names `mine`

#### Scenario: A save of a bad layout
- **GIVEN** a server serves `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `mine` and the one line `output`
- **THEN** the call fails, and its error contains `editor`
- **AND** no file `<dir>/clients/layouts/mine.toml` exists

#### Scenario: A save with a built-in name
- **GIVEN** a server serves `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `classic` and the line `editor`
- **THEN** the call fails, and its error contains `classic`

#### Scenario: A save with a bad name
- **GIVEN** a server serves `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `../x` and the line `editor`
- **THEN** the call fails, and no file is written outside `<dir>/clients/layouts`

#### Scenario: A save with the name of two dots
- **GIVEN** a server serves `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `..` and the lines `output` and `editor below main`
- **THEN** the call fails, and its error contains `..`
- **AND** `<dir>/clients` holds no file

### Requirement: A write makes the directories that it needs
Before a write, the server SHALL make `clients`, `clients/layouts` and `clients/devices` if they are missing. A link to a directory SHALL count as a directory, and the write SHALL go through the link. If one of these paths exists and is not a directory, the call SHALL fail with an error that names the path, and SHALL write nothing.

#### Scenario: A save through a linked directory
- **GIVEN** `<dir>/clients` is a symbolic link to a directory that holds `layouts/mine.toml` with the lines `output` and `editor below main`
- **WHEN** a client calls `EditLayout` on `mine` with the place `tabs above main`
- **THEN** the call succeeds, and the file `layouts/mine.toml` of the linked directory holds the lines `output`, `editor below main` and `tabs above main`
- **AND** `<dir>/clients` is still a symbolic link

#### Scenario: A clients path that is a file
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory, and the user then makes the empty file `<dir>/clients`
- **WHEN** a client calls `SaveLayout` with the name `mine` and the lines `output` and `editor below main`
- **THEN** the call fails, and its error contains `clients`
- **AND** `<dir>/clients` is still an empty file

### Requirement: A write checks first, and keeps links and permissions
The server SHALL make each check of a write call before its first write, so a refused call writes nothing. It SHALL store the lines of a layout as mu writes them. A write to a file that is a symbolic link SHALL replace the file that the link names, and SHALL keep the link. A write SHALL keep the permissions of a file that it replaces. An error of a write SHALL hold no line of a file.

#### Scenario: A refused save writes no layout file
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds `echo = `, and a server serves `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `mine`, the line `editor` and the device `laptop`
- **THEN** the call fails, and `<dir>/clients/layouts/mine.toml` does not exist

#### Scenario: A refused edit of a built-in layout writes no layout file
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds `echo = `, and a server serves `<dir>`
- **WHEN** a client calls `EditLayout` on `classic` with the hide of `tabs` and the device `laptop`
- **THEN** the call fails, and `<dir>/clients/layouts/laptop.toml` does not exist

#### Scenario: A write to a linked device file
- **GIVEN** `<dir>/clients/devices/laptop.toml` is a symbolic link to a file that holds `# shared` and `echo = "show"`
- **WHEN** a client calls `UseLayout` with the device `laptop` and the layout `status`
- **THEN** the path is still a symbolic link, and the linked file holds `# shared` and `layout = "status"`

#### Scenario: A write keeps the permissions
- **GIVEN** `<dir>/clients/devices/laptop.toml` has the mode 0640
- **WHEN** a client calls `UseLayout` with the device `laptop` and the layout `status`
- **THEN** the file has the mode 0640

#### Scenario: An error holds no line of the file
- **GIVEN** the text of a layer is `echo = "show"` and then the line `password hunter2`
- **WHEN** the server edits the field `layout` of that text
- **THEN** the error holds `line 2`, and does not hold `hunter2`

#### Scenario: A save stores the lines as mu writes them
- **GIVEN** a server serves an empty configuration directory
- **WHEN** a client calls `SaveLayout` with the lines `  output  ` and `editor   below   main`
- **THEN** the layout file holds the lines `output` and `editor below main`

### Requirement: SaveLayout with a key sets the entry of a world
The call `SaveLayout` SHALL also take an optional key, which is a world id or the name of a game. With a device name and a key, the server SHALL set the entry of that key in `[world-layouts]` of the device layer, in the same write as the layout file, and SHALL NOT set the field `layout`. A key with no device name SHALL be an error, and SHALL write nothing.

#### Scenario: A save with a key
- **GIVEN** a server serves an empty configuration directory `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `mine`, the lines `output` and `editor below main`, the device `laptop` and the key `alpha/aria`
- **THEN** `<dir>/clients/layouts/mine.toml` holds the two lines
- **AND** `<dir>/clients/devices/laptop.toml` holds the entry `"alpha/aria" = "mine"` in `[world-layouts]`, and no field `layout`

#### Scenario: A save with a key and no device
- **GIVEN** a server serves an empty configuration directory `<dir>`
- **WHEN** a client calls `SaveLayout` with the name `mine`, the line `editor`, no device and the key `alpha/aria`
- **THEN** the call fails, its error contains `device`, and `<dir>/clients/layouts/mine.toml` does not exist

### Requirement: UseLayout sets the layout of a device
The call `UseLayout` SHALL take a device name and the name of a layout. `SaveLayout` with a device name SHALL do the same in the same write as the layout file. The server SHALL set the field `layout` in `clients/devices/<device>.toml`, and SHALL keep each other line of the file, its comments included. With no such file, it SHALL make one. It SHALL then read the directory again.

#### Scenario: A device file with a comment
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds the comment `# my laptop` and `echo = "show"`, and a server serves `<dir>`
- **WHEN** a client calls `UseLayout` with the device `laptop` and the layout `status`
- **THEN** the file holds `# my laptop`, `echo = "show"` and `layout = "status"`

#### Scenario: No device file
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory
- **WHEN** the user runs `mu layout use focus --device laptop`
- **THEN** `<dir>/clients/devices/laptop.toml` holds `layout = "focus"`

### Requirement: UseLayout refuses what it cannot apply
An unknown layout SHALL be an error that names it, and SHALL write nothing. So SHALL a bad device name, and a device file that is a bad layer.

#### Scenario: An unknown layout
- **GIVEN** a server serves `<dir>`
- **WHEN** the user runs `mu layout use gone --device laptop`
- **THEN** the exit status is 1, and stderr contains `gone`
- **AND** no file `<dir>/clients/devices/laptop.toml` exists

#### Scenario: A bad device layer
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds `echo = `, and a server serves `<dir>`
- **WHEN** a client calls `UseLayout` with the device `laptop` and the layout `status`
- **THEN** the call fails, and its error contains `clients/devices/laptop.toml`
- **AND** the file still holds `echo = `

### Requirement: An attached client gets a saved layout
After a `SaveLayout` or a `UseLayout`, the watch of the configuration SHALL send a new hash, as after a reload. Each attached client whose layout changed SHALL draw the new layout.

#### Scenario: A use from the command line
- **GIVEN** `mu client --device laptop` shows the layout `classic`
- **WHEN** the user runs `mu layout use status --device laptop` in another terminal
- **THEN** the bottom row of the client is the statusline

### Requirement: mu layout prints layouts
`mu layout list` SHALL print each layout of `ListLayouts` with a picture of its panes, each pane with the name of its block. `mu layout show <name>` SHALL print the numbered lines of one layout and its picture. If the name is of a bad layout file, it SHALL write the notice of the file to stderr and exit with status 1.

#### Scenario: The picture of classic
- **GIVEN** a server serves `<dir>`
- **WHEN** the user runs `mu layout show classic`
- **THEN** stdout holds the three lines of `classic`
- **AND** stdout holds a picture with the words `tabs`, `main` and `editor` from top to bottom

### Requirement: mu layout reaches only a private socket
`mu layout use <name>` SHALL call `UseLayout` with the device name of the requirement "The device name". Each `mu layout` command SHALL follow the requirement "mu config reaches only a private socket". No `mu layout` command SHALL start a server.

#### Scenario: An open socket directory
- **GIVEN** a server serves `<dir>`, and the directory of its socket has mode 0755
- **WHEN** the user runs `mu layout list` on that socket
- **THEN** the exit status is 1, and stdout is empty

#### Scenario: No server
- **GIVEN** nothing serves `<dir>/s.sock`
- **WHEN** the user runs `mu layout list --server unix://<dir>/s.sock`
- **THEN** the exit status is 1, and no file `<dir>/s.sock` exists

### Requirement: EditLayout changes one line of a layout
The call `EditLayout` SHALL take the name of a layout, one change, and an optional device name. A change SHALL be one of these: place a block, hide a block, set the size of a block, insert a line, replace a line, or delete a line. The server SHALL apply the change with the rules of the client commands of the same names, and save the result as `SaveLayout` does. The reply SHALL hold the new plan, and SHALL say whether mu wrote the whole plan again.

#### Scenario: An edit places a block
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output` and `editor below main`, and a server serves that directory
- **WHEN** the user runs `mu layout place mine "output as notices right of main, 30%"`
- **THEN** `mu layout show mine` holds three lines, and the last is `3. output as notices right of main, 30%`

#### Scenario: An edit gives the same file as the command
- **GIVEN** two servers, each with the layout file `mine` of the lines `output`, `tabs above main` and `editor below main`
- **WHEN** a client of the first server runs the command `/pane hide tabs`, and the user runs `mu layout hide mine tabs` on the second server
- **THEN** the two files `clients/layouts/mine.toml` hold the same lines

#### Scenario: An insert from the command line
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output` and `editor below main`, and a server serves that directory
- **WHEN** the user runs `mu layout insert mine 2 "output as notices right of main"`
- **THEN** `mu layout show mine` holds `2. output as notices right of main` and `3. editor below main`

#### Scenario: A delete by id
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output`, `editor below main` and `output as side right of main`
- **WHEN** the user runs `mu layout delete mine side`
- **THEN** `mu layout show mine` holds two lines

### Requirement: EditLayout refuses what it cannot save
For a built-in layout, `EditLayout` SHALL need the device name, and SHALL save under that name. A change that the layout rules refuse SHALL be an error with the reason, and SHALL write nothing.

#### Scenario: An edit of a built-in layout
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory
- **WHEN** the user runs `mu layout hide classic tabs --device laptop`
- **THEN** `<dir>/clients/layouts/laptop.toml` holds the lines `output` and `editor below main`
- **AND** `mu config show --device laptop` holds `layout = "laptop"`

#### Scenario: A bad edit
- **GIVEN** `clients/layouts/mine.toml` holds the lines `output` and `editor below main`, and a server serves that directory
- **WHEN** the user runs `mu layout hide mine editor`
- **THEN** the exit status is 1, and stderr contains `editor`
- **AND** the file still holds two lines

### Requirement: DescribeLayout gives the panes of a layout
The call `DescribeLayout` SHALL take the name of a layout, a width and a height. It SHALL return one entry for each block of the layout: its id, its name, the number of its line, the rectangle that it takes on a screen of that size, whether it is in front of its stack, and a text that names its data source. `mu layout describe <name>` SHALL print the entries, and with `--json` it SHALL print them as one JSON document.

#### Scenario: Describe gives each block
- **GIVEN** a server serves `<dir>`
- **WHEN** the user runs `mu layout describe sidebar --width 80 --height 24 --json`
- **THEN** stdout is JSON with five entries, one for each of `tabs`, `output`, `notices`, `editor` and `statusline`
- **AND** each entry has an id and a line number
- **AND** the entry of `notices` has the width 24, and the entry of `tabs` has the width 80 and the height 1

#### Scenario: Describe of an unknown layout
- **GIVEN** a server serves `<dir>`
- **WHEN** the user runs `mu layout describe gone`
- **THEN** the exit status is 1, and stderr contains `gone`

### Requirement: The world layouts table names a layout for a world
The table `[world-layouts]` of a client layer SHALL map a world id or the name of a game to the name of a layout. An entry SHALL replace the entry of an earlier layer with the same key, and the other entries SHALL stay. `GetClientConfig` SHALL return each entry with its key, the name and the lines of its layout, and its source. An entry for a world that the server does not have SHALL give no notice.

#### Scenario: A device adds an entry
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `alpha = "focus"`
- **AND** `clients/devices/laptop.toml` holds `[world-layouts]` with `"alpha/aria" = "status"`
- **AND** a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `world-layouts.alpha = "focus"` with the source `clients/global.toml`
- **AND** stdout holds `world-layouts."alpha/aria" = "status"` with the source `clients/devices/laptop.toml`

#### Scenario: A device replaces an entry
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `alpha = "focus"`, and `clients/devices/laptop.toml` holds `[world-layouts]` with `alpha = "status"`
- **AND** a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stdout holds `world-layouts.alpha = "status"` with the source `clients/devices/laptop.toml`
- **AND** stdout does not hold `world-layouts.alpha = "focus"`

#### Scenario: An entry for a world that the server does not have
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `"zeta/zed" = "focus"`, and a server with no world serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr is empty

#### Scenario: A change to the layout file of an entry changes the revision
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `alpha = "mine"`, and the user ran `mu config show --device laptop`
- **WHEN** the user adds a line to `clients/layouts/mine.toml`, runs `mu config reload`, and runs the same `mu config show` again
- **THEN** the two first lines differ

### Requirement: A bad world layouts entry
A key of `[world-layouts]` SHALL be a world id or the name of a game. Any other key SHALL make its layer bad, and the notice SHALL name the file and the key. An entry whose name is of no good layout SHALL NOT be in the reply of `GetClientConfig`. The reply SHALL hold a notice that names the layer, the key and the layout.

#### Scenario: A key with two slashes
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `"a/b/c" = "focus"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `a/b/c`

#### Scenario: An entry with a layout that does not exist
- **GIVEN** `clients/global.toml` holds `[world-layouts]` with `alpha = "gone"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml`, `alpha` and `gone`
- **AND** stdout does not hold `world-layouts.alpha`

### Requirement: UseLayout can set the layout of one world
`UseLayout` SHALL also take a world id or the name of a game. The server SHALL then set that entry of `[world-layouts]` in the device file, and SHALL NOT change the field `layout`. With an empty layout name, it SHALL remove the entry. `mu layout use <name>` with `--world <world>` or `--game <game>` SHALL set the entry. `mu layout forget` with one of the two flags SHALL remove it. A bad key, and a removal of an entry that the device file does not hold, SHALL be an error, and SHALL write nothing.

#### Scenario: A use for one world
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds the comment `# my laptop` and `layout = "status"`, and a server serves `<dir>`
- **WHEN** the user runs `mu layout use focus --world alpha/aria --device laptop`
- **THEN** the file holds `# my laptop`, `layout = "status"`, and `"alpha/aria" = "focus"` in the table `[world-layouts]`

#### Scenario: A use for a game
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory
- **WHEN** the user runs `mu layout use focus --game alpha --device laptop`
- **THEN** `mu config show --device laptop` holds `world-layouts.alpha = "focus"` with the source `clients/devices/laptop.toml`
- **AND** it holds `layout = "classic"` with the source `default`

#### Scenario: Forget the layout of a world
- **GIVEN** `<dir>/clients/devices/laptop.toml` holds `[world-layouts]` with `"alpha/aria" = "focus"` and `alpha = "status"`, and a server serves `<dir>`
- **WHEN** the user runs `mu layout forget --world alpha/aria --device laptop`
- **THEN** the file holds `alpha = "status"`, and does not hold `alpha/aria`

#### Scenario: Forget an entry that is not there
- **GIVEN** a server serves `<dir>`, which holds no `clients` directory
- **WHEN** the user runs `mu layout forget --world alpha/ari --device laptop`
- **THEN** the exit status is 1, stderr holds `alpha/ari`, and `<dir>/clients/devices/laptop.toml` does not exist

#### Scenario: A use with a bad world id
- **GIVEN** a server serves `<dir>`
- **WHEN** the user runs `mu layout use focus --world a/b/c --device laptop`
- **THEN** the exit status is 1, and stderr contains `a/b/c`
- **AND** no file `<dir>/clients/devices/laptop.toml` exists

### Requirement: EditLayout of a built-in layout can name a world
`EditLayout` SHALL also take a world id. For a built-in layout, the server SHALL then save by the rules of the client for the world in focus: the same name, and the same entry or field of the device file.

#### Scenario: An edit of the built-in layout of one world
- **GIVEN** `<dir>/clients/global.toml` holds `[world-layouts]` with `"alpha/aria" = "focus"`, and a server serves `<dir>`
- **WHEN** the user runs `mu layout place focus "tabs at top" --device laptop --world alpha/aria`
- **THEN** `<dir>/clients/layouts/laptop.alpha.aria.toml` holds three lines
- **AND** `mu config show --device laptop` holds `world-layouts."alpha/aria" = "laptop.alpha.aria"` and `layout = "classic"`
