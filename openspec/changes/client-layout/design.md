# Design

## Context

See proposal.md for the motivation. The facts that shape the design:

- `crates/mu-tui/src/layout.rs` holds the layout as a tree: `Node::Split`, `Node::Tabs` and `Node::Leaf`. `view.rs` walks it. `Model::layout` is always `Node::default()`.
- `crates/mu-config` holds the client layer type and `check_keys`. It depends only on `serde`, and the server and the client both use it.
- The server checks a layer in `state()` in `crates/mu-server/src/store.rs`, and treats each file under `clients/` that is not a layer as bad (`layer_rule`). `ReloadConfig` reads the directory again, and `WatchConfig` sends a new hash to each client.
- No call writes the configuration directory. #173 plans the write API.
- `screen.rs` turns on mouse reports for press, release and drag in the SGR format. `update.rs` uses a press and a drag only to select text in the pane in focus.
- `PaneState` holds the lines of a pane and also what the last draw showed. So one pane cannot draw twice in one frame.
- The client model has no state, connect time or send time for a world. Each event has `time_ms`.
- `run.rs` draws after each message. The only timer is the wait of `Effect::RetryConfig`.

## Goals / Non-Goals

**Goals:**

- One model of the layout, with one set of operations. The keys, the mouse, the commands, the file and the server all call it.
- Each layer is done, with its tests and documentation, before the next one starts.
- A new block needs one catalog row and one draw function, and no change to the model.
- No test waits for the wall clock, and no test needs a real mouse.

**Non-Goals:**

- The rules that send a line of a world to a named stream. That work is #102.
- A general write API. This change writes two things only: a layout file, and the field `layout` of a device layer.

## Decisions

### D1. Layer 1: the catalog of blocks

`mu-config` gets a table `BLOCKS`, like `ENTRY_NAMES`. One row holds:

- the name of the block
- how many times a layout can hold it: `editor` one to four times, `output` up to eight times, and each other block at most once
- its size rule: one fixed row, the editor rule, or a free size
- whether it can share a pane as a stack
- whether its pane can show a title row

The rows are `tabs`, `output`, `editor` and `statusline`. The layout model reads only this table. `mu-tui` has one draw function for each row, and a test fails if a row has no draw function or a draw function has no row.

A new block is one row, one draw function and its tests. A spawn needs no new row: D11 shows that it is an `output` with an id.

### D2. Layer 1: the plan is the layout

A layout is a plan: the list of its lines. The plan is what the server stores, what the wire carries and what the model holds. The tree is a result of the plan, and mu builds it again after each change.

Each line parses to one step:

- the block, by its catalog name, and its id (the name, or the word after `as`)
- the place: first, beside one or more anchors on a side, at an edge, or with an anchor
- the size, or the `max` of the editor

The tree has two node kinds. A `Split` has a direction and two or more parts. A `Pane` has one or more blocks, and more than one is a stack. `build(plan)` applies the steps in order:

- A step beside one anchor looks at the split of the pane of the anchor. If that split has the direction of the side, the block becomes a new part next to the anchor. If not, the pane of the anchor becomes a split of that direction that holds both.
- A step beside several anchors first makes sure that their parts are neighbors in one split. It wraps those parts in one new part, and then places the block beside that part.
- A step at an edge makes the root a split that holds the block and the old root.
- A step with an anchor adds the block to the pane of the anchor.

Each part of each split comes from one step: the step that placed its block. The part of the anchor comes from an earlier step, or is the first block. So the size of a part is the size on one line, and a resize is an edit of one line. The anchor side never has a size, so each split always has a part that shares the rest. This removes the rule of the first plan that a split needs one free part.

An id is stable because it is written in the line. No renumbering and no rewrite changes it. The letters of arrange mode are marks on the screen, and are not ids. A pane has no id of its own: a caller names a pane by a block in it. The block in front of a stack is state of the client, and is not saved. At the start, the first block of a stack is in front.

