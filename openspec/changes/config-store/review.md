---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-24T18:35:07+00:00
head: 57b0a353d2a9fd8d9b2bc80f618d1b0bfb74b9b3
round: 17
next_id: 374
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: ok
  test-coverage: partial
  test-honesty: ok
  invariants: partial
  verify: not-applicable
---
# Review

## Incomplete lenses

- test-coverage: partial, gaps: 2
  - Matrix rows rest on test names matched to scenarios and tasks. The assertion bodies were read only for a_bad_*_in_a_call and the head of a_reload_while_the_stream_is_down, so 'covered' for the other rows is a name match (confidence 0.6).
  - The tests were not run, by rule. The delta spec BREAKING markers were not re-read.
- invariants: partial, gaps: 2
  - The mapping of a neutral key to a termina chord in crates/mu-tui/src/keymap.rs was not read.
  - The behaviour requirements of server-process, terminal-client, client-commands and cli-entrypoint were not read against code; they state no new universal invariant beyond those listed.

The verdict covers only what the lenses checked. While the `ctl-lens` row of an incomplete lens is open, the verdict cannot be `approve`.

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-363 | approach | should-fix | in-change | The requirement 'The client names itself to the server' (kind tui, the OS of the machine) has no decision for mu client. D6 gives the kind and OS defaults to mu config show only. D7 and D8 call GetClientConfig without naming its arguments. The code holds the rule in config::fetch. The other client kinds that the proposal plans (remote clients, an MCP server) have no decision to follow. | specs/client-bootstrap/spec.md > Requirement: The client names itself to the server — "`mu client` SHALL ask the server for the configuration of the kind `tui`, the OS of the machine, and its device name." | Add one sentence to D7 step 4: mu client asks for the kind tui and Os::current through mu_tui::config::fetch, the one builder of the request that the start and the event loop share. |  |  | next-change |
| R-364 | fit | should-fix | in-change | render of mu config show writes the echo line from a literal path. mu world show instead loops over FIELD_PATHS with value_of and a guard test. FIELD_PATHS of mu-config says it gives the order of mu config show, but render never reads it, so mu config show silently leaves out any field that #172 adds to FIELD_PATHS. | crates/mu-cli/src/config_verb.rs:142 — "let _ = writeln!(text, "echo = {echo} # {}", source("echo"));" | Make render loop over mu_config::FIELD_PATHS, with a value_of for the reply, as render in world.rs does. Add a test that each path of FIELD_PATHS has a value in a reply. |  |  | next-change |
| R-365 | fit | should-fix | in-change | The new test file bootstrap.rs adds its own helper refused, which asserts exit status 1 and a stderr that holds each text. This is the job of support::fails_naming, the shared helper that world_config.rs and telemetry.rs call 61 times. | crates/mu-cli/tests/bootstrap.rs:30 — "fn refused(output: &Output, texts: &[&str]) {" | Remove refused from bootstrap.rs, and call support::fails_naming in its five places. |  |  | drift |
| R-366 | invariants | should-fix | in-change | The snapshot keys each path by its lossy UTF-8 text, so two non-UTF-8 file names under clients/ that differ only in their invalid bytes become one path. The second insert replaces the first, so two bad layers give one stderr line and one notice, and the shadowed file is not in the store hash. | crates/mu-core/src/world/snapshot.rs:306 — ".map(\|part\| part.as_os_str().to_string_lossy().into_owned())," | Keep a name that is not UTF-8 apart: escape its invalid bytes (for example as \\xNN) in place of U+FFFD, or keep a second entry when the path is already taken. Add a Linux-only test with two such files under clients/devices that asserts two notices. |  |  | unmet-requirement |
| R-367 | invariants | should-fix | in-change | connect_wait refuses a finite value above about 1.8e19 s, because Duration::try_from_secs_f64 overflows, and the message then says it is not a finite number of 1 ns or more. The spec refuses only 0 or less, not finite, or under 1 ns. No test passes the upper edge. | crates/mu-cli/src/client.rs:39 — ".and_then(\|seconds\| Duration::try_from_secs_f64(seconds).ok())" | Saturate a value above Duration::MAX to Duration::MAX (tokio::time::timeout already handles a far deadline), or state the upper limit in the spec and the message. Add a unit test next to a_wait_of_one_nanosecond_is_a_wait that passes 1e20. |  |  | wrong-behaviour |
| R-368 | invariants | should-fix | in-change | The spec makes shift with a character that is not a letter a bad binding, but the parser checks is_lowercase. shift+U+2170 (small roman numeral one, Nl) and shift+U+24D0 (circled small a, So) are Lowercase and pass, and shift with a caseless letter such as U+05D0 is refused. | crates/mu-config/src/key.rs:167 — "&& !c.is_lowercase()" | Make the rule and the spec say the same: check c.is_alphabetic() && !c.is_uppercase(), or reword the spec to a lowercase character. Add unit tests for shift+U+2170 and shift+U+05D0 beside shift_with_a_digit. |  | scope? beyond-scope 0.74 | unmet-requirement |
| R-373 | test-coverage | should-fix | in-change | lens test-coverage reported partial, gaps: 2 | review > lens-status — "test-coverage partial" | rerun the lens or accept the gap in the review |  |  | unmet-requirement |
| R-369 | approach | suggestion | in-change | D1 allows mu-config only serde, with a row of serde and proptest. At head the row also names toml, a dev-dependency of mu-config, and design.md records no deviation. | design.md > D1. A new crate, `mu-config`, holds the client fields and the key grammar — "The row of `mu-config` in `ALLOWED_CRATES` names `serde` and `proptest`, because that check also reads dev-dependencies." | Record in D1 that toml enters the mu-config row as a dev-dependency for the_defaults_set_each_field_path, or move that test so the row stays serde and proptest. |  |  |  |
| R-370 | approach | suggestion | in-change | D2 says the store writes nothing to stderr itself and gives each notice to the sink. When the snapshot task of a reload fails, the store also logs through tracing::error, so one failure reaches the server log twice in two formats. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "The store writes nothing to stderr itself." | Drop the tracing::error call and let the sink report the notice, or record the exception in D2. |  | severity? should-fix 0.75 |  |
| R-371 | approach | suggestion | in-change | D2 says a failed merge of the good layers makes the client part use the defaults with one notice. At head the merge runs for each client in ClientPart::resolve. Its notice goes only into that GetClientConfig reply. The reload notices and the server stderr never get it, and the reload still exits 0. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "If the merge of the good layers fails, the client part uses the defaults and gives one notice." | Reword D2 to say the merge runs for each client in GetClientConfig and its notice reaches that reply alone, or report it at the reload too. |  | severity? should-fix 0.74 |  |
| R-372 | invariants | suggestion | in-change | The grammar accepts any one character that is not a control character, not uppercase and not +, and lets shift go with any lowercase character, but key_names_read_back draws its keys from ASCII only. The round trip, the no-dot rule and the shift rule over non-ASCII keys (such as é, a titlecase letter or a fullwidth stop) are never generated. | crates/mu-config/tests/keys.rs:189 — ""0123456789!@#$%^&*()-=[]{};:'\\",.<>/?`~\|\\\\_"" | In crates/mu-config/tests/keys.rs, add an arm to key_part that draws any::<char>() filtered by the grammar (not control, not uppercase, not +), with shift_ok set to c.is_lowercase(), using proptest as the existing test does. |  | severity? should-fix 0.60 |  |

## Repeats

| id | lens | severity | claim | evidence |
|---|---|---|---|---|
| R-185 | invariants | should-fix | lens invariants reported partial, gaps: 2 | review > lens-status — "invariants partial" |

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-30 | suggestion | deferred | Suggestions with no behavior change: tie ListWorlds.config_dir to a requirement or drop it, and share one TOML-with-sources renderer between mu world show and mu config show | https://github.com/seanb4t/mu-portal/issues/176 |
| R-31 | suggestion | deferred | Suggestions with no behavior change: tie ListWorlds.config_dir to a requirement or drop it, and share one TOML-with-sources renderer between mu world show and mu config show | https://github.com/seanb4t/mu-portal/issues/176 |
| R-138 | should-fix | accepted | Each scenario has a named test in tasks 4.2 and 4.5; the implementation-phase full review builds the coverage matrix rows for them. |  |
| R-139 | should-fix | accepted | Each scenario has a named test in tasks 4.2 and 4.5; the implementation-phase full review builds the coverage matrix rows for them. |  |
| R-140 | should-fix | accepted | Each scenario has a named test in tasks 4.2 and 4.5; the implementation-phase full review builds the coverage matrix rows for them. |  |
| R-141 | should-fix | accepted | Each scenario has a named test in tasks 4.2 and 4.5; the implementation-phase full review builds the coverage matrix rows for them. |  |
| R-158 | should-fix | accepted | a full round 11 reruns the test-honesty lens over the files it did not read |  |
| R-185 | should-fix | accepted | every one of the 163 covering tests passed at head, test-honesty read every changed file over four agents, and the partial lenses only matched by name or skipped specs outside their focus |  |
| R-366 | should-fix | deferred | the same defect as QUAL-19 in issue #181: key the snapshot by OsString or give a non-UTF-8 name an error entry | https://github.com/seanb4t/mu-portal/issues/182 |
| R-373 | should-fix | accepted | every one of the 163 covering tests passed at head, test-honesty read every changed file over four agents, and the partial lenses only matched by name or skipped specs outside their focus |  |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 00dc0995a2d309a82053d7d5fe23f0472b5b4e84 | 4619aaac294581f63428f0fa799a3ecda7f669c87b0c1f49b8766fbac1c6cd30 | changes-requested | 32 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 5cbf017513e9ae12df877428ff4d90c1dc61b993 | 1388d98570750bcc4098ddf6edab3aefbc494dd56f7ce6a9dc12248d5696f2c1 | changes-requested | 10 | 0 | 5 | 24 | 0 |  |
| 3 | plan | full | b50eb43641ff49cb15e6b753d2890a31dc64dbc1 | 97ab398377b3f8615fb16eead2f9c6064308a922f648cf1e647d26efa5df8a8c | changes-requested | 19 | 0 | 0 | 0 | 0 | new scenarios from rounds 1 and 2 need coverage matrix rows (R-33 to R-42) |
| 4 | plan | follow-up | fa7bcb23a3adf12bfe8cbf119449d5f4810ca2a6 | a89e0842f57fda193535b426370b97a26f5a2e39b65523018c521173c0578fa6 | approve-with-notes | 20 | 0 | 1 | 19 | 0 |  |
| 5 | plan | full | bbed9d1bb7fbdf6b054ac27885070e56e2c5ea08 | 440bc848ad092d9105842c541cc1dfa60e5c196e07f5edf35f0ab32e1874d968 | changes-requested | 18 | 0 | 0 | 0 | 0 | the 20 scenarios added in round 3 need coverage matrix rows (R-62 to R-81) |
| 6 | plan | full | 19f6bc08a670ea415cd7599e0b5e4f768db9f792 | c2a5cb0563c68ce536c7a42379c15769c4db26f80df9a3008ee89f3957e7cc29 | changes-requested | 15 | 0 | 1 | 0 | 0 | the store-hash redesign of D5 changes the API and adds scenarios |
| 7 | plan | full | b1d03d37d65cebcad691607e1ffad2b62f8e94af | 38dc182d18fbed15eef71eb5da157c2f41ea1867f6f90a63655bd4c4c947877b | changes-requested | 11 | 0 | 1 | 0 | 0 | round 6 fixes add scenarios and tests that a follow-up round leaves without matrix rows |
| 8 | plan | full | ea59de7d6e452efb5ef24664a68052747ee3f03b | ddee7644513a76b7f438683a477f71fdf4d4018b78bffb0d981d1169de5f0237 | approve-with-notes | 10 | 0 | 3 | 0 | 0 | round 7 fixes add scenarios and tests that a follow-up round leaves without matrix rows |
| 9 | plan | follow-up | 82a1f77d83c213e83f83784a18d06da5bb914926 | 459eb047fe9c77cc49eb8895a431a470e6d37e8447badbfdd74a97d1534d0e45 | approve-with-notes | 6 | 0 | 1 | 12 | 0 |  |
| 10 | implementation | full | 0e6e8318e8c09c31fee92739a9c0f5b14019de2c | 0d267506f13a5af0063464d5f17d96624418b8bd7ed7dbe64695f8a3083339bb | changes-requested | 17 | 0 | 2 | 0 | 0 |  |
| 11 | implementation | full | e512517e66af92850eb62599532b1016ba448d33 | 8a92d109df8a85d610c256437c1ef39b480aa6cb6a2f7e1606f023d0e4ad7e58 | changes-requested | 27 | 0 | 1 | 0 | 0 | the test-honesty lens of round 10 was partial |
| 12 | implementation | follow-up | e1b97a9dade476dadab073a4331db2fc5dc26b1c | 2622986694bb01f0c8faebd284962f57f365bfcc0479c6670b37c4e8b285df03 | changes-requested | 0 | 0 | 1 | 26 | 0 |  |
| 13 | implementation | follow-up | 58e8c309326c197de4cb81c911c30cc9c45ff953 | d0e2c9ddd4322ac6a9c7e4027bdbd9ddd4cf3f26774f056d8c8721cc5c8adfcf | changes-requested | 0 | 0 | 1 | 0 | 0 |  |
| 14 | implementation | follow-up | 64176f447122666fcc32ba051a1aae6d6903f017 | 3466ffedc464a9eb5976faa131091081c55691bde602b270e5ccbeb9077215ac | approve | 0 | 0 | 0 | 1 | 0 |  |
| 15 | implementation | follow-up | c832ad0a3d3cda13bde36e6e777e79a398646f2e | c0b348b8c79099be88eb99a5a15d5fe4eb6655358d7a2b3f31e87a5882ee851a | approve-with-notes | 6 | 0 | 0 | 0 | 0 |  |
| 16 | implementation | full | d85fc7781896e8bc9e85b411fd6b297f9fa4c909 | d0ae1b63adb43dc2d597e7a6d912cd35d6c539b4088a1ac5967d4f36d9a82e46 | approve-with-notes | 13 | 1 | 0 | 0 | 0 | the suite fixes added five scenarios that need coverage matrix rows (R-345 to R-349) |
| 17 | implementation | full | 57b0a353d2a9fd8d9b2bc80f618d1b0bfb74b9b3 | 09a6e0a64c758b051002cda5f6bb79d46cee105e0cc06a38b1a845aa7f71da46 | approve-with-notes | 11 | 1 | 0 | 0 | 0 | the test-honesty lens of round 16 was partial, so its files are split over three agents |

## Judge

