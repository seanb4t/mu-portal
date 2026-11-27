# Design

## Context

See proposal.md for the motivation and the model. These facts shape the design:

- `ResolvedWorld` in `crates/mu-core/src/world/mod.rs` holds `rules: Vec<Entry>`. Each `Entry` is the whole TOML table with its `id` and its `LayerOrigin`. No code reads a field of a rule other than `id`. Tests in `crates/mu-cli/tests/world_config.rs` use rules with only an `id`.
- `check_layer` in `crates/mu-core/src/world/check.rs` checks each layer before the merge. `check_ids` is the check of the rule ids there. Errors are `FieldError` and `ConfigError` values from `crates/mu-core/src/world/error.rs`.
- The session of a world (`crates/mu-core/src/session.rs`) runs `run_hooks` on each `Inbound::Line` and then calls `record`. A hook returns `Pass`, `Replace(String)` or `Gag` (`crates/mu-core/src/script.rs`).
- The core `Event` and `NewEvent` (`crates/mu-core/src/event.rs`) carry a required `world`. The protocol `Event` has the fields 1 to 6.
- `crates/mu-tui/src/model.rs` routes a line through `Layout::route` and `pane_in`. The main output uses `world.pane`, and each other output uses `extra_panes`, keyed by `(Option<world>, block id)`. Editors use `editors`, keyed the same way. `main_output()` in `crates/mu-config/src/layout/mod.rs` picks the stand-in for `main`.
- The field `unshown` of the model keeps the notices that no pane showed, up to `UNSHOWN_MAX`. `show_unshown` shows them later. `Late::Waiting` is a different thing: the wait for late answers of the terminal probe.
- `Layout::from_steps` in `crates/mu-config/src/layout/mod.rs` makes the checks of a whole layout. Each operation of `crates/mu-config/src/layout/change.rs` builds its result through it. A rewritten plan is built from `tree::Block`, which holds `kind`, `id` and `max`, by `step` in `write.rs`.
- `crates/mu-tui/src/convert.rs` walks game text with the `vte` parser. It maps control characters with `mu_config::ControlMap` from `crates/mu-config/src/filter.rs`.
- The names `main` and `notices` exist as `MAIN` in `crates/mu-config/src/layout/block.rs` and `NOTICES_STREAM` in `crates/mu-config/src/layout/mod.rs`.
- `mu-core` can depend on `mu-config`. `mu-tui` can depend on `mu-config` but not on `mu-core`. `crates/mu-cli/tests/workspace_boundary.rs` also lists the third-party crates of each member.
- A layout travels as its lines of text (`NamedLayout.lines`), so a change to the grammar needs no change to the protocol.
- A fake world is a `ResolvedWorld` that `crates/mu-cli/src/server.rs` builds with no layer.
- `uuid` is a workspace dependency with the `v4` feature only.

## Goals / Non-Goals

**Goals:**

- One function decides the streams of a line. The session, `mu world match` and the tests call it.
- The server decides the streams, and the journal keeps them. A client only selects and draws.
- One type for a stream name, shared by the server, the client and the layout grammar.
- One walk of game text, shared by the client and the rules.
- #59 adds actions to the same rule, and #60 routes through the same hook result, with no new seam.

**Non-Goals:**

- More than one action, `priority` or `fallthrough`.
- An owner of an event other than a world.
- Panes that merge several stores.

## Decisions

### D1. A stream name is a block id, in mu-config

`mu-config` gets the module `stream` with `StreamName` and `Streams`. It owns the names `main` and `notices` as `stream::MAIN` and `stream::NOTICES`. `block::MAIN` and `layout::NOTICES_STREAM` go away, and their users read the names from `stream`.

`BlockId::new` turns each upper-case letter to lower case before its check. `StreamName` wraps `BlockId::new`. So a stream name and an output id have one rule, both are always lower case, and one place refuses the words of the sentences. `Streams` is a list of 1 to 8 names with no name twice, in the order of the rule. `Streams::main()` is the list of the one name `main`. `Streams::from_wire(&[String])` drops a name that is in the list twice, before it counts the names. It returns `Streams::main()` for an empty list, a list of more than 8 names, or a bad name. The server, the client and the layout grammar all use these types.

