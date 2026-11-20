---
verdict: approve
phase: implementation
reviewed_at: 2026-11-20T04:50:19+00:00
head: 2e1c509073f1adfcbae21c47781f7b7fc03bd3bc
round: 10
next_id: 95
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: ok
  invariants: ok
  verify: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|

## Repeats

None.

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-38 | suggestion | deferred | Suggestions of the output-pane plan review: the snapshot crate question, the place of the LikeC4 components, and three property tests. None blocks the plan; they are picked up during apply or after. | https://github.com/seanb4t/mu-portal/issues/110 |
| R-39 | suggestion | deferred | Suggestions of the output-pane plan review: the snapshot crate question, the place of the LikeC4 components, and three property tests. None blocks the plan; they are picked up during apply or after. | https://github.com/seanb4t/mu-portal/issues/110 |
| R-40 | suggestion | deferred | Suggestions of the output-pane plan review: the snapshot crate question, the place of the LikeC4 components, and three property tests. None blocks the plan; they are picked up during apply or after. | https://github.com/seanb4t/mu-portal/issues/110 |
| R-41 | suggestion | deferred | Suggestions of the output-pane plan review: the snapshot crate question, the place of the LikeC4 components, and three property tests. None blocks the plan; they are picked up during apply or after. | https://github.com/seanb4t/mu-portal/issues/110 |
| R-42 | suggestion | deferred | Suggestions of the output-pane plan review: the snapshot crate question, the place of the LikeC4 components, and three property tests. None blocks the plan; they are picked up during apply or after. | https://github.com/seanb4t/mu-portal/issues/110 |
| R-74 | should-fix | deferred | Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. | https://github.com/seanb4t/mu-portal/issues/112 |
| R-86 | suggestion | deferred | Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. | https://github.com/seanb4t/mu-portal/issues/112 |
| R-87 | suggestion | deferred | Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. | https://github.com/seanb4t/mu-portal/issues/112 |
| R-88 | suggestion | deferred | Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. | https://github.com/seanb4t/mu-portal/issues/112 |
| R-89 | suggestion | deferred | Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. | https://github.com/seanb4t/mu-portal/issues/112 |
| R-90 | suggestion | deferred | Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. | https://github.com/seanb4t/mu-portal/issues/112 |
| R-91 | suggestion | deferred | Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. | https://github.com/seanb4t/mu-portal/issues/112 |
| R-92 | suggestion | deferred | Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. | https://github.com/seanb4t/mu-portal/issues/112 |
| R-93 | suggestion | deferred | Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. | https://github.com/seanb4t/mu-portal/issues/112 |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 93eb267ca4f4911ab11465de46b166c46c8101ae | baa3258df38d8f5ed8420c21e301822613bb2d5a6832c48857a021ead002704a | changes-requested | 43 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 7ee420e9caf96408526a14bd9aed95ae26367271 | 7f671291c7e74b1a2af3f534e26fdd6ab0d36e2fc20256bdba31b243b644ee81 | approve-with-notes | 22 | 0 | 4 | 32 | 0 |  |
| 3 | plan | follow-up | 53713dbcb396f073d2a1b4a97adadd15134a2ef2 | fee4c3e1ee7750f065f6746466afc216d35cc14e9daca6b3f4ca1c7f6512cd27 | approve-with-notes | 0 | 22 | 1 | 3 | 0 |  |
| 4 | plan | follow-up | 113b6ea10c1388a3d1e3549a07c79216b69c775c | 26a2a7bf3d7dfd77592b193487a93303d7762b30c0c0c23076761a8738f19beb | approve-with-notes | 2 | 22 | 0 | 1 | 0 |  |
| 5 | implementation | full | 2586c67517d5ac968032115f411914ffdd1157ff | 452e9d833907f00e8ab0d846fa90b1a42bfc7cf98ff6bd495d1cf34ada2f204c | approve-with-notes | 26 | 0 | 0 | 0 | 0 |  |
| 6 | implementation | follow-up | 78d4d37b8d17c126435001926b2e379aadaf6268 | 2b491780e6c49a3253d354def358fd699fb9cd3edeb8e52b8ce98da2692941ff | approve-with-notes | 0 | 0 | 1 | 17 | 0 |  |
| 7 | implementation | follow-up | a791882b38af41f3e77b40bf42c73008d32b396e | 504cf63a9cc562d28c9a396896ca15987d375e3e56b99bc600d2620894c93bcb | approve-with-notes | 0 | 0 | 1 | 0 | 0 |  |
| 8 | implementation | follow-up | b3c4b591cffba229bb747c298bb1004aaccf8a6a | 504cf63a9cc562d28c9a396896ca15987d375e3e56b99bc600d2620894c93bcb | approve | 0 | 0 | 0 | 1 | 0 |  |
| 9 | implementation | follow-up | cd59fe6cbe5c799e761ac3c0c4a6030f4a50e0d1 | 1aa4a87e2e77b0f4a3bec1dcff688f986fc2f01395ac9530f0b7186ebbac9f10 | approve-with-notes | 1 | 0 | 0 | 0 | 0 |  |
| 10 | implementation | follow-up | 2e1c509073f1adfcbae21c47781f7b7fc03bd3bc | 1aa4a87e2e77b0f4a3bec1dcff688f986fc2f01395ac9530f0b7186ebbac9f10 | approve | 0 | 0 | 0 | 1 | 0 |  |

## Judge