The order is explicit as numbers in each listing, each error and each reply. The file holds a TOML list, whose order is the order of the lines. The file holds no numbers, because a number in the text can differ from the place in the list.

### D3. Layer 1: the changes

`mu-config` has two kinds of functions from a plan to a plan. Each one builds the new plan and returns it only if `build` and the check pass.

Line edits change the list directly:

| Function | What it does |
|---|---|
| `insert(number, step)` | Puts a line at that number. |
| `replace(line, step)` | Replaces a line. The line is a number or an id. |
| `delete(line)` | Removes a line. It fails if a later line names the block as an anchor, and the error names those lines. |

Screen changes say what the screen must become:

| Function | What it does |
|---|---|
| `place(step)` | For a new block, adds the line at the end. For a block on the screen, moves it. |
| `hide(id)` | Takes the block off the screen. |
| `resize(id, size)` | Sets the size on the line of the block. |
| `resize_border(id, cells, total)` | Finds the placed block beside the border, and sets its size to a whole percent. It keeps the percent from 10 to 90, and each pane at its smallest size or more. |

A screen change keeps the lines of the player where it can. It tries these in order, and takes the first plan that builds to the wanted screen:

1. Change only the line of the block, in its place.
2. Move the changed line to the end of the plan.
3. Write the whole plan again from the wanted tree, with the writer of D4.

The keys `<` and `>` of arrange mode call `resize` with a percent that the same function keeps from 10 to 90. So no change of the screen gives a size that the reader refuses.

The wanted tree comes from the same change on the tree, so step 3 always has a target. The function returns the plan and a flag that says whether step 3 ran. The client shows a notice for that flag, and `EditLayout` returns it. Every caller above this layer uses only these functions: the commands, the mouse, arrange mode and the server.

### D4. Layer 1: the reader and the writer

The reader uses the parser library `winnow`, by the engram rule 5rk42v8qs1. The `toml` crate already brings `winnow` into the lock file, so no new crate enters the build. The grammar is in one module:

```text
line    = block [ "as" id ] [ place ] [ "," size ]
place   = side anchors | "at" edge | "with" id
anchors = id { "and" id }
side    = "above" | "below" | "left of" | "right of"
edge    = "top" | "bottom" | "left" | "right"
size    = percent | cells | "max" cells
block   = a name of the catalog
id      = a letter, then letters, digits and "-", 32 characters at most
```

The parser gives one step for a line, and knows no layout. `build` applies the step, and finds an id twice, an anchor that is not placed, or anchors that are not neighbors. So an error of the grammar names the line, the word and its column, and an error of the layout names the line and the id. Words are in lowercase, and any run of spaces separates two words. An id cannot be a word of the grammar. Task 1.6 reads the `winnow` documentation for the error type that carries the position.

The writer goes from a tree to a plan. It runs only in step 3 of D3. It writes the outer parts first and the inner parts after:

1. For a split, the anchor is the part whose blocks took no size. The block that stands for a part is the anchor of that part, down to a pane.
2. Write the anchor. Then write each other part of the split as `left of`, `right of`, `above` or `below` the anchor, in an order that keeps the places.
3. Then do the same inside each part that is a split, around the block that stands for it.
4. Write each id that is not the block name with `as`.

The property test `a_line_reads_back` generates any step: each side, each edge, `with`, several anchors, `as`, each size and `max`. It makes sure that the written line parses to the same step. The property test `any_layout_reads_back` generates any valid tree with ids, and makes sure that the written plan builds to the same tree with the same ids. The property test `a_screen_change_gives_the_wanted_screen` generates any plan and any screen change, and makes sure that the returned plan builds to the wanted tree. Task 1.1 builds the reader, `build`, the writer and the first test before any other work, because every later layer depends on them.

### D5. Layer 2: layouts on the server

