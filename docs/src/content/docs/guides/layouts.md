---
title: The layout of the client
description: Arrange the screen of mu client with blocks, short sentences and named layouts.
sidebar:
  order: 5
---

This guide shows how to choose and write the layout of `mu client`. The layout says which parts of the screen show, and where.

## The blocks

A block is one part of the screen. mu has four kinds of block:

| Block | Shows | How many in one layout |
|---|---|---|
| `tabs` | One tab for each world | None or one |
| `output` | Lines of text | Up to eight |
| `editor` | The text that you type | One to four |
| `statusline` | One row of facts about the world in focus | None or one |

Each block has an id. The id is the name of the block, with one exception: the id of `output` is `main`. So the line `output` makes the block `main`, and other lines name it as `main`. A second `output` or a second `editor` needs its own id, which you set with `as`: `output as notices`. mu turns each upper-case letter of an id to lower case.

Each layout holds one `output` with the id `main`, and at least one `output` that shows the stream `notices`. A layout without them is bad. A change that removes them, such as a hide of `main`, is refused.

## The sentences

A layout is a plan: a list of lines in order. Each line is one sentence that places one block.

- The first line is one block. It takes the whole screen.
- Each later line places a new block next to an anchor. The anchor is the id of a block of an earlier line.

| Sentence | Result |
|---|---|
| `tabs above main` | The tabs take the top of the pane of `main` |
| `editor below main` | The editor takes the bottom of it |
| `output as notices right of main` | A second output takes the right part of it |
| `output as side left of main` | A second output takes the left part of it |
| `statusline at bottom` | The statusline takes the whole bottom edge of the screen. The other edges are `top`, `left` and `right` |
| `output as channels with notices` | The two outputs share one pane as a stack. A title row shows a tab for each |
| `output as notices right of main and editor` | The new block stands beside two neighbors |
| `output as comms showing chan-*, pages right of main` | The output shows the streams that the patterns name |
| `output as chat of all worlds right of main` | The output shows its streams for every world, in one pane |

A line splits only the pane that its anchor has at that point. So the order of the lines decides the shape, and the same lines in another order can give another screen.

## Sizes

A size goes after a comma at the end of a sentence.

| Size | Meaning |
|---|---|
| `30%` | A percent of the split, from `10%` to `90%` |
| `12` | A count of columns or rows |
| `max 3` | For an editor only: the most rows that it grows to. The default is 8 |

Only a placed block takes a size. The side of its anchor shares the rest. The first line takes no size. `tabs` and `statusline` always take one row. An editor takes one row for each row of its text.

A pane of `output` keeps at least 2 rows and 10 columns while the terminal has room.

## The built-in layouts

mu ships four layouts. The command `mu layout show <name>` prints the numbered lines of a layout and a picture of it.

```text
$ mu layout show classic
1. output
2. tabs above main
3. editor below main
┌──────────────────────────────────────┐
│ tabs                                 │
├──────────────────────────────────────┤
│ main                                 │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
├──────────────────────────────────────┤
│ editor                               │
└──────────────────────────────────────┘
```

`classic` is the default. `status` adds the statusline:

```text
$ mu layout show status
1. output
2. tabs above main
3. editor below main
4. statusline below editor
┌──────────────────────────────────────┐
│ tabs                                 │
├──────────────────────────────────────┤
│ main                                 │
│                                      │
│                                      │
│                                      │
├──────────────────────────────────────┤
│ editor                               │
├──────────────────────────────────────┤
│ statusline                           │
└──────────────────────────────────────┘
```

`sidebar` adds a pane for the notices of the client:

```text
$ mu layout show sidebar
1. output as main showing main
2. tabs above main
3. editor below main
4. statusline below editor
5. output as notices right of main, 30%
┌──────────────────────────────────────┐
│ tabs                                 │
├──────────────────────────┬───────────┤
│ main                     │ notices   │
│                          │           │
│                          │           │
│                          │           │
├──────────────────────────┴───────────┤
│ editor                               │
├──────────────────────────────────────┤
│ statusline                           │
└──────────────────────────────────────┘
```

