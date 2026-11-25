---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-25T21:45:22+00:00
head: 942b5631e0ff1d7cfcb581de3c3eb62182d1e4fc
round: 3
next_id: 56
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
| R-41 | approach | blocker | in-change | The spec forbids `of all worlds` on the output `main`, and the scenario expects an error with `line 1` and `main`. No decision refuses it: D8 reads the scope after any id, and from_steps only checks that main exists. | specs/client-layout/spec.md > Requirement: An output can show the streams of all worlds — "The output `main` SHALL NOT take this form." | Add to D8 where `main` with the scope `AllWorlds` is refused (the parser or from_steps), with the line number and the word `main`. |  |  | unmet-requirement |
| R-42 | approach | blocker | in-change | D9 keeps the kept notices and replay_notices but gives no rule for a pane of Owner::All. At the first focus and after a Reset, replay_notices pushes each kept notice into the notices pane of the world in focus. For an output of all worlds that is the same pane that already shows it, so the notice shows twice. The kept list holds bare strings, so a replayed notice has no owner for its prefix. | design.md > D9. The client routes by selection — "the field `unshown`, `UNSHOWN_MAX`, `show_unshown`, and the branch of `replay_notices` that fills `unshown`." | State in D9 that the kept notices keep their owner, that replay skips a pane of Owner::All (or that a Reset clears it first), and name a unit test for a notice in a pane of all worlds before the first world. |  | severity? should-fix 0.69 | wrong-behaviour |
| R-29 | invariants | blocker | in-change | The removed requirements drop behavior that four existing tests rely on, and tasks 3.2 and 7.1 name none of the four. a_plan_with_no_output_has_no_main_output calls the removed main_output. Three app.rs tests use the layout of one editor, which the client now refuses. | crates/mu-config/tests/layout.rs:1360 — "assert_eq!(plan.main_output(), None);" | Name a_plan_with_no_output_has_no_main_output in task 3.2. Name a_line_and_a_notice_with_no_output_draw, waiting_lines_show_one_time_and_in_their_order and a_waiting_notice_shows_when_the_focus_moves_to_a_world_with_an_output in task 7.1, each as removed or rewritten. | regression |  | false-test |
| R-43 | fit | should-fix | in-change | D8 ends the list of patterns at a fixed list of place words. The parser already knows the first words of a place in its place parser, so the two lists can change apart when a place form is added. | design.md > D8. An output selects streams — "The words of a place are `above`, `below`, `left`, `right`, `with` and `at`." | In D8, end the list of patterns where the existing place parser matches (a winnow peek of place), and drop the fixed word list. |  |  | drift |
| R-44 | fit | should-fix | in-change | D9 routes each notice by its owner, but keeps replay_notices and the owner-less list notices. The replay puts every kept notice into pane_for(NOTICES_STREAM) of the focused world, a second notice route beside D9's. | design.md > D9. The client routes by selection — "and the branch of `replay_notices` that fills `unshown`. `Late::Waiting` and its probe tests stay." | In D9, keep the owner with each kept notice and make Model::notice and replay_notices route through the one owner routing of D9. |  |  | drift |
| R-45 | fit | should-fix | in-change | D9 keys the panes of no world by Owner::Client, but does not name PaneId::NOTICES, the fixed pane of no world that focused_pane and the Reset branch use. Two keys then name the pane of no world. | design.md > D9. The client routes by selection — "A notice with no such world belongs to no world, and its pane owner is `Owner::Client`." | In D9, remove PaneId::NOTICES and focused_pane with world.pane, and name the pane of no world only by PaneKey with Owner::Client. |  |  | drift |
| R-46 | fit | should-fix | in-change | D11 has server.rs resolve the global layer again, while Store::open already resolves every layer from the one start snapshot. The rules of a fake world can then come from another read of global.toml than the configured worlds. | design.md > D11. A fake world takes the rules of global.toml — "`server.rs` resolves the global layer of `--config` alone, with its `disable`, and gives its `Rules` to each fake world." | Resolve the global Rules in Store::open from the same snapshot, and return them in Opened beside worlds. |  |  | drift |
| R-47 | invariants | should-fix | in-change | The client falls back to a layout of one editor and guards it with unreachable!. The change makes that layout bad, and no task changes the fallback, so the fallback can now panic. | crates/mu-tui/src/model.rs:860 — ".unwrap_or_else(\|error\| unreachable!("the plan of one editor is good: {error}"))" | In default_layout, fall back to a plan that holds main, such as `output` and `editor below main`. Name this change in task 3.2 or 7.1, and add a unit test that parses the fallback plan. |  | severity? blocker 0.79 | wrong-behaviour |
| R-48 | invariants | should-fix | in-change | The only test of the accepted 32-character edge of an id parses a layout with no main. The change makes that layout bad, and no task rewrites the test. Stream names now share this limit, and no stream.rs test checks 32 or 33 characters. | crates/mu-config/tests/layout.rs:319 — "assert!(Layout::parse(&["editor", line.as_str()]).is_ok());" | In task 3.2, rewrite an_id_of_32_characters to place `output` and `editor below main` before the 32-character id. In task 2.1, add stream.rs tests for a stream name of 32 characters (accepted) and 33 characters (refused). |  |  | unmet-requirement |
| R-49 | invariants | should-fix | in-change | Streams holds 1 to 8 names with none repeated. Streams::from_wire is specified only for an empty list and for a bad name. No decision or test covers a wire list of 9 names or a list with a repeated name. | design.md > D1. A stream name is a block id, in mu-config — "`Streams::from_wire(&[String])` returns `Streams::main()` for an empty list or a bad name." | State in D1 what from_wire returns for 9 names and for a repeated name. Add the stream.rs tests streams_from_the_wire_with_nine_names and streams_from_the_wire_with_a_name_twice to task 2.1. |  |  | wrong-behaviour |
| R-50 | invariants | should-fix | in-change | The spec stops every match at the backtrack limit. D5 covers only a slow `pattern`. Nothing says what a slow `until` inside a capture does, whether it reports an error or whether the capture goes on, and no test passes one. | specs/line-rules/spec.md > Requirement: A slow match counts as no match — "If a match passes the backtrack limit of the engine, the match SHALL stop." | State in D5 that a slow `until` counts as no match: the line stays in the capture, with one SlowMatch error for the rule. Add a unit test a_slow_until_keeps_the_capture to task 4.4. |  |  | unmet-requirement |
| R-51 | invariants | should-fix | in-change | A rule with `until` whose start line gives a bad name sends that line to main. Nothing says whether a capture still starts. If one does, up to 100 later lines go to main and no other rule runs on them. No test covers this case. | design.md > D5. One function decides the streams — "If one name is not valid, the line gets `Streams::main()` and a `BadName` error." | State in D5 whether a start line with a bad name starts a capture. Add a unit test, such as a_bad_name_starts_no_capture, to task 4.4. |  |  | wrong-behaviour |
| R-52 | invariants | should-fix | in-change | A capture gives each later game line its streams and ends at the first line that matches `until`. D6 skips the rules on a line whose streams a hook set, and a gagged line never reaches route. So a hook of #60 on an `until` line leaves the capture running until its limit. | design.md > D6. A hook can rewrite and route a line in one result — "If a hook on a game line set streams, the session does not run the rules for that line." | State in D5 and D6 how the capture treats a line whose streams a hook set, and a line that a hook gagged: whether it counts the line and whether it tests `until`. Add a scripts.rs test with a hook during a capture to task 4.5. |  |  | next-change |
| R-53 | fit | suggestion | in-change | D2 calls Rule::compile(&Entry) from check_layer, but a layer holds its rules as toml::Table. An Entry exists only after accumulate in resolve.rs, so the per-layer check has no Entry to pass. | design.md > D2. Rules are checked per layer and compiled at the resolve — "`Rule::compile(&Entry)` returns `Ok(None)` for an entry with no action field" | Give Rule::compile the id and the toml::Table, as check_ids reads them, and let finish pass entry.table. |  | severity? blocker 0.60 |  |
| R-54 | fit | suggestion | in-change | D10 prints each rule error on stderr without naming crate::warn, the helper of mu-cli that writes a message of a verb to stderr through the REPORT map. | design.md > D10. `mu world match` — "It prints each error on stderr, and exits 0." | Name crate::warn in D10 for the errors, and crate::print for the lines on stdout. |  |  |  |
| R-55 | invariants | suggestion | in-change | D4 claims that a pattern sees the text the screen shows, without colors, for any line. Only example tests on each side check it. Nothing compares plain with the text of convert over arbitrary input. | design.md > D4. One walk of game text — "So the text that a pattern sees is the text that the screen shows, without its colors." | Add a proptest (workspace `proptest`, as in crates/mu-config/tests/filter.rs) that checks rules::plain(x) == convert(x).text for any string x with ESC and control bytes. Put it in crates/mu-cli/tests, which depends on both mu-core and mu-tui. |  | severity? should-fix 0.62 |  |

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
| 2 | plan | follow-up | 2466ac7447be4a80f8c44b9f7ec06d8e6914f6a6 | bd5f81b131a7420dfd4a456c13a54b9cbd09355372f53a3469dcd7615539d3f8 | approve-with-notes | 6 | 0 | 4 | 30 | 0 |  |
| 3 | plan | full | 942b5631e0ff1d7cfcb581de3c3eb62182d1e4fc | 98de8a29e698b7a5f4f33c0f980da09a923c76fd2e4370c371ca9b3185bea2a1 | changes-requested | 15 | 0 | 1 | 0 | 0 | the new scenarios of round 2 need coverage-matrix rows |