- `ClientLayer` gets `layout: Option<String>`. `FIELD_PATHS` gets `layout`, and the default is `classic`. It merges as `echo` does.
- `ClientLayer` gets the table `[tabs]` with the one field `label`, and `FIELD_PATHS` gets `tabs.label`. It gets the table `[statusline]` with `left` and `right`, and `FIELD_PATHS` gets `statusline.left` and `statusline.right`. Each of the three paths merges as one value, so a later list replaces an earlier list. Both tables refuse an unknown field.
- `state()` of the store checks a layer today with `check_keys`. It also calls the check of `Label` on `tabs.label`, and the check of the segments on each list. A failed check makes the layer bad, and the notice names the file and the field path, as a bad binding does today.
- The type `WorldId` moves from `mu-core` to `mu-config`, with its part rule, `game()`, `character()` and its tests. `mu-core` gets the edge to `mu-config` in `workspace_boundary.rs` and exports the same type again, so its callers do not change. The rule of a world id then has one home. `mu-tui` uses the type for the order of the worlds and for the label, in place of its own split of the text.
- `mu-config` gets `WorldKey`, a world id or the name of a game. Its one parser accepts one part, or two parts with one `/`, each by the part rule.
- `mu-config` holds the built-in layouts as lists of sentences, and a test makes sure that each one passes the check.
- The store reads each `clients/layouts/<name>.toml`. `layer_rule` gets this place, with the name rule of a device. The file type is one struct with the field `place` and unknown fields refused. The store runs the reader of D4 and keeps a good layout or a notice.
- `resolve` looks up the merged name. If no good layout has that name, the reply holds `classic` and a notice.
- `ClientLayer` gets `world_layouts: Option<BTreeMap<String, String>>` for the table `[world-layouts]`. It merges by key, as `[keys]` does. A key parses as a `WorldKey`, and a key that does not parse makes its layer bad. `resolve` drops an entry with no good layout and adds a notice.
- `mu-config` gets `layout_for(world, entries, default)`: the entry of the world id, then the entry of the game, then the default. It also returns which of the three it took, the choice in use. The client and `EditLayout` both call it.
- The revision of a client adds the content of each layout file that its layers name, in `layout` and in `[world-layouts]`. So a change to that file gives a new revision, and `apply_config` sees it.
- `GetClientConfigResponse` gets `repeated WorldLayout world_layouts`, each with a key, a name and its sentences. It also gets `layout_name`, `repeated string layout` with the sentences, `tab_label`, `statusline_left` and `statusline_right`. The sentences on the wire need no TOML parser, and the client reads them with `mu-config`.

### D6. Layer 2: how the client draws

`layout.rs` of `mu-tui` no longer has its own tree. `Model` holds the checked layout of `mu-config`, and `view.rs` solves it into rectangles:

- A split divides its cells by plain arithmetic in `mu-config` (see D8). A count takes its cells. A percent takes that part of the split, rounded down. The parts with no size share the rest, and the first ones take the cells left over. The editor keeps its rule of today.
- A percent is a part of the whole split, with its dividers. Between two parts of a split into columns, the solver takes one column for the divider `│`, from the part that shares the rest. A rectangle of `Placed` does not hold the divider. So 30% of 80 columns gives a pane of 24 columns, a divider in column 56, and 55 columns for the other part.
- The solver takes the sized parts of a split in the order of the plan. It cuts each one to the cells that are left after the smallest size of the part that shares the rest. So two parts of 60%, or a count larger than the screen, give a smaller pane and never a negative rest. The reader does not refuse such a plan.
- An editor wants rows: the wrapped rows of its text at its own width, up to its `max`. A split into columns whose parts are all editors wants the most rows of its parts. Any other part wants no rows and shares the rest.
- The solver then raises each text pane to its smallest size if the screen has room.
- A pane with a title row draws it on its first row: the name of each block of the stack, with the one in front in square brackets.

The solver is a pure function of `mu-config`, `solve(layout, area, rows_of) -> Vec<Placed>`. The caller passes `rows_of`, a function from the id of an editor and a width to its wrapped rows, because `mu-config` does not hold the text. Each `Placed` names a block, its rectangle, and its divider or title row. The draw, the mouse hit test, arrange mode and the pictures of `mu layout` all use this one function.

