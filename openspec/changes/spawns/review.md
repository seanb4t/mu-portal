---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-25T21:20:50+00:00
head: ba7585f62738e6f4f8727d98721222b2a96fdd2a
round: 1
next_id: 35
lenses:
  spec-lint: ok
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
  verify: not-applicable
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-1 | approach | blocker | in-change | The scenario places notices as an output of all worlds and expects a line that starts with `-- `. D9 adds the owner prefix to every push into a pane of Owner::All, so that line starts with an owner and `: `. | specs/client-layout/spec.md > Scenario: A place command with the scope — "the pane of `notices` shows a line that starts with `-- ` and contains `nowhere/nobody`" | Change the THEN of the scenario to the prefixed line, or state in D9 which lines of an Owner::All pane carry no prefix. |  | severity? should-fix 0.81 | unmet-requirement |
| R-2 | approach,invariants | blocker | in-change | app-5: The spec makes `client` the owner of every notice of the client in an output of all worlds. D9 gives a client notice the owner World(focus) whenever a world has the focus. So no decision produces the `client` prefix while a world has the focus.; inv-12: The spec says a client notice always has the owner client. D9 gives it the owner World(focus) when a world has the focus, so in an output of all worlds it gets the prefix of the world. The only test has no world. | specs/client-layout/spec.md > Requirement: A line in an output of all worlds names its owner — "The owner of a notice of the client SHALL be `client`." | app-5: Separate the world that picks the pane of a notice from the owner text of its prefix in D9, or change the requirement to name the world in focus.; inv-12: Separate the pane owner from the prefix in D9, or change the spec, and add a client test with a world in focus. |  | scope? deviates-design 0.65 | unmet-requirement |
| R-3 | approach | blocker | in-change | D9 gives every client notice the owner World(focus). The notice of a refused line must belong to the world of that line. After /focus beta/bryn, D9 puts the refusal of an alpha/aria line under beta/bryn, so `-- down` does not show after /focus alpha/aria. The code at head routes it by the world of the line. | specs/client-layout/spec.md > Scenario: A refused line of a world that lost the focus — "after the command `/focus alpha/aria`, the last line of the pane of `notices` is `-- down`" | State in D9 that the notice of a refused line has the owner World(world of the line). |  | severity? should-fix 0.62 | unmet-requirement |
| R-4 | approach | blocker | in-change | The requirement gives a notice for each stream of a line that no output shows. D9 shows the notice only when the set of panes is empty. A line in the streams pages and main under classic reaches main, so no notice names pages. | specs/client-layout/spec.md > Requirement: A stream with no output gets one notice — "A line can have a stream that no output of the layout of its world shows." | In D9, check each stream of the event against the selections, and give the notice for each stream that no output shows. |  | severity? should-fix 0.81 | unmet-requirement |
| R-5 | approach | blocker | in-change | With lines = 1, D5 starts a capture with a count of 1 and checks the limit only when a later line comes. So the second line also gets the streams of the capture, and the capture holds 2 lines. The spec caps it at lines lines, the first included, and allows lines = 1. | design.md > D5. One function decides the streams — "A rule with `until` starts a capture with a count of 1." | In D5, start no capture when lines is 1, or check the count right after the start line. |  | severity? should-fix 0.68 | unmet-requirement |
| R-6 | approach,invariants | blocker | in-change | app-9: Route carries at most one error, but one line can give two. A slow rule hits the backtrack limit, then a later rule matches with a bad name. The spec wants a script error from each rule. One of them is lost, and the rule is then marked as reported.; inv-4: Route carries one Option<String> error, but one line can give two errors: a slow match in one rule, then a bad name or a second slow match in a later rule. One error is lost, and that rule may be marked as reported. | design.md > D5. One function decides the streams — "Route { streams: Streams, error: Option<String> }" | app-9: Make the errors of Route a list, and record each as a script error event before the line.; inv-4: Make Route hold a list of errors, record each as a script error, and add a unit test with two slow rules on one line. |  | severity? should-fix 0.76 | unmet-requirement |
| R-7 | invariants | blocker | in-change | D9 removes Late::Waiting as a waiting-notice state, but Late::Waiting is the terminal probe's state for late replies. Removing it drops the guard that keeps a late graphics reply out of the input line. The waiting notices are the unshown list. | design.md > D9. The client routes by selection — "The waiting notices (`Late::Waiting` and the list of lines that wait) go away, because each layout has an output for `no" | Correct D9 and task 7.1 to remove the `unshown` list, UNSHOWN_MAX and show_unshown, and keep Late::Waiting and its probe tests. |  | scope? deviates-design 0.86 | wrong-behaviour |
| R-8 | test-coverage | blocker | in-change | Ten scenarios of the MODIFIED client-layout requirements (No tabs, Nine outputs, An unknown block, A block twice, A block that is not placed yet, No editor, A percent out of range, Too many lines, The default layout, The sidebar layout) are named by no task. | specs/client-layout/spec.md > Scenario: The sidebar layout — "row 1 shows `[alpha/aria]`, row 24 is the statusline, and a divider shows in column 56" | Add a task line that names the test for each: in crates/mu-cli/tests/layout.rs (unknown block, block twice, not placed, no editor, percent, too many lines, nine outputs, default layout), and in crates/mu-cli/tests/client.rs (no tabs, the sidebar layout, whose geometry changes with the new sidebar lines). If the tests already exist and stay valid, name them in a task that re-runs them. |  |  | unmet-requirement |
| R-9 | approach | should-fix | in-change | D9 names Late::Waiting as the waiting notices, but Late::Waiting is the wait for late terminal probe answers. The waiting notices are the field unshown with show_unshown and UNSHOWN_MAX. A removal that follows D9 breaks the handling of late probe answers. | design.md > D9. The client routes by selection — "The waiting notices (`Late::Waiting` and the list of lines that wait) go away" | In D9, name the field unshown, show_unshown and UNSHOWN_MAX as the waiting notices, and keep Late::Waiting. |  | severity? blocker 0.71 | wrong-behaviour |
| R-10 | approach,fit | should-fix | in-change | app-2: D8 cites Layout::check, and says each operation of change.rs already calls check. No such method exists. Every operation goes through Layout::from_steps. A new check method that from_steps does not call would leave /pane hide, /place, the mouse and arrange mode free to drop main or the notices.; fit-4: D8 puts the required-output check in Layout::check and says change.rs calls it, but no such function exists; the whole-plan checks live in from_steps, which each operation of change.rs calls. | design.md > D8. An output selects streams — "Each operation of `crates/mu-config/src/layout/change.rs` already calls `check`." | app-2: In D8 and in the table of what enforces each rule, name Layout::from_steps as the place of the check for main and notices.; fit-4: Name from_steps in D8 and tasks 3.2, and add the check for main and notices after its loop over BLOCKS, so a new Layout::check that change.rs never calls is not written. |  | severity? blocker 0.72 | unmet-requirement |
| R-11 | approach | should-fix | in-change | D2 says each compile error of a rule is a WorldError. No type of that name exists. The resolver reports with ConfigError. A new error type would not reach mu world list, mu world show, mu server and the config store with no new code, as D2 claims. | design.md > D2. Rules compile at the resolve — "each error is a `WorldError` with the `LayerOrigin` of the entry, the id and the field." | Name ConfigError::field with the LayerOrigin as the place in D2. |  | severity? blocker 0.65 | unmet-requirement |
| R-12 | fit | should-fix | deviates-design | D4 writes a second vte walk of game text in mu-core beside the one in mu-tui convert.rs, with another rule for control characters, so the text a pattern sees and the text the screen shows can differ. | design.md > D4. The pattern sees the text without color codes — "removes each escape sequence and each control character other than a tab, with the `vte` parser."; scope basis: design.md > D4. The pattern sees the text without color codes — "removes each escape sequence and each control character other than a tab, with the `vte` parser." | Name convert.rs in D4. Put one vte walk of game text in mu-config that yields the printable chars and the SGR calls, and call it from both convert and rules::plain, with one mu_config::ControlMap for the control characters. |  | scope? in-change 0.70 | drift |
| R-13 | fit | should-fix | deviates-design | D2 checks the fields of a rule in finish, after the merge, while each per-entry check runs per layer in check_layer; a rule that a later layer overrides by id is then never checked. D2 also names a WorldError type that does not exist. | design.md > D2. Rules compile at the resolve — "each error is a `WorldError` with the `LayerOrigin` of the entry, the id and the field"; scope basis: design.md > D2. Rules compile at the resolve — "`world/resolve.rs` calls `Rules::compile(&rules)` in `finish`." | Name check_layer and check_ids in D2. Check the fields of each rule with an action per layer in check.rs, beside check_ids, as a FieldError that names the id and the field; keep only the merged-order build of Rules in finish, and name ConfigError instead of WorldError. |  | scope? in-change 0.85; severity? blocker 0.76 | unmet-requirement |
| R-14 | fit | should-fix | deviates-design | ResolvedWorld would hold the rules twice, as rules and rules_compiled. The fake world of D11 is built by struct update in server.rs, so it can set one field and leave the other empty. | design.md > D2. Rules compile at the resolve — "`ResolvedWorld` gets `rules_compiled: Rules`, an `Arc` of the list, because the session and the CLI both hold it."; scope basis: design.md > D2. Rules compile at the resolve — "`ResolvedWorld` gets `rules_compiled: Rules`, an `Arc` of the list, because the session and the CLI both hold it." | Keep one rules field on ResolvedWorld: let Rules keep each Entry, so render in world.rs reads the entries from it, and let D11 hand a fake world one Rules value. |  |  | drift |
| R-15 | fit | should-fix | in-change | D9 keys panes by PaneKey with Owner::Client for no world, while editors stay keyed by (Option<String>, BlockId) with None for no world: two keys for one per-world block state. | design.md > D9. The client routes by selection — "each `output` block has its own pane, keyed by `PaneKey { owner: Owner, id: BlockId }`." | Name the editors key in D9 and key editors by the same PaneKey, or state why the editors keep Option<String>. |  |  | drift |
| R-16 | fit | should-fix | in-change | D1 adds the stream names main and notices in the new stream module without naming the existing MAIN and NOTICES_STREAM constants, so the names can be defined twice. | design.md > D1. A stream name is a block id, in mu-config — "`Streams::main()` is the list of the one name `main`." | State in D1 that the stream module owns the names main and notices, and that MAIN and NOTICES_STREAM move there or read from it. |  |  | drift |
| R-17 | invariants | should-fix | in-change | D8 puts the Selection on the Step, but a rewritten plan is built from tree::Block, which holds only kind, id and max. A move or hide that takes the rewrite path drops the scope and the patterns of every output, and no task gives the tree the Selection. | design.md > D8. An output selects streams — "`mu-config`: the step of an `output` gets a `Selection` with `scope` (`World` or `AllWorlds`) and `patterns`." | Give tree::Block the Selection and have write::step copy it into the Step. Name this in D8 and task 3.1, and add a unit test where a rewritten place keeps the showing of another output. |  | severity? blocker 0.68 | wrong-behaviour |
| R-18 | invariants | should-fix | in-change | A list of only exclusions is bad, but no test passes the empty list (showing then a place word or the end of the line) or a lone !. | specs/client-layout/spec.md > Requirement: An output shows the streams that its patterns match — "A list of only exclusions SHALL be bad." | Add parser tests in crates/mu-config/tests/layout.rs for `output as side showing right of main`, `output as side showing` and `output as side showing ! right of main`. |  |  | unmet-requirement |
| R-19 | invariants | should-fix | in-change | Streams guards 1 to 8 valid names on the server, but the client reads repeated string streams from the wire. No decision or task says what on_event does with an empty list or a name that is not valid. | design.md > D7. Each event has a ULID and its streams — "The protocol `Event` gets `string id = 7` and `repeated string streams = 8`." | Decide in D9 that the client parses the list into Streams and falls back to Streams::main() for an empty or bad list. Add a unit test in crates/mu-tui/tests/update.rs with an event of no streams. |  |  | wrong-behaviour |
| R-20 | invariants | should-fix | in-change | The design names any_layout_reads_back as the guard of writer and parser agreement on scope and patterns, but the test compares trees, and the tree holds no Selection. A writer that drops showing passes it. | design.md > What enforces each rule — "\| The writer and the parser agree on the scope and the patterns \| The property test `any_layout_reads_back` \|" | Make the property test compare written.lines() with layout.lines() or the steps, or put the Selection into the tree (inv-1) so tree equality covers it. Say which in task 3.1. |  |  | false-test |
| R-21 | invariants | should-fix | in-change | The spec allows one bad-name error and one slow-match error for each rule. D5 allows one error for each rule of any kind, so a slow match after a bad name of the same rule records no error. | design.md > D5. One function decides the streams — "A rule reports at most one error until `reset()`." | Either key the reported set by rule and kind of error, or change both requirements to one script error of any kind for each rule. Add a unit test with both kinds on one rule. |  |  | unmet-requirement |
| R-22 | invariants | should-fix | in-change | With lines = 1, D5 starts the capture with count 1 and checks the limit only when a capture runs. The next line is captured too, so the capture holds 2 lines where the spec allows 1. | design.md > D5. One function decides the streams — "If a capture runs, the line gets its streams. The capture counts the line. If the line matches `until`, or the count rea" | In D5, start no capture when lines is 1 (or check the count at the start), and add the unit test a_capture_of_one_line to task 4.3. |  |  | unmet-requirement |
| R-23 | invariants | should-fix | in-change | The range of lines is 1 to 1000, but the only load test passes 1001. No test passes 0, a negative value, a non-integer, or the accepted 1000. | specs/line-rules/spec.md > Requirement: The fields of a rule are checked at the load — "`lines` SHALL be an integer from 1 to 1000." | Add unit tests in crates/mu-core/tests/rules.rs for lines = 0, -1, "5" (refused) and 1000 (accepted). |  |  | unmet-requirement |
| R-24 | invariants | should-fix | in-change | A stream list holds 1 to 8 strings, but the only test passes 9. No test passes an empty list or an accepted list of 8. | specs/line-rules/spec.md > Requirement: The fields of a rule are checked at the load — "`stream` SHALL be a string or a list of 1 to 8 strings." | Add unit tests in crates/mu-core/tests/rules.rs for stream = [] (refused, names stream) and a list of 8 names (accepted). |  |  | unmet-requirement |
| R-25 | invariants | should-fix | in-change | A fixed stream entry is checked as a name at the load, while a template result is lower-cased first. No test fixes which side a fixed upper-case entry such as stream = "Chat" falls on. | specs/line-rules/spec.md > Requirement: The fields of a rule are checked at the load — "Each entry of `stream` with no `$` SHALL be a valid stream name." | State in the spec whether a fixed entry is lower-cased before the check, and add a test for stream = "Chat". |  |  | wrong-behaviour |
| R-26 | spec-lint | should-fix | in-change | The requirement 'The blocks of the screen' has no task line that names its subject: no task line holds 'blocks' or 'screen'. | specs/client-layout/spec.md > The blocks of the screen — "The blocks SHALL be `tabs`, `output`, `editor` and `statusline`." | Add a task line that names the block grammar and the scenarios of 'The blocks of the screen' (the_editor_alone, nine outputs, no tabs) with the test file that holds them. |  |  | unmet-requirement |
| R-27 | spec-lint | should-fix | in-change | The requirement 'Only lines from the game are matched' has no task line with two of its distinctive words (lines, game, matched). Only the test name 'the_line_that_the_player_sends' overlaps. | specs/line-rules/spec.md > Only lines from the game are matched — "The server SHALL run the rules on the lines from the game only." | Name the rule that the server runs on game lines only, with its test, in a task line of group 4 or 6, so the requirement shares two distinctive words with a task line. |  |  | unmet-requirement |
| R-28 | spec-lint | should-fix | in-change | The WHEN step 'the client shows a notice' names no concrete input or state, so the scenario does not say what triggers the notice. | specs/client-layout/spec.md > The main output is not the first output line — "a world sends the line `A line.`, and the client shows a notice" | Name the input that makes the notice, for example the user sends the line `hi` and the server refuses it with the reason `down`. |  |  | unmet-requirement |
| R-29 | approach | suggestion | in-change | The risk for the removal of the main-output role says the tasks name the removed tests. Task 7.1 names none. Tests of the removed rules exist, such as a_stream_with_no_output_goes_to_the_first. | design.md > Risks / Trade-offs — "Each removed requirement goes with its tests, and the tasks name them." | List the tests to remove in tasks 3.2 and 7.1, such as a_stream_with_no_output_goes_to_the_first and with_no_block_main_the_first_output_is_the_main_one. |  | severity? should-fix 0.78 |  |
| R-30 | approach | suggestion | in-change | The risk of notices shown twice says the guide tells the owner to fix a layout file. Task 7.5 lists the topics of layouts.md, and this fix is not one of them. | design.md > Risks / Trade-offs — "A layout file of the owner gets the same fix, and the guide says so." | Add to task 7.5 the advice to write `output as main showing main` in a layout that has its own output for notices. |  | severity? should-fix 0.79 |  |
| R-31 | fit | suggestion | in-change | D7 adds the ulid crate for event ids, but uuid is already a workspace dependency with a time-ordered version; D7 names neither uuid nor a reason to add a second id crate. | design.md > D7. Each event has a ULID and its streams — "the journal makes its id with the monotonic generator of the `ulid` crate." | Let task 1.4 compare uuid v7 with ulid, and name the choice and its reason in D7. |  |  |  |
| R-32 | fit | suggestion | deviates-design | Stream as its own HookAction variant beside Replace means one hook of #60 cannot rewrite a line and route it in one result, so #60 must add a variant or a struct. | design.md > D6. A hook can set the streams — "`HookAction` gets the variant `Stream(Streams)`."; scope basis: design.md > D6. A hook can set the streams — "`HookAction` gets the variant `Stream(Streams)`." | Let a hook result carry an optional text and optional streams together, or state in D6 that #60 runs one hook for each action. |  |  |  |
| R-33 | invariants | suggestion | in-change | D7 claims ULID order follows the order of the sequence numbers, even within one millisecond. That is a monotonicity claim over any count of appends, and ids_in_one_stream checks only the length and that the ids are unique. | design.md > D7. Each event has a ULID and its streams — "So two events in one millisecond still sort in their order." | Add a proptest (workspace dependency proptest, as in the proptest! block of crates/mu-core/src/world/resolve.rs) that appends N events to MemoryJournal and asserts the ids sort in the order of seq. |  |  |  |
| R-34 | invariants | suggestion | in-change | Selection::shows must not depend on the order of its patterns, over any list of patterns and any stream name. One example test, the_order_does_not_matter, covers this. | specs/client-layout/spec.md > Requirement: An output shows the streams that its patterns match — "The order of the patterns SHALL NOT matter." | Add a proptest in crates/mu-config/tests/stream.rs that shuffles a generated pattern list and asserts shows() is unchanged, after the proptest! block of crates/mu-config/tests/layout.rs. |  |  |  |