### D2. Rules are checked per layer and compiled at the resolve

A new module `crates/mu-core/src/rules.rs` holds these types:

- `Rule`: a `Match` and a list of `Action`. `Rule::compile(id, &toml::Table)` returns `Ok(None)` for a table with no action field, so the rules with only an `id` keep their meaning. It takes the id and the table, because a layer holds its rules as tables, as `check_ids` reads them.
- `Action`: one variant, `Stream(Vec<Template>)`. #59 adds variants.
- `Rules`: the rules of one world in merged order. It keeps each `Entry`, and the compiled `Rule` of each entry that has an action.
- `RuleState`: the capture that runs, and the pairs of rule id and error kind that were reported.

`check_layer` calls `Rule::compile` on each rule table of the layer, beside `check_ids`. Each error is a `FieldError` with the id and the field, so it reaches the `ConfigError` of the layer with its `LayerOrigin`. Thus `mu world list`, `mu world show`, `mu server` and the configuration store report it with no new code. A rule that a later layer overrides by id is still checked in its own layer.

`finish` in `world/resolve.rs` builds `Rules` from the merged entries, and passes `entry.table` to `Rule::compile`. Each entry passed `check_layer`, so a compile error there is a bug, and `finish` returns it as a `ConfigError`. `ResolvedWorld.rules` becomes `Rules`, an `Arc` inside, because the session and the CLI both hold it. `render` in `world.rs` reads the entries from it. Thus one field holds the rules, and they cannot drift apart.

Alternative: compile in the session at the connect. Rejected, because a bad pattern then shows only at the connect, and not at the load.

### D3. fancy-regex is the engine

The owner asked for the full syntax of a regular expression: named groups, lookaround and back-references. The `regex` crate has no lookaround and no back-references. `fancy-regex` adds them, and hands each part that needs no backtracking to `regex`. Its license is MIT. So rule 5rk42v8qs1 (reuse first) points to it. `pcre2` was rejected, because it links a C library.

`fancy-regex` enters through `[workspace.dependencies]`, and the row of `mu-core` in the allowlist gets it. `RegexBuilder::backtrack_limit` sets the limit, and the default is 1,000,000 steps. A match at the limit returns `Error::RuntimeError(RuntimeError::BacktrackLimitExceeded)`. `Captures::expand` expands `$1`, `$name`, `${name}` and `$$`. A group that does not exist or did not match gives the empty string. The longest name wins, so `$1a` names the group `1a`, and a template writes `${1}a` for group 1.

### D4. One walk of game text

`mu-config` gets the module `ansi` with `walk(text: &str, map: &ControlMap, sink: &mut impl Sink)`. It runs the `vte` parser with a `Perform` that takes `print(char)` for printable text, `execute(u8)` for a control character, and `csi_dispatch` with the action `m` for an SGR sequence. It gives the sink each printable character after the `ControlMap`, and the parameters of each SGR sequence. It drops each other escape sequence. `vte` moves from the row of `mu-tui` to the row of `mu-config` in the allowlist.

The map `SCREEN` and its placeholder character move from `crates/mu-tui/src/text.rs` to `mu-config` as `ansi::SCREEN`. This map shows a tab as a space. `convert` in `crates/mu-tui/src/convert.rs` becomes a `Sink` that builds the styled text. `rules::plain(&str) -> Cow<str>` is a `Sink` that keeps only the characters. Both use `ansi::SCREEN`. A line with no escape byte and no control character returns borrowed. So the text that a pattern sees is the text that the screen shows, without its colors. The journal records the original text.

### D5. One function decides the streams

```text
RuleState::route(&mut self, rules: &Rules, text: &str) -> Route
Route { streams: Streams, errors: Vec<RuleError> }
RuleError { rule: String, kind: ErrorKind, text: String }
ErrorKind { BadName, SlowMatch }
```