`mu-config` also gets `preview(layout, width, height) -> Vec<String>`, a picture with box characters and the name of each block. `mu layout list` prints it, and the tests of layers 1 and 2 use it as their expected output with `insta`.

`apply_config` reads the sentences and checks them. A good layout replaces the old one. A bad one keeps the old one and gives a notice. `Model` keeps the default layout and the layout of each entry. `Model::layout()` returns the one that `layout_for` gives for the world in focus, so a change of the focus needs no other code to change the screen. The pane states and the editor states stay in the model, so a new layout keeps the scroll positions and the text.

Task 1.2 works the columns of each spec scenario by the rounding rule above, and makes sure that they match. The property test `solve_stays_inside_the_area` takes any valid plan and any area from 0 by 0 to 300 by 100. It makes sure that `solve` returns, that each rectangle is inside the area, and that no two rectangles overlap.

### D7. Layer 2: the streams, the tab label and the statusline

- The pane states of D11 replace the map of panes by `PaneId`. `Model::notice` and `Model::hint` push a line of the stream `notices` for the world in focus.
- `Model` keeps the checked `Label`. `top_row` renders it for each world with its number, and passes the result through `clean`.
- `World` gets `state`, `connected_ms` and `sent_ms`. `learn` sets `state` from each list. An `EVENT_KIND_CONNECTED` event sets `connected_ms`, and an `EVENT_KIND_DISCONNECTED` event clears it. An event with the source `SOURCE_PLAYER` sets `sent_ms`. After a connect or a disconnect event, the client lists the worlds again. A `Reset` clears the three fields.
- `Msg::Tick(i64)` sets `Model::now_ms`. If the layout holds `statusline`, and its segments hold `connected` or `idle`, `update` returns a wait of one second. `Effect::RetryConfig` becomes `Effect::Wait { after, then }`, which carries the wait and the message to give back. The retry and the tick both use it, each with its own state of idle or waiting, so the run loop has one arm for a wait. The run loop sends one tick before the first draw. The tests send `Msg::Tick` with a chosen time.
- The statusline draw joins each side with ` | `, drops a segment with no value, and cuts the left side with the functions of `width.rs`.

Task 1.3 makes sure that `mu client` subscribes from 0, so the events give the connect time.

### D7a. Layer 2: several editors

`Model::pose_editors` maps a world id to one editor state today, and `Model::loose` is the editor of no world. Both become maps with the id of the editor block as a second key. `Model` gets `input: BlockId`, the editor with the input focus. `BlockId` of `mu-config` replaces `EditorId` of `mu-tui`. `EditorId`, `EditorId::POSE` and `Model::editors` go away with `Node`, `Leaf` and `Size`, and `Model::editor` takes a block id. `Model::pose()` returns the state for the world in focus and that id, so each existing entry that edits or sends works on the editor with the input focus with no other change.

- `apply_config` and each change of the layout set `input` to the first editor of the plan if the old one is gone.
- The entry `editor-next` moves `input` in the order of the plan. A press of the left button inside the rectangle of an editor sets it.
- The view puts the terminal cursor in the editor of `input`. Each editor draws its own state, so no state draws twice.
- A hidden editor keeps its texts in the model until the client stops. So an editor that comes back under the same id shows its text again.
- The history stays one for each world. Up and Down walk it in the editor with the input focus.

The grow rule and `max` hold for each editor alone. The limit of four keeps the keys and the screen simple, and is one number in the catalog.

### D8. Layer 3: the write calls

The service gets `ListLayouts`, `DescribeLayout`, `SaveLayout`, `EditLayout` and `UseLayout`. The store gets one write path:

1. Check the request with `mu-config`: the name, the sentences, the device name, and the key of a world entry. For `UseLayout`, the layout must exist, and the device file must not be a bad layer.
2. Make `clients/`, `clients/layouts/` and `clients/devices/` if they are missing. If one of these paths is not a directory, the call fails with an error that names the path, and writes nothing.
3. Write the file. A layout file is written whole to a temporary file in `clients/layouts/`, and then renamed over the target with `tempfile`. A device file is read, changed with `toml_edit`, and written the same way. The change sets the field `layout`, or sets or removes one entry of `[world-layouts]`. Each other line stays.
4. Read the directory again. The watch then sends the new hash, and each client fetches its configuration.