## Judge

judge: 32 questions, 0.001 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| No tabs | 7.3 | covered |  |
| The editor alone | 3.3 | covered |  |
| Nine outputs | 3.3 | covered |  |
| One output shows the notices too | 7.3 | covered |  |
| An output for the notices | 7.3 | covered |  |
| An output with no stream yet | 7.3 | covered |  |
| The main output is not the first output line | 7.3 | covered |  |
| No main output | 3.2, 3.3 | covered |  |
| A refused line goes to the notices | 7.3 | covered |  |
| A refused line of a world that lost the focus | 7.3 | covered |  |
| A line with no world goes to the notices | 7.3 | covered |  |
| A notice shows under the world in focus only | 7.3 | covered |  |
| An output that comes later starts with new lines | 7.3 | covered |  |
| An unknown block | 3.3 | covered |  |
| A block twice | 3.3 | covered |  |
| A block that is not placed yet | 3.3 | covered |  |
| No editor | 3.3 | covered |  |
| No output for the notices | 3.2, 3.3 | covered |  |
| A percent out of range | 3.3 | covered |  |
| Too many lines | 3.3 | covered |  |
| The default layout | 7.3 | covered |  |
| The sidebar layout | 7.3 | covered |  |
| The sidebar keeps notices out of main | 7.3 | covered |  |
| One output for all channels | 7.3 | covered |  |
| An exclusion | 7.3 | covered |  |
| Only exclusions | 3.3 | covered |  |
| An empty list of patterns | 3.3 | covered |  |
| A reserved pattern | 3.3 | covered |  |
| A plan keeps the patterns | 3.3 | covered |  |
| Two outputs show main | 7.3 | covered |  |
| A line in two streams of one output | 7.3 | covered |  |
| A channel with no output | 7.3 | covered |  |
| Hide the main output | 7.3 | covered |  |
| Hide the only output of the notices | 7.3 | covered |  |
| Hide a second output of the notices | 7.3 | covered |  |
| A page from another world | 7.4 | covered |  |
| Patterns of all worlds | 3.3 | covered |  |
| The main output of all worlds | 3.3 | covered |  |
| The id all | 3.3 | covered |  |
| The scope after the place | 3.3 | covered |  |
| A place command with the scope | 7.4 | covered |  |
| Two worlds in one pane | 7.4 | covered |  |
| A notice in an output of all worlds | 7.4 | covered |  |
| A world with another layout | 7.4 | covered |  |
| An id in upper case | 3.3 | covered |  |
| Ids in one stream | 6.1 | covered |  |
| Ids after a restart | 6.1 | covered |  |
| A live event and a replay | 6.4 | covered |  |
| No event without a stream | 6.1 | covered |  |
| A rule with a stream loads | 4.1 | covered |  |
| An unknown field | 4.1 | covered |  |
| A rule with no action | 5.1 | covered |  |
| An action with no pattern | 4.1 | covered |  |
| A pattern that does not compile | 4.1 | covered |  |
| A fixed stream that is not a name | 4.1 | covered |  |
| A fixed stream in upper case | 5.1 | covered |  |
| An empty list of streams | 4.1 | covered |  |
| Too many streams | 4.1 | covered |  |
| Lines out of range | 4.1 | covered |  |
| Lines of zero | 4.1 | covered |  |
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
| A capture of one line | 5.1 | covered |  |
| The default limit | 5.1 | covered |  |
| A drop during a capture | 6.2 | covered |  |
| A pattern that backtracks too much | 6.2 | covered |  |
| An unknown world | 5.1 | covered |  |
| No rules | 5.1 | covered |  |
| A bad name on stderr | 5.1 | covered |  |
| A fake world with a rule | 6.3 | covered |  |
| A fake world with no rules | 6.3 | covered |  |

