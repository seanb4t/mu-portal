# Design: client-shell

## Context

See proposal.md for the reasons. The decision page `terminal-client-architecture` already fixes the shape of `mu-tui`. One model, one `update` that returns effects, and one `view`. A keymap that is data maps a key to an `Action`. Each widget is a `StatefulWidget` keyed by its id. Input and output happen only through effects and `mu-client`.

Facts from the code at e96b221:

- `Action` (`crates/mu-tui/src/update.rs`) is a `Copy` enum of 13 variants with no names. `Keymap` (`keymap.rs`) is a list of `(Chord, Action)` pairs and a wheel list. `Keymap::default` is the only source of bindings. A chord with modifiers and no entry takes the entry of the plain key.
- `mu-client` already has `connect` and `disconnect`. No effect of the client calls them.
- `mu-cli` lists the worlds once before `mu_tui::run`. If no server answers, it starts one. That first call has no time limit. Only the wait after a start has `START_LIMIT` of 5 s. `start_server` passes only `--socket` to the server.
- `run.rs` lists the worlds a second time before `screen::open`, and builds the model with `Model::from_listing`. That wait can keep the terminal blank for up to 30 s (#161).
- `return_waiting` and `put_back` (`update.rs`) join the returned texts into one editor text. The notice of a refusal holds only the reason.
- The model has two kinds of `-- ` line. `Model::notice` keeps a line, which shows again after a reset. `Model::hint` is for one moment.
- `text::line` gives the `> ` and `-- ` prefixes of each source, and cleans the text.
- `mu-cli` reads `server.toml` with `mu_core::world::parse_toml` and a `Hidden` value (`crates/mu-cli/src/server_config.rs`). Its errors name the file, the line and the column. `mu-tui` has no `serde`.
- The world resolver is in `mu-core` (`crates/mu-core/src/world/dir.rs`). It refuses each top-level `.toml` except `global.toml` and `SERVER_FILE`. `is_server_file` also takes that name in another case, and the same file by another name.

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

### D1. The registry is a table of entries, and the keymap binds entries

`Action` stops being `Copy` and gains the variants that need data: `Connect(Option<String>)`, `Disconnect(Option<String>)`, `Focus(String)`, `FocusIndex(u8)`, `ScrollTop`, `ScrollBottom`, `HistoryPrev`, `HistoryNext` and `Help`. A new module `registry.rs` holds one static table. Each row is an `Entry` with these fields:

- `name`: for example `focus-next`.
- `description`: one line.
- `arg`: an `Arg` with a kind and whether it is required. The only kind now is `ArgKind::World` (D10).
- `edits`: true for an entry that acts on the text of the editor.
- `build`: a `fn(Option<String>) -> Action`.

The keymap binds a chord to an `EntryId`, the place of an entry in the table. The wheel does the same. At a key, the loop builds the `Action` from the entry with no argument. So the palette of `client-layout` and the tests can read the key of an entry with no map back from an `Action`.

Three front ends feed one path. The keymap gives an entry for a chord. A command is parsed to an entry and an argument. The picker of `client-layout` will give a chosen row. Each one sends `Msg::Run { entry: EntryId, arg: Option<String> }`, which replaces `Msg::Action(Action)`. `update` calls `build` and then runs one `act(model, Action)`. No message takes an `Action` from outside, so no front end can skip the registry, and the result cannot differ by front end.

Names that take a number become one entry for each value: `scroll-page-up` and `scroll-page-down`, `scroll-up` and `scroll-down` (3 rows), `focus-1` to `focus-9`.

The alternative was a trait object for each command. It was rejected, because the `Action` enum already carries the TEA dispatch, and a table keeps the help and the picker a plain iteration.

Enforced by: `Msg` has no variant that holds an `Action`, which the compiler checks. A unit test that each name is unique and matches `^[a-z]+(-[a-z0-9]+)*$`. A unit test that each binding of `Keymap::default` names an entry with no required argument. The spec test "A key and a command give the same focus".

### D2. `mu-cli` reads the file, `mu-tui` owns the type

`mu-tui` defines `ClientConfig` with `serde::Deserialize` and `deny_unknown_fields`. Its fields are `echo: Echo` and `keys: BTreeMap<String, String>`. `mu-cli` reads `client.toml` as it reads `server.toml`. It joins the name to the directory, and gives the defaults for a missing directory or file. It parses the text with `mu_core::world::parse_toml` and `Hidden::Tables(&[])`, because the file holds no secret. So a parse error names the file, the line and the column, in the format of `server.toml`.

`ClientConfig::keymap()` in `mu-tui` turns the `[keys]` table into a `Keymap` over the default (D3). It refuses an unknown entry, and an entry whose argument is required. It returns an error that names the key and the entry. `mu-cli` puts the path of `client.toml` before that error, so stderr names the file for each bad binding as for a parse error. A file that exists and cannot be read is an error that names the path. If `--config` or `MU_CONFIG` names a missing directory, `mu client` writes the warning of `ConfigArgs::worlds` on stderr before the screen opens, with the verb `client`, and goes on with the defaults.

So `mu-tui` gains only `serde`. The TOML parse stays in `mu-core`, which already does it.

The alternative was a parse inside `mu-tui` with `toml`. It was rejected, because it puts a second TOML stack in the client and splits the error format between two crates.

Enforced by: the `workspace_boundary.rs` allowlist (`serde` added for `mu-tui`, `toml` not added), and the spec tests of `client-config`.

### D3. Key names

A key name is modifiers from `ctrl`, `alt`, `shift` and `super`, then one key, joined with `+`. A key is one character that is not an uppercase letter and not `+`, the name `plus` for the `+` key, or a name of a special key. So digits such as the `1` of `alt+1` are keys. The special keys are `enter`, `tab`, `backspace`, `esc` and `space`. The others are `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`, `insert`, `delete` and `f1` to `f12`. So `+` is only the joiner. An uppercase character is refused, because `shift+a` names that key. The order of the modifiers does not matter, and a repeated modifier is refused.

In the kitty path, Termina keeps the unshifted letter, so Shift+Z arrives as `Char('z')` with `SHIFT`. The default redo at head is the same chord, `Char('z')` with `CONTROL` and `SHIFT`. A legacy terminal gives `Char('Z')`, with or without `SHIFT`. So the parse makes `shift+z` the chord `Char('z')` with `SHIFT`. At a key, the lookup turns an uppercase letter into its lowercase letter with `SHIFT` before it looks up the chord. So `ctrl+shift+z` in `[keys]` replaces the default redo. Shift with a character that is not a letter is refused, because the terminal gives the shifted character instead, such as `!` for Shift+1. The player binds that character itself. Task 1.3 checks these reports in Ghostty and Terminal.app.

`ClientConfig::keymap()` parses each name to a chord. If two names give one chord, it refuses both and names them. `"none"` binds the chord to the entry `ignore`, so the key does nothing. The fallback from a modified chord to the plain key then cannot reach a send.

The engram rule 5rk42v8qs1 of this repository asks for an open source crate before own code. Task 1.2 looks for a crate that parses such names into the key type of Termina, or into a neutral type. `crokey` parses only into crossterm types, so it does not fit as is. If no crate fits, a parser of about 60 lines goes in `keymap.rs`. A property test checks that it reads back each name that it writes. It covers every set of modifiers and every key, `plus` included. It also checks that each order of the modifiers gives the same chord.

Enforced by: the tests of `keymap.rs`, that property test, and the spec scenarios of "A bad binding stops the client".

### D4. Commands are parsed in `update` at a send

At `Submit` or `SubmitLines`, `update` checks the editor text. One line that starts with `/` and not `//` is a command. The text after `/` is split at the first run of spaces into a name and an argument. An argument of only spaces is no argument.

The check runs before `build`, in this order:

1. The name must name an entry, and the entry must not have `edits` set. The entry `ignore` has `edits` set, although it edits nothing, so `/ignore` is a bad command.
2. An entry with no `arg` must get no argument.
3. A required argument must be present.
4. For `ArgKind::World`, an omitted optional world becomes the world in focus. With no world in focus, the command is bad. A given world must be in `candidates` (D10). This holds for `/connect`, `/disconnect` and `/focus` alike.

`act` also resolves an omitted world, for a key bound to `connect` or `disconnect`. With no world in focus, `act` adds the hint `no world` and sends nothing. So both front ends meet the same rule, and the command check only adds that the text stays in the editor.

A bad command adds a hint with `Model::hint` and leaves the editor as it is. A good command first adds its text to the history of the world in focus at the Enter, then clears the editor, and then runs `act`. So `/focus beta/bryn` goes into the history of the world that had the focus before it. With no world in focus, a command enters no history. `//` is removed to one `/`, and the text goes the normal send path.

`Connect` and `Disconnect` give the new effects `Effect::Connect { world }` and `Effect::Disconnect { world }`. The loop calls `mu-client` in a spawned task with the same 30 s limit as a send, and sends back `Msg::Control { world, result }`. An error becomes a hint. A success shows nothing new, because the server already sends its system events.

`Help` adds one hint line for each entry. The line holds the name, the argument of the entry, and the description.

The quit arm: today each key that is not `Quit` clears `quit_armed`. A `Quit` from a command sets the arm with the mark `by_command`. With that mark, no key clears the arm by itself: a key that edits the editor, and the Enter or Alt+Enter that submits, leave it. The parse of the submitted text decides. A `/quit` quits. A text that goes to a game, or any other command, clears the arm. A bad command also clears it, so a mistyped `/qiut` never leaves the client. So the Enter of the second `/quit` cannot clear the arm before the parse.

Enforced by: the spec tests of `client-commands`, and unit tests of the parser in `tests/update.rs` for `/connect `, `/`, `/ focus` and `/quit now`.

### D5. History is a state of each world

`World` gains `history: History`. `History` holds a `VecDeque<String>` of at most 1000 entries. Its one push drops an empty text and a text equal to the newest entry, for every caller: a send, a command, a focus change and `put_back`. History keeps a text as the player typed it. So `//me waves` stays with both slashes, and a recall sends `/me waves` again. The walk state is in the model, not in a world, because one pose editor serves all worlds. It holds the world of the walk, the place in the history, and the draft that the first Up saved. A send ends the walk.

`HistoryPrev` and `HistoryNext` are bound to Up and Down. Up acts only on the first logical line of the pose editor, and Down only on the last. On other lines, `update` gives the key to the editor as before. A logical line is a line between line breaks, not a wrapped row.

One function `set_focus(model, index)` in `update.rs` changes the focus. It sets the index, clears the unseen mark of that world, and ends a walk. `move_focus`, `focus-1` to `focus-9`, `/focus` and the end of the first list all call it. When a walk ends, the editor keeps its text, and the draft goes into the history of the world of the walk.

Down with no walk does nothing. It does not restore an empty draft.

Enforced by: the spec tests of the history requirements, and a property test that a walk of Up and Down with no send gives back the draft.

### D6. Returned sends go to history (#151)

`put_back` changes, for a refusal and for `return_waiting` alike:

1. It ends a walk first. The text that the editor held is then the saved draft, not the entry on screen.
2. The first returned text goes alone into the editor, with the cursor at the start of its first line. So the next Up recalls history at once.
3. The other returned texts, and then the held text, go into the history of their world. They are pushed in reverse order, so the first Up gives the next one. A stream end or a reset puts the held text into the history of the world in focus.

The line of a refusal in `on_reply` stays a push to the pane of the refused world, as at head. It holds the reason. If texts went into history, it also holds their count. With none, it holds the reason alone. The line of `return_waiting` stays a hint, as at head. It keeps its count of each world and adds `Up recalls the rest`.

The alternative was a marker line between the texts in the editor. #151 rejects it, because Alt+Enter sends the marker as a command.

Enforced by: the spec scenarios of "A refused text stops the sends after it" in `tests/app.rs`.

### D7. Echo

`text::line` takes the `line_break` of the world of the event. For a player event it returns the rows: the first part with `> `, and each other part with two spaces. A world with no known `line_break` gives one row. So the prefix rule stays in one place. `update` only chooses the style by `echo`. With `Echo::Dim` the rows get `Modifier::DIM`. With `Echo::Hide` the client adds no row, but the event still sets the unseen mark of a world out of focus. The server keeps each sent line in the journal as before.

Enforced by: the spec scenarios of the echo requirements on `TestBackend`. A test checks that a hidden echo in a world out of focus sets its mark.

### D8. The screen opens before the first list

`run.rs` opens the screen first, and sends the first list as an effect. `Msg::Listed` becomes the one path of every list, the first included. `Model::from_listing` goes away. The tests build a model with `Model::new` and then give it `Msg::Listed`, so they run the path that `run.rs` runs.

The model starts with `Listing::Asked` and a flag `first_list`. The top row shows `listing worlds` while the flag is set. The stream starts from the first event, so its events can add worlds before the list ends. While the flag is set, the focus stays empty: `Model::world` adds the world and gives it no focus. At the end of the list, `set_focus` gives the focus to the first world. Each end of the first list clears the flag: a reply with worlds, a failure, and the 30 s limit. A failed first list adds a notice with the reason, as `from_listing` did. A reset while the first list is with the server keeps the flag, and the list is asked again as today.

The list that `mu-cli` makes before `run` decides whether to start a server. It gets the limit `START_LIMIT`, so a server that accepts and does not answer cannot keep the terminal blank. `start_server` also passes the configuration directory of `mu client`, so the server that it starts reads the same worlds.

Enforced by: `TestBackend` tests of the first view before `Msg::Listed`, of a send before it, and of a failed first list. The spec test "The client takes the flag".

### D9. The architecture model

`architecture/model.c4` gains the part `registry` between `keymap` and `update`, and the edge `update -> registry`. The dynamic view of one key press in `views.c4` names the registry. This follows the contributing page for diagrams: a change that alters the architecture updates the model.

Enforced by: review alone, and the LikeC4 build of the docs.

### D10. Room for completion

Completion of a command name or an argument, with a fuzzy list in the style of fzf, comes in a later change. This change keeps three seams for it:

- Each argument has a kind (`ArgKind`). A function `candidates(model, kind) -> Vec<String>` gives the values of a kind. For `World` it gives the worlds of the model in the order of the top row. Step 4 of D4 uses it, so the seam has a caller and a test from the start.
- The command parser returns a parse that knows where the cursor is: in the name, in the argument, or past it. A completion widget then asks the registry for names or `candidates` for the kind, with no second parser.
- The default keymap leaves Tab, Shift+Tab and Ctrl+Space with no binding. The guide page says that they are kept for completion.

The picker of `client-layout` and completion can share one fuzzy matcher. By the engram rule 5rk42v8qs1 that is a crate, for example `nucleo-matcher`, which a later change picks.

Enforced by: the unit test `world_candidates_follow_the_top_row`, and the test `default_keymap_leaves_tab_free`.

### D11. The resolver takes a list of top-level files

In `crates/mu-core/src/world/dir.rs`, `CLIENT_FILE` goes beside `SERVER_FILE`. `is_server_file` becomes one check over the list of top-level files that make no world. It keeps the case rule and the same-file rule for each name. So `client.toml` gets the same edge behavior as `server.toml`, with no second copy.

Enforced by: the tests of `world_config.rs` for `client.toml` in another case, unreadable, and as a hard link.

## Risks / Trade-offs

- [Up and Down now have two meanings] In a pose of several lines, an Up on the first line recalls history. A player can do that by mistake. → The draft comes back with Down, so nothing is lost.
- [`Action` loses `Copy`] Each match on `Action` that copies it now needs a clone. → The keymap holds an `EntryId` (D1), so only `build` makes an `Action`, once for each key.
- [The echo default changes to dim] A player who wants the old look sets `echo = "show"`. → The configuration page says so (#164).
- [History in memory only] A crash loses it. → History on disk is a later change, beside #156.
- [Termina key names] Alt+digit and Alt+Home depend on how each terminal reports them. → Task 1.3 checks the kitty protocol reports in Ghostty and in Terminal.app, and records any terminal that does not send them.

## Migration Plan

No migration. No release exists, and `client.toml` is new. An existing `MU_CONFIG` directory with no `client.toml` gives the defaults.

## Open Questions

- The exact text of each description in the registry. It changes no spec and no task.