`Store::reload` takes the lock `reload` today, takes a snapshot of the directory, and calls `reload_from`. That lock is not reentrant. So the write path takes the lock once, runs steps 2 and 3, and then takes the snapshot and calls `reload_from` itself. It does not call `Store::reload`. The writes and the reloads then run one at a time. `SaveLayout` with a device does both writes and then one reload. If the second write fails, the first one stays, and the error names the file. The call takes no revision, so the last save wins. #173 adds that check for every write.

The server writes only under `clients/layouts/` and `clients/devices/`. A name follows the device name rule, which allows no `/`. That rule allows `.`, so the check also refuses the names `.` and `..`. The scenario "A save with a bad name" covers it.

`EditLayout` takes the name of a layout, one change of D3, and an optional device. The server reads the plan, applies the one function of D3, and saves the result through the same write path. The reply holds the new plan and the flag of D3. On a built-in layout it needs the device, and it saves by the save rule of D9. With a world id, that rule runs for that world, as it does in a client with that world in focus. So an agent changes one pane with one call, and the client and the server run the same code of `mu-config`.

`ListLayoutsResponse` holds the good layouts, and the notice of each bad layout file with the name of the file. `mu layout list` prints those notices to stderr. `mu layout show <name>` prints the notice of that name to stderr and exits with status 1.

`DescribeLayout` takes a name, a width and a height. It returns one entry for each block: its id, its name, the number of its line, its rectangle, whether it is in front of its stack, and its data source. The data source is a fixed text from the catalog, such as `the journal of the world in focus` for `output`. The rectangles come from the solver of D6, which moves from `mu-tui` to `mu-config` as plain arithmetic, because `mu-server` cannot depend on `mu-tui`. `mu-tui` keeps the ratatui draw and calls that solver. This replaces the use of ratatui `Layout` in D6, and task 1.2 changes to a check of the rounding rule that the solver states.

Task 1.4 reads the documentation of `toml_edit` and of `tempfile::NamedTempFile::persist`. It makes sure that D8 names the calls, and states what `persist` does when the target exists.

### D9. Layer 4: commands and the mouse

- The registry gets the entries `place`, `pane-hide`, `pane-size`, `pane-next`, `layout-use`, `layout-save`, `layout-list`, `layout-next` and `arrange`. `ArgKind` gets a kind for free text, and `/pane hide` and `/layout use` parse as a command of two words. `ENTRY_NAMES` in `mu-config` gets the same rows.
- The registry also gets `layout-plan`, `layout-insert`, `layout-delete` and `layout-edit`, over the line edits of D3.
- Each command calls a function of D3 on `Model::layout`, and then returns `Effect::SaveLayout`. `run.rs` makes the call and returns `Msg::LayoutSaved` with the result. A failure becomes a notice.
- The save rule of the spec is one function of `mu-config`, `save_target(choice in use, layout name, device name)`. It gives the name to save under and the entry or field to set. It runs the check of a layout name on that name, and returns the error of the check for a name over 64 characters, the name of a built-in layout, `.` or `..`. The notice of that error names `/layout save`. The client and `EditLayout` both call it.
- The registry gets `layout-here`. Its effect is a `UseLayout` call with the world in focus as the key, and an empty name removes the entry. `/layout save` and `layout-next` pass the choice in use to the same call.
- `layout-use` sets the field `layout`. It then calls `layout_for` for the world in focus. If the choice in use is an entry of a world or of a game, it adds the notice that names `/layout here`.
- A press of the left button runs a hit test on the result of `solve`. A hit on a divider or on a title row starts a drag state in the model. Each drag report calls `resize_border` on a copy of the plan from the press, so the result does not drift. The release keeps the result and returns `Effect::SaveLayout`. A press and a release on one name with no move is a click, and calls `front`.
- A press that hits no divider and no title row goes to the `output` pane under the mouse, and so do its drag and its release. Today they go to the pane of the world in focus, because the screen has one pane. The hit test of `solve` picks the pane, and the selection code takes its rectangle from `solve`.