## Repeats

None.

## Out of bound

None.

## Accepted

None.

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | ba7585f62738e6f4f8727d98721222b2a96fdd2a | ebf029ee769ac3ccba980ec8da064ebdb7babab2e3d969c6fe364a8333f2bb31 | changes-requested | 34 | 0 | 0 | 0 | 0 |  |

## Judge

judge: 60 questions, 0.002 USD, stopped: cap of 60 questions

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A rule with a stream loads | 4.1 | covered |  |
| An unknown field | 4.1 | covered |  |
| A rule with no action | 5.1 | covered |  |
| An action with no pattern | 4.1 | covered |  |
| A pattern that does not compile | 4.1 | covered |  |
| A fixed stream that is not a name | 4.1 | covered |  |
| Too many streams | 4.1 | covered |  |
| Lines out of range | 4.1 | covered |  |
| A name that is a word of the sentences | 4.1 | covered |  |
| The earlier rule wins | 5.1 | covered |  |
| A character rule after a global rule | 5.1 | covered |  |
| No match | 5.1 | covered |  |
| A rule that names main | 5.1 | covered |  |
| A lookbehind and a named group | 5.1 | covered |  |
| A back-reference | 5.1 | covered |  |
| Case counts | 5.1 | covered |  |
| A colored line | 5.1 | covered |  |
| One stream for each channel | 5.1 | covered |  |
| A name from a capture that is not valid | 5.1 | covered |  |
| A line in two streams | 5.1 | covered |  |
| Two bad names | 6.2 | covered |  |
| The line that the player sends | 6.2 | covered |  |
| A WHO list | 5.1 | covered |  |
| A line that starts and could end | 5.1 | covered |  |
| Another rule during a capture | 5.1 | covered |  |
| A capture that never sees its end | 5.1 | covered |  |
| The default limit | 5.1 | covered |  |
| A drop during a capture | 6.2 | covered |  |
| A pattern that backtracks too much | 6.2 | covered |  |
| An unknown world | 5.1 | covered |  |
| No rules | 5.1 | covered |  |
| A bad name on stderr | 5.1 | covered |  |
| A fake world with a rule | 6.3 | covered |  |
| A fake world with no rules | 6.3 | covered |  |
| Ids in one stream | 6.1 | covered |  |
| Ids after a restart | 6.1 | covered |  |
| A live event and a replay | 6.4 | covered |  |
| No event without a stream | 6.1 | covered |  |
| No tabs |  | missing |  |
| The editor alone | 3.3 | covered |  |
| Nine outputs |  | missing |  |
| One output shows the notices too | 7.2 | covered |  |
| An output for the notices | 7.2 | covered |  |
| An output with no stream yet | 7.2 | covered |  |
| The main output is not the first output line | 7.2 | covered |  |
| No main output | 3.3 | covered |  |
| A refused line goes to the notices | 7.2 | covered |  |
| A refused line of a world that lost the focus | 7.2 | covered |  |
| A line with no world goes to the notices | 7.2 | covered |  |
| A notice shows under the world in focus only | 7.2 | covered |  |
| An output that comes later starts with new lines | 7.2 | covered |  |
| An unknown block |  | missing |  |
| A block twice |  | missing |  |
| A block that is not placed yet |  | missing |  |
| No editor |  | missing |  |
| No output for the notices | 3.3 | covered |  |
| A percent out of range |  | missing |  |
| Too many lines |  | missing |  |
| The default layout |  | missing |  |
| The sidebar layout |  | missing |  |
| The sidebar keeps notices out of main | 7.2 | covered |  |
| One output for all channels | 7.2 | covered |  |
| An exclusion | 7.2 | covered |  |
| Only exclusions | 3.3 | covered |  |
| A reserved pattern | 3.3 | covered |  |
| A plan keeps the patterns | 3.3 | covered |  |
| Two outputs show main | 7.2 | covered |  |
| A line in two streams of one output | 7.2 | covered |  |
| A channel with no output | 7.2 | covered |  |
| Hide the main output | 7.2 | covered |  |
| Hide the only output of the notices | 7.2 | covered |  |
| Hide a second output of the notices | 7.2 | covered |  |
| A page from another world | 7.3 | covered |  |
| Patterns of all worlds | 3.3 | covered |  |
| The main output of all worlds | 3.3 | covered |  |
| The id all | 3.3 | covered |  |
| The scope after the place | 3.3 | covered |  |
| A place command with the scope | 7.3 | covered |  |
| Two worlds in one pane | 7.3 | covered |  |
| A notice in an output of all worlds | 7.3 | covered |  |
| A world with another layout | 7.3 | covered |  |

