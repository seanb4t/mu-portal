---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-23T14:55:45+00:00
head: be93e0fa97b328b69bbce3b5c7dc4aa6e962a716
round: 22
next_id: 186
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: ok
  test-coverage: partial
  test-honesty: ok
  invariants: ok
  verify: not-applicable
---
# Review

## Incomplete lenses

- test-coverage: partial, gaps: 2
  - Assertions read in full only for the command, quit and key-binding tests; the rest are graded by test name plus task text, not by reading each THEN assertion.
  - Tests were not run, by rule.

The verdict covers only what the lenses checked. While the `ctl-lens` row of an incomplete lens is open, the verdict cannot be `approve`.

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-178 | fit | should-fix | in-change | PortalClient::config_dir repeats the body of PortalClient::list_worlds: the same ListWorlds request and call, keeping the other half of one response. Two methods of one RPC can change apart, and mu client makes the call twice after a start. | crates/mu-client/src/lib.rs:95 — "pub async fn list_worlds(&self) -> Result<Vec<proto::World>, ClientError> {" | Give mu-client one method that makes the ListWorlds call and returns the whole reply (worlds and config_dir); make list_worlds and config_dir project it, or drop config_dir and read config_dir from that reply in mu-cli. |  |  | drift |
| R-179 | fit | should-fix | in-change | The word for an argument kind is written in three places: help and arity each match ArgKind to 'world', and ClientConfig::keymap writes 'needs a world' for any required argument without reading its kind. | crates/mu-tui/src/update.rs:533 — "ArgKind::World => format!("{shown} needs a world")," | Add one ArgKind method in registry.rs that gives the noun of a kind, and build the help text, the arity hint and the keymap refusal from it. |  |  | drift |
| R-180 | fit | should-fix | in-change | The change adds five linear scans that find a world by its id (Model::history, Model::walk, the Focus arm, on_reply, put_back), beside the sorted binary search of Model::world. No one lookup without insert exists, so each copy keeps its own match rule. | crates/mu-tui/src/model.rs:296 — ".find(\|each\| each.id == world)" | Add Model::index_of(id) -> Option<usize> over the binary search that Model::world uses, let Model::world call it, and replace the five scans with it. |  |  | drift |
| R-181 | invariants | should-fix | in-change | put_back pushes the held text first and then each returned text, so with 1000 or more texts behind a refused one the pushes drop the held draft and the last returned line, and the count overstates what Up gives. | crates/mu-tui/src/update.rs:980 — "let mut kept = usize::from(entry.history.push(&held));" | Keep the held text and the count true at the limit: cap the returned texts that go to history at HISTORY_LIMIT - 1 (or count the entries left after the pushes), and add a test that refuses the first of 1001 lines sent with Alt+Enter and asserts the draft and the count. |  |  | wrong-behaviour |
| R-185 | test-coverage | should-fix | in-change | lens test-coverage reported partial, gaps: 2 | review > lens-status — "test-coverage partial" | rerun the lens or accept the gap in the review |  |  | unmet-requirement |
| R-182 | test-honesty | should-fix | in-change | history_keeps_its_invariants never checks the 1000-entry cap that task 4.1 says it checks, and 60 steps cannot reach 1000 entries, so a broken cap passes it. | crates/mu-tui/tests/app.rs:2059 — "prop_assert!(history.iter().all(\|entry\| !entry.is_empty()));" | Add a len() <= 1000 assert in the loop, or change task 4.1 to say that history_drops_the_oldest_past_1000 alone covers the cap. |  |  | false-test |
| R-183 | approach | suggestion | in-change | D8 compares config_dir only on an attach to a server that the client did not start. At head mu client compares after every first list, also after it started a server itself, and design.md records no deviation. | design.md > D8. The screen opens before the first list — "On an attach to a server that it did not start, `mu client` resolves its own directory the same way and compares." | Change the D8 sentence: mu client compares after each first list, also after it started a server, because a concurrent client can start the server that wins the lock. |  | severity? should-fix 0.75 |  |
| R-184 | approach | suggestion | in-change | D8 says the list before run decides whether to start a server. At head a bare UnixStream connect decides, and the list comes after it; design.md does not name the probe. | design.md > D8. The screen opens before the first list — "The list that `mu-cli` makes before `run` decides whether to start a server." | Record in D8 that a connect to the socket decides whether to start a server, and that the list with the wait follows it. |  | severity? should-fix 0.71 |  |

## Repeats