The mouse modes stay as they are: press, release and drag reports in the SGR format. A drag needs no report of a move with no button. Task 1.5 reads the termina documentation of these modes and of the mouse event type. It makes sure that D9 states which events a drag gives.

### D10. Layer 5: arrange mode

`Model` gets `arrange: Option<Arrange>`. It holds the layout from before the mode, the stack of layouts for undo, the picked block, and the drag state. While it is set:

- `on_key` sends each key to the mode and not to the keymap. The key of the entry `undo` is read from the keymap. The mode tests a plain letter first, so a letter always picks a block, and an `undo` binding to a plain letter does nothing in the mode.
- Each change is one screen change of D3 on the plan of the model. So the screen shows it at the next draw.
- `view` draws the letter of each `Placed` on top of its first cell, and the keys of the mode on the last row. The solver gets one row less.
- A drag from one block to another uses the rectangles of `solve`. The release point picks a side or the middle by thirds, and the mode marks that part while the button is down.
- Enter drops the mode and returns `Effect::SaveLayout`. Esc puts the first layout back.

The mode adds no operation. It is a second caller of layer 1, so the tests of the mode check keys and marks, and the tests of the operations stay in `mu-config`.

### D11. Streams, and what #102 adds

A stream is a name on a line. An `output` is a pane with one line store, one scroll position and one selection, as a pane is today. The model keeps one pane state for each pair of a world and the id of an `output`, and one set for no world. So no block kind exists for a spawn or for the notices, and no pane state draws twice, because the ids of a plan are unique.

One function of `mu-config`, `route(plan, stream)`, gives the id of the `output` for a stream: the `output` with that id, or the first `output` of the plan. Each line goes into one pane state when it arrives, by `route` on the layout of its world. So the first `output` holds the lines of several streams in one store, in the order of arrival, and needs no merge at the draw.

A line does not move after it arrived. An `output` that the player adds later starts with the new lines only. A hidden `output` keeps its pane state until the client stops, and its stream goes to the first `output` from then on. #102 brings scrollback for each stream from the server, which can fill a new pane.

- The lines of a world go to the stream that a rule names, or to the stream of the world. This change has no rule, so each line of a world is in the stream of the world.
- `Model::notice` and `Model::hint` push to the pane that `route` gives for `notices` and the world in focus. With one `output`, that is the pane of the world in focus, which is the screen of today. A notice does not show under another world, as today.
- `Model::notices` stays the one kept list of the notices, because it must last through a reset of the panes. `replay_notices` stays, and writes that list into the pane that `route` gives for `notices`. The panes are views, and the list is the only store that a reset keeps.
- An `output` whose stream has no line shows only its title row.

#102 adds the rules of the world configuration that send a line to a named stream, the same for each client, and scrollback for each stream. It needs no change to the layout model. The spawns of two worlds can differ, because the rules are in the world configuration. An `output as channels` shows the stream `channels` of the world in focus, and is empty for a world with no such stream.

### D12. What enforces each rule