`focus` shows only the output and the editor:

```text
$ mu layout show focus
1. output
2. editor below main
┌──────────────────────────────────────┐
│ main                                 │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
├──────────────────────────────────────┤
│ editor                               │
└──────────────────────────────────────┘
```

`mu layout list` prints every layout that the server has.

## Choose a layout

The field `layout` of a client layer names the layout. The guide [The client configuration](/guides/client/) describes the layers.

```toml
# clients/global.toml
layout = "status"
```

If you change the file while the server runs, run `mu config reload`. Each attached client then draws the new layout. The text in the editor and the scroll position of each pane stay.

## Write your own layout

A layout of your own is a file `clients/layouts/<name>.toml` in the configuration directory of the server. It holds one field, `place`, with the lines of the plan.

```toml
# clients/layouts/evening.toml
place = [
  "output",
  "tabs above main",
  "editor below main",
  "statusline below editor",
  "output as notices right of main and editor, 33%",
  "output as channels with notices",
]
```

```text
$ mu layout show evening
1. output
2. tabs above main
3. editor below main
4. statusline below editor
5. output as notices right of main and editor, 33%
6. output as channels with notices
┌──────────────────────────────────────┐
│ tabs                                 │
├─────────────────────────┬────────────┤
│ main                    │ notices + c│
│                         │            │
│                         │            │
│                         │            │
├─────────────────────────┤            │
│ editor                  │            │
├─────────────────────────┴────────────┤
│ statusline                           │
└──────────────────────────────────────┘
```

The name of the file follows the rule of a device name. A file cannot have the name of a built-in layout.

If a layout file is bad, the server skips it and writes one notice. The notice names the file, the number of the line and the word. A layer that names that layout gets `classic`.

## Several outputs