judge: 20 questions, 0.001 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| Help lists the verbs | crates/mu-cli/tests/cli.rs: help_lists_verbs | covered | pass |
| Help for one verb | crates/mu-cli/tests/cli.rs: help_for_one_verb | covered | pass |
| The server flag wins | crates/mu-cli/tests/bootstrap.rs: the_server_flag_wins | covered | pass |
| The server variable | crates/mu-cli/tests/bootstrap.rs: the_server_variable | covered | pass |
| An empty server variable | crates/mu-cli/tests/bootstrap.rs: an_empty_server_variable | covered | pass |
| The default | crates/mu-cli/tests/bootstrap.rs: the_default | covered | pass |
| MU_SOCKET does not count | crates/mu-cli/tests/bootstrap.rs: mu_socket_does_not_count | covered | pass |
| A path with a space | crates/mu-cli/tests/bootstrap.rs: a_path_with_a_space | covered | pass |
| Another scheme | crates/mu-cli/tests/bootstrap.rs: another_scheme | covered | pass |
| A host | crates/mu-cli/tests/bootstrap.rs: a_host | covered | pass |
| A user | crates/mu-cli/tests/bootstrap.rs: a_user | covered | pass |
| A password | crates/mu-cli/tests/bootstrap.rs: a_password | covered | pass |
| A fragment | crates/mu-cli/tests/bootstrap.rs: a_fragment | covered | pass |
| No double slash | crates/mu-cli/tests/bootstrap.rs: no_double_slash | covered | pass |
| A relative path | crates/mu-cli/tests/bootstrap.rs: a_relative_path | covered | pass |
| A wait of zero | crates/mu-cli/tests/bootstrap.rs: a_wait_of_zero | covered | pass |
| A negative wait | crates/mu-cli/tests/bootstrap.rs: a_negative_wait | covered | pass |
| A wait that is not a number | crates/mu-cli/tests/bootstrap.rs: a_wait_that_is_not_a_number | covered | pass |
| An infinite wait | crates/mu-cli/tests/bootstrap.rs: an_infinite_wait | covered | pass |
| A wait under one nanosecond | crates/mu-cli/tests/bootstrap.rs: a_wait_under_one_nanosecond | covered | pass |
| An unknown option | crates/mu-cli/tests/bootstrap.rs: an_unknown_option | covered | pass |
| An option twice | crates/mu-cli/tests/bootstrap.rs: an_option_twice | covered | pass |
| A short wait | crates/mu-cli/tests/bootstrap.rs: a_short_wait | covered | pass |
| The device variable | crates/mu-cli/tests/bootstrap.rs: the_device_variable | covered | pass |
| The device flag wins | crates/mu-cli/tests/bootstrap.rs: the_device_flag_wins | covered | pass |
| An empty device variable | crates/mu-cli/tests/bootstrap.rs: an_empty_device_variable | covered | pass |
| The host name | crates/mu-cli/tests/bootstrap.rs: the_host_name | covered | pass |
| A bad name | crates/mu-cli/tests/bootstrap.rs: a_bad_name | covered | pass |
| A name of 64 characters | crates/mu-cli/tests/bootstrap.rs: a_name_of_64_characters | covered | pass |
| A name of 65 characters | crates/mu-cli/tests/bootstrap.rs: a_name_of_65_characters | covered | pass |
| An empty device flag | crates/mu-cli/tests/bootstrap.rs: an_empty_device_flag | covered | pass |
| The kind layer applies | crates/mu-cli/tests/client.rs: the_kind_layer_applies | covered | pass |
| The device layer applies | crates/mu-cli/tests/client.rs: the_device_layer_applies | covered | pass |
| The OS layer applies | crates/mu-cli/tests/client.rs: the_os_layer_applies | covered | pass |
| The socket flag | crates/mu-cli/tests/client.rs: the_socket_flag | covered | pass |
| Connect the world in focus | crates/mu-cli/tests/client.rs: connect_the_world_in_focus | covered | pass |
| Disconnect a named world | crates/mu-cli/tests/client.rs: disconnect_a_named_world | covered | pass |
| A key bound to connect with no world in focus | crates/mu-cli/tests/client.rs: a_key_bound_to_connect_with_no_world_in_focus | covered | pass |
| Connect with no world in focus | crates/mu-tui/tests/app.rs: connect_with_no_world_in_focus | covered | pass |
| A trailing space is no argument | crates/mu-cli/tests/client.rs: a_trailing_space_is_no_argument | covered | pass |
| Disconnect an unknown world | crates/mu-cli/tests/client.rs: disconnect_an_unknown_world | covered | pass |
| A server table | crates/mu-cli/tests/config_store.rs: a_server_table | covered | pass |
| An unknown field | crates/mu-cli/tests/config_store.rs: an_unknown_field | covered | pass |
| Not TOML | crates/mu-cli/tests/config_store.rs: not_toml | covered | pass |
| A file that cannot be read | crates/mu-cli/tests/config_store.rs: a_file_that_cannot_be_read | covered | pass |
| Two names of one key | crates/mu-cli/tests/config_store.rs: two_names_of_one_key | covered | pass |
| An uppercase key | crates/mu-cli/tests/config_store.rs: an_uppercase_key | covered | pass |
| A control character | crates/mu-cli/tests/config_store.rs: a_control_character | covered | pass |
| Shift with a digit | crates/mu-cli/tests/config_store.rs: shift_with_a_digit | covered | pass |
| An unknown entry | crates/mu-cli/tests/config_store.rs: an_unknown_entry | covered | pass |
| An unknown key name | crates/mu-cli/tests/config_store.rs: an_unknown_key_name | covered | pass |
| An entry that needs an argument | crates/mu-cli/tests/config_store.rs: an_entry_that_needs_an_argument | covered | pass |
| A binding of Shift+Tab | crates/mu-cli/tests/client.rs: a_binding_of_shift_tab | covered | pass |
| A binding of Ctrl+Shift+Z | crates/mu-cli/tests/client.rs: a_binding_of_ctrl_shift_z | covered | pass |
| A new binding | crates/mu-cli/tests/client.rs: a_new_binding | covered | pass |
| The default keys stay | crates/mu-cli/tests/client.rs: the_default_keys_stay | covered | pass |
| A removed binding | crates/mu-cli/tests/client.rs: a_removed_binding | covered | pass |
| A binding replaces a default | crates/mu-cli/tests/client.rs: a_binding_replaces_a_default | covered | pass |
| None on a key with a modifier | crates/mu-cli/tests/client.rs: none_on_a_key_with_a_modifier | covered | pass |
| The default dims | crates/mu-cli/tests/client.rs: the_default_dims | covered | pass |
| Show | crates/mu-cli/tests/client.rs: show | covered | pass |
| Hide | crates/mu-cli/tests/client.rs: hide | covered | pass |
| A bad echo value | crates/mu-cli/tests/config_store.rs: a_bad_echo_value | covered | pass |
| A bad device layer at start | crates/mu-cli/tests/config_store.rs: a_bad_device_layer_at_start | covered | pass |
| A file that is not a layer | crates/mu-cli/tests/config_store.rs: a_file_that_is_not_a_layer | covered | pass |
| A device file with a bad name | crates/mu-cli/tests/config_store.rs: a_device_file_with_a_bad_name | covered | pass |
| A link to nothing under clients | crates/mu-cli/tests/config_store.rs: a_link_to_nothing_under_clients | covered | pass |
| A binding of a dot | crates/mu-cli/tests/config_store.rs: a_binding_of_a_dot | covered | pass |
| An unlistable clients directory | crates/mu-cli/tests/config_store.rs: an_unlistable_clients_directory | covered | pass |
| A control character in a layer name | crates/mu-cli/tests/config_store.rs: a_control_character_in_a_layer_name | covered | pass |
| A line break in a layer name | crates/mu-cli/tests/config_store.rs: a_line_break_in_a_layer_name | covered | pass |
| A line break in a binding | crates/mu-cli/tests/config_store.rs: a_line_break_in_a_binding | covered | pass |
| A link to an unreadable directory under clients | crates/mu-cli/tests/config_store.rs: a_link_to_an_unreadable_directory_under_clients | covered | pass |
| A file that is not TOML | crates/mu-cli/tests/config_store.rs: a_file_that_is_not_toml | covered | pass |
| A loop of links under clients | crates/mu-cli/tests/config_store.rs: a_loop_of_links_under_clients | covered | pass |
| A clients directory in another case | crates/mu-cli/tests/config_store.rs: a_clients_directory_in_another_case | covered | pass |
| A layer for an unknown OS | crates/mu-cli/tests/config_store.rs: a_layer_for_an_unknown_os | covered | pass |
| A FIFO under clients | crates/mu-cli/tests/config_store.rs: a_fifo_under_clients | covered | pass |
| A layer for an unknown kind | crates/mu-cli/tests/config_store.rs: a_layer_for_an_unknown_kind | covered | pass |
| No clients directory | crates/mu-cli/tests/config_store.rs: no_clients_directory | covered | pass |
| A device with no file | crates/mu-cli/tests/config_store.rs: a_device_with_no_file | covered | pass |
| The device wins | crates/mu-cli/tests/config_store.rs: the_device_wins | covered | pass |
| Keys merge key by key | crates/mu-cli/tests/config_store.rs: keys_merge_key_by_key | covered | pass |
| Two names of one key in two layers | crates/mu-cli/tests/config_store.rs: two_names_of_one_key_in_two_layers | covered | pass |
| A layer without echo keeps the earlier echo | crates/mu-cli/tests/config_store.rs: a_layer_without_echo_keeps_the_earlier_echo | covered | pass |
| One bad binding skips its whole layer | crates/mu-cli/tests/config_store.rs: one_bad_binding_skips_its_whole_layer | covered | pass |
| A notice for a bad layer that applies | crates/mu-cli/tests/config_store.rs: a_notice_for_a_bad_layer_that_applies | covered | pass |
| A bad device name in a call | crates/mu-server/tests/config_api.rs: a_bad_device_name_in_a_call | covered | pass |
| A bad kind in a call | crates/mu-server/tests/config_api.rs: a_bad_kind_in_a_call | covered | pass |
| A bad OS in a call | crates/mu-server/tests/config_api.rs: a_bad_os_in_a_call | covered | pass |
| No notice for a bad layer of another device | crates/mu-cli/tests/config_store.rs: no_notice_for_a_bad_layer_of_another_device | covered | pass |
| The same files give the same revision | crates/mu-cli/tests/config_store.rs: the_same_files_give_the_same_revision | covered | pass |
| A changed layer gives a new revision | crates/mu-cli/tests/config_store.rs: a_changed_layer_gives_a_new_revision | covered | pass |
| A layer of another device leaves the revision | crates/mu-cli/tests/config_store.rs: a_layer_of_another_device_leaves_the_revision | covered | pass |
| No server | crates/mu-cli/tests/config_store.rs: no_server | covered | pass |
| Another OS | crates/mu-cli/tests/config_store.rs: another_os | covered | pass |
| An unknown OS flag | crates/mu-cli/tests/config_store.rs: an_unknown_os_flag | covered | pass |
| Show filters a control character in a notice | crates/mu-cli/tests/config_store.rs: show_filters_a_control_character_in_a_notice | covered | pass |
| A reload applies a client layer | crates/mu-cli/tests/config_store.rs: a_reload_applies_a_client_layer | covered | pass |
| A reload with a bad layer | crates/mu-cli/tests/config_store.rs: a_reload_with_a_bad_layer | covered | pass |
| A reload does not apply a world change | crates/mu-cli/tests/config_store.rs: a_reload_does_not_apply_a_world_change | covered | pass |
| A reload with two bad world files | crates/mu-cli/tests/config_store.rs: a_reload_with_two_bad_world_files | covered | pass |
| A reload with a bad game of two worlds | crates/mu-cli/tests/config_store.rs: a_reload_with_a_bad_game_of_two_worlds | covered | pass |
| A reload with two stray files | crates/mu-cli/tests/config_store.rs: a_reload_with_two_stray_files | covered | pass |
| A reload with a bad world file | crates/mu-cli/tests/config_store.rs: a_reload_with_a_bad_world_file | covered | pass |
| A reload with a link to nothing in games | crates/mu-cli/tests/config_store.rs: a_reload_with_a_link_to_nothing_in_games | covered | pass |
| A reload lists a changed server file | crates/mu-cli/tests/config_store.rs: a_reload_lists_a_changed_server_file | covered | pass |
| A second reload lists the world change again | crates/mu-cli/tests/config_store.rs: a_second_reload_lists_the_world_change_again | covered | pass |
| A reload with no server | crates/mu-cli/tests/config_store.rs: a_reload_with_no_server | covered | pass |
| A reload of a removed directory | crates/mu-cli/tests/config_store.rs: a_reload_of_a_removed_directory_keeps_the_layers | covered | pass |
| The server writes the notices of a reload | crates/mu-cli/tests/config_store.rs: the_server_logs_each_notice_of_a_reload | covered | pass |
| An open socket directory | crates/mu-cli/tests/config_store.rs: mu_config_refuses_an_open_socket_directory | covered | pass |
| A control character in the reply | crates/mu-cli/tests/config_store.rs: mu_config_filters_what_it_prints | covered | pass |
| A new binding works without a restart | crates/mu-cli/tests/client.rs: a_new_binding_works_without_a_restart | covered | pass |
| A reload while the stream is down | crates/mu-cli/tests/client.rs: a_reload_while_the_stream_is_down | covered | pass |
| A reload across a server restart | crates/mu-cli/tests/client.rs: a_reload_across_a_server_restart | covered | pass |
| A bad layer after a reload | crates/mu-cli/tests/client.rs: a_bad_layer_after_a_reload | covered | pass |
| A clients directory that turns unlistable | crates/mu-cli/tests/client.rs: a_clients_directory_that_turns_unlistable | covered | pass |
| A failed fetch is tried again | crates/mu-tui/tests/update.rs: a_failed_fetch_fetches_again | covered | pass |
| A bad device layer at attach | crates/mu-cli/tests/client.rs: a_bad_device_layer_at_attach | covered | pass |
| A client with no server starts one | crates/mu-cli/tests/client.rs: a_client_with_no_server_starts_one | covered | pass |
| No server answers in time | crates/mu-cli/tests/client.rs: no_server_answers_in_time | covered | pass |
| Two clients start at the same time | crates/mu-cli/tests/client.rs: two_clients_start_at_the_same_time | covered | pass |
| A server that closes the connection | crates/mu-cli/tests/client.rs: a_server_that_closes_the_connection | covered | pass |
| A server that does not answer | crates/mu-cli/tests/client.rs: a_server_that_does_not_answer | covered | pass |
| The started server finds the default directory | crates/mu-cli/tests/client.rs: the_started_server_finds_the_default_directory | covered | pass |
| The started server inherits MU_CONFIG | crates/mu-cli/tests/client.rs: the_started_server_inherits_mu_config | covered | pass |
| No notice about another directory | crates/mu-cli/tests/client.rs: no_notice_about_another_directory | covered | pass |
| No terminal | crates/mu-cli/tests/client.rs: no_terminal | covered | pass |
| No terminal on stdin | crates/mu-cli/tests/client.rs: no_terminal_on_stdin | covered | pass |
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
| A missing default directory | crates/mu-cli/tests/world_config.rs: missing_default_directory_is_quiet | covered | pass |
| The client warns of a missing directory | crates/mu-cli/tests/client.rs: the_client_warns_of_a_missing_directory | covered | pass |
| The client takes the flag | crates/mu-cli/tests/client.rs: the_client_takes_the_flag | covered | pass |
| Worlds from character files | crates/mu-cli/tests/world_config.rs: worlds_from_character_files | covered | pass |
| A character with no game | crates/mu-cli/tests/world_config.rs: character_with_no_game | covered | pass |
| A file name outside the id rule | crates/mu-cli/tests/world_config.rs: file_name_outside_the_id_rule | covered | pass |
| A stray file | crates/mu-cli/tests/world_config.rs: stray_file | covered | pass |
| A stray character file | crates/mu-cli/tests/world_config.rs: a_stray_character_file | covered | pass |
| A stray game file | crates/mu-cli/tests/world_config.rs: a_stray_game_file | covered | pass |
| A stray file under a character | crates/mu-cli/tests/world_config.rs: a_stray_file_under_a_character | covered | pass |
| A link to nothing | crates/mu-cli/tests/world_config.rs: link_to_nothing_is_an_error | covered | pass |
| The file of the server | crates/mu-cli/tests/world_config.rs: server_file_makes_no_world | covered | pass |
| A character named server | crates/mu-cli/tests/world_config.rs: character_named_server_is_a_world | covered | pass |
| The file of the client | crates/mu-cli/tests/world_config.rs: the_file_of_the_client | covered | pass |
| An unreadable server file leaves world list | crates/mu-cli/tests/world_config.rs: an_unreadable_server_file_leaves_world_list | covered | pass |
| A game named clients | crates/mu-cli/tests/world_config.rs: a_game_named_clients | covered | pass |
| A bad client layer does not stop the worlds | crates/mu-cli/tests/world_config.rs: a_bad_client_layer_does_not_stop_the_worlds | covered | pass |
| A leftover client.toml | crates/mu-cli/tests/world_config.rs: a_leftover_client_toml | covered | pass |

### Not run

None.

## Verification runs

