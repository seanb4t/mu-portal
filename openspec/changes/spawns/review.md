---
verdict: changes-requested
phase: implementation
reviewed_at: 2026-11-27T01:59:25+00:00
head: 9bea191f2b2efda0ea49fa1317323127e4ffe59f
round: 16
next_id: 115
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: partial
  test-coverage: ok
  test-honesty: partial
  invariants: partial
  verify: not-applicable
---
# Review

## Incomplete lenses

- fit: partial, gaps: 2
  - The production code of D4 (ansi.rs, convert.rs), D5 (session.rs), D7 (journal.rs), D8 (parse.rs, change.rs) and D9 (model.rs routing) was not read against the closest patterns in this round.
  - The new tests of mu-cli world_config.rs (1184 to 1628) and the rest of journal.rs (284 to 483) were not read for copied helpers in this round.
- test-honesty: partial, gaps: 2
  - Change artifacts (proposal, specs, design, tasks) not read; judged each test against its own name and assertions only
  - Not read this round: crates/mu-tui/tests/app.rs lines 2890-3355, crates/mu-core/tests/scripts.rs lines 94 and 306, crates/mu-tui/tests/update.rs, crates/mu-cli/tests/layout.rs, crates/mu-config/tests/layout.rs
- invariants: partial, gaps: 1
  - D1 to D7, D10 and D11 and their requirements were not re-read against the code in this round; the focus was D8 and D9, and earlier rounds cover the rest.

The verdict covers only what the lenses checked. While the `ctl-lens` row of an incomplete lens is open, the verdict cannot be `approve`.

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-104 | fit | blocker | in-change | The new run_command in arrange.rs types a text and presses Enter, a third copy beside run_command in update.rs and enter in app.rs, while the shared support module already holds type_text and press. | crates/mu-tui/tests/arrange.rs:603 — "fn run_command(app: &mut Model, text: &str) {" | Move run_command (type_text, then press Enter, returning the effects) into tests/support/mod.rs and call it from arrange.rs, update.rs and app.rs. |  | severity? suggestion 0.82 | drift |
| R-105 | fit | blocker | in-change | The new line_of in update.rs builds the Msg of a game line of a world, a third copy of game in arrange.rs and event/game in app.rs; it only adds the streams field. | crates/mu-tui/tests/update.rs:1462 — "fn line_of(world: &str, text: &str, streams: &[&str]) -> Msg {" | Put one game-line constructor with streams in tests/support/mod.rs, and make game of arrange.rs and app.rs call it with no streams. |  |  | drift |
| R-106 | fit | should-fix | in-change | The new GameHook in scripts.rs gives an action to each game line that contains a text, the body of the existing GameGag with the action as a field. GameGag is GameHook with HookAction::Gag. | crates/mu-core/tests/scripts.rs:585 — "struct GameHook {" | Remove GameGag and write its two uses as GameHook with action HookAction::Gag. |  |  | drift |
| R-107 | fit | should-fix | in-change | The property tests of stream.rs and layout.rs each write the same generator text of a pattern. If the characters of a pattern change, one property test keeps the old set. | crates/mu-config/tests/stream.rs:197 — ""!?[a-c*?][a-c*?-]{0,3}"" | Give the generator of a pattern one home, such as a tests/support module of mu-config that both test files use. |  |  | drift |
| R-108 | fit | should-fix | in-change | The new with_rules in client.rs repeats the body of the existing with_layout (layout_dir, attached_showing, the triple) and only adds global.toml, so with_layout is with_rules with no rules. | crates/mu-cli/tests/client.rs:2510 — "let config = layout_dir(lines, "").file("global.toml", global);" | Keep one helper: make with_rules build on with_layout's body by one function that takes the optional global.toml, and call it from both. |  | severity? suggestion 0.82 | drift |
| R-109 | invariants | should-fix | in-change | A move writes `showing at right of main`, which the parser reads as the place `at right`. Probe: mu layout place on `output as side showing at below main` wrote a file that no longer loads (unknown form at `of`). | crates/mu-config/src/layout/parse.rs:213 — "if peek(at_end).parse_next(input).is_ok() \|\| peek(place_next).parse_next(input).is_ok() {" | Refuse in Pattern::new a pattern that is a word of WORDS (it matches no stream name), and let the generator of a_line_reads_back draw patterns from WORDS and digits. |  |  | wrong-behaviour |
| R-110 | invariants | should-fix | in-change | A last pattern of digits on a first line reads back as a size. Probe: mu layout hide tabs rewrote `output as main showing main, notices, 30` as line 1, and the file then fails with bad size. | crates/mu-config/src/layout/parse.rs:216 — "if !patterns.is_empty() && peek(size_next).parse_next(input).is_ok() {" | Refuse in Pattern::new a pattern whose first character is a digit (no stream name starts with one), and add a digit pattern with Place::First to the generator of a_line_reads_back. |  |  | wrong-behaviour |
| R-111 | invariants | should-fix | in-change | A Reset drops the Owner::All keys from each kept notice so the replay refills the shared pane once. No test puts a kept notice in an output of all worlds through a Reset, so keeping the All key would pass. | crates/mu-tui/src/update.rs:1248 — "model.panes.clear_where(\|key\| key.owner != Owner::Client);" | Add a test in crates/mu-tui/tests/update.rs with NOTICES_OF_ALL: keep a notice, send two Resets, and assert the pane of notices holds it exactly once with its owner prefix. |  |  | wrong-behaviour |
| R-112 | test-coverage | should-fix | in-change | The CLI test for nine outputs passes a layout with no `main`, so the no-main error satisfies it and it cannot fail on the output-count rule. | crates/mu-cli/tests/layout.rs:260 — "lines.extend((0..9).map(\|number\| format!("output as o{number} above editor")));" | In nine_outputs of crates/mu-cli/tests/layout.rs make the first output `output` (id main) and the other eight `output as oN`, and assert `line 10` as the unit test does in crates/mu-config/tests/layout.rs. |  |  | false-test |
| R-113 | test-coverage | should-fix | in-change | The scenario A block that is not placed yet expects `line 2` and `output`. The test asserts `main`, and the error text names the anchor, so `output` is never checked. | crates/mu-cli/tests/layout.rs:186 — "bad_layout(&["editor", "tabs above main"], &["line 2", "main"]).await;" | Assert the words the scenario names (`line 2` and `output`) in a_block_that_is_not_placed_yet, or change the scenario THEN to name `main`. |  |  | false-test |
| R-114 | approach | suggestion | in-change | D7 says the time in the id is the time of the event. The ulid generator keeps the millisecond of the previous id when the clock went back or an overflow moved it ahead, so the id then holds a later time than the event. The unit test of the overflow builds that case. | design.md > D7. Each event has a ULID and its streams — "The event takes the same time, so the time in the id is the time of the event." | Narrow the sentence of D7: the id takes the time of the event, unless the clock went back or an overflow moved the generator past it. Then the id keeps the later millisecond of the id before it, so the order holds. |  |  |  |

## Repeats