None.

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-37 | suggestion | deferred | Low-cost polish of the client-shell plan: the dim-echo note in the guide, a shared reader for top-level config files, and runnable test commands in tasks; deferred to one ticket by the owner. | https://github.com/seanb4t/mu-portal/issues/164 |
| R-43 | suggestion | deferred | Low-cost polish of the client-shell plan: the dim-echo note in the guide, a shared reader for top-level config files, and runnable test commands in tasks; deferred to one ticket by the owner. | https://github.com/seanb4t/mu-portal/issues/164 |
| R-172 | suggestion | deferred | The usage text of an argument moves into the registry when client-layout adds the palette, which is its first second reader; until then help is the only reader. | https://github.com/seanb4t/mu-portal/issues/167 |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | cd0f860fa27eb1117378b9c54cf28fff7f1499a8 | 0cc27524b56d703022f85cd2f165b79dc07d5bd9b3417e5a48d6adc5b4474e0e | changes-requested | 43 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | c41b481932f355567bb67947a1f2746b0c929502 | 8f785eb1e63002f496fc05f4bbf2d5535e91ced2c90b207b05f699ac434a5253 | changes-requested | 18 | 0 | 2 | 38 | 0 |  |
| 3 | plan | full | 4bc3384056bc7da30e7d21fd73db25dd1a2bba18 | 93a87287460d2a735e6b2510440b10d48c5950b7056af635108e28d4c94664d7 | changes-requested | 23 | 0 | 1 | 0 | 0 | round 1 added 18 scenarios that have no coverage matrix rows; the owner wants them checked, not accepted |
| 4 | plan | follow-up | 5f99bebdc25626b6b0e0fbdcc8f6d9a9e6ceaade | 126ba40dc35f91b67a41add84ab405309af36f117c815bfe2ee6e8e10e4d0616 | approve-with-notes | 6 | 0 | 1 | 42 | 0 |  |
| 5 | plan | full | 3d30709e6a6becbd7982e1636251721ac15741ea | ce8df9d75a010006686fd27a11d766243c8ff64a6a6ca600e628fdc584811b9c | changes-requested | 19 | 0 | 0 | 0 | 0 | rounds 3 and 4 added scenarios with no coverage matrix rows; the owner wants them checked, not accepted |
| 6 | plan | full | b1628e7f5b7c6fcaba52b69e13606093397ac2cd | 979bd7b0ea4485a85d60b1cdf745ee153b0e53089ac004530786e3ec76389611 | changes-requested | 16 | 0 | 0 | 0 | 0 | round 5 fixes added scenarios that need coverage rows; seanb4t/dotfiles#256 tracks the loop |
| 7 | plan | full | 8ef2ecd711fb484c618e777634a47575ceae05ba | a92b3bd2eeb3965d0c915907445014de6f273cd2c561c470cabd0ac2e59a059b | changes-requested | 15 | 0 | 0 | 0 | 0 | Round 6 fixes add scenarios (dim echo, control character, missing client directory) that only a full round puts in the coverage matrix (seanb4t/dotfiles#256) |
| 8 | plan | follow-up | 869d413a9652e90e8a9a081e62aa2acba52d080e | ec62eff47dbdc3cc42c5044dbe309094d9d6dc632b47a24a3e214739bd7b0a15 | approve-with-notes | 0 | 0 | 2 | 57 | 0 |  |
| 9 | plan | follow-up | a099e007418226878bc8c5d43209ac8f238393ef | e50cb84f88e44bb81bcb1e6adad817b5414d8596995f77bff47eb510f5d99a2e | approve-with-notes | 0 | 0 | 1 | 1 | 0 |  |
| 10 | plan | follow-up | 8ecf9a8204c6010e36f9a929639650242c8560bf | 6019de0da81b5a9f906f44ca7993a7d318310a14bc36e6e6a8da663ed112f5ce | approve-with-notes | 1 | 0 | 1 | 0 | 0 |  |
| 11 | plan | follow-up | d4e2273fb97303757ceb064999c94fc10cd2ada7 | d155389c6dd9637865283a87d9c4ef198c60e59af3c5bd748be91d99d5caa5c1 | approve-with-notes | 1 | 0 | 0 | 2 | 0 |  |
| 12 | plan | full | adb7ef9d45bd8d7f221e824fefe213f7a69a316f | 717676bdfe6a7c46a6f743aa7b43adee44b311a201cf97ec3e18000fa308e6af | changes-requested | 12 | 0 | 0 | 0 | 0 | The owner chose a pose editor for each world, which rewrites D5 and D6 and adds and replaces scenarios |
| 13 | plan | follow-up | 662c14c418afd9f5272f5e40564bd4bb42a4493c | 0267b079207336632636ada23ccb88ff247bf939a1a83507356762e4b2bccf23 | approve | 0 | 0 | 1 | 12 | 0 |  |
| 14 | plan | follow-up | 4b1a72f74a568163ca7369da4134df092b37a019 | bae8b166475602911b08e66a59c847eff7cf6ba6f285df8794204fe49b9d479e | approve | 0 | 0 | 0 | 1 | 0 |  |
| 15 | plan | follow-up | aa9ff96f02f0900ac84fedbc20975c18eb50e3e1 | 2a30ae0de66fb5d7674bfa757bf17c2729078b9880e061dee6f154658bb0669e | approve | 0 | 0 | 0 | 0 | 0 |  |
| 16 | plan | follow-up | b6d8c06fca2698b34fb7bfc34531302cc4fe4bb5 | 14fed7022fa01074c56e7e084d04c1ba3621099e039dbd17c0cd095edf1e9abb | approve-with-notes | 1 | 0 | 0 | 0 | 0 |  |
| 17 | plan | follow-up | 40216b11e741569700fa4bd7d509bbde86bf5037 | 8932bb09d711081f038f9302bd99f449108fdfa8d4c08b760e42fba287bdd5d3 | approve | 0 | 0 | 0 | 1 | 0 |  |
| 18 | implementation | full | 7c2b15c0c15e93779d84997bad0fe7e6b544bb8e | efe6f8accbb9a4b8affd2837b461fe1ec9a672e3aa441be8a9eba566d56c9c2d | changes-requested | 18 | 0 | 0 | 0 | 0 |  |
| 19 | implementation | follow-up | 2790b3cac0e01bae2b24ccc58cd327c3ef88f6c8 | cc60356d45d236c9b474b7b592f08ffb8cd87a1907e5094fc7f4de1c49e662da | approve-with-notes | 0 | 1 | 1 | 17 | 0 |  |
| 20 | implementation | follow-up | 65a9a51fb21f2fdbd08291585c663f1adbb7dfa8 | cc60356d45d236c9b474b7b592f08ffb8cd87a1907e5094fc7f4de1c49e662da | approve-with-notes | 0 | 1 | 0 | 1 | 0 |  |
| 21 | implementation | follow-up | 518e8a5b0a7075488e2a37d495fba82867aa98ae | 9856775cf557a23f9b7f7a5f272c3ba19e8daf6bcd3b318a3e296909c6130fad | approve-with-notes | 4 | 1 | 0 | 0 | 0 |  |
| 22 | implementation | full | be93e0fa97b328b69bbce3b5c7dc4aa6e962a716 | da066dd74257ac9dbcd42d84b302b7ed3c023f2c1bf0d36f68ea2a9269daef07 | approve-with-notes | 8 | 0 | 0 | 0 | 0 | new scenarios for the notice about another server directory need coverage rows |

## Judge

judge: 14 questions, 0.001 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| Help lists the entries | crates/mu-tui/tests/app.rs: help_lists_the_entries | covered | pass |
| A key and a command give the same focus | crates/mu-cli/tests/client.rs: a_key_and_a_command_give_the_same_focus | covered | pass |
| An editing entry is not a command | crates/mu-tui/tests/app.rs: an_editing_entry_is_not_a_command | covered | pass |
| A command does not reach the game | crates/mu-tui/tests/app.rs: a_command_does_not_reach_the_game | covered | pass |
| Several lines are not a command | crates/mu-tui/tests/app.rs: several_lines_are_not_a_command | covered | pass |
| An escaped slash | crates/mu-tui/tests/app.rs: an_escaped_slash | covered | pass |
| An unknown command | crates/mu-tui/tests/app.rs: an_unknown_command | covered | pass |
| A missing world | crates/mu-tui/tests/app.rs: a_missing_world | covered | pass |
| An argument to an entry that takes none | crates/mu-tui/tests/app.rs: an_argument_to_an_entry_that_takes_none | covered | pass |
| An unknown world | crates/mu-tui/tests/app.rs: an_unknown_world | covered | pass |
| Connect the world in focus | crates/mu-cli/tests/client.rs: connect_the_world_in_focus | covered | pass |
| Disconnect a named world | crates/mu-cli/tests/client.rs: disconnect_a_named_world | covered | pass |
| A key bound to connect with no world in focus | crates/mu-cli/tests/client.rs: a_key_bound_to_connect_with_no_world_in_focus | covered | pass |
| Connect with no world in focus | crates/mu-tui/tests/app.rs: connect_with_no_world_in_focus | covered | pass |
| A trailing space is no argument | crates/mu-cli/tests/client.rs: a_trailing_space_is_no_argument | covered | pass |
| Disconnect an unknown world | crates/mu-cli/tests/client.rs: disconnect_an_unknown_world | covered | pass |
| Focus by name | crates/mu-tui/tests/app.rs: focus_by_name | covered | pass |
| A world with a trailing space | crates/mu-tui/tests/app.rs: a_world_with_a_trailing_space | covered | pass |
| Quit | crates/mu-cli/tests/client.rs: quit_by_command | covered | pass |
| Quit twice while a text waits | crates/mu-cli/tests/client.rs: quit_twice_while_a_text_waits | covered | pass |
| Another command cancels the quit | crates/mu-cli/tests/client.rs: another_command_cancels_the_quit | covered | pass |
| A command with no world in focus | crates/mu-tui/tests/app.rs: a_command_with_no_world_in_focus | covered | pass |
| A file that sets the echo | crates/mu-cli/tests/client.rs: a_file_that_sets_the_echo | covered | pass |
| A file that sets dim | crates/mu-cli/tests/client.rs: a_file_that_sets_dim | covered | pass |
| No file | crates/mu-cli/tests/client.rs: no_file | covered | pass |
| An unknown key | crates/mu-cli/tests/client.rs: an_unknown_key | covered | pass |
| A bad echo value | crates/mu-cli/tests/client.rs: a_bad_echo_value | covered | pass |
| A file that cannot be read | crates/mu-cli/tests/client.rs: a_file_that_cannot_be_read | covered | pass |
| Not TOML | crates/mu-cli/tests/client.rs: not_toml | covered | pass |
| A new binding | crates/mu-cli/tests/client.rs: a_new_binding | covered | pass |
| The default keys stay | crates/mu-cli/tests/client.rs: the_default_keys_stay | covered | pass |
| A removed binding | crates/mu-cli/tests/client.rs: a_removed_binding | covered | pass |
| A binding replaces a default | crates/mu-cli/tests/client.rs: a_binding_replaces_a_default | covered | pass |
| None on a key with a modifier | crates/mu-cli/tests/client.rs: none_on_a_key_with_a_modifier | covered | pass |
| Two names of one key | crates/mu-cli/tests/client.rs: two_names_of_one_key | covered | pass |
| A binding of Shift+Tab | crates/mu-cli/tests/client.rs: a_binding_of_shift_tab | covered | pass |
| An uppercase key | crates/mu-cli/tests/client.rs: an_uppercase_key | covered | pass |
| A control character | crates/mu-cli/tests/client.rs: a_control_character | covered | pass |
| Shift with a digit | crates/mu-cli/tests/client.rs: shift_with_a_digit | covered | pass |
| A binding of Ctrl+Shift+Z | crates/mu-cli/tests/client.rs: a_binding_of_ctrl_shift_z | covered | pass |
| An unknown entry | crates/mu-cli/tests/client.rs: an_unknown_entry | covered | pass |
| An unknown key name | crates/mu-cli/tests/client.rs: an_unknown_key_name | covered | pass |
| An entry that needs an argument | crates/mu-cli/tests/client.rs: an_entry_that_needs_an_argument | covered | pass |
| A wait of zero | crates/mu-cli/tests/client.rs: a_wait_of_zero | covered | pass |
| The default dims | crates/mu-tui/tests/app.rs: the_default_dims | covered | pass |
| Hide | crates/mu-tui/tests/app.rs: hide | covered | pass |
| A client with no server starts one | crates/mu-cli/tests/client.rs: client_starts_a_server | covered | pass |
| No server answers in time | crates/mu-cli/tests/client.rs: client_gives_up_after_five_seconds | covered | pass |
| Two clients start at the same time | crates/mu-cli/tests/client.rs: two_clients_start_one_server | covered | pass |
| A server that closes the connection | crates/mu-cli/tests/client.rs: a_server_that_closes_the_connection | covered | pass |
| A server that does not answer | crates/mu-cli/tests/client.rs: a_server_that_does_not_answer | covered | pass |
| The started server finds the default directory | crates/mu-cli/tests/client.rs: the_started_server_finds_the_default_directory | covered | pass |
| The server names its directory | crates/mu-cli/tests/server.rs: the_server_names_its_directory | covered | pass |
| A server with another directory | crates/mu-cli/tests/client.rs: a_server_with_another_directory | covered | pass |
| A server with the same directory | crates/mu-cli/tests/client.rs: a_server_with_the_same_directory | covered | pass |
| Two clients with other directories start at once | crates/mu-cli/tests/client.rs: two_clients_with_other_directories_start_at_once | covered | pass |
| A line goes to the world in focus | crates/mu-tui/tests/app.rs: a_line_goes_to_the_world_in_focus | covered | pass |
| An empty line | crates/mu-tui/tests/app.rs: an_empty_line | covered | pass |
| Enter with no world | crates/mu-tui/tests/app.rs: enter_with_no_world | covered | pass |
| A refused line | crates/mu-tui/tests/app.rs: a_refused_line | covered | pass |
| A refused line after more text | crates/mu-tui/tests/app.rs: a_refused_line_after_more_text | covered | pass |
| Ctrl+M sends | crates/mu-tui/tests/app.rs: ctrl_m_sends | covered | pass |
| A pose of two lines | crates/mu-cli/tests/client.rs: a_pose_of_two_lines | covered | pass |
| A line break that the world refuses | crates/mu-cli/tests/client.rs: a_line_break_that_the_world_refuses | covered | pass |
| A line that the server refuses | crates/mu-tui/tests/app.rs: a_line_that_the_server_refuses | covered | pass |
| The last line is refused | crates/mu-tui/tests/app.rs: the_last_line_is_refused | covered | pass |
| A refused send of Enter with a send after it | crates/mu-tui/tests/app.rs: a_refused_send_of_enter_with_a_send_after_it | covered | pass |
| The sends stay apart | crates/mu-tui/tests/app.rs: the_sends_stay_apart | covered | pass |
| The stream ends while lines wait | crates/mu-tui/tests/app.rs: the_stream_ends_while_lines_wait | covered | pass |
| Lines of two worlds wait | crates/mu-tui/tests/app.rs: lines_of_two_worlds_wait | covered | pass |
| A refusal out of focus | crates/mu-tui/tests/app.rs: a_refusal_out_of_focus | covered | pass |
| The server resets while lines wait | crates/mu-tui/tests/app.rs: the_server_resets_while_lines_wait | covered | pass |
| A refusal during a walk of the history | crates/mu-tui/tests/app.rs: a_refusal_during_a_walk_of_the_history | covered | pass |
| A refusal after the stream ended | crates/mu-tui/tests/app.rs: a_refusal_after_the_stream_ended | covered | pass |
| A send enters the history | crates/mu-tui/tests/app.rs: a_send_enters_the_history | covered | pass |
| A pose is one entry | crates/mu-tui/tests/app.rs: a_pose_is_one_entry | covered | pass |
| No empty entry | crates/mu-tui/tests/app.rs: no_empty_entry | covered | pass |
| Each world has its own history | crates/mu-tui/tests/app.rs: each_world_has_its_own_history | covered | pass |
| No repeat | crates/mu-tui/tests/app.rs: no_repeat | covered | pass |
| An escaped slash in the history | crates/mu-tui/tests/app.rs: an_escaped_slash_in_the_history | covered | pass |
| Down with no walk | crates/mu-tui/tests/app.rs: down_with_no_walk | covered | pass |
| Back to the draft | crates/mu-tui/tests/app.rs: back_to_the_draft | covered | pass |
| Up inside a pose moves the cursor | crates/mu-tui/tests/app.rs: up_inside_a_pose_moves_the_cursor | covered | pass |
| A send during a walk | crates/mu-tui/tests/app.rs: a_send_during_a_walk | covered | pass |
| A focus change keeps each editor | crates/mu-tui/tests/app.rs: a_focus_change_keeps_each_editor | covered | pass |
| A walk stays with its world | crates/mu-tui/tests/app.rs: a_walk_stays_with_its_world | covered | pass |
| The second world | crates/mu-tui/tests/app.rs: the_second_world | covered | pass |
| No world at that place | crates/mu-tui/tests/app.rs: no_world_at_that_place | covered | pass |
| Back to the first world | crates/mu-tui/tests/app.rs: back_to_the_first_world | covered | pass |
| Top and bottom | crates/mu-tui/tests/app.rs: top_and_bottom | covered | pass |
| The echo of a pose of two lines | crates/mu-tui/tests/app.rs: the_echo_of_a_pose_of_two_lines | covered | pass |
| Before the first list | crates/mu-tui/tests/app.rs: before_the_first_list | covered | pass |
| A send before the first list | crates/mu-tui/tests/app.rs: a_send_before_the_first_list | covered | pass |
| Text typed before the first list | crates/mu-tui/tests/app.rs: text_typed_before_the_first_list | covered | pass |
| An event before the first list | crates/mu-tui/tests/app.rs: an_event_before_the_first_list | covered | pass |
| A focus key before the first list | crates/mu-tui/tests/app.rs: a_focus_key_before_the_first_list | covered | pass |
| A failed first list after an event | crates/mu-tui/tests/app.rs: a_failed_first_list_after_an_event | covered | pass |
| A notice before the first list | crates/mu-tui/tests/app.rs: a_notice_before_the_first_list | covered | pass |
| The first list fails | crates/mu-tui/tests/app.rs: the_first_list_fails | covered | pass |
| The flag wins over the environment | crates/mu-cli/tests/world_config.rs: flag_wins_over_the_environment | covered | pass |
| The default directory | crates/mu-cli/tests/world_config.rs: default_directory | covered | pass |
| The directory under the home directory | crates/mu-cli/tests/world_config.rs: directory_under_home | covered | pass |
| An empty MU_CONFIG | crates/mu-cli/tests/world_config.rs: empty_mu_config | covered | pass |
| A relative XDG_CONFIG_HOME | crates/mu-cli/tests/world_config.rs: relative_xdg_config_home | covered | pass |
| No home directory | crates/mu-cli/tests/world_config.rs: no_home_directory | covered | pass |
| A missing directory | crates/mu-cli/tests/world_config.rs: missing_directory | covered | pass |
| The client warns of a missing directory | crates/mu-cli/tests/client.rs: the_client_warns_of_a_missing_directory | covered | pass |
| A missing default directory | crates/mu-cli/tests/world_config.rs: missing_default_directory_is_quiet | covered | pass |
| The client takes the flag | crates/mu-cli/tests/client.rs: the_client_takes_the_flag | covered | pass |
| Worlds from character files | crates/mu-cli/tests/world_config.rs: worlds_from_character_files | covered | pass |
| A character with no game | crates/mu-cli/tests/world_config.rs: character_with_no_game | covered | pass |
| A file name outside the id rule | crates/mu-cli/tests/world_config.rs: file_name_outside_the_id_rule | covered | pass |
| A stray file | crates/mu-cli/tests/world_config.rs: stray_file | covered | pass |
| A link to nothing | crates/mu-cli/tests/world_config.rs: link_to_nothing_is_an_error | covered | pass |
| The file of the server | crates/mu-cli/tests/world_config.rs: server_file_makes_no_world | covered | pass |
| A character named server | crates/mu-cli/tests/world_config.rs: character_named_server_is_a_world | covered | pass |
| The file of the client | crates/mu-cli/tests/world_config.rs: the_file_of_the_client | covered | pass |

### Not run

None.

## Verification runs

- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_bad_echo_value)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_binding_of_ctrl_shift_z)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_binding_of_shift_tab)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_binding_replaces_a_default)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_control_character)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_file_that_cannot_be_read)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_file_that_sets_dim)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_file_that_sets_the_echo)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_key_and_a_command_give_the_same_focus)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_key_bound_to_connect_with_no_world_in_focus)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_line_break_that_the_world_refuses)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_new_binding)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_pose_of_two_lines)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_removed_binding)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_server_that_closes_the_connection)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_server_that_does_not_answer)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_server_with_another_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_server_with_the_same_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_trailing_space_is_no_argument)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_wait_of_zero)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=an_entry_that_needs_an_argument)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=an_unknown_entry)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=an_unknown_key)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=an_unknown_key_name)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=an_uppercase_key)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=another_command_cancels_the_quit)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=client_gives_up_after_five_seconds)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=client_starts_a_server)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=connect_the_world_in_focus)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=disconnect_a_named_world)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=disconnect_an_unknown_world)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=no_file)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=none_on_a_key_with_a_modifier)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=not_toml)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=quit_by_command)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=quit_twice_while_a_text_waits)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=shift_with_a_digit)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_client_takes_the_flag)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_client_warns_of_a_missing_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_default_keys_stay)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_started_server_finds_the_default_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=two_clients_start_one_server)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=two_clients_with_other_directories_start_at_once)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=two_names_of_one_key)'`
- exit 0: `cargo nextest run -p mu-cli --test server --locked -E 'test(=the_server_names_its_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=character_named_server_is_a_world)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=character_with_no_game)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=default_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=directory_under_home)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=empty_mu_config)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=file_name_outside_the_id_rule)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=flag_wins_over_the_environment)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=link_to_nothing_is_an_error)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=missing_default_directory_is_quiet)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=missing_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=no_home_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=relative_xdg_config_home)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=server_file_makes_no_world)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=stray_file)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=the_file_of_the_client)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=worlds_from_character_files)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_command_does_not_reach_the_game)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_command_with_no_world_in_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_failed_first_list_after_an_event)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_focus_change_keeps_each_editor)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_focus_key_before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_line_goes_to_the_world_in_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_line_that_the_server_refuses)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_missing_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_notice_before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_pose_is_one_entry)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_refusal_after_the_stream_ended)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_refusal_during_a_walk_of_the_history)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_refusal_out_of_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_refused_line)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_refused_line_after_more_text)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_refused_send_of_enter_with_a_send_after_it)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_send_before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_send_during_a_walk)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_send_enters_the_history)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_walk_stays_with_its_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_world_with_a_trailing_space)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_argument_to_an_entry_that_takes_none)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_editing_entry_is_not_a_command)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_empty_line)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_escaped_slash)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_escaped_slash_in_the_history)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_event_before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_unknown_command)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_unknown_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=back_to_the_draft)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=back_to_the_first_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=connect_with_no_world_in_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=ctrl_m_sends)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=down_with_no_walk)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=each_world_has_its_own_history)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=enter_with_no_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=focus_by_name)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=help_lists_the_entries)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=hide)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=lines_of_two_worlds_wait)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=no_empty_entry)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=no_repeat)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=no_world_at_that_place)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=several_lines_are_not_a_command)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=text_typed_before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_default_dims)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_echo_of_a_pose_of_two_lines)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_first_list_fails)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_last_line_is_refused)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_second_world)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_sends_stay_apart)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_server_resets_while_lines_wait)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_stream_ends_while_lines_wait)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=top_and_bottom)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=up_inside_a_pose_moves_the_cursor)'`