- exit 0: `cargo nextest run -p mu-cli --test cli --locked -E 'test(=help_lists_verbs)'`
- exit 0: `cargo nextest run -p mu-cli --test cli --locked -E 'test(=help_for_one_verb)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=the_server_flag_wins)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=the_server_variable)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=an_empty_server_variable)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=the_default)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=mu_socket_does_not_count)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_path_with_a_space)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=another_scheme)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_host)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_user)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_password)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_fragment)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=no_double_slash)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_relative_path)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_wait_of_zero)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_negative_wait)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_wait_that_is_not_a_number)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=an_infinite_wait)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_wait_under_one_nanosecond)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=an_unknown_option)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=an_option_twice)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_short_wait)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=the_device_variable)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=the_device_flag_wins)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=an_empty_device_variable)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=the_host_name)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_bad_name)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_name_of_64_characters)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=a_name_of_65_characters)'`
- exit 0: `cargo nextest run -p mu-cli --test bootstrap --locked -E 'test(=an_empty_device_flag)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_kind_layer_applies)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_device_layer_applies)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_os_layer_applies)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_socket_flag)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=connect_the_world_in_focus)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=disconnect_a_named_world)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_key_bound_to_connect_with_no_world_in_focus)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=connect_with_no_world_in_focus)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_trailing_space_is_no_argument)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=disconnect_an_unknown_world)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_server_table)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_unknown_field)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=not_toml)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_file_that_cannot_be_read)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=two_names_of_one_key)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_uppercase_key)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_control_character)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=shift_with_a_digit)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_unknown_entry)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_unknown_key_name)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_entry_that_needs_an_argument)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_binding_of_shift_tab)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_binding_of_ctrl_shift_z)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_new_binding)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_default_keys_stay)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_removed_binding)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_binding_replaces_a_default)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=none_on_a_key_with_a_modifier)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_default_dims)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=show)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=hide)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_bad_echo_value)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_bad_device_layer_at_start)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_file_that_is_not_a_layer)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_device_file_with_a_bad_name)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_link_to_nothing_under_clients)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_binding_of_a_dot)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_unlistable_clients_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_control_character_in_a_layer_name)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_line_break_in_a_layer_name)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_line_break_in_a_binding)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_link_to_an_unreadable_directory_under_clients)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_file_that_is_not_toml)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_loop_of_links_under_clients)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_clients_directory_in_another_case)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_layer_for_an_unknown_os)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_fifo_under_clients)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_layer_for_an_unknown_kind)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=no_clients_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_device_with_no_file)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=the_device_wins)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=keys_merge_key_by_key)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=two_names_of_one_key_in_two_layers)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_layer_without_echo_keeps_the_earlier_echo)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=one_bad_binding_skips_its_whole_layer)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_notice_for_a_bad_layer_that_applies)'`
- exit 0: `cargo nextest run -p mu-server --test config_api --locked -E 'test(=a_bad_device_name_in_a_call)'`
- exit 0: `cargo nextest run -p mu-server --test config_api --locked -E 'test(=a_bad_kind_in_a_call)'`
- exit 0: `cargo nextest run -p mu-server --test config_api --locked -E 'test(=a_bad_os_in_a_call)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=no_notice_for_a_bad_layer_of_another_device)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=the_same_files_give_the_same_revision)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_changed_layer_gives_a_new_revision)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_layer_of_another_device_leaves_the_revision)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=no_server)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=another_os)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=an_unknown_os_flag)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=show_filters_a_control_character_in_a_notice)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_reload_applies_a_client_layer)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_reload_with_a_bad_layer)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_reload_does_not_apply_a_world_change)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_reload_with_two_bad_world_files)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_reload_with_a_bad_game_of_two_worlds)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_reload_with_two_stray_files)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_reload_with_a_bad_world_file)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_reload_with_a_link_to_nothing_in_games)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_reload_lists_a_changed_server_file)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_second_reload_lists_the_world_change_again)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_reload_with_no_server)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=a_reload_of_a_removed_directory_keeps_the_layers)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=the_server_logs_each_notice_of_a_reload)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=mu_config_refuses_an_open_socket_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test config_store --locked -E 'test(=mu_config_filters_what_it_prints)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_new_binding_works_without_a_restart)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_reload_while_the_stream_is_down)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_reload_across_a_server_restart)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_bad_layer_after_a_reload)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_clients_directory_that_turns_unlistable)'`
- exit 0: `cargo nextest run -p mu-tui --test update --locked -E 'test(=a_failed_fetch_fetches_again)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_bad_device_layer_at_attach)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_client_with_no_server_starts_one)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=no_server_answers_in_time)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=two_clients_start_at_the_same_time)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_server_that_closes_the_connection)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=a_server_that_does_not_answer)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_started_server_finds_the_default_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_started_server_inherits_mu_config)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=no_notice_about_another_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=no_terminal)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=no_terminal_on_stdin)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_send_before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=text_typed_before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=an_event_before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_focus_key_before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_failed_first_list_after_an_event)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=a_notice_before_the_first_list)'`
- exit 0: `cargo nextest run -p mu-tui --test app --locked -E 'test(=the_first_list_fails)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=flag_wins_over_the_environment)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=default_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=directory_under_home)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=empty_mu_config)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=relative_xdg_config_home)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=no_home_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=missing_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=missing_default_directory_is_quiet)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_client_warns_of_a_missing_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test client --locked -E 'test(=the_client_takes_the_flag)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=worlds_from_character_files)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=character_with_no_game)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=file_name_outside_the_id_rule)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=stray_file)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_stray_character_file)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_stray_game_file)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_stray_file_under_a_character)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=link_to_nothing_is_an_error)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=server_file_makes_no_world)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=character_named_server_is_a_world)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=the_file_of_the_client)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=an_unreadable_server_file_leaves_world_list)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_game_named_clients)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_bad_client_layer_does_not_stop_the_worlds)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=a_leftover_client_toml)'`

## Dropped

None.

## Lens notes

