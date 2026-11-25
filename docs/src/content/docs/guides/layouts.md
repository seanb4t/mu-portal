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

Each block has an id. The id is the name of the block. A second `output` or a second `editor` needs its own id, which you set with `as`: `output as notices`.

## The sentences

A layout is a plan: a list of lines in order. Each line is one sentence that places one block.

- The first line is one block. It takes the whole screen.
- Each later line places a new block next to an anchor. The anchor is the id of a block of an earlier line.

| Sentence | Result |
|---|---|
| `tabs above output` | The tabs take the top of the pane of `output` |
| `editor below output` | The editor takes the bottom of it |
| `output as notices right of output` | A second output takes the right part of it |
| `output as side left of output` | A second output takes the left part of it |
| `statusline at bottom` | The statusline takes the whole bottom edge of the screen. The other edges are `top`, `left` and `right` |
| `output as channels with notices` | The two outputs share one pane as a stack. A title row shows a tab for each |
| `output as notices right of output and editor` | The new block stands beside two neighbors |

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
2. tabs above output
3. editor below output
┌──────────────────────────────────────┐
│ tabs                                 │
├──────────────────────────────────────┤
│ output                               │
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
2. tabs above output
3. editor below output
4. statusline below editor
┌──────────────────────────────────────┐
│ tabs                                 │
├──────────────────────────────────────┤
│ output                               │
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
1. output
2. tabs above output
3. editor below output
4. statusline below editor
5. output as notices right of output, 30%
┌──────────────────────────────────────┐
│ tabs                                 │
├──────────────────────────┬───────────┤
│ output                   │ notices   │
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
2. editor below output
┌──────────────────────────────────────┐
│ output                               │
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
  "tabs above output",
  "editor below output",
  "statusline below editor",
  "output as notices right of output and editor, 33%",
  "output as channels with notices",
]
```

```text
$ mu layout show evening
1. output
2. tabs above output
3. editor below output
4. statusline below editor
5. output as notices right of output and editor, 33%
6. output as channels with notices
┌──────────────────────────────────────┐
│ tabs                                 │
├─────────────────────────┬────────────┤
│ output                  │ notices + c│
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

An `output` shows the lines of the stream with its id, for the world in focus. A stream is a named set of lines.

- The first `output` of the plan shows the lines of the world. It also shows each stream that has no `output` of its own.
- The notices of the client are the stream `notices`. With `output as notices` in the layout, the notices show there. Without it, they show in the first `output`.
- A line stays in the output where it arrived. An output that you add later starts with the new lines only.

With more than one `output` on the screen, each pane of outputs shows a title row with the id of each of its blocks. The id in square brackets is the block that shows.

## Several editors

A layout can hold up to four editors. One editor has the input focus, and the terminal cursor is in it. The key Alt+E gives the input focus to the next editor. A click in an editor gives it the input focus too.

```toml
# clients/layouts/wide.toml
place = [
  "output",
  "editor below output",
  "editor as scratch right of editor",
]
```

```text
$ mu layout show wide
1. output
2. editor below output
3. editor as scratch right of editor
┌──────────────────────────────────────┐
│ output                               │
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

## Before a step back to an earlier mu

An earlier `mu` does not know the fields of this guide. It refuses a layer that holds one of them. Before you install an earlier binary:

1. Remove `layout`, `[world-layouts]`, `[tabs]` and `[statusline]` from each file under `clients/`.
2. Move the directory `clients/layouts/` out of the configuration directory.
