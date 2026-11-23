# Design: client-shell

## Context

See proposal.md for the reasons. The decision page `terminal-client-architecture` already fixes the shape of `mu-tui`. One model, one `update` that returns effects, and one `view`. A keymap that is data maps a key to an `Action`. Each widget is a `StatefulWidget` keyed by its id. Input and output happen only through effects and `mu-client`.

Facts from the code at e96b221:

- `Action` (`crates/mu-tui/src/update.rs`) is a `Copy` enum of 13 variants with no names. `Keymap` (`keymap.rs`) is a list of `(Chord, Action)` pairs and a wheel list. `Keymap::default` is the only source of bindings. A chord with modifiers and no entry takes the entry of the plain key.
- `mu-client` already has `connect` and `disconnect`. No effect of the client calls them.
- `mu_tui::run` takes only the `PortalClient`. `Model::new` always uses `Keymap::default`, and it writes the focus directly.
- `mu-cli` lists the worlds once before `mu_tui::run`. If no server answers, it starts one. That first call has no time limit. Only the wait after a start has `START_LIMIT` of 5 s. `start_server` passes only `--socket` to the server.
- `run.rs` lists the worlds a second time before `screen::open`, and builds the model with `Model::from_listing`. That wait can keep the terminal blank for up to 30 s (#161).
- The model has one pose editor, the entry `EditorId::POSE` of `Model::editors`, and the layout names that id in its editor leaf. All worlds share it.
- `return_waiting` and `put_back` (`update.rs`) join the returned texts into one editor text. The notice of a refusal holds only the reason.
- The model has two kinds of `-- ` line. `Model::notice` keeps a line, which shows again after a reset. `Model::hint` is for one moment.
- `text::line` gives the `> ` and `-- ` prefixes of each source, and cleans the text.
- `mu-cli` reads `server.toml` with `mu_core::world::parse_toml` and a `Hidden` value (`crates/mu-cli/src/server_config.rs`). Its errors name the file, the line and the column. `mu-tui` has no `serde`.
- The world resolver is in `mu-core` (`crates/mu-core/src/world/dir.rs`). It refuses each top-level `.toml` except `global.toml` and `SERVER_FILE`. `is_server_file` also takes that name in another case, but only when it is the same file as `SERVER_FILE`. A hard link under another name stays an error.

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

Names that take a number become one entry for each value: `scroll-page-up` and `scroll-page-down`, `scroll-up` and `scroll-down` (3 rows), `focus-1` to `focus-9`. The nine rows `focus-1` to `focus-9` come from one helper over the digit. So the name, the description and the `FocusIndex` value of a row come from one number.

The alternative was a trait object for each command. It was rejected, because the `Action` enum already carries the TEA dispatch, and a table keeps the help and the picker a plain iteration.

Enforced by: `Msg` has no variant that holds an `Action`, which the compiler checks. A unit test that each name is unique and matches `^[a-z]+(-[a-z0-9]+)*$`. The unit test `focus_rows_match_their_index` checks each `focus-<n>` row against its index. A unit test that each binding of `Keymap::default` names an entry with no required argument. The spec test "A key and a command give the same focus".

### D2. `mu-cli` reads the file, `mu-tui` owns the type

`mu-tui` defines `ClientConfig` with `serde::Deserialize` and `deny_unknown_fields`. Its fields are `echo: Echo` and `keys: BTreeMap<String, String>`. `mu-cli` reads `client.toml` and `server.toml` with one function, `config_file::read(dir, name, hidden)`, generic over a type with `Deserialize` and `Default`. It joins the name to the directory, gives the defaults for a missing directory or file, and names the path in a read error. `server_config::read` calls it and then makes a relative `certificate` a path under the directory, so the steps of the read are not copied. It parses the text with `mu_core::world::parse_toml` and the `hidden` value of its caller. `server_config::read` passes `Hidden::All`, as at head. The reader of `client.toml` passes `Hidden::Tables(&[])`, because that file holds no secret. So a parse error names the file, the line and the column, in the format of `server.toml`.

`ClientConfig::keymap()` in `mu-tui` turns the `[keys]` table into a `Keymap` over the default (D3). It refuses an unknown entry, and an entry whose argument is required. It returns an error that names the key and the entry. `mu-cli` puts `client.toml: keys.<key>` before that error, as a field error of `server.toml` names the file and the field. So stderr names the file for each bad binding in the format of a parse error. A file that exists and cannot be read is an error that names the path. If `--config` or `MU_CONFIG` names a missing directory, `mu client` warns on stderr before the screen opens, and goes on with the defaults. The check and its text move out of `ConfigArgs::worlds` into `ConfigArgs::warn_missing(verb)` in `config.rs`. `worlds()` and `client::run` both call it, so the client resolves no world to get the warning.

One method `ClientConfig::settings() -> Result<Settings, KeyError>` in `mu-tui` calls `keymap()` and copies the echo. `mu-cli` calls it and passes the result to the client as `mu_tui::run(client, settings)`. So a table that `client-layout` adds changes `mu-tui` only, and `mu-cli` only reads, parses and prints the error. `Settings` is a public type of `mu-tui` that holds the `Keymap` and the `Echo`. `run.rs` builds the model with `Model::starting(settings)` (D8). The tests of `app.rs` give a keymap or an echo to a model with the public methods `Model::with_keymap` and `Model::with_echo`. So `mu client` and the tests set both through one public path.

So `mu-tui` gains only `serde`. The TOML parse stays in `mu-core`, which already does it.

The alternative was a parse inside `mu-tui` with `toml`. It was rejected, because it puts a second TOML stack in the client and splits the error format between two crates.

Enforced by: the `workspace_boundary.rs` allowlist (`serde` added for `mu-tui`, `toml` not added), and the spec tests of `client-config`.

### D3. Key names

A key name is modifiers from `ctrl`, `alt`, `shift` and `super`, then one key, joined with `+`. A key is one character that is not an uppercase letter and not `+`, the name `plus` for the `+` key, or a name of a special key. So digits such as the `1` of `alt+1` are keys. A control character, such as a tab, a carriage return or DEL, is refused, because Termina gives those keys by their special names, not as `Char`. The special keys are `enter`, `tab`, `backspace`, `esc` and `space`. Termina gives Shift+Tab as `KeyCode::BackTab` with `SHIFT`, so the parse makes `shift+tab` that chord, and the lookup turns `Tab` with `SHIFT` into it. The others are `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`, `insert`, `delete` and `f1` to `f12`. A function key name has no leading zero, so `f0`, `f01` and `f13` are refused. So `+` is only the joiner. An uppercase character is refused, because `shift+a` names that key. The order of the modifiers does not matter, and a repeated modifier is refused.

The client pushes only `DISAMBIGUATE_ESCAPE_CODES`. So Shift and a letter alone arrive as text, and Termina gives the uppercase letter, such as `Char('Z')` with `SHIFT`. A chord with Ctrl, Alt or Super keeps the unshifted letter, so Ctrl+Shift+Z arrives as `Char('z')` with `CONTROL` and `SHIFT`. The default redo at head is that chord. A legacy terminal gives `Char('Z')`, with or without `SHIFT`. So the parse makes `shift+z` the chord `Char('z')` with `SHIFT`. At a key, the lookup turns an uppercase letter into its lowercase letter with `SHIFT` before it looks up the chord. Only the lookup sees that lowered chord. A key that no binding takes reaches the editor as it arrived, so Shift and a letter still type the uppercase letter. So `ctrl+shift+z` in `[keys]` replaces the default redo. Shift with a character that is not a letter is refused, because the terminal gives the shifted character instead, such as `!` for Shift+1. The keys `space` and `plus` are characters too, so `shift+space` and `shift+plus` are refused. The player binds that character itself.

Task 1.3 read the parser of Termina 0.3.3 (`src/parse.rs`), the kitty keyboard protocol, the configuration of Ghostty 1.4.0 (`ghostty +show-config --default --docs`) and the Terminal guide of Apple, on 2026-11-23. Termina gives an `ESC` and a key as that key with `ALT`, `CSI Z` as `BackTab` with `SHIFT`, an uppercase legacy character with `SHIFT`, and `CSI 1;3H` as `Home` with `ALT`. These are the terminals where a default key does not arrive:

| Key | Ghostty | Terminal.app |
|---|---|---|
| Alt+1 to Alt+9, Alt+C | arrives on a U.S. keyboard layout, where Option is Alt by default. On another layout, Option types a character unless `macos-option-as-alt` is set. | does not arrive by default. Option types a character, such as `¡`, unless "Use Option as Meta key" is set. |
| Alt+Home, Alt+End | arrives as above. Ghostty binds only Shift+Home, Shift+End, Super+Home and Super+End. | does not arrive by default, as above. |
| Shift and a letter | arrives as the uppercase letter with `SHIFT`. | arrives the same way. |
| Ctrl+Shift+Z | arrives as `CSI 122;6u`, which Termina gives as `Char('z')` with `CONTROL` and `SHIFT`. | does not arrive. Terminal.app has no kitty protocol and sends `0x1A`, the byte of Ctrl+Z, so the key runs undo. |
| Shift+Tab | arrives as `CSI Z`. Ghostty binds only Ctrl+Shift+Tab. | arrives as `CSI Z`. |

No row was checked with a key press in each terminal. In Terminal.app, the player can bind redo and the focus keys to other keys in `[keys]`.

`ClientConfig::keymap()` parses each name to a chord. If two names give one chord, it refuses both and names them. `"none"` binds the chord to the entry `ignore`, so the key does nothing. The fallback from a modified chord to the plain key then cannot reach a send.

The engram rule 5rk42v8qs1 of this repository asks for an open source crate before own code. Task 1.2 looks for a crate that parses such names into the key type of Termina, or into a neutral type. `crokey` 1.5.1 does not fit as is. Its `src/parse.rs` parses into `crossterm::event::KeyCode` and `KeyModifiers`, and its `Cargo.toml` needs `crossterm` ^0.29 with no feature that leaves it out (the crate source from crates.io, read on 2026-11-22). Task 1.2 searched crates.io and GitHub on 2026-11-23. No crate fits, so a parser of about 60 lines goes in `keymap.rs`. These are the candidates and why each one does not fit:

- `crokey` 1.5.1: it parses only into the key types of `crossterm`, as above.
- `keymap` 0.4.1 (MIT): its backends are `crossterm` and `termion`, not Termina.
- `terminput` 0.5.16 (MIT OR Apache-2.0): it converts the events of Termina, but it parses no key name from a string. Its `serde` form is the struct, not a name.
- `keybinds` 0.2.0 (MIT): it parses names into its own neutral type, but its grammar is wider than D3. It accepts any case of a modifier or a name, an uppercase character as a key, sequences of keys joined by spaces, `F1` to `F35`, and the modifiers `cmd`, `mod` and `option`. So the client would have to check the text again for each rule of D3, and that check is most of the 60 lines. A property test checks that it reads back each name that it writes. It covers every set of modifiers and every key, `plus` included. It also checks that each order of the modifiers gives the same chord.

Enforced by: the tests of `keymap.rs`, that property test, and the spec scenarios of "A bad binding stops the client".

### D4. Commands are parsed in `update` at a send

At `Submit` or `SubmitLines`, `update` checks the editor text. One line that starts with `/` and not `//` is a command. The text after `/` is split at the first run of spaces into a name and an argument. The argument loses its spaces at both ends before any check. So an argument of only spaces is no argument, and `/focus beta/bryn ` names `beta/bryn`.

The check runs before `build`, in this order:

1. The name must name an entry, and the entry must not have `edits` set. The entry `ignore` has `edits` set, although it edits nothing, so `/ignore` is a bad command.
2. An entry with no `arg` must get no argument.
3. A required argument must be present.
4. For `ArgKind::World`, an omitted optional world becomes the world in focus. With no world in focus, the command is bad. A given world must be in `candidates` (D10). This holds for `/connect`, `/disconnect` and `/focus` alike.

One function `world_arg(model, Option<String>) -> Result<String, Refusal>` in `update.rs` resolves a world argument. It gives the world in focus for an omitted world, and refuses an unknown world or no world in focus. Step 4 of the command check and `act`, for a key bound to `connect` or `disconnect`, both call it. On a refusal, `act` adds the hint and sends nothing. The command check only adds that the text stays in the editor.

One helper `no_world(what)` gives the text `no world has the focus, so <what>`. The send, the join, the key and the command all use it.

A bad command adds a hint with `Model::hint` and leaves the editor as it is. A good command first adds its text to the history of the world in focus at the Enter, then clears the editor, and then runs `act`. So `/focus beta/bryn` goes into the history of the world that had the focus before it. With no world in focus, a command enters no history. `//` is removed to one `/`, and the text goes the normal send path.

`Connect` and `Disconnect` give the new effects `Effect::Connect { world }` and `Effect::Disconnect { world }`. The loop calls `mu-client` in a spawned task with the same 30 s limit as a send, and sends back `Msg::Control { world, result }`. At head `run.rs` already has one helper for this, `spawn_reply`, which runs a future that ends in a `Msg` and sends that message to the loop. The send uses it with `reply_of`, which bounds the call with `within` and adds the late text that the requirement "A send with no reply is refused" needs. `Connect` and `Disconnect` use `spawn_reply` too, with one new function `control_of(call, world, limit) -> Msg` beside `reply_of`. It bounds the call with `within` and maps the error to `Msg::Control`. So the loop keeps one spawn helper, and the send keeps its reason. An error becomes a hint. A success shows nothing new, because the server already sends its system events.

`Help` adds one hint line for each entry. The line holds the name, the argument of the entry, and the description.

The quit arm: today each key that is not `Quit` clears `quit_armed`. A `Quit` from a command sets the arm with the mark `by_command`. With that mark, no key clears the arm by itself: a key that edits the editor, and the Enter or Alt+Enter that submits, leave it. The parse of the submitted text decides. A `/quit` quits. A text that goes to a game, or any other command, clears the arm. A bad command also clears it, so a mistyped `/qiut` never leaves the client. So the Enter of the second `/quit` cannot clear the arm before the parse.

Enforced by: the spec tests of `client-commands`, and unit tests of the parser in `tests/update.rs` for `/connect `, `/`, `/ focus` and `/quit now`. The registry test `edits_flag_matches_the_editing_rows` checks that `edits` is set exactly on the rows that build `Submit`, `SubmitLines`, `Newline`, `JoinLines`, `DeleteToLineStart`, `Undo`, `Redo`, `Ignore`, `HistoryPrev` and `HistoryNext`.

### D5. Each world has its own editor and history

`World` gains two fields: `history: History` and `walk: Option<Walk>`. The model gains `pose_editors: HashMap<String, EditorState>`, the pose editor of each world by its id. It is a field of the model and not of `World`, because `view` splits the model into `Parts`, which borrows the worlds to read while it borrows the editors to write. A separate field lets `view` take the pose editor of the world in focus mutably beside the worlds. So the pose editor follows the focus. A half-typed pose stays with its world when the focus moves, and a text that comes back to a world goes into the editor of that world. The model keeps one more editor, `loose`, for the time when no world has the focus: before the first list ends, or with no world at all. The layout keeps its one slot `EditorId::POSE`. The view draws the editor of the world in focus in that slot, or `loose` when no world has the focus. One resolver `Model::editor_state(id)`, and its `_mut` form, gives the state of an `EditorId`. It maps `EditorId::POSE` to the entry of `pose_editors` for the world in focus, or to `loose`, and every other id to `Model::editors`. `Model::pose` becomes a call of it. `Model::input`, `Model::set_input`, the key path of the editor, `Model::editor`, the render of an editor leaf and the row count of `Size::Editor` all go through it. `view` resolves the pose editor once, before it builds `Parts`, and passes it in, so no code finds the pose editor its own way. `Model::editors` keeps only the editors that are not the pose editor. A change of the capabilities of the terminal reaches every world editor and `loose`. `Model::world` builds the editor of a new world with the current capabilities, as it builds the pane. In a run the probe ends before the first list, so each world editor gets the probed widths that way.

`History` holds a `VecDeque<String>` of at most 1000 entries. Its one push drops an empty text and a text equal to the newest entry, for every caller: a send, a command and `put_back`. History keeps a text as the player typed it. So `//me waves` stays with both slashes, and a recall sends `/me waves` again. `Walk` holds the place in the history and the draft that the first Up saved. A walk belongs to its world, so a focus change and a new world in the list do not touch it.

`HistoryPrev` and `HistoryNext` are bound to Up and Down. They act on the world in focus. Up acts only on the first logical line of the pose editor, and Down only on the last. On other lines, `update` gives the key to the editor as before. A logical line is a line between line breaks, not a wrapped row. The fallback of `Keymap::get` from a modified chord to the plain key skips `history-prev` and `history-next`. So Shift+Up, Ctrl+Up and Shift+Down reach the editor with their modifiers, as at head.

Down with no walk does nothing. It does not restore an empty draft. With no older entry, Up leaves the text and opens no walk. With no world in focus, Up and Down open no walk and leave the text of `loose`, also before the first list ends.

A send or a command during a walk ends the walk. The saved draft goes into the history of that world first, and then the sent text. So Up gives the sent text, and then the draft.

One method `Model::set_focus(index)` changes the focus. It sets the index and clears the unseen mark of that world. If no world had the focus, it also moves the text of `loose` into the editor of that world, and copies the kept notices into its pane. One method `Model::replay_notices(pane)` does that copy. `set_focus` and the `Reset` arm of `on_client` both call it, so the loop of the `Reset` arm is not copied. The editor of that world is empty then, because no key reached it before it had the focus. While `first_list` is set, it does nothing. `move_focus`, `focus-1` to `focus-9`, `/focus`, the `None` arm of `Model::world` and the end of the first list all call it.

The alternative was one pose editor for all worlds, as TinyFugue has one input line. It was rejected, because a text that comes back from a world out of focus then lands in the editor of another world, and a plain Enter sends it there. Each world already keeps its own history for the same reason.

Enforced by: the spec tests of the history and editor requirements, a property test that a walk of Up and Down with no send gives back the draft, and the test `a_focus_change_keeps_each_editor`.

### D6. Returned sends go to history (#151)

`put_back(world, texts)` changes, for a refusal and for `return_waiting` alike. It acts on the editor and the history of `world` only, whether or not that world has the focus:

1. It ends a walk of that world first, but it does not push the draft. The saved draft becomes the held text, not the entry on screen, and step 3 pushes it once. So the count of the refusal line includes the draft.
2. The first returned text goes alone into the editor of that world, with the cursor at the start of its first line. So the next Up recalls history at once.
3. The other returned texts, and then the held text, go into the history of that world. They are pushed in reverse order, so the first Up gives the next one.

A stream end or a reset calls `put_back` once for each world that has texts that waited. At head a line pushed to a pane sets no unseen mark. So `put_back` sets the unseen mark of its world when that world is out of focus, and the top row shows `*` after its id. A world out of focus shows its returned text when it gets the focus.

The line of a refusal in `on_reply` stays a push to the pane of the refused world, as at head. It holds the reason. If texts went into history, it also holds their count, as `(<n> in history)`. The count is of the entries that the push kept, because Up gives only those. A returned text equal to the newest entry is not counted. With none, it holds the reason alone. The line of `return_waiting` stays a hint, as at head. It keeps its count of each world and adds `Up recalls the rest`.

The alternative was a marker line between the texts in the editor. #151 says that the client has no line that it keeps in the editor but does not send, and that an empty line does not work, because Alt+Enter sends it as an empty command. A marker would need such a line, so this change keeps the texts apart in history instead.

Enforced by: the spec scenarios of "A refused text stops the sends after it" in `tests/app.rs`. The test `a_refusal_during_a_walk_of_the_history` asserts the count `(1 in history)` and one entry `draft`. The test `a_refusal_out_of_focus` asserts that the editor of the world in focus does not change, and that the refused world shows `*`. The test `a_refusal_out_of_focus_during_a_walk` asserts that `put_back` takes the draft of the walk of the refused world.

### D7. Echo

`text::line` takes the `line_break` of the world of the event. For a player event it returns the rows: the first part with `> `, and each other part with two spaces. A world with no known `line_break` gives one row. So the prefix rule stays in one place. `update` only chooses the style by `echo`. With `Echo::Dim` the rows get `Modifier::DIM`. With `Echo::Hide` the client adds no row, but the event still sets the unseen mark of a world out of focus. The server keeps each sent line in the journal as before.

Enforced by: the spec scenarios of the echo requirements on `TestBackend`, and the property test `echo_rows_join_back`. A test checks that a hidden echo in a world out of focus sets its mark.

### D8. The screen opens before the first list

`run.rs` opens the screen first, and sends the first list as an effect. `Msg::Listed` becomes the one path of every list, the first included. `Model::from_listing` goes away. `run.rs` builds the model with `Model::starting(settings)`: no world, `Listing::Asked`, and `first_list` set. The tests of the first list build the same model and then give it `Msg::Listed`, so they run the path that `run.rs` runs. `Model::new(worlds)` stays for the other tests. It leaves `first_list` clear and gives the focus through `set_focus`, so no constructor writes the focus directly.

The model starts with `Listing::Asked` and a flag `first_list`. The top row shows `listing worlds` while the flag is set. The stream starts from the first event, so its events can add worlds before the list ends. While the flag is set, the focus stays empty: `Model::world` adds the world and gives it no focus. Each end of the first list clears the flag: a reply with worlds, a failure, and the 30 s limit. Then, if the model holds a world, `set_focus` gives the focus to the first world, with the kept notices, such as the width notice of the probe. So a world that an event added stays in reach also after a failed list. A failed first list adds a notice with the reason, as `from_listing` did. A reset while the first list is with the server keeps the flag, and the list is asked again as today. The reply to the list asked before the reset does not end the first list. The reply to the list asked again ends it.

The list that `mu-cli` makes before `run` decides whether to start a server. It gets the limit `START_LIMIT`, so a server that accepts and does not answer cannot keep the terminal blank. `start_server` also passes the configuration directory of `mu client`, so the server that it starts reads the same worlds.

Enforced by: `TestBackend` tests of the first view before `Msg::Listed`, of a send before it, and of a failed first list. The spec test "The client takes the flag".

### D9. The architecture model

`architecture/model.c4` gains the part `registry` between `keymap` and `update`, and the edge `update -> registry`. The dynamic view of one key press in `views.c4` names the registry. The description of `keymap` and the label of the edge `keymap -> update` change from an `Action` to a registry entry. This follows the contributing page for diagrams: a change that alters the architecture updates the model.

Enforced by: review alone, and the LikeC4 build of the docs.

### D10. Room for completion

Completion of a command name or an argument, with a fuzzy list in the style of fzf, comes in a later change. This change keeps three seams for it:

- Each argument has a kind (`ArgKind`). A function `candidates(model, kind) -> Vec<String>` gives the values of a kind. For `World` it gives `Model::world_ids`, the worlds in the order of the top row, with no second list. Step 4 of D4 uses it, so the seam has a caller and a test from the start.
- The command parser returns a parse that knows where the cursor is: in the name, in the argument, or past it. A completion widget then asks the registry for names or `candidates` for the kind, with no second parser.
- The default keymap leaves Tab, Shift+Tab and Ctrl+Space with no binding. The guide page says that they are kept for completion.

The picker of `client-layout` and completion can share one fuzzy matcher. By the engram rule 5rk42v8qs1 that is a crate, for example `nucleo-matcher`, which a later change picks.

Enforced by: the unit test `world_candidates_follow_the_top_row`, and the test `default_keymap_leaves_tab_free`.

### D11. The resolver takes a list of top-level files

In `crates/mu-core/src/world/dir.rs`, `CLIENT_FILE` goes beside `SERVER_FILE`. `is_server_file` becomes one check over the list of top-level files that make no world. It keeps the case rule and the same-file rule for each name. So `client.toml` gets the same edge behavior as `server.toml`, with no second copy.

Enforced by: the tests of `world_config.rs` for `client.toml` in another case, unreadable, and as a hard link.

## Risks / Trade-offs

- [The editor follows the focus] A player who types half a pose and moves the focus finds an empty editor in the other world. → The text waits in the editor of the first world, and the focus key brings it back.
- [Up and Down now have two meanings] In a pose of several lines, an Up on the first line recalls history. A player can do that by mistake. → The draft comes back with Down, so nothing is lost.
- [`Action` loses `Copy`] Each match on `Action` that copies it now needs a clone. → The keymap holds an `EntryId` (D1), so only `build` makes an `Action`, once for each key.
- [The echo default changes to dim] A player who wants the old look sets `echo = "show"`. → The configuration page says so (#164).
- [History in memory only] A crash loses it. → History on disk is a later change, #165, beside #156.
- [Termina key names] Alt+digit, Alt+Home and Ctrl+Shift+Z depend on how each terminal reports them. → D3 lists the keys that do not arrive in Terminal.app by default, and the player binds other keys in `[keys]`.

## Migration Plan

No migration. No release exists, and `client.toml` is new. An existing `MU_CONFIG` directory with no `client.toml` gives the defaults.

## Open Questions

- The exact text of each description in the registry. It changes no spec and no task.