### Not run

Plan phase: no verification runs.

## Verification runs

None.

## Dropped

None.

## Lens notes

- approach: D3 cites rule 5rk42v8qs1 (reuse first). The id is in no file of the repository other than change artifacts, so this lens could not check it; it may come from the review plugin.
- approach: D2 says the session holds the rules, but SessionParts in crates/mu-core/src/session.rs takes chosen fields of the world, not the ResolvedWorld. Task 6.2 must also pass the Rules into SessionParts in crates/mu-core/src/portal.rs.
- approach: Task 3.2 cites design.md D8 for the change to the sidebar layout, but only Risks / Trade-offs names it.
- approach: ResolvedWorld derives PartialEq, and a compiled fancy-regex Regex has no PartialEq, so Rules needs its own PartialEq, for example by its entries.
- fit: Cargo.lock already holds fancy-regex 0.11.0 through termwiz. A workspace pin of a newer release adds a second version of the crate to the build; D3 does not mention it.
- fit: The fixes of R-12 (one walk of game text) and R-32 (Change with text and streams) hold at head, so neither row is raised again.
- invariants: I1: A stream name and a block id are 1 to 32 chars of a-z, 0-9 and -, letter first, no sentence word, lower-cased first \| enforced at BlockId::new (crates/mu-config/src/layout/step.rs:26), which StreamName wraps (D1) \| boundary test an_id_of_33_characters; the accepted edge an_id_of_32_characters breaks (inv-2)
- invariants: I2: Streams holds 1 to 8 names, none twice \| enforced at the Streams constructor (D1) \| boundary test streams_hold_one_to_eight, a_list_of_eight_streams_loads, too_many_streams, an_empty_list_of_streams
- invariants: I3: lines is an integer from 1 to 1000 \| enforced at Rule::compile from check_layer (D2) \| boundary test lines_of_zero, lines_below_zero_is_refused, lines_as_a_string_is_refused, lines_of_1000_loads, lines_out_of_range
- invariants: I4: A capture holds at most lines lines, the first included \| enforced at RuleState::route (D5) \| boundary test a_capture_of_one_line, a_capture_ends_at_its_limit, the_default_limit; no test for a hook or a gag during a capture (inv-6)
- invariants: I5: The start line of a capture never ends it \| enforced at RuleState::route (D5) \| boundary test the_first_line_does_not_end_a_capture, a_line_that_starts_and_could_end
- invariants: I6: Each pair of rule and error kind is reported once until reset \| enforced at the reported set of RuleState (D5) \| boundary test a_rule_reports_each_kind_once, two_errors_on_one_line, reset_reports_a_bad_name_again
- invariants: I7: A slow match counts as no match \| enforced at RuleState::route (D5) \| boundary test a_slow_match_counts_as_no_match for pattern; none for until (inv-4)
- invariants: I8: Every event has 1 to 8 streams, live, replayed and on the wire \| enforced at NewEvent.streams: Streams (D7) and Streams::from_wire (D1) \| boundary test no_event_without_a_stream, streams_from_the_wire_with_no_name, streams_from_the_wire_with_a_bad_name; none for 9 or repeated names (inv-3)
- invariants: I9: Event ids are unique and sort in the order of seq \| enforced at the ulid monotonic generator at numbering, inside the MemoryJournal mutex (crates/mu-core/src/journal.rs:148) \| boundary test ids_in_one_stream, ids_after_a_restart, property ids_sort_in_the_order_of_seq. Journal::append is infallible and D7 does not say what it does on the overflow error of the generator; overflow needs 2^80 ids in one millisecond, so no finding
- invariants: I10: A layout always holds main and an output that shows notices \| enforced at Layout::from_steps; every operation of change.rs (insert, replace, delete, place, hide through settle, resize, fill) builds through it \| boundary test no_main_output, no_output_for_the_notices, hide_refuses_main, hide_refuses_the_last_output_of_the_notices, hide_a_second_output_of_the_notices
- invariants: I11: The selection of an output survives a rewrite and arrange mode \| enforced at tree::Block and step in write.rs (D8); arrange_step (crates/mu-tui/src/arrange.rs:273) builds a Step from kind and id alone and must copy it \| boundary test a_rewrite_keeps_the_selection, a_move_keeps_the_selection
- invariants: I12: The writer and the parser agree on the scope and the patterns \| enforced at the writer of D8 \| boundary test property any_layout_reads_back with Selection in its trees
- invariants: I13: Selection::shows does not depend on the order of the patterns \| enforced at Selection::shows \| boundary test property any_order_of_the_patterns_shows_the_same; an empty list, a lone ! and only exclusions have tests
- invariants: I14: A pattern sees the screen text without colors \| enforced at ansi::walk with ansi::SCREEN shared by convert and plain (D4) \| boundary test plain_* and the convert tests; no cross check (inv-7)
- invariants: I15: The fallback layout of the client is valid \| enforced at none after the change \| boundary test none (inv-1)
- invariants: I16: A line shows once in each output, and one notice per stream and world \| enforced at the routing of D9 \| boundary test a_line_shows_once_in_one_output, a_line_in_two_streams_of_one_output, a_stream_with_no_output_gives_one_notice
- invariants: I17: Event fields 7 and 8 are free in the protocol \| enforced at portal.proto Event, which has fields 1 to 6 and no reserved line \| boundary test not applicable
- invariants: Probe: a probe with Python str.lower showed that Unicode lower-casing turns the Kelvin sign U+212A into an ASCII k, so a capture like CHAN-<U+212A> becomes the valid name chan-k. The spec text (turn each upper-case letter to lower case) allows this. D1 does not choose between to_lowercase and to_ascii_lowercase. The probe script is inv-probe/lower.py under the handoff directory
- invariants: Merge of rules replaces an overridden entry in its old position (crates/mu-core/src/world/resolve.rs accumulate), so the merged order keeps the place of the first layer that named the id
- spec-lint: Scenario shape: all 4 delta specs read in full. Every requirement has at least one scenario, and every scenario has a GIVEN or WHEN line and a THEN line.
- spec-lint: Untasked check: each requirement heading was matched against tasks.md. Each has a task line with two or more distinctive words or a test name in the same line.
- spec-lint: Verifiability: each THEN step names a status, a file, a stdout or stderr line, a pane, a field or a count. No THEN restates its WHEN, and no THEN names only a quality. Quality words (works, correct, fast, stable, properly) do not occur in the specs.
- spec-lint: WHEN and GIVEN steps name concrete inputs (commands, stdin lines, files, fake worlds).
- spec-lint: BREAKING: no BREAKING marker in proposal.md or any delta spec, so no mismatch.
- spec-lint: Build targets: mise.toml defines tasks check and docs:check; Cargo workspace members are crates/*, and mu-server and mu-tui exist; crates/mu-cli/tests/workspace_boundary.rs exists; the one-test command in openspec/config.yaml matches the one used in tasks.md.
- spec-lint: Open questions: no open-question section and no TBD or undecided marker in proposal.md or design.md, so no assumed-answer finding.
- spec-lint: Prior rows: R-12, R-29, R-30 and R-32 are still disposition fixed. Their defect is not present at head: design D4 has one walk in mu-config, task 7.1 names the removed tests, task 7.6 carries the owner advice, and D6 has the Change variant. R-35 to R-40 concern matrix rows of review.md, which is outside this lens. No prior row is raised again.
- test-coverage: Most tasks name tests and say 'make sure ... pass' with no literal command; the header of tasks.md points to the one-test command in openspec/config.yaml, so rows are graded covered on that reference.
- test-coverage: No BREAKING requirement in the delta specs.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | fixed_after | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: A place command with the scope"],"the pane of `notices` shows a line that starts with `-- ` and contains `nowhere/nobody`"] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | severity? should-fix 0.81 | The scenario places notices as an output of all worlds and expects a line that starts with `-- `. D9 adds the owner prefix to every push into a pane of Owner::All, so that line starts with an owner and `: `. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: A line in an output of all worlds names its owner"],"The owner of a notice of the client SHALL be `client`."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | scope? deviates-design 0.65 | app-5: The spec makes `client` the owner of every notice of the client in an output of all worlds. D9 gives a client notice the owner World(focus) whenever a world has the focus. So no decision produces the `client` prefix while a world has the focus.; inv-12: The spec says a client notice always has the owner client. D9 gives it the owner World(focus) when a world has the focus, so in an output of all worlds it gets the prefix of the world. The only test has no world. |
| R-3 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: A refused line of a world that lost the focus"],"after the command `/focus alpha/aria`, the last line of the pane of `notices` is `-- down`"] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | severity? should-fix 0.62 | D9 gives every client notice the owner World(focus). The notice of a refused line must belong to the world of that line. After /focus beta/bryn, D9 puts the refusal of an alpha/aria line under beta/bryn, so `-- down` does not show after /focus alpha/aria. The code at head routes it by the world of the line. |
| R-4 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: A stream with no output gets one notice"],"A line can have a stream that no output of the layout of its world shows."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | severity? should-fix 0.81 | The requirement gives a notice for each stream of a line that no output shows. D9 shows the notice only when the set of panes is empty. A line in the streams pages and main under classic reaches main, so no notice names pages. |
| R-5 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D5. One function decides the streams"],"A rule with `until` starts a capture with a count of 1."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | severity? should-fix 0.68 | With lines = 1, D5 starts a capture with a count of 1 and checks the limit only when a later line comes. So the second line also gets the streams of the capture, and the capture holds 2 lines. The spec caps it at lines lines, the first included, and allows lines = 1. |
| R-6 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D5. One function decides the streams"],"Route { streams: Streams, error: Option<String> }"] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | severity? should-fix 0.76 | app-9: Route carries at most one error, but one line can give two. A slow rule hits the backtrack limit, then a later rule matches with a bad name. The spec wants a script error from each rule. One of them is lost, and the rule is then marked as reported.; inv-4: Route carries one Option<String> error, but one line can give two errors: a slow match in one rule, then a bad name or a second slow match in a later rule. One error is lost, and that rule may be marked as reported. |
| R-7 | 1 | 2 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D9. The client routes by selection"],"The waiting notices (`Late::Waiting` and the list of lines that wait) go away, because each layout has an output for `notices`."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | scope? deviates-design 0.86 | D9 removes Late::Waiting as a waiting-notice state, but Late::Waiting is the terminal probe's state for late replies. Removing it drops the guard that keeps a late graphics reply out of the input line. The waiting notices are the unshown list. |
| R-8 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Scenario: The sidebar layout"],"row 1 shows `[alpha/aria]`, row 24 is the statusline, and a divider shows in column 56"] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | Ten scenarios of the MODIFIED client-layout requirements (No tabs, Nine outputs, An unknown block, A block twice, A block that is not placed yet, No editor, A percent out of range, Too many lines, The default layout, The sidebar layout) are named by no task. |
| R-9 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D9. The client routes by selection"],"The waiting notices (`Late::Waiting` and the list of lines that wait) go away"] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | severity? blocker 0.71 | D9 names Late::Waiting as the waiting notices, but Late::Waiting is the wait for late terminal probe answers. The waiting notices are the field unshown with show_unshown and UNSHOWN_MAX. A removal that follows D9 breaks the handling of late probe answers. |
| R-10 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D8. An output selects streams"],"Each operation of `crates/mu-config/src/layout/change.rs` already calls `check`."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | severity? blocker 0.72 | app-2: D8 cites Layout::check, and says each operation of change.rs already calls check. No such method exists. Every operation goes through Layout::from_steps. A new check method that from_steps does not call would leave /pane hide, /place, the mouse and arrange mode free to drop main or the notices.; fit-4: D8 puts the required-output check in Layout::check and says change.rs calls it, but no such function exists; the whole-plan checks live in from_steps, which each operation of change.rs calls. |
| R-11 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D2. Rules compile at the resolve"],"each error is a `WorldError` with the `LayerOrigin` of the entry, the id and the field."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | severity? blocker 0.65 | D2 says each compile error of a rule is a WorldError. No type of that name exists. The resolver reports with ConfigError. A new error type would not reach mu world list, mu world show, mu server and the config store with no new code, as D2 claims. |
| R-12 | 1 | 2 | should-fix | deviates-design | wrong-behaviour | ["artifact",["design.md","D4. The pattern sees the text without color codes"],"removes each escape sequence and each control character other than a tab, with the `vte` parser."] | fixed | bb038add4871d29dbe937cced84f53ca0e1c296d | 2 |  | scope? in-change 0.85 | D4 writes a second vte walk of game text in mu-core beside the one in mu-tui convert.rs, with another rule for control characters, so the text a pattern sees and the text the screen shows can differ. |
| R-13 | 1 | 2 | should-fix | deviates-design | unmet-requirement | ["artifact",["design.md","D2. Rules compile at the resolve"],"each error is a `WorldError` with the `LayerOrigin` of the entry, the id and the field"] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | scope? in-change 0.85; severity? blocker 0.76 | D2 checks the fields of a rule in finish, after the merge, while each per-entry check runs per layer in check_layer; a rule that a later layer overrides by id is then never checked. D2 also names a WorldError type that does not exist. |
| R-14 | 1 | 2 | should-fix | deviates-design | drift | ["artifact",["design.md","D2. Rules compile at the resolve"],"`ResolvedWorld` gets `rules_compiled: Rules`, an `Arc` of the list, because the session and the CLI both hold it."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | ResolvedWorld would hold the rules twice, as rules and rules_compiled. The fake world of D11 is built by struct update in server.rs, so it can set one field and leave the other empty. |
| R-15 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","D9. The client routes by selection"],"each `output` block has its own pane, keyed by `PaneKey { owner: Owner, id: BlockId }`."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | D9 keys panes by PaneKey with Owner::Client for no world, while editors stay keyed by (Option<String>, BlockId) with None for no world: two keys for one per-world block state. |
| R-16 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","D1. A stream name is a block id, in mu-config"],"`Streams::main()` is the list of the one name `main`."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | D1 adds the stream names main and notices in the new stream module without naming the existing MAIN and NOTICES_STREAM constants, so the names can be defined twice. |
| R-17 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. An output selects streams"],"`mu-config`: the step of an `output` gets a `Selection` with `scope` (`World` or `AllWorlds`) and `patterns`."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  | severity? blocker 0.68 | D8 puts the Selection on the Step, but a rewritten plan is built from tree::Block, which holds only kind, id and max. A move or hide that takes the rewrite path drops the scope and the patterns of every output, and no task gives the tree the Selection. |
| R-18 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: An output shows the streams that its patterns match"],"A list of only exclusions SHALL be bad."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | A list of only exclusions is bad, but no test passes the empty list (showing then a place word or the end of the line) or a lone !. |
| R-19 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7. Each event has a ULID and its streams"],"The protocol `Event` gets `string id = 7` and `repeated string streams = 8`."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | Streams guards 1 to 8 valid names on the server, but the client reads repeated string streams from the wire. No decision or task says what on_event does with an empty list or a name that is not valid. |
| R-20 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["design.md","What enforces each rule"],"\| The writer and the parser agree on the scope and the patterns \| The property test `any_layout_reads_back` \|"] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | The design names any_layout_reads_back as the guard of writer and parser agreement on scope and patterns, but the test compares trees, and the tree holds no Selection. A writer that drops showing passes it. |
| R-21 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D5. One function decides the streams"],"A rule reports at most one error until `reset()`."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | The spec allows one bad-name error and one slow-match error for each rule. D5 allows one error for each rule of any kind, so a slow match after a bad name of the same rule records no error. |
| R-22 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D5. One function decides the streams"],"If a capture runs, the line gets its streams. The capture counts the line. If the line matches `until`, or the count reaches `lines`, the capture ends."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | With lines = 1, D5 starts the capture with count 1 and checks the limit only when a capture runs. The next line is captured too, so the capture holds 2 lines where the spec allows 1. |
| R-23 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/line-rules/spec.md","Requirement: The fields of a rule are checked at the load"],"`lines` SHALL be an integer from 1 to 1000."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | The range of lines is 1 to 1000, but the only load test passes 1001. No test passes 0, a negative value, a non-integer, or the accepted 1000. |
| R-24 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/line-rules/spec.md","Requirement: The fields of a rule are checked at the load"],"`stream` SHALL be a string or a list of 1 to 8 strings."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | A stream list holds 1 to 8 strings, but the only test passes 9. No test passes an empty list or an accepted list of 8. |
| R-25 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/line-rules/spec.md","Requirement: The fields of a rule are checked at the load"],"Each entry of `stream` with no `$` SHALL be a valid stream name."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | A fixed stream entry is checked as a name at the load, while a template result is lower-cased first. No test fixes which side a fixed upper-case entry such as stream = "Chat" falls on. |
| R-26 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","The blocks of the screen"],"The blocks SHALL be `tabs`, `output`, `editor` and `statusline`."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | The requirement 'The blocks of the screen' has no task line that names its subject: no task line holds 'blocks' or 'screen'. |
| R-27 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/line-rules/spec.md","Only lines from the game are matched"],"The server SHALL run the rules on the lines from the game only."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | The requirement 'Only lines from the game are matched' has no task line with two of its distinctive words (lines, game, matched). Only the test name 'the_line_that_the_player_sends' overlaps. |
| R-28 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","The main output is not the first output line"],"a world sends the line `A line.`, and the client shows a notice"] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | The WHEN step 'the client shows a notice' names no concrete input or state, so the scenario does not say what triggers the notice. |
| R-29 | 1 | 3 | blocker | in-change | false-test | ["artifact",["design.md","Risks / Trade-offs"],"Each removed requirement goes with its tests, and the tasks name them."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  | severity? should-fix 0.62 | The risk for the removal of the main-output role says the tasks name the removed tests. Task 7.1 names none. Tests of the removed rules exist, such as a_stream_with_no_output_goes_to_the_first. |
| R-30 | 1 | 2 | suggestion | in-change | unmet-requirement | ["artifact",["design.md","Risks / Trade-offs"],"A layout file of the owner gets the same fix, and the guide says so."] | fixed | bb038add4871d29dbe937cced84f53ca0e1c296d | 2 |  | severity? should-fix 0.69 | The risk of notices shown twice says the guide tells the owner to fix a layout file. Task 7.5 lists the topics of layouts.md, and this fix is not one of them. |
| R-31 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D7. Each event has a ULID and its streams"],"the journal makes its id with the monotonic generator of the `ulid` crate."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | D7 adds the ulid crate for event ids, but uuid is already a workspace dependency with a time-ordered version; D7 names neither uuid nor a reason to add a second id crate. |
| R-32 | 1 | 2 | should-fix | deviates-design | next-change | ["artifact",["design.md","D6. A hook can set the streams"],"`HookAction` gets the variant `Stream(Streams)`."] | fixed | bb038add4871d29dbe937cced84f53ca0e1c296d | 2 |  |  | Stream as its own HookAction variant beside Replace means one hook of #60 cannot rewrite a line and route it in one result, so #60 must add a variant or a struct. |
| R-33 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D7. Each event has a ULID and its streams"],"So two events in one millisecond still sort in their order."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | D7 claims ULID order follows the order of the sequence numbers, even within one millisecond. That is a monotonicity claim over any count of appends, and ids_in_one_stream checks only the length and that the ids are unique. |
| R-34 | 1 | 2 | suggestion | in-change |  | ["artifact",["specs/client-layout/spec.md","Requirement: An output shows the streams that its patterns match"],"The order of the patterns SHALL NOT matter."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | Selection::shows must not depend on the order of its patterns, over any list of patterns and any stream name. One example test, the_order_does_not_matter, covers this. |
| R-35 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An empty list of patterns"] | fixed | 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 2 |  |  | delta scenario 'An empty list of patterns' has no matrix row |
| R-36 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An id in upper case"] | fixed | 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 2 |  |  | delta scenario 'An id in upper case' has no matrix row |
| R-37 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A fixed stream in upper case"] | fixed | 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 2 |  |  | delta scenario 'A fixed stream in upper case' has no matrix row |
| R-38 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An empty list of streams"] | fixed | 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 2 |  |  | delta scenario 'An empty list of streams' has no matrix row |
| R-39 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Lines of zero"] | fixed | 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 2 |  |  | delta scenario 'Lines of zero' has no matrix row |
| R-40 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A capture of one line"] | fixed | 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 2 |  |  | delta scenario 'A capture of one line' has no matrix row |
| R-41 | 3 | 3 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: An output can show the streams of all worlds"],"The output `main` SHALL NOT take this form."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | The spec forbids `of all worlds` on the output `main`, and the scenario expects an error with `line 1` and `main`. No decision refuses it: D8 reads the scope after any id, and from_steps only checks that main exists. |
| R-42 | 3 | 3 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D9. The client routes by selection"],"the field `unshown`, `UNSHOWN_MAX`, `show_unshown`, and the branch of `replay_notices` that fills `unshown`."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  | severity? should-fix 0.69 | D9 keeps the kept notices and replay_notices but gives no rule for a pane of Owner::All. At the first focus and after a Reset, replay_notices pushes each kept notice into the notices pane of the world in focus. For an output of all worlds that is the same pane that already shows it, so the notice shows twice. The kept list holds bare strings, so a replayed notice has no owner for its prefix. |
| R-43 | 3 | 3 | should-fix | in-change | drift | ["artifact",["design.md","D8. An output selects streams"],"The words of a place are `above`, `below`, `left`, `right`, `with` and `at`."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | D8 ends the list of patterns at a fixed list of place words. The parser already knows the first words of a place in its place parser, so the two lists can change apart when a place form is added. |
| R-44 | 3 | 3 | should-fix | in-change | drift | ["artifact",["design.md","D9. The client routes by selection"],"and the branch of `replay_notices` that fills `unshown`. `Late::Waiting` and its probe tests stay."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | D9 routes each notice by its owner, but keeps replay_notices and the owner-less list notices. The replay puts every kept notice into pane_for(NOTICES_STREAM) of the focused world, a second notice route beside D9's. |
| R-45 | 3 | 3 | should-fix | in-change | drift | ["artifact",["design.md","D9. The client routes by selection"],"A notice with no such world belongs to no world, and its pane owner is `Owner::Client`."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | D9 keys the panes of no world by Owner::Client, but does not name PaneId::NOTICES, the fixed pane of no world that focused_pane and the Reset branch use. Two keys then name the pane of no world. |
| R-46 | 3 | 3 | should-fix | in-change | drift | ["artifact",["design.md","D11. A fake world takes the rules of global.toml"],"`server.rs` resolves the global layer of `--config` alone, with its `disable`, and gives its `Rules` to each fake world."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | D11 has server.rs resolve the global layer again, while Store::open already resolves every layer from the one start snapshot. The rules of a fake world can then come from another read of global.toml than the configured worlds. |
| R-47 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/model.rs",860] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  | severity? blocker 0.79 | The client falls back to a layout of one editor and guards it with unreachable!. The change makes that layout bad, and no task changes the fallback, so the fallback can now panic. |
| R-48 | 3 | 3 | should-fix | in-change | unmet-requirement | ["path","crates/mu-config/tests/layout.rs",319] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | The only test of the accepted 32-character edge of an id parses a layout with no main. The change makes that layout bad, and no task rewrites the test. Stream names now share this limit, and no stream.rs test checks 32 or 33 characters. |
| R-49 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D1. A stream name is a block id, in mu-config"],"`Streams::from_wire(&[String])` returns `Streams::main()` for an empty list or a bad name."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | Streams holds 1 to 8 names with none repeated. Streams::from_wire is specified only for an empty list and for a bad name. No decision or test covers a wire list of 9 names or a list with a repeated name. |
| R-50 | 3 | 3 | should-fix | in-change | unmet-requirement | ["artifact",["specs/line-rules/spec.md","Requirement: A slow match counts as no match"],"If a match passes the backtrack limit of the engine, the match SHALL stop."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | The spec stops every match at the backtrack limit. D5 covers only a slow `pattern`. Nothing says what a slow `until` inside a capture does, whether it reports an error or whether the capture goes on, and no test passes one. |
| R-51 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. One function decides the streams"],"If one name is not valid, the line gets `Streams::main()` and a `BadName` error."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | A rule with `until` whose start line gives a bad name sends that line to main. Nothing says whether a capture still starts. If one does, up to 100 later lines go to main and no other rule runs on them. No test covers this case. |
| R-52 | 3 | 3 | should-fix | in-change | next-change | ["artifact",["design.md","D6. A hook can rewrite and route a line in one result"],"If a hook on a game line set streams, the session does not run the rules for that line."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | A capture gives each later game line its streams and ends at the first line that matches `until`. D6 skips the rules on a line whose streams a hook set, and a gagged line never reaches route. So a hook of #60 on an `until` line leaves the capture running until its limit. |
| R-53 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","D2. Rules are checked per layer and compiled at the resolve"],"`Rule::compile(&Entry)` returns `Ok(None)` for an entry with no action field"] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  | severity? blocker 0.60 | D2 calls Rule::compile(&Entry) from check_layer, but a layer holds its rules as toml::Table. An Entry exists only after accumulate in resolve.rs, so the per-layer check has no Entry to pass. |
| R-54 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","D10. `mu world match`"],"It prints each error on stderr, and exits 0."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | D10 prints each rule error on stderr without naming crate::warn, the helper of mu-cli that writes a message of a verb to stderr through the REPORT map. |
| R-55 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","D4. One walk of game text"],"So the text that a pattern sees is the text that the screen shows, without its colors."] | fixed | 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  | severity? should-fix 0.62 | D4 claims that a pattern sees the text the screen shows, without colors, for any line. Only example tests on each side check it. Nothing compares plain with the text of convert over arbitrary input. |