judge: 0 questions, 0.000 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A long line wraps at a space | crates/mu-tui/tests/wrap.rs: long_line_wraps_at_a_space | covered | pass |
| A word wider than the pane | crates/mu-tui/tests/wrap.rs: word_wider_than_the_pane | covered | pass |
| A wide grapheme at the end of a row | crates/mu-tui/tests/pane.rs: wide_grapheme_at_the_end_of_a_row | covered | pass |
| An empty line | crates/mu-tui/tests/wrap.rs: empty_line_takes_a_row | partial | pass |
| A grapheme wider than the pane | crates/mu-tui/tests/wrap.rs: grapheme_wider_than_the_pane | partial | pass |
| At the bottom | crates/mu-tui/tests/pane.rs: follows_at_the_bottom | covered | pass |
| Scrolled back | crates/mu-tui/tests/pane.rs: holds_when_scrolled_back | covered | pass |
| Scrolled back by one row | crates/mu-tui/tests/pane.rs: holds_when_scrolled_back_one_row | covered | pass |
| Back at the bottom | crates/mu-tui/tests/pane.rs: follows_again_at_the_bottom | covered | pass |
| A narrower pane | crates/mu-tui/tests/pane.rs: narrower_pane_keeps_the_top_line | covered | pass |
| A resize at the bottom | crates/mu-tui/tests/pane.rs: resize_at_the_bottom | covered | pass |
| The oldest line goes | crates/mu-tui/tests/store.rs: oldest_line_goes | covered | pass |
| The scroll position was on a dropped line | crates/mu-tui/tests/pane.rs: anchor_on_a_dropped_line | covered | pass |
| A selection on a dropped line | crates/mu-tui/tests/pane.rs: selection_on_a_dropped_line | covered | pass |
| A selection with both ends on dropped lines | crates/mu-tui/tests/pane.rs: selection_cleared_when_both_ends_drop | covered | pass |
| An empty pane | crates/mu-tui/tests/store.rs: empty_pane_trims | partial | pass |
| Halfway | crates/mu-tui/tests/pane.rs: scroll_bar_halfway | covered | pass |
| All rows fit | crates/mu-tui/tests/pane.rs: no_scroll_bar_when_rows_fit | covered | pass |
| Exactly as many rows as the pane | crates/mu-tui/tests/pane.rs: no_scroll_bar_when_rows_equal_the_height | covered | pass |
| The scroll bar takes a column | crates/mu-tui/tests/pane.rs: scroll_bar_takes_a_column | covered | pass |
| Rows that fit only at the full width | crates/mu-tui/tests/pane.rs: no_scroll_bar_when_rows_fit_only_at_full_width | covered | pass |
| A red word that wraps | crates/mu-tui/tests/pane.rs: style_follows_its_text_across_a_wrap | covered | pass |
| Replaced by the finished line | crates/mu-tui/tests/pane.rs: unfinished_line_replaced_by_line | covered | pass |
| Replaced by a new unfinished line | crates/mu-tui/tests/pane.rs: unfinished_line_replaced_by_unfinished_line | covered | pass |
| A selection across a wrapped line | crates/mu-tui/tests/pane.rs: selection_copies_game_lines | covered | pass |
| The selection shows | crates/mu-tui/tests/pane.rs: selection_shows_in_reverse | covered | pass |
| The second cell of a wide grapheme | crates/mu-tui/tests/pane.rs: second_cell_of_a_wide_grapheme | covered | pass |
| Past the end of a line | crates/mu-tui/tests/pane.rs: past_the_end_of_a_line | covered | pass |
| No position | crates/mu-tui/tests/pane.rs: no_position | covered | pass |
| A profile that adds code point widths | crates/mu-tui/tests/pane.rs: cells_take_the_learned_width | covered | pass |
| The default profile | crates/mu-tui/tests/pane.rs: cells_take_the_default_width | covered | pass |
| An image at two widths | crates/mu-tui/tests/pane.rs: image_rows_do_not_wrap | covered | pass |
| Placeholder cells | crates/mu-tui/tests/pane.rs: image_placeholder_cells | covered | pass |
| An image id above 255 | crates/mu-tui/tests/pane.rs: image_id_above_255 | covered | pass |
| The size limits of an image | crates/mu-tui/tests/pane.rs: image_rows_capped_at_297 | covered | pass |
| An image wider than the pane | crates/mu-tui/tests/pane.rs: image_wider_than_the_pane | covered | pass |
| No kitty graphics | crates/mu-tui/tests/pane.rs: image_url_fallback | covered | pass |
| A URL with control characters | crates/mu-tui/tests/pane.rs: url_with_control_characters | covered | pass |
| The copy of an image | crates/mu-tui/tests/pane.rs: image_copies_its_url | covered | pass |
| The first view | crates/mu-tui/tests/app.rs: first_view | covered | pass |
| The order of the worlds | crates/mu-tui/tests/app.rs: world_order | covered | pass |
| No worlds | crates/mu-tui/tests/app.rs: no_worlds | covered | pass |
| Prefixes | crates/mu-tui/tests/app.rs: line_prefixes | covered | pass |
| Control characters | crates/mu-tui/tests/app.rs: control_characters_are_replaced | covered | pass |
| Colors | crates/mu-tui/tests/app.rs: colors | covered | pass |
| Other escape sequences | crates/mu-tui/tests/app.rs: other_escape_sequences | covered | pass |
| A color across two lines | crates/mu-tui/tests/app.rs: color_across_two_lines | covered | pass |
| Indexed and 24-bit colors | crates/mu-tui/tests/app.rs: indexed_and_24_bit_colors | covered | pass |
| A line in two reads | crates/mu-cli/tests/client.rs: line_in_two_reads | covered | pass |
| A prompt with no line end | crates/mu-tui/tests/app.rs: prompt_with_no_line_end | covered | pass |
| A read that ends inside an escape sequence | crates/mu-tui/tests/app.rs: read_ends_inside_an_escape | covered | pass |
| A prompt does not carry its style | crates/mu-tui/tests/app.rs: prompt_does_not_carry_its_style | covered | pass |
| An open OSC | crates/mu-tui/tests/app.rs: open_osc | covered | pass |
| A reset | crates/mu-tui/tests/app.rs: reset_clears_each_pane | covered | pass |
| Back and forward | crates/mu-tui/tests/app.rs: back_and_forward | covered | pass |
| A new line while scrolled back | crates/mu-tui/tests/app.rs: new_line_while_scrolled_back | covered | pass |
| Enter returns to the bottom | crates/mu-tui/tests/app.rs: enter_returns_to_the_bottom | covered | pass |
| The mouse wheel | crates/mu-tui/tests/app.rs: mouse_wheel_scrolls_three_rows | covered | pass |
| A pane of one row | crates/mu-tui/tests/app.rs: page_up_in_a_pane_of_one_row | covered | pass |
| A pane of no rows | crates/mu-tui/tests/app.rs: page_keys_in_a_pane_of_no_rows | covered | pass |
| Back to a scrolled world | crates/mu-tui/tests/app.rs: back_to_a_scrolled_world | covered | pass |
| Copy by drag | crates/mu-cli/tests/client.rs: copy_by_drag | covered | pass |
| A click clears | crates/mu-tui/tests/app.rs: click_clears_selection | covered | pass |
| A terminal that adds code point widths | crates/mu-cli/tests/client.rs: probe_adds_code_point_widths | covered | pass |
| A terminal that does not answer | crates/mu-cli/tests/client.rs: probe_with_no_answer | covered | pass |
| A late answer | crates/mu-cli/tests/client.rs: late_answer | covered | pass |
| The modes at exit | crates/mu-cli/tests/client.rs: modes_at_exit | covered | pass |
| Mode 2027 was already on | crates/mu-cli/tests/client.rs: mode_2027_already_on | covered | pass |

### Not run

None.

## Verification runs

- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=copy_by_drag)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=late_answer)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=line_in_two_reads)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=mode_2027_already_on)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=modes_at_exit)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=probe_adds_code_point_widths)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=probe_with_no_answer)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=back_and_forward)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=back_to_a_scrolled_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=click_clears_selection)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=color_across_two_lines)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=colors)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=control_characters_are_replaced)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=enter_returns_to_the_bottom)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=first_view)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=indexed_and_24_bit_colors)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=line_prefixes)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=mouse_wheel_scrolls_three_rows)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=new_line_while_scrolled_back)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=no_worlds)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=open_osc)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=other_escape_sequences)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=page_keys_in_a_pane_of_no_rows)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=page_up_in_a_pane_of_one_row)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=prompt_does_not_carry_its_style)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=prompt_with_no_line_end)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=read_ends_inside_an_escape)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=reset_clears_each_pane)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=world_order)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=anchor_on_a_dropped_line)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=cells_take_the_default_width)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=cells_take_the_learned_width)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=follows_again_at_the_bottom)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=follows_at_the_bottom)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=holds_when_scrolled_back)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=holds_when_scrolled_back_one_row)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=image_copies_its_url)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=image_id_above_255)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=image_placeholder_cells)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=image_rows_capped_at_297)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=image_rows_do_not_wrap)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=image_url_fallback)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=image_wider_than_the_pane)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=narrower_pane_keeps_the_top_line)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=no_position)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=no_scroll_bar_when_rows_equal_the_height)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=no_scroll_bar_when_rows_fit)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=no_scroll_bar_when_rows_fit_only_at_full_width)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=past_the_end_of_a_line)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=resize_at_the_bottom)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=scroll_bar_halfway)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=scroll_bar_takes_a_column)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=second_cell_of_a_wide_grapheme)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=selection_cleared_when_both_ends_drop)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=selection_copies_game_lines)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=selection_on_a_dropped_line)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=selection_shows_in_reverse)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=style_follows_its_text_across_a_wrap)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=unfinished_line_replaced_by_line)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=unfinished_line_replaced_by_unfinished_line)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=url_with_control_characters)'`
- exit 0: `cargo nextest run -p mu-tui --test pane --locked -E 'test(=wide_grapheme_at_the_end_of_a_row)'`
- exit 0: `cargo nextest run -p mu-tui --test store --locked -E 'test(=empty_pane_trims)'`
- exit 0: `cargo nextest run -p mu-tui --test store --locked -E 'test(=oldest_line_goes)'`
- exit 0: `cargo nextest run -p mu-tui --test wrap --locked -E 'test(=empty_line_takes_a_row)'`
- exit 0: `cargo nextest run -p mu-tui --test wrap --locked -E 'test(=grapheme_wider_than_the_pane)'`
- exit 0: `cargo nextest run -p mu-tui --test wrap --locked -E 'test(=long_line_wraps_at_a_space)'`
- exit 0: `cargo nextest run -p mu-tui --test wrap --locked -E 'test(=word_wider_than_the_pane)'`