- If a capture runs, the line gets its streams. The capture counts the line. If the line matches `until`, or the count reaches `lines`, the capture ends. `until` is never tried on the line that started the capture.
- If no capture runs, each rule tries `pattern` in order. The first match expands each template, turns it to lower case, and makes a `Streams`. If one name is not valid, or is `notices`, the line gets `Streams::main()` and a `BadName` error, and no capture starts. Only the client writes the stream `notices`, so the compile also refuses a fixed entry of `stream` that is `notices`, in any case. One function of `rules.rs` holds that rule for a fixed entry and for an expanded name. `RuleState::hooked` checks the streams that a hook set: streams that name `notices` give `Streams::main()` and one error until `reset()`. That error has its own flag, apart from the pairs of rule id and kind, so a rule with the id `hook` keeps its own reports.
- A rule with `until` starts a capture with a count of 1. If the count already reaches `lines`, which is the case for `lines = 1`, the capture ends at once. So a capture never holds more than `lines` lines.
- A match error at the backtrack limit counts as no match, and gives a `SlowMatch` error for the rule. For `pattern`, the next rule then runs, and it can give a second error for the same line. For `until`, the line stays in the capture, and the capture goes on.
- Each pair of rule id and error kind is reported at most one time until `reset()`. The session calls `reset()` at each connect and each disconnect, and `reset()` also ends a capture.
- No match gives `Streams::main()`.

The session calls `route` after the hooks on each `Inbound::Line`, and only there. A capture sees each game line, so `route` also runs on a line that a hook gagged or routed. It counts the line and tries `until` on the text after the hooks. The session masks the password of the world in that text before it calls `route`, so a pattern, a template and an error never hold the password. For a gagged line, that is the text that the gagging hook received. The errors of `route` are recorded also on a line whose streams a hook set. The streams that a hook set win for that line, and a gagged line makes no event. The session records each error of `route` as a `ScriptError` event, in order, before the line. Each other event gets `Streams::main()`.

### D6. A hook can rewrite and route a line in one result

`HookAction` becomes `Pass`, `Gag` or `Change { text: Option<String>, streams: Option<Streams> }`. `Change` with only `text` is the old `Replace`. `run_hooks` returns the text and the streams that the hooks set. If a hook on a game line set streams, those streams win over the rules for that line. The capture state of D5 still sees the line. `run_hooks` also serves the hooks on player lines. A player event always has the one stream `main`, so the session drops the streams of a `Change` from a player-line hook. No hook of this change sets streams. It is the seam for #60. A test hook in `crates/mu-core/tests/scripts.rs` proves that it works.

### D7. Each event has a ULID and its streams

The core `NewEvent` gets `streams: Streams`, and the core `Event` gets `id: Ulid` and `streams`. At the numbering of an event, the journal reads the clock once, and makes its id from that time with `ulid::Generator::generate_from_datetime()`, the monotonic generator of the `ulid` crate. The event takes the same time, so the time in the id is the time of the event. Two events in one millisecond still sort in their order. The protocol `Event` gets `string id = 7` and `repeated string streams = 8`. `mu-server` maps both. A ULID holds 80 random bits, so ids stay unique across restarts without a store.

The owner chose a ULID. Its text has 26 characters and sorts by time. `uuid` v7 was the alternative, because `uuid` is already a dependency. It was rejected, because it needs a new feature of `uuid` and its text has 36 characters. `generate()` returns `Err(Overflow)` when the random part of one millisecond runs out. The journal then calls `Overflow::commit_overflow_increment()`, which moves the id into the next millisecond, so the order holds. The unit test `an_id_after_a_full_millisecond_keeps_the_order` fills the random part with one bits to reach that branch. Its source of one bits implements `rand_core::TryRng`, so `rand_core` is a dev-dependency of `mu-core`, and the workspace boundary allows it.

### D8. An output selects streams

`mu-config`: `tree::Block` and the step of an `output` both get a `Selection` with `scope` (`World` or `AllWorlds`) and `patterns`. `step` in `write.rs` copies the `Selection` from the block into the step, so a rewritten plan keeps it. A `Pattern` is a glob and a flag for `!`. `Selection::shows(&StreamName) -> bool` is the one match rule of the spec. With no `showing`, the patterns are the id of the block, or `main` and `notices` for `main`.