| id | lens | severity | claim | evidence |
|---|---|---|---|---|
| R-84 | fit | should-fix | lens fit reported partial, gaps: 2 | review > lens-status — "fit partial" |
| R-75 | test-honesty | should-fix | lens test-honesty reported partial, gaps: 2 | review > lens-status — "test-honesty partial" |
| R-98 | invariants | should-fix | lens invariants reported partial, gaps: 1 | review > lens-status — "invariants partial" |

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-74 | should-fix | accepted | a full round 14 runs the lenses fit, test-coverage and test-honesty again, each on the files that no earlier round read |  |
| R-75 | should-fix | accepted | a full round 14 runs the lenses fit, test-coverage and test-honesty again, each on the files that no earlier round read |  |
| R-84 | should-fix | accepted | a full round 14 runs the lenses fit, test-coverage and test-honesty again, each on the files that no earlier round read |  |
| R-98 | should-fix | accepted | a full round 15 runs the lens invariants again with a focus on D10 and run_match |  |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | ba7585f62738e6f4f8727d98721222b2a96fdd2a | ebf029ee769ac3ccba980ec8da064ebdb7babab2e3d969c6fe364a8333f2bb31 | changes-requested | 34 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 2466ac7447be4a80f8c44b9f7ec06d8e6914f6a6 | bd5f81b131a7420dfd4a456c13a54b9cbd09355372f53a3469dcd7615539d3f8 | approve-with-notes | 6 | 0 | 4 | 30 | 0 |  |
| 3 | plan | full | 942b5631e0ff1d7cfcb581de3c3eb62182d1e4fc | 98de8a29e698b7a5f4f33c0f980da09a923c76fd2e4370c371ca9b3185bea2a1 | changes-requested | 15 | 0 | 1 | 0 | 0 | the new scenarios of round 2 need coverage-matrix rows |
| 4 | plan | follow-up | aedd20ff0417110f792f5a8f5dd2fc972dcd66fe | 7ac128cf9c34b79b8976101b7d50382d31600301b50318e260f3e90436c7a688 | changes-requested | 0 | 0 | 2 | 14 | 0 |  |
| 5 | plan | follow-up | c6c660e986a9c71aa487274e73d8aba0abde359e | 6561008f068803d5324b79d5102e8dd89bf91545a936b5ebf05fbe1ca4838eb8 | changes-requested | 1 | 0 | 1 | 1 | 0 |  |
| 6 | plan | follow-up | 5a2f91766e2df48222525df7c121c36726befbd6 | a6ec267d6250680589df66865fe6ad30798fa957f78c56ef05b327d7f10de327 | changes-requested | 0 | 0 | 1 | 1 | 0 |  |
| 7 | plan | follow-up | 1d3f311021442edf187180d00652d30e3347dc50 | 85d6b409b09049b2ff758d1601c05f69f9a80ae23ef0cdd419824fb2149602fd | changes-requested | 0 | 0 | 1 | 0 | 0 |  |
| 8 | plan | follow-up | 4092a7a492de974963a527438bb58cbad1e7d558 | 3aa495c9ad862e5270561d9b544ecf2f69bfbb88fd83df1996ab160ffdbbf479 | changes-requested | 0 | 0 | 1 | 0 | 0 |  |
| 9 | plan | follow-up | 44b186825714cda4ed214bf53700fbdfaee81721 | 0ae0965742e62667dd5791e9e3e05b6370ffe04dfa9584f96fd29cd61a3da55e | changes-requested | 1 | 0 | 1 | 0 | 0 |  |
| 10 | plan | follow-up | 91c8e35e2a1be320a84f3108404c69939f305fd3 | 0cd5ac22c6fcb2a5044319acc4996ddf6f2840390ac19daac87d5aa9c1c7bb3b | approve | 0 | 0 | 0 | 2 | 0 |  |
| 11 | implementation | full | 3f3234a0df4b282fce7ff6854a864c9a41f21113 | 188416239a2c6a435b89f4250b94f9dc42db67fa3cba69b898688d98c7189c34 | changes-requested | 18 | 0 | 0 | 0 | 0 |  |
| 12 | implementation | full | cf9cb24b8f27664d6543ef97ad591224843e3873 | 851e329d2369632f9b19a6611d499f9ae821d82c9e8267f7ffa9843f666d4028 | approve-with-notes | 9 | 1 | 0 | 0 | 0 | run again the lenses test-coverage and test-honesty, which were partial in round 11 |
| 13 | implementation | full | 9ab627bef59d0a891174d22b27a7b8ea178b330f | 095f2a554577c506ab0072af6c6949b3575516dee16490ec0a0baa5f7912d519 | approve-with-notes | 8 | 3 | 1 | 0 | 0 | run the lenses fit and test-honesty again on the files that they did not read in round 12 |
| 14 | implementation | full | b88816503b9157da21297b2cc6c1f287ba4c6d1c | 72fa2e589142b7ea7bdfd02dbe091b41586f990779f9bd839c9d3b725782973f | approve-with-notes | 6 | 0 | 0 | 0 | 0 | run the lenses fit, test-coverage and test-honesty again on the files that no earlier round read |
| 15 | implementation | full | 9da154dc35d5d7206f715cffacc99ad96b3929ba | 1409821cd38ebc1dd2f07c88f78f12dfad5fcc727066fdd65bee918ee64f2b05 | changes-requested | 5 | 3 | 0 | 0 | 0 | run the lens invariants again with a focus on D10, which round 14 did not read |
| 16 | implementation | full | 9bea191f2b2efda0ea49fa1317323127e4ffe59f | 09d7df900d32588432df1c89f21c900a574d84b909513320c65a8d5876c73ffb | changes-requested | 11 | 3 | 0 | 0 | 0 | run the lenses fit, test-honesty and invariants again on the files that round 15 did not read |

## Judge

judge: 22 questions, 0.001 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A rule with a stream loads | crates/mu-cli/tests/world_config.rs: a_rule_with_a_stream_loads | covered | pass |
| An unknown field | crates/mu-cli/tests/world_config.rs: a_rule_with_an_unknown_field | covered | pass |
| A rule with no action | crates/mu-cli/tests/world_config.rs: a_rule_with_no_action | covered | pass |
| An action with no pattern | crates/mu-cli/tests/world_config.rs: an_action_with_no_pattern | covered | pass |
| A pattern that does not compile | crates/mu-cli/tests/world_config.rs: a_pattern_that_does_not_compile | covered | pass |
| A fixed stream that is not a name | crates/mu-cli/tests/world_config.rs: a_fixed_stream_that_is_not_a_name | covered | pass |
| A fixed stream in upper case | crates/mu-cli/tests/world_config.rs: a_fixed_stream_in_upper_case | covered | pass |
| An empty list of streams | crates/mu-cli/tests/world_config.rs: an_empty_list_of_streams | covered | pass |
| Too many streams | crates/mu-cli/tests/world_config.rs: too_many_streams | covered | pass |
| Lines out of range | crates/mu-cli/tests/world_config.rs: lines_out_of_range | covered | pass |
| Lines of zero | crates/mu-cli/tests/world_config.rs: lines_of_zero | covered | pass |
| Lines without until | crates/mu-cli/tests/world_config.rs: lines_without_until | covered | pass |
| A fixed stream names the notices | crates/mu-cli/tests/world_config.rs: a_list_of_streams_names_the_notices | covered | pass |
| A name that is a word of the sentences | crates/mu-cli/tests/world_config.rs: a_stream_name_that_is_a_word | covered | pass |
| The earlier rule wins | crates/mu-cli/tests/world_config.rs: the_earlier_rule_wins | covered | pass |
| A character rule after a global rule | crates/mu-cli/tests/world_config.rs: a_character_rule_after_a_global_rule | covered | pass |
| No match | crates/mu-cli/tests/world_config.rs: no_match | covered | pass |
| A rule that names main | crates/mu-cli/tests/world_config.rs: a_rule_that_names_main | covered | pass |
| A lookbehind and a named group | crates/mu-cli/tests/world_config.rs: a_lookbehind_and_a_named_group | covered | pass |
| A back-reference | crates/mu-cli/tests/world_config.rs: a_back_reference | covered | pass |
| Case counts | crates/mu-cli/tests/world_config.rs: case_counts | covered | pass |
| A colored line | crates/mu-cli/tests/world_config.rs: a_colored_line | covered | pass |
| One stream for each channel | crates/mu-cli/tests/world_config.rs: one_stream_for_each_channel | covered | pass |
| A name from a capture that is not valid | crates/mu-cli/tests/world_config.rs: a_name_from_a_capture_that_is_not_valid | covered | pass |
| A line in two streams | crates/mu-cli/tests/world_config.rs: a_line_in_two_streams | covered | pass |
| Two bad names | crates/mu-cli/tests/journal.rs: two_bad_names | covered | pass |
| A capture names the notices | crates/mu-cli/tests/world_config.rs: a_capture_names_the_notices | covered | pass |
| The line that the player sends | crates/mu-cli/tests/journal.rs: the_line_that_the_player_sends | covered | pass |
| A WHO list | crates/mu-cli/tests/world_config.rs: a_who_list | covered | pass |
| A line that starts and could end | crates/mu-cli/tests/world_config.rs: a_line_that_starts_and_could_end | covered | pass |
| Another rule during a capture | crates/mu-cli/tests/world_config.rs: another_rule_during_a_capture | covered | pass |
| A capture that never sees its end | crates/mu-cli/tests/world_config.rs: a_capture_that_never_sees_its_end | covered | pass |
| A capture of one line | crates/mu-cli/tests/world_config.rs: a_capture_of_one_line | covered | pass |
| The default limit | crates/mu-cli/tests/world_config.rs: the_default_limit | covered | pass |
| A drop during a capture | crates/mu-cli/tests/journal.rs: a_drop_during_a_capture | covered | pass |
| A pattern that backtracks too much | crates/mu-cli/tests/journal.rs: a_pattern_that_backtracks_too_much | covered | pass |
| An unknown world | crates/mu-cli/tests/world_config.rs: match_an_unknown_world | covered | pass |
| No rules | crates/mu-cli/tests/world_config.rs: match_with_no_rules | covered | pass |
| A bad name on stderr | crates/mu-cli/tests/world_config.rs: a_bad_name_on_stderr | covered | pass |
| A carriage return in a line | crates/mu-cli/tests/world_config.rs: match_drops_a_carriage_return_as_the_session_does | covered | pass |
| A line longer than a session holds | crates/mu-cli/tests/world_config.rs: match_cuts_a_long_line_as_the_session_does | covered | pass |
| Ids in one stream | crates/mu-cli/tests/journal.rs: ids_in_one_stream | covered | pass |
| Ids after a restart | crates/mu-cli/tests/journal.rs: ids_after_a_restart | covered | pass |
| A live event and a replay | crates/mu-cli/tests/journal.rs: a_live_event_and_a_replay | covered | pass |
| No event without a stream | crates/mu-cli/tests/journal.rs: no_event_without_a_stream | covered | pass |
| A fake world with a rule | crates/mu-cli/tests/journal.rs: a_fake_world_with_a_rule | covered | pass |
| A fake world with no rules | crates/mu-cli/tests/journal.rs: a_fake_world_with_no_rules | covered | pass |
| No tabs | crates/mu-cli/tests/client.rs: no_tabs | covered | pass |
| The editor alone | crates/mu-cli/tests/layout.rs: the_editor_alone | covered | pass |
| Nine outputs | crates/mu-cli/tests/layout.rs: nine_outputs | partial | pass |
| One output shows the notices too | crates/mu-cli/tests/client.rs: one_output_shows_the_notices_too | covered | pass |
| An output for the notices | crates/mu-cli/tests/client.rs: an_output_for_the_notices | covered | pass |
| An output with no stream yet | crates/mu-cli/tests/client.rs: an_output_with_no_stream_yet | covered | pass |
| The main output is not the first output line | crates/mu-cli/tests/client.rs: the_main_output_is_not_the_first_output_line | covered | pass |
| No main output | crates/mu-cli/tests/layout.rs: no_main_output | covered | pass |
| A refused line goes to the notices | crates/mu-cli/tests/client.rs: a_refused_line_goes_to_the_notices | covered | pass |
| A refused line of a world that lost the focus | crates/mu-cli/tests/client.rs: a_refused_line_of_a_world_that_lost_the_focus | covered | pass |
| A refused line after a Reset | crates/mu-tui/tests/update.rs: a_refused_line_notice_comes_back_after_a_reset | covered | pass |
| A line with no world goes to the notices | crates/mu-cli/tests/client.rs: a_line_with_no_world_goes_to_the_notices | covered | pass |
| A notice shows under the world in focus only | crates/mu-cli/tests/client.rs: a_notice_shows_under_the_world_in_focus_only | covered | pass |
| An output that comes later starts with new lines | crates/mu-cli/tests/client.rs: an_output_that_comes_later_starts_with_new_lines | covered | pass |
| An unknown block | crates/mu-cli/tests/layout.rs: an_unknown_block | covered | pass |
| A block twice | crates/mu-cli/tests/layout.rs: a_block_twice | covered | pass |
| A block that is not placed yet | crates/mu-cli/tests/layout.rs: a_block_that_is_not_placed_yet | partial | pass |
| No editor | crates/mu-cli/tests/layout.rs: no_editor | covered | pass |
| No output for the notices | crates/mu-cli/tests/layout.rs: no_output_for_the_notices | covered | pass |
| A percent out of range | crates/mu-cli/tests/layout.rs: a_percent_out_of_range | covered | pass |
| Too many lines | crates/mu-cli/tests/layout.rs: too_many_lines | covered | pass |
| The default layout | crates/mu-cli/tests/config_store.rs: the_default_layout | covered | pass |
| The sidebar layout | crates/mu-cli/tests/client.rs: the_sidebar_layout | covered | pass |
| The sidebar keeps notices out of main | crates/mu-cli/tests/client.rs: the_sidebar_keeps_notices_out_of_main | covered | pass |
| One output for all channels | crates/mu-cli/tests/client.rs: one_output_for_all_channels | covered | pass |
| An exclusion | crates/mu-cli/tests/client.rs: an_exclusion | covered | pass |
| Only exclusions | crates/mu-cli/tests/layout.rs: only_exclusions | covered | pass |
| An empty list of patterns | crates/mu-cli/tests/layout.rs: an_empty_list_of_patterns | covered | pass |
| A reserved pattern | crates/mu-cli/tests/layout.rs: a_reserved_pattern | covered | pass |
| A plan keeps the patterns | crates/mu-cli/tests/layout.rs: a_plan_keeps_the_patterns | covered | pass |
| Two outputs show main | crates/mu-cli/tests/client.rs: two_outputs_show_main | covered | pass |
| A line in two streams of one output | crates/mu-cli/tests/client.rs: a_line_in_two_streams_of_one_output | covered | pass |
| A channel with no output | crates/mu-cli/tests/client.rs: a_channel_with_no_output | covered | pass |
| The notice of a stream comes back after a Reset | crates/mu-tui/tests/update.rs: the_hint_of_a_stream_with_no_output_comes_back_after_a_reset | covered | pass |
| The notice of a stream goes to the world of the line | crates/mu-tui/tests/update.rs: the_hint_of_a_stream_with_no_output_goes_to_the_world_of_the_line | covered | pass |
| The oldest stream gets its notice again | crates/mu-tui/tests/update.rs: the_oldest_stream_with_no_output_gives_its_hint_again | covered | pass |
| Hide the main output | crates/mu-cli/tests/client.rs: hide_the_main_output | covered | pass |
| Hide the only output of the notices | crates/mu-cli/tests/client.rs: hide_the_only_output_of_the_notices | covered | pass |
| Hide a second output of the notices | crates/mu-cli/tests/client.rs: hide_a_second_output_of_the_notices | covered | pass |
| A page from another world | crates/mu-cli/tests/client.rs: a_page_from_another_world | covered | pass |
| Patterns of all worlds | crates/mu-cli/tests/layout.rs: patterns_of_all_worlds | covered | pass |
| The main output of all worlds | crates/mu-cli/tests/layout.rs: the_main_output_of_all_worlds | covered | pass |
| The id all | crates/mu-cli/tests/layout.rs: the_id_all | covered | pass |
| The scope after the place | crates/mu-cli/tests/layout.rs: the_scope_after_the_place | covered | pass |
| A place command with the scope | crates/mu-cli/tests/client.rs: a_place_command_with_the_scope | covered | pass |
| A prompt in two outputs | crates/mu-tui/tests/update.rs: a_prompt_goes_to_each_output_that_shows_its_stream | covered | pass |
| No prompt in an output of all worlds | crates/mu-tui/tests/update.rs: an_output_of_all_worlds_shows_no_prompt | covered | pass |
| A place command keeps the streams | crates/mu-config/tests/layout.rs: a_place_keeps_the_streams_of_an_output_on_the_screen | covered | pass |
| A place command with showing sets the streams | crates/mu-config/tests/layout.rs: a_place_with_showing_sets_the_streams | covered | pass |
| Two worlds in one pane | crates/mu-cli/tests/client.rs: two_worlds_in_one_pane | covered | pass |
| A notice in an output of all worlds | crates/mu-cli/tests/client.rs: a_notice_in_an_output_of_all_worlds | covered | pass |
| A world with another layout | crates/mu-cli/tests/client.rs: a_world_with_another_layout | covered | pass |
| An id in upper case | crates/mu-cli/tests/layout.rs: an_id_in_upper_case | covered | pass |