## Dropped

None.

## Lens notes

- verify: R-94: the Reset now pushes the kept notices again only when the pane in focus is not the notices pane. A world pane is never PaneId::NOTICES, because PaneId::NOTICES is PaneId(0) and next_pane starts at 1, so the guard cannot skip a world pane.
- verify: R-94: with no world, the notices pane is not cleared by the Reset and keeps its notices, so each notice shows once. With a world in focus, the Reset clears the world pane and the notices come back once there.
- verify: The test notice_with_no_world_shows_once_after_a_reset renders Model::new([]) on a 20 by 4 TestBackend after a Reset and counts the rows that read -- note. Before the fix, the pane would hold two such rows, both on the screen, so the test fails on the old code.
- verify: The doc comment of Model::notice (model.rs 186-187) matches the new behaviour: the line comes back in the pane of the world in focus after a Reset.
- verify: The hunks of openspec/changes/output-pane/review.md in diff-hunks.txt were not read, because this agent never reads review.md.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D3. The line store follows the output-pane decision"],"The store keeps lines in chunks of 4096 lines, the size that the spike measured."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  | severity? should-fix 0.61 | D3 trims only whole chunks of 4096 lines. A pane with a capacity of 1000 then cannot drop just line 1 when line 1001 arrives. The scenario 'The oldest line goes' and the rule 'keep at most its capacity' need a drop of single lines, and the design has none. |
| R-2 | 1 | 2 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D4. SGR to styles at the arrival of a line, with a parser from a crate"],"The SGR state of each world lasts from one game line to the next."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | app-2: No decision says how the text of a prompt event is converted, or whether it changes the SGR state of its world. The framer gives the partial line as a prompt and keeps it, so the finished line parses the same bytes again. If the prompt changes the state, the line starts in the wrong style. A prompt can also end inside an escape sequence and leave the parser in the middle of it.; inv-2: No rule says the escape parser starts each event in the ground state. A prompt ends mid-sequence at a read boundary and the framer resends the whole line; a parser that persists eats the first char of that line, and an open OSC swallows every later line. |
| R-3 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D8. The width probe runs before the first frame, and the test test helper answers it"],"It answers DA1 at once and answers each cursor position query from the cursor of the screen."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | 'The modes at exit' needs a terminal that reports mode 2027 as known and off, so that the client turns it on. The test helper of D8 answers DA1 and cursor position queries only, and its variants are a width table and no answer. No variant answers DECRQM 2027. |
| R-4 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D8. The width probe runs before the first frame, and the test test helper answers it"],"If the probe turned mode 2027 on, the client turns it off at exit. The panic hook restores it too."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  | severity? should-fix 0.76 | app-4: No decision turns mouse capture off at exit or in the panic hook. D8 restores only mode 2027, and D6 only turns capture on. Today run.rs restores the terminal with ratatui::restore alone, which does not turn mouse capture off.; fit-6: D8 and task 9.4 extend 'the panic hook', but today the hook is ratatui's own from try_init, which restores only raw mode and the alternate screen; the design does not say how it adds to it. |
| R-5 | 1 | 2 | blocker | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: Lines show their source"],"The client SHALL show a line from the game with the styles of its SGR sequences, and drop each other escape sequence."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | The modified requirement drops the rule that ESC in a game line shows as U+FFFD, yet task 5.2 requires the unchanged property test no_control_character_is_drawn to pass; its oracle maps ESC to U+FFFD for game lines, and vte drops a lone ESC. |
| R-6 | 1 | 6 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","What enforces each rule"],"`cargo deny` and the workspace lint in CI"] | resolved | resolved in round 6 from fixed f8d93fe7cb885d737fbbf123ee23e6ae7d6e1730 |  |  | The rule table names cargo deny and a workspace lint as the guard of third-party crates. At head, the guard is the boundary test, with a list of the allowed crates of each member, and mu-tui lists only six. Each new crate fails workspace_boundary until it joins that list: the parser, insta, base64, Termina and ratatui-image. Tasks 2.4, 5.1 and 8.1 check only cargo deny. |
| R-7 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["tasks.md","1. Sources and the backend"],"the test `spike_crates_have_decision_pages` passes with the page of task 2.1"] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | Task 1.3 relies on spike_crates_have_decision_pages passing with the page of task 2.1. That page is an .mdx file, and the test reads only .md pages. So no page names spikes/terminal-backend, and the test fails. |
| R-8 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["tasks.md","5. Colors"],"Copy the escape sequences of the six transcripts in `crates/mu-core/tests/fixtures/telnet` into `crates/mu-tui/tests/data/` as game lines."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | Task 5.3 copies 'the escape sequences of the six transcripts'. The transcripts are hex of raw telnet reads, and the Evennia reads are MCCP-compressed. Only aresmush and rhostmush hold an ESC byte in plain hex. pennmush, tinymux-latin1 and tinymux-utf8 hold none, so the TinyMUX colors of the proposal have no transcript evidence. transcript_lines_convert passes trivially on those files. |
| R-9 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","D4. SGR to styles at the arrival of a line, with a parser from a crate"],"A control character that remains becomes U+FFFD, and a tab becomes a space."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | D4 restates the control-character rule of `clean` for game text, so the game path and the player and system path get two copies of one rule. |
| R-10 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. An unfinished line is state of the pane, not a line of the store"],"A selection cannot start in the unfinished line."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | D5 says a selection cannot start in the unfinished line, and no task creates the check or its test. That row is not in the store, so a press on it has no line id to map to. |
| R-11 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/app.rs",151] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | Today a Reset clears the lines of every world. No requirement or task says that a Reset also clears the unfinished line, the anchor, the selection and the SGR state of each world, so a stale prompt or color can survive a server restart. |
| R-12 | 1 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/output-pane/spec.md","Requirement: The pane follows new lines at the bottom"],"If the newest line is on the screen, a new line SHALL scroll the pane, so the newest line stays on the last row."] | resolved | resolved in round 8 from fixed 026c9b15307f9d9918d01eae7f9149d0e5da51e8 |  |  | The spec follows new lines whenever the newest line is on the screen, but D3 follows only a pane with no anchor. A pane scrolled back by one row still shows the newest line, and nothing says the anchor clears there or after PageDown reaches the bottom. |
| R-13 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: Keys and the mouse scroll the pane"],"PageUp SHALL scroll the pane back by its height less one row, and PageDown SHALL scroll it forward by the same amount."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  | severity? blocker 0.80 | PageUp scrolls by the height less one row. A pane of 1 row scrolls by 0, and a 2-row terminal gives the pane 0 rows, where a u16 subtraction underflows. No rule bounds the step. |
| R-14 | 1 | 3 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","3. The line store"],"Add a property test for any line and any width from 1 to 300."] | resolved | resolved in round 3 from fixed 43d8406742f6886ae58ba7d5b4bf6a37744e3054 |  |  | inv-14: The wrap property covers widths 1 to 300, but a pane of 1 column that shows the scroll bar leaves a text width of 0, and a terminal can shrink to 0 columns. No test passes width 0, where a wrap that places at least one grapheme per row can loop or panic.; inv-4: The wrap invariant has no rule for a grapheme wider than the area. At width 1 a CJK grapheme, or at width 5 a family emoji with a learned width of 8, fits no row, so no wrap keeps every byte with no row wider than the width. |
| R-15 | 1 | 4 | should-fix | in-change | false-test | ["artifact",["specs/output-pane/spec.md","Requirement: The scroll bar shows the position"],"If all rows fit, the pane SHALL show no scroll bar."] | resolved | resolved in round 4 from fixed c0afe377089ad45535ac113aea9fa7aa0696cebc |  |  | The scroll bar scenarios use 3 lines and 100 lines in 10 rows. No test passes rows equal to the height, nor the case where the bar column makes the rows wrap past the height, so the bar can flip on and off with each render. |
| R-16 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: The client restores the terminal"],"it SHALL turn off mouse capture and each mode that it turned on, mode 2027 included, and leave the alternate screen."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | The client turns off only the modes it turned on, but the only test gives a terminal where the client turned mode 2027 on. No test passes a terminal where 2027 was already on, where the client must write no reset at exit. |
| R-17 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. The width probe runs before the first frame, and the test test helper answers it"],"The probe waits at most 500 milliseconds for the DA1 reply."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | The probe waits at most 500 ms, then the event loop starts. No rule says what happens to a reply that arrives after the wait; a late cursor report or XTVERSION reply can reach the input line as keys and go to the game on Enter. |
| R-18 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["dependency",["ratatui-core@0.1.2","src/buffer/cell.rs"],31] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | The pane forces the learned width with ForcedWidth, which holds a NonZeroU16, yet no rule bounds a learned width to 1 or more. A class that a terminal reports as 0 wide, or a zero-width grapheme, has no ForcedWidth value. |
| R-19 | 1 | 2 | should-fix | in-change | next-change | ["artifact",["design.md","What enforces each rule"],"\| No text of a game sends a sequence to the terminal \| the scenarios \\"Control characters\\" and \\"Other escape sequences\\" \|"] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | The rule that no text sends a sequence to the terminal is enforced only for game lines. The URL fallback row and the image name are text that #13 will take from the game, and no task passes them through clean. |
| R-20 | 1 | 6 | should-fix | in-change | next-change | ["artifact",["specs/output-pane/spec.md","Requirement: An image is a line of its own"],"The placeholder SHALL carry its row mark and its column mark, and its foreground color SHALL be the image id."] | resolved | resolved in round 6 from fixed 835b0c3afa58bcfe9c91cb80b1b9c177051aa70c |  |  | The placeholder test uses image id 7. The id is carried in the foreground color, so ids above 255 need 24-bit color and ids above 2^24 need the third diacritic; no test passes either edge, nor id 0. |
| R-21 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D3. The line store follows the output-pane decision"],"A trim drops whole chunks from the front."] | duplicate | R-1 |  |  | A trim of whole chunks of 4096 lines cannot hold exactly the capacity: at capacity 1000 or 100000 the only results keep capacity+1 lines or drop a whole chunk, so the scenario The oldest line goes cannot hold. |
| R-22 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/output-pane/spec.md","Requirement: An image is a line of its own"],"An image line SHALL take a fixed number of rows at every width, and the pane SHALL NOT wrap it."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | An image line is never wrapped, and a row holds no cell past the right edge, yet no rule says what a pane narrower than the image columns draws. An image of up to 297 columns in an 80-column pane breaks one of the two invariants. |
| R-23 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/output-pane/spec.md","Requirement: An image is a line of its own"],"An image SHALL take at most 297 rows and 297 columns."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | The image size clamp is tested only with 400 rows. No test passes 297 or 298 rows, any column count above 297, or an image of 0 rows or 0 columns, which would give a line with no row for the anchor. |
| R-24 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["tasks.md","3. The line store"],"The rows must keep every byte of the line in order. No row can be wider than the width."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | The wrap property admits a wrap that gives an empty line zero rows, since zero rows keep every byte of an empty line. No test checks that a blank game line takes one row, and an anchor on byte 0 of it then holds no row. |
| R-25 | 1 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/output-pane/spec.md","Requirement: The pane drops its oldest lines"],"A scroll position on a dropped line SHALL move to the oldest line that the pane keeps."] | resolved | resolved in round 3 from fixed 43d8406742f6886ae58ba7d5b4bf6a37744e3054 |  |  | Only the scroll anchor has a rule for a dropped line. A selection whose start is on a dropped line, or a mouse position from a row map rendered before a trim, has no rule, so the copy text reads a line that the store no longer holds. |
| R-26 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/output-pane/spec.md","Requirement: A selection copies game lines"],"A selection SHALL run from one position to another, where a position is a line and a byte in that line."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  | severity? blocker 0.65 | No rule says a mouse position maps to a grapheme boundary. A press on the second cell of a wide or forced-width grapheme, past the end of a line or on the scroll bar column has no defined byte, and a Rust slice at a non-char boundary panics. |
| R-27 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["specs/output-pane/spec.md","Requirement: A resize keeps the top line"],"**WHEN** the pane becomes narrower"] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | The WHEN of the scenario 'A resize at the bottom' names no width, so a test cannot know the size it resizes to. |
| R-28 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["specs/output-pane/spec.md","Requirement: The pane drops its oldest lines"],"**WHEN** new lines arrive and the pane drops that line"] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | The WHEN of the scenario 'The scroll position was on a dropped line' says 'new lines arrive' with no count or content, so the drop it needs is not defined. |
| R-29 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["specs/terminal-client/spec.md","Requirement: The client measures character widths at startup"],"**GIVEN** a terminal that answers the queries as a terminal that adds the width of each code point"] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | The GIVEN of 'A terminal that adds code point widths' describes the terminal by its behaviour and gives no concrete width table, so the expected column 9 cannot be set up from it. |
| R-30 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","2. The architecture and the skeleton"],"Make sure that every test in `crates/mu-tui/tests/app.rs` passes"] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | Scenario 'The order of the worlds' is modified but no task names its test; only blanket 'every test passes' covers it. |
| R-31 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","4. The output pane widget"],"Make sure that every test in `crates/mu-tui/tests/app.rs` and `crates/mu-cli/tests/client.rs` passes"] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | Scenario 'No worlds' has no task that names its test; only a blanket pass of app.rs covers it. |
| R-32 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","2. The architecture and the skeleton"],"Point the tests in `crates/mu-tui/tests/app.rs` at the new API, and keep each assertion."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | Scenario 'Prefixes' has no named test; the move and the pane swap could drop its '> ' and '-- ' assertions unseen. |
| R-33 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D2. The skeleton: TEA with an Action enum and widgets by pane id"],"`ScrollPage(Up\|Down)`, `ScrollRows(n)`, `ScrollToBottom`"] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | D2 adds the action ScrollToBottom, but no key maps to it and no requirement needs it. Enter returns to the bottom through Submit. |
| R-34 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D7. A drag selects, and the release copies through OSC 52"],"The base64 encoding comes from a crate, under the rule of reuse. Task 8.1 names it."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | D7 writes the OSC 52 sequence by hand with a new base64 crate, while both candidate backends already have an OSC 52 clipboard command. |
| R-35 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D3. The line store follows the output-pane decision"],"The scroll bar uses lines, not rows, because the pane keeps no index of rows for the whole store."] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | The scroll bar of D3 and task 4.2 does not name ratatui's Scrollbar widget, whose state takes a content length and a position in lines; the spike drew its own bar. |
| R-36 | 1 | 1 | suggestion | in-change |  | ["path","docs/src/content/docs/contributing/workspace.md",58] | duplicate | R-6 |  |  | Tasks that add third-party crates to mu-tui follow only half of the documented wiring: none adds the crate to ALLOWED_CRATES of the boundary test. |
| R-37 | 1 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. The width probe runs before the first frame, and the test test helper answers it"],"With no reply, every class keeps the `unicode-width` value, and an image line shows its URL."] | resolved | resolved in round 3 from fixed 43d8406742f6886ae58ba7d5b4bf6a37744e3054 |  | severity? blocker 0.72 | The pane segments graphemes and measures widths, and D8 names unicode-width values, but no decision or task names unicode-segmentation or unicode-width, which both spikes used. |
| R-38 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D10. Tests: update, TestBackend snapshots and the binary"],"Snapshots of the rendered buffer use a snapshot crate, `insta` being the candidate, named in task 2.4 after a read of its documentation."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/110; reason: Suggestions of the output-pane plan review: the snapshot crate question, the place of the LikeC4 components, and three property tests. None blocks the plan; they are picked up during apply or after. |  |  | D10 adds insta snapshots beside explicit cell asserts, while the existing tests assert rows and ratatui's TestBackend already has assert_buffer_lines; the design names neither. |
| R-39 | 1 | 1 | suggestion | in-change |  | ["artifact",["proposal.md","What Changes"],"with a LikeC4 component view of `mu-tui` and a dynamic view of one key press"] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/110; reason: Suggestions of the output-pane plan review: the snapshot crate question, the place of the LikeC4 components, and three property tests. None blocks the plan; they are picked up during apply or after. |  |  | The component view 'of mu-tui' does not say where its components go; the model puts runtime parts under the system element (mu.server), and muTui is a crate in the crate-layout copy of the allowlist. |
| R-40 | 1 | 1 | suggestion | in-change |  | ["artifact",["specs/output-pane/spec.md","Requirement: A resize keeps the top line"],"After a resize, the top row SHALL be the row that holds that byte, in the new wrap."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/110; reason: Suggestions of the output-pane plan review: the snapshot crate question, the place of the LikeC4 components, and three property tests. None blocks the plan; they are picked up during apply or after. |  |  | The anchor promises that a resize keeps the top line at any width; a round trip from width A to B and back to A should give the same top row, over unbounded widths and lines. |
| R-41 | 1 | 1 | suggestion | in-change |  | ["path","crates/mu-tui/tests/app.rs",373] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/110; reason: Suggestions of the output-pane plan review: the snapshot crate question, the place of the LikeC4 components, and three property tests. None blocks the plan; they are picked up during apply or after. |  |  | Each world has one pane id, and a world can appear through any event. The existing property focus_is_always_valid walks random steps but checks only the focus, not that each world has a pane state. |
| R-42 | 1 | 1 | suggestion | in-change |  | ["artifact",["specs/output-pane/spec.md","Requirement: A selection copies game lines"],"The copy text of a selection SHALL be the selected text of each line, with one LF between lines, not the wrapped rows."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/110; reason: Suggestions of the output-pane plan review: the snapshot crate question, the place of the LikeC4 components, and three property tests. None blocks the plan; they are picked up during apply or after. |  |  | The copy text should not depend on the drag direction; a drag from B up to A must give the same text as from A to B, and the scenario tests one direction only. |
| R-43 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D9. The image path: kitty placeholders, fed only by tests and an example"],"Before code depends on this design, task 10.1 checks `ratatui-image`"] | resolved | resolved in round 2 from fixed 2eee1a80af222ddce34c96e88dd0880f86a59658 |  |  | Task 10.2 adds placeholder cells to the pane before task 10.1 decides whether ratatui-image renders them, so it assumes the answer that D9 leaves open. |
| R-44 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An empty line"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'An empty line' has no matrix row |
| R-45 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A grapheme wider than the pane"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'A grapheme wider than the pane' has no matrix row |
| R-46 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Scrolled back by one row"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'Scrolled back by one row' has no matrix row |
| R-47 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Back at the bottom"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'Back at the bottom' has no matrix row |
| R-48 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A selection on a dropped line"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'A selection on a dropped line' has no matrix row |
| R-49 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Exactly as many rows as the pane"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'Exactly as many rows as the pane' has no matrix row |
| R-50 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The scroll bar takes a column"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'The scroll bar takes a column' has no matrix row |
| R-51 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The second cell of a wide grapheme"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'The second cell of a wide grapheme' has no matrix row |
| R-52 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Past the end of a line"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'Past the end of a line' has no matrix row |
| R-53 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"No position"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'No position' has no matrix row |
| R-54 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An image id above 255"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'An image id above 255' has no matrix row |
| R-55 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The size limits of an image"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'The size limits of an image' has no matrix row |
| R-56 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An image wider than the pane"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'An image wider than the pane' has no matrix row |
| R-57 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A URL with control characters"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'A URL with control characters' has no matrix row |
| R-58 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A read that ends inside an escape sequence"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'A read that ends inside an escape sequence' has no matrix row |
| R-59 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A prompt does not carry its style"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'A prompt does not carry its style' has no matrix row |
| R-60 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An open OSC"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'An open OSC' has no matrix row |
| R-61 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A reset"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'A reset' has no matrix row |
| R-62 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A pane of one row"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'A pane of one row' has no matrix row |
| R-63 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A pane of no rows"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'A pane of no rows' has no matrix row |
| R-64 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A late answer"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'A late answer' has no matrix row |
| R-65 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Mode 2027 was already on"] | refuted | Each of these scenarios was added in round 1 fixes and tasks.md names its test (for example empty_line_takes_a_row in 3.2, late_answer in 9.3); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'Mode 2027 was already on' has no matrix row |
| R-66 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A selection with both ends on dropped lines"] | refuted | Both scenarios were added in the round 3 fix and task 4.2 names their tests (selection_cleared_when_both_ends_drop, no_scroll_bar_when_rows_fit_only_at_full_width); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'A selection with both ends on dropped lines' has no matrix row |
| R-67 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Rows that fit only at the full width"] | refuted | Both scenarios were added in the round 3 fix and task 4.2 names their tests (selection_cleared_when_both_ends_drop, no_scroll_bar_when_rows_fit_only_at_full_width); the coverage matrix predates them because a follow-up round runs no coverage lens. |  |  | delta scenario 'Rows that fit only at the full width' has no matrix row |
| R-68 | 5 | 6 | should-fix | in-change | next-change | ["artifact",["design.md","D6. Scroll keys and the mouse are default entries of the keymap"],"The mouse wheel maps to `ScrollRows(3)` and `ScrollRows(-3)`."] | resolved | resolved in round 6 from fixed 06f54bb9387a5094368b0440845f40eb959829ad |  |  | app-1: D6 makes the mouse wheel a default entry of the keymap, which #101 reads from a file. The code binds only PageUp and PageDown in the keymap and hard-codes the wheel in on_mouse. design.md records no deviation, so #101 cannot remap the wheel through the keymap.; fit-1: D6 makes the mouse wheel a default entry of the keymap, but update.rs hard-codes the wheel in on_mouse with WHEEL_ROWS, outside Keymap. #101 reads the keymap from a file and must move the wheel binding into it. |
| R-69 | 5 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["specs/output-pane/spec.md","Requirement: The pane follows new lines at the bottom"],"If the last row of the newest line is on the screen, the pane is at the bottom."] | duplicate | R-12 |  |  | Only a scroll clears the anchor. If a resize brings the last row of the newest line onto the screen, the pane is at the bottom by the spec, but it keeps its anchor. It then stops following once new lines pass the height. fits() also returns false for any anchor, so the bar shows while all rows fit. |
| R-70 | 5 | 6 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. The line store follows the output-pane decision"],"A span is a byte offset and an id in the style table of the pane."] | resolved | resolved in round 6 from fixed b4d14b9a10b36fc3847c91056849a1b17e19cb21 |  |  | D3 gives each pane a style table, but no decision bounds it or reclaims entries when a trim drops lines. The table only grows until a Reset. After 65536 distinct styles, which 24-bit gradients can reach in a long session, each new style is drawn as the default. |
| R-71 | 5 | 6 | should-fix | in-change | next-change | ["artifact",["design.md","D8. The width probe runs before the first frame, reads the terminal itself, and the test helper answers it"],"It takes the size of a cell in pixels from the pixel fields of the Termina window size, so it sends no cell size query."] | resolved | resolved in round 6 from fixed de7ca0618d418e21174d0ea8b161d905effb38c3 |  |  | D8 says the probe takes the cell size in pixels from the Termina window size. The probe outcome and Caps hold no cell size. Only the image-pane example reads the pixel fields, so #13 has no cell size from the client to size an image. |
| R-72 | 5 | 6 | should-fix | in-change | drift | ["path","crates/mu-tui/examples/image-pane.rs",51] | resolved | resolved in round 6 from fixed de7ca0618d418e21174d0ea8b161d905effb38c3 |  |  | The image-pane example copies private_mode and the terminal open and leave sequence of run.rs. The copy has already drifted: it calls probe::run, which sets mode 2027, but its exit and its panic hook never reset the mode, while run.rs leave does. |
| R-73 | 5 | 6 | should-fix | in-change | drift | ["path","crates/mu-core/examples/transcript_lines.rs",18] | resolved | resolved in round 6 from fixed c9566766e36fbb8a0d8b7fc7051e2c09bb3faba5 |  |  | The new example transcript_lines parses the transcript record format a second time. The test support of mu-core already has transcript_reads for the same format, with a check of the direction and the seconds that the example skips. |
| R-74 | 5 | 5 | should-fix | in-change | drift | ["path","crates/mu-cli/tests/support/process.rs",755] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/112; reason: Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. |  | severity? suggestion 0.82 | The new wait_screen of the pseudo-terminal helper repeats the existing wait_rows line for line: the same receiver, note, bound and wait_for, with only the check differing. |
| R-75 | 5 | 6 | should-fix | in-change | drift | ["path","crates/mu-tui/src/pane.rs",537] | resolved | resolved in round 6 from fixed 92c052c13bf4fbaa9bfbe8eb2da39a4790f5d9f9 |  |  | The pane looks up the style at a byte twice, once over store spans and once over the spans of the unfinished line, with the same take_while and last rule. A change to the span rule in one place renders a prompt unlike the same text once it is a line. |
| R-76 | 5 | 6 | should-fix | in-change | drift | ["path","crates/mu-tui/src/store.rs",224] | resolved | resolved in round 6 from fixed 92c052c13bf4fbaa9bfbe8eb2da39a4790f5d9f9 |  |  | The store records the display width of each line with unicode-width, while the pane measures every grapheme with the learned Profile. Two mechanisms give one quantity, and no production code reads the stored width, so a later reader gets the wrong width. |
| R-77 | 5 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: The client restores the terminal"],"it SHALL turn off mouse capture and each mode that it turned on, mode 2027 included, and leave the alternate screen."] | resolved | resolved in round 6 from fixed a1189d02d2e93510439dc05fe09d40cb641d3956 |  | severity? blocker 0.61; repeat? R-16 0.75 | The probe always writes the set of mode 2027, but the exit turns the mode off only when a DECRPM reply of 2 arrived within 500 ms. A terminal that answers late (the Late variant, or a slow SSH link) gets mode 2027 on and never off. update drops the late DECRPM event, so the client never learns it. late_answer checks only the input line, not the exit bytes. |
| R-78 | 5 | 6 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/pane.rs",593] | resolved | resolved in round 6 from fixed 803102a1b573b4d3a25d2a28b3749a6bae4fdd72 |  |  | The wrap and the cell layout use Profile::width for every class, and the probe learns rules for the ambiguous and combining classes too, but draw_text forces the width only for risky classes. A terminal that measures the ambiguous probe U+2460 as 2 gets Const(2): the pane counts 2 cells and resets the next cell, while ratatui and the Termina backend write that next cell contiguously, so the terminal shifts the rest of the row by one. |
| R-79 | 5 | 5 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/store.rs",193] | duplicate | R-70 |  | scope? beyond-scope 0.67 | The style table of a store holds at most 65536 styles, never drops a style at a trim, and gives the default style to every new style past the limit. A long session with 24-bit color gradients reaches that limit and then draws every new color as the default. No test passes the full table or one style past it. |
| R-80 | 5 | 6 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/tests/store.rs",51] | resolved | resolved in round 6 from fixed 6c8a913264901cdcd20b502d13d1abe1c4494c99 |  |  | No test reads the kept lines across a chunk free at 4096 lines. oldest_line_goes stays inside one chunk, and the property test checks only len <= capacity and the newest line, so a store that drops too many lines, or indexes the wrong chunk after base moves, still passes. |
| R-81 | 5 | 6 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/wrap.rs",57] | resolved | resolved in round 6 from fixed 6c8a913264901cdcd20b502d13d1abe1c4494c99 |  |  | 'A grapheme wider than the pane' asserts only wrap row slices; no test renders the pane and checks U+FFFD in one cell. |
| R-82 | 5 | 6 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/wrap.rs",41] | resolved | resolved in round 6 from fixed 6c8a913264901cdcd20b502d13d1abe1c4494c99 |  |  | 'An empty line' is tested on the wrap row count only; no render shows rows a, an empty row, b. |
| R-83 | 5 | 6 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/store.rs",27] | resolved | resolved in round 6 from fixed 6c8a913264901cdcd20b502d13d1abe1c4494c99 |  |  | 'An empty pane' is tested in the store (trim of an empty store); no test renders a pane with no lines or drops zero lines on a pane. |
| R-84 | 5 | 6 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/app.rs",444] | resolved | resolved in round 6 from fixed 6c8a913264901cdcd20b502d13d1abe1c4494c99 |  |  | game_line_draws_no_control_character computes its expected text with convert, the function the pane path uses, so the text equality cannot fail on a wrong conversion. |
| R-85 | 5 | 6 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/update.rs",49] | resolved | resolved in round 6 from fixed 6c8a913264901cdcd20b502d13d1abe1c4494c99 |  |  | ctrl_p_moves_the_focus_back_with_no_effect uses two worlds, where Ctrl+P and Ctrl+N both land on the second world, so a keymap that binds Ctrl+P to the next world passes. |
| R-86 | 5 | 5 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"Each snapshot test also asserts the cells that its scenario names."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/112; reason: Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. |  | severity? should-fix 0.79 | The snapshot risk is mitigated by cell asserts in each snapshot test. The one snapshot test, first_view_snapshot, asserts only the snapshot. |
| R-87 | 5 | 5 | suggestion | in-change |  | ["artifact",["design.md","D2. The skeleton: TEA with an Action enum and widgets by pane id"],"`ScrollPage(Up\|Down)`, `ScrollRows(n)`"] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/112; reason: Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. |  |  | D2 names the action ScrollPage(Up\|Down). At head it is ScrollPage(i32), a signed count of pages. |
| R-88 | 5 | 5 | suggestion | in-change |  | ["artifact",["design.md","D2. The skeleton: TEA with an Action enum and widgets by pane id"],"`Msg` is every input of the model: a terminal event, a client event, a reply and a probe result."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/112; reason: Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. |  | severity? should-fix 0.69 | D2 makes a probe result a Msg that update takes. Msg has no probe variant. run.rs changes the model with set_caps outside update, against the model doc that only update changes it. |
| R-89 | 5 | 5 | suggestion | in-change |  | ["artifact",["design.md","D8. The width probe runs before the first frame, reads the terminal itself, and the test helper answers it"],"With no DA1 reply in 500 milliseconds, every class keeps the `unicode-width` value, and an image line shows its URL."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/112; reason: Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. |  | severity? should-fix 0.77 | D8 says that with no DA1 reply in time an image line shows its URL. The code sets graphics from a kitty reply even when DA1 never came, and only the width profile waits for DA1. |
| R-90 | 5 | 5 | suggestion | in-change |  | ["artifact",["design.md","D8. The width probe runs before the first frame, reads the terminal itself, and the test helper answers it"],"It opens `/dev/tty`, polls it for at most 500 milliseconds"] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/112; reason: Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. |  | scope? deviates-design 0.61; severity? should-fix 0.75 | D8 says the probe opens /dev/tty and polls it. The probe reads stdin when stdin is a terminal and opens /dev/tty only otherwise, as Termina does. It waits with VMIN 0 and VTIME 1, not with poll. |
| R-91 | 5 | 5 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"The probe waits at most 500 milliseconds, once."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/112; reason: Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. |  | severity? should-fix 0.75 | The design bounds the probe at 500 ms. The loop checks the deadline and then starts a read that can block for one VTIME of 100 ms, so a silent terminal waits up to about 600 ms. |
| R-92 | 5 | 5 | suggestion | in-change |  | ["artifact",["proposal.md","Impact"],"Dependencies: a parser of escape sequences from an existing crate, and a snapshot test library"] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/112; reason: Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. |  | severity? should-fix 0.83 | mu-tui gains rustix with the termios feature, and it joins the mu-tui row of ALLOWED_CRATES. Neither the Dependencies part of the proposal nor any decision names it. The probe of D8 uses it for its timed read. |
| R-93 | 5 | 5 | suggestion | in-change |  | ["path","crates/mu-tui/src/probe.rs",137] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/112; reason: Output-pane implementation review, round 5: the wait_screen and wait_rows duplication in the pty helper, and design text that differs from the code (ScrollPage(n), the probe as a Msg, the tty read and its bound, rustix, graphics without DA1, the CSI parser, the snapshot asserts). None changes behaviour. |  | severity? should-fix 0.72 | The probe parses the cursor position, DECRPM and DA1 replies by hand, while the termina::Parser it already feeds the other bytes gives each of them as Event::Csi. Only the APC reply needs a parser of its own, and D8 names no reason for the CSI part. |
| R-94 | 9 | 10 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/update.rs",293] | resolved | resolved in round 10 from fixed d53d2beddfbecb32040d249df86c75b1e4b8e998 |  |  | A Reset pushes each kept notice again into the pane in focus, but clears only the panes of worlds. With no world, the pane in focus is the notices pane, which the Reset does not clear, so each notice shows once more at each Reset. mu-client gives a Reset as the first item of the stream, so a terminal that gives no widths and a server with no world show the width notice twice at startup. |

## Reviewed files

| path | content |
|---|---|
| Cargo.lock | dcba417bf3dea58a9dfb49c8ebab992e4c78444c |
| Cargo.toml | b41ef43534f9a429ce8a0d56d392915d5d5ac320 |
| architecture/model.c4 | 002b00f517c07007aab8b2c4a0e705a06af87cf4 |
| architecture/views.c4 | 4401a3d7ead5ee4bf34da01cce78c1ead0c90a82 |
| crates/mu-cli/tests/client.rs | 0cf8ff9cfd8699c60aaa2f3497b506d1699a45f1 |
| crates/mu-cli/tests/support/process.rs | 6d29ed6614d697818ad38312c0b570516c84c4e2 |
| crates/mu-cli/tests/workspace_boundary.rs | 30a6cf7834f80a1128fdba4eab3a4eb2e11fbaf9 |
| crates/mu-core/examples/transcript_lines.rs | ab496354f3811a784d7bda902a8a29578b33dfe7 |
| crates/mu-core/tests/support/mod.rs | efe56a459e86d4907fb42bea1b851c25502f3aaa |
| crates/mu-core/tests/support/transcript.rs | bf2c31bb9dbf5951cc8066fd697fa72bcbc96c06 |
| crates/mu-core/tests/telnet_transcripts.rs | 42150e4a228b88752f91f0e8c4a940b84ccac027 |
| crates/mu-tui/Cargo.toml | 3785799cce4ce95c9e2055fb94ca3d5e96a36750 |
| crates/mu-tui/examples/image-pane.rs | 021f9fe6b7f33651f76ac3ce27fcfe1ed3adf8a5 |
| crates/mu-tui/src/app.rs | absent |
| crates/mu-tui/src/convert.rs | 7e7cb1b48b5e68d2c5512fe71564920bf3f073e7 |
| crates/mu-tui/src/diacritics.rs | 6030a14cfbbdbacfcc2fb55c4384286078f13ede |
| crates/mu-tui/src/keymap.rs | ab82acedba3698bca8c5e0478974af126c3150de |
| crates/mu-tui/src/kitty.rs | 8d9ae18e195c47aaeb9d3bfa78df64a3dfb2d755 |
| crates/mu-tui/src/layout.rs | 1c0527ed55f420181f4466e059076ab9a82d5b91 |
| crates/mu-tui/src/lib.rs | d3aca2f8da9fa91dd9ba3b291e0a2cb5269e4137 |
| crates/mu-tui/src/model.rs | 0c206364fa01631c01c25676fbc1cab259f2cc08 |
| crates/mu-tui/src/pane.rs | 2fa1665bd444546cc82b7496b3e57413e509f165 |
| crates/mu-tui/src/probe.rs | 4474b2f92ac76702c6ee51d9215c8d2145b619b1 |
| crates/mu-tui/src/run.rs | b0de97ec4b43627c4304f619f5c6079dee128a14 |
| crates/mu-tui/src/screen.rs | 2cb155eb1c81a859cef4010bd372ab41d45179da |
| crates/mu-tui/src/store.rs | 05642b53d76b5c36d614d12aa72051d7a2ad9720 |
| crates/mu-tui/src/text.rs | 59209e40281df68d6c3e92862797ff8f888450ff |
| crates/mu-tui/src/update.rs | 0c2498b3e28d58a7eee2d933a5032b04a4f59473 |
| crates/mu-tui/src/view.rs | b68a45df8b1eeea6937df4577fcbc809cf61cfc0 |
| crates/mu-tui/src/width.rs | 8eaff2bf04da276226d0517d3ae50a984f75ef34 |
| crates/mu-tui/src/wrap.rs | 71f7dcac265fa495459b2c8359b572ef8305311c |
| crates/mu-tui/tests/app.proptest-regressions | c3d13b81b74af37408bb3df9956830bb61235f11 |
| crates/mu-tui/tests/app.rs | a0a35ccfbc2bcd21d97d507bf33c00753da68cb6 |
| crates/mu-tui/tests/data/char-width-ghostty.tsv | bb27e98275a7bbf6f6e7d0424f6f964bde2c672a |
| crates/mu-tui/tests/data/char-width-rex.tsv | acfb4e21ecd7673aca5cf7ad647037dfc1a1e85d |
| crates/mu-tui/tests/data/char-width-terminal.tsv | 3b2a59f6fdb7a6d931acf3a544d73e10dfa25c25 |
| crates/mu-tui/tests/data/transcript-lines.txt | 34083fad979786f7212be1ee0e50fbead6ac828c |
| crates/mu-tui/tests/keymap.rs | 8b37a9a9242c1a949e505772cccceed422b96260 |
| crates/mu-tui/tests/kitty.rs | e39a2f5666d10bd0cf9b327341d49cfd2a613f0d |
| crates/mu-tui/tests/pane.rs | f9d815f29f24ff7cd73144982c56692df09c8d2f |
| crates/mu-tui/tests/probe.rs | b90b0e8a633181fb71db1be018860cb3df3a2624 |
| crates/mu-tui/tests/screen.rs | 14f8e8ca1998447c4c9da91a2b6b783c3dd9a2e0 |
| crates/mu-tui/tests/snapshots/app__first_view_snapshot.snap | 4f61093601e1a81c3933bc090c0442d1357a46e4 |
| crates/mu-tui/tests/store.rs | ab07a16294c360039247880430e6b3af563bf20f |
| crates/mu-tui/tests/update.rs | 0bbe6886d9b23533c976fb64b8d94dedb7f8cde6 |
| crates/mu-tui/tests/width.rs | 5f005e395f9bd44faf22491afaa354d87102dfd0 |
| crates/mu-tui/tests/wrap.rs | 6e2abe5dc9525f323b04f4a1c8af0582bdc34709 |
| deny.toml | 94857513f354b7d42f5c02e797edc200d11ca90d |
| docs/src/content/docs/architecture/terminal-client.mdx | 0e8f734774a78b6bacbb5cdcd9d34a0bd46e5fde |
| docs/src/content/docs/decisions/character-width.md | 8e6b847a8d7ed734a22348e689aceba5652047fe |
| docs/src/content/docs/decisions/output-pane.md | 03a0cbb0c577797a56e62f206da10e9af39198e8 |
| docs/src/content/docs/decisions/terminal-client-architecture.md | 4887a78c9fe8c63fd74dbee9155b7536ebee07aa |
| docs/src/content/docs/guides/first-run.md | 1d7c40e14152f113c50ab9cf1034450c8861eee9 |
| openspec/changes/output-pane/.openspec.yaml | 56d16f001e73f64dfad7c20b1dc221e8c7dcba66 |
| openspec/changes/output-pane/design.md | eee82112d87f098aecd5b34fe45862011fda1478 |
| openspec/changes/output-pane/proposal.md | 374ccd82e6777249f4e2faae8a673edea3e96ddf |
| openspec/changes/output-pane/specs/output-pane/spec.md | 0b5ad47cfcdae971643d73fc9e55b2867dc407ce |
| openspec/changes/output-pane/specs/terminal-client/spec.md | 7e17ae38d5617956806ed066c9a3a5b3b30237c6 |
| openspec/changes/output-pane/tasks.md | sha256:3fa6c454d3e176030f82ef0ae5a19267766c9b105752c387c3971393c1040f59 |
| openspec/review.json | absent |
| renovate.json | 1379cb325dbbdee25f463bfc3d18e79ba78f1d26 |
| spikes/Cargo.lock | e8bfc666d56a8da1a0eea5ec35878fb612f30829 |
| spikes/terminal-backend/Cargo.toml | 2c327de16eed5c56b21e3a1ecbaf002eb7e4502d |
| spikes/terminal-backend/results/crossterm-rex.tsv | 8b81c1fc52eab3b86256d20a4de81ec7b5f2fb34 |
| spikes/terminal-backend/results/termina-rex.tsv | 4bd81e7b91db3ac3823cba76c267beca5d95bd98 |
| spikes/terminal-backend/src/bin/crossterm-check.rs | 6ed697ea99b435054fe5e79c822c1273f36d878c |
| spikes/terminal-backend/src/bin/termina-check.rs | 549d5f0dee8866ff20d76a74b45ffbae67ac8054 |
| spikes/terminal-backend/src/lib.rs | 5bb060e28e0e2559c661aa2b99e12e5e3bf99231 |

## Plan review (2026-11-20)

Verdict approve at head 113b6ea10c1388a3d1e3549a07c79216b69c775c.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-66 | test-coverage | should-fix | in-change | delta scenario 'A selection with both ends on dropped lines' has no matrix row | review > absent-row — "A selection with both ends on dropped lines" | add the row with its covering task or test |  |  | unmet-requirement |
| R-67 | test-coverage | should-fix | in-change | delta scenario 'Rows that fit only at the full width' has no matrix row | review > absent-row — "Rows that fit only at the full width" | add the row with its covering task or test |  |  | unmet-requirement |

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A long line wraps at a space | 3.2 | covered |  |
| A word wider than the pane | 3.2 | covered |  |
| A wide grapheme at the end of a row | 4.3 | covered |  |
| At the bottom | 4.2 | covered |  |
| Scrolled back | 4.2 | covered |  |
| A narrower pane | 4.2 | covered |  |
| A resize at the bottom | 4.2 | covered |  |
| The oldest line goes | 3.1 | covered |  |
| The scroll position was on a dropped line | 4.2 | covered |  |
| An empty pane | 3.1 | covered |  |
| Halfway | 4.2 | covered |  |
| All rows fit | 4.2 | covered |  |
| A red word that wraps | 4.3 | covered |  |
| Replaced by the finished line | 4.4 | covered |  |
| Replaced by a new unfinished line | 4.4 | covered |  |
| A selection across a wrapped line | 4.5 | covered |  |
| The selection shows | 4.5 | covered |  |
| A profile that adds code point widths | 9.2 | covered |  |
| The default profile | 9.2 | covered |  |
| An image at two widths | 10.2 | covered |  |
| Placeholder cells | 10.2 | covered |  |
| An image that is too tall | 10.2 | covered |  |
| No kitty graphics | 10.2 | covered |  |
| The copy of an image | 10.2 | covered |  |
| The first view | 2.4 | covered |  |
| The order of the worlds | 2.3 | partial |  |
| No worlds | 2.3 | partial |  |
| Prefixes | 2.3 | partial |  |
| Control characters | 5.2 | covered |  |
| Colors | 5.2 | covered |  |
| Other escape sequences | 5.2 | covered |  |
| A color across two lines | 5.2 | covered |  |
| Indexed and 24-bit colors | 5.2 | covered |  |
| A line in two reads | 6.2 | covered |  |
| A prompt with no line end | 6.1 | covered |  |
| Back and forward | 7.1 | covered |  |
| A new line while scrolled back | 7.1 | covered |  |
| Enter returns to the bottom | 7.1 | covered |  |
| The mouse wheel | 7.1 | covered |  |
| Back to a scrolled world | 7.1 | covered |  |
| Copy by drag | 8.2 | covered |  |
| A click clears | 8.2 | covered |  |
| A terminal that adds code point widths | 9.3 | covered |  |
| A terminal that does not answer | 9.3 | covered |  |
| The modes at exit | 9.4 | covered |  |

### Not run

Plan phase: no verification runs.
