---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-19T22:12:15+00:00
head: 6b9cca003948ce9e84dce8b42eda70873e6ad4ef
round: 5
next_id: 72
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
| R-67 | test-coverage | should-fix | in-change | delta scenario 'A bad configuration stops the start' has no matrix row | review > absent-row — "A bad configuration stops the start" | add the row with its covering task or test |  |  | unmet-requirement |
| R-68 | test-coverage | should-fix | in-change | delta scenario 'A fake world with a configured id' has no matrix row | review > absent-row — "A fake world with a configured id" | add the row with its covering task or test |  |  | unmet-requirement |
| R-69 | test-coverage | should-fix | in-change | delta scenario 'The secret is not read at the start' has no matrix row | review > absent-row — "The secret is not read at the start" | add the row with its covering task or test |  |  | unmet-requirement |
| R-70 | test-coverage | should-fix | in-change | delta scenario 'An autoconnect world whose secret is missing' has no matrix row | review > absent-row — "An autoconnect world whose secret is missing" | add the row with its covering task or test |  |  | unmet-requirement |
| R-71 | test-coverage | should-fix | in-change | delta scenario 'A write that hangs' has no matrix row | review > absent-row — "A write that hangs" | add the row with its covering task or test |  |  | unmet-requirement |

## Repeats

| id | lens | severity | claim | evidence |
|---|---|---|---|---|
| R-64 | test-coverage | should-fix | covered row 'The game closes the connection' has no verification run | review > not-run — "The game closes the connection" |
| R-65 | test-coverage | should-fix | covered row 'A world that waits to reconnect' has no verification run | review > not-run — "A world that waits to reconnect" |
| R-66 | test-coverage | should-fix | covered row 'A world that logs in by hand' has no verification run | review > not-run — "A world that logs in by hand" |

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-31 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |
| R-32 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |
| R-33 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |
| R-34 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |
| R-35 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |
| R-36 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |
| R-55 | suggestion | deferred | Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. | https://github.com/seanb4t/mu-portal/issues/105 |
| R-56 | suggestion | deferred | Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. | https://github.com/seanb4t/mu-portal/issues/105 |
| R-57 | suggestion | deferred | Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. | https://github.com/seanb4t/mu-portal/issues/105 |
| R-59 | suggestion | deferred | Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. | https://github.com/seanb4t/mu-portal/issues/105 |
| R-61 | suggestion | deferred | Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. | https://github.com/seanb4t/mu-portal/issues/105 |
| R-63 | suggestion | deferred | Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. | https://github.com/seanb4t/mu-portal/issues/105 |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | a32c34daafa7a31407dcf7c92e0d67910d3a1522 | e439794dc29725eff48a8d3a420dd26c7e3a50aa73bbb92763cd879d5bf2abc8 | changes-requested | 36 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 42c490126b69c4fd643395340e27409ea483bf2d | 88f74e9b2b52e841a79fbefd78846432b739436477847f7668a3172514bc07d4 | approve-with-notes | 11 | 0 | 1 | 27 | 0 |  |
| 3 | implementation | full | e1ff199f16da2b4eeadd597d9b431849b0fd81b7 | 617b0e8d0b1a0fe60fa8a6cef3189d650f65850d181fb2ffd90b542dd962b040 | approve-with-notes | 19 | 0 | 0 | 0 | 0 |  |
| 4 | implementation | follow-up | 410683bb7fa8cef6e86c8782e736e7ca4d640c7d | 7731db6da7b540f3332c1f9df835e7ab58ea2048f1b8dde8a7771c2baebdec76 | approve-with-notes | 0 | 3 | 0 | 11 | 0 |  |
| 5 | implementation | follow-up | 6b9cca003948ce9e84dce8b42eda70873e6ad4ef | d5a90507d70dea997ed11bca9d343623a90f4d53da76bfd127c29f97893060e9 | approve-with-notes | 5 | 3 | 0 | 0 | 0 |  |

## Judge