| Rule | Enforcement |
|---|---|
| A plan that mu writes again gives the same screen and ids | The property test `any_layout_reads_back`. |
| A screen change gives the wanted screen and a valid plan | The property test `a_screen_change_gives_the_wanted_screen`. |
| A screen change keeps the other lines where it can | The unit tests of the three steps of D3, and the scenario "A resize changes one line". |
| An id never changes | The two property tests compare the ids, and the scenario "Edit a line by its id". |
| One check on the server and the client | Both call `mu-config`. The scenario "A layout with no editor from the server" covers the client side. |
| Each block has a draw function | The test `each_block_has_a_draw`. |
| One editor has the input focus, and it is on the screen | The property test `the_input_focus_is_an_editor_of_the_plan`. It applies any list of layout changes and focus keys. |
| `classic` is the screen of today | The existing scenarios of `terminal-client`, and a snapshot of `preview`. |
| The server writes only under `clients/layouts/` and `clients/devices/` | The scenario "A save with a bad name", and review. |
| A device file keeps its other lines | The scenarios "A device file with a comment" and "A use for one world". |
| One written line parses to the same step | The property test `a_line_reads_back`. |
| The solver never fails and never overlaps two panes | The property test `solve_stays_inside_the_area`, and the unit tests `two_percents_past_the_split` and `a_count_past_the_screen`. |
| A size from a drag or a key is a size that the reader accepts | `resize` and `resize_border` keep 10 to 90 percent. The unit tests `a_border_stops_at_ten_percent` and `a_larger_block_stops_at_ninety_percent`. |
| One rule of a world id | `WorldId` in `mu-config`, which `mu-core` exports again. The existing tests of the id move with it. |
| One home of each line | `route` in `mu-config`, with the unit tests of its two cases. |
| One lookup of the layout of a world | `layout_for` in `mu-config`, with the unit tests of its three steps. The client and the server call it. |
| A save from a built-in layout sets only the choice in use | `save_target` in `mu-config`, and the scenario "A save from the built-in layout of one world". |
| No test waits for the clock or needs a real mouse | Review, and `review-lint` flags a sleep in a test. |
| The crate rules | `crates/mu-cli/tests/workspace_boundary.rs`, with `toml_edit` added to the row of `mu-server`, `winnow` and `insta` to the row of `mu-config`, and the edge from `mu-core` to `mu-config`. The row of `mu-server` already holds `tempfile`, which moves from `[dev-dependencies]` to `[dependencies]`. |
| An agent and a person change a layout the same way | `EditLayout` and the client commands call the same operations. The scenario "An edit gives the same file as the command" compares the two. |

## Risks / Trade-offs

- [The writer of plans is the hardest part.] → It runs only when no edit of one line gives the wanted screen. Task 1.1 builds it first with its property test. If the round trip fails for a shape, the plan stops there.
- [The order of the lines carries the shape, and a reader can miss it.] → Each listing shows the numbers, and `mu layout show` prints the picture beside the lines.
- [A rewrite of the plan changes the lines that the player wrote.] → It happens only at step 3 of D3, the ids stay, and the client says so.
- [A key press writes a file on the server.] → The server checks each write and writes a whole file with a rename. It writes only two places.
- [Two clients save at once, and one save is lost.] → One person uses mu. The proposal states that the last save wins, and #173 adds the check.
- [A terminal sends no mouse reports.] → Every mouse action has a command and a key in arrange mode.
- [A drag resizes when the player wants to select text.] → Only a press on a divider or on a title row starts a resize.
- [A change to a layout that several worlds use changes each of them.] → The proposal states it. `/layout save <name>` gives the world in focus its own copy.
- [Two devices can give one saved name. The device `a.b` with the game `c` gives the name of the device `a` with the world `b/c`.] → Both are files of one person, and the last save wins. `/layout save <name>` avoids it.
- [The screen changes shape when the focus changes.] → Only for a player who sets a world entry. Each world keeps its scroll positions and its editor texts.
- [The change is large.] → Five layers, each with its tests. Layers 1 and 2 give a usable result with no write path.
- [`Node`, `Leaf` and `Size` of `mu-tui` go away.] → They are public types with no user outside the workspace.

## Migration Plan

None for the way forward. A configuration directory with no new field gives the screen of today. The project has no release, so the new calls and fields need no compatibility step.

A step back to a binary from before this change needs one manual step, because that binary refuses the new fields and makes the layer bad:

1. Remove `layout`, `[world-layouts]`, `[tabs]` and `[statusline]` from each file under `clients/`.
2. Move the directory `clients/layouts/` out of the configuration directory.

The guide page of task 3.10 states these two steps.

## Open Questions

- The picture of `mu layout list` has a fixed size of 40 columns and 12 rows. A flag for another size can come later.