The parser reads `of all worlds` and then `showing`, right after `as <id>`. The list of patterns ends where the existing `place` parser of `parse.rs` matches, at a comma and a size that end the line, or at the end of the line. Only the end at a comma and a size comes after at least one pattern, so a size after `showing` is not read as the end of an empty list. A place or the end of the line can come right after `showing`, and the empty list is then refused. So the comma of a size stays after the place, and a new place form needs no second list of words. A pattern holds only the characters of a stream name, `*`, `?` and a first `!`. The parser turns it to lower case. An empty list, a lone `!`, a list of only exclusions, and a pattern that starts with `re:` or `/` are refused. The writer of the plan writes the scope and the patterns back. If the patterns are the default, it writes no `showing`.

The glob match uses `wildmatch` (MIT). `WildMatch::new(pattern).matches(name)` takes `*` and `?`. It has no required dependency. `globset` was rejected, because it pulls `regex-automata`, `aho-corasick` and `bstr` for paths, and a stream name is not a path.

`Layout::from_steps` refuses a layout with no `main` or with no output that shows `notices`, after its loop over the blocks. It also refuses `main` with the scope `AllWorlds`, with the line number and the word `main`. Each operation of `change.rs` builds its result through `from_steps`. So `/pane hide`, `/place`, the mouse and arrange mode refuse such a result, with no new code in `mu-tui`. `main_output()` and `route()` go away.

A move keeps the selection of an output. `Change::place(text)` reads a `/place` sentence. It gives `Change::Move` for an `output` line with neither `of all worlds` nor `showing`, and `Change::Place` for any other line. `Layout::apply` gives `Change::Move` the `Selection` of the output with the same id on the screen, through `Layout::keeping`, and then places the step. Arrange mode sends `Change::Move` for a block on the screen, so one rule keeps the selection for the command, the keys and the mouse.

### D9. The client routes by selection

`mu-tui`: each `output` block has its own pane, keyed by `PaneKey { owner: Owner, id: BlockId }`. `Owner` is `World(String)`, `Client` or `All`. A block of scope `AllWorlds` uses `Owner::All`. The editors use the same `PaneKey`, with `Owner::Client` where they used `None`. `world.pane`, `PaneId::NOTICES` and the stand-in rules go away. So `PaneKey` with `Owner::Client` is the one name of a pane that belongs to no world. `focused_pane` stays for the scroll keys, the selection and Submit. It returns the pane of `main` for the owner `World(focus)`, or for `Owner::Client` when no world has the focus.

`default_layout` in `model.rs` falls back to the plan `output` and `editor below main`, because a plan of one editor is now bad.

On an event, the client reads its streams with `Streams::from_wire`. It takes the layout of the world of the event. For each stream, it finds the outputs whose `Selection` shows it. It pushes the line one time into each pane that one or more streams found. A prompt is the unfinished line of each such pane of the world of the event. A pane of all worlds is shared, so it shows no prompt. For each stream that found no output, it shows the notice of a stream with no output. That notice is a line of the stream `notices` of the world of the event, and the client does not keep it. The notice of a refused line is a kept notice of the world of that line, so it comes back after a Reset. It remembers the pair of stream and world, at most 256 pairs with the oldest forgotten first, and does not show that notice again for a pair it remembers. A Reset forgets every pair, so the replay of the journal shows the notice again.

A notice of the client is a line of the stream `notices` that belongs to a world. The notice of a refused line and the notice of a stream with no output belong to the world of the line. Each other notice belongs to the world in focus. A notice with no such world belongs to no world, and its pane owner is `Owner::Client`. The client routes a notice as a line of its world. `Model::notice` routes each new notice through this same routing.

Each kept notice keeps its owner and the set of `PaneKey` values of the panes that hold it. One rule fills panes from the kept notices: a replay routes each kept notice through the layout of its owner, and pushes it only into a pane that shows it and is not in its set. The push adds the pane to the set. A replay runs at two moments only. At the first focus, each kept notice of no world first becomes a notice of the world in focus. So a notice from before the first list of the worlds shows in the pane of the first world, as the terminal-client scenario "A notice before the first list" requires. After a Reset, the replay runs for every kept notice. A Reset clears only the panes that the replay of the journal fills again: the panes of each world and the panes of `Owner::All`. It removes those panes from the set of each kept notice. Any other focus change replays no notice, because the panes keep their lines.

In a pane of `Owner::All`, each push adds the dim prefix of the owner of the line as a styled span: the world id, or `client` for a notice that belongs to no world.