judge: 0 questions, 0.000 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| A command under its call | crates/mu-cli/tests/telemetry.rs: command_under_its_call | covered | pass |
| A command of the server | crates/mu-cli/tests/telemetry.rs: command_of_the_server | covered | pass |
| A command that fails | crates/mu-cli/tests/telemetry.rs: failed_command_span_is_error | covered | pass |
| The connection of a fake world | crates/mu-cli/tests/telemetry.rs: connection_of_a_fake_world | covered | pass |
| A connect that fails | crates/mu-cli/tests/telemetry.rs: failed_connect_span_is_error | covered | pass |
| The game closes the connection | crates/mu-cli/tests/telemetry.rs: closed_connection_span_is_not_error; crates/mu-cli/tests/connection.rs: game_close_reconnects | covered | not-run |
| A connection that is lost | crates/mu-cli/tests/telemetry_engine.rs: lost_connection_span_is_error | covered | pass |
| A command past the depth limit | crates/mu-cli/tests/telemetry_engine.rs: command_past_the_depth_limit | covered | pass |
| A command over the rate | crates/mu-cli/tests/telemetry_engine.rs: command_over_the_rate | covered | pass |
| Commands at the rate | crates/mu-cli/tests/telemetry_engine.rs: commands_at_the_rate | covered | pass |
| A world that never connects | crates/mu-cli/tests/telemetry.rs: world_that_never_connects | covered | pass |
| The state at the stop | crates/mu-cli/tests/telemetry.rs: state_at_the_stop | covered | pass |
| The keepalive lines | crates/mu-cli/tests/telemetry_engine.rs: keepalive_lines_are_counted | covered | pass |
| A world that waits to reconnect | crates/mu-cli/tests/telemetry_engine.rs: gauge_shows_reconnecting; crates/mu-cli/tests/worlds.rs: list_worlds_shows_reconnecting; crates/mu-core/tests/connection.rs: send_to_reconnecting_world | covered | not-run |
| Each built-in type reads as quirks | crates/mu-core/tests/world_config.rs: built_in_types_hold_only_quirks | covered | pass |
| A global preference is not hidden by a type | crates/mu-cli/tests/world_config.rs: global_preference_is_not_hidden_by_a_type | covered | pass |
| A user type with a built-in name | crates/mu-cli/tests/world_config.rs: user_type_with_a_built_in_name | covered | pass |
| An unknown field | crates/mu-cli/tests/world_config.rs: unknown_field | covered | pass |
| A port out of range | crates/mu-cli/tests/world_config.rs: port_out_of_range | covered | pass |
| A template with no password | crates/mu-cli/tests/world_config.rs: template_with_no_password | covered | pass |
| The case of a character set | crates/mu-cli/tests/world_config.rs: charset_case | covered | pass |
| A character set that the engine cannot read | crates/mu-cli/tests/world_config.rs: charset_the_engine_cannot_read | covered | pass |
| An unknown mode | crates/mu-cli/tests/world_config.rs: unknown_mode | covered | pass |
| The edges of each range | crates/mu-cli/tests/world_config.rs: edges_of_each_range | covered | pass |
| The edges of the attempts | crates/mu-cli/tests/world_config.rs: edges_of_the_attempts | covered | pass |
| A line break in a line field | crates/mu-cli/tests/world_config.rs: line_break_in_a_line_field | covered | pass |
| A plugin sets a field of the connection | crates/mu-core/tests/world_config.rs: plugin_cannot_set_the_connection | covered | pass |
| A plugin sets a preference | crates/mu-core/tests/world_config.rs: plugin_can_set_a_preference | covered | pass |
| A world with no host or port | crates/mu-cli/tests/world_config.rs: world_with_no_host_or_port | covered | pass |
| A login name with no secret | crates/mu-cli/tests/world_config.rs: login_name_with_no_secret | covered | pass |
| A login secret with no name | crates/mu-cli/tests/world_config.rs: login_secret_with_no_name | covered | pass |
| Half a keepalive | crates/mu-cli/tests/world_config.rs: half_a_keepalive | covered | pass |
| Half a keepalive that is off | crates/mu-cli/tests/world_config.rs: half_keepalive_that_is_off | covered | pass |
| A mode with no keepalive | crates/mu-cli/tests/world_config.rs: mode_with_no_keepalive | covered | pass |
| A world that logs in by hand | crates/mu-cli/tests/world_config.rs: world_that_logs_in_by_hand; crates/mu-core/tests/connection.rs: no_login_sends_no_line | covered | not-run |
| The defaults | crates/mu-cli/tests/world_config.rs: connection_defaults | covered | pass |
| A first wait over the cap | crates/mu-cli/tests/world_config.rs: first_wait_over_the_cap | covered | pass |
| A first wait equal to the cap | crates/mu-cli/tests/world_config.rs: first_wait_equal_to_the_cap | covered | pass |
| A world reaches its game | crates/mu-cli/tests/connection.rs: world_reaches_its_game | covered | pass |
| Nothing listens on the port | crates/mu-cli/tests/connection.rs: nothing_listens_on_the_port | covered | pass |
| A connect that hangs | crates/mu-core/tests/connection.rs: connect_that_hangs | covered | pass |
| A TLS world | crates/mu-cli/tests/connection.rs: tls_world_is_refused | covered | pass |
| A Latin-1 game | crates/mu-cli/tests/connection.rs: latin1_game | covered | pass |
| The window size of the world | crates/mu-cli/tests/connection.rs: window_size_of_the_world | covered | pass |
| A missing variable | crates/mu-cli/tests/connection.rs: missing_secret_variable_at_connect | covered | pass |
| The login line hides the password | crates/mu-cli/tests/connection.rs: login_line_hides_the_password | covered | pass |
| A secret that is gone at a reconnect | crates/mu-core/tests/connection.rs: secret_gone_at_reconnect_stops | covered | pass |
| A banner and then quiet | crates/mu-core/tests/connection.rs: login_after_banner_and_quiet | covered | pass |
| A silent game | crates/mu-core/tests/connection.rs: login_after_a_silent_second | covered | pass |
| A game that never goes quiet | crates/mu-core/tests/connection.rs: login_cap_for_a_noisy_game | covered | pass |
| A send during the quiet second | crates/mu-core/tests/connection.rs: send_during_quiet_second_is_refused | covered | pass |
| Idle | crates/mu-core/tests/connection.rs: keepalive_idle | covered | pass |
| Interval | crates/mu-core/tests/connection.rs: keepalive_interval | covered | pass |
| Off | crates/mu-core/tests/connection.rs: keepalive_off | covered | pass |
| The journal holds no keepalive | crates/mu-core/tests/connection.rs: journal_holds_no_keepalive | covered | pass |
| No keepalive before the login | crates/mu-core/tests/connection.rs: no_keepalive_before_login | covered | pass |
| The player disconnects | crates/mu-cli/tests/connection.rs: player_disconnect_does_not_reconnect | covered | pass |
| Reconnect off | crates/mu-cli/tests/connection.rs: reconnect_off_stays_disconnected | covered | pass |
| The waits of a game that refuses every connect | crates/mu-core/tests/connection.rs: waits_grow_until_the_last_attempt | covered | pass |
| The extra time at its cap | crates/mu-core/src/world/mod.rs: extra_time_reaches_its_cap | covered | pass |
| The last attempt | crates/mu-core/tests/connection.rs: world_stops_after_last_attempt | covered | pass |
| A game that closes each connection after the login | crates/mu-core/tests/connection.rs: short_connections_count_as_attempts | covered | pass |
| No limit | crates/mu-core/tests/connection.rs: attempts_zero_never_stops | covered | pass |
| A stable connection starts the count again | crates/mu-core/tests/connection.rs: stable_connection_resets_the_count | covered | pass |
| A short connection keeps the count | crates/mu-core/tests/connection.rs: short_connection_keeps_the_count | covered | pass |
| Connect during a wait | crates/mu-core/tests/connection.rs: connect_during_a_wait | covered | pass |
| Disconnect during a wait | crates/mu-core/tests/connection.rs: disconnect_during_a_wait | covered | pass |
| The fake world answers | crates/mu-cli/tests/worlds.rs: fake_world_answers | covered | pass |
| The fake world answers a mark in order | crates/mu-fake-server/src/memory.rs: fake_game_answers_mark_in_order | covered | pass |
| The flag is hidden | crates/mu-cli/tests/worlds.rs: fake_world_flag_is_hidden | covered | pass |
| A missing password variable | crates/mu-cli/tests/worlds.rs: missing_password_variable | covered | pass |
| A fake world does not reconnect | crates/mu-cli/tests/worlds.rs: fake_world_does_not_reconnect | covered | pass |
| Two characters on one game | crates/mu-core/tests/portal.rs: two_characters_on_one_game | covered | pass |
| An unknown world | crates/mu-core/tests/portal.rs: send_to_unknown_world | covered | pass |
| A world that is not connected | crates/mu-core/tests/portal.rs: send_to_disconnected_world | covered | pass |
| A write that fails | crates/mu-core/tests/portal.rs: failed_write_records_disconnect | covered | pass |
| A world that is still connecting | crates/mu-core/tests/portal.rs: send_to_connecting_world | covered | pass |
| An empty line | crates/mu-core/tests/portal.rs: empty_send_reaches_game | covered | pass |
| A carriage return in the text | crates/mu-core/tests/portal.rs: send_with_cr_is_refused | covered | pass |
| A line break in the text | crates/mu-core/tests/portal.rs: send_with_lf_is_refused | covered | pass |
| Two worlds connect | crates/mu-cli/tests/worlds.rs: two_worlds_connect | covered | pass |
| A configured world waits | crates/mu-cli/tests/worlds.rs: configured_world_waits | covered | pass |
| A configured world that connects at the start | crates/mu-cli/tests/worlds.rs: autoconnect_world_connects_at_start | covered | pass |
| A configured world connects on request | crates/mu-cli/tests/worlds.rs: configured_world_connects_on_request | covered | pass |
| A failed connect at the start | crates/mu-cli/tests/worlds.rs: failed_autoconnect_does_not_stop_start | covered | pass |
| A wait outlasts the quiet before the login | crates/mu-core/tests/connection.rs: wait_covers_pending_login | covered | pass |
| A wait on a world that waits to reconnect | crates/mu-core/tests/connection.rs: wait_returns_during_reconnect_wait | covered | pass |

### Not run

- The game closes the connection: no verification run names this row
- A world that waits to reconnect: no verification run names this row
- A world that logs in by hand: no verification run names this row

## Verification runs

- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=command_under_its_call)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=command_of_the_server)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=failed_command_span_is_error)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=connection_of_a_fake_world)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=failed_connect_span_is_error)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=closed_connection_span_is_not_error)'`
- exit 0: `cargo nextest run -p mu-cli --test connection --locked -E 'test(=game_close_reconnects)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry_engine --locked -E 'test(=lost_connection_span_is_error)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry_engine --locked -E 'test(=command_past_the_depth_limit)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry_engine --locked -E 'test(=command_over_the_rate)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry_engine --locked -E 'test(=commands_at_the_rate)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=world_that_never_connects)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry --locked -E 'test(=state_at_the_stop)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry_engine --locked -E 'test(=keepalive_lines_are_counted)'`
- exit 0: `cargo nextest run -p mu-cli --test telemetry_engine --locked -E 'test(=gauge_shows_reconnecting)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=list_worlds_shows_reconnecting)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=send_to_reconnecting_world)'`
- exit 0: `cargo nextest run -p mu-core --test world_config --locked -E 'test(=built_in_types_hold_only_quirks)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=global_preference_is_not_hidden_by_a_type)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=user_type_with_a_built_in_name)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=unknown_field)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=port_out_of_range)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=template_with_no_password)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=charset_case)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=charset_the_engine_cannot_read)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=unknown_mode)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=edges_of_each_range)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=edges_of_the_attempts)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=line_break_in_a_line_field)'`
- exit 0: `cargo nextest run -p mu-core --test world_config --locked -E 'test(=plugin_cannot_set_the_connection)'`
- exit 0: `cargo nextest run -p mu-core --test world_config --locked -E 'test(=plugin_can_set_a_preference)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=world_with_no_host_or_port)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=login_name_with_no_secret)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=login_secret_with_no_name)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=half_a_keepalive)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=half_keepalive_that_is_off)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=mode_with_no_keepalive)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=world_that_logs_in_by_hand)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=no_login_sends_no_line)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=connection_defaults)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=first_wait_over_the_cap)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=first_wait_equal_to_the_cap)'`
- exit 0: `cargo nextest run -p mu-cli --test connection --locked -E 'test(=world_reaches_its_game)'`
- exit 0: `cargo nextest run -p mu-cli --test connection --locked -E 'test(=nothing_listens_on_the_port)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=connect_that_hangs)'`
- exit 0: `cargo nextest run -p mu-cli --test connection --locked -E 'test(=tls_world_is_refused)'`
- exit 0: `cargo nextest run -p mu-cli --test connection --locked -E 'test(=latin1_game)'`
- exit 0: `cargo nextest run -p mu-cli --test connection --locked -E 'test(=window_size_of_the_world)'`
- exit 0: `cargo nextest run -p mu-cli --test connection --locked -E 'test(=missing_secret_variable_at_connect)'`
- exit 0: `cargo nextest run -p mu-cli --test connection --locked -E 'test(=login_line_hides_the_password)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=secret_gone_at_reconnect_stops)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=login_after_banner_and_quiet)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=login_after_a_silent_second)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=login_cap_for_a_noisy_game)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=send_during_quiet_second_is_refused)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=keepalive_idle)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=keepalive_interval)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=keepalive_off)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=journal_holds_no_keepalive)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=no_keepalive_before_login)'`
- exit 0: `cargo nextest run -p mu-cli --test connection --locked -E 'test(=player_disconnect_does_not_reconnect)'`
- exit 0: `cargo nextest run -p mu-cli --test connection --locked -E 'test(=reconnect_off_stays_disconnected)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=waits_grow_until_the_last_attempt)'`
- exit 0: `cargo nextest run -p mu-core --lib --locked -E 'test(/(^|::)extra_time_reaches_its_cap$/)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=world_stops_after_last_attempt)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=short_connections_count_as_attempts)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=attempts_zero_never_stops)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=stable_connection_resets_the_count)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=short_connection_keeps_the_count)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=connect_during_a_wait)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=disconnect_during_a_wait)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=fake_world_answers)'`
- exit 0: `cargo nextest run -p mu-fake-server --lib --locked -E 'test(/(^|::)fake_game_answers_mark_in_order$/)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=fake_world_flag_is_hidden)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=missing_password_variable)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=fake_world_does_not_reconnect)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=two_characters_on_one_game)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=send_to_unknown_world)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=send_to_disconnected_world)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=failed_write_records_disconnect)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=send_to_connecting_world)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=empty_send_reaches_game)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=send_with_cr_is_refused)'`
- exit 0: `cargo nextest run -p mu-core --test portal --locked -E 'test(=send_with_lf_is_refused)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=two_worlds_connect)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=configured_world_waits)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=autoconnect_world_connects_at_start)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=configured_world_connects_on_request)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=failed_autoconnect_does_not_stop_start)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=wait_covers_pending_login)'`
- exit 0: `cargo nextest run -p mu-core --test connection --locked -E 'test(=wait_returns_during_reconnect_wait)'`

## Dropped

None.

## Lens notes