- approach: Checked at head: D1 crate edges and entry-table test, D2 snapshot walk, link limit, sink, server_config::check, merge helper and reload, D3 skip and the client.toml branch, D4 framing and Sha256::digest, D5 proto, watch stream and name checks, D6 URL parser, device rule, checked_dir and answered, D7 start order, D8 apply and retry, D9 LikeC4 views and pages.
- approach: Third-party facts confirmed in the local registry sources: walkdir 2.5.0 check_loop gives a path-less error through Error::from_io, and figment2 0.11.5 coalesce merges two dicts recursively.
- approach: Every test named in the Enforcement table exists at head.
- approach: app-1, app-2 and app-4 are suggestions, below the table severity. None has one of the five consequences, because the code holds each rule and only the document differs from it.
- fit: The prompt names no path for the repository rows of the library-preference table, so only the generic rows of the rulebook applied.
- fit: Checked against existing patterns, with no new finding: the shared Reopen of the two streams in mu-client; the merge_layers helper, which world() and ClientPart::resolve both call; the one ControlMap filter with its REPORT, STORE and SCREEN maps; server_config::check, which the start and the reload both call; the answered and timed helpers; and connect_wait and shown, which parse_url reuses.
- fit: Two small copies are not reported, because their fallbacks cannot run (mu_client::backoff has no end): the literal 5 s fallback in config_failed of update.rs, and DEFAULT_WAIT as the fallback in answers of client.rs. LONGEST_WAIT of mu-client is the source of both.
- fit: The test helpers start in config_store.rs and serve in bootstrap.rs repeat the same ServerBuilder chain, which worlds.rs and client.rs also write inline. They are not reported, because the chain is three builder calls.
- invariants: I1: A device name is always 1 to 64 characters from a-z, 0-9, -, _ and . \| enforced at crates/mu-config/src/names.rs DeviceName::new, used by mu-cli and GetClientConfig (service.rs:143) \| boundary test a_device_name_of_64, a_device_name_of_65, an_empty_device_name
- invariants: I2: A kind is always tui and an OS always macos or linux \| enforced at Kind::parse, Os::parse, and layer_rule for layer files \| boundary test an_unknown_kind, an_unknown_os, a_bad_kind_in_a_call, a_bad_os_in_a_call
- invariants: I3: No canonical key name holds a dot \| enforced at KeyName Display with the NAMED row period \| boundary test a_dot_key_has_the_canonical_name_period and property key_names_read_back (ASCII keys only, see inv-3)
- invariants: I4: Two names of one key are one key, in one layer and across layers \| enforced at check_keys and the canonical names in store.rs state() \| boundary test two_names_of_one_key_in_one_layer, two_names_across_layers_are_one_key, the shuffle of key_names_read_back
- invariants: I5: shift goes only with a letter \| enforced at key.rs:167 as is_lowercase, not as the spec's letter rule \| boundary test shift_with_a_digit (see inv-4)
- invariants: I6: Two different entry lists never give the same framed input \| enforced at store.rs framed() \| boundary test a_layer_that_holds_the_next_entry_changes_the_revision and property two_layer_lists_give_two_inputs
- invariants: I7: The revision changes exactly when a layer or a layer directory entry of the client changes \| enforced at ClientPart::resolve over layer_dirs and the client files \| boundary test the_revision_ignores_other_devices, an_unreadable_layer_changes_the_revision, an_unlistable_clients_directory_changes_the_revision
- invariants: I8: A notice and a restart path never hold a control character \| enforced at mu_config::filter with STORE in Notice::whole and changed() \| boundary test the_store_map_replaces_a_line_break and property the_filter_leaves_no_control_character. U+2028 and Cf characters are not Cc and pass, which the spec's control-character wording allows
- invariants: I9: The snapshot holds one entry per file \| enforced at none for non-UTF-8 names (lossy path key) \| boundary test none (see inv-1)
- invariants: I10: The walk enters at most LINKED_DIRS_MAX directories through links, and a link to an ancestor is an error \| enforced at snapshot.rs link() with == LINKED_DIRS_MAX and the chain of ancestors \| boundary test the_walk_enters_exactly_the_limit_of_linked_directories (64 and 65), a_loop_of_links_ends
- invariants: I11: A bad client layer never stops the start \| enforced at DirSnapshot::take, which fails only at the root, and ClientPart::new, which cannot fail \| boundary test the config_store.rs scenario tests of the start
- invariants: I12: connect_wait is a wait of 1 ns or more and finite \| enforced at client.rs connect_wait \| boundary test a_wait_of_one_nanosecond_is_a_wait, a_wait_under_one_nanosecond, a_wait_of_zero, a_negative_wait, an_infinite_wait. The upper edge has none (see inv-2)
- invariants: I13: A reply applies only if its store hash is the latest hash \| enforced at update.rs Msg::Config \| boundary test a_stale_reply_is_dropped and property the_latest_hash_wins
- invariants: I14: A reload with no change sends no hash \| enforced at store.rs send_if_modified on store_hash \| boundary test a_reload_with_no_change_sends_nothing
- invariants: I15: A reload of a missing directory keeps the client layers \| enforced at Store::reload try_exists \| boundary test a_reload_of_a_removed_directory_keeps_the_layers
- invariants: I16: The reload reports a world error whenever the start would stop on it \| enforced at dir.rs world_errors, which resolves each character alone \| boundary test a_reload_reports_a_bad_world_file and the two-file tests. resolve has no cross-character rule at head, so the equivalence holds by reading. No property test compares resolve and world_errors
- invariants: I17: The merge of checked layers never fails \| enforced at none (review alone, as the Enforcement table says) \| boundary test none. The fallback to the defaults has a notice
- invariants: I18: A failed fetch while stale shows one notice and waits by backoff, reset by an apply or a hash back to the applied one \| enforced at update.rs config_failed and the StoreHash arm \| boundary test each_failed_fetch_waits_longer_until_an_apply, a_hash_back_to_the_applied_one_resets_the_failures
- invariants: I19: The watch never opens again without a wait after a loss \| enforced at mu-client Reopen::pause \| boundary test the_watch_waits_before_each_open_after_a_loss
- invariants: The task 8.3 rg check was run at head: it lists the migration section of guides/client.md and one line of guides/configuration.md, as the task allows.
- invariants: No enforcement point is a script, so no probe input was written. inv-2 and inv-4 rest on documented std behaviour: Duration::try_from_secs_f64 fails above u64::MAX seconds, and char::is_lowercase uses the Unicode Lowercase property, which includes Other_Lowercase.
- test-coverage: All 163 scenario headings have a named test in a file that exists at head. No scenario lacks a test name.
- test-coverage: Tasks 9.3 stays unchecked in tasks.md; its fix tests (a_reload_of_a_removed_directory_keeps_the_layers, the_server_logs_each_notice_of_a_reload, mu_config_refuses_an_open_socket_directory, mu_config_filters_what_it_prints, a_started_server_that_does_not_answer_names_the_last_error) exist in the test files.
- test-honesty: Read every added test file: mu-server tests store.rs and config_api.rs, mu-core tests snapshot.rs, mu-config tests keys.rs, names.rs, filter.rs, layer.rs. No disabled test, no test without an assertion, no mock echo.
- test-honesty: Weak spots judged not to be findings: a_reload_keeps_the_worlds is named for the worlds but asserts the restart lines, which is the spec THEN; a_fifo_named_server_toml_in_another_case_is_not_opened asserts only is_ok, and the failure it guards is a hang; a_reload_with_no_change_sends_nothing uses now_or_never, and a_reload_sends_a_new_hash proves the stream can emit.
- test-honesty: The inline test modules of the changed src files (service.rs from line 411, world/mod.rs) lie outside the changed ranges and were not judged.
- test-honesty: Read in full: tests/config_store.rs, tests/bootstrap.rs, the new tests of tests/client.rs (lines 944-1521), tests/world_config.rs 240-399, the two telemetry hunks and one server.rs hunk. No ignored or skipped test, no assert!(true), no \|\| true, no test without an assertion.
- test-honesty: Every test drives the built binary or a real server process, so no mock-echo or tautology found. Expected values are literals from the spec, not output of the code under test.
- test-honesty: tests/world_config.rs unreadable_top_file_makes_no_world sets mode 000 but, unlike lock() in config_store.rs, never checks that the mode keeps the user out. Run as root it passes with the read succeeding. A_fifo and a_broken_link siblings cover the same scenario, so it is not reported as a finding.
- test-honesty: Read the changed tests in mu-tui/tests/update.rs (live config), tests/keymap.rs (key names, entry table parity, chord proptest), tests/app.rs (echo tests) and the mu-client unit tests. Each has an assertion that can fail for its named reason. No disabled, ignored or assertion-free test found.
- test-honesty: Tests in the diff are the unit tests in bootstrap.rs, client.rs, server_config.rs and socket.rs. The tests/support helpers and workspace_boundary.rs are not test cases of their own and were not read line by line.
- test-honesty: No disabled test, no test without an assertion, and no mock-echo found.
- test-honesty: one_wait_covers_the_three_steps checks both the on-time and the late case with a paused clock, so it can fail for the reason it names.
- test-honesty: combined from four agents over split file lists: th-a mu-cli tests, th-b mu-core mu-server mu-config, th-c mu-tui mu-client, th-d mu-cli support and src; th-d closes the gap of th-a

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | fixed_after | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D5. The API"],"SubscribeResponse.config_changed {} (not in the journal, no sequence number)"] | fixed | 30253ffee5f0416968dc01cb22997ee738fa1972 | 2 |  | severity? should-fix 0.66 | A config_changed sent while a client's stream is lost is never seen: the stream resumes silently, and no decision has the client fetch GetClientConfig again on Resumed or Reset. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu config reload reads the directory again"],"The command SHALL print one line for each bad file to stderr."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The reload must report a bad server.toml, but D2 keeps the server.toml parser in mu-cli and the store holds only its bytes, and mu-server cannot depend on mu-cli. No decision checks server.toml at reload. |
| R-3 | 1 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: GetClientConfig resolves one client"],"A bad name SHALL be an error that names it."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  | severity? should-fix 0.76 | app-3: The server must check device names (layer file names, and the device of GetClientConfig), but the design puts the device-name rule only in mu-cli. D1 gives mu-config the key grammar and entry names, not the device, kind or OS names.; inv-6: The server-side check of the kind, the OS and the device of GetClientConfig has no task and no test. Only the CLI checks the device name, and remote callers reach the API directly. |
| R-4 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The connect_wait option"],"The query option `connect_wait` SHALL be the longest time, in seconds, that a client waits for a server to answer."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  | severity? should-fix 0.84 | The connect_wait requirement and the scenario A short wait apply to mu config show, but only D7 (mu client) applies connect_wait. D6 gives mu config the flags and nothing about the wait. |
| R-5 | 1 | 2 | blocker | in-change | unmet-requirement | ["path","crates/mu-cli/tests/workspace_boundary.rs",70] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  | scope? beyond-scope 0.85; severity? should-fix 0.69 | The rule of workspace_boundary.rs refuses sha2 in mu-server: its ALLOWED_CRATES row has no sha2, and no task adds it or a workspace dependency entry. |
| R-6 | 1 | 2 | blocker | in-change | unmet-requirement | ["path","crates/mu-cli/tests/workspace_boundary.rs",150] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  | severity? should-fix 0.73 | The rule of workspace_boundary.rs refuses url in mu-cli: its ALLOWED_CRATES row has no url, and no task adds it or a workspace dependency entry. |
| R-7 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"From the snapshot it resolves the worlds with `mu_core::world::resolve`, as `mu-cli` does today."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | D2 resolves the worlds from the in-memory snapshot with mu_core::world::resolve, as mu-cli does. At head, resolve reads through a LayerSource, and the only source, TomlDirSource, reads the disk itself. So the worlds and the snapshot come from two reads, and a file edited between them is compared against bytes that never ran. |
| R-8 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. The world scan skips `clients/` and rejects `client.toml`"],"This uses the existing stray-file rule, so no new error path is needed."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | D3 says the existing stray-file rule yields an error that points to clients/global.toml with no new path. At head the stray-file error is the fixed list of layer files, so the pointer the Migration Plan promises needs a new branch. |
| R-9 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","Migration Plan"],"The user moves the content of `client.toml` to `clients/global.toml` and drops the table `[server]`."] | refuted | mu-portal has no release (engram 3j32zhc4bj): breaking changes to configuration are fine and no downgrade path is owed |  |  | severity? blocker 0.75 | The Migration Plan moves client.toml to clients/global.toml with no rollback step. The binary at head treats any file under clients/ as a stray file, so after the move a downgrade stops mu server and mu world. |
| R-10 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["tasks.md","4. The store"],"It merges the layers in order, field by field and key by key."] | fixed | a1f53d1d347f0578fd8a83b0761cf7effbb37076 | 2 |  | scope? deviates-design 0.62 | The store hand-writes a layer merge with a source per field, which figment2 already does for the world layers; the design does not name figment2 or say why it does not fit, and #172 adds more fields to that merge. |
| R-11 | 1 | 2 | should-fix | in-change | drift | ["artifact",["tasks.md","2. The crate mu-config"],"The error names the binding or the field. If TOML gives a line, the error names it too."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | mu-config gets its own TOML parse that names the line of an error, a copy of mu_core::world::parse_toml and its line-and-column helper, which also keeps the rule never to quote the line. |
| R-12 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"At the start, the store reads every `.toml` file under the configuration directory into a snapshot: each relative path with its bytes."] | fixed | 30253ffee5f0416968dc01cb22997ee738fa1972 | 2 |  |  | The store adds a second reader of the configuration directory (its own walk and sort into worlds, server.toml and clients/) beside TomlDirSource, whose walk and top-file rule read the same tree; resolve takes only a LayerSource, and the design does not say how the snapshot reaches it. |
| R-13 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","D1. A new crate, `mu-config`, holds the client fields and the key grammar"],"It also holds the table of entry names, with the entries that need an argument."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  | severity? blocker 0.81 | fit-4: D1 keeps two tables of the entries, one in mu-config with the entries that need an argument and the registry in mu-tui with its own required flag; the guard test compares names only, so the argument flag can drift.; inv-5: The parity test compares entry names only. If the needs-an-argument flag differs between mu-config and the registry, the server accepts a binding to an entry that refuses to run without its argument. |
| R-14 | 1 | 2 | should-fix | in-change | drift | ["path","crates/mu-tui/src/config.rs",102] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The binding check of ClientConfig::keymap in mu-tui (unknown entry, entry that needs an argument, two names of one key) stays, and task 2.2 adds the same check in mu-config; no task removes the mu-tui copy or ClientConfig. |
| R-15 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7. The start of `mu client`"],"Make the first list and the call `GetClientConfig`, both within one wait of `connect_wait`."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | A reload between the GetClientConfig call at the start and the first subscribe of the stream is never seen by that client: config_changed goes only to open streams, is not journaled, and the stream opens after the screen and the terminal probe. |
| R-16 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-bootstrap/spec.md","Requirement: A URL names a Unix socket"],"A server URL SHALL be `unix://`, then an absolute path, then an optional query."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | None of the listed refusals catches unix:/tmp/s.sock, which has no authority. The spec requires unix:// before the path, and task 1.1 does not parse this form. |
| R-17 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. The world scan skips `clients/` and rejects `client.toml`"],"drops `client.toml` from the top-level files and skips the directory `clients`."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | D3 skips the directory clients without saying top level only. The scan recurses, and no test covers a game named clients, so characters/clients/<c>.toml could vanish with no error. |
| R-18 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-client/src/lib.rs",46] | fixed | 30253ffee5f0416968dc01cb22997ee738fa1972 | 2 |  |  | A config_changed sent while a client's stream is lost and waiting to resubscribe, or a change of client layers across a server restart, is never replayed, so the client keeps an old revision until the next reload. |
| R-19 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D4. The revision is a hash of the layers that apply"],"Its input is the sorted list of the layer files that exist for the kind, the OS and the device of the client, each path with its bytes."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | D4 hashes each path with its bytes but names no framing, so removing one layer and appending its path and bytes to the previous layer gives the same input and the same revision. |
| R-20 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D4. The revision is a hash of the layers that apply"],"A bad layer counts too, so a layer that turns bad changes the revision."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | D4 does not say what the hash takes for a layer that cannot be read. If it takes no bytes, an empty layer that turns unreadable keeps its revision, and the client then shows no notice for it. |
| R-21 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/config-store/spec.md","Requirement: The layer files of a client"],"A device SHALL be a valid device name."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | No task checks the name of a devices/<name>.toml file against the device grammar, and no test covers one. On a case-insensitive file system, devices/Laptop.toml then applies to no device and gives no notice. |
| R-22 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The device name"],"A device name SHALL have 1 to 64 characters from `a` to `z`, `0` to `9`, `-`, `_` and `.`."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The device name length rule of 1 to 64 has no test at 64, at 65 or for an empty --device. The only bad-name test is a name with a space. |
| R-23 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The connect_wait option"],"So SHALL a query option with any other name, and an option that is there twice."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The refusal of a query option that appears twice has no test. Task 1.1 only parses it in a scratch test. |
| R-24 | 1 | 2 | should-fix | in-change | next-change | ["artifact",["proposal.md","What Changes"],"**BREAKING** `client.toml` is removed. `mu client` no longer takes `--config` and reads no configuration file."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The proposal marks three changes BREAKING, but no delta spec carries a BREAKING marker. |
| R-25 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Requirement: The configuration directory"],"The flag `--config <dir>` of `mu server` and `mu world` SHALL name the configuration directory"] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The requirement 'The configuration directory' has no task line that names its subject; only the word 'directory' matches. Its new scenarios are named in tasks 3.1 and 6.1 only. |
| R-26 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","6.2"],"`a_key_bound_to_connect_with_no_world_in_focus`"] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | Five scenarios of the modified requirement 'Connect and disconnect a world' are named by no task; only the full-suite run in 9.2 reaches them. |
| R-27 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","6.3"],"Make sure that the tests of the requirement \\"The top row shows the first list\\" in `crates/mu-tui/tests/app.rs` still pass"] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The eight terminal-client scenarios are covered only by 'the tests of the requirement still pass'; no test is named per scenario. |
| R-28 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","3.1"],"and that every other test in `world_config.rs` passes"] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | Seven world-config scenarios (flag wins, default dir, XDG, empty MU_CONFIG, no HOME, missing dir, character files and others) are covered only by 'every other test in world_config.rs passes'. |
| R-29 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["proposal.md","Bootstrap"],"`mu client` no longer takes `--config` and reads no configuration file."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The BREAKING removal of --socket and --config from mu client has no scenario; only MU_SOCKET and the client takes the flag are covered. |
| R-30 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D5. The API"],"`ListWorlds.config_dir` stays. `mu config show` does not need it, but a remote client or an MCP server can use it to name the directory."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/176; reason: Suggestions with no behavior change: tie ListWorlds.config_dir to a requirement or drop it, and share one TOML-with-sources renderer between mu world show and mu config show |  |  |  | D5 keeps ListWorlds.config_dir for a remote client or MCP server, but no requirement needs it and the proposal puts the remote client out of scope. |
| R-31 | 1 | 1 | suggestion | in-change |  | ["artifact",["tasks.md","5. The API and the bootstrap values"],"`show` prints the TOML with a source comment on each field and binding, and the revision on its first line."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/176; reason: Suggestions with no behavior change: tie ListWorlds.config_dir to a requirement or drop it, and share one TOML-with-sources renderer between mu world show and mu config show |  |  |  | mu config show prints TOML with a source comment on each line, the same form that render in world.rs writes for mu world show; the two vary only in their rows. |
| R-32 | 1 | 2 | suggestion | in-change | next-change | ["path","crates/mu-tui/tests/keymap.rs",260] | fixed | 30253ffee5f0416968dc01cb22997ee738fa1972 | 2 |  | severity? should-fix 0.64 | The rule that two names of one key are one key is order independence over all modifier sets, and the merge relies on it. D1 allows mu-config only serde and toml, so the existing proptest cannot move with the grammar. |
| R-33 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"No double slash"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 2 |  |  | delta scenario 'No double slash' has no matrix row |
| R-34 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An option twice"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 2 |  |  | delta scenario 'An option twice' has no matrix row |
| R-35 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A name of 64 characters"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 2 |  |  | delta scenario 'A name of 64 characters' has no matrix row |
| R-36 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A name of 65 characters"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 2 |  |  | delta scenario 'A name of 65 characters' has no matrix row |
| R-37 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An empty device flag"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 2 |  |  | delta scenario 'An empty device flag' has no matrix row |
| R-38 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The socket flag"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 2 |  |  | delta scenario 'The socket flag' has no matrix row |
| R-39 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A device file with a bad name"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 2 |  |  | delta scenario 'A device file with a bad name' has no matrix row |
| R-40 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A bad device name in a call"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 2 |  |  | delta scenario 'A bad device name in a call' has no matrix row |
| R-41 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A reload while the stream is down"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 2 |  |  | delta scenario 'A reload while the stream is down' has no matrix row |
| R-42 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A game named clients"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 2 |  |  | delta scenario 'A game named clients' has no matrix row |
| R-43 | 3 | 4 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D5. The API"],"So the change adds `ClientEvent::Opened`, which `mu-client` sends for every `Hello`, the quiet one too."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | The scenario A new binding works without a restart needs config_changed to reach mu-tui, but mu-tui reads the stream only as ClientEvent items, and D5 adds only ClientEvent::Opened. No decision makes mu-client yield an item for config_changed; today Resume::next drops every message that is not a Hello or an Event. |
| R-44 | 3 | 4 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu config show prints one client configuration"],"The default of `--kind` SHALL be `tui`. The default of `--os` SHALL be the OS of the machine."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  | severity? should-fix 0.75 | The requirement of mu config show gives it the flags --kind and --os with defaults, and the scenario Another OS runs --os linux, but no decision creates either flag. D6 gives mu config only --server and --device. |
| R-45 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"Outside `clients/`, the error still stops the read, as today."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | D2 says a read error outside clients/ stops the snapshot as today, and TomlDirSource becomes a snapshot. But today the scan never reads server.toml: an unreadable server.toml is left to its reader, and mu world list still succeeds. A snapshot that reads the bytes of server.toml and stops on its read error makes mu world list fail, against the non-goal that mu world reads the directory as before. |
| R-46 | 3 | 4 | should-fix | in-change | drift | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"Each layer is a `Named` provider, the merge is `Figment::merge`, and the source of a field is `find_metadata`."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | D2 rebuilds in mu-server the figment2 block of the world merge: a Named provider per layer over a defaults layer, then find_metadata mapped back to a layer origin. resolve.rs already has that block, and the two copies can drift, for example in the find_value guard or the index-as-name trick. |
| R-47 | 3 | 4 | should-fix | in-change | drift | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"`mu-cli` gives the store a check function for it, `server_config::check`, which takes the bytes and returns the error of the file."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | D2 adds server_config::check for the reload. At start, server.toml is checked by server_config::read and then telemetry::export. Two checks of one file can drift: a value that telemetry::export refuses at start can pass the reload check. |
| R-48 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["tasks.md","4. The store"],"It writes one stderr line for each bad layer."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | Task 4.2 has the store in mu-server write its own stderr line for each bad layer. mu-cli already has report and warn, which write the line in one write and turn control characters into U+FFFD, so a file name under clients/ cannot send an escape sequence to the terminal. The design does not name them. |
| R-49 | 3 | 4 | should-fix | in-change | drift | ["artifact",["tasks.md","2. The crate mu-config"],"It holds the types of a client layer, `echo` and `[keys]`, with unknown fields refused."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | Task 2.1 puts the echo type of a client layer in mu-config, but task 6.1 removes only ClientConfig from mu-tui. So the Echo enum of mu-tui, with its default Dim, stays beside the new type. The default of echo is then stated in two crates. |
| R-50 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The connect_wait option"],"A value that is 0 or less, not finite, or less than 1 ns SHALL stop the command."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | The connect_wait rule refuses 0 or less, a value that is not finite and one under 1 ns, but task 5.2 names only a_wait_of_zero. No test sends nan, inf, a negative value or 1e-10 through the query, and no task keeps the unit test that covers them today. |
| R-51 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-bootstrap/spec.md","Requirement: A URL names a Unix socket"],"A URL with another scheme, a relative path, a host, a user, a password or a fragment SHALL stop the command before it connects."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | The URL rule refuses a user, a password and a fragment, but task 5.2 names tests only for a scheme, a host, no double slash and a relative path. A fragment such as unix:///tmp/s.sock#x keeps a good path, so a missing check connects silently and no test fails. |
| R-52 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The server URL"],"An empty `MU_SERVER` SHALL count as unset."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | An empty MU_SERVER must count as unset, but no test sets MU_SERVER to the empty string. If the parser reads it as a URL, every command with MU_SERVER= stops with a URL error. |
| R-53 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The device name"],"An empty `MU_DEVICE` SHALL count as unset."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | An empty MU_DEVICE must count as unset and fall back to the host name, but the only empty-name test is an_empty_device_flag, which expects a refusal. Nothing tells the empty variable from the empty flag. |
| R-54 | 3 | 4 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-core/src/world/dir.rs",150] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | A bad client layer must never stop the server, but the only walk-error test under clients/ is a broken link. A directory that stats but cannot be listed (clients/ itself, or clients/devices/ with mode 000) fails at read_dir. D2 does not say whether clients/ itself is under clients/, so the start can stop. |
| R-55 | 3 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu config reload reads the directory again"],"The good layers SHALL apply in both cases."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | A reload must apply the good client layers even when a file is bad, but D2 keeps a walk error outside clients/ fatal to the snapshot. A link to nothing in games/ then stops the reload before clients/ is read, and no good layer applies. The only bad-world reload test uses a parse error. |
| R-56 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","Context"],"`mu-tui` parses `client.toml`."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  | scope? deviates-design 0.73 | The Context says mu-tui parses client.toml. At head mu-tui holds the type ClientConfig, and mu-cli reads and parses the file through config_file::read. |
| R-57 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","D5. The API"],"-> revision, echo, echo_source, bindings[{key, entry, source}], notices[{path, reason}]"] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  | scope? deviates-design 0.67 | The GetClientConfig reply of D5 carries a separate echo_source field. The existing pattern keeps sources in one map keyed by field path. #172 adds the layout to the client configuration, so each new field there needs its own source field in the proto. |
| R-58 | 3 | 4 | suggestion | in-change |  | ["artifact",["tasks.md","5. The API and the bootstrap values"],"Add to `mu-cli` the parser of the server URL and of `connect_wait` (design.md D6)."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | Task 5.2 adds a parser of connect_wait. mu-cli already has connect_wait(), which applies the same rule: not finite, or under 1 ns, is no wait. It also has shown() for the time in the message. Neither D6 nor the task names them. |
| R-59 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"`figment2` splits a key path at each `.` and has no escape, so no canonical name holds a dot."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  | severity? should-fix 0.73 | The merge needs every canonical key name to hold no dot, because figment2 splits a path at each '.'. That rule covers every key name. Only the unit test for '.' checks it, while key_names_read_back already generates '.' and does not assert it. |
| R-60 | 3 | 4 | suggestion | in-change | false-test | ["artifact",["design.md","D4. The revision is a hash of the layers that apply"],"Each entry is framed, so no two lists give the same input"] | fixed | 6c632f1824bd951bfa34015b8745e39be539f233 | 4 |  | severity? should-fix 0.63 | D4 says the framing gives two different layer lists two different inputs. That rule covers every list, but task 4.4 tests one hand-made pair. |
| R-61 | 3 | 4 | suggestion | in-change |  | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"A nested table merges key by key in `figment2`, so `[keys]` gets its rule from the library."] | resolved | resolved in round 4 from fixed cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | D2 states as settled that figment2 merges a nested table key by key, which the design lists as an unchecked third-party fact, and task 4.3 builds on it. |
| R-62 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The server flag wins"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'The server flag wins' has no matrix row |
| R-63 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The server variable"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'The server variable' has no matrix row |
| R-64 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An empty server variable"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'An empty server variable' has no matrix row |
| R-65 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A user"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'A user' has no matrix row |
| R-66 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A password"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'A password' has no matrix row |
| R-67 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A fragment"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'A fragment' has no matrix row |
| R-68 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A negative wait"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'A negative wait' has no matrix row |
| R-69 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A wait that is not a number"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'A wait that is not a number' has no matrix row |
| R-70 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An infinite wait"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'An infinite wait' has no matrix row |
| R-71 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A wait under one nanosecond"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'A wait under one nanosecond' has no matrix row |
| R-72 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The device variable"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'The device variable' has no matrix row |
| R-73 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The device flag wins"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'The device flag wins' has no matrix row |
| R-74 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An empty device variable"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'An empty device variable' has no matrix row |
| R-75 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An unlistable clients directory"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'An unlistable clients directory' has no matrix row |
| R-76 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A control character in a layer name"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'A control character in a layer name' has no matrix row |
| R-77 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A bad kind in a call"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'A bad kind in a call' has no matrix row |
| R-78 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A bad OS in a call"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'A bad OS in a call' has no matrix row |
| R-79 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An unknown OS flag"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'An unknown OS flag' has no matrix row |
| R-80 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A reload with a link to nothing in games"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'A reload with a link to nothing in games' has no matrix row |
| R-81 | 4 | 4 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An unreadable server file leaves world list"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 4 |  |  | delta scenario 'An unreadable server file leaves world list' has no matrix row |
| R-82 | 5 | 5 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu config reload reads the directory again"],"It SHALL NOT apply a change to a world or to `server.toml`. For each such changed file, the command SHALL print one line that says that the change applies at restart."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | The reload must list a changed server.toml as applying at restart, but D2 builds the snapshot with the top-file rule, which drops server.toml, and says the snapshot does not read its bytes. No decision keeps the server.toml content of the start for the reload to compare. |
| R-83 | 5 | 5 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: The server checks every client layer at start"],"So is a file under `clients/` that is not one of the layer files."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  | severity? should-fix 0.83 | app-2: The requirement makes any file under clients/ that is not a layer file a bad layer, but D2 reuses the walk of TomlDirSource, which keeps only names that end in .toml, so clients/notes.txt or clients/global.toml~ is never reported.; inv-3: The spec makes any file under clients/ that is not a layer file a bad layer. The snapshot reuses the walk of TomlDirSource, which keeps only names that end in .toml. So clients/global.toml~ or clients/notes.txt is never reported. |
| R-84 | 5 | 5 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. A live change in the client"],"The effect calls `GetClientConfig`. The result replaces the keymap and the echo setting of the model, and each new notice shows once."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | D8 fetches GetClientConfig on every Opened and ConfigChanged with no in-flight guard. The closest pattern, the Listing state of the world list, keeps one call with the server and asks again after a reset; D8 names neither it nor a reason, so two crossing replies can apply the older revision. |
| R-85 | 5 | 5 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/support/mod.rs",22] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | The change adds MU_SERVER and MU_DEVICE as inputs of mu client and mu config, but no task adds them to test_env, the helper that removes MU_SOCKET so the developer's socket never reaches a test. A developer's MU_SERVER then reaches every test that gives no --server, such as The default. |
| R-86 | 5 | 5 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"The walk stops today at the first entry that it cannot read, stat or list. The snapshot does not stop."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  | severity? blocker 0.75 | The walk that DirSnapshot reuses follows links and recurses. Today the first error stops it, which bounds a link loop. D2 lets the snapshot keep each error and go on, so two links to . under clients/ make the walk exponential, and the start and each reload hang. |
| R-87 | 5 | 5 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"`mu config show` and `mu config reload` print the notices of their reply through `warn` too."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | D2 sends the notices of show and reload through warn, which turns control characters into U+FFFD. The restart lines of a reload name a changed file too, and go to stdout, and no decision sends them through the same filter. A new file name with ESC outside clients/ then reaches the terminal. |
| R-88 | 5 | 5 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-core/src/world/resolve.rs",210] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | In figment2 a None field serializes as Empty, and a merge takes the later value even when it is Empty. The world types skip None for this reason. No decision or task makes the client layer types do the same, so a device layer without echo can reset the echo that global sets. |
| R-89 | 5 | 5 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. The world scan skips `clients/` and rejects `client.toml`"],"It skips the directory `clients` at the top level only, so `characters/clients/<character>.toml` is still the world of a game named `clients`."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | The top-file rule of the scan ignores case and compares files, but D3 does not say whether the skip of clients does. If the scan skips Clients/ and the store reads only clients/, Clients/global.toml on macOS is silently ignored. No test passes Clients/. |
| R-90 | 5 | 10 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"Under `clients/`, the directory `clients/` itself included, the entry is a bad client layer."] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  | severity? should-fix 0.77 | D2 makes an unlistable clients/ or clients/devices/ a bad client layer, but the revision input of D4 lists only layer files. For a client with no file there, the input is empty before and after, so the revision stays and a live client that compares revisions shows no notice. |
| R-91 | 5 | 5 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. The API"],"At `Hello` the stream is already open. So a change after the fetch arrives on the stream."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | D5 relies on a stream receiving every config_changed sent after its Hello. The Hello is the head of the stream, and no decision or task says that the stream registers for config_changed before it sends the Hello. A receiver made after it loses a reload in between. |
| R-92 | 5 | 5 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. The API"],"After a reload that changes any client layer, the server sends `config_changed` on every stream."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | config_changed is useful only if it goes out after the new client part is visible to GetClientConfig. No named test fetches on config_changed and asserts the new revision, so a broadcast before the swap passes a_reload_sends_config_changed and races in the PTY tests. |
| R-93 | 5 | 5 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. The bootstrap values"],"`mu config show` and `mu config reload` take the same `--server` and `--device` flags, and never start a server."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | mu config reload must never start a server, but only mu config show has a test with no server. A reload that shares the start path of mu client passes every named test. |
| R-94 | 5 | 5 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"A changed world file or `server.toml` is listed as \\"applies at restart\\" and does not replace anything."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | D2 compares a reload with the current snapshot and does not replace its world part, so each reload lists a world change again until a restart. Only one reload is tested. A store that replaces the whole snapshot reports nothing on the second reload while the running worlds differ. |
| R-95 | 5 | 5 | should-fix | in-change | false-test | ["artifact",["specs/config-store/spec.md","Requirement: mu config reload reads the directory again"],"the user adds the world `beta/bryn` to `<dir>` and runs `mu config reload`"] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | The WHEN of 'A reload does not apply a world change' names the world beta/bryn but no file content and no game file, so a test cannot know what to write. |
| R-96 | 5 | 5 | should-fix | in-change | unmet-requirement | ["artifact",["specs/server-process/spec.md","Requirement: The client notes a server with another directory"],"**Reason**: **BREAKING** The client has no configuration directory, so it has nothing to compare with the directory of the server (#169)."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  |  | The BREAKING removal of 'The client notes a server with another directory' has no scenario for the removal; task 6.1 only says its tests are gone. |
| R-97 | 5 | 5 | suggestion | in-change |  | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"So the line of an error and the rule never to quote a value come from one parser."] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  | severity? should-fix 0.72 | D2 says parse_toml brings the rule never to quote a value. At head the parser never quotes the line; it drops a serde message, which can quote the value, only for a hidden table or Hidden::All. D2 names no Hidden value for the client layers. |
| R-98 | 5 | 5 | suggestion | in-change |  | ["artifact",["design.md","Enforcement"],"\| Two different layer lists give two different revision inputs \| A property test of the framing in `mu-server` \|"] | fixed | e6223fcbd0b854c49944d18f1878715e7592ece5 | 5 |  | severity? should-fix 0.72 | The Enforcement table puts a property test of the revision framing in mu-server, but the mu-server row of ALLOWED_CRATES has no proptest, and D6 and task 4.1 add only sha2 and mu-config to that row. The boundary test then refuses the dev-dependency. |
| R-99 | 5 | 9 | suggestion | in-change |  | ["artifact",["design.md","D1. A new crate, `mu-config`, holds the client fields and the key grammar"],"Its only allowed third-party crate is `serde`."] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  | severity? should-fix 0.65 | D1 allows mu-config one third-party crate, serde, and proptest only for tests. ALLOWED_CRATES applies to any dependency table, so proptest in [dependencies] passes workspace_boundary. |
| R-100 | 6 | 6 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The connect_wait option"],"So SHALL a query option with any other name, and an option that is there twice."] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  | severity? should-fix 0.71; repeat? R-23 0.55 | The connect_wait requirement refuses a query option with another name and an option that is there twice. D6 lists every refusal of the URL parser and names neither, and no decision says how the query is read, so a parser that keeps the last connect_wait meets D6. |
| R-101 | 6 | 6 | should-fix | in-change | false-test | ["artifact",["specs/config-store/spec.md","Scenario: A reload does not apply a world change"],"the user writes `host = \\"localhost\\"` and `port = 4201` to `<dir>/games/beta.toml`"] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  |  | The scenario A reload does not apply a world change writes games/beta.toml with host and port only. At head a game with no extends is a bad file, so beta/bryn can never resolve, and the test passes even if a reload applied a good world change. |
| R-102 | 6 | 6 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. The API"],"`WatchConfig` sends the current store hash when it opens, and every watcher gets the same later hashes."] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  |  | The new WatchConfig server stream has no stop rule. The one existing server stream, event_stream, ends on the stopping receiver so that the graceful shutdown does not wait for it. D5 and task 5.1 name neither that pattern nor a reason, so an attached client's open watch can hold mu server at SIGTERM. |
| R-103 | 6 | 6 | should-fix | in-change | drift | ["artifact",["design.md","D6. The bootstrap values"],"`workspace_boundary.rs` gets a check that `proptest` is only a dev-dependency of `mu-config` and `mu-server`."] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  |  | D6 adds a one-off check that proptest is only a dev-dependency of mu-config and mu-server. workspace_boundary.rs already has a dev-only mechanism, ALLOWED_DEV_EDGES with its Development-kind check. proptest is also dev-only in mu-core, mu-tui and mu-cli, and insta and tempfile are dev-only too, and the new check leaves them out. |
| R-104 | 6 | 6 | should-fix | in-change | drift | ["artifact",["design.md","D5. The API"],"When the watch stream ends, `mu-client` opens it again with the backoff of `Subscribe`."] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  |  | The watch stream re-opens with the backoff of Subscribe. The re-open loop of Resume (a wait flag, a sleep by the backoff, a new backoff at the first message) is in Resume::open and Resume::hello, and no decision names it, so task 7.1 writes a second copy of that loop. |
| R-105 | 6 | 10 | should-fix | in-change | drift | ["artifact",["design.md","D6. The bootstrap values"],"A message about a server that did not answer names the URL and that time, as for `mu client`."] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  | severity? suggestion 0.72 | mu config show and mu config reload each wait at most connect_wait and print the no-answer message of mu client. That block (a timeout of the first call and the message with the place and the time) is inline in client::run, and no decision makes it one function, so the three verbs get three copies of the message. |
| R-106 | 6 | 6 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. The API"],"The client applies a reply only if the `store_hash` of the reply equals the latest hash that it has seen."] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  |  | The client drops or applies a reply by its store_hash, so the store_hash of a reply must be the hash of the layers in that reply. No decision or task makes GetClientConfig read the hash and the layers from one swapped value; a reply that pairs a new hash with old layers is applied and never fetched again. |
| R-107 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: The server checks every client layer at start"],"For each bad layer, the server SHALL write one line to stderr that names the file and the reason."] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  |  | The server must write one line for each bad layer, and the line names the file. The filter that D2 names (warn, through report) keeps a line break, so a file under clients/ whose name holds a newline gives two lines, and the second can pose as another line of the log. A probe of report_to confirmed it. |
| R-108 | 6 | 7 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D8. A live change in the client"],"A reply with a new revision replaces the keymap and the echo setting of the model, and each new notice shows once."] | fixed | 8ba5492033e73ad84283d74b57e2b02541d17cbb | 7 |  |  | D8 shows each new notice once, but no named test passes the edge: the same bad layer in two replies with two revisions (another layer changed). A client that shows the notice again on every reload, or one that never shows a notice that comes back after a fix, passes every named test. |
| R-109 | 6 | 6 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7. The start of `mu client`"],"Make the first list, open `WatchConfig`, and call `GetClientConfig` after the first hash. All three fit within one wait of `connect_wait`."] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  | severity? blocker 0.69 | D7 bounds the first list, the first hash of WatchConfig and GetClientConfig by one wait of connect_wait. At head the timeout wraps only the first list, and task 6.1 adds the two calls with no deadline and no test; a server that answers the list but not the watch hangs mu client before the screen, with no message. |
| R-110 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-config/spec.md","Requirement: The echo field sets the echo line"],"The field `echo` SHALL be one of `show`, `dim` and `hide`, and its default SHALL be `dim`."] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  |  | The requirement 'The echo field sets the echo line' has no task line that names its subject. Only the word 'echo' matches in tasks 2.1 and 6.1, and no task line holds two of its distinctive words. |
| R-111 | 6 | 6 | should-fix | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Scenario: A second reload lists the world change again"],"the scenario \\"A reload does not apply a world change\\" ran"] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  |  | The GIVEN of 'A second reload lists the world change again' names no concrete state. It only says that another scenario ran, so a test cannot set up the state from this scenario alone. |
| R-112 | 6 | 6 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"[A host name with characters that a device name refuses] \\u2192 The command stops and names the host name and `--device`."] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  | severity? should-fix 0.83 | The risk of a host name that a device name refuses is mitigated by a message that names the host name and --device. No task produces that message or tests it; task 5.2 names only the_host_name. |
| R-113 | 6 | 6 | suggestion | in-change |  | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"So it keeps the device and inode of each directory on its current path."] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  |  | D2 hand-writes a loop check of the walk with the device and inode of each directory on the path, and a walk that keeps each error and goes on. walkdir, with follow_links, does both and is already in Cargo.lock, and same-file, which mu-core already uses, compares two entries as one file. D2 names neither. |
| R-114 | 6 | 6 | suggestion | in-change |  | ["artifact",["tasks.md","2. The crate mu-config"],"Make `mu-tui` build its keymap from the neutral key names of `mu-config`, and map each one to a `termina` chord."] | fixed | d38f405169d905c7c28108475ab447c475659d7b | 6 |  |  | Task 2.4 adds a map in mu-tui from the neutral key of mu-config to a termina chord, over every character key. A binding works only if that chord equals the chord that of_key makes from the key event the terminal sends; that round trip holds over an unbounded set of keys and no test states it. |
| R-115 | 7 | 12 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu config reload reads the directory again"],"The command SHALL print one line for each bad file to stderr."] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  | severity? should-fix 0.85 | The reload must print one stderr line for each bad file, but D2 checks the world files through mu_core::world::resolve, which returns only the first error of the layers, so a reload with two bad world files prints one line. |
| R-116 | 7 | 7 | should-fix | in-change | unmet-requirement | ["path","crates/mu-cli/src/env_path.rs",18] | fixed | 8ba5492033e73ad84283d74b57e2b02541d17cbb | 7 |  |  | Every flag with a variable in mu-cli is one clap arg with env and the value parser parse_path, which turns an empty value into none, so the empty flag and the empty variable are one case. The device rule needs them apart (empty MU_DEVICE is unset, --device "" is refused). D6 names neither the pattern nor how the device flag leaves it. |
| R-117 | 7 | 7 | should-fix | in-change | drift | ["path","crates/mu-tui/src/model.rs",85] | fixed | 8ba5492033e73ad84283d74b57e2b02541d17cbb | 7 |  |  | D8 has the model keep the notices that it showed, to show each one once. The model already keeps every notice line in Model::notices, for replay after a Reset. D8 does not name that list, so a second record of shown notices can come in beside it, and the two can disagree on what was shown. |
| R-118 | 7 | 7 | should-fix | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: The server checks every client layer at start"],"For each bad layer, the server SHALL write one line to stderr that names the file and the reason."] | fixed | 8ba5492033e73ad84283d74b57e2b02541d17cbb | 7 |  |  | One bad layer must give one stderr line, but D2 turns line breaks into U+FFFD only in the path of a notice. The reason names the binding and the entry raw, so a binding "alt+\\n" or an entry with a line break gives two lines. |
| R-119 | 7 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: The server checks every client layer at start"],"A bad client layer SHALL NOT stop the start."] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  |  | D2 keeps each walk error as a path with its error. walkdir 2.5.0 gives no path when a link under clients/ points to a directory that cannot be opened. The reader cannot place that error under clients/, so it either stops the start or is dropped with no line. |
| R-120 | 7 | 7 | should-fix | in-change | false-test | ["artifact",["specs/world-config/spec.md","Scenario: A stray file"],"a directory with one of `worlds.toml`, `characters/alpha.toml`, `games/old/alpha.toml` and `characters/alpha/aria/x.toml`"] | fixed | 8ba5492033e73ad84283d74b57e2b02541d17cbb | 7 |  |  | The scenario 'A stray file' GIVEN names four files after 'one of', and its THEN says 'that path'. No single input is fixed, so a test that passes for one file can leave the other three unchecked. |
| R-121 | 7 | 7 | should-fix | in-change | false-test | ["artifact",["specs/config-store/spec.md","Scenario: The same files give the same revision"],"GIVEN a server serves `<dir>` on `<dir>/s.sock`"] | fixed | 8ba5492033e73ad84283d74b57e2b02541d17cbb | 7 |  |  | The GIVEN of 'The same files give the same revision' names no layer file, so the revision is compared on an empty set of layers. A revision that ignores the layer contents would still pass. |
| R-122 | 7 | 7 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"[The user misses the notice] \\u2192 The prominent notice of #172 is the follow-up."] | fixed | 8ba5492033e73ad84283d74b57e2b02541d17cbb | 7 |  | severity? should-fix 0.77 | The mitigation of the risk that the user misses the notice is a follow-up change (#172), which the proposal puts out of this change; it names no task and no code of this change. |
| R-123 | 7 | 7 | suggestion | in-change |  | ["path","crates/mu-core/src/portal.rs",228] | fixed | 8ba5492033e73ad84283d74b57e2b02541d17cbb | 7 |  |  | The store keeps a current client part that GetClientConfig reads and a separate publish of its store hash to watchers, with a hand-kept rule that the publish comes after the swap. tokio::sync::watch, the pattern of the world status in Portal, holds the current value and wakes each receiver after the value is visible; D2 and D5 do not name it. |
| R-124 | 7 | 9 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-cli/src/lib.rs",33] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  | severity? blocker 0.70 | D2 adds a third control-character filter in mu-server for notice paths. report_to in mu-cli and printable in mu-tui already map control characters to U+FFFD, and the three differ only in which characters they keep (line break and tab, tab as a space, none). |
| R-125 | 7 | 7 | suggestion | in-change |  | ["artifact",["design.md","D5. The API"],"Otherwise a newer fetch is already on its way, so two replies can never apply in the wrong order."] | fixed | 8ba5492033e73ad84283d74b57e2b02541d17cbb | 7 |  |  | Applying a reply only when its store hash is the latest hash is an ordering claim over any mix of hashes and replies, including A then B then A. Three example tests in update.rs cover three fixed orders. |
| R-126 | 8 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D6. The bootstrap values"],"The path is percent-decoded."] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  |  | The server URL replaces SocketArgs for mu client, but D6 and D7 name neither SocketArgs::private_socket, which creates and checks the private socket directory before the client opens server.log, nor default_socket for the default URL. |
| R-127 | 8 | 9 | should-fix | in-change | drift | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"The snapshot also keeps the bytes of `server.toml`, only so that a reload can see a change to it."] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  |  | server.toml is read twice at each start and each reload: server_config::check reads it from disk through config_file::read, and DirSnapshot reads its bytes again for the restart compare, so the checked file and the compared bytes can differ. |
| R-128 | 8 | 9 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-server/src/service.rs",185] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  |  | Each method of PortalApi makes its own span with call_span or traced, which the requirement A span for each API call needs. D5 and task 5.1 add GetClientConfig, WatchConfig and ReloadConfig and name neither the pattern nor the span rule of a stream. |
| R-129 | 8 | 9 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7. The start of `mu client`"],"The notices of `GetClientConfig` go to the screen through the existing `notices` of `mu_tui::run`."] | fixed | 31d549cfc19395e87242bb1158a210d7134bf1ec | 9 |  |  | fit-5: A GetClientConfig reply is applied in two places: at start mu-cli passes keymap and echo through Settings and the notices through the notices argument of mu_tui::run, and on a new hash the model applies the reply with its store hash and its last notices (D8).; inv-7: Each notice shows once. The attach notices go through notices of mu_tui::run, and D8 compares a new reply with the last applied reply. No test starts with a bad layer, then reloads a change to another layer, so a model that does not seed the last reply from the attach reply shows the notice twice. |
| R-130 | 8 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"So the reload resolves each world on its own, from its character file and the game files that it names. Each bad world file is then reported once"] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  |  | D2 resolves each world on its own and says each bad world file is then reported once. A file in the chain of several worlds (global.toml, a type, or a game with two characters) is checked once per world, and no step removes the repeated lines. |
| R-131 | 8 | 9 | should-fix | in-change | unmet-requirement | ["path","crates/mu-core/src/world/dir.rs",56] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  |  | A stray world file or a name outside the id rule fails in LayerSource::layers, which stops at the first such file. D2 keeps that path, so a reload with two stray files prints one line, against one line for each bad file. |
| R-132 | 8 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu config reload reads the directory again"],"If every file is good, the exit status SHALL be 0."] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  |  | Exit 0 must hold when every file is good, also when the reload lists a world or server.toml change for a restart. The two scenarios that list a restart line assert no exit status, so a reload that exits 1 on a pending restart passes every named test. |
| R-133 | 8 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["proposal.md","Choices you will notice in use"],"A bad layer, so that a misspelled name is reported."] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  |  | The store checks the name of each kind, OS and device file, but only the kind and the device branches have a test. A file clients/os/windows.toml or clients/os/macOS.toml is never sent through the store. |
| R-134 | 8 | 9 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"It keeps every file under `clients/`, whatever its name ends with, so the store can report a file that is not a layer file."] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  | scope? deviates-design 0.83; severity? blocker 0.74 | The snapshot reads every non-directory entry under clients/ whatever its name. Nothing limits the read to regular files, so a FIFO (or a link to one, or to /dev/zero) blocks the start and holds the reload mutex. A probe shows a FIFO read still blocked after 2 s. |
| R-135 | 8 | 9 | suggestion | in-change |  | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"The change moves the filter to one function in `mu-config` that takes the characters to keep."] | resolved | resolved in round 9 from fixed 06ceb7c506813ad781d986fb789fb4a2731f3e57 | 8 |  |  | The shared control filter must give text with no control character for any string, which is what makes one bad layer one line. The planned tests are fixed examples (ESC, a line break). |
| R-136 | 9 | 9 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"The snapshot keeps the device and inode of each directory that a walk entered. A link to one of them is an error of the link."] | fixed | 31d549cfc19395e87242bb1158a210d7134bf1ec | 9 |  |  | The loop guard of the R-119 fix makes a link to any directory that the walk already entered an error, not only a link to an ancestor. Today the walk of the world files follows a link to a sibling directory, such as characters/alpha2 to characters/alpha, and makes its worlds. Under the new rule that link is an error that stops the world resolve, and only when the real directory was entered first. D2 names no walk order, so the same directory gives worlds or an error by the read order of the file system. This goes against the non-goal that mu world reads the directory as before. |
| R-137 | 9 | 9 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"The restart lines of a reload name a file, so they go to stdout through the same filter."] | fixed | 31d549cfc19395e87242bb1158a210d7134bf1ec | 9 |  |  | The fix of R-124 gives each caller its own map, and the map of report_to keeps the line break and the tab. D2 sends the restart lines of a reload through that same filter, while only the notices are mapped by the store with U+FFFD for every control character. A new world file whose name holds a line break is a changed file, so its restart line splits into two stdout lines, against one line for each changed file. |
| R-138 | 9 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A layer for an unknown OS"] | accepted | ticket: none; reason: Each scenario has a named test in tasks 4.2 and 4.5; the implementation-phase full review builds the coverage matrix rows for them. |  |  |  | delta scenario 'A layer for an unknown OS' has no matrix row |
| R-139 | 9 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A FIFO under clients"] | accepted | ticket: none; reason: Each scenario has a named test in tasks 4.2 and 4.5; the implementation-phase full review builds the coverage matrix rows for them. |  |  |  | delta scenario 'A FIFO under clients' has no matrix row |
| R-140 | 9 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A reload with a bad game of two worlds"] | accepted | ticket: none; reason: Each scenario has a named test in tasks 4.2 and 4.5; the implementation-phase full review builds the coverage matrix rows for them. |  |  |  | delta scenario 'A reload with a bad game of two worlds' has no matrix row |
| R-141 | 9 | 9 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A reload with two stray files"] | accepted | ticket: none; reason: Each scenario has a named test in tasks 4.2 and 4.5; the implementation-phase full review builds the coverage matrix rows for them. |  |  |  | delta scenario 'A reload with two stray files' has no matrix row |
| R-142 | 10 | 10 | blocker | in-change | wrong-behaviour | ["path","crates/mu-core/src/world/snapshot.rs",110] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | On a file system that ignores case, server.toml named in another case (SERVER.toml) is no longer read at the start or at a reload. The snapshot keeps the entry under its name on disk, and server_file looks up the exact key server.toml. Before the change, config_file::read opened <dir>/server.toml, which the file system resolved to SERVER.toml. So its [telemetry] table is now ignored, and a bad SERVER.toml no longer stops the start. A probe at head ran mu server on a directory with SERVER.toml holding [telemetry] endpont = "x", and the server served. With server.toml, the same file stops the start with exit 1. |
| R-143 | 10 | 10 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. The revision is a hash of the layers that apply"],"The error of a directory on the path of a layer of the client also counts, as an entry with tag 1."] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | ClientPart::resolve adds a notice for any bad entry at clients/, clients/kinds, clients/os or clients/devices. The revision counts such an entry only when it is a walk error. Example: a regular file or a FIFO named clients/os on a server whose client has no OS layer. GetClientConfig returns a new notice with the same revision, so D8 changes nothing. A live client never shows that notice, but it shows at attach. |
| R-144 | 10 | 10 | should-fix | in-change | drift | ["path","crates/mu-core/src/world/dir.rs",67] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  | severity? suggestion 0.87 | The change writes the read of a snapshot entry as UTF-8 text three times: the world layer in dir.rs, server.toml in server_config.rs and the client layer in store.rs, each with its own copy of the UTF-8 message and the not-regular case. |
| R-145 | 10 | 10 | should-fix | in-change | drift | ["path","crates/mu-core/src/world/dir.rs",152] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  | severity? suggestion 0.69 | The file path of a ConfigError place, without the line and the column, is cut in two places: path_of in dir.rs, which world_errors uses to keep one error per path, and an inline copy in the reload of store.rs, which names the path of each notice. |
| R-146 | 10 | 10 | should-fix | in-change | drift | ["path","crates/mu-cli/src/client.rs",145] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | The GetClientConfig request of the terminal client (kind tui, the OS of the machine, the device) is built twice: once by mu-cli before the screen and once by config_of in mu-tui on each new hash. A change to one makes the first reply and the later ones describe two different clients. |
| R-147 | 10 | 10 | should-fix | in-change | drift | ["path","crates/mu-server/src/store.rs",151] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | The store states the layout of the layer files four times: the prefixes of layer_rule, the file list of resolve, LAYER_DIRS and the text NOT_A_LAYER. A new layer place must change all four, and a missed one makes a file a bad layer or leaves it out of the revision. |
| R-148 | 10 | 10 | should-fix | in-change | drift | ["path","crates/mu-config/src/names.rs",33] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | Kind::parse and Os::parse write the list of names into their error text by hand, beside Kind::ALL and Os::ALL. A kind added to ALL, as the remote clients and the MCP server of the proposal may need, leaves the message naming the old list. |
| R-149 | 10 | 10 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/config_store.rs",559] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | a_clients_directory_in_another_case returns before any assertion on a case-sensitive file system, so it is vacuous on Linux CI. |
| R-150 | 10 | 10 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/config_store.rs",133] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | an_unlistable_clients_directory returns with no assertion when the process can list a mode 000 directory (root), so the scenario is silently untested. |
| R-151 | 10 | 10 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/config_store.rs",157] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | a_link_to_an_unreadable_directory_under_clients returns with no assertion for root; also it only asserts a non-empty list of lines, not the one line the spec names. |
| R-152 | 10 | 10 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/config_store.rs",834] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | a_file_that_cannot_be_read returns with no assertion when the file opens despite mode 000 (root). |
| R-153 | 10 | 10 | should-fix | in-change | false-test | ["path","crates/mu-server/tests/store.rs",309] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  | severity? blocker 0.71 | an_unreadable_layer_changes_the_revision returns with no assertion when mode 000 does not lock the file (root). |
| R-154 | 10 | 10 | should-fix | in-change | false-test | ["path","crates/mu-server/tests/store.rs",326] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | an_unlistable_clients_directory_changes_the_revision returns with no assertion when mode 000 does not lock the directory (root). |
| R-155 | 10 | 10 | suggestion | in-change |  | ["artifact",["design.md","D4. The revision is a hash of the layers that apply"],"`Sha256::new()`, one `Digest::update` call for each slice, then `finalize()`."] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | D4 names the sha2 calls Sha256::new, one Digest::update for each slice, then finalize. The code builds one framed Vec and calls Sha256::digest once. |
| R-156 | 10 | 10 | suggestion | in-change |  | ["artifact",["design.md","D6. The bootstrap values"],"Every flag of `mu-cli` with a variable is one `clap` argument with `env` and the value parser `parse_path`, which turns an empty value into none."] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  | severity? should-fix 0.75 | D6 says that every mu-cli flag with a variable is one clap argument with env and parse_path. At head, --server has env, but its value is a plain String, and its own filter makes an empty flag unset too. D6 names only --device as the exception. |
| R-157 | 10 | 10 | suggestion | in-change |  | ["path","crates/mu-server/src/store.rs",145] | fixed | c23878086671b1a01ca1858a1dc8a5fa12246650 | 10 |  |  | digest hand-writes the hex form of the first 8 bytes of the hash. The output of Sha256::digest in sha2 0.10.9 is a GenericArray, whose LowerHex takes a precision, so format!("{:.16x}", ...) gives the first 16 hex digits. The design does not name it. |
| R-158 | 10 | 16 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"test-honesty partial"] | accepted | ticket: none; reason: a full round 11 reruns the test-honesty lens over the files it did not read |  |  |  | lens test-honesty reported partial, gaps: 1 |
| R-159 | 11 | 14 | blocker | in-change | wrong-behaviour | ["path","crates/mu-core/src/world/snapshot.rs",177] | resolved | resolved in round 14 from fixed 50ec9a0257165f27f220ff2393dde8f63467143d | 13 |  | severity? should-fix 0.61 | The snapshot marks server.toml as a top file only when it is a regular file. A FIFO or other non-regular entry named server.toml now stops mu world list and the server start with a stray-file error. The base scan skipped any top-level entry of that name. |
| R-160 | 11 | 12 | should-fix | in-change | drift | ["path","crates/mu-core/src/world/snapshot.rs",130] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | The rule of which snapshot paths lie under clients/ is written twice: is_under_clients in snapshot.rs, and an inline copy in changed() of store.rs that picks the paths outside clients/ for the restart lines. |
| R-161 | 11 | 12 | should-fix | in-change | drift | ["path","crates/mu-cli/tests/support/process.rs",428] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | The change adds support::process::server_url, which percent-escapes the socket path, and four more test copies that build the server URL from the path with no escape, in bootstrap.rs and config_store.rs. |
| R-162 | 11 | 12 | should-fix | in-change | drift | ["artifact",["design.md","D6. The bootstrap values"],"They move to one function in `crates/mu-cli/src/client.rs`, `within`, that takes the wait, a future and the message of a late answer."] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  | severity? suggestion 0.62 | D6 adds within to mu-cli: a timeout of a call that gives a late message. mu_tui::run::within already does this, with a different message and with whole seconds. D6 does not name it. |
| R-163 | 11 | 12 | should-fix | in-change | drift | ["path","crates/mu-cli/src/bootstrap.rs",61] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | BootstrapArgs::server works out the default socket from XDG_RUNTIME_DIR and the uid itself. SocketArgs::resolve, which mu server uses, already does this. If one copy changes, mu client and mu server look for different default sockets. |
| R-164 | 11 | 12 | should-fix | in-change | drift | ["path","crates/mu-config/src/layer.rs",13] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | Echo has two spellings of its values. serde rename_all spells them for a layer file. Echo::name and Echo::named spell them for the reply and the client. The world enums, such as KeepaliveMode, use serde alone. The two spellings can drift apart, and then mu config show prints a value that a layer file refuses. |
| R-165 | 11 | 12 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-core/src/world/snapshot.rs",195] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | D2 says the world read ignores a read error of server.toml. A link to nothing at server.toml is kept as an Error entry outside top_files, so mu world list fails on it. The only boundary test uses a mode-000 regular file. |
| R-166 | 11 | 12 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-core/src/world/snapshot.rs",212] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | The loop guard proves only that the walk ends, not that its size is bounded. With k sibling directories under clients/ that link to each other, the walk enters every simple path: 56 links took 11.5 s and 205 MB, about 8 times more per directory. The start and each reload (under the reload mutex) take that time. |
| R-167 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-core/tests/snapshot.rs",62] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  | severity? blocker 0.69 | The test returns early with no assertion when the user can read a mode-000 directory (root), so it passes without checking the link error. |
| R-168 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",164] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | a_host passes if a host is accepted: no server listens on /tmp/s.sock, and the connect error prints the whole URL. |
| R-169 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",169] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | a_user passes if a user is accepted: the connect error prints the whole URL, which holds me@. |
| R-170 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",174] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  | severity? blocker 0.63 | a_password passes if a password is accepted: the connect error prints the whole URL, which holds me:pw@. |
| R-171 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",185] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | no_double_slash passes if unix:/tmp/s.sock is accepted: the connect error prints the URL text. |
| R-172 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",190] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | a_relative_path passes if unix:s.sock is accepted: the connect error prints the URL text. |
| R-173 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/client.rs",1484] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | The test returns early as root and asserts nothing about the notice for an unlistable clients directory. |
| R-174 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",205] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | a_wait_of_zero passes if the refusal is removed: a failed connect also exits 1 and prints the URL, which holds connect_wait=0. |
| R-175 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",210] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | a_negative_wait passes if the refusal is removed: the connect error echoes the URL with connect_wait=-1 and exits 1. |
| R-176 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",215] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | a_wait_that_is_not_a_number passes without the refusal: a failed connect echoes the URL with connect_wait=nan. |
| R-177 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",220] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | an_infinite_wait passes without the refusal: a failed connect echoes the URL with connect_wait=inf. |
| R-178 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",225] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | a_wait_under_one_nanosecond passes without the refusal: a failed connect echoes the URL with connect_wait=1e-10. |
| R-179 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",230] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | an_unknown_option passes without the refusal: the failed connect prints the URL, which holds colour=red. |
| R-180 | 11 | 12 | should-fix | in-change | false-test | ["path","crates/mu-cli/tests/bootstrap.rs",235] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | an_option_twice passes if a repeated option is accepted: the failed connect prints the URL with connect_wait twice. |
| R-181 | 11 | 12 | suggestion | in-change |  | ["artifact",["design.md","D7. The start of `mu client`"],"and passes no attach notices in the `notices` argument."] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | D7 says that mu-cli passes no attach notices in the notices argument of mu_tui::run. At head, mu_tui::run has no notices argument. It takes the client, the device and the GetClientConfig reply. |
| R-182 | 11 | 12 | suggestion | in-change |  | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"`find_metadata` takes a key path, finds the value with `Value::find_ref`, which splits the path at each `.`"] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  | severity? should-fix 0.74 | D2 says find_metadata finds the value with Value::find_ref. In figment2 0.11.5, find_metadata calls find_value, which uses Value::find. Both split the path at each '.', so the rule of D2 still holds. Only the cited name is wrong. |
| R-183 | 11 | 12 | suggestion | in-change |  | ["artifact",["design.md","D5. The API"],"notices[{path, reason}]"] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  |  | The D5 sketch gives a notice the fields path and reason. The proto Notice at head has path and text, and text is the whole notice with the path in it, as D2 states. A reader of D5, such as the writes of #173 or an MCP server, looks for a reason field that does not exist. |
| R-184 | 11 | 12 | suggestion | in-change |  | ["path","crates/mu-config/src/names.rs",64] | resolved | resolved in round 12 from fixed db98ea07b5a72094dca2dfa5ab076edfbaf2d264 | 11 |  | scope? beyond-scope 0.63 | Kind::parse and Os::parse are the same lookup by name in ALL, with a NameError. Only the word for the name and the plural in the reason differ. Echo::named is the same lookup over an inline list. |
| R-185 | 11 | 17 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"invariants partial"] | accepted | ticket: none; reason: every one of the 163 covering tests passed at head, test-honesty read every changed file over four agents, and the partial lenses only matched by name or skipped specs outside their focus |  |  |  | lens invariants reported partial, gaps: 3 |
| R-344 | 15 | 15 | should-fix | in-change | unmet-requirement | ["path","crates/mu-tui/src/update.rs",854] | fixed | 38a8ff2a6599d4e1a8b319b5fe4b71f4e4760926 | 15 |  |  | config_failed counts and notices a failed fetch even when the configuration is no longer stale. Fetches run concurrently, so an older fetch that fails after a newer one applied shows a spurious notice and leaves config_failures at 1 until the next apply. The next real failure then shows no notice and waits 200 ms, against the scenario A failed fetch is tried again (one notice, a wait of 100 ms). |
| R-345 | 15 | 15 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A reload of a removed directory"] | fixed | 38a8ff2a6599d4e1a8b319b5fe4b71f4e4760926 | 15 |  |  | delta scenario 'A reload of a removed directory' has no matrix row |
| R-346 | 15 | 15 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The server writes the notices of a reload"] | fixed | 38a8ff2a6599d4e1a8b319b5fe4b71f4e4760926 | 15 |  |  | delta scenario 'The server writes the notices of a reload' has no matrix row |
| R-347 | 15 | 15 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An open socket directory"] | fixed | 38a8ff2a6599d4e1a8b319b5fe4b71f4e4760926 | 15 |  |  | delta scenario 'An open socket directory' has no matrix row |
| R-348 | 15 | 15 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A control character in the reply"] | fixed | 38a8ff2a6599d4e1a8b319b5fe4b71f4e4760926 | 15 |  |  | delta scenario 'A control character in the reply' has no matrix row |
| R-349 | 15 | 15 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A failed fetch is tried again"] | fixed | 38a8ff2a6599d4e1a8b319b5fe4b71f4e4760926 | 15 |  |  | delta scenario 'A failed fetch is tried again' has no matrix row |
| R-350 | 16 | 16 | should-fix | in-change | next-change | ["artifact",["specs/config-store/spec.md","Requirement: A failed fetch is tried again"],"the client SHALL show one notice and SHALL fetch again after a wait that grows with each failure."] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | The requirement 'A failed fetch is tried again' (one notice, then a fetch again after a growing wait, reset by an apply) has no decision. D8 names only the fetch on a new hash and the apply; the retry lives only in update.rs and an Enforcement row. |
| R-351 | 16 | 16 | should-fix | in-change | next-change | ["artifact",["specs/config-store/spec.md","Requirement: The server keeps and writes the result of a reload"],"If the configuration directory does not exist at a reload, the server SHALL keep the current client layers"] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | The requirement 'The server keeps and writes the result of a reload' has no decision: no decision says that a reload of a missing directory keeps the current client layers and fails with a line that names the directory. D2 covers only a walk error inside the snapshot. |
| R-352 | 16 | 16 | should-fix | in-change | next-change | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"The store writes nothing to stderr. It returns its notices, and `mu-cli` writes each one through `warn`"] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | D2 says the store writes nothing to stderr and mu-cli writes each notice through warn. At head the reload in the store logs each notice itself with tracing::warn, and design.md records no deviation. |
| R-353 | 16 | 16 | should-fix | in-change | next-change | ["artifact",["specs/config-store/spec.md","Requirement: mu config reaches only a private socket"],"`mu config show` and `mu config reload` SHALL refuse a socket directory that exists and that another user owns"] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | The requirement 'mu config reaches only a private socket' has no decision. D6 names private_dir for mu client and says mu config creates no directory, but no decision has mu config refuse an open or foreign socket directory (checked_dir at head) or filter what show prints to stdout. |
| R-354 | 16 | 16 | should-fix | in-change | next-change | ["artifact",["specs/server-process/spec.md","Requirement: The client starts a server"],"The client SHALL NOT pass `--config` to the new server, which finds its directory from the environment that it inherits."] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | No decision removes --config from mu client or keeps it off the server that the client starts. The scenario 'The client takes the flag' (exit 2) and the rule that the client passes no --config rest on the proposal alone; D7 says only that the directory compare goes away. |
| R-355 | 16 | 16 | should-fix | in-change | drift | ["path","crates/mu-server/src/store.rs",461] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | A bad client layer reaches the server's stderr by two mechanisms. At start, mu-cli writes each store notice through warn. At reload, the store writes it with tracing::warn!. So one bad layer gives two line formats, and D2 says the store writes nothing to stderr. |
| R-356 | 16 | 16 | should-fix | in-change | drift | ["path","crates/mu-core/src/world/snapshot.rs",310] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | snapshot.rs writes the case rule of a top-level name (the same name in any case, and the same entry by same_entry) twice: is_top_file for server.toml, and an inline copy in Walker::parts for clients. The copies already differ. is_top_file turns a stat error into an error entry, but the clients copy turns it into 'not clients' with unwrap_or(false). |
| R-357 | 16 | 16 | should-fix | in-change | next-change | ["path","crates/mu-server/src/store.rs",319] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | The source paths of a client layer are an inline vec!["echo"] in ClientPart::resolve, with inline defaults. The closest pattern, the world merge, keeps the paths in FIELD_PATHS beside the field types, and the test each_field_has_a_source guards them. #172 adds layout fields, and a field missing from the inline list gets no source, so mu config show prints it as default. D2 and D5 name neither the pattern nor a reason. |
| R-358 | 16 | 16 | should-fix | in-change | false-test | ["path","crates/mu-core/src/world/snapshot.rs",229] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | The walk enters at most LINKED_DIRS_MAX directories through links, but the only test asserts limits > 0 and a loose size bound. An off-by-one that enters 65 directories, or a check of > in place of ==, still passes it. |
| R-359 | 16 | 16 | should-fix | in-change | unmet-requirement | ["path","crates/mu-cli/src/client.rs",38] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | connect_wait refuses a value under 1 ns and must accept 1 ns itself. Tests pass only 1e-10 and 0. No test passes 1e-9, so a change of >= to > refuses the edge the spec allows and every test still passes. |
| R-360 | 16 | 16 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-tui/src/update.rs",222] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | The failure count of a fetch resets only at an apply. If the hash returns to the applied one with no reply (a reverted file and a reload), the count stays. The next failed fetch then shows no notice and waits 200 ms, not 100 ms. |
| R-361 | 16 | 16 | should-fix | in-change | false-test | ["path","crates/mu-server/tests/store.rs",283] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  |  | the_revision_frames_each_layer passes without length framing: path bytes between the two layers already make the unframed inputs differ. |
| R-362 | 16 | 16 | suggestion | in-change |  | ["artifact",["design.md","D6. The bootstrap values"],"So the parser uses `Url::parse`, `scheme`, `has_authority`, `host_str`, `username`, `password`, `fragment`, `query_pairs` and `to_file_path`."] | fixed | c91dfedc43728b96578685fcaf4237fcd9afdd2a | 16 |  | severity? should-fix 0.74 | D6 lists username and password among the calls of the URL parser. parse_url at head calls neither; a user or a password is refused only because Url::parse fails on the empty host. |
| R-363 | 17 | 17 | should-fix | in-change | next-change | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The client names itself to the server"],"`mu client` SHALL ask the server for the configuration of the kind `tui`, the OS of the machine, and its device name."] | fixed | 62a3d903b5451f1c20efaf650ab3295eb03587ff | 17 |  |  | The requirement 'The client names itself to the server' (kind tui, the OS of the machine) has no decision for mu client. D6 gives the kind and OS defaults to mu config show only. D7 and D8 call GetClientConfig without naming its arguments. The code holds the rule in config::fetch. The other client kinds that the proposal plans (remote clients, an MCP server) have no decision to follow. |
| R-364 | 17 | 17 | should-fix | in-change | next-change | ["path","crates/mu-cli/src/config_verb.rs",142] | fixed | 62a3d903b5451f1c20efaf650ab3295eb03587ff | 17 |  |  | render of mu config show writes the echo line from a literal path. mu world show instead loops over FIELD_PATHS with value_of and a guard test. FIELD_PATHS of mu-config says it gives the order of mu config show, but render never reads it, so mu config show silently leaves out any field that #172 adds to FIELD_PATHS. |
| R-365 | 17 | 17 | should-fix | in-change | drift | ["path","crates/mu-cli/tests/bootstrap.rs",30] | fixed | 62a3d903b5451f1c20efaf650ab3295eb03587ff | 17 |  |  | The new test file bootstrap.rs adds its own helper refused, which asserts exit status 1 and a stderr that holds each text. This is the job of support::fails_naming, the shared helper that world_config.rs and telemetry.rs call 61 times. |
| R-366 | 17 | 17 | should-fix | in-change | unmet-requirement | ["path","crates/mu-core/src/world/snapshot.rs",306] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/182; reason: the same defect as QUAL-19 in issue #181: key the snapshot by OsString or give a non-UTF-8 name an error entry |  |  |  | The snapshot keys each path by its lossy UTF-8 text, so two non-UTF-8 file names under clients/ that differ only in their invalid bytes become one path. The second insert replaces the first, so two bad layers give one stderr line and one notice, and the shadowed file is not in the store hash. |
| R-367 | 17 | 17 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-cli/src/client.rs",39] | fixed | 62a3d903b5451f1c20efaf650ab3295eb03587ff | 17 |  |  | connect_wait refuses a finite value above about 1.8e19 s, because Duration::try_from_secs_f64 overflows, and the message then says it is not a finite number of 1 ns or more. The spec refuses only 0 or less, not finite, or under 1 ns. No test passes the upper edge. |
| R-368 | 17 | 17 | should-fix | in-change | unmet-requirement | ["path","crates/mu-config/src/key.rs",167] | fixed | 62a3d903b5451f1c20efaf650ab3295eb03587ff | 17 |  | scope? beyond-scope 0.74 | The spec makes shift with a character that is not a letter a bad binding, but the parser checks is_lowercase. shift+U+2170 (small roman numeral one, Nl) and shift+U+24D0 (circled small a, So) are Lowercase and pass, and shift with a caseless letter such as U+05D0 is refused. |
| R-369 | 17 | 17 | suggestion | in-change |  | ["artifact",["design.md","D1. A new crate, `mu-config`, holds the client fields and the key grammar"],"The row of `mu-config` in `ALLOWED_CRATES` names `serde` and `proptest`, because that check also reads dev-dependencies."] | fixed | 62a3d903b5451f1c20efaf650ab3295eb03587ff | 17 |  |  | D1 allows mu-config only serde, with a row of serde and proptest. At head the row also names toml, a dev-dependency of mu-config, and design.md records no deviation. |
| R-370 | 17 | 17 | suggestion | in-change |  | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"The store writes nothing to stderr itself."] | fixed | 62a3d903b5451f1c20efaf650ab3295eb03587ff | 17 |  | severity? should-fix 0.75 | D2 says the store writes nothing to stderr itself and gives each notice to the sink. When the snapshot task of a reload fails, the store also logs through tracing::error, so one failure reaches the server log twice in two formats. |
| R-371 | 17 | 17 | suggestion | in-change |  | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"If the merge of the good layers fails, the client part uses the defaults and gives one notice."] | fixed | 62a3d903b5451f1c20efaf650ab3295eb03587ff | 17 |  | severity? should-fix 0.74 | D2 says a failed merge of the good layers makes the client part use the defaults with one notice. At head the merge runs for each client in ClientPart::resolve. Its notice goes only into that GetClientConfig reply. The reload notices and the server stderr never get it, and the reload still exits 0. |
| R-372 | 17 | 17 | suggestion | in-change |  | ["path","crates/mu-config/tests/keys.rs",189] | fixed | 62a3d903b5451f1c20efaf650ab3295eb03587ff | 17 |  | severity? should-fix 0.60 | The grammar accepts any one character that is not a control character, not uppercase and not +, and lets shift go with any lowercase character, but key_names_read_back draws its keys from ASCII only. The round trip, the no-dot rule and the shift rule over non-ASCII keys (such as é, a titlecase letter or a fullwidth stop) are never generated. |
| R-373 | 17 | 17 | should-fix | in-change | unmet-requirement | ["artifact",["review","lens-status"],"test-coverage partial"] | accepted | ticket: none; reason: every one of the 163 covering tests passed at head, test-honesty read every changed file over four agents, and the partial lenses only matched by name or skipped specs outside their focus |  |  |  | lens test-coverage reported partial, gaps: 2 |