The waiting notices go away, because each layout has an output for `notices`: the field `unshown`, `UNSHOWN_MAX`, `show_unshown`, and the branch of `replay_notices` that fills `unshown`. `Late::Waiting` and its probe tests stay.

### D10. `mu world match`

`crates/mu-cli/src/world.rs` gets the verb `match`. It resolves the world as `mu world show` does, builds one `RuleState`, and calls `route` on each line of stdin. It splits stdin with `LineFramer`, the line rule of a session: it drops each CR and cuts a line at `MAX_LINE` bytes. A last line with no line break still counts. It prints the names of `Streams` with commas, a tab and the line, with `crate::print`. It prints each error on stderr with `crate::warn`, and exits 0. It needs no server, so it is the main test seam for D5.

### D11. A fake world takes the rules of global.toml

`Store::open` in `crates/mu-server/src/store.rs` resolves the global layer, with its `disable`, from the same start snapshot as the worlds. It returns the global `Rules` in `Opened`, beside the worlds. `server.rs` gives these `Rules` to each fake world. A fake world takes no other field. This is a test seam, and it lets the owner try a rule with `--fake-world`.

### What enforces each rule

| Rule | Enforced by |
|---|---|
| A bad rule fails the load, and names the file, the id and the field | `check_layer`, and tests of `mu world list` |
| A rule with no action is not run and not checked | `Rule::compile` returns `None`, the test `a_rule_with_no_action`, and the tests of `world-config` that stay green |
| The edges of `lines` and of the list of streams | Unit tests in `rules.rs` |
| The first match wins, the templates, lower case, the name check | Tests of `mu world match`, and unit tests of `Rules` |
| A capture, its limit, and its end at a disconnect | Tests of `mu world match`, and a test of the server with a fake world |
| One error of each kind for each rule until a connect | The set of reported pairs in `RuleState`, unit tests, and a test of the server with a fake world |
| Every event has a ULID and at least one stream, also on a replay | The type `Streams`, tests with one and two subscribers, and a property test of the order of ids |
| The client reads a bad list of streams as `main` | `Streams::from_wire`, and a unit test in `update.rs` |
| A hook can rewrite and route a line | A test hook in `scripts.rs` |
| `mu-core`, `mu-config` and `mu-tui` use only allowed crates | `workspace_boundary.rs` |
| The selection of an output, `!`, the defaults, lower case | Unit tests and a property test of `Selection` in `mu-config` |
| A layout holds `main` and an output for `notices` | `Layout::from_steps`, unit tests, tests of `mu layout show`, and tests of `/pane hide` in the client |
| A rewrite keeps the selection of each output | The `Selection` in `tree::Block`, and a unit test |
| The writer and the parser agree on the scope and the patterns | The property test `a_line_reads_back`, which writes a step with a scope and patterns as text and parses it back |
| A line shows one time in each output that matches it | Unit tests of the routing in `mu-tui`, and tests of `mu client` |
| No narrating comments, no sleeps in tests | `/sb:review-lint` and review |

## Risks / Trade-offs

- [A pattern with heavy backtracking slows the session of a world] → The backtrack limit stops each match. One script error names the rule. A pattern with no fancy feature runs on the linear `regex` engine.
- [A line in several streams shows in several panes, and the owner sees it twice] → That is the choice of the owner. A `!` pattern removes a stream from an output.
- [A layout from #172 can have an output for `notices` and a bare `output`. Then the notices show twice.] → The built-in `sidebar` writes `output as main showing main`. The guide tells the owner to make the same change to a layout file, and task 7.6 adds that advice.
- [The words `all`, `worlds` and `showing` can no longer be ids] → No release exists. A layout with such an id fails with the line number.
- [The shared walk of game text changes `convert.rs`] → The tests of `convert` stay green, and they cover the client side of the walk.
- [The removal of the main-output role touches code that #172 tested] → Tasks 3.2 and 7.1 name each test that goes with a removed requirement.

## Migration plan

None. The project has no release. A layout from before this change can have an output `main` and no new word as an id. Such a layout keeps its meaning, with one exception. If it has a bare `output` and an output for `notices`, the notices now show twice, because `main` shows them by default. The owner writes `output as main showing main` as its first line, as the guide says. An id in upper case now loads in lower case.