- verify: prior.json holds no row with the disposition fixed, so fixes is empty.
- verify: Every range of diff-hunks.txt outside review.md was read at head: session.rs, error.rs, portal.rs, transport.rs, world/layer.rs, world/mod.rs, scripted.rs, server.rs, and the changed tests of mu-core and mu-cli. No regression was found.
- verify: The write timeout of send_bytes covers every write of the session: write_all and flush occur only inside it, and write_line, write_mark and the replies of on_bytes all pass through it.
- verify: on_bytes keeps recording the lines and marks of a read after a failed reply, skips each later reply, and calls down with Cause::Lost after the loop, as the commit names.
- verify: The doc comments changed by a1f0574 match the code at head: WorldState::Connecting, NotConnected, NoSecret, WorldSetup::new, connect_at_start, KeepaliveFields.seconds, Keepalive::with_mode, Step::Silent and configured_setup.
- verify: source_rules.rs at head applies the telnet rules only to mu-core and only the sleep rule to mu-fake-server; session.rs holds one sleep_until and scripted.rs one sleep, the counts that ALLOWED names.
- verify: The doc comment of configured_setup in crates/mu-cli/src/server.rs breaks its last sentence over two lines after 'A TLS world'. It is cosmetic and has no consequence.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","5. The TCP transport"],"It resolves the host and connects inside one `tokio::time::timeout` of 10 seconds"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The scenario A connect that hangs needs a 10-second timeout on a transport that never ends its connect on a paused clock, but decision 5 puts the timeout inside TcpTransport only, and the engine tests do not use TcpTransport. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","Requirement: A wait of the engine covers a pending login"],"The session SHALL answer a control frame only after the login of its world went out."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | No decision makes a control frame wait for a pending login. Decision 1 covers frames in Waiting only, and the session at head writes the marks of waiting frames right after on_connect, which a deferred login would then precede. |
| R-3 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-connection/spec.md","Requirement: The login waits until the game is quiet"],"Before the login line, the world SHALL send the game only the replies of the telnet protocol."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | app-3: No decision says what a Send, a script command, a keepalive deadline or a mark does while the login is pending. At head Send writes whenever the connection is up, so the rule that only telnet replies go out before the login has no producing decision.; inv-4: The session is Connected and accepts Send and Login as soon as the stream opens, so a player or script line can reach the game during the quiet wait; no decision or task guards writes while the login is pending, only control frames. |
| R-4 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","6. The secret is a source, read at each connect"],"A `Connect` reads the source before it changes the state."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | app-4: Decision 6 reads the secret only on a Connect request. An automatic attempt from Waiting has no caller, and no decision says whether it reads the secret or what a failed read does, though the spec reads it at each connect and forbids a reconnect after a failed read.; inv-10: The secret is read only on a Connect request, before a state change; a reconnect attempt from Waiting starts in progress with no Connect, so no decision says it reads the secret or what a failed read does there, though the spec forbids a reconnect after it. |
| R-5 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","9. The configuration fields"],"`keepalive` as an enum: `Idle(Keepalive)`, `Interval(Keepalive)` or `Off`"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  | severity? should-fix 0.61 | app-5: The keepalive enum of decision 9 has no case for the default mode idle with no command and no seconds, the case of aresmush and of an untyped world. Such a world cannot hold mode idle, so mu world show cannot print mode = idle with the source default.; inv-2: With the default keepalive.mode = idle, a world whose layers set neither keepalive half (every aresmush world) has no value of the enum Idle(Keepalive) \| Interval(Keepalive) \| Off; no rule says it resolves to Off or fails, and an explicit mode = interval with no command is silently accepted. |
| R-6 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-connection/spec.md","Requirement: A world stops after its last attempt"],"`reconnect.attempts = 0` SHALL mean no limit."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | app-6: No decision produces reconnect.attempts = 0 as no limit, nor the stop after the last attempt with its disconnect event. Decisions 3 and 4 name the attempts field and the wait only.; inv-7: reconnect.attempts = 0 means no limit, but only the config edge test passes 0; no engine test runs a world with attempts = 0, so a check that stops at once on 0 passes every planned test. |
| R-7 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","11. Telemetry (#95)"],"Each attempt is a new `Conn::Connecting`, so each attempt already opens its own `world.connection` span."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | Decision 11 says each attempt already opens its own world.connection span because it is a new Conn::Connecting. At head the span is made in the Connect command handler, not by Conn::Connecting, so an attempt that the Waiting timer starts gets no span from that code. |
| R-8 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","Migration Plan"],"Rollback is a revert of the change."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The rollback is a revert only. A configuration that sets autoconnect, keepalive.mode or reconnect then fails as an unknown field, and the server does not start. The plan has no step for that configuration. |
| R-9 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","6. The secret is a source, read at each connect"],"A `Connect` reads the source before it changes the state. A failed read returns a new `PortalError::NoSecret` that names the variable, and records nothing."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  | severity? blocker 0.61 | Decision 6 reads the secret of a configured world at each connect, but names neither load_password nor its checks, so a read through SecretStore::get skips the mask and line-break guards that the fake world gets. |
| R-10 | 1 | 2 | should-fix | deviates-design | drift | ["artifact",["design.md","5. The TCP transport"],"A world with `tls = true` gets no transport. Its `Connect` fails with a new `PortalError::TlsNotSupported`"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  | scope? in-change 0.95 | TlsNotSupported is a second error for a world with no transport. After the change the TLS world is the only world without a transport, so NoTransport and its text 'only fake worlds can connect' stay beside it and say something false. |
| R-11 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","6. The secret is a source, read at each connect"],"A failed read returns a new `PortalError::NoSecret` that names the variable, and records nothing."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | NoSecret is a second error for a login that has no secret. Decision 6 does not say what happens to NoPassword and its two checks in the session, so two errors with two texts cover one failure. |
| R-12 | 1 | 4 | should-fix | deviates-design | drift | ["artifact",["design.md","9. The configuration fields"],"`reconnect` as an enum: `Backoff { initial, max, attempts }` or `Off`"] | resolved | resolved in round 4 from fixed 5f858dc7810237d1b347f6ac2be1bc3dd945c869 |  | scope? deviates-design 0.64 | Decision 9 adds keepalive and reconnect enums to ResolvedWorld whose cases repeat the kinds of KeepalivePolicy and ReconnectPolicy in decision 3, field for field. A new kind then needs an edit in both, and the map between them in WorldSetup. |
| R-13 | 1 | 2 | should-fix | deviates-design | drift | ["artifact",["design.md","2. Why a world goes down decides whether it reconnects"],"The same cause gives the value of `mu.connection.end`, and `Player` and `Script` both map to `player`."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  | scope? in-change 0.71 | The causes Player, Script and Stopped repeat Origin: a Disconnect already carries Origin::Player, Origin::Script or, from Portal::stop, Origin::System. A second enum for the origin of a disconnect can drift from Origin. |
| R-14 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Scenario: The edges of each range"],"the exit status is 0 for 1 and the top of the range"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The planned edge test sets reconnect.initial = 3600 alone, but the default reconnect.max = 60 and the rule initial <= max reject it, so the scenario's exit status 0 at the top edge cannot hold. |
| R-15 | 1 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-connection/spec.md","Requirement: Connect and Disconnect act on a world that waits"],"`Connect` SHALL start an attempt at once and start the count again."] | resolved | resolved in round 4 from fixed 886c13e26c0f562b4f013436302270a8bea0026a |  |  | Connect in a wait must start the count again, but the scenario only asserts a connection within 1 second; an implementation that keeps the old attempt number passes it. |
| R-16 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-connection/spec.md","Requirement: The login waits until the game is quiet"],"A quiet second is 1 second with no byte from the game, from the connect or from the last byte."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The quiet second counts from the connect when the game sends nothing, but the planned tests cover only a banner, a noisy game and the cap; the empty case (login 1 second after the connect) is untested. |
| R-17 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Scenario: A first wait over the cap"],"the game `alpha` sets `reconnect.initial = 120` and `reconnect.max = 60`"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The rule initial <= max has one planned test, 120 over 60; the equal side (initial = max) is never passed, so a check written as initial >= max passes. |
| R-18 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/telemetry/spec.md","Requirement: A span for each connection of a world"],"A span that ends with `connect_failed`, `timeout` or `lost` SHALL have the status `error`."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The status error is required for timeout and lost as well as connect_failed, but the span tests pass only connect_failed and closed, and the cause map test of task 4.3 covers the attribute, not the status; the lost side of the lost/closed edge is untested. |
| R-19 | 1 | 2 | should-fix | in-change | false-test | ["path","crates/mu-core/src/world/layer.rs",206] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The quirk schema reuses KeepaliveFields, so adding mode to that struct makes built-in types accept keepalive.mode; the only planned test reads the shipped types and passes whether or not the schema refuses mode, autoconnect or reconnect. |
| R-20 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","4. The strategies"],"Add unit tests for the next deadline after a connect, after a write and after a keepalive, for each kind."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The keepalive deadline starts at the connect, and keepalive.seconds can be 1 while the login waits up to 10 seconds, so a keepalive line can precede the login line; nothing orders the two deadlines. |
| R-21 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","4. The wait policy"],"`wait(k) = min(initial * 2^(k-1), max)`"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The wait raises 2 to k-1 while k reaches 10000, or grows without end when attempts = 0; 2^(k-1) passes u32 at k = 33, 3600 * 2^(k-1) passes u64 at k = 54, and the property test stops at k = 50 with initial = 1, so an overflow panic in the session goes untested. |
| R-22 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-connection/spec.md","Requirement: A world stops after its last attempt"],"A connection that stays up for `reconnect.max` seconds SHALL start the count and the wait again from attempt 1."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The reset after a stable connection is tested only at exactly max seconds; no test keeps a connection up for max - 1 seconds and asserts the count does not reset, so a reset on every successful connect also passes. |
| R-23 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","Risks / Trade-offs"],"`reconnect.attempts = 30` by default sets a limit, and the wait grows to `max`."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The design relies on attempts = 30 to end the wrong-password loop, but the spec counts only failed attempts in a row, and a connect that opens and is closed after the login is not a failed attempt; nothing makes a short connection count, so that loop never stops. |
| R-24 | 1 | 2 | should-fix | in-change | next-change | ["artifact",["proposal.md","What Changes"],"**BREAKING** for a client that treats an unknown state as an error."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The proposal marks the new state reconnecting as BREAKING, but no delta spec carries a BREAKING marker. |
| R-25 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","A world has a place to connect"],"After the merge, each world SHALL have `host` and `port`, and `host` SHALL NOT be empty."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The requirement 'A world has a place to connect' has no task line that names its subject. |
| R-26 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Each field is known"],"Each layer SHALL set only the fields of a world."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The requirement 'Each field is known' has no task line with both of its distinctive words; the field list is only partly covered. |
| R-27 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Each value is in its range"],"The range of `port`, `columns` and `rows` SHALL be 1 to 65535."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The requirement 'Each value is in its range' has no task line that names both 'value' and 'range'; the ranges appear only as a bare word. |
| R-28 | 1 | 1 | should-fix | in-change | false-test | ["artifact",["tasks.md","3. The configuration fields"],"Make the quirk schema of the built-in types refuse `keepalive.mode`, `autoconnect` and `reconnect`."] | duplicate | R-19 |  |  | Task 3.3 makes the quirk schema refuse the new fields, but its only check passes if the schema stays open to them. |
| R-29 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","8. Start-up connects"],"A `Connect` that fails at the start goes to the log at `WARN` and does not stop the server."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | Design 8 says a failed Connect at the start only logs and does not stop the server. No scenario and no task test it. |
| R-30 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/telemetry/spec.md","A span for each connection of a world"],"A span that ends with `connect_failed`, `timeout` or `lost` SHALL have the status `error`."] | duplicate | R-18 |  |  | The rule that timeout and lost set the span status error has a test for connect_failed only. Nothing checks the status for those causes. |
| R-31 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","5. The TCP transport"],"It sets `TCP_NODELAY`, because a MU sends short lines."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | TCP_NODELAY in decision 5 answers no requirement, non-goal or risk of the change. |
| R-32 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","12. The crate `mu-fake-server`"],"`mu-core \\u2192 mu-fake-server` goes there, and `mu-fake-server \\u2192 mu-core` and `mu-cli \\u2192 mu-fake-server` go into `ALLOWED_EDGES`."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | Decision 12 adds a crate but follows only half of the procedure of the workspace page: it names ALLOWED_EDGES and the LikeC4 model, not the ALLOWED_CRATES row of mu-fake-server or the crate table and the 'two allowlists' text of that page. |
| R-33 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","4. The wait policy"],"`backon` is already in `[workspace.dependencies]` for `mu-client`."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | Decision 4 reuses backon in mu-core but names only [workspace.dependencies]. backon is allowed for mu-client alone, and the workspace page asks for a row in the second allowlist for each crate that uses it. |
| R-34 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","12. The crate `mu-fake-server`"],"The TCP fake game listens on `127.0.0.1:0` and runs a script of steps: send bytes, wait for bytes, go silent, close."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | Decision 12 lists transcript replay as a new part beside the scripted fake game, but a replay is a script whose steps are sends of the recorded reads. The parameter is the list of steps. |
| R-35 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","3. Three strategies, chosen per world in `WorldSetup`"],"For example, `LoginMoment` gives the deadline of the login from the connect time and the time of the last byte."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | The LoginMoment seam sees only the connect time and the time of the last byte, so the banner trigger that the proposal names as the next kind cannot fit it without a change to the seam and to what the session gives it. |
| R-36 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","3. Three strategies, chosen per world in `WorldSetup`"],"`LoginMoment` gives the deadline of the login from the connect time and the time of the last byte."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | The login deadline over any sequence of byte arrivals is a bound over an unbounded domain: never later than connect + 10 s and never earlier than min(last byte + 1 s, connect + 10 s). |
| R-37 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A connection that is lost"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A connection that is lost' has no matrix row |
| R-38 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A mode with no keepalive"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A mode with no keepalive' has no matrix row |
| R-39 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A first wait equal to the cap"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A first wait equal to the cap' has no matrix row |
| R-40 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A secret that is gone at a reconnect"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A secret that is gone at a reconnect' has no matrix row |
| R-41 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A silent game"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A silent game' has no matrix row |
| R-42 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A send during the quiet second"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A send during the quiet second' has no matrix row |
| R-43 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"No keepalive before the login"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'No keepalive before the login' has no matrix row |
| R-44 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A game that closes each connection after the login"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A game that closes each connection after the login' has no matrix row |
| R-45 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"No limit"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'No limit' has no matrix row |
| R-46 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A short connection keeps the count"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A short connection keeps the count' has no matrix row |
| R-47 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A failed connect at the start"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A failed connect at the start' has no matrix row |
| R-48 | 3 | 4 | should-fix | in-change | drift | ["path","crates/mu-core/src/world/resolve.rs",375] | resolved | resolved in round 4 from fixed 839c1805ba7dabb408eac9936ed2bcc825f8dedd |  |  | The rule that reconnect.initial is not greater than reconnect.max is checked twice with two messages: once in resolve::reconnect and once in check_first_wait, which Backoff::new calls. The resolve copy always runs first, so the other message is unreachable from a configuration. |
| R-49 | 3 | 4 | should-fix | in-change | drift | ["path","crates/mu-cli/tests/connection.rs",16] | resolved | resolved in round 4 from fixed f01e26f76c9fd0a04a736fa1fbaff5149050b3e0 |  | severity? suggestion 0.69 | Three binary test files each build the same configuration of alpha/aria on a TCP fake game: the same game file text and the same character path. A change to that world (for example its type) must be made three times. |
| R-50 | 3 | 4 | should-fix | in-change | drift | ["artifact",["design.md","4. The keepalive and the reconnect are enums"],"with `mode` one of `Idle` and `Interval`"] | resolved | resolved in round 4 from fixed 10dcd1b3d2e8bac69939d3e6651d728fdfe81883 |  |  | A world with no keepalive has two forms: None, which the resolver gives for keepalive.mode = off, and Some(Keepalive) with KeepaliveMode::Off, whose deadline is None. The engine test of the mode off builds the second form, which no configuration produces. |
| R-51 | 3 | 4 | should-fix | in-change | drift | ["artifact",["design.md","1. The session gets a fourth connection state, `Waiting`"],"One function of the session starts every connect: the `Connect` handler and each attempt from `Waiting` call it."] | resolved | resolved in round 4 from fixed f26bdbac6fb0ee674a06e36128c78b2bc987a080 |  |  | The guards of a connect sit in its callers as well as in the connect function: the Connect handler and Session::connect both refuse a world with no transport, and the Connect handler and on_attempt each check the stop flag before they call connect. |
| R-52 | 3 | 4 | should-fix | in-change | drift | ["path","crates/mu-core/src/world/mod.rs",139] | resolved | resolved in round 4 from fixed ba77d71fafde7b93d1bce4e7bb35ac6fbe5b7539 |  |  | The spelling of keepalive.mode and reconnect.mode lives twice: serde rename_all reads and merges it, and a hand-written name() prints it in mu world show. The closest pattern, Charset, keeps one spelling list (name, and from_name over it). |
| R-53 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Requirement: The first wait is not longer than the last"],"After the merge, `reconnect.initial` SHALL NOT be greater than `reconnect.max`. Such a world SHALL be an error that names the world and both fields."] | resolved | resolved in round 4 from fixed 839c1805ba7dabb408eac9936ed2bcc825f8dedd |  |  | The rule that reconnect.initial is not greater than reconnect.max is skipped for reconnect.mode = off: the resolver returns Reconnect::Off before the check, so a world with initial 120, max 60 and mode off resolves with exit status 0. |
| R-54 | 3 | 4 | should-fix | in-change | false-test | ["artifact",["specs/world-connection/spec.md","Requirement: The wait before each attempt grows"],"The extra time SHALL be at least 0 and at most `min(0.05 * k, 0.20)` times that wait."] | resolved | resolved in round 4 from fixed f459673c7dc97b7ed469d92f0b9a9a62fe7935c1 |  |  | The cap of the extra time, min(0.05 * k, 0.20), is never passed at its edge for attempts 1 to 3: the property test starts at attempt 4, and the engine tests that check k = 1 to 3 get one random fastrand draw each, so a share of 0.20 or 0.05 * (k + 1) for small k passes them in a share of runs. |
| R-55 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","1. The session gets a fourth connection state, `Waiting`"],"`Conn` gains `Waiting { attempt, until }`."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/105; reason: Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. |  |  | Decision 1 names the new state Waiting { attempt, until }, but at head Conn::Waiting holds only until, and the count of failed attempts lives on Session as failed. |
| R-56 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","4. The keepalive and the reconnect are enums"],"`mu-core` defines `Keepalive { mode, command, seconds }` with `mode` one of `Idle` and `Interval`"] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/105; reason: Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. |  |  | Decision 4 gives Keepalive a mode of Idle or Interval and ResolvedWorld only keepalive and reconnect; at head KeepaliveMode also has Off, which also exists as keepalive None, and ResolvedWorld gains keepalive_mode. |
| R-57 | 3 | 3 | suggestion | in-change |  | ["artifact",["proposal.md","Impact"],"Dependencies: if a backoff crate fits the growing jitter, it enters through `[workspace.dependencies]`."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/105; reason: Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. |  | severity? should-fix 0.84 | mu-core gains the new dependency fastrand for the random draw of the wait, but the proposal names a dependency only for a backoff crate that fits, and decision 5 names no source of the draw. |
| R-58 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","7. The secret is a source, read at each connect"],"The session keeps the secret of the open connection only, so that `record` can mask the login line."] | resolved | resolved in round 4 from fixed c19314cee3fff0948efe48044fc28c0ed1d3d17d |  | severity? should-fix 0.66 | Decision 7 says the session keeps the secret of the open connection only, but at head down replaces the connection and never clears the stored secret, so the session keeps the secret of the last connect after the connection ends. |
| R-59 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","Open Questions"],"The exact text of the disconnect event after the last attempt, beyond the count that the spec asks for."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/105; reason: Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. |  |  | The open question on the text of the disconnect event after the last attempt is already answered by the code, which appends 'and the world stops after N attempts' to the reason. |
| R-60 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","3. The login moment is a strategy, and the login gates the state `connected`"],"At the login line, the world goes to `connected` and the keepalive deadline starts."] | duplicate | R-62 |  |  | Decision 3 moves the world to connected at the login line, but logged_in sets Connected and records the connect event before it writes the login line, so a failed login write leaves a connected event with no login. |
| R-61 | 3 | 3 | suggestion | in-change |  | ["path","crates/mu-cli/tests/workspace_boundary.rs",210] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/105; reason: Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. |  |  | allowed_dev is allowed_crates with another table: both find the member in a (&str, &[&str]) table and give an empty slice when it is absent. The parameter is the table. |
| R-62 | 3 | 4 | suggestion | in-change |  | ["artifact",["specs/world-connection/spec.md","Requirement: The login waits until the game is quiet"],"Until the login line goes out, the world SHALL stay in the state `connecting`."] | resolved | resolved in round 4 from fixed 40cf900e40e47384741c9e54c85f92f8115e070e |  | severity? blocker 0.75 | logged_in moves the world to connected and records the connected event before it writes the login line, so during the write the state is connected, and a login write that fails leaves a connected state and event for a connection that never logged in. |
| R-63 | 3 | 3 | suggestion | in-change |  | ["path","crates/mu-core/src/world/mod.rs",228] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/105; reason: Suggestions of the world-sessions implementation review: design drift to record and test ideas, none of which changes behavior; deferred to one follow-up issue. |  |  | Keepalive::deadline is a bound over any times of connect, last write and last keepalive: for idle it is the latest of them plus the period, for interval the last keepalive or the connect plus the period, never earlier than connected + period. Four and three table rows test it. |
| R-64 | 3 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"The game closes the connection"] | refuted | Each test of these rows ran at head with exit 0 (verification-runs.txt): closed_connection_span_is_not_error, game_close_reconnects, gauge_shows_reconnecting, list_worlds_shows_reconnecting, send_to_reconnecting_world, world_that_logs_in_by_hand, no_login_sends_no_line. run_keys does not split a covered_by cell that joins several tests with '; ', so no run matched the whole cell. |  |  | covered row 'The game closes the connection' has no verification run |
| R-65 | 3 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"A world that waits to reconnect"] | refuted | Each test of these rows ran at head with exit 0 (verification-runs.txt): closed_connection_span_is_not_error, game_close_reconnects, gauge_shows_reconnecting, list_worlds_shows_reconnecting, send_to_reconnecting_world, world_that_logs_in_by_hand, no_login_sends_no_line. run_keys does not split a covered_by cell that joins several tests with '; ', so no run matched the whole cell. |  |  | covered row 'A world that waits to reconnect' has no verification run |
| R-66 | 3 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"A world that logs in by hand"] | refuted | Each test of these rows ran at head with exit 0 (verification-runs.txt): closed_connection_span_is_not_error, game_close_reconnects, gauge_shows_reconnecting, list_worlds_shows_reconnecting, send_to_reconnecting_world, world_that_logs_in_by_hand, no_login_sends_no_line. run_keys does not split a covered_by cell that joins several tests with '; ', so no run matched the whole cell. |  |  | covered row 'A world that logs in by hand' has no verification run |
| R-67 | 5 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A bad configuration stops the start"] | refuted | Tool gap: a follow-up round reuses the coverage matrix of the last full round, so scenarios added after it have no row. Each has a passing test at 6b9cca0: R-67 worlds.rs bad_configuration_stops_the_start, R-68 worlds.rs fake_world_with_a_configured_id, R-69 worlds.rs server_does_not_read_a_configured_secret, R-70 worlds.rs autoconnect_world_with_a_missing_secret (task 7.3), R-71 mu-core connection.rs write_that_hangs (task 5.6). |  |  | delta scenario 'A bad configuration stops the start' has no matrix row |
| R-68 | 5 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A fake world with a configured id"] | refuted | Tool gap: a follow-up round reuses the coverage matrix of the last full round, so scenarios added after it have no row. Each has a passing test at 6b9cca0: R-67 worlds.rs bad_configuration_stops_the_start, R-68 worlds.rs fake_world_with_a_configured_id, R-69 worlds.rs server_does_not_read_a_configured_secret, R-70 worlds.rs autoconnect_world_with_a_missing_secret (task 7.3), R-71 mu-core connection.rs write_that_hangs (task 5.6). |  |  | delta scenario 'A fake world with a configured id' has no matrix row |
| R-69 | 5 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The secret is not read at the start"] | refuted | Tool gap: a follow-up round reuses the coverage matrix of the last full round, so scenarios added after it have no row. Each has a passing test at 6b9cca0: R-67 worlds.rs bad_configuration_stops_the_start, R-68 worlds.rs fake_world_with_a_configured_id, R-69 worlds.rs server_does_not_read_a_configured_secret, R-70 worlds.rs autoconnect_world_with_a_missing_secret (task 7.3), R-71 mu-core connection.rs write_that_hangs (task 5.6). |  |  | delta scenario 'The secret is not read at the start' has no matrix row |
| R-70 | 5 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An autoconnect world whose secret is missing"] | refuted | Tool gap: a follow-up round reuses the coverage matrix of the last full round, so scenarios added after it have no row. Each has a passing test at 6b9cca0: R-67 worlds.rs bad_configuration_stops_the_start, R-68 worlds.rs fake_world_with_a_configured_id, R-69 worlds.rs server_does_not_read_a_configured_secret, R-70 worlds.rs autoconnect_world_with_a_missing_secret (task 7.3), R-71 mu-core connection.rs write_that_hangs (task 5.6). |  |  | delta scenario 'An autoconnect world whose secret is missing' has no matrix row |
| R-71 | 5 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A write that hangs"] | refuted | Tool gap: a follow-up round reuses the coverage matrix of the last full round, so scenarios added after it have no row. Each has a passing test at 6b9cca0: R-67 worlds.rs bad_configuration_stops_the_start, R-68 worlds.rs fake_world_with_a_configured_id, R-69 worlds.rs server_does_not_read_a_configured_secret, R-70 worlds.rs autoconnect_world_with_a_missing_secret (task 7.3), R-71 mu-core connection.rs write_that_hangs (task 5.6). |  |  | delta scenario 'A write that hangs' has no matrix row |

## Reviewed files

| path | content |
|---|---|
| Cargo.lock | ce7ad59ac5a19280e089dc2314bfd096f1988051 |
| Cargo.toml | daf03cabb354f3bb4714bc98f452015fb8d71c25 |
| architecture/model.c4 | ae0dadffc865aca8b849b7cea380383f76008310 |
| architecture/views.c4 | daa87e6e451d5b46c6e26f2a5a3dc01768a45d2c |
| crates/mu-cli/Cargo.toml | 64a1351c960e8946c582237f139cc1919390cad8 |
| crates/mu-cli/src/server.rs | 4fdd6073aecb31d4cc71825006e72dc1115b9e6f |
| crates/mu-cli/src/world.rs | 6c0912c61fe1e68054b3d54957098b80e091b728 |
| crates/mu-cli/tests/connection.rs | 95c415d9998ba9cb83a18ef408f342eafa09c9a5 |
| crates/mu-cli/tests/fixtures/base/crates/mu-fake-server/Cargo.toml | b378743ba1cff09436ca741d2ebc3bfcbdcd2e70 |
| crates/mu-cli/tests/fixtures/base/crates/mu-fake-server/src/lib.rs | e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 |
| crates/mu-cli/tests/source_rules.rs | a3928d98743034b5824864eebcc23bd9cd906ab3 |
| crates/mu-cli/tests/support/mod.rs | 85ed622654dfb50b59d339edd11f08a563a2952e |
| crates/mu-cli/tests/support/process.rs | f935d81746f5057d3cc96c5319ea8547b7297131 |
| crates/mu-cli/tests/telemetry.rs | 60c79e0d01b5689d9ec4c1ef4d5c5b196e63c368 |
| crates/mu-cli/tests/telemetry_engine.rs | e503d2f61438944b2d8209453020873956a0a163 |
| crates/mu-cli/tests/workspace_boundary.rs | a61f50256bd8fe31fdb4f4d50e0a79fbaedb10d5 |
| crates/mu-cli/tests/world_config.rs | 29dd9f111298bc668bfbf366aa10f51b92d3e3eb |
| crates/mu-cli/tests/worlds.rs | ee5ebb1773035b8b4564dd43d514db3a9e608261 |
| crates/mu-core/Cargo.toml | c996e90ef5e7923ab6864e8a74df8d9e053281b2 |
| crates/mu-core/proptest-regressions/world/mod.txt | cc35c8f4a6c6fd88db13e57e26c00ba8cfc9df25 |
| crates/mu-core/src/connection.rs | 201ea3e74eae86e7804614f8c259a8f7314dbcb5 |
| crates/mu-core/src/error.rs | 3d11f5ae47fdcd34714b1d9ab7995ec1578251e9 |
| crates/mu-core/src/fake.rs | absent |
| crates/mu-core/src/framer.rs | b89771990a7df1af6183c3253a5b4375d25cb882 |
| crates/mu-core/src/lib.rs | 2d622cbc260b135704d7b58f27c339be8ac3636f |
| crates/mu-core/src/portal.rs | a586b1af2abfa97f4a6942e01b614ff9e1177f5b |
| crates/mu-core/src/secret.rs | 09fd0f66b7b61350b4e001c6e07c916e49b99170 |
| crates/mu-core/src/session.rs | 56f3de5d05bec637d4908f5cc1c4a6c17a3f1d83 |
| crates/mu-core/src/transport.rs | e6341984e008f561356832c3b4b9e88687e2a527 |
| crates/mu-core/src/world/builtin.rs | f7fcf7a280c8608660f6cb7a3407e7ef4041272f |
| crates/mu-core/src/world/check.rs | ab6388ad333119dc93bd99f73365573770944205 |
| crates/mu-core/src/world/layer.rs | c02790cbee87cef93f5ef0c1c4101f3ec7f8f253 |
| crates/mu-core/src/world/mod.rs | 1954394a17707181c94fb3915f26bc31e5b65965 |
| crates/mu-core/src/world/resolve.rs | 02a5fea79c3bde4c1b27f86a8dfd63b55a1a4aaf |
| crates/mu-core/tests/connection.rs | 4191425115c5dbda11f5320cbb643c7bf1d58b49 |
| crates/mu-core/tests/portal.rs | 9a61831c4d2bc0ab009ba69ab1366b9a061bc71c |
| crates/mu-core/tests/scripts.rs | 51e4f40594db4b5274286ad0aaf7f00979f66c26 |
| crates/mu-core/tests/support/mod.rs | 97f24fd6ab116bde71e2225a7679fd8b6c435027 |
| crates/mu-core/tests/telnet_transcripts.rs | d53ef3d06453ef1a92c738a81e472b3b44a1c3aa |
| crates/mu-core/tests/world_config.rs | 8e6a7e8f2145c87422d5cb6bcdf8abcf37e495ac |
| crates/mu-fake-server/Cargo.toml | 409490e038dec47cd1bd666dc1fc704cea6e0170 |
| crates/mu-fake-server/src/lib.rs | b3ceb28cd0c37b92a260a2da473a7628f1e4cae3 |
| crates/mu-fake-server/src/memory.rs | 3e8248029c91d1fc42b9c177bfe308c5cd8d2c6f |
| crates/mu-fake-server/src/scripted.rs | d1b76c5fadfae846bafc9be23f020875ccd5c261 |
| crates/mu-proto/proto/mu/v1/portal.proto | fb544c2312001e1632d566f4333976137b4312b5 |
| crates/mu-server/src/convert.rs | 0102074139aeb6f15c53dd869ac051a2148377a8 |
| crates/mu-server/src/serve.rs | 3e3d548737c5197919149e1307129c1c620ad481 |
| crates/mu-server/src/service.rs | b3a61e9fd6d25849fedb60fe34da690e0104b06e |
| docs/src/content/docs/architecture/crate-layout.mdx | 93bceaad4e89dedb7d5d3b2530d3aac87aa7ccd2 |
| docs/src/content/docs/contributing/workspace.md | 240fcd05687f6f1ea8ffb646a01ca69abaa4528e |
| docs/src/content/docs/decisions/telemetry.md | ef9722478437c74ce85f32d48e32458a71334cb8 |
| docs/src/content/docs/guides/configuration.md | 0fce77c32be61941961a9e0f8e31de6514b1fcd8 |
| docs/src/content/docs/guides/first-run.md | 81ba59c4a11693fbce62a9d25ab4148f4b2947a9 |
| docs/src/content/docs/guides/telemetry.md | 576b135b321ccb033f7991c3ca50e0f5d0b5f90a |
| openspec/changes/world-sessions/.openspec.yaml | 56d16f001e73f64dfad7c20b1dc221e8c7dcba66 |
| openspec/changes/world-sessions/design.md | dfa9f4ba20fc1d2be014215dcc3b09739b428b6b |
| openspec/changes/world-sessions/proposal.md | 68f7a4f1e743aff4af369fcaa36e5b91b5a1a7d2 |
| openspec/changes/world-sessions/specs/telemetry/spec.md | 9f61875617bb0cd60e29a380eddf245ce81b5dea |
| openspec/changes/world-sessions/specs/world-config/spec.md | 7720ab7fce097cfd9b1d1a1b64b87f047e292dab |
| openspec/changes/world-sessions/specs/world-connection/spec.md | ee7d2b81323fa7648ccb01058228375f2d4364a0 |
| openspec/changes/world-sessions/specs/world-sessions/spec.md | b5e9e9b99681d4334c9ff3299e543bd010e8a893 |
| openspec/changes/world-sessions/tasks.md | sha256:22817e22d15f06ab03e88f7536bf172c41cf5d355ac92f8d5eda6aadc0796184 |
| openspec/review.json | absent |

## Plan review (2026-11-19)

Verdict approve-with-notes at head 42c490126b69c4fd643395340e27409ea483bf2d.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-37 | test-coverage | should-fix | in-change | delta scenario 'A connection that is lost' has no matrix row | review > absent-row — "A connection that is lost" | add the row with its covering task or test |  |  | unmet-requirement |
| R-38 | test-coverage | should-fix | in-change | delta scenario 'A mode with no keepalive' has no matrix row | review > absent-row — "A mode with no keepalive" | add the row with its covering task or test |  |  | unmet-requirement |
| R-39 | test-coverage | should-fix | in-change | delta scenario 'A first wait equal to the cap' has no matrix row | review > absent-row — "A first wait equal to the cap" | add the row with its covering task or test |  |  | unmet-requirement |
| R-40 | test-coverage | should-fix | in-change | delta scenario 'A secret that is gone at a reconnect' has no matrix row | review > absent-row — "A secret that is gone at a reconnect" | add the row with its covering task or test |  |  | unmet-requirement |
| R-41 | test-coverage | should-fix | in-change | delta scenario 'A silent game' has no matrix row | review > absent-row — "A silent game" | add the row with its covering task or test |  |  | unmet-requirement |
| R-42 | test-coverage | should-fix | in-change | delta scenario 'A send during the quiet second' has no matrix row | review > absent-row — "A send during the quiet second" | add the row with its covering task or test |  |  | unmet-requirement |
| R-43 | test-coverage | should-fix | in-change | delta scenario 'No keepalive before the login' has no matrix row | review > absent-row — "No keepalive before the login" | add the row with its covering task or test |  |  | unmet-requirement |
| R-44 | test-coverage | should-fix | in-change | delta scenario 'A game that closes each connection after the login' has no matrix row | review > absent-row — "A game that closes each connection after the login" | add the row with its covering task or test |  |  | unmet-requirement |
| R-45 | test-coverage | should-fix | in-change | delta scenario 'No limit' has no matrix row | review > absent-row — "No limit" | add the row with its covering task or test |  |  | unmet-requirement |
| R-46 | test-coverage | should-fix | in-change | delta scenario 'A short connection keeps the count' has no matrix row | review > absent-row — "A short connection keeps the count" | add the row with its covering task or test |  |  | unmet-requirement |
| R-47 | test-coverage | should-fix | in-change | delta scenario 'A failed connect at the start' has no matrix row | review > absent-row — "A failed connect at the start" | add the row with its covering task or test |  |  | unmet-requirement |
| R-15 | verify | should-fix | in-change | REGRESSED: the fix 5b0f3ae026beba48cc7a36ed5c229139adb07a12 of R-15 did not hold. Connect in a wait must start the count again, but the scenario only asserts a connection within 1 second; an implementation that keeps the old attempt number passes it. | specs/world-connection/spec.md > Scenario: Connect during a wait — "the wait after that refused connect is at most 1.05 seconds" | fix it again | regression |  | unmet-requirement |

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| The fake world answers | 2.2 | covered |  |
| The fake world answers a mark in order | 2.2 | covered |  |
| The flag is hidden | 2.2 | covered |  |
| A missing password variable | 5.1 | covered |  |
| A fake world does not reconnect | 6.2 | covered |  |
| Two characters on one game | 9.1 | covered |  |
| An unknown world | 9.1 | covered |  |
| A world that is not connected | 9.1 | covered |  |
| A write that fails | 9.1 | covered |  |
| A world that is still connecting | 9.1 | covered |  |
| A world that waits to reconnect | 5.4, 7.2, 8.2 | covered |  |
| An empty line | 9.1 | covered |  |
| A carriage return in the text | 9.1 | covered |  |
| A line break in the text | 9.1 | covered |  |
| Two worlds connect | 7.1 | covered |  |
| A configured world waits | 7.1 | covered |  |
| A configured world that connects at the start | 7.1 | covered |  |
| A configured world connects on request | 7.1 | covered |  |
| A wait outlasts the quiet before the login | 5.2 | covered |  |
| A wait on a world that waits to reconnect | 5.4 | covered |  |
| Each built-in type reads as quirks | 3.3 | partial |  |
| A global preference is not hidden by a type | 9.1 | covered |  |
| A user type with a built-in name | 9.1 | covered |  |
| An unknown field | 9.1 | covered |  |
| A port out of range | 9.1 | covered |  |
| A template with no password | 9.1 | covered |  |
| The case of a character set | 9.1 | covered |  |
| A character set that the engine cannot read | 9.1 | covered |  |
| An unknown mode | 3.1 | covered |  |
| The edges of each range | 3.1 | covered |  |
| The edges of the attempts | 3.1 | covered |  |
| A line break in a line field | 9.1 | covered |  |
| A plugin sets a field of the connection | 3.3 | covered |  |
| A plugin sets a preference | 9.1 | covered |  |
| A world with no host or port | 9.1 | covered |  |
| A login name with no secret | 9.1 | covered |  |
| A login secret with no name | 9.1 | covered |  |
| Half a keepalive | 3.2 | covered |  |
| Half a keepalive that is off | 3.2 | covered |  |
| A world that logs in by hand | 5.2, 9.1 | covered |  |
| The defaults | 3.1 | covered |  |
| A first wait over the cap | 3.1 | covered |  |
| A command under its call | 9.1 | covered |  |
| A command of the server | 9.1 | covered |  |
| A command that fails | 8.1 | covered |  |
| The connection of a fake world | 8.1 | covered |  |
| A connect that fails | 8.1 | covered |  |
| The game closes the connection | 6.2, 8.1 | covered |  |
| A command past the depth limit | 9.1 | covered |  |
| A command over the rate | 9.1 | covered |  |
| Commands at the rate | 9.1 | covered |  |
| A world that never connects | 8.2 | covered |  |
| The state at the stop | 8.2 | covered |  |
| The keepalive lines | 8.2 | covered |  |
| A world reaches its game | 6.1 | covered |  |
| Nothing listens on the port | 6.1 | covered |  |
| A connect that hangs | 5.4 | covered |  |
| A TLS world | 6.1 | covered |  |
| A Latin-1 game | 6.1 | covered |  |
| The window size of the world | 6.1 | covered |  |
| A missing variable | 6.1 | covered |  |
| The login line hides the password | 6.1 | covered |  |
| A banner and then quiet | 5.2 | covered |  |
| A game that never goes quiet | 5.2 | covered |  |
| Idle | 5.3 | covered |  |
| Interval | 5.3 | covered |  |
| Off | 5.3 | covered |  |
| The journal holds no keepalive | 5.3 | covered |  |
| The player disconnects | 6.2 | covered |  |
| Reconnect off | 6.2 | covered |  |
| The waits of a game that refuses every connect | 5.4 | covered |  |
| The extra time at its cap | 4.3 | covered |  |
| The last attempt | 5.4 | covered |  |
| A stable connection starts the count again | 5.4 | covered |  |
| Connect during a wait | 5.4 | covered |  |
| Disconnect during a wait | 5.4 | covered |  |

### Not run

Plan phase: no verification runs.