## Reviewed files

| path | content |
|---|---|
| Cargo.lock | 1050d63e83e2b8d280cd2081487dc385e61fd9b9 |
| Cargo.toml | e78f9d0a9c18451aa5b4841ae5b5ea19a9c9400b |
| architecture/model.c4 | 57efc47d38ab8d091fc45278356f6225e52620e0 |
| architecture/views.c4 | 089983f9b810713767ab80770a20d556455e046d |
| crates/mu-cli/Cargo.toml | c1fc9f7c542a93a820e08f655f08e959133d0489 |
| crates/mu-cli/src/bootstrap.rs | c8e408b4c725ace592611b9bcf03103c18c24623 |
| crates/mu-cli/src/client.rs | ef79b53029a36bef21027a4557096893067ab22b |
| crates/mu-cli/src/config_file.rs | 0b4bb1bc1734e095a93da357efaa49defa239ee6 |
| crates/mu-cli/src/config_verb.rs | df6b5446bd3ee8ffcb6bfbe16b0540d5752a97c8 |
| crates/mu-cli/src/lib.rs | 38821a654e56d33d606b0cb70d5c3c73a1f4d89b |
| crates/mu-cli/src/server.rs | adb8596ed302c00a4c50077e523439040673563d |
| crates/mu-cli/src/server_config.rs | 2cc909c126c791802487888bd92884f286f09f1b |
| crates/mu-cli/src/socket.rs | 572e8c547719bb268ca9a06207f6a797cdf9c8e3 |
| crates/mu-cli/tests/bootstrap.rs | 1eacbd505e4fff0d0da0843f7f37aae1b19135d2 |
| crates/mu-cli/tests/cli.rs | b8d8d0c42184f67280cee6f03008a928cbc1b728 |
| crates/mu-cli/tests/client.rs | 414dcdb06632cfaa82073e8cb912ce3dcbba18a1 |
| crates/mu-cli/tests/config_store.rs | f159ddaa6dc567e294493a65324a8ac029040ad0 |
| crates/mu-cli/tests/server.rs | c8187b3c41d4744438447579175527eedaf9a51c |
| crates/mu-cli/tests/support/mod.rs | 3eb693474d9011199d0738076deb3d6cc318cef8 |
| crates/mu-cli/tests/support/process.rs | 4c2dba2036cda6d3658a705cf39596d5a619848e |
| crates/mu-cli/tests/telemetry.rs | 445b219fe1f0991b751a73a0257ce23d838c3dc7 |
| crates/mu-cli/tests/workspace_boundary.rs | b794b6c5ed88c34f12c58786ca5c5bb10471b447 |
| crates/mu-cli/tests/world_config.rs | 2196d6727d92719459c9df70a3be9ab6c2e7107e |
| crates/mu-client/src/lib.rs | a5119f73be50f90c313532e9cd65966981f3c678 |
| crates/mu-config/Cargo.toml | 88782c13c2b8ab6648fc876174beaf837d6aaaf2 |
| crates/mu-config/src/entries.rs | c370c0fb5df7488f47598cb3c8743d35585d3b12 |
| crates/mu-config/src/filter.rs | 1d9211887f44127409a7e3a4073e9061fbefd336 |
| crates/mu-config/src/key.rs | 9118c919774ffb428d61ccf49c88c1f07f31a121 |
| crates/mu-config/src/layer.rs | 82d2342512945626b14bf0a75ae491f392c46f54 |
| crates/mu-config/src/lib.rs | a835025c0f895ecace18062dcc8411ab49dd0359 |
| crates/mu-config/src/names.rs | 6ef3c9529a623bbe814b7253c58bf649ccc128af |
| crates/mu-config/tests/filter.rs | 079c8cbdae4bee04d58a50671e353d46978650bd |
| crates/mu-config/tests/keys.rs | bb5cf24bf25e6493a4fb71a876935f74474077dc |
| crates/mu-config/tests/layer.rs | cf435aaa15031c25cbcf7372cee807f2aae3dfb7 |
| crates/mu-config/tests/names.rs | d638942e8bf197267f1c49b7a4bdeeed741e1fda |
| crates/mu-core/Cargo.toml | b656a43167026ec80d8f3dcdbd3e814beb9e57a9 |
| crates/mu-core/src/world/dir.rs | 9f8922541795b28e4fec3e390c51f61fad3b98b0 |
| crates/mu-core/src/world/error.rs | 03a6baeaed46cc964dcd214c32997d650b507387 |
| crates/mu-core/src/world/merge.rs | 8a983e35e23556411e260bf2c031e1b8e6e07aee |
| crates/mu-core/src/world/mod.rs | eda2e248b7ad3b0691a8c320a45fa27a724757f5 |
| crates/mu-core/src/world/resolve.rs | 8a876532606736602185ecbc71d491f0f9b16f04 |
| crates/mu-core/src/world/snapshot.rs | 23889ce2dd5b4bf7d1ce4a7fc9c3d515d6af8dc0 |
| crates/mu-core/tests/snapshot.rs | 9b9a337909b09e68758ae5486751d1f4f368d239 |
| crates/mu-proto/proto/mu/v1/portal.proto | 1cc61ac8f992927724d21d5f0eaec79a000e6952 |
| crates/mu-server/Cargo.toml | 7202d3211f25c2708f335f83cafa2a6b915543be |
| crates/mu-server/src/lib.rs | 7ccb4a4323be6f54e245c1d774ddcb8ea753db6b |
| crates/mu-server/src/serve.rs | d1609036cd448031117a836cd5be4867fa6890d3 |
| crates/mu-server/src/service.rs | 99c2a105fbb39a1694e096155565034dd74ea681 |
| crates/mu-server/src/store.rs | e1895c958e9d81afef8ccc0996ac8775c1338c9d |
| crates/mu-server/tests/config_api.rs | e6fb87d815fdf8bc3dceb749c6ade73e37dbe120 |
| crates/mu-server/tests/store.proptest-regressions | 3c9fea0c261bebfa8a8749284b5fde16b429a727 |
| crates/mu-server/tests/store.rs | c1215b8f146d5b2de8fe5b93fe5bb633e1cb66de |
| crates/mu-tui/Cargo.toml | 3262d77d63d3fb01667e89dee193433e2637bcee |
| crates/mu-tui/src/config.rs | 59b896887ba53a1d96a582386c4189f2d602075e |
| crates/mu-tui/src/keymap.rs | 1b60266e2d012163ab6dff879b99a1b57610a398 |
| crates/mu-tui/src/lib.rs | a63c5958eef8aaac8dee5ad0034b260ab8b18ff6 |
| crates/mu-tui/src/model.rs | 64c4e9d318936cf283447c48c81497e2820eed2d |
| crates/mu-tui/src/run.rs | 0a60b352c05446e6fd146ef28d95993af346e23f |
| crates/mu-tui/src/text.rs | 493248060c333e890196085ef6fc1be2a9b65a9a |
| crates/mu-tui/src/update.rs | 8d8d468c0c715ef337b3ef216cd808ae7372fc75 |
| crates/mu-tui/tests/app.rs | f306b1e88247808630337d827452ba146f537bdb |
| crates/mu-tui/tests/keymap.rs | 94decbdf7633009227997cbdf39bc051a6719316 |
| crates/mu-tui/tests/update.proptest-regressions | f6c6a9dd68ea23ba04711ca882a7c630471ff619 |
| crates/mu-tui/tests/update.rs | 478a6f539a33d4440835da28a7293a5aec883f8b |
| docs/src/content/docs/architecture/crate-layout.mdx | 3054ce7e1a3861696434eadc5e0133b1cc711cfa |
| docs/src/content/docs/architecture/server-and-clients.mdx | dd2549b652c032fd908536906aac3e36cae1d846 |
| docs/src/content/docs/architecture/terminal-client.mdx | 22a5cdea73a87b05105957ea98856f3a533de4f5 |
| docs/src/content/docs/contributing/workspace.md | 9e3d4cba29efd751c37df149509d985429212841 |
| docs/src/content/docs/decisions/configuration-ownership.md | 4d1cbdbe01c149dcfa7b4626d17bce8d8a2fcee4 |
| docs/src/content/docs/decisions/terminal-client-architecture.md | eda053b8161cd2f0213e9a093c2285dffc817338 |
| docs/src/content/docs/guides/client.md | 05afd654110ad9920c788865a47eda7031a7dcd0 |
| docs/src/content/docs/guides/configuration.md | e217c216bfaa09fbbab774fed20c046554b0cd1b |
| docs/src/content/docs/guides/first-run.md | 273e4a32b56b7167b7b6099f1a837ca81f1e8985 |
| docs/src/content/docs/guides/telemetry.md | 6d232a5907a7bc0a7927b44310fa9c3d3b64520c |
| docs/src/content/docs/index.md | 94e8a3be6d9512c32b039819b4d07c1deeee46f0 |
| mise.toml | 3483112b3eb01ff8abaf18c07f2a87193069e984 |
| openspec/changes/config-store/.openspec.yaml | a0171eea41a9ef975ac5c180be78365a94f5bee0 |
| openspec/changes/config-store/design.md | e405e13b53b8baab3cb323a9380a889a9a3b57a1 |
| openspec/changes/config-store/proposal.md | d459ab83ac317ba928f478856d2e07d050e79cb9 |
| openspec/changes/config-store/specs/cli-entrypoint/spec.md | e58709b8a27afcfd635388b7e7982ac633245c70 |
| openspec/changes/config-store/specs/client-bootstrap/spec.md | 195967911288c2fa7a8dbd622508f039a5277927 |
| openspec/changes/config-store/specs/client-commands/spec.md | 68a8762e8fc76955a5637ec22bba61a5039d5e5c |
| openspec/changes/config-store/specs/client-config/spec.md | d4f770f9cafb6de690262ed42c28b1d39304757f |
| openspec/changes/config-store/specs/config-store/spec.md | 2f6f1e20df72d1901f99aad63cc060deb3e6399e |
| openspec/changes/config-store/specs/server-process/spec.md | b042cd312cd519e5aad0413a62a18d3f38b10d23 |
| openspec/changes/config-store/specs/terminal-client/spec.md | 161a454fcf502072a84ac2b52c10b782f2e6ecaf |
| openspec/changes/config-store/specs/world-config/spec.md | 356551ef3788fafa9bb3434524b4138d12ce4f77 |
| openspec/changes/config-store/tasks.md | sha256:eb582007a4968e65469b2c928134472c58d1aee6da3b29c84adb519207d2112b |
| openspec/review.json | absent |

