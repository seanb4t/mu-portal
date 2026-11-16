---
verdict: changes-requested
phase: implementation
reviewed_at: 2026-11-16T06:02:02+00:00
head: a8727663cd4c4c87c27c6af8623bd1cb24533269
round: 8
next_id: 173
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: ok
  invariants: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-157 | fit | blocker | in-change | fit-1: The fix of R-157 moved journal.rs to TWO_WORLDS, but worlds.rs still writes the alpha/aria and beta/bryn list inline twice, once for two_worlds_connect and once for one_world_drops. The two-world list again has three copies: TWO_WORLDS in the support module and two in worlds.rs.; fit-2: The fix of R-157 made PW the one test password, but typed_password_is_masked still writes its value s3cret-pw as a literal in the text it sends. The value now lives in PW and in journal.rs, and a change to PW breaks this test. | crates/mu-cli/tests/worlds.rs:89 — "let mut session = Session::start(builder(&["alpha/aria", "beta/bryn"])).await;" | fit-1: Use Session::start(ServerBuilder::new().fake_worlds(TWO_WORLDS)) at worlds.rs:89 and Session::with_worlds(TWO_WORLDS) at worlds.rs:222.; fit-2: Send format!("my pw is {PW}") in typed_password_is_masked. | regression | severity? should-fix 0.86 |
| R-171 | test-coverage | blocker | in-change | covered row 'A hook puts a line break in a player line' failed its verification run | review > verification-failed — "A hook puts a line break in a player line: crates/mu-core/tests/scripts.rs: line_break_from_hook" | fix the test or the code, then rerun the review |  |  |
| R-168 | approach | should-fix | in-change | The design lists PtyClient::wait_greeting among the helpers of process.rs that more than one test file uses. At head only client.rs calls it, three times. | design.md > Test helpers — "`Session::with_worlds` and `PtyClient::wait_greeting` in `process.rs`, which more than one test file uses" | Say in Test helpers that wait_greeting is a method of PtyClient, which only client.rs uses, or drop it from the list of helpers that more than one test file uses. |  |  |
| R-169 | fit | should-fix | in-change | The new PtyClient::wait_greeting writes the greeting of alpha/aria as a literal. Session::welcome in the same file builds the same text from the world id, and resume.rs writes the literal a third time. The shared support now holds two rules for one greeting. | crates/mu-cli/tests/support/process.rs:562 — "rows.iter().any(\|row\| row.trim() == "Welcome, aria.")" | Add one fn greeting(world) in process.rs that gives the greeting of the fake game, and call it from Session::welcome, PtyClient::wait_greeting and resume_after_cut_connection. |  |  |
| R-172 | test-coverage | should-fix | in-change | covered row 'A line goes to the world in focus' has no verification run | review > not-run — "A line goes to the world in focus" | run the covering test and record it in verification-runs.txt |  |  |
| R-170 | invariants | suggestion | in-change | The rule that no control character from the server reaches the screen covers any world id and any player, system or credential line, but the new test passes one fixed string per part; the property test feeds game lines only. | crates/mu-tui/tests/app.rs:146 — "let world = "alpha/a\\u{1b}x";" | With proptest, as in no_control_character_is_drawn in crates/mu-tui/tests/app.rs, draw the same generated string as the world id and as a player, system and credential line, and compare the top row and the rows with the cleaned text. |  | severity? should-fix 0.62 |

## Repeats

| id | lens | severity | claim | evidence |
|---|---|---|---|---|
| R-166 | test-coverage | should-fix | covered row 'No world connects during the stop' has no verification run | review > not-run — "No world connects during the stop" |
| R-162 | test-coverage | should-fix | covered row 'An empty line' has no verification run | review > not-run — "An empty line" |

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-87 | should-fix | accepted | The controller checked pull requests 42 and 50 with gh api in this session: each has 0 review threads and one approving review with 0 findings, so the design claim holds. |  |
| R-156 | should-fix | accepted | Round 7 runs the test-honesty lens again on the files that changed since round 6, so its gap of round 6 closes there. |  |
| R-167 | should-fix | accepted | Round 8 runs the test-coverage lens again on the files that changed since round 7, so its gap of round 7 closes there. |  |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 904524521ced503b3eb800db0ff4ff0ce939d2b3 | 84352b9db5135133d271a296e308345f3b2ba602d6b1d5663493e312830be46d | changes-requested | 54 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 25fd0f577e88893e74979126424cc155c0ca258c | f21e80db40fb823e009c1e334862a95d90500551228223969eae6f4d9fc95a57 | changes-requested | 33 | 0 | 3 | 0 |  |
| 3 | plan | follow-up | 6f5af27a95b37101e37db8ea11d156f3a22217ce | 14cc6ab413646f3bbb7edf580172cc29a933c283ffc6496f725b9a1c7591b3a5 | changes-requested | 16 | 0 | 6 | 4 |  |
| 4 | plan | follow-up | 93dd7c11c91778ecef2263ef1f1d8e84725cd832 | c37f9ff3e3e836ba81ace2406d3dbd580d7856e5b52fdb16700772622df1af3b | changes-requested | 15 | 0 | 4 | 3 | operator decision |
| 5 | implementation | full | 8a7f2a2554801f540571de9ece3dffa7b8ffdefd | 20ad10d94acf1c74083efffa1974cb46a04fb4eafd80b660b3be7761d2e3baaa | changes-requested | 26 | 0 | 4 | 0 |  |
| 6 | implementation | follow-up | 36f80479faa245a57689d0bdad6e9789f29abe78 | 235a4508309df1bb8e3996b1e4bd4029dd3cece20059c1fd0e40e4331cc53877 | approve-with-notes | 12 | 0 | 1 | 1 |  |
| 7 | implementation | follow-up | f96e29b69e0c268ca4817dd14f8b761c27a98cf8 | bf0014f429274cdabf2743547f2056cf42e84e69b520c8ea7823af5adc51604d | changes-requested | 11 | 1 | 1 | 0 |  |
| 8 | implementation | follow-up | a8727663cd4c4c87c27c6af8623bd1cb24533269 | 30fe1c4936a585056f5032fab4be9a96072fa442520ab52c16467c33a3444245 | changes-requested | 5 | 2 | 1 | 0 | operator decision |

## Judge

judge: 8 questions, 0.000 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| The server serves the API on the named socket | crates/mu-cli/tests/server.rs: serves_on_named_socket | covered | pass |
| The environment names the socket | crates/mu-cli/tests/server.rs: env_names_socket | covered | pass |
| The flag wins over the environment | crates/mu-cli/tests/server.rs: flag_wins_over_env | covered | pass |
| The default socket under the runtime directory | crates/mu-cli/tests/server.rs: default_socket_under_runtime_dir | covered | pass |
| An empty socket variable | crates/mu-cli/tests/server.rs: empty_socket_env_is_ignored | covered | pass |
| An empty runtime directory | crates/mu-cli/tests/server.rs: empty_runtime_dir_is_ignored | covered | pass |
| A bare socket name | crates/mu-cli/tests/server.rs: bare_socket_name_is_in_the_working_directory | covered | pass |
| New directory and socket are private | crates/mu-cli/tests/server.rs: new_socket_dir_is_private | covered | pass |
| A directory of another user is refused | crates/mu-cli/src/socket.rs: private_dir_refuses_another_owner | covered | pass |
| An open directory is refused | crates/mu-cli/tests/server.rs: open_socket_dir_is_refused | covered | pass |
| A second server is refused | crates/mu-cli/tests/server.rs: second_server_is_refused | covered | pass |
| A stale socket file is replaced | crates/mu-cli/tests/server.rs: stale_socket_is_replaced | covered | pass |
| A file that is not a socket is kept | crates/mu-cli/tests/server.rs: regular_file_at_socket_path_is_kept | covered | pass |
| SIGTERM stops the server | crates/mu-cli/tests/server.rs: sigterm_stops_server | covered | pass |
| The streams get the disconnects | crates/mu-cli/tests/server.rs: sigterm_sends_the_disconnects_first | covered | pass |
| No world connects during the stop | crates/mu-core/tests/scripts.rs: handler_cannot_connect_after_stop; crates/mu-core/tests/portal.rs: stop_disconnects_every_world_and_refuses_connect | covered | not-run |
| SIGINT stops the server | crates/mu-cli/tests/server.rs: sigint_stops_server | covered | pass |
| A client with no server starts one | crates/mu-cli/tests/client.rs: client_starts_a_server | covered | pass |
| No server answers in time | crates/mu-cli/tests/client.rs: client_gives_up_after_five_seconds | covered | pass |
| Two clients start at the same time | crates/mu-cli/tests/client.rs: two_clients_start_one_server | covered | pass |
| No terminal | crates/mu-cli/tests/server.rs: client_needs_a_terminal | covered | pass |
| No terminal on stdin | crates/mu-cli/tests/server.rs: client_needs_a_terminal | covered | pass |
| A bad world id is refused | crates/mu-cli/tests/worlds.rs: bad_world_id_is_refused | covered | pass |
| Ids outside the rule | crates/mu-core/src/id.rs: ids_outside_the_rule | covered | pass |
| Ids inside the rule | crates/mu-core/src/id.rs: ids_inside_the_rule | covered | pass |
| The fake world answers | crates/mu-cli/tests/worlds.rs: fake_world_answers | covered | pass |
| The flag is hidden | crates/mu-cli/tests/worlds.rs: fake_world_flag_is_hidden | covered | pass |
| A missing password variable | crates/mu-cli/tests/worlds.rs: missing_password_variable | covered | pass |
| Two worlds connect | crates/mu-cli/tests/worlds.rs: two_worlds_connect | covered | pass |
| Two characters on one game | crates/mu-cli/tests/worlds.rs: two_characters_on_one_game | covered | pass |
| An unknown world | crates/mu-cli/tests/worlds.rs: send_to_unknown_world | covered | pass |
| A world that is not connected | crates/mu-cli/tests/worlds.rs: send_to_disconnected_world | covered | pass |
| A write that fails | crates/mu-core/tests/portal.rs: failed_write_records_disconnect | covered | pass |
| A world that is still connecting | crates/mu-core/tests/portal.rs: send_to_connecting_world | covered | pass |
| An empty line | crates/mu-cli/tests/worlds.rs: empty_send_reaches_game; crates/mu-cli/tests/client.rs: empty_enter_reaches_the_server | covered | not-run |
| A carriage return in the text | crates/mu-cli/tests/worlds.rs: send_with_cr_is_refused | covered | pass |
| A line break in the text | crates/mu-cli/tests/worlds.rs: send_with_lf_is_refused | covered | pass |
| One world drops | crates/mu-cli/tests/worlds.rs: one_world_drops | covered | pass |
| A world connects again | crates/mu-cli/tests/worlds.rs: world_connects_again | covered | pass |
| A hook gags a game line | crates/mu-core/tests/scripts.rs: hook_gags_game_line | covered | pass |
| A hook replaces a player line | crates/mu-core/tests/scripts.rs: hook_replaces_player_line | covered | pass |
| A hook puts a line break in a player line | crates/mu-core/tests/scripts.rs: line_break_from_hook | covered | fail |
| A hook gags a player line | crates/mu-core/tests/scripts.rs: hook_gags_player_line | covered | pass |
| A handler answers a line | crates/mu-core/tests/scripts.rs: handler_answers_line | covered | pass |
| A loop of handlers stops | crates/mu-core/tests/scripts.rs: handler_loop_stops_at_depth_limit | covered | pass |
| A handler that answers every event | crates/mu-core/tests/scripts.rs: handler_on_every_event_stops | covered | pass |
| A handler that answers its own echo | crates/mu-core/tests/scripts.rs: handler_loop_through_game_is_rate_limited | covered | pass |
| The limit opens again | crates/mu-core/tests/scripts.rs: rate_limit_refills | covered | pass |
| A handler connects | crates/mu-core/tests/scripts.rs: handler_connect_has_source_system | covered | pass |
| Events of two worlds share one sequence | crates/mu-cli/tests/journal.rs: events_of_two_worlds_share_one_sequence | covered | pass |
| Sources of a session | crates/mu-cli/tests/journal.rs: sources_of_a_session | covered | pass |
| The password is replaced | crates/mu-cli/tests/journal.rs: password_is_replaced | covered | pass |
| A typed password is masked | crates/mu-cli/tests/journal.rs: typed_password_is_masked | covered | pass |
| A password with a star | crates/mu-cli/tests/journal.rs: password_with_star_masks_whole_text | covered | pass |
| A password of stars is refused | crates/mu-cli/tests/journal.rs: password_of_stars_is_refused | covered | pass |
| A stream from the middle | crates/mu-cli/tests/journal.rs: stream_from_the_middle | covered | pass |
| A stream from 0 | crates/mu-cli/tests/journal.rs: stream_from_zero | covered | pass |
| A stream from the next number | crates/mu-cli/tests/journal.rs: stream_from_the_next_number | covered | pass |
| A stream past the end | crates/mu-cli/tests/journal.rs: stream_past_the_end | covered | pass |
| Resume after a cut connection | crates/mu-cli/tests/resume.rs: resume_after_cut_connection | covered | pass |
| Resume after a restart of the server | crates/mu-cli/tests/resume.rs: resume_after_server_restart | covered | pass |
| A subscriber that does not read | crates/mu-server/src/service.rs: lagged_stream_ends_with_resource_exhausted | covered | pass |
| A subscriber at the edge of the buffer | crates/mu-core/src/journal.rs: receiver_at_capacity_is_not_cut | covered | pass |
| The first view | crates/mu-tui/tests/app.rs: first_view | covered | pass |
| The order of the worlds | crates/mu-tui/tests/app.rs: world_order | covered | pass |
| No worlds | crates/mu-tui/tests/app.rs: no_worlds | covered | pass |
| Prefixes | crates/mu-tui/tests/app.rs: line_prefixes | covered | pass |
| Control characters | crates/mu-tui/tests/app.rs: control_characters_are_replaced | covered | pass |
| A line goes to the world in focus | crates/mu-tui/tests/app.rs: enter_sends_to_world_in_focus; crates/mu-cli/tests/client.rs: typed_line_reaches_the_server | covered | not-run |
| Enter with no world | crates/mu-tui/tests/app.rs: enter_with_no_world | covered | pass |
| A refused line | crates/mu-tui/tests/app.rs: refused_line_keeps_input | covered | pass |
| Next world | crates/mu-tui/tests/app.rs: ctrl_n_moves_focus | covered | pass |
| Wrap around | crates/mu-tui/tests/app.rs: ctrl_p_wraps_around | covered | pass |
| Next world wraps at the end | crates/mu-tui/tests/app.rs: ctrl_n_wraps_at_the_end | covered | pass |
| Keys with no world | crates/mu-tui/tests/app.rs: keys_with_no_world | covered | pass |
| One world keeps the focus | crates/mu-tui/tests/app.rs: one_world_keeps_focus | covered | pass |
| Activity mark | crates/mu-tui/tests/app.rs: activity_mark | covered | pass |
| A lost stream | crates/mu-cli/tests/client.rs: lost_stream_resumes_through_relay | covered | pass |
| The client resumes before a new event | crates/mu-tui/tests/app.rs: resumed_clears_disconnected | covered | pass |
| Quit | crates/mu-cli/tests/client.rs: ctrl_c_quits_client_only | covered | pass |

