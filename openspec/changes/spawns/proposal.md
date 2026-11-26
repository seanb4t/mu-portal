# Proposal

## Why

Every channel line, page and pose of a world arrives in one stream today. The owner cannot play an evening the way BeipMU allows: each channel in its own pane, pages in a pane for direct messages, and a WHO list away from the scene. `client-layout` (#189) built several outputs. But nothing sends a line of a game to a named stream, and nothing runs the `rules` array of `world-config`.

Tracked by #102. It is change 12 of the Phase 1 plan in #61.

## The model

1. **One log.** Each event of the journal has a ULID and one or more streams. The stream is `main`, written on the event, unless a rule sets another one.
2. **Rules label events on the server.** A rule is a match part and actions. This change builds the match part in full and one action, `stream`. #59 adds more actions to the same rules.
3. **Outputs show streams.** Each `output` of a layout shows the streams that its glob patterns match, for the world in focus or for all worlds. A line shows in each output that matches it.
4. **Nothing is implicit.** Each layout holds the output `main` and an output that shows `notices`. A stream that no output shows gets one notice, not a silent home.

## What Changes

- **Line rules.** A rule with an action runs on each line from the game, after the hooks and before the journal records the line. Its `pattern` has the full syntax of `fancy-regex`: named groups, lookaround and back-references. The first rule that matches applies.
- **Streams from captures.** `stream` is a template, or a list of up to 8 templates. `chan-$1` sends each channel to its own stream. `["pages", "main"]` sends a line to two streams.
- **Captures of several lines.** `until` ends a capture that `pattern` starts. `lines` caps it.
- **Events.** Each event gets the fields `id` (a ULID) and `streams`. The sequence number stays the order and the point of resume.
- **Outputs.** `output as comms showing chan-*, pages` shows the streams that its patterns match, and `!` excludes. With no `showing`, an output shows the stream of its own name. `main` shows `main` and `notices` by default.
- **Outputs of all worlds.** `output as chat of all worlds right of main` is one pane for every world. Each line starts with its owner.
- **Required outputs.** A layout without `main`, or without an output for `notices`, is bad. `/pane hide main` is refused. The "main output" role of `client-layout`, with its stand-ins and waiting lines, goes away.
- **`mu world match`** reads lines from stdin and prints the streams of each one.
- **A fake world takes the rules of `global.toml`**, so the tests and a trial with `--fake-world` can use rules.
- **The hook result can carry new text and streams together**, so a Lua hook of #60 can rewrite and route a line as a rule does.

## Capabilities

### New Capabilities

- `line-rules`: the match part, the `stream` action and the checks of a rule, and the order of the rules. Also names from captures, captures of several lines, and `mu world match`.

### Modified Capabilities

- `event-journal`: each event has a ULID and its streams.
- `client-layout`: outputs select streams with patterns and a world scope. A layout must hold `main` and an output for `notices`. The main-output role and the waiting notices are removed. The `sidebar` layout writes its `main` line in full.
- `world-sessions`: a fake world takes the rules of `global.toml`.

## Choices you will notice in use

| Choice | Value |
|---|---|
| A rule that runs | A rule with an action. A rule with only an `id`, or with fields of #59, does nothing yet and is not checked. |
| Fields | `id`, `pattern`, `until`, `lines` and `stream`. Another field in a rule with an action is an error that names the file, the id and the field. |
| Pattern syntax | `fancy-regex`. Case counts unless the pattern starts with `(?i)`. |
| What a pattern sees | The text of the line without color codes. The line keeps its colors on the screen. |
| Which lines | Lines from the game. Each other event has the stream `main`. |
| Order | The merged order of the rules, from global to character. The first match applies, and no later rule runs. This is the default of TinyFugue, so a later `fallthrough` does not change a rule written now. |
| `stream` | A string, or a list of 1 to 8 strings. `$1`, `$name`, `${name}` and `$$` take captures. Each name, fixed or from a template, is turned to lower case. |
| Stream names and ids | Always lower case. mu turns upper case to lower case before it checks a name, also in a layout. The rules of a block id: 1 to 32 characters from `a` to `z`, `0` to `9` and `-`, a letter first, not a word of the sentences. |
| A bad name from a capture | The line gets the stream `main`. The server records one bad-name error for the rule until the world connects again. |
| `until` and `lines` | A capture runs from the line that matches `pattern` to the line that matches `until`. `lines` defaults to 100, from 1 to 1000. A disconnect ends a capture. |
| A slow pattern | The backtrack limit of `fancy-regex` stops it. The line counts as no match, with one slow-match error for the rule until the world connects again. |
| Event id | A ULID of 26 characters, unique across restarts. |
| `showing` | Globs with `*` and `?`, and `!` to exclude. The order does not matter. A list of only exclusions is bad. Patterns that start with `re:` or `/` are kept for a later syntax. |
| Default streams of an output | Its own id. For `main`, `main` and `notices`. A `showing` replaces the default. |
| Overlap | A line shows in each output that matches one of its streams, and one time in each. |
| A stream with no output | One notice for each stream and world, for the world of the line. The client remembers at most 256 pairs, and a Reset forgets them. |
| Required outputs | `main`, and an output that shows `notices`. A layout file without them is bad. A command, a drag or arrange mode that removes them is refused. |
| `of all worlds` | Right after the id, before `showing`. Not on `main`. |
| The owner prefix | Dim, on each line of an output of all worlds. A notice has the prefix of its world, such as `alpha/aria: `. A notice that belongs to no world has `client: `. |
| New words of the sentences | `all`, `worlds` and `showing`. They cannot be ids or stream names. |
| `mu world match` output | For each line: the streams with commas, a tab, and the line. Script errors go to stderr. |

## Where mu differs from the convention of the field

- BeipMU makes a window for each new spawn title. In mu a layout places each output, and a glob such as `chan-*` covers the names that a rule makes.
- BeipMU keeps the case of a title. mu turns a stream name to lower case, because a stream name is also an id.
- BeipMU's Capture Until has no line limit. mu caps a capture at `lines`.
- Most clients let a line show in one window. In mu a line can be in several streams, and each output that matches shows it.

## Not in this change

- Highlights, gags, triggers, `priority` and `fallthrough` (#59).
- Lua access to rules and captures (#60). The hook result can carry streams, so #60 needs no new seam.
- Events with an owner that is not a world, such as the server. When the first one exists, an output of all worlds already gives it a home.
- A pane that appears by itself for a new stream name.
- A mark on a stack tab whose stream got a line while another tab showed.
- An `of all worlds` scope or a `showing` list from arrange mode. `/place` and the layout file can set both, and arrange mode keeps them.
- Rules on the lines that the player sends.
- A preference that picks another pattern syntax.

## Not tested

- The cost of real patterns on a busy game. The tests check the limit, not the speed.
- The channel formats of real TinyMUX, AresMUSH and Evennia games. The owner tries them in play.

## Impact

- `crates/mu-proto/proto/mu/v1/portal.proto`: `Event` gets `id = 7` and `streams = 8`.
- `crates/mu-core`: a `rules` module, the stream and the id of each event, and the capture state in the session of a world.
- `crates/mu-config`: the stream names, `showing`, `of all worlds`, the required `main` and `notices`, the `sidebar` layout, and one walk of game text with `vte` for the client and the rules.
- `crates/mu-tui`: routing by the streams of an event, the outputs of all worlds, and the notice for a stream with no output. The code of the main-output role and the waiting notices goes away. `convert.rs` uses the shared walk of game text.
- `crates/mu-cli`: the verb `mu world match`, and the rules of a fake world.
- New workspace dependencies: `fancy-regex`, `ulid` and a glob crate. `mu-config` also starts to use `vte`.