## Plan review (2026-11-23)

Verdict approve-with-notes at head 82a1f77d83c213e83f83784a18d06da5bb914926.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-138 | test-coverage | should-fix | in-change | delta scenario 'A layer for an unknown OS' has no matrix row | review > absent-row — "A layer for an unknown OS" | add the row with its covering task or test |  |  | unmet-requirement |
| R-139 | test-coverage | should-fix | in-change | delta scenario 'A FIFO under clients' has no matrix row | review > absent-row — "A FIFO under clients" | add the row with its covering task or test |  |  | unmet-requirement |
| R-140 | test-coverage | should-fix | in-change | delta scenario 'A reload with a bad game of two worlds' has no matrix row | review > absent-row — "A reload with a bad game of two worlds" | add the row with its covering task or test |  |  | unmet-requirement |
| R-141 | test-coverage | should-fix | in-change | delta scenario 'A reload with two stray files' has no matrix row | review > absent-row — "A reload with two stray files" | add the row with its covering task or test |  |  | unmet-requirement |
| R-129 | verify | should-fix | in-change | PARTIAL: the fix 06ceb7c506813ad781d986fb789fb4a2731f3e57 of R-129 did not hold. fit-5: A GetClientConfig reply is applied in two places: at start mu-cli passes keymap and echo through Settings and the notices through the notices argument of mu_tui::run, and on a new hash the model applies the reply with its store hash and its last notices (D8).; inv-7: Each notice shows once. The attach notices go through notices of mu_tui::run, and D8 compares a new reply with the last applied reply. No test starts with a bad layer, then reloads a change to another layer, so a model that does not seed the last reply from the attach reply shows the notice twice. | design.md > D7. The start of `mu client` — "The notices of `GetClientConfig` go to the screen through the existing `notices` of `mu_tui::run`." | fix it again | regression |  | wrong-behaviour |
| R-136 | verify | should-fix | in-change | The loop guard of the R-119 fix makes a link to any directory that the walk already entered an error, not only a link to an ancestor. Today the walk of the world files follows a link to a sibling directory, such as characters/alpha2 to characters/alpha, and makes its worlds. Under the new rule that link is an error that stops the world resolve, and only when the real directory was entered first. D2 names no walk order, so the same directory gives worlds or an error by the read order of the file system. This goes against the non-goal that mu world reads the directory as before. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "The snapshot keeps the device and inode of each directory that a walk entered. A link to one of them is an error of the " | Make a link an error only when its target is a directory on the path of the current walk, as the loop check of walkdir does, or apply the entered-directory rule under clients/ only. Add a world-side test with a link to a sibling directory. |  |  | wrong-behaviour |
| R-137 | verify | should-fix | in-change | The fix of R-124 gives each caller its own map, and the map of report_to keeps the line break and the tab. D2 sends the restart lines of a reload through that same filter, while only the notices are mapped by the store with U+FFFD for every control character. A new world file whose name holds a line break is a changed file, so its restart line splits into two stdout lines, against one line for each changed file. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "The restart lines of a reload name a file, so they go to stdout through the same filter." | Map the path of each restart line with the store map (U+FFFD for every control character) before it reaches warn's filter, and add a line break case to a_reload_filters_a_control_character_in_a_restart_line. |  |  | wrong-behaviour |

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| The server flag wins | 5.2 | covered |  |
| The server variable | 5.2 | covered |  |
| An empty server variable | 5.2 | covered |  |
| The default | 5.2 | covered |  |
| MU_SOCKET does not count | 5.2 | covered |  |
| A path with a space | 5.2 | covered |  |
| Another scheme | 5.2 | covered |  |
| A host | 5.2 | covered |  |
| A user | 5.2 | covered |  |
| A password | 5.2 | covered |  |
| A fragment | 5.2 | covered |  |
| No double slash | 5.2 | covered |  |
| A relative path | 5.2 | covered |  |
| A wait of zero | 5.2 | covered |  |
| A negative wait | 5.2 | covered |  |
| A wait that is not a number | 5.2 | covered |  |
| An infinite wait | 5.2 | covered |  |
| A wait under one nanosecond | 5.2 | covered |  |
| An unknown option | 5.2 | covered |  |
| An option twice | 5.2 | covered |  |
| A short wait | 5.2 | covered |  |
| The device variable | 5.2 | covered |  |
| The device flag wins | 5.2 | covered |  |
| An empty device variable | 5.2 | covered |  |
| The host name | 5.2 | covered |  |
| A bad name | 5.2 | covered |  |
| A name of 64 characters | 5.2 | covered |  |
| A name of 65 characters | 5.2 | covered |  |
| An empty device flag | 5.2 | covered |  |
| The kind layer applies | 6.2 | covered |  |
| The device layer applies | 6.2 | covered |  |
| The OS layer applies | 6.2 | covered |  |
| The socket flag | 6.1 | covered |  |
| Connect the world in focus | 6.4 | covered |  |
| Disconnect a named world | 6.4 | covered |  |
| A key bound to connect with no world in focus | 6.4 | covered |  |
| Connect with no world in focus | 6.4 | covered |  |
| A trailing space is no argument | 6.4 | covered |  |
| Disconnect an unknown world | 6.4 | covered |  |
| A server table | 5.4 | covered |  |
| An unknown field | 5.4 | covered |  |
| Not TOML | 5.4 | covered |  |
| A file that cannot be read | 5.4 | covered |  |
| Two names of one key | 5.4 | covered |  |
| An uppercase key | 5.4 | covered |  |
| A control character | 5.4 | covered |  |
| Shift with a digit | 5.4 | covered |  |
| An unknown entry | 5.4 | covered |  |
| An unknown key name | 5.4 | covered |  |
| An entry that needs an argument | 5.4 | covered |  |
| A binding of Shift+Tab | 6.2 | covered |  |
| A binding of Ctrl+Shift+Z | 6.2 | covered |  |
| A new binding | 6.2 | covered |  |
| The default keys stay | 6.2 | covered |  |
| A removed binding | 6.2 | covered |  |
| A binding replaces a default | 6.2 | covered |  |
| None on a key with a modifier | 6.2 | covered |  |
| The default dims | 6.3 | covered |  |
| Show | 6.3 | covered |  |
| Hide | 6.3 | covered |  |
| A bad echo value | 6.3 | covered |  |
| A bad device layer at start | 4.2 | covered |  |
| A file that is not a layer | 4.2 | covered |  |
| A device file with a bad name | 4.2 | covered |  |
| A link to nothing under clients | 4.2 | covered |  |
| A binding of a dot | 5.3 | covered |  |
| An unlistable clients directory | 4.2 | covered |  |
| A control character in a layer name | 4.2 | covered |  |
| A line break in a layer name | 4.2 | covered |  |
| A line break in a binding | 4.2 | covered |  |
| A link to an unreadable directory under clients | 4.2 | covered |  |
| A file that is not TOML | 4.2 | covered |  |
| A loop of links under clients | 4.2 | covered |  |
| A clients directory in another case | 4.2 | covered |  |
| A layer for an unknown kind | 4.2 | covered |  |
| No clients directory | 5.3 | covered |  |
| A device with no file | 5.3 | covered |  |
| The device wins | 5.3 | covered |  |
| Keys merge key by key | 5.3 | covered |  |
| Two names of one key in two layers | 5.3 | covered |  |
| A layer without echo keeps the earlier echo | 5.3 | covered |  |
| One bad binding skips its whole layer | 5.3 | covered |  |
| A notice for a bad layer that applies | 5.3 | covered |  |
| A bad device name in a call | 5.1 | covered |  |
| A bad kind in a call | 5.1 | covered |  |
| A bad OS in a call | 5.1 | covered |  |
| No notice for a bad layer of another device | 5.3 | covered |  |
| The same files give the same revision | 5.3 | covered |  |
| A changed layer gives a new revision | 5.3 | covered |  |
| A layer of another device leaves the revision | 5.3 | covered |  |
| No server | 5.3 | covered |  |
| Another OS | 5.3 | covered |  |
| An unknown OS flag | 5.3 | covered |  |
| Show filters a control character in a notice | 5.3 | covered |  |
| A reload applies a client layer | 5.3 | covered |  |
| A reload with a bad layer | 5.3 | covered |  |
| A reload does not apply a world change | 5.3 | covered |  |
| A reload with two bad world files | 4.5 | covered |  |
| A reload with a bad world file | 5.3 | covered |  |
| A reload with a link to nothing in games | 4.5 | covered |  |
| A reload lists a changed server file | 5.3 | covered |  |
| A second reload lists the world change again | 5.3 | covered |  |
| A reload with no server | 5.3 | covered |  |
| A new binding works without a restart | 7.1 | covered |  |
| A reload while the stream is down | 7.1 | covered |  |
| A reload across a server restart | 7.1 | covered |  |
| A bad layer after a reload | 7.1 | covered |  |
| A clients directory that turns unlistable | 7.1 | covered |  |
| A bad device layer at attach | 6.2 | covered |  |
| A client with no server starts one | 6.1 | covered |  |
| No server answers in time | 6.1 | covered |  |
| Two clients start at the same time | 6.1 | covered |  |
| A server that closes the connection | 6.1 | covered |  |
| A server that does not answer | 6.1 | covered |  |
| The started server finds the default directory | 6.1 | covered |  |
| The started server inherits MU_CONFIG | 6.1 | covered |  |
| No notice about another directory | 6.1 | covered |  |
| No terminal | 6.1 | covered |  |
| No terminal on stdin | 6.1 | covered |  |
| Before the first list | 6.5 | covered |  |
| A send before the first list | 6.5 | covered |  |
| Text typed before the first list | 6.5 | covered |  |
| An event before the first list | 6.5 | covered |  |
| A focus key before the first list | 6.5 | covered |  |
| A failed first list after an event | 6.5 | covered |  |
| A notice before the first list | 6.5 | covered |  |
| The first list fails | 6.5 | covered |  |
| The flag wins over the environment | 3.3 | covered |  |
| The default directory | 3.3 | covered |  |
| The directory under the home directory | 3.3 | covered |  |
| An empty MU_CONFIG | 3.3 | covered |  |
| A relative XDG_CONFIG_HOME | 3.3 | covered |  |
| No home directory | 3.3 | covered |  |
| A missing directory | 3.3 | covered |  |
| A missing default directory | 3.3 | covered |  |
| The client warns of a missing directory | 6.1 | covered |  |
| The client takes the flag | 6.1 | covered |  |
| Worlds from character files | 3.3 | covered |  |
| A character with no game | 3.3 | covered |  |
| A file name outside the id rule | 3.3 | covered |  |
| A stray file | 3.3 | covered |  |
| A stray character file | 3.3 | covered |  |
| A stray game file | 3.3 | covered |  |
| A stray file under a character | 3.3 | covered |  |
| A link to nothing | 3.3 | covered |  |
| The file of the server | 3.3 | covered |  |
| A character named server | 3.3 | covered |  |
| The file of the client | 3.2 | covered |  |
| An unreadable server file leaves world list | 4.2 | covered |  |
| A game named clients | 3.2 | covered |  |
| A bad client layer does not stop the worlds | 3.2 | covered |  |
| A leftover client.toml | 3.2 | covered |  |

### Not run

Plan phase: no verification runs.