### Not run

- No world connects during the stop: no verification run names this row
- An empty line: no verification run names this row
- A line goes to the world in focus: no verification run names this row

## Verification runs

- exit 0: `cargo nextest run -p mu-cli --lib --locked -E 'test(~::private_dir_refuses_another_owner)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=client_gives_up_after_five_seconds)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=client_starts_a_server)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=ctrl_c_quits_client_only)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=empty_enter_reaches_the_server)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=lost_stream_resumes_through_relay)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=two_clients_start_one_server)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=typed_line_reaches_the_server)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=events_of_two_worlds_share_one_sequence)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=password_is_replaced)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=password_of_stars_is_refused)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=password_with_star_masks_whole_text)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=sources_of_a_session)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=stream_from_the_middle)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=stream_from_the_next_number)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=stream_from_zero)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=stream_past_the_end)'`
- exit 0: `cargo nextest run -p mu-cli --test journal --locked -E 'test(=typed_password_is_masked)'`
- exit 0: `cargo nextest run -p mu-cli --test resume --locked -E 'test(=resume_after_cut_connection)'`
- exit 0: `cargo nextest run -p mu-cli --test resume --locked -E 'test(=resume_after_server_restart)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=bare_socket_name_is_in_the_working_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=client_needs_a_terminal)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=default_socket_under_runtime_dir)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=empty_runtime_dir_is_ignored)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=empty_socket_env_is_ignored)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=env_names_socket)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=flag_wins_over_env)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=new_socket_dir_is_private)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=open_socket_dir_is_refused)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=regular_file_at_socket_path_is_kept)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=second_server_is_refused)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=serves_on_named_socket)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=sigint_stops_server)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=sigterm_sends_the_disconnects_first)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=sigterm_stops_server)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=stale_socket_is_replaced)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=bad_world_id_is_refused)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=empty_send_reaches_game)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=fake_world_answers)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=fake_world_flag_is_hidden)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=missing_password_variable)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=one_world_drops)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=send_to_disconnected_world)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=send_to_unknown_world)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=send_with_cr_is_refused)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=send_with_lf_is_refused)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=two_characters_on_one_game)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=two_worlds_connect)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=world_connects_again)'`
- exit 0: `cargo nextest run -p mu-core --lib --locked -E 'test(~::ids_inside_the_rule)'`
- exit 0: `cargo nextest run -p mu-core --lib --locked -E 'test(~::ids_outside_the_rule)'`
- exit 0: `cargo nextest run -p mu-core --lib --locked -E 'test(~::receiver_at_capacity_is_not_cut)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=failed_write_records_disconnect)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=send_to_connecting_world)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=stop_disconnects_every_world_and_refuses_connect)'`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=handler_answers_line)'`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=handler_cannot_connect_after_stop)'`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=handler_connect_has_source_system)'`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=handler_loop_stops_at_depth_limit)'`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=handler_loop_through_game_is_rate_limited)'`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=handler_on_every_event_stops)'`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=hook_gags_game_line)'`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=hook_gags_player_line)'`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=hook_replaces_player_line)'`
- exit 4: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=line_break_from_hook)'`
- exit 0: `cargo nextest run -p mu-core --test scripts --locked -E 'test(=rate_limit_refills)'`
- exit 0: `cargo nextest run -p mu-server --lib --locked -E 'test(~::lagged_stream_ends_with_resource_exhausted)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=activity_mark)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=control_characters_are_replaced)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=ctrl_n_moves_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=ctrl_n_wraps_at_the_end)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=ctrl_p_wraps_around)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=enter_sends_to_world_in_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=enter_with_no_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=first_view)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=keys_with_no_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=line_prefixes)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=no_worlds)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=one_world_keeps_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=refused_line_keeps_input)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=resumed_clears_disconnected)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=world_order)'`

## Dropped

None.

## Lens notes

- approach: Follow-up round: the bound is the files of diff-files.txt. design.md and tasks.md were read whole; the exact changed lines are not known without git, so the sections that the fixes of R-157 to R-161 touch (Test helpers, The server process, The terminal client, tasks 6.2, 7.1 and 7.3) were taken as changed.
- approach: Portal::stop and Portal::connect_all now share to_every_world, and stop sets the flag first. This matches step 7 of The server process, which calls stop the twin of connect_all.
- approach: PW, TWO_WORLDS, ServerBuilder::fake_worlds and Session::with_worlds are in process.rs and each is used by more than one test file, as Test helpers says. mode is in mod.rs, as the design now says.
- approach: control_characters_are_replaced_in_every_part in mu-tui/tests/app.rs covers a world id, a player line, a system line and a credential line, as task 7.1 says, and clean in mu-tui/src/app.rs uses char::is_control with a tab as a space, as The terminal client says.
- approach: Every test named by tasks 6.4, 6.5, 6.6 and 7.3 exists in worlds.rs, journal.rs, resume.rs and client.rs at head.
- approach: The cargo-deny rule and the Renovate group for crossterm that Third-party crates cites hold at head (deny.toml line 32, renovate.json line 48).
- fit: R-161 holds at head: Portal::stop and Portal::connect_all both go through to_every_world in crates/mu-core/src/portal.rs.
- fit: R-159 holds at head: the three PtyClient starts in client.rs share PtyClient::wait_greeting. ctrl_c_quits_client_only could call attached(), but the block left is three lines.
- fit: The design section Test helpers matches the placement of mode, PW, TWO_WORLDS, fake_worlds, with_worlds and wait_greeting at head.
- fit: No spec file changed since the base, so the follow-up round did not read the specs.
- fit: No changed line in crates/mu-tui/tests/app.rs raises a fit finding.
- invariants: I1: Portal::stop sets the stopped flag before it submits any Disconnect \| enforced at crates/mu-core/src/portal.rs:241 (store Release) and crates/mu-core/src/session.rs:354 (load Acquire on Connect) \| boundary test handler_cannot_connect_after_stop in crates/mu-core/tests/scripts.rs:430, stop_disconnects_every_world_and_refuses_connect in crates/mu-core/tests/portal.rs:310
- invariants: I2: stop and connect_all reach every world, in the order of the ids \| enforced at crates/mu-core/src/portal.rs:254 (loop over the BTreeMap router) \| boundary test stop_disconnects_every_world_and_refuses_connect with two worlds and the ordered disconnect list
- invariants: I3: A Disconnect at stop of a world that is already down records no event \| enforced at crates/mu-core/src/session.rs:365 \| boundary test stop_disconnects_every_world_and_refuses_connect checks exactly one disconnected event per world
- invariants: I4: No control character of a world id, an event line or a refusal reason reaches the screen; a tab shows as a space \| enforced at crates/mu-tui/src/app.rs:289 (clean), called at :178, :240, :303, :305, :307 \| boundary test control_characters_are_replaced (game line, reason) and control_characters_are_replaced_in_every_part (id, player, system, credential) in crates/mu-tui/tests/app.rs:124 and :145
- invariants: I5: The redaction tests of the binary type the password that the server holds \| enforced by the shared PW in crates/mu-cli/tests/support/process.rs:27 \| typed_password_is_masked in crates/mu-cli/tests/journal.rs:112 types the literal, but it asserts the masked text 'my pw is ****', so a change of PW fails the test and cannot pass vacuously; no finding.
- invariants: The refactor of Portal::stop and connect_all into to_every_world keeps each command, the origin System and the dropped result, so no check is weakened (R-161).
- invariants: The new test control_characters_are_replaced_in_every_part closes each part named by R-160: the world id, the player arm, and the system and credential arm.
- invariants: No enforcement point in the changed text is a script, so no probe ran.
- invariants: Follow-up round: only the files of diff-files.txt were read for findings; unchanged decisions and requirements are outside the bound.
- test-coverage: Follow-up round: no changed test file shows a weaker assertion than its THEN step.
- test-honesty: Read the changed tests: mu-cli client.rs, journal.rs, resume.rs, worlds.rs, support/process.rs and mu-tui app.rs. Each test asserts the THEN of its scenario against a built binary or App output, with no stub echo, no disabled test and no assertion-free test.
- test-honesty: mu-core/src/portal.rs has no test module in the diff.
- test-coverage gap: Scenario names matched to tests by reading changed test files and test lists; unchanged tests in mu-core/src (id.rs, socket.rs, journal.rs lag) were matched by name only.
- test-coverage gap: The delta of cli-entrypoint has no #### Scenario heading, so it has no matrix row; the BREAKING removal is proven by server-process scenarios named in its Migration text.

## Ledger