## Dropped

None.

## Lens notes

- approach: Each requirement of the five delta specs has a decision that produces it: D1 and D4 for client-commands, D2, D3 and D7 for client-config, D8 for server-process, D5 to D8 for terminal-client, D2 and D11 for world-config.
- approach: Decisions D1 to D11 were read against the code at head. The registry, keymap and key names, run_entry and Msg::Run, the command check and quit arm, History and its walk, set_focus and replay_notices, put_back and return_waiting, echo_rows, the first list, config_file::read, warn_missing and the resolver list of top files match their decisions.
- approach: The fixed row R-174 no longer holds at head: mu client now compares config_dir after it starts a server too. app-1 records that D8 still says otherwise.
- approach: The facts of the Context section are stated as of e96b221, not head, so they were not judged as stale citations.
- approach: The Migration Plan has no deployed configuration, so no rollback step is needed.
- fit: The prompt names no path for repository library-preference rows; only the generic rows of the rulebook were applied.
- fit: Keymap::bind and Keymap::bind_wheel, and Keymap::find and Keymap::wheel, are two pairs of the same upsert and lookup over two lists; the pairs were there before the change, which only changed their value type, so no finding is raised.
- fit: The Tab-with-Shift-to-BackTab rule is written in Chord::parse and Chord::of_key; D3 names both sides and shift_tab_matches_back_tab covers them, so no finding is raised.
- fit: Earlier fit rows (spawn_reply, config_file::read, replay_notices, pose_of, in_history, focus_place!, cells_of, echo_rows) were checked at head and hold.
- invariants: I1: A history holds at most 1000 entries, none empty, none equal to the one before it \| enforced at crates/mu-tui/src/history.rs:20 History::push \| boundary test crates/mu-tui/tests/history.rs history_drops_the_oldest_past_1000 (1000 and 1001)
- invariants: I2: The place of an open walk is always the place of an entry \| enforced at history.rs:21, every push clears the walk, and push is the one change of the entries \| boundary test app.rs history_keeps_its_invariants (place < len) and tests/history.rs
- invariants: I3: After a refusal, Up gives the returned texts in order and then the held text, and the count equals the entries Up gives \| enforced at crates/mu-tui/src/update.rs:965 put_back, but not at the history limit (inv-1) \| boundary test none: returned_texts_come_back_in_order draws 2 to 5 texts
- invariants: I4: connect_wait_seconds is finite and at least 1 ns \| enforced at crates/mu-cli/src/client.rs:56 connect_wait \| boundary test client.rs a_wait_that_is_no_time_is_refused (0, -1, 1e-10, 0.9e-9, NaN, inf, f64::MAX refused; 1e-9 gives 1 ns)
- invariants: I5: A key name is modifiers from ctrl, alt, shift, super once each, then one key that is not uppercase, not a control character and not +; shift goes only with a lowercase character or a special key \| enforced at crates/mu-tui/src/keymap.rs:49 Chord::parse \| boundary test tests/keymap.rs bad_key_names_are_refused_with_the_name (f0, f01, f13, empty, +, ctrl+, alt+DEL) and f1_and_f12_are_read, property key_names_read_back
- invariants: I6: Two names of one key, an unknown entry and an entry with a required argument stop the client \| enforced at crates/mu-tui/src/config.rs:102 ClientConfig::keymap \| boundary test client.rs (mu-cli unit) a_binding_error_starts_with_the_file_and_the_key and the client.rs scenario tests
- invariants: I7: client.toml has no unknown key at the top or in [server] \| enforced at crates/mu-tui/src/config.rs:26 and :50 deny_unknown_fields \| boundary test an_unknown_key (top level); no test of an unknown key inside [server], but serde enforces it
- invariants: I8: No message carries an Action, so each front end goes through run_entry \| enforced at crates/mu-tui/src/update.rs:31 Msg (compiler) \| boundary test not applicable
- invariants: I9: Entry names are unique and kebab case \| enforced at registry.rs ENTRIES (static) \| boundary test tests/keymap.rs entry_names_are_unique_and_kebab
- invariants: I10: A command is one line that starts with / and not //, and an argument of only spaces is no argument \| enforced at crates/mu-tui/src/command.rs:29 parse \| boundary test update.rs parser tests for /, /connect with a space, / focus, and property trailing_spaces_are_no_argument
- invariants: I11: FocusIndex outside 1 to the count of worlds does nothing \| enforced at update.rs:593 checked_sub and index < len \| boundary test app.rs no_world_at_that_place (Alt+3 with two worlds)
- invariants: I12: The focus index follows its world when a world is inserted before it \| enforced at crates/mu-tui/src/model.rs:381 \| boundary test app.rs a_world_added_during_a_walk
- invariants: I13: set_focus does nothing while the first list is open \| enforced at model.rs:314 \| boundary test app.rs a_focus_key_before_the_first_list
- invariants: I14: client.toml and server.toml make no world, in any case only as the same file \| enforced at crates/mu-core/src/world/dir.rs is_top_file \| boundary test world_config.rs tests of another case, unreadable and hard link
- invariants: I15: The client and the server resolve the directory the same way before the compare \| enforced at crates/mu-cli/src/config.rs:15 shown_dir, used by server.rs and client.rs \| boundary test mu-cli unit a_missing_directory_is_named_no_directory (None on each side and both), client.rs a_server_with_the_same_directory (symlink)
- invariants: I16: A client that started a server still compares config_dir (R-174) \| enforced at crates/mu-cli/src/client.rs:129, after start_and_wait \| boundary test client.rs two_clients_with_other_directories_start_at_once. R-174 holds at head, so it is not raised again.
- invariants: I17: The echo rows joined with the line break give the cleaned text \| enforced at crates/mu-tui/src/text.rs:85 echo_rows, and model.rs:346 drops an empty line break or one with a control character \| property test echo_rows_join_back
- invariants: Probe: a Python model of History::push and put_back at head, inv-probe/put_back.py under the handoff directory, gives for 999 texts behind kept=1000 with the draft kept, for 1000 behind kept=1001 with 1000 entries and the draft gone, and for 1001 behind kept=1002 with the draft and the last line gone.
- invariants: tasks.md 4.1 says that history_keeps_its_invariants checks at most 1000 entries, but its loop asserts no length and runs at most 60 steps. The bound itself is tested at 1000 and 1001 in tests/history.rs.
- invariants: The enforcement points are Rust code, not scripts, so no other probe ran. No repository test ran.
- test-coverage: Every delta scenario heading maps to a named test; no missing or partial rows found among those read.
- test-honesty: a_file_that_cannot_be_read (client.rs) and unreadable_server_file_in_another_case_names_the_error (world_config.rs) return early with a pass when the user can read a mode 0o000 file, such as root. The tasks name this skip, so it is not raised as disabled; a CI container that runs as root skips both silently.
- test-honesty: The quit-cancel tests (client.rs 900-948) were checked: each ends with a wait on a hint that a client that quit would not show, and the hint of the first quit sits in another pane or is replaced, so they can fail.
- test-honesty gap: crates/mu-tui/tests/editor.rs, tests/history.rs, tests/support/process.rs and the remaining tests of app.rs before line 1504 were not read in full.
- test-honesty gap: Tests of mu-core dir.rs and mu-server were not read.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | fixed_after | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-commands/spec.md","Requirement: Quit by command"],"`/quit` SHALL do what Ctrl+C does, under the requirement \\"Ctrl+C quits the client only\\"."] | resolved | resolved in round 4 from fixed 7b6443877bc61629ce3453f165d7d7729ed95d19 |  |  |  | app-1: /quit cannot confirm a quit while texts wait: the keys that type a second /quit reset quit_armed, and no decision says how a command confirms.; inv-1: /quit can never leave while a send waits: each typed key of a second /quit clears the quit arm, so the double quit of Ctrl+C never completes by command. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-commands/spec.md","Requirement: A bad command keeps its text"],"It can also lack a required world, or name a world that the client does not hold."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | No decision checks the world of /disconnect: D10 checks only /focus and /connect against candidates, so /disconnect with an unknown world is not a bad command. |
| R-3 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-commands/spec.md","Requirement: Connect and disconnect a world"],"With no world and no world in focus, it SHALL be a bad command."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | app-3: No decision makes /connect or /disconnect with no world and no world in focus a bad command; D4 clears the editor and runs act for a good command.; inv-10: The spec makes /connect and /disconnect with no world and no world in focus a bad command, but no scenario or task test passes that null case. |
| R-4 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: A refused text stops the sends after it"],"It SHALL show the reason as a line with the prefix `-- `, and the count of the texts in the history."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | severity? should-fix 0.60 | The refusal notice must hold the count of the texts put in history, but D6 only extends the existing count of the return_waiting notice; the refusal notice at head has no count. |
| R-5 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Scenario: The sends stay apart"],"**AND** after Up the editor holds `c` alone"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | severity? should-fix 0.70 | In 'The sends stay apart' the editor gets the lines a and b through set_input, which puts the cursor at the end, on the last line; by D5 Up there moves the cursor and recalls nothing. |
| R-6 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D2. `mu-cli` reads the file, `mu-tui` owns the type"],"It returns an error that names the key and the value, and `mu-cli` prints that error."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | severity? should-fix 0.78 | The keymap error of D2 names only the key and the value, and mu-cli prints it as 'mu client: <error>', so stderr does not name client.toml for the three [keys] error scenarios. |
| R-7 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. The screen opens before the first list"],"That list has its own 5 s limit."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | D8 says the list that mu-cli makes before run has its own 5 s limit, but the first list_worlds call in client.rs has no limit; only the retry after starting a server does. |
| R-8 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","D2. `mu-cli` reads the file, `mu-tui` owns the type"],"It parses the file with `toml` and `serde_path_to_error`, and gives the value to `mu_tui::run`."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | D2 plans a second TOML parse in mu-cli with toml and serde_path_to_error. mu_core::world::parse_toml already does that parse, with the file:line:column error format that server.toml uses. mu-cli does not depend on serde_path_to_error, and its allowlist row does not hold it. |
| R-9 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["proposal.md","Impact"],"The world resolver accepts `client.toml`."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | scope? beyond-scope 0.73 | fit-3: The proposal puts the world resolver change in mu-cli. The resolver and its one top-level exception, is_server_file with SERVER_FILE, live in mu-core dir.rs. The design names neither that crate nor how the exception takes a second file, so a change kept to the named crates cannot make the scenario The file of the client hold.; app-10: The world resolver that refuses stray top-level files is in mu-core (world/dir.rs), but the proposal's Impact places its change under crates/mu-cli and lists no mu-core change. |
| R-10 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"`Help` adds one notice line for each entry."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | scope? deviates-design 0.72 | D4 makes a bad command and /help add notices. The model has two kinds of -- line. A notice is kept, and shows again after each Reset and for the first world. A hint is for one moment. The help list and a bad command are of the moment, so as notices they come back after each Reset. |
| R-11 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","D8. The screen opens before the first list"],"The model starts with `Listing::Asked` and a flag `first_list`, and the top row shows `listing worlds` until `Msg::Listed` arrives."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | D8 takes the first list through Msg::Listed, but does not say what happens to Model::from_listing. That constructor takes the first list now, and the tests build the model with it. The two paths handle a failed list differently: from_listing adds a notice, and Msg::Listed only sets list_failure. |
| R-12 | 1 | 19 | should-fix | in-change | drift | ["artifact",["design.md","D7. Echo"],"It shows the first part with `> ` and each other part with two spaces."] | resolved | resolved in round 19 from fixed 1c4ee80f96410210fee04a29f94dab11cb87c89d | 18 |  |  | D7 puts the split of a player line and its prefixes in update. text::line already owns the prefix rule of each source and the clean of the text, so the > prefix of a player line would have two places. |
| R-13 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["path","docs/src/content/docs/guides/configuration.md",36] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The configuration guide links each top-level file of the directory to its own page, and says that server.toml is the one exception to the stray-file rule. Task 6.5 adds a page for client.toml, but no task updates the configuration guide, so after the change it tells the player that client.toml is an error. |
| R-14 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-commands/spec.md","Requirement: A bad command keeps its text"],"A bad command names no entry, or an entry that does not run as a command."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | Nothing refuses an argument to an entry that takes none, so `/quit now` or `/focus-next beta/bryn` runs and drops the argument; the bad-command list omits that case. |
| R-15 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"The text after `/` is split at the first run of spaces into a name and an argument."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The D4 split leaves the empty argument undefined: `/connect ` with a trailing space may give Some("") and an unknown world instead of the world in focus; no test passes it. |
| R-16 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/world-config/spec.md","Requirement: The configuration directory"],"The flag `--config <dir>` of `mu server`, `mu world` and `mu client` SHALL name the configuration directory"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | `mu client --config <dir>` starts a server without the flag, so the client reads client.toml of <dir> while the server it starts reads worlds from MU_CONFIG or the XDG directory. |
| R-17 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-core/src/world/dir.rs",107] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The world resolver treats server.toml in any case, unreadable and hard-linked with edge tests, but task 6.3 tests client.toml only by its exact name, so `Client.toml` on a case-insensitive disk or an unreadable client.toml is untested. |
| R-18 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-config/spec.md","Requirement: A bad client.toml stops the client"],"A bad file is one that the client cannot read or parse"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | A client.toml that exists but cannot be read is a bad file, but no scenario or task test passes it; only a missing file and parse errors are tested. |
| R-19 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Scenario: No world at that place"],"**WHEN** the user presses Alt+5"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The focus-by-number test passes Alt+5 with two worlds, not Alt+3, the first place past the last world, where an off-by-one index check would fail. |
| R-20 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: The top row shows the first list"],"a key that sends SHALL show that no world is in focus."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The rule that a key that sends during the first list shows that no world is in focus has no test; before_the_first_list checks only the top row. |
| R-21 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: Each world keeps an input history"],"The client SHALL NOT add an empty text, or a text equal to the newest entry."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The rule that an empty text never enters the history has no test; no_repeat covers only equal texts, and an empty Enter is a normal send. |
| R-22 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7. Echo"],"With `Echo::Hide` the client adds no line, but the event still counts for the unseen mark."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | D7 keeps the unseen mark for a hidden echo, but the hide scenario only checks the focused world, so the mark for a player event of a world not in focus is untested. |
| R-23 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-config/spec.md","Requirement: The keys table binds a key to an entry"],"Each binding SHALL replace the default binding of that key, and `\\"none\\"` SHALL remove it."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | "none" on a modified chord whose plain key is bound falls back to the plain entry, so `"shift+enter" = "none"` would send instead of doing nothing; the only test unbinds ctrl+n, whose plain key has no entry. |
| R-24 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. The screen opens before the first list"],"the top row shows `listing worlds` until `Msg::Listed` arrives."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The first list can end in a failure, which sets only list_failure; no test checks that the top row leaves `listing worlds` then, so the flag of D8 can stay set forever. |
| R-25 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-config/spec.md","Requirement: The keys table binds a key to an entry"],"Each binding SHALL replace the default binding of that key"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | No scenario binds a key that already has a default, so the rule that a binding replaces the default is never tested; a_new_binding uses alt+n, which has no default. |
| R-26 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Key names"],"The order of the modifiers does not matter."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | Two [keys] names of one chord (`ctrl+shift+z` and `shift+ctrl+z`) are distinct BTreeMap keys, so one binding silently wins by key order; nothing refuses the pair. |
| R-27 | 1 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D3. Key names"],"A key is one character or one name from this list:"] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | The key-name grammar has `+` both as the joiner and as a legal one-character key, so `ctrl++` is ambiguous; no task tests `+`, an uppercase character or a repeated modifier. |
| R-28 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. History is a state of each world"],"the place of the walk, and the draft that the first Up saved."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The draft lives in the History of one world while the pose editor is shared, so a focus change during a walk leaves the draft in the old world and the new world's walk saves a recalled entry as its draft. |
| R-29 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. Returned sends go to history (#151)"],"The other returned texts, and then the text that the editor held, go into the history of their world."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | A refusal or a stream end during a walk pushes the recalled entry as 'the text that the editor held', and nothing pushes the saved draft, so the draft is lost; the D5 property test excludes any put_back. |
| R-30 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"A good command clears the editor, adds the text to the history of the world in focus, and runs `act`."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | A good command goes into the history of the world in focus, but focus is None with no world or before the first list ends; no test runs a command in that null case. |
| R-31 | 1 | 2 | should-fix | in-change | next-change | ["artifact",["proposal.md","What Changes"],"**BREAKING** (spec only, no release yet): after a refusal, a timeout, a stream end or a reset, only the first returned send goes back"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The proposal marks the refusal return path as BREAKING, but no delta spec carries a BREAKING marker. |
| R-32 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["specs/world-config/spec.md","Scenario: The client takes the flag"],"**THEN** the client opens the screen"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The THEN step of the scenario 'The client takes the flag' names no observable: 'the client opens the screen' names no row, status or value. |
| R-33 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","2. The registry and one dispatch"],"every test of `crates/mu-tui` still passes"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | Six unchanged scenarios of the modified Enter requirement are named by no task. Only the blanket 2.1 line covers them. |
| R-34 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","7. Echo and the first list"],"`a_pose_of_two_lines` (echo) in `app.rs` pass"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The heading 'A pose of two lines' occurs twice (Enter requirement, echo requirement). Task 7.1 names one test '(echo)', so the %r send scenario has no named test. |
| R-35 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","6. Configuration"],"Make sure that `the_file_of_the_client` in `world_config.rs` passes, and that `a_stray_file` still passes."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | Eleven unchanged world-config scenarios (flag, defaults, env, missing dir, character files, server file) are named by no task. Task 6.3 names only a_stray_file and the_file_of_the_client. |
| R-36 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D3. Key names"],"Repository rule 5rk42v8qs1 asks for a crate first."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | Repository rule 5rk42v8qs1 is not found anywhere in the repository outside an archived review, so the rule that D3 and D10 rely on cannot be read. |
| R-37 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"A player who wants the old look sets `echo = \\"show\\"`. \\u2192 The configuration page says so."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/164; reason: Low-cost polish of the client-shell plan: the dim-echo note in the guide, a shared reader for top-level config files, and runnable test commands in tasks; deferred to one ticket by the owner. |  |  |  | The risk of the new dim default is mitigated by the configuration page, but task 6.5 checks only that the page names /help, not that it states the old look is echo = "show". |
| R-38 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","Context"],"`Action` (`crates/mu-tui/src/update.rs`) is a `Copy` enum of 15 variants with no names."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | Action has 13 variants at head, not 15. |
| R-39 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","Context"],"`mu-cli` already parses TOML with `toml` and `serde_path_to_error` (`crates/mu-cli/src/config.rs`)."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | mu-cli does not parse TOML with serde_path_to_error in config.rs; it calls mu_core::world::parse_toml (server_config.rs), and serde_path_to_error is a dependency of mu-core only. |
| R-40 | 1 | 9 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-cli/src/server_config.rs",63] | resolved | resolved in round 9 from fixed 9982a1170cc3c0bb92c64168b3b217d68eec7ac1 |  |  |  | The reader of client.toml repeats server_config::read. Both join the file name to the directory, give the default for a missing directory or file, and name the path in a read error. Only the file name, the Hidden value and the certificate fix-up of server.toml differ. |
| R-41 | 1 | 2 | suggestion | in-change |  | ["path","crates/mu-tui/src/keymap.rs",53] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | The keymap keeps an Action for each chord, not the registry entry. The command palette of client-layout lists the registry, and it must map each Action back to an entry to show its key. The D1 test that each default action comes from an entry needs the same reverse map. |
| R-42 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D3. Key names"],"A property test checks that it reads back each name that it writes."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | D3 promises a property test that the key-name parser reads back each name it writes, but no task creates it. |
| R-43 | 1 | 1 | suggestion | in-change |  | ["artifact",["tasks.md","3. Client commands"],"Make sure that `connect_the_world_in_focus` and `disconnect_a_named_world` in `client.rs` pass."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/164; reason: Low-cost polish of the client-shell plan: the dim-echo note in the guide, a shared reader for top-level config files, and runnable test commands in tasks; deferred to one ticket by the owner. |  |  |  | Tasks name test functions but only 6.1 gives a runnable command. Each test is named without a cargo nextest line. |
| R-44 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An argument to an entry that takes none"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'An argument to an entry that takes none' has no matrix row |
| R-45 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Connect with no world in focus"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'Connect with no world in focus' has no matrix row |
| R-46 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A trailing space is no argument"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'A trailing space is no argument' has no matrix row |
| R-47 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Disconnect an unknown world"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'Disconnect an unknown world' has no matrix row |
| R-48 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Quit twice while a text waits"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'Quit twice while a text waits' has no matrix row |
| R-49 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A command with no world in focus"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'A command with no world in focus' has no matrix row |
| R-50 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A file that cannot be read"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'A file that cannot be read' has no matrix row |
| R-51 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A binding replaces a default"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'A binding replaces a default' has no matrix row |
| R-52 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"None on a key with a modifier"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'None on a key with a modifier' has no matrix row |
| R-53 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Two names of one key"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'Two names of one key' has no matrix row |
| R-54 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An uppercase key"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'An uppercase key' has no matrix row |
| R-55 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A refusal during a walk of the history"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'A refusal during a walk of the history' has no matrix row |
| R-56 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"No empty entry"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'No empty entry' has no matrix row |
| R-57 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A focus change ends a walk"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'A focus change ends a walk' has no matrix row |
| R-58 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Back to the first world"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'Back to the first world' has no matrix row |
| R-59 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The echo of a pose of two lines"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'The echo of a pose of two lines' has no matrix row |
| R-60 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A send before the first list"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'A send before the first list' has no matrix row |
| R-61 | 2 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The first list fails"] | resolved | resolved in round 4 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  |  | delta scenario 'The first list fails' has no matrix row |
| R-62 | 3 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-config/spec.md","Requirement: A bad binding stops the client"],"an unknown key, an unknown entry, an entry that needs an argument, a repeated modifier, and two names of one key."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  | severity? should-fix 0.77 | No decision makes ClientConfig::keymap() refuse a binding to an entry with a required argument. D3 lists only key-name refusals, and D1 builds a key's Action with no argument. |
| R-63 | 3 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Requirement: The configuration directory"],"If `--config` or `MU_CONFIG` names a missing directory, stderr SHALL name it."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  | severity? should-fix 0.83 | The modified requirement makes mu client name a missing --config or MU_CONFIG directory on stderr, but D2 only gives the defaults for a missing directory, and mu client never calls the warning of ConfigArgs::worlds. |
| R-64 | 3 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: The top row shows the first list"],"Keys typed in that time SHALL edit the editor, and a key that sends SHALL show that no world is in focus."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | The event stream starts from event 1 and Model::world gives the focus to the first new world, so an event before the first list ends puts a world in focus and a send goes to it. No decision of D8 keeps the focus empty until the list ends. |
| R-65 | 3 | 4 | blocker | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: Each world keeps an input history"],"Each key that sends SHALL add its whole text to the history of the world that it goes to."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  | severity? should-fix 0.76 | No decision says whether a // send enters history as typed or as sent. If the sent /me waves is stored, Up and Enter run it as an unknown command instead of sending it. |
| R-66 | 3 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Key names"],"An uppercase character is refused, because `shift+a` names that key."] | resolved | resolved in round 8 from fixed 2e33cdcadeb66c467e677d4f301822f0a2a09e48 |  |  |  | app-5: D3 says shift+a names the key of an uppercase A, but Termina gives Shift+A as Char('A') with SHIFT, and the client pushes only DISAMBIGUATE, so a chord of Char('a') with SHIFT never matches.; inv-8: A bound name must fire on the key it names. Termina reports Shift with a character key as the shifted character, so a parsed shift+a chord need not match; the D3 property test only checks the parser against itself. |
| R-67 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. Returned sends go to history (#151)"],"The notice of `return_waiting` keeps its count of each world and adds `Up recalls the rest`."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  | scope? deviates-design 0.63 | D6 calls the line of return_waiting a notice, but at head it is a hint. By the Context, a notice shows again after each Reset, so the line of a reset would come back at every later reset. |
| R-68 | 3 | 4 | should-fix | in-change | drift | ["path","crates/mu-tui/src/update.rs",31] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | Msg::Action already takes an Action from a source other than the keymap straight into act. D1 adds the registry path of entry and argument through build for the same job, and no decision removes or reroutes Msg::Action, so a front end can still skip the registry. |
| R-69 | 3 | 8 | should-fix | in-change | drift | ["path","crates/mu-tui/src/update.rs",526] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | move_focus sets the focus and clears the unseen mark. D1 adds FocusIndex and Focus as further focus changes, and D5 ends a walk at each focus change, but no decision names one focus setter, so each arm repeats the set, the mark clear and the walk end. |
| R-70 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-commands/spec.md","Requirement: Quit by command"],"Any other send or command SHALL cancel the arm."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | No planned test passes the cancel side of the quit arm: nothing checks that a send or another command after a by-command /quit clears the arm, so a later /quit can drop waiting texts at once. |
| R-71 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: Up and Down walk the history"],"Past the newest entry, Down SHALL bring back the text that the editor held before the first Up."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | Down on the last line with no walk is the null case of HistoryNext: no draft was saved, and no test checks that the typed text stays; a HistoryNext that restores an empty draft clears it. |
| R-72 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: Each world keeps an input history"],"The client SHALL NOT add an empty text, or a text equal to the newest entry."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  | repeat? R-21 0.80 | The no-empty-entry rule is tested only on the send path. D6 pushes the held text into history, and the held text is empty in the common refusal; no test passes that empty value through put_back. |
| R-73 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: A refused text stops the sends after it"],"It SHALL show the reason as a line with the prefix `-- `, and the count of the texts in the history."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  | repeat? R-4 0.66 | The count in the refusal notice has no test at any value. Its edges are 0 (a lone refused send with an empty editor), 1 and 2, and D6 does not say what the notice holds at 0. |
| R-74 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"4. For `ArgKind::World`, an omitted optional world becomes the world in focus. With no world in focus, the command is bad."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | A key may bind connect or disconnect, whose world is optional. Only the command check of D4 turns an omitted world into the focus or refuses it; the key path builds Connect(None) and nothing handles no world in focus. |
| R-75 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: Each world keeps an input history"],"A command of the client SHALL go into the history of the world in focus."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  | severity? blocker 0.60 | A command enters the history of the world in focus, but D4 fixes no order between act and that push; /focus and /focus-next change the focus in act, so the entry can land in either world. No test passes such a command. |
| R-76 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. The screen opens before the first list"],"It gets the limit `START_LIMIT`, so a server that accepts and does not answer cannot keep the terminal blank."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | D8 bounds the first list of mu-cli with START_LIMIT, but no test passes a server that accepts and does not answer; the tests D8 names never reach that call. |
| R-77 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"The entry `ignore` counts as an editing entry, so `/ignore` is a bad command."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | ignore is the one entry that D4 counts as editing although it edits nothing, so a flag taken from what an entry does would miss it; no test passes /ignore, only /undo. |
| R-78 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Scenario: The first list fails"],"**WHEN** the list fails"] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | The WHEN step of 'The first list fails' names no concrete failure: the list fails with no input, mechanism or state that a test can produce. |
| R-79 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-config/spec.md","Scenario: A file that sets the echo"],"**THEN** the screen shows `> hello` in the default style"] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | The THEN step of 'A file that sets the echo' names 'the default style', which is not defined and names no observable cell style. |
| R-80 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-config/spec.md","Scenario: A file that cannot be read"],"**GIVEN** `<dir>/client.toml` exists, and the user cannot read it"] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | The GIVEN step of 'A file that cannot be read' names the state 'the user cannot read it' with no concrete mechanism, and tasks.md 6.2 notes that as root the file opens, so the state is not produced. |
| R-81 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","Context"],"It refuses each top-level `.toml` except `SERVER_FILE`."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  | severity? should-fix 0.66 | The Context says the resolver refuses each top-level .toml except SERVER_FILE, but global.toml at the top is a layer and is accepted. |
| R-82 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"[`Action` loses `Copy`] Each match on `Action` that copies it now needs a clone. \\u2192 `Action` stays small."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | The mitigation of the risk that Action loses Copy names no task or check. Action stays small is a claim, not a step, and the keymap lookups at head copy the Action out. |
| R-83 | 3 | 4 | suggestion | in-change |  | ["artifact",["specs/terminal-client/spec.md","Requirement: Each world keeps an input history"],"A history SHALL keep at most 1000 entries, and drop the oldest first."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | History has three invariants over any sequence of pushes from sends, commands, focus changes and put_back: at most 1000 entries, no empty entry, no entry equal to the one before it. Only examples test them. |
| R-84 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"An argument of only spaces is no argument."] | resolved | resolved in round 4 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | The rule that an argument of only spaces is no argument holds over any name and any run of spaces; D4 tests it with the single input /connect with one space. |
| R-85 | 4 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. Returned sends go to history (#151)"],"The notice of a refusal in `on_reply` holds the reason. If texts went into history, it also holds their count."] | resolved | resolved in round 8 from fixed 2e33cdcadeb66c467e677d4f301822f0a2a09e48 |  |  |  | D6 calls the refusal line of on_reply a notice, the term that the Context gives to Model::notice, a line that shows again after each Reset. At head that line is a plain push to the pane of the refused world. A build with Model::notice would move it to the pane in focus and bring it back at each Reset. |
| R-86 | 4 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A key bound to connect with no world in focus"] | resolved | resolved in round 8 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | delta scenario 'A key bound to connect with no world in focus' has no matrix row |
| R-87 | 4 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Another command cancels the quit"] | resolved | resolved in round 8 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | delta scenario 'Another command cancels the quit' has no matrix row |
| R-88 | 4 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An escaped slash in the history"] | resolved | resolved in round 8 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | delta scenario 'An escaped slash in the history' has no matrix row |
| R-89 | 4 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Down with no walk"] | resolved | resolved in round 8 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | delta scenario 'Down with no walk' has no matrix row |
| R-90 | 4 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An event before the first list"] | resolved | resolved in round 8 from fixed 798df9e31612ffe38e179887669093ca6e6100c7 |  |  |  | delta scenario 'An event before the first list' has no matrix row |
| R-91 | 5 | 8 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-config/spec.md","Requirement: The keys table binds a key to an entry"],"Each binding SHALL replace the default binding of that key."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  | severity? should-fix 0.67 | D3 parses shift+tab as Tab with SHIFT, but Termina gives Shift+Tab as KeyCode::BackTab in both the legacy and the kitty path, so a [keys] binding of shift+tab never fires. No decision maps it. |
| R-92 | 5 | 8 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D8. The screen opens before the first list"],"While the flag is set, the focus stays empty: `Model::world` adds the world and gives it no focus."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  | severity? should-fix 0.81 | At head the world that takes an empty focus gets the kept notices in its pane. D8 gives no focus during the first list, and set_focus does not copy the notices. Msg::Probe always runs before Msg::Listed, so the width notice stays in the notices pane, which is hidden once a world has the focus. |
| R-93 | 5 | 8 | blocker | in-change | wrong-behaviour | ["path","crates/mu-tui/src/model.rs",237] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  | severity? should-fix 0.77 | D8 gives no focus while the first list runs, and set_focus does not copy the notices, so the first world's pane never shows a notice added before the list ends, such as the width notice of the probe. |
| R-94 | 5 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","Context"],"`is_server_file` also takes that name in another case, and the same file by another name."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | The Context says is_server_file takes the same file by another name. At head it compares only a name equal to server.toml in any case, and a hard link under another name is an error. |
| R-95 | 5 | 8 | should-fix | in-change | drift | ["artifact",["design.md","D2. `mu-cli` reads the file, `mu-tui` owns the type"],"`mu client` writes the warning of `ConfigArgs::worlds` on stderr before the screen opens, with the verb `client`, and goes on with the defaults."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | D2 has mu client write the warning of ConfigArgs::worlds, but that warning is fused into worlds(), which also resolves every world. mu client resolves no world, so it must copy the missing-directory check and its text, or resolve worlds only to get the warning. |
| R-96 | 5 | 8 | should-fix | in-change | drift | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"4. For `ArgKind::World`, an omitted optional world becomes the world in focus. With no world in focus, the command is bad."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  | repeat? R-74 0.58 | D4 resolves an omitted world twice: step 4 of the command check, and act for a key. Two copies of a rule that the front-end requirement keeps the same for each front end can change apart. |
| R-97 | 5 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: The echo line shows each line of a pose"],"A world with no known `line_break` SHALL show the text in one row."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | The echo split has a null case: a world with no known line_break gives one row. Task 7.1 passes only alpha/aria with %r, so a split on a missing or unlisted line_break is untested. |
| R-98 | 5 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: The top row shows the first list"],"no world SHALL have the focus, also if events of a world arrive."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | Only Model::world keeps the focus empty during the first list. focus-1 to focus-9 and /focus call set_focus on worlds that events added, so Alt+1 gives a world the focus before the list ends. |
| R-99 | 5 | 8 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/model.rs",50] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | An event can add a world during the first list, and the list can then fail. D8 gives the focus only 'at the end of the list', and Model::world gives it only to a new world, so that world can stay without focus forever. |
| R-100 | 5 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Key names"],"The special keys are `enter`, `tab`, `backspace`, `esc` and `space`."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | Termina gives Shift+Tab as KeyCode::BackTab, in the legacy and the kitty path. D3 parses shift+tab to Tab with SHIFT, a chord that never arrives, so a binding of shift+tab never fires and nothing refuses it. |
| R-101 | 5 | 8 | should-fix | in-change | false-test | ["artifact",["specs/terminal-client/spec.md","Requirement: Each world keeps an input history"],"A history SHALL keep at most 1000 entries, and drop the oldest first."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | The only test of the cap is history_drops_the_oldest_past_1000. No test checks that 1000 entries keep the oldest, so a cap of 999 passes the test. |
| R-102 | 5 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. History is a state of each world"],"A send ends the walk."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | A send during a walk ends the walk, but D5 does not say where the saved draft goes then. The only draft property test has no send, so a draft typed before Up and then a send of the recalled entry can be lost. |
| R-103 | 5 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: A refused text stops the sends after it"],"If texts went into the history, the line SHALL also hold their count."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | Each waiting text already entered history at its own send, so the push of a returned text is often dropped as equal to the newest entry. The count of the refusal line is undefined then, and no test asserts it. |
| R-104 | 5 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"The text after `/` is split at the first run of spaces into a name and an argument. An argument of only spaces is no argument."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | D4 trims only an argument of only spaces. A given world keeps its trailing spaces, so `/focus beta/bryn ` is an unknown world, and no test passes a world with a trailing space. |
| R-105 | 5 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-commands/spec.md","Requirement: Each front end runs an entry the same way"],"A key and a command SHALL only choose an entry and its argument."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | The requirement 'Each front end runs an entry the same way' has no task line with two of its distinctive words. Task 2.1 names only 'Entry', and task 3.4 names only 'same' from the heading. |
| R-106 | 5 | 8 | suggestion | in-change |  | ["artifact",["design.md","D3. Key names"],"In the kitty path, Termina keeps the unshifted letter, so Shift+Z arrives as `Char('z')` with `SHIFT`."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  | severity? should-fix 0.62 | D3 says that on the kitty path Shift+Z arrives as Char('z') with SHIFT. The client pushes only DISAMBIGUATE_ESCAPE_CODES, so Shift and a letter alone come as text, and Termina gives Char('Z') with SHIFT. Only chords with Ctrl, Alt or Super carry the unshifted letter. The lookup fix covers both cases. |
| R-107 | 5 | 8 | suggestion | in-change |  | ["artifact",["design.md","D3. Key names"],"Task 1.3 checks these reports in Ghostty and Terminal.app."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  | scope? deviates-design 0.69; severity? should-fix 0.82 | D3 says that task 1.3 checks the Shift reports in Ghostty and Terminal.app, but task 1.3 checks only Alt+digit, Alt+Home and Alt+End. |
| R-108 | 5 | 8 | suggestion | in-change |  | ["path","crates/mu-tui/src/update.rs",432] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | The refusal for no world in focus already has two texts of one form, and D4 adds the bare hint `no world` twice more. The texts vary on purpose only in what does not happen. |
| R-109 | 5 | 8 | suggestion | in-change |  | ["artifact",["specs/terminal-client/spec.md","Requirement: A refused text stops the sends after it"],"Up SHALL then give them in that order."] | resolved | resolved in round 8 from fixed a86584969cad0c1b81b0be952476c5155859e78b |  |  |  | The return order holds for any count of returned texts and any held text: successive Ups give them in order. The tests pass only one to three texts, and history_keeps_its_invariants checks no order. |
| R-110 | 6 | 8 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D2. `mu-cli` reads the file, `mu-tui` owns the type"],"`mu-cli` reads `client.toml` as it reads `server.toml`."] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | No decision carries the parsed ClientConfig (its echo and the keymap built from [keys]) from mu-cli into mu_tui::run or the Model. At head run takes only the PortalClient and Model::new always uses Keymap::default, and the model fields are pub(crate), so neither mu client nor the app.rs tests can set the echo or the keymap. |
| R-111 | 6 | 8 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D6. Returned sends go to history (#151)"],"It ends a walk first. The text that the editor held is then the saved draft, not the entry on screen."] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  | severity? should-fix 0.72 | For a refusal during a walk, put_back first ends the walk, and the walk end pushes the draft. The later push of the held text (the same draft) is then dropped as equal to the newest entry, so by D6 it is not counted. The draft did go into the history, but the refusal line omits it from its count, against the requirement. |
| R-112 | 6 | 8 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-commands/spec.md","Requirement: Each front end runs an entry the same way"],"A key can run every entry."] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | The front-end requirement says a key can run every entry, but D2 refuses a binding to an entry whose argument is required, and the client-config spec makes such a binding a bad file. No decision lets a key run focus, so the requirement as written cannot hold. |
| R-113 | 6 | 8 | should-fix | in-change | drift | ["artifact",["proposal.md","What Changes"],"Alt+1 to Alt+9 give the focus to the world at that place in the top row. Alt+Home and Alt+End scroll the pane to its first and last line."] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | The change adds default keys to Keymap::default (Alt+1 to Alt+9, Alt+Home, Alt+End, Up and Down for history), but no task updates the hand copies of the default keys in first-run.md and input-editor.md, so both pages miss the new keys. |
| R-114 | 6 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. History is a state of each world"],"`HistoryPrev` and `HistoryNext` are bound to Up and Down. Up acts only on the first logical line of the pose editor, and Down only on the last."] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | D5 binds plain Up and Down to the history, and Keymap::get gives a modified chord with no entry the plain entry, so Shift+Up, Ctrl+Up and Shift+Down walk the history instead of their editor moves. |
| R-115 | 6 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Key names"],"A key is one character that is not an uppercase letter and not `+`, the name `plus` for the `+` key, or a name of a special key."] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | The key grammar admits any one character that is not uppercase and not +, so a control character such as a tab, CR or DEL parses to a Char chord that Termina never sends; the binding never fires and nothing refuses it. |
| R-116 | 6 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D5. History is a state of each world"],"When a walk ends, the editor keeps its text, and the draft goes into the history of the world of the walk."] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | During a walk, the walk end of put_back pushes the draft, so the held-text push of D6 then drops it as equal to the newest entry; the count of kept entries is 0, the spec asks for a count, and no task asserts it. |
| R-117 | 6 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. History is a state of each world"],"It holds the world of the walk, the place in the history, and the draft that the first Up saved."] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | The walk state holds the world of the walk, but Model::world shifts world indices when an event adds a world that sorts before it; if the walk keeps an index, the draft lands in another world. No test adds a world during a walk. |
| R-118 | 6 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-config/spec.md","Requirement: The echo field sets the echo line"],"The field `echo` SHALL be one of `show`, `dim` and `hide`, and its default SHALL be `dim`."] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | Of the three echo values, only show and hide are parsed from a file; dim is reached only as the default with no file, so a file with echo = "dim" is never read by a test. |
| R-119 | 6 | 8 | should-fix | in-change | false-test | ["path","crates/mu-tui/src/model.rs",138] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | D5 routes every focus change through set_focus, which does nothing during the first list, but Model::new writes the focus directly; D8 has tests build Model::new and then give Msg::Listed, so a model built with worlds has a focus while first_list is set. |
| R-120 | 6 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-commands/spec.md","Scenario: Another command cancels the quit"],"**AND** the screen shows the count of the texts that wait a second time"] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | The THEN step of 'Another command cancels the quit' names the count of the waiting texts without any value, so no test can check the line. |
| R-121 | 6 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-commands/spec.md","Scenario: Quit twice while a text waits"],"**THEN** the screen shows a line that starts with `-- ` and holds the count of the texts that wait"] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | The THEN step of 'Quit twice while a text waits' names a line that holds the count of waiting texts, but states no count value or format. |
| R-122 | 6 | 8 | should-fix | in-change | false-test | ["artifact",["tasks.md","3. Client commands"],"The last test also sends a game text and `/focus-next` between two `/quit` commands, and asserts that the client still runs."] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | Task 3.3 cancels the quit arm with a game text and /focus-next, but the scenario cancels it with a bad command (/fly); that path has no named test. |
| R-123 | 6 | 8 | suggestion | in-change |  | ["artifact",["design.md","D1. The registry is a table of entries, and the keymap binds entries"],"Names that take a number become one entry for each value"] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  | severity? should-fix 0.74 | The nine rows focus-1 to focus-9 of the registry differ only in a number that sets the name, the description and the FocusIndex value, so a row written by hand can carry a name and an index that disagree. |
| R-124 | 6 | 8 | suggestion | in-change |  | ["artifact",["tasks.md","4. Input history"],"The second property test runs random sends, refusals and focus changes, and checks in each history at most 1000 entries"] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  | severity? should-fix 0.62 | The walk place must stay inside the history of the walk world under any mix of Up, Down, sends, refusals and focus changes, but history_keeps_its_invariants draws no Up or Down, so no generated run has a walk open at a push. |
| R-125 | 6 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"approach partial"] | resolved | resolved in round 8 from fixed 3372ec1d63a2d7d5a0643b31a14f8ea20f9ba88d |  |  |  | lens approach reported partial, gaps: 2 |
| R-126 | 7 | 19 | blocker | in-change | drift | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"The loop calls `mu-client` in a spawned task with the same 30 s limit as a send, and sends back `Msg::Control { world, result }`."] | resolved | resolved in round 19 from fixed 1c4ee80f96410210fee04a29f94dab11cb87c89d | 18 |  | severity? suggestion 0.75 | D4 adds the effects Connect and Disconnect as two more arms of the run loop that clone the client, spawn a task, map the mu-client error to a string and wrap the call in within with CALL_LIMIT. At head that block is already in three places (the first list, Send and ListWorlds). D8 removes the first list, so the change leaves four copies, and D4 names no shared helper. |
| R-127 | 7 | 11 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: A refused text stops the sends after it"],"It SHALL put each text that waited behind it, and then the text that the editor held, into the input history of that world. Up SHALL then give them in that order."] | resolved | resolved in round 11 from fixed 0d869e27435893318f6a953d0c35d78e94f50ee1 |  |  |  | After a refusal, Up gives the waiting texts and then the held text only if the refused world has the focus. A refusal of a world out of focus puts the texts in its history, but Up walks the world in focus, and the held text of that world goes into the refused world's history. |
| R-128 | 7 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-commands/spec.md","Requirement: Each front end runs an entry the same way"],"An entry that acts on the text of the editor SHALL NOT run from a command, because the command is that text."] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  |  | The rule that no editing entry runs from a command rests on a hand-set edits flag of each row, but the tests pass only /undo and /ignore. A row such as submit with the flag unset makes /submit clear the editor and send an empty line. |
| R-129 | 7 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: Up and Down walk the history"],"With no older entry, Up SHALL NOT change the text."] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  |  | Up with no older entry must leave the text, but the only test of an empty history presses Up on an empty editor, so an Up that blanks typed text on an empty history passes it. |
| R-130 | 7 | 8 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: The top row shows the first list"],"Keys typed in that time SHALL edit the editor, and a key that sends SHALL show that no world is in focus."] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  | repeat? R-64 0.54 | HistoryPrev and HistoryNext read the history of the world in focus, but no decision or test covers Up or Down with no world in focus, the state of every key before the first list ends and of a server with no world. |
| R-131 | 7 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D3. Key names"],"The others are `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`, `insert`, `delete` and `f1` to `f12`."] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  |  | D3 limits function keys to f1 to f12, but no parser test passes f0 or f13, the values one past each end, so a parser that takes any number after f accepts a name that D3 does not list. |
| R-132 | 7 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-config/spec.md","Requirement: A bad binding stops the client"],"`shift` with a character that is not a letter, a control character, and two names of one key."] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  |  | Shift with a key that is not a letter must be refused, but the named keys space and plus are characters too; no test passes shift+space or shift+plus, so a check only on the one-character branch accepts chords that the terminal sends as plain text. |
| R-133 | 7 | 8 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-config/spec.md","Requirement: The client reads client.toml"],"A missing file, or no directory, SHALL give the defaults and SHALL NOT be an error."] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  |  | No directory at all must give the defaults, but every client.rs test sets MU_CONFIG to an existing empty directory, and no task passes the case where the rule finds no directory. |
| R-134 | 7 | 8 | should-fix | in-change | unmet-requirement | ["path","openspec/changes/client-shell/specs/terminal-client/spec.md",245] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  |  | Scenario 'An event before the first list' has a THEN step that names no observable for the no-send case: 'the client sends no text'. |
| R-135 | 7 | 8 | should-fix | in-change | unmet-requirement | ["path","openspec/changes/client-shell/specs/terminal-client/spec.md",239] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  |  | Scenario 'A send before the first list' has a THEN step that names no observable for the no-send case: 'the client sends no text'. |
| R-136 | 7 | 8 | should-fix | in-change | unmet-requirement | ["path","openspec/changes/client-shell/specs/terminal-client/spec.md",90] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  | severity? suggestion 0.63 | Scenario 'The stream ends while lines wait' has a THEN step that names no observable: 'the client sends no text' after the late reply. |
| R-137 | 7 | 8 | should-fix | in-change | unmet-requirement | ["path","openspec/changes/client-shell/specs/terminal-client/spec.md",93] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  |  | Scenario 'Lines of two worlds wait' has a GIVEN step that names no concrete lines or texts, only that lines wait behind sends. |
| R-138 | 7 | 8 | suggestion | in-change |  | ["artifact",["design.md","D2. `mu-cli` reads the file, `mu-tui` owns the type"],"`mu-cli` puts the path of `client.toml` before that error, so stderr names the file for each bad binding as for a parse error."] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  | severity? should-fix 0.66 | D2 puts the full path of client.toml before a keymap error. A post-parse field error of server.toml names the bare file name and the field path, and parse_toml errors of client.toml will name the bare name too, so one file gets two error formats. D2 does not name the pattern or give a reason. |
| R-139 | 7 | 8 | suggestion | in-change |  | ["artifact",["design.md","Goals / Non-Goals"],"A configuration type that `client-layout` can extend with `[layout]`, `[tabs]` and `[statusline]` without a change to how the file is found or checked."] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  | severity? should-fix 0.78 | D2 does not say which crate turns ClientConfig into Settings. If mu-cli builds Settings field by field from keymap() and echo, then each table that client-layout adds ([layout], [tabs], [statusline]) also changes mu-cli. That goes against the goal that the type extends without a change to how the file is checked. |
| R-140 | 7 | 8 | suggestion | in-change |  | ["artifact",["design.md","D7. Echo"],"For a player event it returns the rows: the first part with `> `, and each other part with two spaces."] | resolved | resolved in round 8 from fixed 902f8e9f3a8f53ab09079bdfaa44fac32ba5f9e0 |  |  |  | The echo split is a round trip over any text and any line_break: the rows with their prefixes removed, joined with line_break, give the cleaned text. Only a single %r example tests it, so a break at the end or two breaks in a row are untested. |
| R-141 | 10 | 11 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: A refused text stops the sends after it"],"If texts went into the history, the line SHALL also hold their count, as `(<n> in history)`."] | resolved | resolved in round 11 from fixed 0d869e27435893318f6a953d0c35d78e94f50ee1 |  |  |  | The fix sends the held text of a refusal out of focus to the world in focus. When only that history keeps a text (nothing waited, or each returned text equalled the newest entry), the count rule falls back to the bare form `(<n> in history)`. That line goes to the pane of the refused world, whose Up gives nothing, so the line counts entries of a world that it does not name. D6 says Up in each world gives exactly the count the line shows for it, and this case breaks that. Before the fix the one history was always the refused world's, so the bare form was right. |
| R-142 | 11 | 13 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Requirement: A refused text stops the sends after it"],"If the refused world does not have the focus, the client SHALL NOT change the editor."] | resolved | resolved in round 13 from fixed 686603313ca3b90cc3713fdfcb4ce3ad68df5bc8 |  |  |  | The fix makes a refusal out of focus leave the editor alone, but the Enter requirement of the same delta still says without a condition that a refused text goes back in the editor. The two SHALL statements conflict for a refusal that arrives after a focus change. A build that follows the Enter requirement puts the refused text in the editor of the world in focus, where a plain Enter sends it to the wrong world, which is the case that the fix removes. |
| R-143 | 12 | 13 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"One helper `spawn_call(tx, call, limit, wrap)` takes the `mu-client` future, the limit and the `Msg` constructor."] | resolved | resolved in round 13 from fixed 882e0254a993bceba3a95f46f10b25695870b5e9 |  |  | severity? should-fix 0.62 | app-1: D4 moves the send onto spawn_call(tx, call, limit, wrap), and that helper has no parameter for the text of a late answer. At head the send gets the suffix that the requirement 'A send with no reply is refused' needs through reply_of. No decision keeps that reason once the send uses the helper. The tests of tests/send.rs call reply_of directly, so they still pass when the reason is lost.; fit-1: D4 adds the helper spawn_call to spawn a mu-client call and send its message back. run.rs already has spawn_reply for that job, and D4 does not name it, so the loop gets two helpers for one mechanism. |
| R-144 | 12 | 13 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. Returned sends go to history (#151)"],"A world out of focus shows its returned text when it gets the focus, and its line sets its unseen mark, as any line does."] | resolved | resolved in round 13 from fixed 882e0254a993bceba3a95f46f10b25695870b5e9 |  |  |  | D6 says the line of a returned send sets the unseen mark of a world out of focus, as any line does. At head that is not so. The refusal line of on_reply goes through Model::push, which sets no mark, and the line of return_waiting is a hint in the pane in focus. A refusal in a world out of focus therefore shows no `*`. The requirement 'Keys move the focus' asks for that mark, and no task builds it. |
| R-145 | 12 | 13 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"The list, the send, `ListWorlds`, `Connect` and `Disconnect` all use it, so the change leaves one copy, not four."] | resolved | resolved in round 13 from fixed 882e0254a993bceba3a95f46f10b25695870b5e9 |  |  |  | D4 moves the send onto spawn_call(tx, call, limit, wrap), which has no late text. The send at head goes through reply_of, whose timeout reason adds that the game can have the line or not, so the send arm loses that reason or cannot use spawn_call. |
| R-146 | 12 | 13 | should-fix | in-change | drift | ["artifact",["design.md","D5. Each world has its own editor and history"],"`Model::input`, `Model::set_input` and the key path of the editor all go through one method `Model::pose_mut`, which gives that editor."] | resolved | resolved in round 13 from fixed 882e0254a993bceba3a95f46f10b25695870b5e9 |  |  |  | D5 routes the update side through a new Model::pose_mut, but the view reaches the pose editor by EditorId in two places, and Model::pose and Model::editor do the same. D5 names none of them, so each keeps its own way to find the pose editor. |
| R-147 | 12 | 13 | should-fix | in-change | drift | ["artifact",["design.md","D5. Each world has its own editor and history"],"and copies the kept notices into its pane, as `Model::world` does at head."] | resolved | resolved in round 13 from fixed 882e0254a993bceba3a95f46f10b25695870b5e9 |  |  |  | D5 has set_focus copy the kept notices into the pane of the world that takes the focus. The Reset arm of on_client already copies the kept notices into the pane in focus with the same loop, and D5 does not name it, so the two copies stay apart. |
| R-148 | 12 | 13 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. Each world has its own editor and history"],"A change of the capabilities of the terminal reaches every world editor and `loose`."] | resolved | resolved in round 13 from fixed 882e0254a993bceba3a95f46f10b25695870b5e9 |  |  |  | Every world editor must use the probed widths. D5 states this only for a change of the caps, but in a run the probe ends before the first list, so each world editor is made after it. No test passes that case. |
| R-149 | 12 | 13 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. Returned sends go to history (#151)"],"It ends a walk of that world first, but it does not push the draft."] | resolved | resolved in round 13 from fixed 882e0254a993bceba3a95f46f10b25695870b5e9 |  |  |  | A walk now stays open in a world out of focus, and put_back must take the draft of the walk of the refused world. No test refuses a text of a world out of focus while that world has an open walk, so a put_back that reads the walk of the world in focus passes. |
| R-150 | 12 | 13 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Key names"],"At a key, the lookup turns an uppercase letter into its lowercase letter with SHIFT before it looks up the chord."] | resolved | resolved in round 13 from fixed 882e0254a993bceba3a95f46f10b25695870b5e9 |  |  |  | The lookup now turns Char('Z') into Char('z') with SHIFT. A key that no binding takes must still reach the editor as it arrived, or each capital letter types lowercase. No test types an unbound Shift and a letter and checks the editor text. |
| R-151 | 12 | 13 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Scenario: A notice before the first list"],"**THEN** the pane of `alpha/aria` shows the notice of the probe"] | resolved | resolved in round 13 from fixed 882e0254a993bceba3a95f46f10b25695870b5e9 |  |  |  | The THEN step of 'A notice before the first list' names the notice only as 'the notice of the probe', with no text, value or cell that a test can compare. |
| R-152 | 12 | 13 | suggestion | in-change |  | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"At head the block that spawns a call, bounds it with `within` and sends back a message is in three places in `run.rs`."] | resolved | resolved in round 13 from fixed 882e0254a993bceba3a95f46f10b25695870b5e9 |  |  | severity? should-fix 0.77 | D4 says that at head the block that spawns a call, bounds it with within and sends back a message is in three places in run.rs. At head the first list is awaited inline before the screen opens. It spawns nothing and sends no message, so the block is in two places. The spawn and the send-back are also already one helper, spawn_reply, which D4 does not name. |
| R-153 | 12 | 13 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"[History in memory only] A crash loses it. \\u2192 History on disk is a later change, beside #156."] | resolved | resolved in round 13 from fixed 882e0254a993bceba3a95f46f10b25695870b5e9 |  |  | severity? should-fix 0.73 | The mitigation of the risk 'History in memory only' is a later change beside #156. No task and no follow-up line of tasks.md names that change, so the mitigation names nothing that the plan does. |
| R-154 | 12 | 14 | suggestion | in-change | false-test | ["artifact",["specs/terminal-client/spec.md","Requirement: Each world keeps its own editor"],"A change of the focus SHALL NOT change the text of an editor, and SHALL NOT end a walk."] | resolved | resolved in round 14 from fixed b5e655971c09b67b2f92eacc5f4fec0a6cd61577 |  |  | severity? should-fix 0.61 | A key edits only the editor of the world in focus, and a focus change changes no editor text. That holds over any run of typing, sends, refusals and focus changes, but only the example a_focus_change_keeps_each_editor tests it. |
| R-155 | 16 | 17 | should-fix | in-change | unmet-requirement | ["artifact",["proposal.md","Choices you will notice"],"Shift with a key that is not a letter, such as `shift+1`, `shift+space` or `shift+plus`, is refused"] | resolved | resolved in round 17 from fixed 2437c591d7683966284b68eb246bbdf2925eaebb | 16 |  |  | The changed choice line refuses Shift with any key that is not a letter. That rule covers shift+tab, but the spec and D3 refuse only Shift with a non-letter character, and the scenario A binding of Shift+Tab binds shift+tab. |
| R-156 | 18 | 19 | should-fix | in-change | drift | ["path","crates/mu-tui/src/update.rs",456] | resolved | resolved in round 19 from fixed 1c4ee80f96410210fee04a29f94dab11cb87c89d | 18 |  |  | The Focus arm of act repeats the unknown-world check and its hint text that world_arg owns, so /focus and a key or the picker refuse an unknown world through two copies. |
| R-157 | 18 | 19 | should-fix | in-change | drift | ["path","crates/mu-tui/src/model.rs",275] | resolved | resolved in round 19 from fixed 1c4ee80f96410210fee04a29f94dab11cb87c89d | 18 |  | scope? beyond-scope 0.60; severity? suggestion 0.61 | Model::pose repeats the entry-or-insert block of Model::pose_of, and Model::world makes the same editor a third way. Model::world always inserts the editor, so the two fallbacks are dead copies of how a world editor is made. |
| R-158 | 18 | 19 | should-fix | in-change | drift | ["path","crates/mu-tui/src/update.rs",929] | resolved | resolved in round 19 from fixed 1c4ee80f96410210fee04a29f94dab11cb87c89d | 18 |  | severity? suggestion 0.70 | The count text `(<n> in history)` that the spec fixes is built twice, in on_reply and in return_waiting, each with its own zero case. |
| R-159 | 18 | 19 | should-fix | in-change | drift | ["path","crates/mu-tui/src/keymap.rs",209] | resolved | resolved in round 19 from fixed 1c4ee80f96410210fee04a29f94dab11cb87c89d | 18 |  |  | The keymap lists the nine names focus-1 to focus-9 by hand and zips them with the digits, while the registry builds the same names from the digit in focus_place!. A reorder of the list binds Alt+1 to another place. |
| R-160 | 18 | 19 | should-fix | in-change | drift | ["path","crates/mu-cli/tests/support/process.rs",819] | resolved | resolved in round 19 from fixed 50eec65501eaa53c8e1331e0cd03c1bf864eb473 | 18 |  | scope? beyond-scope 0.61; severity? suggestion 0.92 | The new test helper dims repeats looks line for line: it finds the first row that starts with the text and maps each of its cells. Only the read of the cell differs. |
| R-161 | 18 | 20 | should-fix | in-change | unmet-requirement | ["path","crates/mu-cli/src/client.rs",58] | resolved | resolved in round 20 from fixed fe05b395a770df74a1cba727c500fb5be5f190e4 | 19 |  | scope? beyond-scope 0.69 | connect_wait checks that the f64 is above 0, but not the Duration. A positive value under 1 ns, such as 1e-10, passes and gives a zero wait. The first list then always times out with a socket error, not the client.toml error. |
| R-162 | 18 | 19 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-cli/src/client.rs",179] | resolved | resolved in round 19 from fixed f980bad417ff81e7ba60466f84cac8b2c73c2e0a | 18 |  |  | start_server now passes --config=<dir> as one argument, so that a path that starts with - stays a value. No test passes such a path, for example a relative MU_CONFIG of -conf. A return to two arguments would pass every test, and the started server would then fail on its own flags. |
| R-163 | 18 | 19 | should-fix | in-change | false-test | ["path","crates/mu-tui/src/update.rs",1007] | resolved | resolved in round 19 from fixed 832260cb4ae6823641ba71532e4eaf16b2f8d3ab | 18 |  |  | The hint of return_waiting adds (<n> in history) only when kept > 0. No test asserts that the hint lacks the count when kept is 0. lines_of_two_worlds_wait has kept 0 but checks a substring, so it also passes with (0 in history) appended. The refusal line tests both sides, and this hint tests only the side above 0. |
| R-164 | 18 | 19 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/update.rs",392] | resolved | resolved in round 19 from fixed 832260cb4ae6823641ba71532e4eaf16b2f8d3ab | 18 |  |  | world_candidates_follow_the_top_row compares candidates() with Model::world_ids(), the one call that candidates() makes, so it cannot fail. |
| R-165 | 18 | 19 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/app.rs",1741] | resolved | resolved in round 19 from fixed 832260cb4ae6823641ba71532e4eaf16b2f8d3ab | 18 |  |  | The no_repeat test cannot tell a history that drops equal texts from one that keeps them: with entries one, one, two the three Up presses give one, one in both cases. |
| R-166 | 18 | 19 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/app.rs",2172] | resolved | resolved in round 19 from fixed 832260cb4ae6823641ba71532e4eaf16b2f8d3ab | 18 |  |  | echo_rows_join_back takes its expected cleaned text from echo_rows(text, None), the function under test, so a wrong clean of the text passes in both calls. |
| R-167 | 18 | 19 | suggestion | in-change |  | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"One function `world_arg(model, Option<String>) -> Result<String, Refusal>` in `update.rs` resolves a world argument."] | resolved | resolved in round 19 from fixed aa67cf5b2104fc3a11b3159cfb8db5a5c6251826 | 18 |  |  | D4 cites world_arg(model, Option<String>) -> Result<String, Refusal>. At head it also takes the entry name and returns Result<String, String>; no type Refusal exists. |
| R-168 | 18 | 19 | suggestion | in-change |  | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"A `Quit` from a command sets the arm with the mark `by_command`."] | resolved | resolved in round 19 from fixed aa67cf5b2104fc3a11b3159cfb8db5a5c6251826 | 18 |  | severity? should-fix 0.63 | D4 names the quit arm of a command the mark by_command. At head the arm is the enum QuitArm with the variants Key and Command; no name by_command exists. |
| R-169 | 18 | 19 | suggestion | in-change |  | ["artifact",["design.md","D1. The registry is a table of entries, and the keymap binds entries"],"Each one sends `Msg::Run { entry: EntryId, arg: Option<String> }`, which replaces `Msg::Action(Action)`."] | resolved | resolved in round 19 from fixed aa67cf5b2104fc3a11b3159cfb8db5a5c6251826 | 18 |  | severity? should-fix 0.72 | D1 says each front end, the keymap included, sends Msg::Run. At head a key and a command call run_entry inside update, and only a test sends Msg::Run; the one path is run_entry, which D1 does not name. |
| R-170 | 18 | 19 | suggestion | in-change |  | ["artifact",["design.md","D7. Echo"],"So the prefix rule stays in one place."] | resolved | resolved in round 19 from fixed aa67cf5b2104fc3a11b3159cfb8db5a5c6251826 | 18 |  | severity? should-fix 0.79 | D7 says echo_rows keeps the prefix rule in one place, but text::line still writes the > prefix for a player event. That arm has no caller now, so the > prefix has two places. |
| R-171 | 18 | 19 | suggestion | in-change |  | ["artifact",["design.md","Open Questions"],"The exact text of each description in the registry. It changes no spec and no task."] | resolved | resolved in round 19 from fixed aa67cf5b2104fc3a11b3159cfb8db5a5c6251826 | 18 |  |  | The one open question is the text of each registry description, but registry.rs at head already gives every entry its description, so the question is answered. |
| R-172 | 18 | 18 | suggestion | in-change |  | ["path","crates/mu-tui/src/update.rs",705] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/167; reason: The usage text of an argument moves into the registry when client-layout adds the palette, which is its first second reader; until then help is the only reader. |  |  | severity? should-fix 0.62 | The text of an entry's argument (`<world>` or `[world]`) is built inside update::help. The palette of client-layout lists the registry and must show the same, so it must copy that match or move it into the registry. |
| R-173 | 18 | 21 | should-fix | in-change | unmet-requirement | ["artifact",["review","not-run"],"The default dims"] | refuted | Both tests of the row ran at head and passed: verification runs list the_default_dims and the_client_with_no_directory with exit 0. The merge matches a run only to a row that names one test, so a row with two tests reads as not-run. |  |  |  | covered row 'The default dims' has no verification run |
| R-174 | 21 | 21 | should-fix | in-change | unmet-requirement | ["path","crates/mu-cli/src/client.rs",123] | fixed | edfe9c40f6398484074b306ef1f33b383d9f6400 | 21 |  |  | A client that finds nothing on the socket never compares config_dir. If two clients with other directories start at once, the second one's server loses the lock and exits, and that client attaches to the first client's server, which it did not start, with no notice. |
| R-175 | 21 | 21 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The server names its directory"] | fixed | 518e8a5b0a7075488e2a37d495fba82867aa98ae | 21 |  |  | delta scenario 'The server names its directory' has no matrix row |
| R-176 | 21 | 21 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A server with another directory"] | fixed | 518e8a5b0a7075488e2a37d495fba82867aa98ae | 21 |  |  | delta scenario 'A server with another directory' has no matrix row |
| R-177 | 21 | 21 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A server with the same directory"] | fixed | 518e8a5b0a7075488e2a37d495fba82867aa98ae | 21 |  |  | delta scenario 'A server with the same directory' has no matrix row |
| R-178 | 22 | 22 | should-fix | in-change | drift | ["path","crates/mu-client/src/lib.rs",95] | fixed | d3b092f5c8cac62289fec11937863afbcafc2407 | 22 |  |  | PortalClient::config_dir repeats the body of PortalClient::list_worlds: the same ListWorlds request and call, keeping the other half of one response. Two methods of one RPC can change apart, and mu client makes the call twice after a start. |
| R-179 | 22 | 22 | should-fix | in-change | drift | ["path","crates/mu-tui/src/update.rs",533] | fixed | c1f7d643e5da4f43b9c41349142dbbbf1d57e118 | 22 |  |  | The word for an argument kind is written in three places: help and arity each match ArgKind to 'world', and ClientConfig::keymap writes 'needs a world' for any required argument without reading its kind. |
| R-180 | 22 | 22 | should-fix | in-change | drift | ["path","crates/mu-tui/src/model.rs",296] | fixed | 25f105707947c1f397f86700d57a70abb61c7520 | 22 |  |  | The change adds five linear scans that find a world by its id (Model::history, Model::walk, the Focus arm, on_reply, put_back), beside the sorted binary search of Model::world. No one lookup without insert exists, so each copy keeps its own match rule. |
| R-181 | 22 | 22 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/update.rs",980] | fixed | 2e9b00459767e43c0aabdf1f696865f43e3123bd | 22 |  |  | put_back pushes the held text first and then each returned text, so with 1000 or more texts behind a refused one the pushes drop the held draft and the last returned line, and the count overstates what Up gives. |
| R-182 | 22 | 22 | should-fix | in-change | false-test | ["path","crates/mu-tui/tests/app.rs",2059] | fixed | 41a65b436b4a8ec538f06ea7f0b8f982bbd813a9 | 22 |  |  | history_keeps_its_invariants never checks the 1000-entry cap that task 4.1 says it checks, and 60 steps cannot reach 1000 entries, so a broken cap passes it. |
| R-183 | 22 | 22 | suggestion | in-change |  | ["artifact",["design.md","D8. The screen opens before the first list"],"On an attach to a server that it did not start, `mu client` resolves its own directory the same way and compares."] | fixed | 41a65b436b4a8ec538f06ea7f0b8f982bbd813a9 | 22 |  | severity? should-fix 0.75 | D8 compares config_dir only on an attach to a server that the client did not start. At head mu client compares after every first list, also after it started a server itself, and design.md records no deviation. |
| R-184 | 22 | 22 | suggestion | in-change |  | ["artifact",["design.md","D8. The screen opens before the first list"],"The list that `mu-cli` makes before `run` decides whether to start a server."] | fixed | 41a65b436b4a8ec538f06ea7f0b8f982bbd813a9 | 22 |  | severity? should-fix 0.71 | D8 says the list before run decides whether to start a server. At head a bare UnixStream connect decides, and the list comes after it; design.md does not name the probe. |
| R-185 | 22 | 22 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"test-coverage partial"] |  |  |  |  |  | lens test-coverage reported partial, gaps: 2 |

## Reviewed files

| path | content |
|---|---|
| Cargo.lock | f0935658614fed31bdee0876a5ec6fc0dc240d8a |
| architecture/model.c4 | de0d8250e2b32b9f8eb4830106a16eec74dbe4c9 |
| architecture/views.c4 | 717d223162471a4c7ff60f04983b37862ecca58d |
| crates/mu-cli/src/client.rs | 59a861b213e8a91a572d0e1b97eeae61f20b97af |
| crates/mu-cli/src/config.rs | c2100c2eefbfae7e6240c2eea48c38a021672aa0 |
| crates/mu-cli/src/config_file.rs | 0cdf1cf8be27ec0ffa7a2e515da1103aba0ff2cd |
| crates/mu-cli/src/lib.rs | b95c38ec64b888cbc94cec8c70ad33855955dd6a |
| crates/mu-cli/src/server.rs | fbd7a47ce318b43296252f9e90cdd19d95d6a06a |
| crates/mu-cli/src/server_config.rs | bbe084932305de93217609b7f9ecfe648df3855f |
| crates/mu-cli/tests/client.rs | 7f78a1ad98f089aaf648016ccc6a8328a80ab3af |
| crates/mu-cli/tests/server.rs | 3097ddf730abbad92668e4d4d3e6c8496d3fd188 |
| crates/mu-cli/tests/support/process.rs | 9506e59932cb92fd0c929999d2508a30bdb151af |
| crates/mu-cli/tests/workspace_boundary.rs | 72318b1b01439268d6978870625c59517efc3a81 |
| crates/mu-cli/tests/world_config.rs | 0365226492201277a55be1507b43c95c808bfd02 |
| crates/mu-client/src/lib.rs | 97bbce2fcc6177888e8cc7959ee33666fd764cb2 |
| crates/mu-core/src/world/dir.rs | 19f66de7d16879d500e0d0fdc50cb273d0f2e5fc |
| crates/mu-core/src/world/mod.rs | 770247846d487e5740e4cc56d790e2d151cf0f0b |
| crates/mu-proto/proto/mu/v1/portal.proto | 5ba3dbf8e79753889893c152c42a6ff5c34ee4d1 |
| crates/mu-server/src/serve.rs | 63fde9d4de6afc73363471d8285ceb8a6aef197f |
| crates/mu-server/src/service.rs | e18a9d3bdc1b0332201a9a5822339fc31da6f44d |
| crates/mu-tui/Cargo.toml | 00eb86b87ed3b96c5147c131a54b9a9662759f77 |
| crates/mu-tui/src/command.rs | 4317699680b210cf929759b1860b141cfc075377 |
| crates/mu-tui/src/config.rs | 2823f3961513f132528a5c55aa74f5b0c2902de4 |
| crates/mu-tui/src/editor.rs | d162d6a62fb062a9a1a652dd0cb32e29492fe29f |
| crates/mu-tui/src/history.rs | a57c6a02208023eabb5e413dbda1fd6c49a8192c |
| crates/mu-tui/src/keymap.rs | 290b2f2365ffe7405a397902e3fa67e7b6219475 |
| crates/mu-tui/src/lib.rs | c77c9da92104480ee3221e056921ac4fb167330f |
| crates/mu-tui/src/model.rs | 68992b685aa45bbd4a0aa11c4547f7b345867f3d |
| crates/mu-tui/src/pane.rs | 7a6eb6c699ad9d7ede4ef79abeaa0d01c282e3d2 |
| crates/mu-tui/src/probe.rs | a2174ee68afdc0b6e6b5c52807e7a12e2924ae9a |
| crates/mu-tui/src/registry.rs | d2260e1a8867da5494d2bc853c30fbd0ea430ceb |
| crates/mu-tui/src/run.rs | 5851ff9587cad0adc2e65fc8cceb87ab737b8c00 |
| crates/mu-tui/src/text.rs | d17176e2d881884e87e8c2eecaacfc05c1832e24 |
| crates/mu-tui/src/update.rs | f16e18a2d2230245242489a93490df1d26fd33bf |
| crates/mu-tui/src/view.rs | c0d1cf99d3cf5797733d8807c6d3ff56219b530b |
| crates/mu-tui/tests/app.proptest-regressions | 943492e6fe7b9a499e12b97ecaf2b78caaa52f6b |
| crates/mu-tui/tests/app.rs | b380ee442da21cd39a866d3a6a268e3347de6dc5 |
| crates/mu-tui/tests/editor.rs | 40eb0bb21d471d734ea4fb747754d649fd44a284 |
| crates/mu-tui/tests/history.rs | 6d4b3e4e70b58bf180445121b507c87bd69784f3 |
| crates/mu-tui/tests/keymap.rs | 0592e99c3fb3af6c25b3a863cf3988be34e6bd3f |
| crates/mu-tui/tests/update.rs | 3e02c066e50e3de261247dc6ea138b884c99be38 |
| docs/src/content/docs/architecture/terminal-client.mdx | 80c1c26e4542676cffb172c3cc560a1a85e041ab |
| docs/src/content/docs/decisions/input-editor.md | 111a5a5738933fa1c69ae125cdb0feec80caee51 |
| docs/src/content/docs/decisions/terminal-client-architecture.md | 5921a051ae562b4c9571105a6368b01ef9578f9c |
| docs/src/content/docs/guides/client.md | 3aa074414267c0c1fdece380ba0473bb9a1c5acc |
| docs/src/content/docs/guides/configuration.md | 31ab6d6de0839a80dd6ca6231868a19e9e976d7a |
| docs/src/content/docs/guides/first-run.md | 14fb49ebf76010d2faf14345b20d2a1c711af4b7 |
| openspec/changes/client-shell/.openspec.yaml | b6809d6be7a6faad01a1bd5da9bea068ac829d8f |
| openspec/changes/client-shell/design.md | 3b3bade2962ecffad16ef3faa75eb74d3e7c765f |
| openspec/changes/client-shell/proposal.md | 8af63d31dc6cf1916c5a3c23825548c35b170618 |
| openspec/changes/client-shell/specs/client-commands/spec.md | e0930b8d51ba76cf8bd3326d22807f042e1b849b |
| openspec/changes/client-shell/specs/client-config/spec.md | 7914965008639b9e2ed35fac6f4f85e857badd5d |
| openspec/changes/client-shell/specs/server-process/spec.md | 2b2e2bc879bc948702ba016c8bf786d4b7a1d70f |
| openspec/changes/client-shell/specs/terminal-client/spec.md | 097fce94a330389b7caf6525769286ba67da51a6 |
| openspec/changes/client-shell/specs/world-config/spec.md | 515b84297760374f211078b26da0c4ce297e435c |
| openspec/changes/client-shell/tasks.md | sha256:6b7aa8b6d0e898f8f3e852f39b7523f06c833be2d7c7efcd968fecbfee9a9b58 |
| openspec/review.json | absent |

## Plan review (2026-11-23)

Verdict approve at head 40216b11e741569700fa4bd7d509bbde86bf5037.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| Help lists the entries | 3.3 | covered |  |
| A key and a command give the same focus | 3.4 | covered |  |
| An editing entry is not a command | 3.1 | covered |  |
| A command does not reach the game | 3.1 | covered |  |
| Several lines are not a command | 3.1 | covered |  |
| An escaped slash | 3.1 | covered |  |
| An unknown command | 3.1 | covered |  |
| A missing world | 3.1 | covered |  |
| An argument to an entry that takes none | 3.1 | covered |  |
| An unknown world | 3.1 | covered |  |
| Connect the world in focus | 3.2 | covered |  |
| Disconnect a named world | 3.2 | covered |  |
| A key bound to connect with no world in focus | 3.2 | covered |  |
| Connect with no world in focus | 3.1 | covered |  |
| A trailing space is no argument | 3.2 | covered |  |
| Disconnect an unknown world | 3.2 | covered |  |
| Focus by name | 3.1 | covered |  |
| A world with a trailing space | 3.4 | covered |  |
| Quit | 3.3 | covered |  |
| Quit twice while a text waits | 3.3 | covered |  |
| Another command cancels the quit | 3.3 | covered |  |
| A command with no world in focus | 3.1 | covered |  |
| A file that sets the echo | 6.2 | covered |  |
| A file that sets dim | 6.2 | covered |  |
| No file | 6.2 | covered |  |
| An unknown key | 6.2 | covered |  |
| A bad echo value | 6.2 | covered |  |
| A file that cannot be read | 6.2 | covered |  |
| Not TOML | 6.2 | covered |  |
| A new binding | 6.4 | covered |  |
| The default keys stay | 6.4 | covered |  |
| A removed binding | 6.4 | covered |  |
| A binding replaces a default | 6.4 | covered |  |
| None on a key with a modifier | 6.4 | covered |  |
| Two names of one key | 6.4 | covered |  |
| A binding of Shift+Tab | 6.4 | covered |  |
| An uppercase key | 6.4 | covered |  |
| A control character | 6.4 | covered |  |
| Shift with a digit | 6.4 | covered |  |
| A binding of Ctrl+Shift+Z | 6.4 | covered |  |
| An unknown entry | 6.4 | covered |  |
| An unknown key name | 6.4 | covered |  |
| An entry that needs an argument | 6.4 | covered |  |
| The default dims | 7.1 | covered |  |
| Hide | 7.1 | covered |  |
| A line goes to the world in focus | 2.2 | covered |  |
| An empty line | 2.2 | covered |  |
| Enter with no world | 2.2 | covered |  |
| A refused line | 2.2 | covered |  |
| A refused line after more text | 5.1 | covered |  |
| Ctrl+M sends | 2.2 | covered |  |
| A pose of two lines | 2.2 | covered |  |
| A line break that the world refuses | 2.2 | covered |  |
| A line that the server refuses | 5.1 | covered |  |
| The last line is refused | 5.1 | covered |  |
| A refused send of Enter with a send after it | 5.1 | covered |  |
| The sends stay apart | 5.1 | covered |  |
| The stream ends while lines wait | 5.1 | covered |  |
| Lines of two worlds wait | 5.1 | covered |  |
| A refusal out of focus | 5.1 | covered |  |
| The server resets while lines wait | 5.1 | covered |  |
| A refusal during a walk of the history | 5.1 | covered |  |
| A refusal after the stream ended | 5.1 | covered |  |
| A send enters the history | 4.1 | covered |  |
| A pose is one entry | 4.1 | covered |  |
| No empty entry | 4.1 | covered |  |
| Each world has its own history | 4.1 | covered |  |
| No repeat | 4.1 | covered |  |
| An escaped slash in the history | 4.1 | covered |  |
| Down with no walk | 4.2 | covered |  |
| Back to the draft | 4.2 | covered |  |
| Up inside a pose moves the cursor | 4.2 | covered |  |
| A send during a walk | 4.3 | covered |  |
| A focus change keeps each editor | 4.2 | covered |  |
| A walk stays with its world | 4.2 | covered |  |
| The second world | 2.3 | covered |  |
| No world at that place | 2.3 | covered |  |
| Back to the first world | 2.3 | covered |  |
| Top and bottom | 2.3 | covered |  |
| The echo of a pose of two lines | 7.1 | covered |  |
| Before the first list | 7.2 | covered |  |
| A send before the first list | 7.2 | covered |  |
| Text typed before the first list | 7.2 | covered |  |
| An event before the first list | 7.2 | covered |  |
| A focus key before the first list | 7.2 | covered |  |
| A failed first list after an event | 7.2 | covered |  |
| A notice before the first list | 7.2 | covered |  |
| The first list fails | 7.2 | covered |  |
| The flag wins over the environment | 6.3 | covered |  |
| The default directory | 6.3 | covered |  |
| The directory under the home directory | 6.3 | covered |  |
| An empty MU_CONFIG | 6.3 | covered |  |
| A relative XDG_CONFIG_HOME | 6.3 | covered |  |
| No home directory | 6.3 | covered |  |
| A missing directory | 6.3 | covered |  |
| The client warns of a missing directory | 6.2 | covered |  |
| A missing default directory | 6.3 | covered |  |
| The client takes the flag | 6.2 | covered |  |
| Worlds from character files | 6.3 | covered |  |
| A character with no game | 6.3 | covered |  |
| A file name outside the id rule | 6.3 | covered |  |
| A stray file | 6.3 | covered |  |
| A link to nothing | 6.3 | covered |  |
| The file of the server | 6.3 | covered |  |
| A character named server | 6.3 | covered |  |
| The file of the client | 6.3 | covered |  |

### Not run

Plan phase: no verification runs.