### Not run

None.

## Verification runs

- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_rule_with_a_stream_loads)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_rule_with_an_unknown_field)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_rule_with_no_action)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=an_action_with_no_pattern)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_pattern_that_does_not_compile)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_fixed_stream_that_is_not_a_name)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_fixed_stream_in_upper_case)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=an_empty_list_of_streams)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=too_many_streams)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=lines_out_of_range)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=lines_of_zero)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=lines_without_until)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_list_of_streams_names_the_notices)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_stream_name_that_is_a_word)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=the_earlier_rule_wins)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_character_rule_after_a_global_rule)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=no_match)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_rule_that_names_main)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_lookbehind_and_a_named_group)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_back_reference)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=case_counts)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_colored_line)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=one_stream_for_each_channel)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_name_from_a_capture_that_is_not_valid)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_line_in_two_streams)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=two_bad_names)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_capture_names_the_notices)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=the_line_that_the_player_sends)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_who_list)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_line_that_starts_and_could_end)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=another_rule_during_a_capture)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_capture_that_never_sees_its_end)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_capture_of_one_line)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=the_default_limit)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=a_drop_during_a_capture)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=a_pattern_that_backtracks_too_much)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=match_an_unknown_world)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=match_with_no_rules)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_bad_name_on_stderr)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=match_drops_a_carriage_return_as_the_session_does)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=match_cuts_a_long_line_as_the_session_does)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=ids_in_one_stream)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=ids_after_a_restart)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=a_live_event_and_a_replay)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=no_event_without_a_stream)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=a_fake_world_with_a_rule)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=a_fake_world_with_no_rules)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=no_tabs)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=the_editor_alone)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=nine_outputs)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=one_output_shows_the_notices_too)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=an_output_for_the_notices)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=an_output_with_no_stream_yet)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_main_output_is_not_the_first_output_line)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=no_main_output)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_refused_line_goes_to_the_notices)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_refused_line_of_a_world_that_lost_the_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test update --locked -E 'test(=a_refused_line_notice_comes_back_after_a_reset)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_line_with_no_world_goes_to_the_notices)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_notice_shows_under_the_world_in_focus_only)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=an_output_that_comes_later_starts_with_new_lines)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=an_unknown_block)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_block_twice)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_block_that_is_not_placed_yet)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=no_editor)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=no_output_for_the_notices)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_percent_out_of_range)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=too_many_lines)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=the_default_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_sidebar_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_sidebar_keeps_notices_out_of_main)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=one_output_for_all_channels)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=an_exclusion)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=only_exclusions)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=an_empty_list_of_patterns)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_reserved_pattern)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=a_plan_keeps_the_patterns)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=two_outputs_show_main)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_line_in_two_streams_of_one_output)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_channel_with_no_output)'`
- exit 0: `cargo nextest run -p mu-tui --test update --locked -E 'test(=the_hint_of_a_stream_with_no_output_comes_back_after_a_reset)'`
- exit 0: `cargo nextest run -p mu-tui --test update --locked -E 'test(=the_hint_of_a_stream_with_no_output_goes_to_the_world_of_the_line)'`
- exit 0: `cargo nextest run -p mu-tui --test update --locked -E 'test(=the_oldest_stream_with_no_output_gives_its_hint_again)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=hide_the_main_output)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=hide_the_only_output_of_the_notices)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=hide_a_second_output_of_the_notices)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_page_from_another_world)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=patterns_of_all_worlds)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=the_main_output_of_all_worlds)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=the_id_all)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=the_scope_after_the_place)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_place_command_with_the_scope)'`
- exit 0: `cargo nextest run -p mu-tui --test update --locked -E 'test(=a_prompt_goes_to_each_output_that_shows_its_stream)'`
- exit 0: `cargo nextest run -p mu-tui --test update --locked -E 'test(=an_output_of_all_worlds_shows_no_prompt)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=a_place_keeps_the_streams_of_an_output_on_the_screen)'`
- exit 0: `cargo nextest run -p mu-config --test layout --locked -E 'test(=a_place_with_showing_sets_the_streams)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=two_worlds_in_one_pane)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_notice_in_an_output_of_all_worlds)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_world_with_another_layout)'`
- exit 0: `cargo nextest run -p mu-cli --test layout --locked -E 'test(=an_id_in_upper_case)'`

## Dropped

None.

## Lens notes

- approach: D1 to D6 and D8 to D11 hold against the code at head: stream.rs, step.rs, rules.rs, session.rs, script.rs, check.rs, resolve.rs, store.rs, server.rs, world.rs, ansi.rs, filter.rs, parse.rs, change.rs, layout/mod.rs, model.rs, update.rs and arrange.rs.
- approach: The cited facts of fancy-regex 0.20.2 (the default backtrack limit, BacktrackLimitExceeded, Captures::expand and the longest name), ulid 3.0.0 (generate_from_datetime, Overflow::commit_overflow_increment) and wildmatch 2.7.0 (MIT, no required dependency) hold in the sources of the registry.
- approach: The workspace allowlist at head matches D3, D4, D7 and D8: fancy-regex, ulid and rand_core for mu-core, and vte and wildmatch for mu-config.
- approach: The Risks and the Migration plan name things that exist: the sidebar layout writes `output as main showing main`, and layouts.md gives the advice for a layout with its own output for notices.
- approach: The Context facts describe the code before the change. The change removes several of them, such as Layout::route, main_output and the field unshown, as its decisions say. This lens does not count them as stale.
- fit: The prompt names Cargo.toml as the path of the fit rules; it holds the workspace dependencies and no library-preference rows.
- fit: Read in this round: scripts.rs and the mu-core test support, mu-config stream.rs, layout.rs and ansi.rs tests, mu-tui arrange.rs and its support module, mu-cli screen_text.rs, layout.rs 278 to 410, client.rs 2500 to 2590, journal.rs 205 to 285, mu-config stream.rs, rules.rs 100 to 230 and 470 to 505, mu-cli world.rs and server.rs 145 to 200, mu-tui update.rs 1236 to 1405.
- invariants: I1: A line that the writer writes for a step reads back as the same step \| enforced at Step Display (step.rs) and showing() in parse.rs \| boundary test a_line_reads_back, whose pattern generator uses only a-c, *, ?, - (see inv-1, inv-2)
- invariants: I2: The list after showing is never empty, a lone ! is bad, a list of only exclusions is bad \| enforced at Pattern::new and Selection::new (stream.rs) \| boundary tests showing_with_no_pattern_before_a_place, showing_at_the_end_of_the_line, showing_a_lone_exclusion
- invariants: I3: A size after showing ends the list only after one pattern \| enforced at parse.rs:216 \| boundary test patterns_end_at_the_place covers place then size; probe showed `showing x, 30` with no place reads 30 as a size (inv-2)
- invariants: I4: A pattern that starts with re: or / is bad \| enforced at stream.rs:173, before the lower-case step \| probe: `RE:x` is still refused, with the generic reason, not the reserved one. Refused either way, so no finding
- invariants: I5: of all worlds comes only right after the id, and main never takes it \| enforced at line() in parse.rs and Layout::from_steps \| boundary tests the_scope_after_the_place, the_main_output_of_all_worlds
- invariants: I6: A Reset clears the panes of the worlds and of Owner::All, keeps Owner::Client panes, and replays each kept notice once per pane \| enforced at update.rs:1248-1251 and Model::replay_notices \| boundary tests a_reset_keeps_the_pane_of_no_world, a_reset_puts_each_kept_notice_back_once, a_reset_shows_a_notice_of_no_world_once; no Owner::All case (inv-3)
- invariants: I7: A kept notice is pushed at most once into each pane \| enforced at the HashSet insert in replay_notices (model.rs:754) \| boundary tests a_replay_skips_a_pane_that_holds_the_notice, the_first_focus_fills_a_shared_pane_that_lacks_the_notice
- invariants: I8: At most 256 pairs of stream and world are remembered, the oldest forgotten first \| enforced at Model::first_no_output (model.rs:953) \| boundary: the scenario chan-0 to chan-256 covers both sides of the limit
- invariants: Probes: scratch servers from target/debug/mu (built after the last change of parse.rs) under the handoff directories probe, probe2, probe4; mu layout place and mu layout hide wrote the layout files that then failed to load.
- test-coverage: The bodies of a_prompt_goes_to_each_output_that_shows_its_stream and an_output_of_all_worlds_shows_no_prompt in update.rs were matched by name and a skim only.
- test-coverage: ids_after_a_restart builds a second Session; if that session has a fresh memory journal, the test proves only that two journals give different ids, not that a persisted journal does.
- test-honesty: Read in full: mu-config/tests/stream.rs, mu-cli/tests/world_config.rs 1155-1628, mu-cli/tests/journal.rs 207-482, mu-core/tests/scripts.rs 583-750, mu-tui/tests/arrange.rs 585-622, mu-cli/tests/client.rs 2800-3082, mu-tui/tests/app.rs 2403-2890 and 3355-3424. No test found that cannot fail for the reason it names.
- test-honesty: each_block_has_a_draw compares drawn_blocks (DRAWS) with catalog_blocks (BLOCKS), two independent lists, so it is not a tautology.

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
| R-23 | 1 | 11 | should-fix | in-change | false-test | ["artifact",["specs/line-rules/spec.md","Requirement: The fields of a rule are checked at the load"],"`lines` SHALL be an integer from 1 to 1000."] | fixed | bca5208b23e035c9841307d8552034cb7668bdb9 | 11 |  |  | The range of lines is 1 to 1000, but the only load test passes 1001. No test passes 0, a negative value, a non-integer, or the accepted 1000. |
| R-24 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/line-rules/spec.md","Requirement: The fields of a rule are checked at the load"],"`stream` SHALL be a string or a list of 1 to 8 strings."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | A stream list holds 1 to 8 strings, but the only test passes 9. No test passes an empty list or an accepted list of 8. |
| R-25 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/line-rules/spec.md","Requirement: The fields of a rule are checked at the load"],"Each entry of `stream` with no `$` SHALL be a valid stream name."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | A fixed stream entry is checked as a name at the load, while a template result is lower-cased first. No test fixes which side a fixed upper-case entry such as stream = "Chat" falls on. |
| R-26 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","The blocks of the screen"],"The blocks SHALL be `tabs`, `output`, `editor` and `statusline`."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | The requirement 'The blocks of the screen' has no task line that names its subject: no task line holds 'blocks' or 'screen'. |
| R-27 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/line-rules/spec.md","Only lines from the game are matched"],"The server SHALL run the rules on the lines from the game only."] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | The requirement 'Only lines from the game are matched' has no task line with two of its distinctive words (lines, game, matched). Only the test name 'the_line_that_the_player_sends' overlaps. |
| R-28 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","The main output is not the first output line"],"a world sends the line `A line.`, and the client shows a notice"] | resolved | resolved in round 2 from fixed 7fb4fdcffc5d0c31fe50af3ccbb4260410865ac9 | 1 |  |  | The WHEN step 'the client shows a notice' names no concrete input or state, so the scenario does not say what triggers the notice. |
| R-29 | 1 | 4 | blocker | in-change | false-test | ["artifact",["design.md","Risks / Trade-offs"],"Each removed requirement goes with its tests, and the tasks name them."] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  | severity? should-fix 0.62 | The risk for the removal of the main-output role says the tasks name the removed tests. Task 7.1 names none. Tests of the removed rules exist, such as a_stream_with_no_output_goes_to_the_first. |
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
| R-41 | 3 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-layout/spec.md","Requirement: An output can show the streams of all worlds"],"The output `main` SHALL NOT take this form."] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | The spec forbids `of all worlds` on the output `main`, and the scenario expects an error with `line 1` and `main`. No decision refuses it: D8 reads the scope after any id, and from_steps only checks that main exists. |
| R-42 | 3 | 10 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D9. The client routes by selection"],"the field `unshown`, `UNSHOWN_MAX`, `show_unshown`, and the branch of `replay_notices` that fills `unshown`."] | resolved | resolved in round 10 from fixed 9073e89fe1f05a5ef836f13dceb63020cbe087c1 | 9 |  | severity? should-fix 0.64 | D9 keeps the kept notices and replay_notices but gives no rule for a pane of Owner::All. At the first focus and after a Reset, replay_notices pushes each kept notice into the notices pane of the world in focus. For an output of all worlds that is the same pane that already shows it, so the notice shows twice. The kept list holds bare strings, so a replayed notice has no owner for its prefix. |
| R-43 | 3 | 4 | should-fix | in-change | drift | ["artifact",["design.md","D8. An output selects streams"],"The words of a place are `above`, `below`, `left`, `right`, `with` and `at`."] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | D8 ends the list of patterns at a fixed list of place words. The parser already knows the first words of a place in its place parser, so the two lists can change apart when a place form is added. |
| R-44 | 3 | 4 | should-fix | in-change | drift | ["artifact",["design.md","D9. The client routes by selection"],"and the branch of `replay_notices` that fills `unshown`. `Late::Waiting` and its probe tests stay."] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | D9 routes each notice by its owner, but keeps replay_notices and the owner-less list notices. The replay puts every kept notice into pane_for(NOTICES_STREAM) of the focused world, a second notice route beside D9's. |
| R-45 | 3 | 5 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D9. The client routes by selection"],"A notice with no such world belongs to no world, and its pane owner is `Owner::Client`."] | resolved | resolved in round 5 from fixed 7f12c47df28659270d345f2dba955bf8f254e191 | 4 |  |  | D9 keys the panes of no world by Owner::Client, but does not name PaneId::NOTICES, the fixed pane of no world that focused_pane and the Reset branch use. Two keys then name the pane of no world. |
| R-46 | 3 | 4 | should-fix | in-change | drift | ["artifact",["design.md","D11. A fake world takes the rules of global.toml"],"`server.rs` resolves the global layer of `--config` alone, with its `disable`, and gives its `Rules` to each fake world."] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | D11 has server.rs resolve the global layer again, while Store::open already resolves every layer from the one start snapshot. The rules of a fake world can then come from another read of global.toml than the configured worlds. |
| R-47 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/model.rs",860] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  | severity? blocker 0.79 | The client falls back to a layout of one editor and guards it with unreachable!. The change makes that layout bad, and no task changes the fallback, so the fallback can now panic. |
| R-48 | 3 | 4 | should-fix | in-change | unmet-requirement | ["path","crates/mu-config/tests/layout.rs",319] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | The only test of the accepted 32-character edge of an id parses a layout with no main. The change makes that layout bad, and no task rewrites the test. Stream names now share this limit, and no stream.rs test checks 32 or 33 characters. |
| R-49 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D1. A stream name is a block id, in mu-config"],"`Streams::from_wire(&[String])` returns `Streams::main()` for an empty list or a bad name."] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | Streams holds 1 to 8 names with none repeated. Streams::from_wire is specified only for an empty list and for a bad name. No decision or test covers a wire list of 9 names or a list with a repeated name. |
| R-50 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/line-rules/spec.md","Requirement: A slow match counts as no match"],"If a match passes the backtrack limit of the engine, the match SHALL stop."] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | The spec stops every match at the backtrack limit. D5 covers only a slow `pattern`. Nothing says what a slow `until` inside a capture does, whether it reports an error or whether the capture goes on, and no test passes one. |
| R-51 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. One function decides the streams"],"If one name is not valid, the line gets `Streams::main()` and a `BadName` error."] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | A rule with `until` whose start line gives a bad name sends that line to main. Nothing says whether a capture still starts. If one does, up to 100 later lines go to main and no other rule runs on them. No test covers this case. |
| R-52 | 3 | 4 | should-fix | in-change | next-change | ["artifact",["design.md","D6. A hook can rewrite and route a line in one result"],"If a hook on a game line set streams, the session does not run the rules for that line."] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | A capture gives each later game line its streams and ends at the first line that matches `until`. D6 skips the rules on a line whose streams a hook set, and a gagged line never reaches route. So a hook of #60 on an `until` line leaves the capture running until its limit. |
| R-53 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","D2. Rules are checked per layer and compiled at the resolve"],"`Rule::compile(&Entry)` returns `Ok(None)` for an entry with no action field"] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  | severity? blocker 0.60 | D2 calls Rule::compile(&Entry) from check_layer, but a layer holds its rules as toml::Table. An Entry exists only after accumulate in resolve.rs, so the per-layer check has no Entry to pass. |
| R-54 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","D10. `mu world match`"],"It prints each error on stderr, and exits 0."] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  |  | D10 prints each rule error on stderr without naming crate::warn, the helper of mu-cli that writes a message of a verb to stderr through the REPORT map. |
| R-55 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","D4. One walk of game text"],"So the text that a pattern sees is the text that the screen shows, without its colors."] | resolved | resolved in round 4 from fixed 980174bd2dd7f8ec85aeb990303720827d3b4c55 | 3 |  | severity? should-fix 0.62 | D4 claims that a pattern sees the text the screen shows, without colors, for any line. Only example tests on each side check it. Nothing compares plain with the text of convert over arbitrary input. |
| R-56 | 5 | 6 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D9. The client routes by selection"],"The kept notices keep their owner. `Model::notice` and `replay_notices` route each one through this same routing."] | resolved | resolved in round 6 from fixed 4add08b11af8e98e19d725f76821dd52775dea3b | 5 |  |  | The fix of D9 makes replay_notices route each kept notice through the notice routing, and keeps the panes of Owner::Client at a Reset. It exempts only the panes of Owner::All from the replay. So after a Reset, a kept notice of no world goes again into the Owner::Client pane that kept it, and shows twice there. The same happens at the first focus, when the replay routes the kept notices of no world back to the Owner::Client pane that already shows them. The code at head avoids this: the Reset replays only when a world has the focus, into the cleared pane of that world. |
| R-57 | 9 | 10 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D9. The client routes by selection"],"skips each pane of `Owner::All`, because that pane already shows the notice with the prefix `client`"] | resolved | resolved in round 10 from fixed 9073e89fe1f05a5ef836f13dceb63020cbe087c1 | 9 |  |  | The first-focus carry skips every pane of Owner::All because it assumes that pane already shows the notice. A notice of no world is routed by layout_of(None), the default layout, but the carry routes it by the layout of the world in focus, which can be another layout. If the default layout has no all-worlds output with the same id that shows notices, and the world's layout sends notices only to an output of all worlds, then the carry skips the one pane that should get the notice, and that pane never had it. The notice from before the first list then shows in no pane of the first world until a Reset replays it. |
| R-58 | 11 | 11 | blocker | in-change | drift | ["path","crates/mu-tui/src/update.rs",1368] | fixed | 86624a39e3fe497dd915ae403596f80d9b32bb9d | 11 |  | severity? should-fix 0.60 | The notice of a stream with no output writes the notice mark `-- ` by hand, a further copy of the format that Model::notice, on_reply and text.rs already each write. The spec keys the notices on a line that starts with `-- `. |
| R-59 | 11 | 11 | should-fix | in-change | next-change | ["artifact",["specs/line-rules/spec.md","Requirement: A bad name is reported once"],"A name from a capture is bad if it is not valid, or if it is `notices`, because only the client writes that stream."] | fixed | 8d78189dc08cc54cec2f3f4b6a24ccebac0dd186 | 11 |  |  | app-1: The spec makes a name from a capture that is `notices` a bad name. No decision of design.md states it: D5 gives BadName only for a name that is not valid, and `notices` is a valid stream name by D1. The rule lives only in the code of Rule::streams.; inv-1: The spec says only the client writes the stream notices. Rule::compile refuses notices only from a capture: a fixed stream = "notices" loads, and a probe of mu world match routes a game line to notices. |
| R-60 | 11 | 11 | should-fix | in-change | next-change | ["artifact",["specs/client-layout/spec.md","Requirement: A place keeps the streams of an output"],"A `/place` of an `output` that is on the screen, with no `of all worlds` and no `showing`, SHALL keep the streams and the scope of that output."] | fixed | 8d78189dc08cc54cec2f3f4b6a24ccebac0dd186 | 11 |  |  | The requirement that a `/place` with no scope and no `showing` keeps the streams and the scope of an output on the screen has no decision. D8 covers only the rewrite through tree::Block. The code adds Change::Move and Layout::keeping, and Change::place picks Move or Place by whether the line names a selection, none of which design.md names. |
| R-61 | 11 | 11 | should-fix | in-change | next-change | ["artifact",["design.md","D5. One function decides the streams"],"It counts the line and tries `until` on the text after the hooks."] | fixed | 8d78189dc08cc54cec2f3f4b6a24ccebac0dd186 | 11 |  |  | D5 says route sees the text after the hooks. The session passes the text after the hooks with the password masked, so a pattern, a template and an error never see the password. design.md records no such deviation, and #59 and #60 build on D5 for what text the rules see. |
| R-62 | 11 | 11 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D9. The client routes by selection"],"For each stream, it finds the outputs whose `Selection` shows it. It pushes the line one time into each pane that one or more streams found."] | fixed | 86624a39e3fe497dd915ae403596f80d9b32bb9d | 11 |  | scope? deviates-design 0.69 | D9 routes each event to the panes whose Selection shows one of its streams. on_event puts a prompt only into the pane of main of its world, as its unfinished line, whatever the selections. So main keeps a role beyond its default showing: an output `log showing main` gets no prompt, and a main whose showing leaves out `main` still gets it. design.md records no such exception. |
| R-63 | 11 | 11 | should-fix | in-change | next-change | ["artifact",["design.md","D8. An output selects streams"],"The list of patterns ends where the existing `place` parser of `parse.rs` matches, or at the end of the line."] | fixed | 8d78189dc08cc54cec2f3f4b6a24ccebac0dd186 | 11 |  |  | D8 ends the list of patterns at a place or at the end of the line. The parser also ends it at a comma and a size that end the line, so `showing a, 30%` with no place reads 30% as a size. design.md does not record this third end of the list. |
| R-64 | 11 | 11 | should-fix | in-change | drift | ["path","crates/mu-core/src/rules.rs",404] | fixed | bca5208b23e035c9841307d8552034cb7668bdb9 | 11 |  |  | rules::plain decides that a line is already screen text with its own copy of the SCREEN map (control or PLACEHOLDER), while text.rs asks the map itself. If SCREEN gains a character, plain returns a line borrowed that the screen shows changed, so a pattern no longer sees the screen text. |
| R-65 | 11 | 11 | should-fix | in-change | drift | ["path","crates/mu-config/src/stream.rs",178] | fixed | bca5208b23e035c9841307d8552034cb7668bdb9 | 11 |  |  | Pattern::new restates the byte set of a block id (a-z, 0-9, -) beside BlockId::new. D8 defines a pattern as the characters of a stream name plus * and ?, so if the id set changes, a valid stream name can no longer be named by any pattern. |
| R-66 | 11 | 11 | should-fix | in-change | drift | ["path","crates/mu-config/src/layout/mod.rs",257] | fixed | bca5208b23e035c9841307d8552034cb7668bdb9 | 11 |  |  | stream.rs gives StreamName::main() for MAIN, but the name notices is built twice by a fallible StreamName::new(NOTICES), with two failure rules: the layout check returns an error, the client routes notices to no pane in silence. |
| R-67 | 11 | 11 | should-fix | in-change | drift | ["path","crates/mu-cli/src/world.rs",79] | fixed | bca5208b23e035c9841307d8552034cb7668bdb9 | 11 |  |  | run_match repeats the load of the worlds, the find by id and the not-found message of run, where D10 says it resolves the world as mu world show does. A change to the lookup or its message in one verb leaves the other behind. |
| R-68 | 11 | 11 | should-fix | in-change | drift | ["path","crates/mu-tui/src/arrange.rs",266] | fixed | 86624a39e3fe497dd915ae403596f80d9b32bb9d | 11 |  |  | Two mechanisms keep the selection of a moved output: /place sends Change::Move, which copies only the selection through Layout::keeping; arrange mode copies the whole shown step by struct update and sends Change::Place. A new field of Step is kept by one and dropped by the other. |
| R-69 | 11 | 11 | should-fix | in-change | drift | ["path","crates/mu-core/src/rules.rs",120] | fixed | bca5208b23e035c9841307d8552034cb7668bdb9 | 11 |  |  | The limit of a list of streams is STREAMS_MAX, but the messages of the rule check and of Streams::new write 8 by hand, so a change of the constant leaves both messages wrong. |
| R-70 | 11 | 11 | should-fix | in-change | drift | ["path","crates/mu-core/src/rules.rs",82] | fixed | bca5208b23e035c9841307d8552034cb7668bdb9 | 11 |  | severity? suggestion 0.82 | The range of lines is LINES_MAX, but the message of the check writes 1000 by hand beside the constant it checks against. |
| R-71 | 11 | 11 | should-fix | in-change | false-test | ["path","crates/mu-config/tests/layout.rs",1673] | fixed | bca5208b23e035c9841307d8552034cb7668bdb9 | 11 |  |  | The test is named for 'output one to eight times' but passes a layout with no output and asserts only that the error mentions main, so it cannot fail on the output-count rule. |
| R-72 | 11 | 11 | suggestion | in-change |  | ["path","crates/mu-core/src/rules.rs",176] | fixed | bca5208b23e035c9841307d8552034cb7668bdb9 | 11 |  | severity? blocker 0.63 | Rule::streams reads only the first action and returns no streams if it is not Stream. #59 adds actions to the same rules, so a rule with a highlight before its stream would lose its streams, and #59 must rewrite this lookup. |
| R-73 | 11 | 11 | suggestion | in-change |  | ["path","crates/mu-config/src/stream.rs",97] | fixed | bca5208b23e035c9841307d8552034cb7668bdb9 | 11 |  |  | The server writes the streams with Streams::to_wire and the client reads them with Streams::from_wire. Their round trip over any valid list of 1 to 8 names has only example tests. |
| R-74 | 11 | 13 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"test-coverage partial"] | accepted | ticket: none; reason: a full round 14 runs the lenses fit, test-coverage and test-honesty again, each on the files that no earlier round read |  |  |  | lens test-coverage reported partial, gaps: 1 |
| R-75 | 11 | 16 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"test-honesty partial"] | accepted | ticket: none; reason: a full round 14 runs the lenses fit, test-coverage and test-honesty again, each on the files that no earlier round read |  |  |  | lens test-honesty reported partial, gaps: 2 |
| R-76 | 12 | 12 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D9. The client routes by selection"],"That notice is a line of the stream `notices` of the world of the event, and the client does not keep it."] | fixed | 6406d0b6cf8e7f7eb35f2fa82eaefd5ec3278847 | 12 |  |  | D9 names only the notice of a stream with no output as a notice that the client does not keep. on_reply shows the notice of a refused line with show_notice and keeps it nowhere. A Reset clears the panes of each world and replays only the kept notices, so the refusal notice is lost. |
| R-77 | 12 | 12 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","Migration plan"],"A layout from before this change can have an output `main` and no new word as an id. Such a layout keeps its meaning."] | fixed | d8a10b7aa6e2256ce5476ef93c856d7ca93824fe | 12 |  |  | The migration plan says a layout from before this change that has an output `main` keeps its meaning. That is not so at head for a layout with `output` and `output as notices`. Before, the notices went only to `notices`. Now `main` also shows them by default, so they show twice, as the design's own risk states. |
| R-78 | 12 | 12 | should-fix | in-change | drift | ["path","crates/mu-core/src/rules.rs",110] | fixed | 938f895d87fc8ee3d1c7f933353f1e8b8fb9f9a4 | 12 |  | severity? suggestion 0.85 | The rule that a rule cannot name the stream notices is written twice in rules.rs: once for a fixed entry in template, once for an expanded name in Rule::streams, each with its own lower-casing and its own check before StreamName::new. |
| R-79 | 12 | 12 | should-fix | in-change | drift | ["path","crates/mu-core/src/session.rs",543] | fixed | 938f895d87fc8ee3d1c7f933353f1e8b8fb9f9a4 | 12 |  |  | on_bytes masks the password of the world in the line before route with its own copy of the masking that record_in already does, so the text the rules see and the text the journal records are masked by two code paths. |
| R-80 | 12 | 13 | should-fix | in-change | drift | ["path","crates/mu-config/src/layout/parse.rs",195] | fixed | 8205a555c8dffb749a71f06f950dfc2376a41de5 | 13 |  |  | size_next restates the tail of the line grammar (a comma, a size, the end) that line already parses, and line does not use the new comma helper. If the size tail of a line changes, the end of a showing list no longer agrees with it. |
| R-81 | 12 | 12 | should-fix | in-change | drift | ["path","crates/mu-core/tests/support/mod.rs",46] | fixed | 938f895d87fc8ee3d1c7f933353f1e8b8fb9f9a4 | 12 |  |  | world_with_rules repeats the body of resolved_on (the ResolvedWorld struct update and the WorldSetup::new call) instead of passing the rules to it, as world_joining passes its line break. A change to the fake world setup of the tests then misses the worlds with rules. |
| R-82 | 12 | 12 | should-fix | in-change | next-change | ["artifact",["design.md","D5. One function decides the streams"],"Only the client writes the stream `notices`, so the compile also refuses a fixed entry of `stream` that is `notices`, in any case."] | fixed | 938f895d87fc8ee3d1c7f933353f1e8b8fb9f9a4 | 12 |  | severity? blocker 0.68 | D5 states that only the client writes the stream notices. The rule path enforces it, but the streams of a hook pass to the journal unchecked, and Streams admits notices, so the #60 seam can write it. |
| R-83 | 12 | 12 | suggestion | in-change |  | ["artifact",["design.md","D5. One function decides the streams"],"So a capture never holds more than `lines` lines."] | fixed | 938f895d87fc8ee3d1c7f933353f1e8b8fb9f9a4 | 12 |  |  | D5 claims a capture never holds more than lines lines, for any lines from 1 to 1000 and any input. Only example tests check it, at lines 1, 2 and the default 100. |
| R-84 | 12 | 16 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"fit partial"] | accepted | ticket: none; reason: a full round 14 runs the lenses fit, test-coverage and test-honesty again, each on the files that no earlier round read |  |  |  | lens fit reported partial, gaps: 1 |
| R-85 | 13 | 13 | should-fix | in-change | next-change | ["artifact",["design.md","D8. An output selects streams"],"Each of these ends comes after at least one pattern."] | fixed | 06aaad8b035646ac792881ddcfce48887250d584 | 13 |  |  | D8 says each end of the pattern list (a place, a comma and a size, the end of the line) comes after at least one pattern. The parser at head ends the list at a place or at the end of the line with no pattern, and only the size end needs one. A parser built from D8 would read `right` of `showing right of main` as a pattern and fail at `of`, so the scenario An empty list of patterns, which names `showing`, would not hold. design.md records no such deviation. |
| R-86 | 13 | 13 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-core/src/rules.rs",18] | fixed | 8205a555c8dffb749a71f06f950dfc2376a41de5 | 13 |  |  | app-2: D5 gives each pair of rule id and error kind one report, and the hook check its own error until reset(). The code records the hook error under the pair (`hook`, BadName) in the same set as the rules. A rule id can be any string, so a rule with the id `hook` shares that pair: after a hook names the notices, the first bad name of that rule records no script error, which the requirement A bad name is reported once demands.; inv-1: RuleState::hooked records its notices error under the rule id `hook`, and a rule id is any string. A rule named `hook` and the hook check share one (id, BadName) pair, so the first error silences the other until reset. |
| R-87 | 13 | 13 | should-fix | in-change | drift | ["path","crates/mu-cli/tests/client.rs",2526] | fixed | 6cc30383249bc2e305184dd36ded38b9f1e02761 | 13 |  |  | The new helper packed in client.rs repeats the packing of the screen rows that the existing wait_notice already does inline, so the tests of the streams and the older notice tests pack the rows by two copies. |
| R-88 | 13 | 13 | should-fix | in-change | drift | ["path","crates/mu-tui/tests/update.rs",1434] | fixed | 6cc30383249bc2e305184dd36ded38b9f1e02761 | 13 |  |  | The new helper strings in update.rs has the same body as the existing helper texts in the same file. |
| R-89 | 13 | 13 | should-fix | in-change | drift | ["path","crates/mu-tui/tests/update.rs",1531] | fixed | 6cc30383249bc2e305184dd36ded38b9f1e02761 | 13 |  |  | The new helper run_line in update.rs types a text and presses Enter, the same body as the existing run_command, which only also returns the effects. |
| R-90 | 13 | 13 | should-fix | in-change | drift | ["path","crates/mu-tui/tests/update.rs",1439] | fixed | 6cc30383249bc2e305184dd36ded38b9f1e02761 | 13 |  |  | The new helper layouts in update.rs rebuilds the GetClientConfigResponse with a layout and world layouts that the existing layout_config already builds, with its own revision, echo and world-layout names. |
| R-91 | 13 | 13 | should-fix | in-change | drift | ["path","crates/mu-cli/tests/journal.rs",271] | fixed | 6cc30383249bc2e305184dd36ded38b9f1e02761 | 13 |  | severity? suggestion 0.77 | The change writes the TOML of one rule table three times: rule in journal.rs, rule in world_config.rs, and the format of game_rule in world_config.rs, while the tests share a support module. |
| R-92 | 13 | 13 | should-fix | in-change | drift | ["path","crates/mu-cli/tests/client.rs",2582] | fixed | 6cc30383249bc2e305184dd36ded38b9f1e02761 | 13 |  |  | The new helper sidebar in client.rs builds the same configuration and client as the existing test the_sidebar_layout, which does not call it. |
| R-93 | 14 | 14 | should-fix | in-change | drift | ["path","crates/mu-tui/src/model.rs",778] | fixed | fb156fd33285361b773ab0c406a61d8b8d44f891 | 14 |  | severity? suggestion 0.78 | Model::key_of in mu-tui finds the output step of an id in a layout and reads its selection, the same lookup that the private Layout::keeping in change.rs writes. Two hand-written finds of the selection of the output of an id. |
| R-94 | 14 | 14 | should-fix | in-change | drift | ["path","crates/mu-tui/src/view.rs",51] | fixed | fb156fd33285361b773ab0c406a61d8b8d44f891 | 14 |  |  | solved in view.rs builds the editor key of the world in focus by hand and reads model.editors, the lookup that Model::editor already does. The change rewrote both with the new PaneKey, so the editor key rule now has a copy outside the model. |
| R-95 | 14 | 14 | should-fix | in-change | drift | ["path","crates/mu-core/src/rules.rs",352] | fixed | fb156fd33285361b773ab0c406a61d8b8d44f891 | 14 |  |  | The rule that a capture is full is written twice in RuleState::route: as count >= lines for a running capture, and as lines > 1 with an implicit start count of 1 when a capture starts. D5 states one rule, that the capture ends when the count reaches lines. |
| R-96 | 14 | 14 | suggestion | in-change |  | ["artifact",["design.md","What enforces each rule"],"\| The writer and the parser agree on the scope and the patterns \| The property test `any_layout_reads_back`, which compares trees that hold the `Selection` \|"] | fixed | 6f9e1f610a08918dd0478ede184b800d8ce7f1c9 | 14 |  | severity? should-fix 0.74 | The table What enforces each rule names any_layout_reads_back as the guard that the writer and the parser agree on the scope and the patterns. That test builds the written layout with Layout::written, which is from_steps(write::plan(tree)), and never parses a line of text. The test that writes a step as text and parses it back, with scopes and patterns in its generator, is a_line_reads_back. |
| R-97 | 14 | 14 | suggestion | in-change |  | ["artifact",["design.md","D7. Each event has a ULID and its streams"],"The journal then calls `Overflow::commit_overflow_increment()`, which moves the id into the next millisecond, so the order holds."] | fixed | a0837537136fe0272e0f4db7d3a5515fc6e59213 | 14 |  |  | D7 relies on commit_overflow_increment to keep ids in order when the random part of one millisecond runs out. No test reaches that branch: ids_sort_in_the_order_of_seq appends at most 2000 events from a random start. |
| R-98 | 14 | 16 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"invariants partial"] | accepted | ticket: none; reason: a full round 15 runs the lens invariants again with a focus on D10 and run_match |  |  |  | lens invariants reported partial, gaps: 1 |
| R-99 | 15 | 15 | blocker | in-change | drift | ["path","crates/mu-tui/tests/update.rs",1477] | fixed | de13c109d7501230444375ec7a9da2036076de65 | 15 |  | severity? suggestion 0.63 | The change adds a third copy of the test helper block(id) in update.rs, beside the same body in app.rs and arrange.rs, while tests/support/mod.rs holds the helpers that the tests of update and view share. |
| R-100 | 15 | 15 | should-fix | deviates-design | drift | ["path","crates/mu-core/src/journal.rs",153] | fixed | 6967679f73af2d66e9eb745a7f87a1bc1d2d02d1 | 15 |  | scope? in-change 0.76 | MemoryJournal::append reads the clock twice for one event: generate() takes its own now for the time part of the id, and the event takes a second SystemTime::now() as its time, so the time in the id and time_ms on the wire can differ. |
| R-101 | 15 | 15 | should-fix | in-change | drift | ["path","crates/mu-tui/tests/update.rs",1483] | fixed | de13c109d7501230444375ec7a9da2036076de65 | 15 |  | severity? suggestion 0.87 | The new rows_of in update.rs draws an 80 by 24 screen, finds the pane of an id through solved.pane_of and collects its trimmed non-empty rows, which rect_of and block_rows in app.rs already do; it only also drops the title row. |
| R-102 | 15 | 15 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/line-rules/spec.md","Requirement: `mu world match` shows the streams of each line"],"It SHALL match each one as a line from the game of that world, with one capture state for the whole input."] | fixed | dd9b2377611cd10a558de13b33ac5119fa2280ce | 15 |  | severity? blocker 0.69 | inv-1: mu world match keeps a bare CR inside a stdin line, while the framer of the session drops each CR, so match and the server give different streams for the same game line.; inv-2: The framer cuts a game line at MAX_LINE bytes into two lines, and a capture counts both. mu world match never cuts a stdin line, so its streams and capture counts differ from the server past that limit. |
| R-103 | 15 | 15 | suggestion | in-change |  | ["path","Cargo.toml",60] | fixed | 927dadd645a6c683894916ef089352b975b76139 | 15 |  | severity? should-fix 0.80 | The change adds rand_core as a new workspace dependency and a dev-dependency of mu-core, for the unit test of the ULID overflow. The proposal lists the new workspace dependencies as fancy-regex, ulid and a glob crate, and neither D7 nor task 1.7 names rand_core. |
| R-104 | 16 | 16 | blocker | in-change | drift | ["path","crates/mu-tui/tests/arrange.rs",603] |  |  |  |  | severity? suggestion 0.82 | The new run_command in arrange.rs types a text and presses Enter, a third copy beside run_command in update.rs and enter in app.rs, while the shared support module already holds type_text and press. |
| R-105 | 16 | 16 | blocker | in-change | drift | ["path","crates/mu-tui/tests/update.rs",1462] |  |  |  |  |  | The new line_of in update.rs builds the Msg of a game line of a world, a third copy of game in arrange.rs and event/game in app.rs; it only adds the streams field. |
| R-106 | 16 | 16 | should-fix | in-change | drift | ["path","crates/mu-core/tests/scripts.rs",585] |  |  |  |  |  | The new GameHook in scripts.rs gives an action to each game line that contains a text, the body of the existing GameGag with the action as a field. GameGag is GameHook with HookAction::Gag. |
| R-107 | 16 | 16 | should-fix | in-change | drift | ["path","crates/mu-config/tests/stream.rs",197] |  |  |  |  |  | The property tests of stream.rs and layout.rs each write the same generator text of a pattern. If the characters of a pattern change, one property test keeps the old set. |
| R-108 | 16 | 16 | should-fix | in-change | drift | ["path","crates/mu-cli/tests/client.rs",2510] |  |  |  |  | severity? suggestion 0.82 | The new with_rules in client.rs repeats the body of the existing with_layout (layout_dir, attached_showing, the triple) and only adds global.toml, so with_layout is with_rules with no rules. |
| R-109 | 16 | 16 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-config/src/layout/parse.rs",213] |  |  |  |  |  | A move writes `showing at right of main`, which the parser reads as the place `at right`. Probe: mu layout place on `output as side showing at below main` wrote a file that no longer loads (unknown form at `of`). |
| R-110 | 16 | 16 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-config/src/layout/parse.rs",216] |  |  |  |  |  | A last pattern of digits on a first line reads back as a size. Probe: mu layout hide tabs rewrote `output as main showing main, notices, 30` as line 1, and the file then fails with bad size. |
| R-111 | 16 | 16 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/update.rs",1248] |  |  |  |  |  | A Reset drops the Owner::All keys from each kept notice so the replay refills the shared pane once. No test puts a kept notice in an output of all worlds through a Reset, so keeping the All key would pass. |
| R-112 | 16 | 16 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/layout.rs",260] |  |  |  |  |  | The CLI test for nine outputs passes a layout with no `main`, so the no-main error satisfies it and it cannot fail on the output-count rule. |
| R-113 | 16 | 16 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/layout.rs",186] |  |  |  |  |  | The scenario A block that is not placed yet expects `line 2` and `output`. The test asserts `main`, and the error text names the anchor, so `output` is never checked. |
| R-114 | 16 | 16 | suggestion | in-change |  | ["artifact",["design.md","D7. Each event has a ULID and its streams"],"The event takes the same time, so the time in the id is the time of the event."] |  |  |  |  |  | D7 says the time in the id is the time of the event. The ulid generator keeps the millisecond of the previous id when the clock went back or an overflow moved it ahead, so the id then holds a later time than the event. The unit test of the overflow builds that case. |

## Reviewed files

| path | content |
|---|---|
| Cargo.lock | 7f8daca8b1940a927c7536ecd04d3744e776252a |
| Cargo.toml | 0843b6d5410a05b1c9f385710afc8d0726c6e2ea |
| architecture/model.c4 | e1e57c0d14aa3fcfda301ec9ae2867776591857f |
| crates/mu-cli/src/server.rs | 6a73114f704b108eca05c778b1bd15bd00a4c221 |
| crates/mu-cli/src/world.rs | 5eb55b9a4c676dc14a469f50a96d20f4af7e8af6 |
| crates/mu-cli/tests/client.rs | f39569330acf9c8eaf9dfda333907562aad1edc4 |
| crates/mu-cli/tests/journal.rs | 46636c935301e08586fd4d0b40bd1f1e5de36ed3 |
| crates/mu-cli/tests/layout.rs | 2ea69032328ff750e4c83987b97fa9813d807bb9 |
| crates/mu-cli/tests/screen_text.rs | cce4d493ff33d33eaec8552e221a491c4be946f9 |
| crates/mu-cli/tests/support/mod.rs | 6a37812f9bf6fc3bf2ba706070ef32268dcb2fc3 |
| crates/mu-cli/tests/telemetry_engine.rs | 7a176ddee087240764cf6b4d37f5b2d71815b512 |
| crates/mu-cli/tests/workspace_boundary.rs | 4131d78041bdfbeba174f0d6445042b3313e8846 |
| crates/mu-cli/tests/world_config.rs | 990feb0ae99c1db0c5f13ee4a6e83467a93d853d |
| crates/mu-config/Cargo.toml | adc953b9f6262b59b68944dc3b6f1f330cd9d55d |
| crates/mu-config/src/ansi.rs | 06426ae82ea35faf9e6033798cd8344f03b1e607 |
| crates/mu-config/src/filter.rs | d8da1e0e0623654141003277f0e3be8976409f1b |
| crates/mu-config/src/layout/block.rs | 778486ac08745208efffe5ad9444315290231c7d |
| crates/mu-config/src/layout/builtin.rs | 45ae8a92d1d1b90f2344f0b329c506409ca36503 |
| crates/mu-config/src/layout/change.rs | f60a05aefa5287f8210d52be47080e16ada35517 |
| crates/mu-config/src/layout/mod.rs | c85916f12a1ec3f4d8d9a0db40b5c463efe93189 |
| crates/mu-config/src/layout/parse.rs | da25f5f859a061bb66440e1f2d788829e51be9ee |
| crates/mu-config/src/layout/solve.rs | 83993e07875c05fca973d7134d8895e47c079d3a |
| crates/mu-config/src/layout/step.rs | 185a7b33439e4e5dc4bdd134916cc5ea07c0f8c5 |
| crates/mu-config/src/layout/tree.rs | cc13e135672d40ad751f03b7d104678a9bdcbc6e |
| crates/mu-config/src/layout/write.rs | fca378ad4dd1b634466ad9076b4e041f50e27afd |
| crates/mu-config/src/lib.rs | 0b821309f605ec76f6b4213b84f0502dca2fc860 |
| crates/mu-config/src/stream.rs | 79c36e0bee81ce217b34c33a933cfa03c7f54e04 |
| crates/mu-config/tests/ansi.rs | 3d17cc1656211cbdaebcc9432d0d8106eceeaf34 |
| crates/mu-config/tests/filter.rs | 3a6c7c262a5aca4abfff7091f8a23135531d690e |
| crates/mu-config/tests/layout.proptest-regressions | 0d5a4f76504a01ac92c31edccf2dc7e950a8cebe |
| crates/mu-config/tests/layout.rs | 83739f75ec68d41d2f79d5c823eadd4e821ea32b |
| crates/mu-config/tests/stream.rs | 53a0d97f8f1e7f3e076426c0019fed887a1b1f99 |
| crates/mu-core/Cargo.toml | 19b583c2e2a1fd07b36875273f71549f58250b80 |
| crates/mu-core/proptest-regressions/rules.txt | 041dbee3bcca38cb61c0b7c9550ef58830eec6ec |
| crates/mu-core/src/event.rs | c8b5761e6a296f1b35a3339e06832232ea871578 |
| crates/mu-core/src/journal.rs | 7b7e75ddfdf470e2374339f216d5fc07aed6772e |
| crates/mu-core/src/lib.rs | 247cfb3f9321be4490125e59c612d97243a1500e |
| crates/mu-core/src/portal.rs | 3d4a3e2c33f17a5c8049e1b794383be7f7541ad8 |
| crates/mu-core/src/rules.rs | ee4175c9ff1799178b586075f184d7f4ffb18fe2 |
| crates/mu-core/src/script.rs | 0a8a9cdeda54fcc701d485c5973bfb69700d6786 |
| crates/mu-core/src/session.rs | 8c657b8744d6df33610c2b0d90774b418ffeb5fb |
| crates/mu-core/src/world/check.rs | 86d01e37bf8bb0979cf5eb8cd1d8814fea99ed96 |
| crates/mu-core/src/world/mod.rs | 22ecbc75ea1ee7d6048fbcae703bf1eef5577ea4 |
| crates/mu-core/src/world/resolve.rs | 40ef0b3127de66f4414493fc3ea2438745eb5cf0 |
| crates/mu-core/tests/portal.rs | cc6b59808cf3cf698b68c92e00b5273bf558c28c |
| crates/mu-core/tests/scripts.rs | ab97f85504580d16d9b7a0e0c0896e9734f6d26d |
| crates/mu-core/tests/support/mod.rs | 313e9455684794c1d136b21eaf4ffb0e8d37b16a |
| crates/mu-proto/proto/mu/v1/portal.proto | 543ef9668b0e0ecf397429c538a3a551e4aa9472 |
| crates/mu-server/src/convert.rs | 1e0162cbe843de6a0a9715dd6a04d765f49bdb24 |
| crates/mu-server/src/service.rs | 1851b2656a62c15e8aecc69508dc3c88c1d0fa09 |
| crates/mu-server/src/store.rs | f5402d56df34db9314965f6b779a5b4467ce2005 |
| crates/mu-server/tests/store.rs | 628546a1039945e8023db21dccf6ec9fee77cf2f |
| crates/mu-tui/Cargo.toml | 63614d797c4305edcbc72d4488be358b5d79b3de |
| crates/mu-tui/src/arrange.rs | 78ab7954852a9687077acfc8e733c2ed52fb020f |
| crates/mu-tui/src/config.rs | 6f95fde3a3d5357c99d3cbdd3003c88273820a76 |
| crates/mu-tui/src/convert.rs | 7fdac4479b018b1ddc6fcc33642871047f0de30d |
| crates/mu-tui/src/layouts.rs | fe685d35e8abb0337a275dfd6e94f1f1e1748804 |
| crates/mu-tui/src/lib.rs | e42022d498ccee7a365dea36ce17e4320b8029fd |
| crates/mu-tui/src/model.rs | 52e9076b90c2ed980c0d902e6d11b3a38c8d37b2 |
| crates/mu-tui/src/text.rs | f0924b687440fbe5c1a13255a4dc9b1c9c3d9551 |
| crates/mu-tui/src/update.rs | 6eb6f9fc21861dedcfdbb70d3de1424a5cb5f5ee |
| crates/mu-tui/src/view.rs | c9386fe558541847cff2eee6a69a00cb4e339cb7 |
| crates/mu-tui/tests/app.rs | d71ae26bad37c4de8b57a10fbe80e166d71a4d4b |
| crates/mu-tui/tests/arrange.rs | d0af7ab933eebd8b28b800e662529de6e82ba2b3 |
| crates/mu-tui/tests/support/mod.rs | 7fe72d5155337eacce373e6e108b6ece82dab802 |
| crates/mu-tui/tests/update.rs | 9e1f0471c0bd8e633ae1c4a78dbd6ba981738541 |
| docs/src/content/docs/architecture/crate-layout.mdx | 91df8bc1eea234411533c387dd17efaf13eefcfc |
| docs/src/content/docs/contributing/workspace.md | 29fdf59ff1e4cdfbe2b507b4bcaa30f80ff8c8a9 |
| docs/src/content/docs/decisions/layout-model.md | b12c83b283c76466e26a56894f29a169d264ca9f |
| docs/src/content/docs/guides/configuration.md | a3352be142ebffd380f4041f35e97854d71d11a6 |
| docs/src/content/docs/guides/layouts.md | a26c3194ac99f5e3cf5133644d32a57d4b96efff |
| openspec/changes/spawns/.openspec.yaml | b38535fe8ea362c909c7df3cf34b2b1224b77125 |
| openspec/changes/spawns/design.md | e8d91f1182c10f2a9edafbfd5c964419104eb22a |
| openspec/changes/spawns/proposal.md | e448df1dbcc49b6226afb0ed5e2db2760cbf6b5b |
| openspec/changes/spawns/specs/client-layout/spec.md | 9c656982677a258e78e8557730c9af3386498f0d |
| openspec/changes/spawns/specs/event-journal/spec.md | ebb2f3616778cd003c7971de45cd282126665276 |
| openspec/changes/spawns/specs/line-rules/spec.md | 78bf0d22a683e261c0c6f18edcbea2c292b16fd5 |
| openspec/changes/spawns/specs/world-sessions/spec.md | 40c4d71ed45dc9df8536a8a9a53c58b2343fcf32 |
| openspec/changes/spawns/tasks.md | sha256:be58a818f871786c9be14cced8833b359c62b30f542d6457c2a0b81606284021 |
| openspec/review.json | absent |

## Plan review (2026-11-26)

Verdict approve at head 91c8e35e2a1be320a84f3108404c69939f305fd3.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|

### Coverage matrix

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