| id | first | last | severity | scope | location | disposition | basis | override | claim |
|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The client starts a server"],"`mu client` tries `ListWorlds` on the socket. If the connect fails, it runs `current_exe()` with `server --socket <path>`"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | No decision produces the requirement that the client needs a terminal. The design starts the client with a ListWorlds probe and a server start, and names no terminal check before them. |
| R-2 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The client starts a server"],"stdin from `/dev/null`, and stdout and stderr to `server.log`"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The client opens server.log in the socket directory before the server creates that directory. No decision says who creates a missing directory, with which mode, so the first start on the default path cannot produce the log, or makes a directory that step 2 of the server refuses. |
| R-3 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The server process"],"Bind, set mode 0600, write the line with the path to stderr, and serve."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | No decision makes each world connect and log in at the start of the server. The steps of the server process go from the bind to the signal and never build the Portal or send Connect to a session. |
| R-4 | 1 | 1 | blocker | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: A missing password variable"],"**THEN** stderr contains `ALPHA_PW`"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The scenario of a missing password variable needs the server to exit 1 at the start. The design resolves the secret only in the session, at the login line, and names no step that reads each secret before the server serves. |
| R-5 | 1 | 1 | blocker | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: A handler answers a line"],"**AND** the journal holds an event with the text `wave` and the source script"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The scenario of a handler answer needs the event of the handler send to have the source script. The Command enum is the same for the API and a handler and carries only a depth, so the design names no field from which the session records script instead of player. |
| R-6 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The client and its resume"],"On an error in the middle of the stream, it yields `Lost` and subscribes again from the next number."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The terminal-client requirement resumes on any end of the stream, and the server ends each stream on SIGTERM. The design resumes only on an error, so a clean end of the stream has no decision that shows disconnected and resumes. |
| R-7 | 1 | 1 | should-fix | in-change | ["artifact",["proposal.md","What Changes"],"`mu-core` no longer depends on `mu-proto`, so the boundary test keeps protobuf types out of the engine."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The proposal says mu-core no longer depends on mu-proto. At head mu-core has no dependency at all. Only the allowlist and the base fixture hold the edge, and task 1.3 does not name the fixture that allowed_edges_have_no_violation reads. |
| R-8 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Open Questions"],"The specs name the socket through the flag in each test, so this choice does not change them."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The open question says the specs name the socket through the flag in each test. Two scenarios of server-process name no flag: one uses MU_SOCKET and one uses the default under XDG_RUNTIME_DIR. |
| R-9 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Third-party crates"],"The versions are the latest at the time of the work, as the dependency policy says."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The design cites the dependency policy for the rule that each version is the latest. The dependency policy page names cargo-deny rules only and says nothing about the latest version. |
| R-10 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The server process"],"`clap` reads the flag and the variable."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The design says clap reads the MU_SOCKET variable. At head the workspace enables only the derive feature of clap, and clap reads a variable only with its env feature. The crate table and task 1.4 do not name clap. |
| R-11 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","What enforces each rule"],"\| `mu-core` has no protobuf type \| the boundary test \|"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | app-9: The design says the boundary test keeps protobuf types out of mu-core. The test checks only the edges between members, so mu-core can depend on buffa from the workspace table and pass.; inv-3: The boundary test checks only edges between members, so mu-core can take buffa or connectrpc directly and hold protobuf types while the test passes. |
| R-12 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The journal and the stream"],"A receiver that lags gets `RecvError::Lagged`, and the adapter ends its stream with `resource_exhausted`."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The adapter turns the broadcast receiver into a stream and maps Lagged to resource_exhausted, but the crate table leaves out tokio-stream, whose BroadcastStream the spike used for exactly this. |
| R-13 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","1. Crates and dependencies"],"Add a fixture test `core_depends_on_proto_fails` for that edge."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The base fixture of the boundary test holds the mu-core to mu-proto edge. After the allowlist drops it, allowed_edges_have_no_violation fails, and task 1.3 does not name the fixture change. |
| R-14 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The client and its resume"],"The wait between two tries starts at 100 milliseconds and doubles to at most 5 seconds."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The resume of PortalClient hand-writes an exponential backoff, and the design names no retry crate as the alternative or a reason to write it. |
| R-15 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The server process"],"Find the socket path: the flag, then `MU_SOCKET`, then the default in the spec."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The rule for the socket path (flag, MU_SOCKET, default) is placed in mu-server for the server, while the client gets its own --socket in ClientArgs, and the design names no one home for the rule both verbs need. |
| R-16 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","6. The verbs"],"Add `crates/mu-cli/tests/server.rs` with one test for each scenario of the server-process spec that needs no terminal"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The helper that runs the built mu binary is local to cli.rs. The new test files of tasks 6.3 and 6.4 run mu to its exit too, and the plan moves only a server helper into the shared support module. |
| R-17 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","5. The client library"],"Write a relay for tests in `crates/mu-client/tests/support/`."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | Task 5.1 puts the test relay in crates/mu-client/tests/support/, which no other crate can reach, yet the terminal-client lost-stream scenario also reads through a relay, so mu-tui must copy it. |
| R-18 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","5. The client library"],"Add the tests `resume_after_cut_connection` (scenario: resume after a cut connection) and `resume_after_server_restart`"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The resume tests of mu-client need a server that records events and restarts, but mu-client may depend only on mu-proto, dev edges included, so the tests must write a second PortalService beside the one of task 4.2. |
| R-19 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The schema"],"Make sure that `cargo build -p mu-proto --locked` passes and that `buf lint` reports no error on the file."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | fit-8: Task 2.1 checks the schema with buf lint, but buf is neither a tool nor a task in mise.toml, the one list of both, and the repository has no buf configuration.; inv-23: Task 2.1 checks buf lint once, but buf is not a tool in mise.toml and no check task runs it, so the schema rule is not kept. |
| R-20 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","8. Documentation"],"Add a page `docs/src/content/docs/guides/first-run.md` that shows how to start a server with two fake worlds and attach a client."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | Task 8.1 adds a page in a new guides directory, but the sidebar lists each directory by name, and no task adds a guides group, so the page is not in the sidebar. |
| R-21 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","1. Crates and dependencies"],"Change the allowlist in `crates/mu-cli/tests/workspace_boundary.rs` to the edges of the design, and remove the edge from `mu-core` to `mu-proto`."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | Task 1.3 changes the allowlist but not the base fixture: its mu-core still depends on mu-proto, and it has no mu-server or mu-client, so no fixture passes the new allowed edges. |
| R-22 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","3. The engine"],"The breaking ids hold an uppercase letter, the character `_`, a leading digit, an empty part, or three parts."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The breaking ids of world_id_rules omit a leading hyphen, an id with no slash and a non-ASCII lowercase letter, which char::is_lowercase accepts. |
| R-23 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The client starts a server"],"stdin from `/dev/null`, and stdout and stderr to `server.log`."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The client opens server.log in the socket directory before any server exists; on a first run that directory is missing, and no rule makes the client create it 0700, so the server then refuses it. |
| R-24 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The server process"],"Remove a stale socket file. The lock makes sure that no other server serves it."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The server removes whatever file is at the socket path, so a regular file named by --socket is deleted. |
| R-25 | 1 | 4 | should-fix | in-change | ["artifact",["specs/server-process/spec.md","Requirement: Only the owner can reach the socket"],"or whose mode gives access to the group or to others."] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The open directory test uses 0755, which sets every group and other bit; a mask that checks only some bits passes it, and no test uses a single bit such as 0701. |
| R-26 | 1 | 3 | should-fix | in-change | ["artifact",["specs/server-process/spec.md","Requirement: The client needs a terminal"],"If stdin or stdout of `mu client` is not a terminal, the client SHALL start no server."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | The rule is stdin or stdout, but the test pipes stdout only and Command::output also nulls stdin, so a check of stdin alone passes it. |
| R-27 | 1 | 1 | should-fix | in-change | ["path","openspec/specs/cli-entrypoint/spec.md",53] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The removed rule said a stub verb touches no file; its replacement checks only that s.sock is absent, so a client that creates the directory or server.log first still passes. |
| R-28 | 1 | 2 | should-fix | in-change | ["artifact",["design.md","The client starts a server"],"it runs `current_exe()` with `server --socket <path>`"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | A server started by the client has no world, so the view has an empty list, and no App test passes zero worlds or one world to Ctrl+N, Ctrl+P and Enter. |
| R-29 | 1 | 1 | should-fix | in-change | ["artifact",["specs/terminal-client/spec.md","Requirement: Enter sends the input to the world in focus"],"An empty input line SHALL send an empty line."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The spec says an empty input line sends an empty line, and no test presses Enter on an empty line. |
| R-30 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Crates and their edges"],"mu-tui -> mu-client, mu-proto terminal view"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | mu-client and mu-server are new routes from mu-tui to mu-core, and no fixture test refuses mu-client -> mu-core or mu-tui -> mu-server. |
| R-31 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A command reaches only its world"],"If the world is not connected, the call SHALL fail with the code `failed_precondition`."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | Not connected covers two states, connecting and disconnected, and the tests send only to a disconnected world. |
| R-32 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The API"],"\| `Send` \| `world`, `text` \| the sequence number of the player event \|"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | Send replies with the sequence number of the player event, but a gagged player line makes no event, so the reply has no value to carry. |
| R-33 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A hook can change a line before the journal"],"A gagged player line SHALL not reach the game."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The gag of a player line has no test: the hook tests gag a game line and replace a player line only. |
| R-34 | 1 | 1 | should-fix | in-change | ["artifact",["specs/terminal-client/spec.md","Requirement: The screen has three parts"],"The top row SHALL list the worlds in the order of their ids"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The top row lists worlds in the order of their ids, and the tests use alpha and beta only, where a string order and a (game, character) order agree. |
| R-35 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","3. The engine"],"Add the tests `env_secret_is_read`, `missing_env_secret_names_the_variable` and `secret_debug_hides_value`."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The secret tests cover an unset variable only; a variable that is set but empty, or not Unicode, has no test and no rule. |
| R-36 | 1 | 1 | should-fix | in-change | ["artifact",["specs/event-journal/spec.md","Requirement: The journal never holds a password"],"No event, and no line that the server writes to stderr or to its log, SHALL contain the password."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The requirement says no event holds the password, but only the login event is redacted; a player line with the password, and the fake world's echo of it, reach the journal. |
| R-37 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A command reaches only its world"],"The API call `Send` SHALL send one line to the world it names."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | Send must send one line, but nothing refuses a text with CR or LF; such a text writes two lines to the game, and the second skips the hooks and its own event. |
| R-38 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The engine: one actor for each world"],"At the depth limit, the engine records a script error and drops the command."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | Handlers run on every recorded event, and a dropped command records a script_error event; a handler that answers every event then loops through script errors with no bound. |
| R-39 | 1 | 5 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: A loop of handlers stops"],"**AND** the journal holds one event of the kind script error that names the depth limit"] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | The loop test checks for one script error but not the count of commands that ran, so a limit of 7 or 9 also passes. |
| R-40 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The journal and the stream"],"A test sets a smaller capacity through the constructor."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | tokio rounds the broadcast capacity up to a power of two and panics at zero, so a slow_subscriber_is_cut test with a capacity such as 3 and capacity+1 events never lags. |
| R-41 | 1 | 1 | should-fix | in-change | ["artifact",["specs/event-journal/spec.md","Requirement: A stream starts from a sequence number"],"0 and 1 SHALL both mean the start of the journal."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | Subscribe has edges at 0, at last+1 and past the end, and task 3.2 tests only from 4 of 6; resume after a restart asks from 6 on a shorter new journal. |
| R-42 | 1 | 1 | should-fix | in-change | ["artifact",["proposal.md","What Changes"],"**BREAKING**: `mu client` and `mu server` no longer print \\"not implemented\\"."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The proposal marks the removal of the 'not implemented' output as BREAKING, but no delta spec carries a BREAKING marker. |
| R-43 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The schema"],"that `buf lint` reports no error on the file."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | Task 2.1 names the build target `buf lint`, but the repository has no buf configuration and mise.toml does not install buf. |
| R-44 | 1 | 1 | should-fix | in-change | ["artifact",["specs/server-process/spec.md","Scenario: Two clients start at the same time"],"**THEN** both clients attach"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The THEN step 'both clients attach' names no observable value, status, or state. |
| R-45 | 1 | 1 | should-fix | in-change | ["artifact",["specs/event-journal/spec.md","Scenario: Events of two worlds share one sequence"],"**WHEN** a client sends one line to each world and reads the stream from sequence 1"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The WHEN step 'sends one line to each world' names no concrete line text. |
| R-46 | 1 | 1 | should-fix | in-change | ["artifact",["specs/event-journal/spec.md","Scenario: A subscriber that does not read"],"**WHEN** the server records more events than the live buffer holds"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The WHEN step 'the server records more events than the live buffer holds' names no number of events or buffer capacity. |
| R-47 | 1 | 2 | should-fix | in-change | ["artifact",["specs/terminal-client/spec.md","Scenario: A lost stream"],"**WHEN** the relay cuts the connection, the world gets a line, and the relay passes again"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The WHEN step 'the world gets a line' names no line, so the THEN step 'the client shows the line once' cannot be checked against a known text. |
| R-48 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: One world drops"],"**AND** a line sent to `beta/bryn` still gets its answer"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The THEN step 'a line sent to beta/bryn still gets its answer' names no concrete line and no expected text. |
| R-49 | 1 | 1 | should-fix | in-change | ["artifact",["specs/terminal-client/spec.md","Scenario: Activity mark"],"**WHEN** `beta/bryn` gets a new line from the game"] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The WHEN step 'beta/bryn gets a new line from the game' names no concrete line or text. |
| R-50 | 1 | 5 | should-fix | deviates-design | ["artifact",["design.md","Seams and their first implementations"],"#52 puts the telnet engine between the stream and the lines, and the session does not change."] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | The session splits the stream into lines itself, but the telnet engine of #52 returns bytes to send and flushes a prompt with no line end, so #52 must change the reader and the writer of the session. |
| R-51 | 1 | 5 | should-fix | in-change | ["artifact",["tasks.md","6. The verbs"],"It waits for the line on stderr with an upper bound, returns a `PortalClient`, and kills the process on drop."] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | Tasks 6.2 and 7.3 each build a bounded wait, one on a stderr line and one on the screen, and neither names a shared helper. |
| R-52 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","The client starts a server"],"Then it tries again for up to 5 seconds."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | The client has a second retry loop for the start of a server, next to the resume backoff. The two differ only in their stop rule. |
| R-53 | 1 | 1 | suggestion | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A world id names one character on one game"],"Each world SHALL have an id of the form `<game>/<character>`."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | WorldId parsing is a rule over unbounded strings, with a round trip of parse and display. |
| R-54 | 1 | 1 | suggestion | in-change | ["artifact",["specs/event-journal/spec.md","Requirement: A client resumes without a gap"],"The events of both streams together SHALL hold each sequence number exactly once and in order."] | fixed | 62215c7b0b9156133d5a18fd1b8205710157cd9c |  | Resume with no gap must hold for any cut point and any number of events recorded during the cut. |
| R-55 | 2 | 2 | blocker | in-change | ["artifact",["design.md","The engine: one actor for each world"],"The source of each event comes from the origin of its command."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The design takes the source of each event from the origin of its command. That rule gives an API Connect or Disconnect the source player and the login line the source system, while the spec asks for system and credential. It names no source for a game line. |
| R-56 | 2 | 2 | blocker | in-change | ["artifact",["design.md","The terminal client"],"`mu-tui` has an `App` with three methods: `on_event`, `on_key` and `draw`."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The refused-line scenario needs App to learn that Send failed, but App has only on_event, on_key and draw, and ClientEvent holds only Reset, Event and Lost. No input carries the reply of a call to App, so refused_line_keeps_input cannot drive it. |
| R-57 | 2 | 2 | blocker | in-change | ["artifact",["design.md","The client and its resume"],"In both cases, `events()` yields `Lost` and subscribes again from the next number."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | events() yields Lost on a lost stream and Reset only on a new instance id. A resume to the same server yields nothing until the next event, so with a quiet world the top row keeps showing disconnected after the resume. |
| R-58 | 2 | 2 | blocker | in-change | ["artifact",["design.md","The engine: one actor for each world"],"It owns a map from `WorldId` to a session handle, one journal and the server handlers."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The Portal owns the server handlers, but no decision says which task runs a server handler on the events of each world. An event has no depth field, so a server handler that runs outside the session cannot give its command the depth of its event plus one. |
| R-59 | 2 | 2 | blocker | in-change | ["artifact",["design.md","The server process"],"1. Read the secret of each world. A failure ends the start with status 1."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The bad-id scenario needs the exit status 2 of a usage error. The design builds the worlds from the --fake-world flags, puts no step that parses the id in clap, and ends each failure of the server start with status 1. |
| R-60 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The server process"],"2. Take the private directory from `private_dir`."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | app-6: Step 2 of the server in mu-server takes the directory from private_dir, but the design puts private_dir in mu-cli, and mu-server may depend only on mu-core and mu-proto. The step order cannot hold as written.; fit-4: private_dir is in mu-cli, but step 2 of the life of the process in mu-server calls it, and mu-server cannot depend on mu-cli. So mu-server needs a second copy of the directory check, or the order of the steps breaks. |
| R-61 | 2 | 3 | should-fix | in-change | ["artifact",["tasks.md","1. Crates and dependencies"],"Change the crate layout view in `architecture/model.c4` and the crate table of the same page."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | Task 1.4 puts the crate layout view in architecture/model.c4. At head the view and its description of four crates are in architecture/views.c4, and the crate-layout page also says four crates. |
| R-62 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","2. The schema"],"Generate the code in `build.rs` with `connectrpc-build`."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | connectrpc-build shells out to protoc, but no task pins protoc in mise.toml, the one list of tools that CI installs. So mu-proto cannot build in CI. |
| R-63 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Third-party crates"],"\| `ratatui`, `crossterm`, `ratatui-textarea` \| the terminal client \|"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The design adds crossterm as its own workspace crate. The input editor spike uses the crossterm that ratatui re-exports, and the design names neither that pattern nor a reason, so the key types can come from two crossterm versions. |
| R-64 | 2 | 2 | should-fix | deviates-design | ["artifact",["design.md","Seams and their first implementations"],"\| Layer source \| `LayerSource`: returns the fields of one layer \| not used yet, the trait only \| #53 TOML, #60 Lua \|"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | LayerSource is an own trait for one layer of fields, but the profile decision has figment merge the layers. figment's Provider trait is that seam already, and the design does not name it as the alternative. |
| R-65 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Crates and their edges"],"So the boundary test gets a second list: the third-party crates that a member must not use."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The boundary test keeps its rules as an allowlist, named the single source of the rule. The new list of third-party crates is a denylist, and the design names neither the allowlist pattern nor a reason, so a protobuf crate that is not on the list passes. |
| R-66 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","1. Crates and dependencies"],"Add the fixture tests `core_depends_on_proto_fails`, `core_depends_on_buffa_fails`, `client_depends_on_core_fails` and `tui_depends_on_server_fails`."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The new fixture tests are named <from>_depends_on_<to>_fails. The existing edge tests are named <from>_to_<to>_fails, and the task gives no reason for the new form. |
| R-67 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: A line break in the text"],"**WHEN** a client sends `one` followed by LF and `two` to `alpha/aria`"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The rule refuses CR or LF, but the one line-break test passes only LF, so a check for LF alone passes while a CR still splits the line for a game that ends lines on CR. |
| R-68 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The client and its resume"],"an exponential backoff from 100 milliseconds to at most 5 seconds, with no end."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The backoff of the resume has no end, but the ExponentialBuilder of backon stops after 3 retries by default; no named test runs past the third retry, and the resume tests succeed in fewer. |
| R-69 | 2 | 2 | should-fix | in-change | ["artifact",["specs/server-process/spec.md","Requirement: Only the owner can reach the socket"],"The server SHALL refuse a socket directory that another user owns, or whose mode gives access to the group or to others."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | private_dir refuses a directory of another owner, and no test passes another owner: the directory tests change only the mode. |
| R-70 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","What enforces each rule"],"\| No handler loop without a bound \| the depth limit test and the script error test \|"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The design says no handler loop runs without a bound, but the depth passes only from event to command; a game line starts at depth 0, so a handler that sends on each game line loops forever through the echo of the fake world. |
| R-71 | 2 | 2 | should-fix | in-change | ["artifact",["specs/terminal-client/spec.md","Scenario: An empty line"],"**THEN** the server records a player event with an empty text for `alpha/aria`"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | An empty text must reach the server as a player event, but only the App test passes it, and App has no server; no engine or binary test sends an empty text through Send. |
| R-72 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","4. The server adapter"],"It uses the capacity 4, and checks that the stream is open after 4 events and ends after the fifth."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The lag edge 4/5 exists only at an unpolled broadcast receiver. A check of 'still open' that polls reads event 1 and moves the cut to the sixth event, and over a socket the HTTP/2 window lets the server drain the receiver. |
| R-73 | 2 | 2 | should-fix | in-change | ["artifact",["specs/server-process/spec.md","Requirement: The server listens on a Unix socket"],"If `XDG_RUNTIME_DIR` is not set, the directory is `mu-<uid>` under the temporary directory of the user."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | An empty XDG_RUNTIME_DIR is set, so the rule gives the relative path mu/server.sock and private_dir makes `mu` in the working directory; the two unit tests cover only a set path and no variable. |
| R-74 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Crates and their edges"],"`mu-core` and `mu-tui` must not use `buffa`, `connectrpc`, `axum` or `connectrpc-build`."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The list of third-party crates holds mu-tui, but no fixture test gives mu-tui one of those crates, so an empty mu-tui entry passes every test. |
| R-75 | 2 | 2 | should-fix | in-change | ["artifact",["specs/event-journal/spec.md","Scenario: A stream past the end"],"**WHEN** a client subscribes from sequence 9"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The limit of Subscribe is last+1; one past it is last+2 (8 for a journal of 6), but the past-end test uses 9, so a check of from > last + 2 passes it. |
| R-76 | 2 | 2 | should-fix | in-change | ["artifact",["specs/terminal-client/spec.md","Scenario: Wrap around"],"**WHEN** the user presses Ctrl+P"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The wrap of Ctrl+N at the last world has no test: the wrap test presses Ctrl+P, and the one-world test also passes with a clamp to the last index. |
| R-77 | 2 | 2 | should-fix | in-change | ["artifact",["specs/terminal-client/spec.md","Scenario: The order of the worlds"],"**GIVEN** a server with the fake worlds `alpha-2/bea` and `alpha/aria`"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The order test has two different game parts only, so the tie on the game part, where the character part decides, has no test; a sort by game alone keeps the server order and passes. |
| R-78 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: Ids outside the rule"],"**WHEN** the engine parses `-alpha/aria`, `alpha`, `alpha/b/c`, `alpha/`, `1alpha/aria` and `alpha/ar\\u00efa`"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | No refused id of the engine holds an ASCII uppercase letter or `_` in a valid shape, and the binary case Alpha_1 also lacks a slash, so a parser on is_ascii_alphanumeric passes. Task 3.1 names only the round trip, not the refusal property of the design. |
| R-79 | 2 | 3 | blocker | in-change | ["artifact",["design.md","Secrets and redaction"],"It replaces each occurrence of the password of that world with `****`."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | For a password with `*` the mask leaves the password in the event: a probe gives `a****` for `aa*` with password `a*`, and the login text `connect aria ****` holds the password `**`. No test passes such a password. |
| R-80 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The journal and the stream"],"So `MemoryJournal::new` refuses 0 and a capacity that is not a power of two."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The constructor guards the tokio panics, but tokio also panics above usize::MAX / 2, and 2^63 is a power of two above it, so MemoryJournal::new accepts a capacity on which tokio panics. |
| R-81 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","6. The verbs"],"`new_socket_dir_is_private`, and `open_socket_dir_is_refused` with the modes 0755, 0750, 0705 and 0701"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | Task 6.3 names new_socket_dir_is_private for the scenario, but the task states only the directory and never the socket mode 0600 that the THEN step also asserts. |
| R-82 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","3. The engine"],"`handler_loop_stops_at_depth_limit` counts exactly 8 custom events after the first one."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | Task 3.8 gives handler_on_every_event_stops no assertion; the scenario needs exactly one script error and no event after it, while 3.8 states exact counts only for the other loop test. |
| R-83 | 2 | 4 | should-fix | in-change | ["artifact",["specs/cli-entrypoint/spec.md","Requirement: A verb without an implementation says so"],"**Reason**: **BREAKING**: `mu client` and `mu server` no longer print `not implemented`."] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The REMOVED requirement marked BREAKING has no scenario for the removal; the migration says None, and the proof sits only in task text (6.1 deletes tests, 8.1 greps docs), not in a scenario. |
| R-84 | 2 | 3 | suggestion | in-change | ["artifact",["design.md","Open Questions"],"No test depends on the default on macOS."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | The open question says no test depends on the default on macOS, but task 6.1 adds default_socket_without_runtime_dir, which pins the default under the temporary directory. A later move to ~/Library changes that test. |
| R-85 | 2 | 2 | suggestion | in-change | ["artifact",["design.md","Test helpers"],"`mu(args)`, moved from `cli.rs`, which runs the built binary to its exit"] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The shared mu(args) and ServerProcess take no environment, but task 6.3 tests need MU_SOCKET, XDG_RUNTIME_DIR or no flag. So each such test builds its own Command, as the stub test in cli.rs did. |
| R-86 | 2 | 2 | suggestion | in-change | ["artifact",["design.md","Secrets and redaction"],"So the session masks each event of its world before it records the event."] | fixed | c2e58c83114ffdc7800794b1c58093ad6577c92f |  | The mask is a rule over every text and every password: the masked text never contains the password. Only two fixed texts test it. |
| R-87 | 2 | 2 | should-fix | in-change | ["artifact",["review","lens-status"],"approach partial"] | accepted | ticket: none; reason: The controller checked pull requests 42 and 50 with gh api in this session: each has 0 review threads and one approving review with 0 findings, so the design claim holds. |  | lens approach reported partial, gaps: 1 |
| R-88 | 3 | 3 | blocker | in-change | ["artifact",["design.md","The engine: one actor for each world"],"A connect and a disconnect have the source system, whatever command caused them."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | app-1: The handler requirement says the result of each handler command is recorded with the source script. The kind rule of the design gives a Connect or Disconnect from a handler the source system and a Login the source credential. It names no event kind for Echo, so an Echo has no source at all. No decision produces the requirement as written.; inv-7: Two rules give the source of a connect from a handler. The design gives a connect the source system whatever command caused it, while the world-sessions spec records the result of each handler command with the source script. No test passes a handler `Connect`. |
| R-89 | 3 | 5 | blocker | in-change | ["artifact",["specs/terminal-client/spec.md","Scenario: A lost stream"],"**WHEN** the relay passes again, and a test sends `after the cut` to `alpha/aria` through the API"] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | The lost-stream scenario needs a relay that refuses new connections after the cut until a step lets them through again. Without that, the top row is never reliably seen showing disconnected. The design's Relay can only forward and cut. The spike relay it comes from lets a new connection through at once, so the client resumes within its first backoff, and no step produces the pass-again step. The cut-connection resume scenario has the same gap for the events recorded while the client is away. |
| R-90 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Third-party crates"],"\| `tokio`, `tokio-stream` \| the runtime, channels, signals, the Unix socket, and `BroadcastStream` \|"] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | app-4: The design takes BroadcastStream from tokio-stream, as the spike did. tokio-stream exports BroadcastStream only with its sync feature, which the spike enables. The crate table and task 1.5 name a feature for clap only.; app-5: The engine tests and the backoff tests rely on `#[tokio::test(start_paused = true)]`. tokio compiles start_paused only with its test-util feature. The tokio row and task 1.5 name no tokio feature. |
| R-91 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The client and its resume"],"The `ExponentialBuilder` of `backon` stops after 3 tries by default, so the client turns that limit off."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | The design says the backon ExponentialBuilder stops after 3 tries by default. Its default max_times counts retries, so the default gives 4 tries. If the test of task 5.1 checks only a fourth try, it passes with the default limit still on. |
| R-92 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The terminal client"],"The loop uses the copy of `crossterm` that `ratatui` exports."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | The loop takes crossterm only through the ratatui export, but ratatui exposes no switch for the event-stream feature of crossterm. So the loop cannot use crossterm's EventStream and must hand-write a bridge from the blocking event read into the async join, and the design names neither the library stream nor a reason. |
| R-93 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The engine: one actor for each world"],"Each world accepts at most 20 commands from handlers in one second. The engine drops each command over the limit"] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | The rate limit of handler commands is a hand-written window counter, and the design names no rate-limit crate such as governor as the alternative, nor a reason to write it. |
| R-94 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The socket path and its directory"],"An empty or relative `XDG_RUNTIME_DIR` counts as unset, so the default path is always absolute."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | An empty `XDG_RUNTIME_DIR` counts as unset, but an empty `MU_SOCKET` has no rule and no test. clap reads the variable as it is, and its path parser refuses an empty value, so an exported empty `MU_SOCKET` makes a bare `mu` fail with a usage error. |
| R-95 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The engine: one actor for each world"],"Each world accepts at most 20 commands from handlers in one second."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | The rate limit is 20 commands in one second, but its test ends in the first second: the drop stops the echo loop, so no test reaches the next second. A counter that never resets, or one script error for all time, passes. |
| R-96 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The engine: one actor for each world"],"If it lags, it subscribes again from its last number, because the journal keeps every event."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | The server-handler task resubscribes from its last number, where the client takes the next number. From the last number read, the handler runs twice on that event, and the lag test names no check that each number reaches the handler once. |
| R-97 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The engine: one actor for each world"],"The world handlers run in the task of their world. The server handlers run in one more task, the server-handler task."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | World handlers and server handlers run in two tasks, so the skip of a `script_error` event has two enforcement points. Only the server handler gets the test that answers every event; no world handler is given a script error. |
| R-98 | 3 | 4 | blocker | in-change | ["artifact",["design.md","The engine: one actor for each world"],"Only an input takes its source from the origin of its command: `Player` gives player, and `Script` gives script."] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The source rule leaves three cases open: an input with the origin `System`, an event of the kind `message`, and the event of an `Echo` command, which has no kind. `event_source_follows_kind` checks each kind and each origin, so it has no expected value there. |
| R-99 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The client and its resume"],"The start of a server by the client uses the same crate, with a total limit of 5 seconds."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | The start of a server claims a total limit of 5 seconds, but no test runs the start to its bound, and the design names no outcome at it. Kept at the default of 3 tries that the design names, the wait from 100 ms ends after about 0.7 seconds. |
| R-100 | 3 | 3 | should-fix | in-change | ["artifact",["specs/server-process/spec.md","Scenario: A directory of another user is refused"],"the check expects an owner other than its owner"] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | The GIVEN of 'A directory of another user is refused' names no concrete owner; 'an owner other than its owner' is not an input a test can set. |
| R-101 | 3 | 3 | should-fix | in-change | ["artifact",["specs/event-journal/spec.md","Scenario: A subscriber that does not read"],"**AND** the second subscriber reads every event, in order"] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | The lag scenario needs a second subscriber that reads every event in order, and no task names a test with two subscribers. |
| R-102 | 3 | 4 | suggestion | in-change | ["artifact",["design.md","The journal and the stream"],"Each server start makes a new random instance id with the `uuid` crate."] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The instance id belongs to the server start, not to the journal. Once #57 keeps the journal across a restart, a new id on each start makes every client reset and drop events the journal still holds, so #57 must move the id into the journal. |
| R-103 | 3 | 3 | suggestion | in-change | ["artifact",["tasks.md","1. Crates and dependencies"],"Add the new failure to the list in `docs/src/content/docs/contributing/workspace.md`, and change the crate table and the text that says four crates."] | fixed | fe389cf77db46eed6bffa1b862000607c4279e8e |  | The second allowlist makes each third-party crate a third entry, but task 1.4 changes only the failure list of workspace.md. Its how-to still says a new crate needs only the workspace table and the crate, so #53, which adds figment, follows the page and fails the boundary test. |
| R-104 | 4 | 4 | blocker | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: A handler connects"],"**WHEN** the fake world drops the connection"] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The scenario of a handler connect needs the fake world to drop the connection from the game side, and the session to record a disconnect when its transport ends. The design gives the fake game and FakeTransport no drop, and no decision records a disconnect at the end of the transport. |
| R-105 | 4 | 4 | blocker | in-change | ["artifact",["tasks.md","6. The verbs"],"`client_needs_a_terminal`, which gives the client a terminal on stdin and a pipe on stdout."] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The two no-terminal scenarios need a pseudo-terminal on one stream and a pipe on the other. The design names only portable-pty, whose slave spawns a command with stdin, stdout and stderr all on the pty. It gives no fd for one stream, and the design names no other way to open a pty. |
| R-106 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","The engine: one actor for each world"],"`governor` was the other choice. Its clock is separate from the clock of tokio, so a test with the paused clock cannot move it."] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The design rejects governor because a paused tokio clock cannot move its clock. governor takes any clock through its Clock trait (RateLimiter::direct_with_clock), so a clock over tokio::time::Instant follows the paused clock. |
| R-107 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","Third-party crates"],"Its version is the one that `ratatui` uses, so the build has one copy"] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The design pins crossterm a second time beside the crossterm that ratatui pins, and says the two match. The repository's pattern for two coupled pins is a Renovate group plus a check, as for hk. The design names neither, and cargo-deny only warns on two versions. |
| R-108 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","Test helpers"],"`crates/mu-cli/tests/support/mod.rs` holds every helper that more than one test file uses:"] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | Two test files spawn mu client on a pseudo-terminal: server.rs gives it a terminal on stdin, client.rs on stdout and on both. The design's rule puts a helper of two files in support/mod.rs, but its list names no pseudo-terminal helper, so each file writes its own. |
| R-109 | 4 | 4 | should-fix | in-change | ["artifact",["specs/event-journal/spec.md","Requirement: A password that cannot be masked is refused"],"At the start, the server SHALL refuse a password that `****` holds, and a password with CR or LF."] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | No task creates the refusal of a password that `****` holds or that has CR or LF. Task 6.5 tests it, but tasks 3.3 and 6.1 only read the secret, and EnvSecretStore refuses only a missing, empty or non-Unicode value. |
| R-110 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","The client and its resume"],"an exponential backoff from 100 milliseconds to at most 5 seconds, with no end."] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The cap of 5 seconds on the resume backoff is first reached at the seventh wait (100 ms times 2^6 is 6.4 s). No task names the waits that `backoff_schedule` checks, and `backoff_runs_past_four_tries` stops at the fifth try. So a builder with no `with_max_delay`, which falls back to the default of backon, passes both tests. |
| R-111 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","Secrets and redaction"],"A property test makes sure that no masked text holds its password, for any text and any password."] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The property test claims that no masked text holds its password for any password. That is false for `*` to `****` and for the empty password: the probe masks `**` in `**` to `****`, which holds `**`. The only rule that rules those passwords out is at step 1 in mu-cli, so mu-core has no guard for the generator to use. |
| R-112 | 4 | 4 | should-fix | in-change | ["artifact",["specs/event-journal/spec.md","Scenario: A password of stars is refused"],"the environment variable `ALPHA_PW` holds `**`"] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The refusal of a password of stars is tested with `**` only. The edges are `*` and `****`, which must be refused, and `*****`, which must be accepted because the probe masks it fully. A check for exactly two stars, or for fewer than four, passes the test. |
| R-113 | 4 | 4 | should-fix | in-change | ["artifact",["tasks.md","6. The verbs"],"`password_of_stars_is_refused`, `password_with_line_break_is_refused`"] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The rule refuses a password with CR or LF, but it has one test, `password_with_line_break_is_refused`, which names neither character. A check for LF alone passes it, and then a CR still splits the login line. |
| R-114 | 4 | 4 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: The limit opens again"],"one second passes, and a client sends `start` again"] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | Only the open side of the one-second window is tested, at exactly one second. With the paused clock, every command of the loop runs at one instant, so a window of 1 ms passes both rate-limit tests. |
| R-115 | 4 | 4 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Requirement: A loop through the game has a bound"],"It SHALL record at most one script error for each second in which it dropped a command."] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The rule of at most one script error per second has no test with two drops in one second. The echo loop stops at the first drop, because a dropped send gets no echo. So an engine that records one script error for each dropped command passes `handler_loop_through_game_is_rate_limited`. |
| R-116 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","The engine: one actor for each world"],"Only an input and an echo take their source from the origin of their command: `Player` gives player, `Script` gives script, and `System` gives system."] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | The design now takes the source of an echo from its origin, but `event_source_follows_kind` covers an input from each origin only. A source function that gives every echo the source game, or one fixed source, passes it. |
| R-117 | 4 | 4 | should-fix | in-change | ["artifact",["specs/server-process/spec.md","Scenario: No server answers in time"],"after at most 5 seconds stderr contains `<dir>/s.sock` and `server.log`"] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | Only the upper side of the 5-second limit for starting a server is tested. A client that gives up after its first failed retry also writes the error and exits with status 1 in under 5 seconds, so it passes `client_gives_up_after_five_seconds`. |
| R-118 | 4 | 4 | should-fix | in-change | ["artifact",["tasks.md","6. The verbs"],"`serves_on_named_socket`, `env_names_socket`, `flag_wins_over_env`, `default_socket_under_runtime_dir`, `empty_runtime_dir_is_ignored`"] | fixed | 3df82b84f1fe5bc12995e17a9dd1efcd74819915 |  | Task 6.3 names serves_on_named_socket without the check that stderr lacks 'not implemented'. The REMOVED requirement's migration relies on that scenario step as its proof. |
| R-119 | 5 | 5 | should-fix | in-change | ["path","crates/mu-server/src/serve.rs",134] | fixed | 3748244ae5de715f20f3e1d9d62bcde0b3591876 |  | Step 7 of the design stops the sessions and then ends the streams. The code ends every stream in the graceful shutdown and disconnects the worlds only after axum returns, so no client stream gets the disconnect events, and design.md records no deviation. |
| R-120 | 5 | 5 | should-fix | in-change | ["path","crates/mu-cli/src/socket.rs",135] | fixed | d27ef6e72664e2c2781ff4ce1f4350a61e720f0a |  | The design says rustix reads the user id and the owner of the socket directory. At head rustix reads only the user id; private_dir reads the owner through std MetadataExt::uid. |
| R-121 | 5 | 7 | should-fix | in-change | ["path","crates/mu-cli/tests/support/process.rs",341] | fixed | 95d662bc9149e2ae21b8ad761b9821d57078ca83 |  | The design puts every shared test helper in support/mod.rs. At head ServerProcess, Relay, mu_on_pty and wait_until are in support/process.rs, and mu_on_pty and wait_until take other arguments than the design names. |
| R-122 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",193] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | The code that runs handlers on an event is written twice, once in the session for world handlers and once in the server-handler task. Both copies skip a script_error event and route the commands at the event depth. |
| R-123 | 5 | 5 | should-fix | in-change | ["path","crates/mu-cli/src/client.rs",42] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | fit-3: Both verbs repeat the same start: resolve the socket, take its parent with a fallback to /, call private_dir with current_uid, then build a tokio runtime and map each error through fail.; inv-7: mu client takes the socket directory the same way, so --socket s.sock or MU_SOCKET=s.sock fails in private_dir on the empty path before any server starts. |
| R-124 | 5 | 5 | should-fix | in-change | ["path","crates/mu-cli/tests/journal.rs",17] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | journal.rs and worlds.rs each define their own started helper. Both fold fake worlds with the variables PW_<n> into a ServerBuilder, start a Session and wait for each welcome. The design puts a helper that two test files use in the support module. |
| R-125 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",171] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | mu-core states the rule that a CR or LF breaks a line twice: once for a sent text in the session and once for a password in the secret check. The framer owns what a line is, and #52 changes the line ends. |
| R-126 | 5 | 5 | should-fix | in-change | ["path","crates/mu-cli/tests/support/process.rs",25] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | The test support defines its own SERVING text and parses the serving line itself, beside mu_server::SERVING. mu-cli already depends on mu-server, so its tests can use that constant. Today the two texts differ by a trailing space. |
| R-127 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",331] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | A player hook that replaces a line with text holding CR or LF writes two lines to the game; the line-break check runs only on the text before the hooks. |
| R-128 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/portal.rs",244] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | Task 3.8 is checked, but server_handler_resumes_after_lag does not exist; the lag branch of the server-handler task, which must resubscribe from the next number, has no test. |
| R-129 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/tests/scripts.rs",167] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | server_handler_sees_every_world does not exist, and every server-handler test starts one world, so a server handler that only sees one world passes. |
| R-130 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/tests/scripts.rs",219] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | inv-5: The rate-limit tests take their expected count from RATE_PER_SECOND, so a rate of 19 or 25 passes while the scenario fixes exactly 20 lines.; hon-5: handler_loop_through_game_is_rate_limited counts the sent lines against RATE_PER_SECOND from production code. The spec says exactly 20, so a different constant still passes. |
| R-131 | 5 | 5 | should-fix | in-change | ["path","crates/mu-cli/src/server.rs",74] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | For a bare socket name such as --socket s.sock, parent() is the empty path, so mu server fails in private_dir with an error that names no directory; a probe gives NotFound for the empty path. |
| R-132 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",236] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | A Send whose write to the game fails returns failed_precondition and also records a disconnected event, so a failed call records an event. |
| R-133 | 5 | 5 | should-fix | in-change | ["path","crates/mu-cli/tests/client.rs",30] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | client_starts_a_server runs the WHEN step but never quits the client, never calls the API after exit, and never checks server.log exists. |
| R-134 | 5 | 5 | should-fix | in-change | ["path","crates/mu-cli/tests/client.rs",72] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | client_gives_up_after_five_seconds checks only the text 'within 5 seconds' on the screen, not the socket path or server.log, and uses a regular file instead of the lock the scenario names. |
| R-135 | 5 | 5 | should-fix | in-change | ["path","crates/mu-tui/tests/app.rs",149] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | enter_sends_to_world_in_focus checks the Action value from App, but no test sends the typed line through the loop to the server and checks the player event. |
| R-136 | 5 | 5 | should-fix | in-change | ["path","crates/mu-tui/tests/app.rs",163] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | empty_enter_sends_empty_line checks the Action only; no test shows the server records the empty player event after Enter in the client. |
| R-137 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/journal.rs",307] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | receiver_at_capacity_is_not_cut reads only event 1, where the scenario needs events 1 to 4 in order; a cut after the first read passes it. |
| R-138 | 5 | 5 | should-fix | in-change | ["path","crates/mu-cli/tests/client.rs",68] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | client_gives_up_after_five_seconds allows exit up to 20 s and never checks the socket path or server.log that the THEN names. It also uses a regular file, not a held lock. |
| R-139 | 5 | 5 | should-fix | in-change | ["path","crates/mu-server/src/convert.rs",109] | fixed | 3748244ae5de715f20f3e1d9d62bcde0b3591876 |  | each_kind_and_source_has_a_schema_value is named for kind and source, but it checks only the kind. The source map has no assertion. |
| R-140 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/tests/scripts.rs",246] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | two_drops_give_one_script_error sets its burst and its expected line count from RATE_PER_SECOND, so the test follows the constant it should pin to 20. |
| R-141 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/tests/scripts.rs",230] | fixed | 5eb406c81381a3b9fd54c82259387022750b2de3 |  | rate_limit_refills computes its expected counts (rate and rate+1) from the production RATE_PER_SECOND, so it cannot fail when the limit of 20 changes. |
| R-142 | 5 | 5 | suggestion | in-change | ["path","crates/mu-cli/src/client.rs",28] | fixed | fe7eb91083b2b6dc3f87cc995bd9da169bbf6099 |  | The two verbs each define a private fail that writes one error line to stderr and returns status 1. They differ only in the verb prefix and in whether the line is written in one write. |
| R-143 | 5 | 5 | suggestion | in-change | ["path","crates/mu-tui/src/app.rs",293] | fixed | 117c8e72b4bddd38071496d6bf6dcf5377d6f2a1 |  | The view must never draw a control character from remote text, a rule over any string; the one test passes only ESC, BEL and TAB, not DEL or a C1 character such as U+009B. |
| R-144 | 5 | 5 | note | in-change | ["path","crates/mu-tui/tests/app.rs",124] | fixed | d27ef6e72664e2c2781ff4ce1f4350a61e720f0a |  | control_characters_are_replaced is explained by no scenario and no task. |
| R-145 | 6 | 6 | should-fix | in-change | ["artifact",["specs/server-process/spec.md","Scenario: A bare socket name"],"**WHEN** the user runs `mu server --socket s.sock`"] | fixed | e7280b06380c618187387cd81214fb1ff5cac951 |  | The server-process spec makes a relative socket path relative to the working directory, with the bare socket name scenario. No decision in design.md produces it: the socket path section covers only an empty or relative XDG_RUNTIME_DIR and an empty MU_SOCKET. The code makes the path absolute with std::path::absolute before private_dir. |
| R-146 | 6 | 6 | should-fix | in-change | ["artifact",["specs/terminal-client/spec.md","Scenario: Control characters"],"**WHEN** the game sends a line that holds ESC, BEL, DEL, a tab and the C1 character U+009B"] | fixed | e7280b06380c618187387cd81214fb1ff5cac951 |  | The terminal-client spec shows a tab as a space and every other control character, now with DEL and the C1 range, as U+FFFD. The terminal client decision names no filter of remote text, so no decision produces the rule that the code holds with char::is_control. |
| R-147 | 6 | 6 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: A hook puts a line break in a player line"],"**AND** the journal holds one script error and no player event"] | fixed | e7280b06380c618187387cd81214fb1ff5cac951 |  | The world-sessions spec now drops a player line that a hook replaced with text holding CR or LF and records a script error. The design refuses a line break only in the text of the API call, before the hooks, and names no check after the hooks. The session holds the check at line 338. |
| R-148 | 6 | 6 | should-fix | in-change | ["artifact",["specs/world-sessions/spec.md","Scenario: A write that fails"],"**AND** the server records one disconnected event for `alpha/aria`"] | fixed | e7280b06380c618187387cd81214fb1ff5cac951 |  | The spec makes a Send whose write to the game fails return failed_precondition, record the disconnect of the world and no player event. The design maps NotConnected only for the states connecting and disconnected, and no decision says what a failed write does. The session does it in write_line. |
| R-149 | 6 | 6 | should-fix | in-change | ["path","crates/mu-server/src/service.rs",70] | fixed | 676430d038f8c76d3204e36d3026b57f7e05fbfe |  | The adapter no longer turns the subscription into a stream with Subscription::into_stream. event_stream writes its own unfold over Subscription::next with a biased select on the stop signal, so the stop waits until no event is ready. into_stream is used only by a journal unit test. design.md and the checked task 4.2 still name into_stream and record no deviation. |
| R-150 | 6 | 6 | should-fix | in-change | ["path","crates/mu-tui/tests/app.rs",280] | fixed | e9d8e63fa02f5482002de6d5a32977e7e522e3a2 |  | app-7: This round adds the property test no_control_character_is_drawn. The design still says six rules get property tests and lists six, without the control-character filter, so its list of property tests is not so at head.; inv-3: The filter replaces exactly the control characters, U+0000-U+001F and U+007F-U+009F. The tests pass each inside edge, but the kept outside edges are U+007E and U+00A0, and only 'a' and space are kept chars. So a filter that also replaces U+00A0 or `~` passes every test. |
| R-151 | 6 | 6 | should-fix | in-change | ["path","crates/mu-server/src/service.rs",75] | fixed | 676430d038f8c76d3204e36d3026b57f7e05fbfe |  | fit-1: The adapter's event_stream writes its own unfold over Subscription::next, with the end-after-lag rule, beside Subscription::into_stream in mu-core. Task 4.2 and the design say the adapter uses into_stream, which now has only a unit-test call site.; inv-1: event_stream must send each event that is ready when the stop is set, and only then end. The biased select enforces it, but no unit test sets the stop while an event is ready: stopping_ends_the_stream stops an empty stream, and the binary test reaches the edge only when the scheduler happens to hold the last disconnect. |
| R-152 | 6 | 6 | should-fix | in-change | ["path","crates/mu-server/src/serve.rs",136] | fixed | f78fa166041873b49af60169e767552f96629701 |  | fit-2: The stop loops over the worlds in mu-server and calls Portal::disconnect, which acts as the player. The start uses Portal::connect_all in mu-core with the origin System. The design names neither the pattern nor a reason for the stop to differ.; inv-4: The spec says the server closes its connections to the worlds first, then ends the streams. The disconnects now run inside the shutdown future. axum keeps accepting connections until that future ends, and nothing in the Portal refuses a Connect after the stop. So a Connect from the API, or from a handler that reconnects on a disconnect, can bring a world back up before the streams end. |
| R-153 | 6 | 6 | should-fix | in-change | ["path","crates/mu-cli/tests/server.rs",215] | fixed | 20c39535c0b2387d3c18eb9b7788d75368a0b669 |  | sigterm_sends_the_disconnects_first builds the alpha/aria and beta/bryn server with the same two fake_world calls as two_worlds() in client.rs. The support module already has ServerBuilder::fake_worlds for this, and the design puts a helper of two test files there. |
| R-154 | 6 | 6 | should-fix | in-change | ["path","crates/mu-cli/tests/client.rs",34] | fixed | 20c39535c0b2387d3c18eb9b7788d75368a0b669 |  | client_starts_a_server reads the permission bits of the directory inline. That is the same expression as the mode helper of server.rs, so two test files each hold the check that the design's helper rule puts in the support module. |
| R-155 | 6 | 6 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",269] | fixed | f78fa166041873b49af60169e767552f96629701 |  | Each new connection of a world gets a new framer from the world's NewFramer. world_uses_its_framer checks only the first connection. A reconnect that falls back to LineFramer, or that keeps the framer and its partial line, passes every test. |
| R-156 | 6 | 7 | should-fix | in-change | ["artifact",["review","lens-status"],"test-honesty partial"] | accepted | ticket: none; reason: Round 7 runs the test-honesty lens again on the files that changed since round 6, so its gap of round 6 closes there. |  | lens test-honesty reported partial, gaps: 1 |
| R-157 | 7 | 8 | blocker | in-change | ["path","crates/mu-cli/tests/support/process.rs",27] |  | regressed from fixed 36c63890620287cf91438bc99f57fa7a8f5b4562 |  | fit-1: This round adds TWO_WORLDS to the shared support for the alpha/aria and beta/bryn list, but journal.rs still writes the same two-world list inline twice, so the list now has three copies.; fit-2: TWO_WORLDS writes the test password s3cret-pw as a literal, beside a const PW with the same value in journal.rs, worlds.rs and resume.rs. The shared support now holds a fourth copy instead of the one constant. |
| R-158 | 7 | 7 | should-fix | in-change | ["path","crates/mu-cli/tests/support/mod.rs",216] | fixed | 95d662bc9149e2ae21b8ad761b9821d57078ca83 |  | The design says support/mod.rs holds mu and mu_with and process.rs holds the rest, and it lists mode among the rest. At head mode is in support/mod.rs, and server.rs and client.rs import it from there. |
| R-159 | 7 | 7 | should-fix | in-change | ["path","crates/mu-cli/tests/client.rs",201] | fixed | 36c63890620287cf91438bc99f57fa7a8f5b4562 |  | client.rs starts a PtyClient on a two-world server and waits for the row Welcome, aria. three times: in attached(), in lost_stream_resumes_through_relay and in ctrl_c_quits_client_only. The three blocks are alike but for the socket. |
| R-160 | 7 | 7 | should-fix | in-change | ["artifact",["design.md","The terminal client"],"Before `App` draws any text from the server, a world id, an event line or the reason of a refusal"] | fixed | a94f3458d03ae9786c21a830fbc21bbbaf626df6 |  | inv-1: The design now says App cleans each world id from the server before it draws it, and top_row calls clean on the id. No test gives App a world id that holds a control character, so dropping that clean call passes every test.; inv-2: A player line is cleaned in its own match arm of line(). Every control-character test passes a game line or a refusal reason, so a player line with ESC reaches the screen unfiltered if that arm drops clean. Another client's typed text can hold ESC, because the server refuses only CR and LF.; inv-3: A system or credential line is cleaned in a third match arm of line(). No test passes a control character through that arm. line_prefixes uses plain text only, so a mutant without clean on the `-- ` arm passes. |
| R-161 | 7 | 7 | suggestion | in-change | ["path","crates/mu-core/src/portal.rs",245] | fixed | 8f8de4ebdc7c0b1efcad05632df1531113a90895 |  | Portal::stop and Portal::connect_all hold the same loop over router.keys() that submits a system envelope to each world and drops the result. They differ only in the command, and stop sets the flag first. |
| R-162 | 7 | 8 | should-fix | in-change | ["artifact",["review","not-run"],"An empty line"] | refuted | Each test that this row names ran alone at head with exit 0 and one test run, as lines of /private/tmp/claude-501/-Volumes-Code-github-com-seanb4t-mu-portal/2b9a00cd-6a2b-4449-a699-e4907b80bcc6/scratchpad/handoff/review-portal-skeleton-implementation-r7/verification-runs.txt show. The row names several tests, and the merge matches a run to the whole text after the first colon, so no single run can match it. |  | covered row 'An empty line' has no verification run |
| R-163 | 7 | 7 | should-fix | in-change | ["artifact",["review","not-run"],"A lost stream"] | refuted | Each test that this row names ran alone at head with exit 0 and one test run, as lines of /private/tmp/claude-501/-Volumes-Code-github-com-seanb4t-mu-portal/2b9a00cd-6a2b-4449-a699-e4907b80bcc6/scratchpad/handoff/review-portal-skeleton-implementation-r7/verification-runs.txt show. The row names several tests, and the merge matches a run to the whole text after the first colon, so no single run can match it. |  | covered row 'A lost stream' has no verification run |
| R-164 | 7 | 7 | should-fix | in-change | ["artifact",["review","not-run"],"Quit"] | refuted | Each test that this row names ran alone at head with exit 0 and one test run, as lines of /private/tmp/claude-501/-Volumes-Code-github-com-seanb4t-mu-portal/2b9a00cd-6a2b-4449-a699-e4907b80bcc6/scratchpad/handoff/review-portal-skeleton-implementation-r7/verification-runs.txt show. The row names several tests, and the merge matches a run to the whole text after the first colon, so no single run can match it. |  | covered row 'Quit' has no verification run |
| R-165 | 7 | 7 | should-fix | in-change | ["artifact",["review","not-run"],"Sources of a session"] | refuted | Each test that this row names ran alone at head with exit 0 and one test run, as lines of /private/tmp/claude-501/-Volumes-Code-github-com-seanb4t-mu-portal/2b9a00cd-6a2b-4449-a699-e4907b80bcc6/scratchpad/handoff/review-portal-skeleton-implementation-r7/verification-runs.txt show. The row names several tests, and the merge matches a run to the whole text after the first colon, so no single run can match it. |  | covered row 'Sources of a session' has no verification run |
| R-166 | 7 | 8 | should-fix | in-change | ["artifact",["review","not-run"],"No world connects during the stop"] | refuted | Each test that this row names ran alone at head with exit 0 and one test run, as lines of /private/tmp/claude-501/-Volumes-Code-github-com-seanb4t-mu-portal/2b9a00cd-6a2b-4449-a699-e4907b80bcc6/scratchpad/handoff/review-portal-skeleton-implementation-r7/verification-runs.txt show. The row names several tests, and the merge matches a run to the whole text after the first colon, so no single run can match it. |  | covered row 'No world connects during the stop' has no verification run |
| R-167 | 7 | 7 | should-fix | in-change | ["artifact",["review","lens-status"],"test-coverage partial"] | accepted | ticket: none; reason: Round 8 runs the test-coverage lens again on the files that changed since round 7, so its gap of round 7 closes there. |  | lens test-coverage reported partial, gaps: 1 |
| R-168 | 8 | 8 | should-fix | in-change | ["artifact",["design.md","Test helpers"],"`Session::with_worlds` and `PtyClient::wait_greeting` in `process.rs`, which more than one test file uses"] |  |  |  | The design lists PtyClient::wait_greeting among the helpers of process.rs that more than one test file uses. At head only client.rs calls it, three times. |
| R-169 | 8 | 8 | should-fix | in-change | ["path","crates/mu-cli/tests/support/process.rs",562] |  |  |  | The new PtyClient::wait_greeting writes the greeting of alpha/aria as a literal. Session::welcome in the same file builds the same text from the world id, and resume.rs writes the literal a third time. The shared support now holds two rules for one greeting. |
| R-170 | 8 | 8 | suggestion | in-change | ["path","crates/mu-tui/tests/app.rs",146] |  |  |  | The rule that no control character from the server reaches the screen covers any world id and any player, system or credential line, but the new test passes one fixed string per part; the property test feeds game lines only. |
| R-171 | 8 | 8 | blocker | in-change | ["artifact",["review","verification-failed"],"A hook puts a line break in a player line: crates/mu-core/tests/scripts.rs: line_break_from_hook"] |  |  |  | covered row 'A hook puts a line break in a player line' failed its verification run |
| R-172 | 8 | 8 | should-fix | in-change | ["artifact",["review","not-run"],"A line goes to the world in focus"] |  |  |  | covered row 'A line goes to the world in focus' has no verification run |

## Reviewed files

| path | content |
|---|---|
| .config/nextest.toml | feb1109a3edb427b7782174bfe90be94e486c497 |
| Cargo.lock | 81e8bb1ceb6b32a44c0696c6045e73b5192f5054 |
| Cargo.toml | e8103307796520e54458f6ed4d6591a9983935dc |
| README.md | 99c609614d9783b3c10c975871bb6f70b87be1db |
| architecture/model.c4 | 5208392c4938f9590764c98954ed39b38f14d5dc |
| architecture/views.c4 | e06a75240b52a0b6fb59fdfb039c0c419fab5001 |
| crates/mu-cli/Cargo.toml | 2229de33f2847fbc7a7965966bc092b83c2fd783 |
| crates/mu-cli/src/client.rs | d69ed35f89fe2d0cdfe66908615df0ce252ab906 |
| crates/mu-cli/src/lib.rs | b4780abceddcf1e7dac402df3244db2660f4c031 |
| crates/mu-cli/src/server.rs | ada307e0a504a02275ac4883aeb9050d164f1703 |
| crates/mu-cli/src/socket.rs | e12f41ef33482740a69a51f540cc402ec655949f |
| crates/mu-cli/tests/cli.rs | 6622746244a3f9630f0d1c00b31fbb5297dfbca9 |
| crates/mu-cli/tests/client.rs | 2361659a0bbb7e3bd4630d411f9469a6b13a1440 |
| crates/mu-cli/tests/fixtures/base/Cargo.toml | f2abcd3ae3dcc41e16d372312f9054cb5bd3eb49 |
| crates/mu-cli/tests/fixtures/base/crates/mu-cli/Cargo.toml | 72bb356dca1bc2fc15f5a9e212c382c0949721bc |
| crates/mu-cli/tests/fixtures/base/crates/mu-client/Cargo.toml | ce4171a94f29597db70058c7c65ab5be93a63efe |
| crates/mu-cli/tests/fixtures/base/crates/mu-client/src/lib.rs | e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 |
| crates/mu-cli/tests/fixtures/base/crates/mu-core/Cargo.toml | 3aabcbfc8e3aa1b13b56f7b175440b75bd1653a0 |
| crates/mu-cli/tests/fixtures/base/crates/mu-server/Cargo.toml | 91ca0daf2ce491f4893fc6791e783108a7f5ea93 |
| crates/mu-cli/tests/fixtures/base/crates/mu-server/src/lib.rs | e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 |
| crates/mu-cli/tests/fixtures/base/crates/mu-tui/Cargo.toml | f0f9958ceab73b38ffe7e7aba54ccb12b6511225 |
| crates/mu-cli/tests/journal.rs | f90e1f17094c731e7f01f65c001ca19bb1cc1792 |
| crates/mu-cli/tests/resume.rs | c8bb5305943578e0c390608c8aad36a8866d222a |
| crates/mu-cli/tests/server.rs | 7bf9be2bed15615911db1308aa36d9e0cdde0163 |
| crates/mu-cli/tests/support/mod.rs | cb02438cf9ce6cecce26f223fb4e6dd16d8c511c |
| crates/mu-cli/tests/support/process.rs | 0e3f4d2d5272df53abc7d16c21b3e32ee7f55da7 |
| crates/mu-cli/tests/workspace_boundary.rs | 6e89d62b66f369268866946980ff9109c28e83e6 |
| crates/mu-cli/tests/worlds.rs | 414c78bddfc656e79e894b52cd536013b5916852 |
| crates/mu-client/Cargo.toml | 127e327f1d2dd51900293abd02014efe6d3db42d |
| crates/mu-client/src/lib.rs | 9601cf8af61aedfdda5fb171a6675b9f4a4f79f2 |
| crates/mu-core/Cargo.toml | ef1f063cf6293f72646b6e4b098bbd4f6e04ff7e |
| crates/mu-core/src/command.rs | d232ec212ac62f2a93b8092ccad007e611dc92da |
| crates/mu-core/src/error.rs | 0ba8d0859c17ccf44eb9d5816935944db1336578 |
| crates/mu-core/src/event.rs | 72ec331c8478e4046d917333cbd2211bb99882fe |
| crates/mu-core/src/fake.rs | e866ffb9681fc9ea2f99390987aedef87a91f9ed |
| crates/mu-core/src/framer.rs | c75fc68e17c0ab3837d39fbbb4cc595507817b07 |
| crates/mu-core/src/id.rs | b5a941b7451465947b520a4262a4d2969fe35ba4 |
| crates/mu-core/src/journal.rs | 4d9fe74819df35d6a16c8e478901a9e77a0892f1 |
| crates/mu-core/src/lib.rs | 9aa97adfcd4a6605385e5ad8f56f18c885219a1b |
| crates/mu-core/src/portal.rs | 4c8effeb901e73e4e38083b975fe0d79443c87ab |
| crates/mu-core/src/script.rs | 10337c8657adbb5afe3ae0a4bc45d5c650cb64c8 |
| crates/mu-core/src/secret.rs | bd5963ad4ab954f3694dc554fc99388ecb8ad92c |
| crates/mu-core/src/session.rs | d43cdd270574e946be238631d7b404e85b4d35ee |
| crates/mu-core/src/transport.rs | 35d9902f0827521afeec533219bb4686d4718f46 |
| crates/mu-core/src/world.rs | 48808d4a9d20743c8b82cc6a45bdef7d1f5968b5 |
| crates/mu-core/tests/portal.rs | b3316cb0718b918edba716e4e0b3d527de247954 |
| crates/mu-core/tests/scripts.rs | 28dd9ef1d7999b6e3b9f18a049d9e2d39de2ee4f |
| crates/mu-core/tests/support/mod.rs | 59688c7d6ab4292616da5b6f5c709b00dab3a54b |
| crates/mu-proto/Cargo.toml | 3450f77894b2d4f23686501bde5ba83730988cc1 |
| crates/mu-proto/build.rs | 7436f4510d69da2416cad0f0d359421933d12122 |
| crates/mu-proto/proto/mu/v1/portal.proto | d11c6d249afc4aeddd1fc95d3035f4459a1ac965 |
| crates/mu-proto/src/lib.rs | 570886d87396607844d7e91bede184a7f4f3852d |
| crates/mu-server/Cargo.toml | 235121d00a84bfa4a0d0b6c1aa61dca976126cf6 |
| crates/mu-server/src/convert.rs | 3df53bfc56c809b4e1b3adcfd26b0d43de4e2783 |
| crates/mu-server/src/lib.rs | 73bfe699930bf2bda23890e64f823161613b272d |
| crates/mu-server/src/serve.rs | bc59e79b2111ff1b43539866ee43ad185db9a87e |
| crates/mu-server/src/service.rs | a96d58737ce2bca5eca7255a6efa6089ac60364a |
| crates/mu-tui/Cargo.toml | e4400aff0af1e8c7f591966940e40239e4f76015 |
| crates/mu-tui/src/app.rs | fc92eb2f2551d7b37dcab6a0c066738c1598cf14 |
| crates/mu-tui/src/lib.rs | 4a0bd6fc98b9c27bbe11e4e643ad39a4fae8952c |
| crates/mu-tui/src/run.rs | 1277c44ec3b526f714b7cb7cccad1c316ee2ff89 |
| crates/mu-tui/tests/app.rs | de7bac45f3e52f63de4533ed89b062bffc136816 |
| deny.toml | 2e2a653a991384ee01b6f8fcfd17d6aec7f00dfd |
| docs/astro.config.mjs | e2c998991f642d55ec6ba46650218ced83af8491 |
| docs/src/content/docs/architecture/crate-layout.mdx | 8bb2d3149e1897f1b7c8152ca42330fe2b53a30d |
| docs/src/content/docs/contributing/openspec.md | f6bbcb1c4462e7c1cde2d94577af76786e61bcbe |
| docs/src/content/docs/contributing/workspace.md | cfaefae5981f22fc9c1f65009c81a7f5d025d3bb |
| docs/src/content/docs/guides/first-run.md | 7c6aad8e2c44aeec1271ae5c03050120aa1fa243 |
| docs/src/content/docs/index.md | ec9f39356bfdb665591105f77a1d646910c7be85 |
| mise.toml | c5397ad2b5c6076704b3775c50006e9d02b876d3 |
| openspec/changes/portal-skeleton/.openspec.yaml | 70d2b31bd6d4fd60ee6b4fd384aeff2bf8551165 |
| openspec/changes/portal-skeleton/design.md | 1d87e2126b5f544d909063755ece1cf84662bbfd |
| openspec/changes/portal-skeleton/proposal.md | 87acd9f78a975a0b148b7f3c3fc3fe5ffed2eb4a |
| openspec/changes/portal-skeleton/specs/cli-entrypoint/spec.md | 90fedb75f1dd53f146e97fb7980ef43c82f9d7cc |
| openspec/changes/portal-skeleton/specs/event-journal/spec.md | c0283663519b913856f50578545493db2f7e5217 |
| openspec/changes/portal-skeleton/specs/server-process/spec.md | 87f44ad8a70319c855c35e48e34abb5b077cf35a |
| openspec/changes/portal-skeleton/specs/terminal-client/spec.md | a606ea4e3370556ac17d71954e3a12b501a15a45 |
| openspec/changes/portal-skeleton/specs/world-sessions/spec.md | b8f427fab7ad22abc0f139e07308aa729f6e9f71 |
| openspec/changes/portal-skeleton/tasks.md | sha256:e297c3c7a12655d3f56ea48cfea2ca9e5b1c3e6b50f015dc5ef98d8ca2e16533 |
| renovate.json | 26ed96ac538a0fd329caca308c93021ca1c5df5e |

## Plan review (2026-11-16)

Verdict changes-requested at head 93dd7c11c91778ecef2263ef1f1d8e84725cd832.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-98 | approach | blocker | in-change | Each event must carry a kind. The source rule now names the event of an Echo command, but the list of kinds has no kind for it, so no decision gives that event a kind, and event_source_follows_kind, which checks each kind, cannot check it. | design.md > The engine: one actor for each world — "Only an input and an echo take their source from the origin of their command" | Add an `echo` kind to the list of kinds and to the kind enum of task 2.1, or name the existing kind that an Echo command records. Extend `event_source_follows_kind` to an echo from each origin. | regression |  |
| R-104 | approach | blocker | in-change | The scenario of a handler connect needs the fake world to drop the connection from the game side, and the session to record a disconnect when its transport ends. The design gives the fake game and FakeTransport no drop, and no decision records a disconnect at the end of the transport. | specs/world-sessions/spec.md > Scenario: A handler connects — "**WHEN** the fake world drops the connection" | In the seams decision, let the fake game close its stream on a test call, and state that the session records a `disconnected` event with the source system when its transport ends. Name the drop in task 3.5 and in `handler_connect_has_source_system`. |  | severity? should-fix 0.67 |
| R-105 | approach | blocker | in-change | The two no-terminal scenarios need a pseudo-terminal on one stream and a pipe on the other. The design names only portable-pty, whose slave spawns a command with stdin, stdout and stderr all on the pty. It gives no fd for one stream, and the design names no other way to open a pty. | tasks.md > 6. The verbs — "`client_needs_a_terminal`, which gives the client a terminal on stdin and a pipe on stdout." | Name in the design how a test puts one stream on a pty. For example, open a pty with the pty calls of `rustix` and pass the slave fd as the `Stdio` of stdin or stdout of `std::process::Command`. Add the crate feature to the crates table and the read to task 1.2. |  | severity? should-fix 0.80 |
| R-106 | approach | should-fix | in-change | The design rejects governor because a paused tokio clock cannot move its clock. governor takes any clock through its Clock trait (RateLimiter::direct_with_clock), so a clock over tokio::time::Instant follows the paused clock. | design.md > The engine: one actor for each world — "`governor` was the other choice. Its clock is separate from the clock of tokio, so a test with the paused clock cannot m" | State a true reason, or use governor with a clock over tokio::time::Instant. A true reason: the GCRA of governor refills one cell every 50 ms for 20 per second, so it accepts more than 20 commands in the first second and has no fixed window for one script error each second, as the spec asks. If the design keeps a fact about governor, add governor to the documentation reads of task 1.2. |  |  |
| R-107 | fit | should-fix | in-change | The design pins crossterm a second time beside the crossterm that ratatui pins, and says the two match. The repository's pattern for two coupled pins is a Renovate group plus a check, as for hk. The design names neither, and cargo-deny only warns on two versions. | design.md > Third-party crates — "Its version is the one that `ratatui` uses, so the build has one copy" | In task 1.5, group ratatui and crossterm in a Renovate packageRule, as hk is grouped, and make a second crossterm copy fail, for example with a deny-multiple-versions entry for crossterm in deny.toml. Name both in the crate row. |  |  |
| R-108 | fit | should-fix | in-change | Two test files spawn mu client on a pseudo-terminal: server.rs gives it a terminal on stdin, client.rs on stdout and on both. The design's rule puts a helper of two files in support/mod.rs, but its list names no pseudo-terminal helper, so each file writes its own. | design.md > Test helpers — "`crates/mu-cli/tests/support/mod.rs` holds every helper that more than one test file uses:" | Add a ClientTerminal helper to the list of Test helpers and to task 6.2: it starts mu client with a pseudo-terminal on the streams a test names. Use it from server.rs and client.rs, or move client_needs_a_terminal to client.rs. |  |  |
| R-25 | invariants | should-fix | in-change | The six modes still leave three single bits untested. In the probe, a check that misses group read (040), group execute (010) or other read (004) refuses all six modes, and it accepts 0740, 0710 or 0704. | tasks.md > 6. The verbs — "`open_socket_dir_is_refused` with the modes 0755, 0750, 0705, 0701, 0720 and 0702" | Test one mode for each single bit: 0740, 0720, 0710, 0704, 0702 and 0701. | regression |  |
| R-109 | invariants | should-fix | in-change | No task creates the refusal of a password that `****` holds or that has CR or LF. Task 6.5 tests it, but tasks 3.3 and 6.1 only read the secret, and EnvSecretStore refuses only a missing, empty or non-Unicode value. | specs/event-journal/spec.md > Requirement: A password that cannot be masked is refused — "At the start, the server SHALL refuse a password that `****` holds, and a password with CR or LF." | Add a task step that creates the check as a constructor in mu-core, for example on the password of `ResolvedWorld`. It refuses a value that `****` holds and a value with CR or LF, and its error names the world and the variable. Step 1 in task 6.1 calls it. |  | severity? blocker 0.79 |
| R-110 | invariants | should-fix | in-change | The cap of 5 seconds on the resume backoff is first reached at the seventh wait (100 ms times 2^6 is 6.4 s). No task names the waits that `backoff_schedule` checks, and `backoff_runs_past_four_tries` stops at the fifth try. So a builder with no `with_max_delay`, which falls back to the default of backon, passes both tests. | design.md > The client and its resume — "an exponential backoff from 100 milliseconds to at most 5 seconds, with no end." | In task 5.1, name the waits that `backoff_schedule` checks: 100, 200, 400, 800, 1600, 3200, 5000 and 5000 ms, so the test reaches the cap and one wait past it. |  |  |
| R-111 | invariants | should-fix | in-change | The property test claims that no masked text holds its password for any password. That is false for `*` to `****` and for the empty password: the probe masks `**` in `**` to `****`, which holds `**`. The only rule that rules those passwords out is at step 1 in mu-cli, so mu-core has no guard for the generator to use. | design.md > Secrets and redaction — "A property test makes sure that no masked text holds its password, for any text and any password." | In the design and in task 3.7, limit the generator of `mask_never_leaves_the_password` to the passwords that the mu-core constructor of inv-1 accepts: not empty, not held by `****`, and with no CR or LF. Build each generated password through that constructor. |  |  |
| R-112 | invariants | should-fix | in-change | The refusal of a password of stars is tested with `**` only. The edges are `*` and `****`, which must be refused, and `*****`, which must be accepted because the probe masks it fully. A check for exactly two stars, or for fewer than four, passes the test. | specs/event-journal/spec.md > Scenario: A password of stars is refused — "the environment variable `ALPHA_PW` holds `**`" | Add a unit test of the refusal with `*` and `****` (refused) and `*****` (accepted). |  |  |
| R-113 | invariants | should-fix | in-change | The rule refuses a password with CR or LF, but it has one test, `password_with_line_break_is_refused`, which names neither character. A check for LF alone passes it, and then a CR still splits the login line. | tasks.md > 6. The verbs — "`password_of_stars_is_refused`, `password_with_line_break_is_refused`" | Test both values: a password with CR and a password with LF, as `send_with_cr_is_refused` and `send_with_lf_is_refused` do for Send. |  |  |
| R-114 | invariants | should-fix | in-change | Only the open side of the one-second window is tested, at exactly one second. With the paused clock, every command of the loop runs at one instant, so a window of 1 ms passes both rate-limit tests. | specs/world-sessions/spec.md > Scenario: The limit opens again — "one second passes, and a client sends `start` again" | In `rate_limit_opens_in_next_second`, first advance the paused clock to just under one second (for example 999 ms). Send `start` and check that the game gets no `again` and that no second script error appears. Then advance to one second and check for `again`. |  |  |
| R-115 | invariants | should-fix | in-change | The rule of at most one script error per second has no test with two drops in one second. The echo loop stops at the first drop, because a dropped send gets no echo. So an engine that records one script error for each dropped command passes `handler_loop_through_game_is_rate_limited`. | specs/world-sessions/spec.md > Requirement: A loop through the game has a bound — "It SHALL record at most one script error for each second in which it dropped a command." | Add a test with a handler that returns 25 sends for one game event, in one second of the paused clock. Check that the game gets 20 lines and that the journal holds exactly one script error. |  |  |
| R-116 | invariants | should-fix | in-change | The design now takes the source of an echo from its origin, but `event_source_follows_kind` covers an input from each origin only. A source function that gives every echo the source game, or one fixed source, passes it. | design.md > The engine: one actor for each world — "Only an input and an echo take their source from the origin of their command: `Player` gives player, `Script` gives scri" | Extend `event_source_follows_kind` with the event of an `Echo` from each of `Player`, `Script` and `System`. |  |  |
| R-117 | invariants | should-fix | in-change | Only the upper side of the 5-second limit for starting a server is tested. A client that gives up after its first failed retry also writes the error and exits with status 1 in under 5 seconds, so it passes `client_gives_up_after_five_seconds`. | specs/server-process/spec.md > Scenario: No server answers in time — "after at most 5 seconds stderr contains `<dir>/s.sock` and `server.log`" | Also assert the lower side. Either check that the client does not exit before a bound close to 5 seconds, or add a paused-clock unit test of the start loop that checks it keeps trying until 5 seconds pass. |  |  |
| R-83 | test-coverage | should-fix | in-change | The migration of the BREAKING removal points to server-process scenarios 'A named socket' and 'No terminal'. No scenario has the first name, so the proof of the removal cannot be traced. | specs/cli-entrypoint/spec.md > Requirement: A verb without an implementation says so — "The server-process scenarios "A named socket" and "No terminal" check that neither verb prints it." | Change the migration text of cli-entrypoint/spec.md to name the scenario 'The server serves the API on the named socket' and 'No terminal'. | regression |  |
| R-118 | test-coverage | should-fix | in-change | Task 6.3 names serves_on_named_socket without the check that stderr lacks 'not implemented'. The REMOVED requirement's migration relies on that scenario step as its proof. | tasks.md > 6. The verbs — "`serves_on_named_socket`, `env_names_socket`, `flag_wins_over_env`, `default_socket_under_runtime_dir`, `empty_runtime_d" | In task 6.3 add to serves_on_named_socket the assertion that the captured stderr does not contain `not implemented`, besides the socket path line and the ListWorlds call. |  |  |
| R-102 | fit | suggestion | in-change | The design now states that #57 must move the instance id from the server start into the journal. The defect of R-102 is still there: the next change must move the component, where the Journal seam could own the id now with the same behavior. | design.md > The client and its resume — "When #57 keeps the journal on disk, the id moves into the journal, so a restart does not reset each client." | Give the Journal seam the instance id: MemoryJournal::new makes it with uuid, and the adapter reads it for Hello. Each start still gets a new id, because each start makes a new journal, so #57 changes only its own journal. | regression | severity? should-fix 0.71 |

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| Events of two worlds share one sequence | 6.5 | covered |  |
| Sources of a session | 6.5 | covered |  |
| The password is replaced | 6.5 | covered |  |
| A typed password is masked | 3.7 | covered |  |
| A password with a star | 3.7 | covered |  |
| A password of stars is refused | 6.5 | covered |  |
| A stream from the middle | 3.2 | covered |  |
| A stream from 0 | 3.2 | covered |  |
| A stream from the next number | 3.2 | covered |  |
| A stream past the end | 3.2 | covered |  |
| Resume after a cut connection | 6.6 | covered |  |
| Resume after a restart of the server | 6.6 | covered |  |
| A subscriber that does not read | 4.2 | covered |  |
| A subscriber at the edge of the buffer | 3.2 | covered |  |
| The server serves the API on the named socket | 6.3 | partial |  |
| The environment names the socket | 6.3 | covered |  |
| The flag wins over the environment | 6.3 | covered |  |
| The default socket under the runtime directory | 6.3 | covered |  |
| An empty socket variable | 6.3 | covered |  |
| An empty runtime directory | 6.3 | covered |  |
| New directory and socket are private | 6.3 | covered |  |
| A directory of another user is refused | 6.1 | covered |  |
| An open directory is refused | 6.3 | covered |  |
| A second server is refused | 6.3 | covered |  |
| A stale socket file is replaced | 6.3 | covered |  |
| A file that is not a socket is kept | 6.3 | covered |  |
| SIGTERM stops the server | 6.3 | covered |  |
| SIGINT stops the server | 6.3 | covered |  |
| A client with no server starts one | 7.3 | covered |  |
| No server answers in time | 7.3 | covered |  |
| Two clients start at the same time | 7.3 | covered |  |
| No terminal | 6.3 | covered |  |
| No terminal on stdin | 7.3 | covered |  |
| A bad world id is refused | 6.4 | covered |  |
| Ids outside the rule | 3.1 | covered |  |
| Ids inside the rule | 3.1 | covered |  |
| The fake world answers | 6.4 | covered |  |
| The flag is hidden | 6.4 | covered |  |
| A missing password variable | 6.4 | covered |  |
| Two worlds connect | 6.4 | covered |  |
| Two characters on one game | 3.7 | covered |  |
| An unknown world | 3.7 | covered |  |
| A world that is not connected | 3.7 | covered |  |
| A world that is still connecting | 3.7 | covered |  |
| An empty line | 3.7 | covered |  |
| A carriage return in the text | 3.7 | covered |  |
| A line break in the text | 3.7 | covered |  |
| One world drops | 3.7 | covered |  |
| A world connects again | 3.7 | covered |  |
| A hook gags a game line | 3.8 | covered |  |
| A hook replaces a player line | 3.8 | covered |  |
| A hook gags a player line | 3.8 | covered |  |
| A handler answers a line | 3.8 | covered |  |
| A loop of handlers stops | 3.8 | covered |  |
| A handler that answers every event | 3.8 | covered |  |
| A handler that answers its own echo | 3.8 | covered |  |
| The limit opens again | 3.8 | covered |  |
| A handler connects | 3.8 | covered |  |
| The first view | 7.1 | covered |  |
| The order of the worlds | 7.1 | covered |  |
| No worlds | 7.1 | covered |  |
| Prefixes | 7.1 | covered |  |
| A line goes to the world in focus | 7.1 | covered |  |
| An empty line | 7.1 | covered |  |
| Enter with no world | 7.1 | covered |  |
| A refused line | 7.1 | covered |  |
| Next world | 7.1 | covered |  |
| Wrap around | 7.1 | covered |  |
| Next world wraps at the end | 7.1 | covered |  |
| Keys with no world | 7.1 | covered |  |
| One world keeps the focus | 7.1 | covered |  |
| Activity mark | 7.1 | covered |  |
| A lost stream | 7.3 | covered |  |
| The client resumes before a new event | 7.1 | covered |  |
| Quit | 7.3 | covered |  |

### Not run

Plan phase: no verification runs.