A stream is a named set of lines. The lines of a world are in the stream `main`, and the notices of the client are in the stream `notices`. The line rules of the server send a line of a game to other streams. The guide [The configuration](/guides/configuration/#line-rules) describes the rules.

- An `output` with no `showing` shows the stream with the same name as its id. The output `main` with no `showing` shows the streams `main` and `notices`.
- After `showing`, a list of patterns separated by commas names the streams of the output. A pattern is a glob: `*` matches any text and `?` matches one character. A pattern that starts with `!` excludes the streams that it matches, and an exclusion wins. The list holds at least one pattern without `!`. A pattern that starts with `re:` or `/` is refused, because those forms are kept for a later syntax.
- The words `of all worlds` come right after the id, and before `showing`. They make one pane that shows the streams of every world. The output `main` cannot take them.
- The words `all`, `worlds` and `showing` are words of the sentences, so they are not ids.
- A line shows in each output that shows one of its streams, and one time in each such output. A line stays in the outputs where it arrived. An output that you add later starts with the new lines only.
- If no output of the layout of a world shows a stream of a line, the client shows one notice that names the stream and the world. The client shows no other such notice for that stream and that world until it starts again. The line stays in the journal of the server.

If a layout has its own output for `notices`, write `output as main showing main` as its first line. Without `showing`, the output `main` shows the notices too. `sidebar` writes `output as main showing main`, so its notices show only in the pane of `notices`.

### An output for each channel

An output that shows the streams of a rule is a spawn. This example sends each channel and each page of a game to a pane of its own. First, the rules in `global.toml` name the streams:

```toml
# global.toml
[[rules]]
id = "chan"
pattern = '^\[([\w-]+)\] '
stream = "chan-$1"

[[rules]]
id = "page"
pattern = '^\w+ pages: '
stream = "pages"
```

Then the layout places one output for the channels and one for the pages:

```toml
# clients/layouts/spawns.toml
place = [
  "output as main showing main",
  "tabs above main",
  "editor below main",
  "output as comms showing chan-*, !chan-ooc right of main, 40%",
  "output as pages of all worlds below comms",
  "output as notices with comms",
]
```

The pane of `comms` shows each channel except `chan-ooc`. No output shows `chan-ooc`, so its first line gives one notice. The pane of `pages` shows the pages of every world. The pane of `notices` shares one pane with `comms` as a stack.

### The owner of a line

In an output of all worlds, each line starts with its owner and `: `, in dim text. The owner of a line of a world is the id of the world, such as `alpha/aria: `. The owner of a notice is the world that the notice is about. A notice that is about no world has the owner `client`.

An output of all worlds is one pane for every world. It keeps its lines when the focus moves. A line of a world goes only to the outputs that the layout of that world names. So if the world in focus has a layout with no such output, the pane does not show.

With more than one `output` on the screen, each pane of outputs shows a title row with the id of each of its blocks. The id in square brackets is the block that shows.

## Several editors

A layout can hold up to four editors. One editor has the input focus, and the terminal cursor is in it. The key Alt+E gives the input focus to the next editor. A click in an editor gives it the input focus too.

```toml
# clients/layouts/wide.toml
place = [
  "output",
  "editor below main",
  "editor as scratch right of editor",
]
```

```text
$ mu layout show wide
1. output
2. editor below main
3. editor as scratch right of editor
┌──────────────────────────────────────┐
│ main                                 │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
├───────────────────┬──────────────────┤
│ editor            │ scratch          │
└───────────────────┴──────────────────┘
```

Each editor keeps one text for each world. Enter sends the text of the editor with the input focus to the world in focus. The editors of a world share its input history. Editors side by side take the rows of the tallest one.

## A layout for one world

The table `[world-layouts]` of a client layer gives a world or a game its own layout. The key is a world id or the name of a game.

```toml
# clients/global.toml
layout = "classic"

[world-layouts]
"alpha/aria" = "status"
alpha = "focus"
```

The client draws the layout of the world in focus. It takes the entry of the world id. With no such entry, it takes the entry of the game. With neither, it takes the field `layout`. So the screen can change when the focus changes.

If an entry names a layout that does not exist, the entry does not apply, and a notice names it.

## Change the layout in the client

You can change the layout while the client runs. The screen shows each change at once, and the client saves it on the server.

| Command | Result |
|---|---|
| `/place <sentence>` | Puts a block on the screen. A block that is on the screen moves |
| `/pane hide <id>` | Takes a block off the screen. The last editor stays |
| `/pane size <id> <size>` | Sets the size of a block. The size `fill` removes it |
| `/layout plan` | Shows the numbered lines of the layout |
| `/layout insert <number> <sentence>` | Puts a line at that number |
| `/layout edit <line> <sentence>` | Replaces a line. The line is a number or the id of its block |
| `/layout delete <line>` | Removes a line |
| `/layout use <name>` | Sets the layout of this device |
| `/layout here <name>` | Sets the layout of the world in focus. With no name, it removes that choice |
| `/layout save <name>` | Saves the layout on the screen under a new name |
| `/layout list` | Shows the name of each layout |

Two entries have no command of two words. `pane-next` shows the next block of a stack, and `layout-next` uses the next layout in the order of the names. Neither has a default key, so bind them in `[keys]`.

The mouse changes sizes:

- Drag the divider between two panes left or right.
- Drag the title row of a pane up or down, if the pane is below another pane.
- Click a name in a title row to show that block of a stack.

A drag inside a pane still selects text. A size from a drag is a whole percent, from 10% to 90%.

### Arrange mode

Alt+L, or the command `/arrange`, starts arrange mode. Each block shows a letter at its top left corner, and the last row shows the keys of the mode. Typed text does not reach the editor, and the editor keeps its text.

| Key | Result |
|---|---|
| A letter | Picks that block |
| A second letter, then an arrow | Places the picked block on that side of the second block |
| A second letter, then `=` | Puts the picked block in the pane of the second block, as a stack |
| `>` and `<` | Make the picked block 5 percent larger or smaller |
| `-` | Hides the picked block |
| `+` | Lists the blocks that the layout can take. A letter of the list picks one |
| The undo key | Takes back the last change of the mode |
| Enter | Saves the layout and leaves the mode |
| Esc | Drops the pick. With no pick, it restores the layout and leaves the mode |

You can also drag a block onto another block. A release near an edge places it on that side, and a release in the middle makes a stack. The part where the block lands shows a mark while the button is down.

The client saves nothing before Enter. If the layout rules refuse a move, the screen stays, and the last row shows the reason. If the configuration changes while arrange mode is on, the mode ends with no save.

### Where a change is saved

- If the layout on the screen is one of your own layouts, the change goes into that file. Each world that uses the layout gets the change.
- If the layout is built in, the change makes a new layout with the name of the device. The layer of the device then names it.
- If an entry of `[world-layouts]` chose the built-in layout, the new layout has the name `<device>.<game>.<character>`. Only that entry then names it, so the other worlds keep their screen.

If the new name is not a good layout name, the client does not save, and a notice tells you to use `/layout save <name>`. That happens for a name of more than 64 characters, and for a device with the name of a built-in layout.

If the server refuses a save, a notice gives the reason, and the screen goes back to the layout that the server holds.

A layout file or a device file can be a symbolic link to a regular file. A save then writes the linked file and keeps the link. The server refuses a save through a link to anything else.

A line of a plan has at most 256 characters, and names an anchor only once.

## Change a layout from the shell

The verb `mu layout` changes a layout on a running server. A script or an agent uses the same commands. Each attached client draws the result.

| Command | Result |
|---|---|
| `mu layout list` | Each layout with its picture |
| `mu layout show <name>` | The numbered lines of one layout and its picture |
| `mu layout use <name>` | The layout of this device |
| `mu layout use <name> --world <world>` | The layout of one world on this device. `--game <game>` sets it for a game |
| `mu layout forget --world <world>` | Removes the layout of one world. `--game <game>` removes it for a game |
| `mu layout place <layout> "<sentence>"` | Puts a block on the screen, or moves it |
| `mu layout hide <layout> <id>` | Takes a block off the screen |
| `mu layout size <layout> <id> <size>` | Sets the size of a block |
| `mu layout insert <layout> <number> "<sentence>"` | Puts a line at that number |
| `mu layout edit <layout> <line> "<sentence>"` | Replaces a line. The line is a number or the id of its block |
| `mu layout delete <layout> <line>` | Removes a line |
| `mu layout describe <name>` | Each block with its rectangle and what it shows. `--json` prints one JSON document |

Each command takes `--server` and `--device` as `mu client` does. No command starts a server.

You cannot change a built-in layout. A change of one saves a new layout with the name of the device, and the layer of that device then names it. With `--world`, the new layout has the name `<device>.<game>.<character>`, and only the entry of that world names it.

A change keeps your other lines where it can. If no change of one line gives the screen, mu writes the whole plan again and says so. Each block keeps its id.

`mu layout describe` tells a caller with no screen where each block is:

```text
$ mu layout describe sidebar
1. main (output): column 1, row 2, 55 by 21: the stream main of the world in focus
2. tabs (tabs): column 1, row 1, 80 by 1: one tab for each world of the server
3. editor (editor): column 1, row 23, 80 by 1: the text that the player types for the world in focus
4. statusline (statusline): column 1, row 24, 80 by 1: the segments of the world in focus
5. notices (output): column 57, row 2, 24 by 21: the stream notices of the world in focus
```

An agent cannot see the live screen of a client: the world in focus, the block in front of a stack, and the scroll position.

## Before a step back to an earlier mu

An earlier `mu` does not know the fields of this guide. It refuses a layer that holds one of them. Before you install an earlier binary:

1. Remove `layout`, `[world-layouts]`, `[tabs]` and `[statusline]` from each file under `clients/`.
2. Move the directory `clients/layouts/` out of the configuration directory.
