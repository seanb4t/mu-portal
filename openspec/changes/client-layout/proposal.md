# Proposal

## Why

The terminal client always draws one fixed screen. `client-shell` (#168) and `config-store` (#183) built the ground for a layout that the player arranges, which #101 asked for. A first plan wrote the layout as a TOML tree, and the owner was not able to read it at a glance. A grid form and a line form failed the same test. The clients that players like, such as BeipMU and Mudlet, let the player arrange the live screen, and the client remembers the result. This change does the same. The player arranges the screen with the mouse, with keys or with commands. A file holds the result as short sentences.

Tracked by #172. It takes the layout part of the write API from #173.

## What Changes

The change has five layers. Each layer works and has its tests before the next one starts.

- **Layer 1: blocks and the layout model.** A block is one part of the screen: `tabs`, `output`, `editor` or `statusline`. A layout can hold several outputs and several editors, each with its own id. One catalog names each block and its rules. A layout places blocks in panes, splits the screen into panes, and can stack several blocks in one pane. A small set of operations changes a layout: place, hide, resize and stack. A layout reads and writes as placement sentences, one line for each block, such as `output as notices right of output, 30%`.
- **Layer 2: named layouts.** mu ships the layouts `classic`, `status`, `sidebar` and `focus`. A player layout is a file `clients/layouts/<name>.toml` on the server. The field `layout` of a client layer names the layout in use, and the table `[world-layouts]` names another one for a world or a game. The client draws the layout of the world in focus, with a divider between panes side by side and a title row where a pane needs one. `mu layout list` prints each layout as a picture.
- **Layer 3: the layout API.** The new calls `ListLayouts`, `DescribeLayout`, `SaveLayout`, `EditLayout` and `UseLayout` let a client or an agent read a layout, change one block of it, write it and choose it for a device. The server checks, writes and reloads. Each attached client gets the result.
- **Layer 4: commands and the mouse.** `/place`, `/pane hide`, `/pane size` and `/layout use` change the screen at once and save. A drag of a divider or of a title row resizes a pane and saves at the release.
- **Layer 5: arrange mode.** `/arrange` marks each pane with a letter. The player moves, stacks, adds, hides and resizes blocks with keys or with a drag, and sees each step. Enter saves, and Esc restores the old screen.
- **Tab labels and the statusline.** `[tabs]` sets the label of a world tab. `statusline` is a block with the segments `world`, `state`, `connected`, `idle` and `typed`, and `[statusline]` picks them.
- The prominent notice of #172 is not built. `config-store` removed the notice that it was for.

## Capabilities

### New Capabilities

- `client-layout`: the blocks, panes, splits and stacks, the placement sentences, the built-in layouts, how the client draws a layout, and the `[tabs]` label.
- `layout-editing`: the commands, the mouse actions and arrange mode that change the layout in the client, and when a change is saved.
- `client-statusline`: the `statusline` block, its segments, and the `[statusline]` table.

### Modified Capabilities

- `client-config`: a client layer holds `layout`, `[tabs]`, `[statusline]` and `[world-layouts]` besides `echo` and `[keys]`.
- `config-store`: the server reads and checks `clients/layouts/`, serves the layouts, and writes a layout and the layout choice of a device.
- `terminal-client`: the top row is the block `tabs`, not a fixed part of the screen. The text of a tab comes from the label.

## Choices you will notice in use

| Choice | Value |
|---|---|
| Blocks | `tabs`, `output`, `editor`, `statusline`. A layout holds one to four editors, up to eight outputs, and each other block at most once. |
| Several outputs | Each output after the first needs an id: `output as channels right of output`. An output shows the lines of the stream with its id. The first output also shows every stream that has no output of its own. |
| Notices | The notices of the client are the stream `notices`. With an `output as notices`, they show there. Without one, they show in the first output, as today. |
| A line stays where it arrived | Each line goes into one output when it arrives. An output that you add later starts with the new lines only. |
| Notices and worlds | A notice shows under the world in focus only, as today. |
| Editors side by side | They take the rows of the tallest one. |
| Several editors | Each editor after the first needs an id: `editor as scratch right of editor`. Each keeps one text for each world. Enter sends the editor that you type in to the world in focus. They share the history of the world. |
| Input focus | Alt+E goes to the next editor, and a click in an editor picks it. The terminal cursor shows which one. |
| Sentence forms | `<block>` on the first line. Then `<block> above <anchor>`, `below`, `left of`, `right of`, `with`, and `<block> at top`, `at bottom`, `at left`, `at right`. |
| Several anchors | `output as notices right of output and editor` puts a block beside two or more neighbors. |
| Order | The order of the lines decides the shape. Each listing and each error shows the number of the line. The file holds a list with no numbers. |
| Ids | Each block has an id, its name by default. `output as side right of output` sets another. Anchors, commands and calls use the id, and it never changes. |
| The plan | `/layout plan` lists the numbered lines. `/layout insert`, `/layout edit` and `/layout delete` change one line, by its number or its id. |
| Your lines | A change by a command, the mouse or arrange mode changes only the line of its block where it can. If it cannot, mu writes the plan again, keeps the ids, and says so. |
| `with` | Puts two blocks in one pane as a stack. A title row shows a tab for each, and one shows at a time. Only outputs stack. |
| Sizes too large for the screen | The solver cuts them to the room that is left. The plan stays good. |
| Sizes | After a comma: a percent from `10%` to `90%`, or a count of cells. With no size, the block shares the rest. The first line takes no size, so the side of an anchor always shares the rest. |
| Fixed blocks | `tabs` and `statusline` take 1 row and refuse a size. |
| Editor size | It grows with its text. `max N` after a comma sets the most rows, from 1 to 100, default 8. |
| Smallest text pane | 2 rows and 10 columns. A resize stops there. |
| Limits | 32 lines in a layout. 8 levels of splits. A layout name follows the rule of a device name. |
| Built-in layouts | `classic` is the screen of today. `status` adds the statusline at the bottom. `sidebar` is `status` with `output as notices` right of `output` at 30%. `focus` is `output` and `editor` only. |
| Default layout | `classic`. |
| A layout for one world | `[world-layouts]` maps a world id or a game to a layout. The client looks for the world, then its game, then `layout`. The screen changes when the focus changes. |
| `/layout here` | `/layout here <name>` sets the layout of the world in focus, on this device. With no name, it removes that choice. |
| A world layout that does not exist | The entry does not count, and a notice names it. The world then takes the next choice. |
| Layout files | `clients/layouts/<name>.toml` with one field, `place`, a list of sentences. A file cannot take the name of a built-in layout. |
| A bad layout file | Skipped with a notice that names the file, the line and the word. A layer that names it gets `classic` and a notice. |
| Divider | One column of `│` between two panes side by side. |
| Title row | A pane of outputs shows one, with the id of each, if it is a stack or if the screen holds more than one output. |
| Mouse resize | Drag a divider left or right. Drag a title row up or down. The size becomes a whole percent of the split. |
| Mouse without arrange mode | A drag inside a pane still selects text, as today. |
| Save | Each finished change saves: a command, a drag at its release, and Enter in arrange mode. |
| Where a save goes | To the layout on the screen, if it is a player layout. Each world that uses that layout gets the change. |
| A save from a built-in layout | To a new layout. Its name is the device name, or `<device>.<game>.<character>` if a world entry chose the built-in layout. The device layer then names it. |
| A saved name that is not good | No save, and a notice names `/layout save`. Such a name has more than 64 characters, or is a built-in name, `.` or `..`. |
| A failed save | The screen keeps the change, and a notice gives the reason. |
| Two clients that save at once | The last save wins. |
| Arrange mode key | `alt+l`, and the command `/arrange`. |
| A letter bound to `undo` | In arrange mode, a letter always picks a block. |
| Arrange mode keys | A letter picks a block. A second letter and an arrow place it on that side of the second block. `=` stacks it. `<` and `>` resize. `-` hides. `+` adds a block that is not on the screen. The undo key undoes. Enter saves. Esc cancels. |
| Arrange mode mouse | Drag a block onto another. A release near an edge places it on that side. A release in the middle stacks it. |
| Typing in arrange mode | Off. The editor keeps its text. |
| `layout-next` | An entry with no default key. It takes the next layout by name and saves the choice. It changes the world entry if one chose the layout on the screen. |
| `/layout use` behind a world entry | It sets the layout of the device. A notice says that this world has its own layout. |
| `mu layout` | `list` prints each name and a picture. `show <name>` prints the sentences and a picture. `use <name>` sets the layout of a device, or of one world with `--world` or `--game`. `forget` removes the layout of a world. `place`, `hide`, `size`, `insert`, `edit` and `delete` change one line. `describe <name>` prints each block with its rectangle and its data source, as JSON with `--json`. |
| Agents | An agent uses the same calls as the client. It cannot see the live screen of a client: the focus, the tab in front and the scroll position. #175 holds that work. |
| Tab label | `[tabs] label`, default `{world}`. Fields `{world}`, `{game}`, `{character}`, `{number}`. 1 to 64 characters. The client keeps the marks `[ ]`, `*` and `(N waiting)`. |
| Statusline default | `left = ["world", "state"]`, `right = ["idle"]`, with ` | ` between segments. |
| Time format | `M:SS` under one hour, `H:MM:SS` from one hour. |
| Clock tick | Once a second, only while `connected` or `idle` shows. |
| Untested | A drag in a terminal other than Rex. The tests send mouse reports to the client, and no test drives a real mouse. |

## Not in this change

- Spawns (#102). A spawn needs no block of its own: it is a stream with a name, and an `output` with that id shows it. This change builds the outputs and the stream `notices`. #102 builds the rules of a world that send its lines to a named stream.
- A pane fixed to one world.
- A statusline or a tab label for one world. Only the layout has a choice for each world.
- The command palette, and the usage text of an argument in the registry (#167).
- The live state of a client for an agent: its focus, its terminal size and what each pane shows (#175).
- The rest of the write API (#173): keys, echo, worlds, and a check of the revision before a write.
- A layout that changes with the size of the terminal. A device layer already gives each terminal its own layout.
- Floating panes, borders around a pane, and colors of the statusline.
- A block or a layout from a script. Lua layouts come after Phase 1.

## Differences from convention

- A step back to an older binary needs two manual steps, which the guide states. That binary refuses the new fields.
- tmux, Zellij and kitty keep a layout as a tree or a code that the tool writes. This change keeps sentences that a person can read, and the client writes the same form.
- The sentences are a small language. The exploration of #61 ruled out a new language for configuration. Each sentence is one TOML text in a list, and the same words are the commands of the client, so no second file format comes in.
- A terminal multiplexer saves a layout only on request. This change saves each finished change, as BeipMU does.
- A key or a drag in the client writes a file on the server. Before this change, only a person with an editor wrote the configuration.

## Impact

- Code: `crates/mu-config` gets the type `WorldId` from `mu-core`, which now depends on `mu-config`. It also gets the catalog, the layout model, the operations, the sentences, the label and the segments. `crates/mu-server` reads `clients/layouts/`, serves the three calls and writes files. `crates/mu-proto` gets the calls and the new fields. `crates/mu-tui` draws layouts, and gets the commands, the mouse actions, arrange mode, the statusline and the tick. `crates/mu-cli` gets the verb `layout` and prints the new fields in `mu config show`.
- New dependencies: `toml_edit` in `mu-server`, to set one field of a device layer and keep the rest of the file. `winnow` in `mu-config`, to parse the sentences. The lock file already holds `winnow`. `insta` in `mu-config`, for the pictures in its tests. `tempfile` of `mu-server` becomes a dependency of the build.
- Documentation: a new guide page for layouts, the guides `client.md` and `configuration.md`, the page `architecture/terminal-client.mdx`, and a decision page for the layout model.
- Diagrams: the LikeC4 model gets the write path from the client to the configuration store, and one dynamic view of a save.
