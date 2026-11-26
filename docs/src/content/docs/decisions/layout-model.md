---
title: The layout model
description: The layout of the client is a short plan of lines that mu reads and writes.
sidebar:
  order: 24
---

## Decision

The screen of the client is made of blocks. A block is one part of the screen: `tabs`, `output`, `editor` or `statusline`. One catalog in the `mu-config` crate names each block, how many times a layout can hold it, and how it gets its size.

A layout is a plan: an ordered list of lines. Each line places one block.

```text
1. output
2. tabs above main
3. editor below main
4. output as notices right of main, 30%
```

The first line is one block that takes the whole screen. Each later line places a new block next to an anchor, which is a block of an earlier line. The forms are `above`, `below`, `left of`, `right of`, `at` an edge of the screen, and `with` for a stack.

mu builds a tree from the plan. The tree has two kinds of node:

- A pane holds one block, or several blocks as a stack. A stack shows one block at a time.
- A split holds two or more parts in one direction, either rows or columns.

The plan is the layout. The server stores the plan, the API carries it, and the client holds it. The tree is a result of the plan, and mu builds it again after each change.

Each block has an id. The id is the name of the block, or the word after `as`. The one exception is `output`, whose id is `main`. Commands, anchors and API calls name a block by its id. An id is written in the line, so no change to another line changes it. mu turns each upper-case letter of an id to lower case.

An output shows the streams that its line names after `showing`. With no `showing`, it shows the stream with the name of its id, and `main` shows `main` and `notices`. The id `main` gives the block no other role. A line goes to each output that shows one of its streams. So no block takes the lines of another block, and a hide or a move never sends lines to a new pane. Each layout must hold `main` and an output that shows `notices`, so each line of a world and each notice has an output.

A size belongs to one line. Only a placed block takes a size, and the side of its anchor shares the rest. So a resize is an edit of one line.

Every change of the screen is one operation on the plan: place, hide or resize. The commands, the mouse, arrange mode and the server all call the same operations. A change keeps the other lines of the player where it can:

1. It changes only the line of the block.
2. If that does not give the screen, it moves the line to the end.
3. If that does not give the screen, mu writes the whole plan again and says so. The ids stay.

## Reason

A player must be able to read a layout file at a glance, and the client must be able to write the same file. A tree written by hand in TOML failed the first test: nested tables do not look like a screen. A grid of names failed too, because one block that spans two rows needs a repeated name in each row.

A sentence for each block reads as a person says it: "the notices are right of the output". The same words are the commands of the client, so the file format is not a second thing to learn.

Clients that players like, such as BeipMU and Mudlet, let the player arrange the live screen and then remember the result. The plan supports that. A drag or a key is one operation, and the result is a small change to the file.

The plan is the source of truth, and not the tree, for one reason: the lines of the player stay as the player wrote them. A model that stores the tree and writes sentences from it writes the whole file again after each drag.

## Consequences

- The order of the lines decides the shape. The same lines in another order can give another screen. Each listing shows the number of each line, and `mu layout show` prints a picture beside the lines.
- No one writes the tree by hand, and no file holds it.
- A new block is one row of the catalog and one draw function. The model does not change.
- A reader of sentences is part of mu. It uses the `winnow` parser library, which the `toml` crate already brings into the build.
- Two property tests guard the writer. A line that mu writes reads back to the same step. A plan that mu writes builds the same tree, with the same ids.
- A pane that floats over other panes does not fit this model, because each cell of the screen belongs to one pane.
- The client refuses a change that leaves no `main` or no output of the notices. A layout with only an editor is bad.
- A stream that no output shows gets one notice for each world. Its lines stay in the journal.
