# Design: client-shell

## Context

See proposal.md for the reasons. The decision page `terminal-client-architecture` already fixes the shape of `mu-tui`. One model, one `update` that returns effects, and one `view`. A keymap that is data maps a key to an `Action`. Each widget is a `StatefulWidget` keyed by its id. Input and output happen only through effects and `mu-client`.

Facts from the code at e96b221:

- `Action` (`crates/mu-tui/src/update.rs`) is a `Copy` enum of 15 variants with no names. `Keymap` (`keymap.rs`) is a list of `(Chord, Action)` pairs and a wheel list. `Keymap::default` is the only source of bindings.
- `mu-client` already has `connect` and `disconnect`. No effect of the client calls them.
- `mu-cli` lists the worlds once before `mu_tui::run`. If no server answers, it starts one. `run.rs` then lists them a second time before `screen::open`. That second list can keep the terminal blank for up to 30 s (#161).
- `return_waiting` and `put_back` (`update.rs`) join the returned texts into one editor text.
- `mu-cli` already parses TOML with `toml` and `serde_path_to_error` (`crates/mu-cli/src/config.rs`). `mu-tui` has no `serde`.
- `config.rs` in `mu-cli` resolves the configuration directory for `mu server` and `mu world`. The world resolver refuses any top-level `.toml` other than `server.toml`.

## Goals / Non-Goals

Goals:

- One registry that keys, commands and the later picker all reach through the same dispatch. A front end only chooses an entry and its argument.
- A configuration type that `client-layout` can extend with `[layout]`, `[tabs]` and `[statusline]` without a change to how the file is found or checked.

Non-goals:

- The picker itself. `client-layout` adds it as an overlay that lists the registry.
- Key sequences of more than one chord, such as `ctrl+x ctrl+c`.
- A reload of `client.toml` while the client runs.
- Completion of arguments and suggestions while the player types. D10 keeps room for them.

## Decisions

### D1. The registry is a table of entries over `Action`

`Action` stops being `Copy` and gains the variants that need data: `Connect(Option<String>)`, `Disconnect(Option<String>)`, `Focus(String)`, `FocusIndex(u8)`, `ScrollTop`, `ScrollBottom`, `HistoryPrev`, `HistoryNext` and `Help`. A new module `registry.rs` holds one static table. Each row is an `Entry` with these fields:

- `name`: for example `focus-next`.
- `description`: one line.
- `arg`: an `Arg` with a kind and whether it is required. The only kind now is `ArgKind::World`. The kind, not a plain string, says what an argument can hold (D10).
- `edits`: true for an entry that acts on the text of the editor.
- `build`: a `fn(Option<String>) -> Action`.

Three front ends feed one path. The keymap gives an `Action` for a chord. A command is parsed to an entry and an argument, and then `build`. The picker of `client-layout` will do the same with a chosen row. `update` has one `act(model, Action)` that every front end calls, so the result cannot differ by front end.

Names that take a number become one entry for each value: `scroll-page-up` and `scroll-page-down`, `scroll-up` and `scroll-down` (3 rows), `focus-1` to `focus-9`.

The alternative was a trait object for each command. It was rejected, because the `Action` enum already carries the TEA dispatch, and a table keeps the help and the picker a plain iteration.

Enforced by: a unit test that each name is unique and matches `^[a-z]+(-[a-z0-9]+)*$`. A unit test that each action of `Keymap::default` comes from an entry, so no key bypasses the registry. The spec test "A key and a command give the same focus".

### D2. `mu-cli` reads the file, `mu-tui` owns the type

`mu-tui` defines `ClientConfig` with `serde::Deserialize` and `deny_unknown_fields`. Its fields are `echo: Echo` and `keys: BTreeMap<String, String>`. `mu-cli` finds `client.toml` with the resolver of the configuration directory. It parses the file with `toml` and `serde_path_to_error`, and gives the value to `mu_tui::run`. `ClientConfig::keymap()` in `mu-tui` turns the `[keys]` table into a `Keymap` over the default. It returns an error that names the key and the value, and `mu-cli` prints that error.

So `mu-tui` gains only `serde`. The TOML parse stays in the crate that already does it.

The alternative was a parse inside `mu-tui` with `toml`. It was rejected, because it puts a second TOML stack in the client and splits the error format between two crates.

Enforced by: the `workspace_boundary.rs` allowlist (`serde` added for `mu-tui`, `toml` not added), and the spec tests of `client-config`.

### D3. Key names

A key name is lowercase modifiers from `ctrl`, `alt`, `shift` and `super`, then one key, joined with `+`. A key is one character or one name from this list: `enter`, `tab`, `backspace`, `esc`, `space`, `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`, `insert`, `delete` and `f1` to `f12`. The order of the modifiers does not matter.

Repository rule 5rk42v8qs1 asks for a crate first. Task 1.2 looks for a crate that parses such names into the key type of Termina, or into a neutral type. `crokey` parses only into crossterm types, so it does not fit as is. If no crate fits, a parser of about 60 lines goes in `keymap.rs`. A property test checks that it reads back each name that it writes.

Enforced by: the tests of `keymap.rs`, and the spec scenario "An unknown key name".

### D4. Commands are parsed in `update` at a send

At `Submit` or `SubmitLines`, `update` checks the editor text. One line that starts with `/` and not `//` is a command. The text after `/` is split at the first run of spaces into a name and an argument. A bad command adds a notice and leaves the editor as it is. A good command clears the editor, adds the text to the history of the world in focus, and runs `act`. `//` is removed to one `/`, and the text goes the normal send path.

`Connect` and `Disconnect` give the new effects `Effect::Connect { world }` and `Effect::Disconnect { world }`. The loop calls `mu-client` in a spawned task with the same 30 s limit as a send, and sends back `Msg::Control { world, result }`. An error becomes a notice. A success shows nothing new, because the server already sends its system events.

`Help` adds one notice line for each entry. The line holds the name, the argument of the entry, and the description.

Enforced by: the spec tests of `client-commands`, and unit tests of the parser in `tests/update.rs`.

### D5. History is a state of each world

`World` gains `history: History`. `History` holds a `VecDeque<String>` of at most 1000 entries, the place of the walk, and the draft that the first Up saved. A send resets the walk. `HistoryPrev` and `HistoryNext` are bound to Up and Down. Up acts only on the first logical line of the pose editor, and Down only on the last. On other lines, `update` gives the key to the editor as before. A logical line is a line between line breaks, not a wrapped row. So in a long wrapped line, Up recalls history from any wrapped row of the first line.

Enforced by: the spec tests of the history requirements, and a property test that a walk of Up and Down with no send gives back the draft.

### D6. Returned sends go to history (#151)

`put_back` changes. The first returned text goes alone into the editor. The other returned texts, and then the text that the editor held, go into the history of their world. They are pushed in reverse order, so the first Up gives the next one. A stream end or a reset puts the held text into the history of the world in focus. The notice keeps its count of each world and adds `Up recalls the rest`.

The alternative was a marker line between the texts in the editor. #151 rejects it, because Alt+Enter sends the marker as a command.

Enforced by: the spec scenarios of "A refused text stops the sends after it" in `tests/app.rs`.

### D7. Echo

`Model` gains `echo: Echo`. When `update` shows a player event, it splits the text at the `line_break` of its world, if the world is listed. It shows the first part with `> ` and each other part with two spaces. With `Echo::Dim` the lines get `Modifier::DIM`. With `Echo::Hide` the client adds no line, but the event still counts for the unseen mark. The server keeps each sent line in the journal as before.

Enforced by: the spec scenarios of the echo requirements, on `TestBackend`.

### D8. The screen opens before the first list

`run.rs` opens the screen first, and sends the first list as an effect. The model starts with `Listing::Asked` and a flag `first_list`, and the top row shows `listing worlds` until `Msg::Listed` arrives. The list that `mu-cli` makes before `run` stays, because it decides whether to start a server. That list has its own 5 s limit.

Enforced by: a `TestBackend` test of the first view before `Msg::Listed`.

### D9. The architecture model

`architecture/model.c4` gains the part `registry` between `keymap` and `update`, and the edge `update -> registry`. The dynamic view of one key press in `views.c4` names the registry. This follows the contributing page for diagrams: a change that alters the architecture updates the model.

Enforced by: review alone, and the LikeC4 build of the docs.

### D10. Room for completion

Completion of a command name or an argument, with a fuzzy list in the style of fzf, comes in a later change. This change keeps three seams for it:

- Each argument has a kind (`ArgKind`). A function `candidates(model, kind) -> Vec<String>` gives the values of a kind. For `World` it gives the worlds of the model in the order of the top row. This change uses it to check `/focus` and `/connect`, so the seam has a caller and a test from the start.
- The command parser returns a parse that knows where the cursor is: in the name, in the argument, or past it. A completion widget then asks the registry for names or `candidates` for the kind, with no second parser.
- The default keymap leaves Tab and Shift+Tab with no binding, and the guide page says that they are kept for completion. Ctrl+Space also stays free for a suggestion list.

The picker of `client-layout` and completion can share one fuzzy matcher. By rule 5rk42v8qs1 that is a crate, for example `nucleo-matcher`, which a later change picks.

Enforced by: the unit test `world_candidates_follow_the_top_row`, and the test `default_keymap_leaves_tab_free`.

## Risks / Trade-offs

- [Up and Down now have two meanings] In a pose of several lines, an Up on the first line recalls history. A player can do that by mistake. → The draft comes back with Down, so nothing is lost.
- [`Action` loses `Copy`] Each match on `Action` that copies it now needs a clone. → `Action` stays small, and the keymap returns a clone.
- [The echo default changes to dim] A player who wants the old look sets `echo = "show"`. → The configuration page says so.
- [History in memory only] A crash loses it. → History on disk is a later change, beside #156.
- [Termina key names] Alt+digit and Alt+Home depend on how each terminal reports them. → Task 1.3 checks the kitty protocol reports in Ghostty and in Terminal.app, and records any terminal that does not send them.

## Migration Plan

No migration. No release exists, and `client.toml` is new. An existing `MU_CONFIG` directory with no `client.toml` gives the defaults.

## Open Questions

- The exact text of each description in the registry. It changes no spec and no task.