### Not run

Plan phase: no verification runs.

## Verification runs

None.

## Dropped

None.

## Lens notes

- approach: ResolvedWorld derives Clone and PartialEq (crates/mu-core/src/world/mod.rs:388-399). D2 adds rules_compiled, an Arc of compiled fancy-regex patterns. PartialEq of the compiled type is not verified here, and D2 does not say how equality works.
- approach: default_layout in crates/mu-tui/src/model.rs:859 falls back to the plan ["editor"] under unreachable!. Under D8 that plan has no main and is bad, so the text of the fallback becomes false.
- approach: The design context says that the tests in world_config.rs use rules with only an id. world_config.rs:1019 also has rules with a pattern and no action. D2 skips those too, so the claim has no effect.
- approach: fancy-regex 0.11.0 is already in Cargo.lock as a transitive dependency. Task 1.2 reads the current release, which may differ.
- fit: D3 (fancy-regex), D10 (mu world match in world.rs beside list and show) and the glob crate choice of D8 fit the repository patterns and the library rows of openspec/config.yaml.
- fit: D5 adds a once-per-rule throttle of script errors beside the once-per-second throttle rate_drop in crates/mu-core/src/session.rs:551; the two throttle different sources, so no finding.
- fit: D8 lists the words that end a pattern list (above, below, left, right, with, at). The parser already holds them in side and place of crates/mu-config/src/layout/parse.rs:88-126; ending the list with a lookahead on place keeps one source.
- invariants: I1: A stream name is 1 to 32 chars from a-z, 0-9 and -, a letter first, not a word \| enforced at BlockId::new (crates/mu-config/src/layout/step.rs:26), which StreamName wraps \| boundary test an_id_of_33_characters and the 32-character id test in crates/mu-config/tests/layout.rs:318-324
- invariants: I2: Streams always hold 1 to 8 names with none twice \| enforced at the planned type Streams (D1) \| boundary test streams_hold_one_to_eight (planned)
- invariants: I3: lines is from 1 to 1000 \| enforced at Rules::compile (planned, D2) \| boundary test lines_out_of_range passes 1001 only; see inv-7
- invariants: I4: A capture holds at most lines lines, the first included \| enforced at RuleState::route (planned, D5), which misses lines = 1 \| boundary test a_capture_ends_at_its_limit with lines = 2 only; see inv-6
- invariants: I5: One script error for each rule until a connect \| enforced at RuleState reported set and reset() (planned, D5); Route has one error slot \| boundary test two_bad_names; see inv-4 and inv-5
- invariants: I6: Rule ids are unique within a world \| enforced at accumulate in crates/mu-core/src/world/resolve.rs:251, where the same id replaces \| boundary test the existing world_config tests
- invariants: I7: Every event has a ULID and 1 to 8 streams, also on a replay \| enforced at MemoryJournal append (planned, D7) and the Streams type \| boundary test no_event_without_a_stream and a_live_event_and_a_replay (planned); client decoding has none (inv-11)
- invariants: I8: ULID order follows seq order \| enforced at the monotonic generator under the journal lock (planned, D7) \| boundary test none; see inv-13
- invariants: I9: Every layout holds an output main and an output that shows notices \| enforced at Layout::from_steps (crates/mu-config/src/layout/mod.rs:130). design.md names Layout::check, which does not exist; each operation of change.rs calls from_steps \| boundary test no_main_output, no_output_for_the_notices, hide_refuses_main (planned)
- invariants: I10: Writer and parser agree on scope and patterns \| enforced at write::plan, which builds from tree::Block with no Selection \| boundary test any_layout_reads_back compares trees only; see inv-1 and inv-2
- invariants: I11: A list of only exclusions is bad, and re: or / is refused \| enforced at the planned Selection parser (D8) \| boundary test only_exclusions_are_bad; the empty list is untested (inv-10)
- invariants: I12: A line shows once in each output that matches \| enforced at the planned set of PaneKey (D9) \| boundary test a_line_in_two_streams_of_one_output (planned)
- invariants: I13: Protocol Event fields 7 and 8 are free \| checked: crates/mu-proto/proto/mu/v1/portal.proto Event uses 1 to 6 and has no reserved line \| no runtime-rejects finding
- invariants: The plan phase has no enforcement script to probe, so no probe ran.
- invariants: The proptest generator grown() in crates/mu-config/tests/layout.rs:100 starts from a lone editor and ids b<n>. Under the new rule that every layout holds main and an output for notices, no grown step passes from_steps, and the final expect panics. Task 3.1 must seed main.
- invariants: A `$$` template always yields `$`, which no stream name holds, so `$$` can only give main and a script error.
- invariants: D7 has no decision for the overflow error of the ulid monotonic generator, although append returns an Event and cannot fail today.
- invariants: D6 does not say whether a line routed by a hook, or a gagged line, counts toward a running capture.
- spec-lint: design.md has no open-questions section, so no assumed-answer check applied.
- spec-lint: No BREAKING marker in proposal.md or in any delta spec, so no breaking-mismatch finding.
- spec-lint: Build targets mise run check and mise run docs:check exist in mise.toml, and crates/mu-cli/tests/workspace_boundary.rs exists, so no unknown-target finding.
- spec-lint: All scenarios of the four delta specs have WHEN or GIVEN and THEN lines, and every requirement has at least one scenario.
- spec-lint: Every task of tasks.md names a command, a test or a file, so no unverifiable-task finding.
- test-coverage: Tasks name each test and say it must pass. The one-test command is in openspec/config.yaml, so those rows are graded covered.
- test-coverage: The two REMOVED requirements have scenarios for the new refusal (No main output, Hide the main output), so no breaking-unproven finding.
- test-coverage gap: design.md not read; not needed for the matrix.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | fixed_after | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: A place command with the scope"],"the pane of `notices` shows a line that starts with `-- ` and contains `nowhere/nobody`"] |  |  |  |  | severity? should-fix 0.81 | The scenario places notices as an output of all worlds and expects a line that starts with `-- `. D9 adds the owner prefix to every push into a pane of Owner::All, so that line starts with an owner and `: `. |
| R-2 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: A line in an output of all worlds names its owner"],"The owner of a notice of the client SHALL be `client`."] |  |  |  |  | scope? deviates-design 0.65 | app-5: The spec makes `client` the owner of every notice of the client in an output of all worlds. D9 gives a client notice the owner World(focus) whenever a world has the focus. So no decision produces the `client` prefix while a world has the focus.; inv-12: The spec says a client notice always has the owner client. D9 gives it the owner World(focus) when a world has the focus, so in an output of all worlds it gets the prefix of the world. The only test has no world. |
| R-3 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: A refused line of a world that lost the focus"],"after the command `/focus alpha/aria`, the last line of the pane of `notices` is `-- down`"] |  |  |  |  | severity? should-fix 0.62 | D9 gives every client notice the owner World(focus). The notice of a refused line must belong to the world of that line. After /focus beta/bryn, D9 puts the refusal of an alpha/aria line under beta/bryn, so `-- down` does not show after /focus alpha/aria. The code at head routes it by the world of the line. |
| R-4 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: A stream with no output gets one notice"],"A line can have a stream that no output of the layout of its world shows."] |  |  |  |  | severity? should-fix 0.81 | The requirement gives a notice for each stream of a line that no output shows. D9 shows the notice only when the set of panes is empty. A line in the streams pages and main under classic reaches main, so no notice names pages. |
| R-5 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D5. One function decides the streams"],"A rule with `until` starts a capture with a count of 1."] |  |  |  |  | severity? should-fix 0.68 | With lines = 1, D5 starts a capture with a count of 1 and checks the limit only when a later line comes. So the second line also gets the streams of the capture, and the capture holds 2 lines. The spec caps it at lines lines, the first included, and allows lines = 1. |
| R-6 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D5. One function decides the streams"],"Route { streams: Streams, error: Option<String> }"] |  |  |  |  | severity? should-fix 0.76 | app-9: Route carries at most one error, but one line can give two. A slow rule hits the backtrack limit, then a later rule matches with a bad name. The spec wants a script error from each rule. One of them is lost, and the rule is then marked as reported.; inv-4: Route carries one Option<String> error, but one line can give two errors: a slow match in one rule, then a bad name or a second slow match in a later rule. One error is lost, and that rule may be marked as reported. |
| R-7 | 1 | 1 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D9. The client routes by selection"],"The waiting notices (`Late::Waiting` and the list of lines that wait) go away, because each layout has an output for `notices`."] |  |  |  |  | scope? deviates-design 0.86 | D9 removes Late::Waiting as a waiting-notice state, but Late::Waiting is the terminal probe's state for late replies. Removing it drops the guard that keeps a late graphics reply out of the input line. The waiting notices are the unshown list. |
| R-8 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: The sidebar layout"],"row 1 shows `[alpha/aria]`, row 24 is the statusline, and a divider shows in column 56"] |  |  |  |  |  | Ten scenarios of the MODIFIED client-layout requirements (No tabs, Nine outputs, An unknown block, A block twice, A block that is not placed yet, No editor, A percent out of range, Too many lines, The default layout, The sidebar layout) are named by no task. |
| R-9 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D9. The client routes by selection"],"The waiting notices (`Late::Waiting` and the list of lines that wait) go away"] |  |  |  |  | severity? blocker 0.71 | D9 names Late::Waiting as the waiting notices, but Late::Waiting is the wait for late terminal probe answers. The waiting notices are the field unshown with show_unshown and UNSHOWN_MAX. A removal that follows D9 breaks the handling of late probe answers. |
| R-10 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D8. An output selects streams"],"Each operation of `crates/mu-config/src/layout/change.rs` already calls `check`."] |  |  |  |  | severity? blocker 0.72 | app-2: D8 cites Layout::check, and says each operation of change.rs already calls check. No such method exists. Every operation goes through Layout::from_steps. A new check method that from_steps does not call would leave /pane hide, /place, the mouse and arrange mode free to drop main or the notices.; fit-4: D8 puts the required-output check in Layout::check and says change.rs calls it, but no such function exists; the whole-plan checks live in from_steps, which each operation of change.rs calls. |
| R-11 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D2. Rules compile at the resolve"],"each error is a `WorldError` with the `LayerOrigin` of the entry, the id and the field."] |  |  |  |  | severity? blocker 0.65 | D2 says each compile error of a rule is a WorldError. No type of that name exists. The resolver reports with ConfigError. A new error type would not reach mu world list, mu world show, mu server and the config store with no new code, as D2 claims. |
| R-12 | 1 | 1 | should-fix | deviates-design | drift | ["artifact",["design.md","D4. The pattern sees the text without color codes"],"removes each escape sequence and each control character other than a tab, with the `vte` parser."] |  |  |  |  | scope? in-change 0.70 | D4 writes a second vte walk of game text in mu-core beside the one in mu-tui convert.rs, with another rule for control characters, so the text a pattern sees and the text the screen shows can differ. |
| R-13 | 1 | 1 | should-fix | deviates-design | unmet-requirement | ["artifact",["design.md","D2. Rules compile at the resolve"],"each error is a `WorldError` with the `LayerOrigin` of the entry, the id and the field"] |  |  |  |  | scope? in-change 0.85; severity? blocker 0.76 | D2 checks the fields of a rule in finish, after the merge, while each per-entry check runs per layer in check_layer; a rule that a later layer overrides by id is then never checked. D2 also names a WorldError type that does not exist. |
| R-14 | 1 | 1 | should-fix | deviates-design | drift | ["artifact",["design.md","D2. Rules compile at the resolve"],"`ResolvedWorld` gets `rules_compiled: Rules`, an `Arc` of the list, because the session and the CLI both hold it."] |  |  |  |  |  | ResolvedWorld would hold the rules twice, as rules and rules_compiled. The fake world of D11 is built by struct update in server.rs, so it can set one field and leave the other empty. |
| R-15 | 1 | 1 | should-fix | in-change | drift | ["artifact",["design.md","D9. The client routes by selection"],"each `output` block has its own pane, keyed by `PaneKey { owner: Owner, id: BlockId }`."] |  |  |  |  |  | D9 keys panes by PaneKey with Owner::Client for no world, while editors stay keyed by (Option<String>, BlockId) with None for no world: two keys for one per-world block state. |
| R-16 | 1 | 1 | should-fix | in-change | drift | ["artifact",["design.md","D1. A stream name is a block id, in mu-config"],"`Streams::main()` is the list of the one name `main`."] |  |  |  |  |  | D1 adds the stream names main and notices in the new stream module without naming the existing MAIN and NOTICES_STREAM constants, so the names can be defined twice. |
| R-17 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. An output selects streams"],"`mu-config`: the step of an `output` gets a `Selection` with `scope` (`World` or `AllWorlds`) and `patterns`."] |  |  |  |  | severity? blocker 0.68 | D8 puts the Selection on the Step, but a rewritten plan is built from tree::Block, which holds only kind, id and max. A move or hide that takes the rewrite path drops the scope and the patterns of every output, and no task gives the tree the Selection. |
| R-18 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: An output shows the streams that its patterns match"],"A list of only exclusions SHALL be bad."] |  |  |  |  |  | A list of only exclusions is bad, but no test passes the empty list (showing then a place word or the end of the line) or a lone !. |
| R-19 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7. Each event has a ULID and its streams"],"The protocol `Event` gets `string id = 7` and `repeated string streams = 8`."] |  |  |  |  |  | Streams guards 1 to 8 valid names on the server, but the client reads repeated string streams from the wire. No decision or task says what on_event does with an empty list or a name that is not valid. |
| R-20 | 1 | 1 | should-fix | in-change | false-test | ["artifact",["design.md","What enforces each rule"],"\| The writer and the parser agree on the scope and the patterns \| The property test `any_layout_reads_back` \|"] |  |  |  |  |  | The design names any_layout_reads_back as the guard of writer and parser agreement on scope and patterns, but the test compares trees, and the tree holds no Selection. A writer that drops showing passes it. |
| R-21 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D5. One function decides the streams"],"A rule reports at most one error until `reset()`."] |  |  |  |  |  | The spec allows one bad-name error and one slow-match error for each rule. D5 allows one error for each rule of any kind, so a slow match after a bad name of the same rule records no error. |
| R-22 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D5. One function decides the streams"],"If a capture runs, the line gets its streams. The capture counts the line. If the line matches `until`, or the count reaches `lines`, the capture ends."] |  |  |  |  |  | With lines = 1, D5 starts the capture with count 1 and checks the limit only when a capture runs. The next line is captured too, so the capture holds 2 lines where the spec allows 1. |
| R-23 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/line-rules/spec.md","Requirement: The fields of a rule are checked at the load"],"`lines` SHALL be an integer from 1 to 1000."] |  |  |  |  |  | The range of lines is 1 to 1000, but the only load test passes 1001. No test passes 0, a negative value, a non-integer, or the accepted 1000. |
| R-24 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/line-rules/spec.md","Requirement: The fields of a rule are checked at the load"],"`stream` SHALL be a string or a list of 1 to 8 strings."] |  |  |  |  |  | A stream list holds 1 to 8 strings, but the only test passes 9. No test passes an empty list or an accepted list of 8. |
| R-25 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/line-rules/spec.md","Requirement: The fields of a rule are checked at the load"],"Each entry of `stream` with no `$` SHALL be a valid stream name."] |  |  |  |  |  | A fixed stream entry is checked as a name at the load, while a template result is lower-cased first. No test fixes which side a fixed upper-case entry such as stream = "Chat" falls on. |
| R-26 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","The blocks of the screen"],"The blocks SHALL be `tabs`, `output`, `editor` and `statusline`."] |  |  |  |  |  | The requirement 'The blocks of the screen' has no task line that names its subject: no task line holds 'blocks' or 'screen'. |
| R-27 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/line-rules/spec.md","Only lines from the game are matched"],"The server SHALL run the rules on the lines from the game only."] |  |  |  |  |  | The requirement 'Only lines from the game are matched' has no task line with two of its distinctive words (lines, game, matched). Only the test name 'the_line_that_the_player_sends' overlaps. |
| R-28 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","The main output is not the first output line"],"a world sends the line `A line.`, and the client shows a notice"] |  |  |  |  |  | The WHEN step 'the client shows a notice' names no concrete input or state, so the scenario does not say what triggers the notice. |
| R-29 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"Each removed requirement goes with its tests, and the tasks name them."] |  |  |  |  | severity? should-fix 0.78 | The risk for the removal of the main-output role says the tasks name the removed tests. Task 7.1 names none. Tests of the removed rules exist, such as a_stream_with_no_output_goes_to_the_first. |
| R-30 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"A layout file of the owner gets the same fix, and the guide says so."] |  |  |  |  | severity? should-fix 0.79 | The risk of notices shown twice says the guide tells the owner to fix a layout file. Task 7.5 lists the topics of layouts.md, and this fix is not one of them. |
| R-31 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D7. Each event has a ULID and its streams"],"the journal makes its id with the monotonic generator of the `ulid` crate."] |  |  |  |  |  | D7 adds the ulid crate for event ids, but uuid is already a workspace dependency with a time-ordered version; D7 names neither uuid nor a reason to add a second id crate. |
| R-32 | 1 | 1 | suggestion | deviates-design |  | ["artifact",["design.md","D6. A hook can set the streams"],"`HookAction` gets the variant `Stream(Streams)`."] |  |  |  |  |  | Stream as its own HookAction variant beside Replace means one hook of #60 cannot rewrite a line and route it in one result, so #60 must add a variant or a struct. |
| R-33 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D7. Each event has a ULID and its streams"],"So two events in one millisecond still sort in their order."] |  |  |  |  |  | D7 claims ULID order follows the order of the sequence numbers, even within one millisecond. That is a monotonicity claim over any count of appends, and ids_in_one_stream checks only the length and that the ids are unique. |
| R-34 | 1 | 1 | suggestion | in-change |  | ["artifact",["specs/client-layout/spec.md","Requirement: An output shows the streams that its patterns match"],"The order of the patterns SHALL NOT matter."] |  |  |  |  |  | Selection::shows must not depend on the order of its patterns, over any list of patterns and any stream name. One example test, the_order_does_not_matter, covers this. |
