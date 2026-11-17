---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-17T17:00:53+00:00
head: edc00b3e1bc9459a93fafafe81c76ace52c2a6e2
round: 10
next_id: 94
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
| R-92 | fit | should-fix | in-change | The new warn in lib.rs copies fail line for line: the same one-write stderr call, the same format prefix and the same comment on a closed stderr. Its doc even says it writes as fail does, so a change to the policy of fail must be made twice. | crates/mu-cli/src/lib.rs:28 — "let _ = std::io::stderr().write_all(format!("mu {verb}: warning: {message}\\n").as_bytes());" | Make warn call fail with the text prefixed by `warning: ` and drop its result, so the stderr write and its one-write rule have one copy. |  |  |
| R-93 | invariants | should-fix | in-change | The new print helper splits stdout write errors in two: BrokenPipe returns success, and any other kind fails the verb with status 1. No test passes either side, so a helper that swallows all errors or fails on BrokenPipe still passes every test. | crates/mu-cli/src/lib.rs:35 — "Err(error) if error.kind() != std::io::ErrorKind::BrokenPipe => fail(verb, &error)," | Move the write into a helper that is generic over the writer, used by print. In lib.rs, add a unit test with two failing writers. One returns ErrorKind::BrokenPipe and must give success. The other returns another kind, such as StorageFull, and must give status 1 with `mu <verb>:` on stderr. Or run `mu version` in a test with stdout on a pipe whose read end is closed. |  |  |

## Repeats

None.

## Out of bound

| lens | severity | claim | evidence |
|---|---|---|---|
| test-honesty | should-fix | each_field_has_a_source takes its expected key set from FIELD_PATHS, the list the resolver loops over to fill sources, so a field dropped from FIELD_PATHS passes. | crates/mu-core/tests/world_config.rs:75 — "assert_eq!(paths, FIELD_PATHS.iter().copied().collect());" |

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-80 | suggestion | accepted | Only TomlDirSource checks which layer kinds may set extends: a game must name a type, and global, character and plugin layers must not. The resolver does not check it, so a later layer source, such as the Lua source, can give a game with no type or a character with extends, and nothing refuses it. | https://github.com/seanb4t/mu-portal/issues/85 |
| R-81 | note | accepted | The wrong-type requirement of the spec already covers a table written as an array, and the test names the case; a scenario of its own adds no new behavior. |  |
| R-83 | note | accepted | The test covers the skip side of the file-name rule of the design; a scenario of its own adds no new behavior, as with R-81. |  |
| R-86 | should-fix | accepted | The lens had no diff tool. The only hunk it did not read is the switch of cli.rs to mu_command, which the fit and invariants lenses checked in this round. |  |
| R-88 | should-fix | accepted | TomlDirSource documents a missing root as an empty configuration, which the server needs; the CLI check only names a directory that the user chose, so the two checks answer different questions. |  |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 24231853aa33a631c42924f6d4fbd71382a53c2a | 43d710b636d358e848719344785e9f0dd0ec877ee969ec935c24da9f81cc5acb | changes-requested | 37 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 11c17ba73571a89b109c3b521bf7545997170757 | e7474d5caadfca061fb8aab67156e20ee6fd8b966a8e8353ca2598c592d7e29e | changes-requested | 14 | 0 | 2 | 0 |  |
| 3 | plan | follow-up | d480822aa96f2e98c97aa8da76f641e75423ca0e | 8ef3974cf52d28db4892f08142a7378b4c978d2c3b049fbd071714cacb8076f9 | changes-requested | 9 | 0 | 2 | 1 |  |
| 4 | plan | follow-up | 370a2621585b22b9a902ca04813279ce01fe8c00 | 7044b6b91f696a97397d08c9e3eb4a5f67cb2f250036e146010936a74d956b8c | changes-requested | 7 | 0 | 2 | 3 | operator decision |
| 5 | implementation | full | 53dd9579d67dea304ac7811dd0223d9c7d3ee53d | 36fec2cc349d5aa2f698a10a821107a64b23b8949455863ada08443502e777fb | changes-requested | 9 | 0 | 0 | 0 |  |
| 6 | implementation | follow-up | b29e2c0b13402597119e0600af32f34c36db9acf | 1a73ecd38a0293386993af0f23199524ba176cb81b66f81db33e35cb8a34c73b | approve-with-notes | 5 | 0 | 0 | 1 |  |
| 7 | implementation | follow-up | d1c4e17fae9f518a8197a438d66ac6fc044c1248 | 7c044470648832a32489633be15a56f3ffc25f0f128e54634db96580ea019807 | approve-with-notes | 5 | 0 | 0 | 1 |  |
| 8 | implementation | follow-up | e93268cd0285174dd3a37447c882c356a18a8c13 | cb84c7c9a3d25d111612b019fcc03f33b433d9182307c242d6dbad5eb5636cbc | approve | 0 | 0 | 0 | 0 | operator decision |
| 9 | implementation | follow-up | 9e9821d94110bd81d0680322241d02e47a5d55d7 | dd280ce143c9e8e44f06b96d5ca1cd1a1febdce25c2eb0712c376e9507a56c82 | approve-with-notes | 5 | 2 | 0 | 2 | operator decision |
| 10 | implementation | follow-up | edc00b3e1bc9459a93fafafe81c76ace52c2a6e2 | 24b95a47109fda1f02ee2b69b93ca580129193043b7e2f082937c0e39ebd3d58 | approve-with-notes | 2 | 0 | 0 | 1 | operator decision |

## Judge

judge: 4 questions, 0.000 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| The flag wins over the environment | crates/mu-cli/tests/world_config.rs: flag_wins_over_the_environment | covered | pass |
| The default directory | crates/mu-cli/tests/world_config.rs: default_directory | covered | pass |
| The directory under the home directory | crates/mu-cli/tests/world_config.rs: directory_under_home | covered | pass |
| An empty MU_CONFIG | crates/mu-cli/tests/world_config.rs: empty_mu_config | covered | pass |
| A relative XDG_CONFIG_HOME | crates/mu-cli/tests/world_config.rs: relative_xdg_config_home | covered | pass |
| No home directory | crates/mu-cli/tests/world_config.rs: no_home_directory | covered | pass |
| A missing directory | crates/mu-cli/tests/world_config.rs: missing_directory | covered | pass |
| A missing default directory | crates/mu-cli/tests/world_config.rs: missing_default_directory_is_quiet | covered | pass |
| Worlds from character files | crates/mu-cli/tests/world_config.rs: worlds_from_character_files | covered | pass |
| A character with no game | crates/mu-cli/tests/world_config.rs: character_with_no_game | covered | pass |
| A file name outside the id rule | crates/mu-cli/tests/world_config.rs: file_name_outside_the_id_rule | covered | pass |
| A stray file | crates/mu-cli/tests/world_config.rs: stray_file | covered | pass |
| A link to nothing | crates/mu-cli/tests/world_config.rs: link_to_nothing_is_an_error | covered | pass |
| The last layer wins | crates/mu-cli/tests/world_config.rs: last_layer_wins | covered | pass |
| A global default reaches each world | crates/mu-cli/tests/world_config.rs: global_default_reaches_each_world | covered | pass |
| A table merges field by field | crates/mu-cli/tests/world_config.rs: table_merges_field_by_field | covered | pass |
| A user type extends a built-in type | crates/mu-cli/tests/world_config.rs: user_type_extends_built_in_type | covered | pass |
| An unknown type | crates/mu-cli/tests/world_config.rs: unknown_type | covered | pass |
| A game with no type | crates/mu-cli/tests/world_config.rs: game_with_no_type | covered | pass |
| A loop of types | crates/mu-cli/tests/world_config.rs: loop_of_types | covered | pass |
| A type that extends itself | crates/mu-cli/tests/world_config.rs: type_that_extends_itself | covered | pass |
| A character that names a parent | crates/mu-cli/tests/world_config.rs: character_that_names_a_parent | covered | pass |
| Entries from each layer | crates/mu-cli/tests/world_config.rs: entries_from_each_layer | covered | pass |
| A layer disables an inherited rule | crates/mu-cli/tests/world_config.rs: layer_disables_an_inherited_rule | covered | pass |
| A layer replaces an inherited rule | crates/mu-cli/tests/world_config.rs: layer_replaces_an_inherited_rule | covered | pass |
| An entry with no id | crates/mu-cli/tests/world_config.rs: entry_with_no_id | covered | pass |
| A disable with no target | crates/mu-cli/tests/world_config.rs: disable_with_no_target | covered | pass |
| An id twice in one layer | crates/mu-cli/tests/world_config.rs: id_twice_in_one_layer | covered | pass |
| A rule and a script with one id | crates/mu-cli/tests/world_config.rs: rule_and_script_with_one_id | covered | pass |
| Each built-in type reads as quirks | crates/mu-core/tests/world_config.rs: built_in_types_hold_only_quirks | covered | pass |
| A global preference is not hidden by a type | crates/mu-cli/tests/world_config.rs: global_preference_is_not_hidden_by_a_type | covered | pass |
| A user type with a built-in name | crates/mu-cli/tests/world_config.rs: user_type_with_a_built_in_name | covered | pass |
| An unknown field | crates/mu-cli/tests/world_config.rs: unknown_field | covered | pass |
| A port out of range | crates/mu-cli/tests/world_config.rs: port_out_of_range | covered | pass |
| A template with no password | crates/mu-cli/tests/world_config.rs: template_with_no_password | covered | pass |
| The case of a character set | crates/mu-cli/tests/world_config.rs: charset_case | covered | pass |
| A character set that the engine cannot read | crates/mu-cli/tests/world_config.rs: charset_the_engine_cannot_read | covered | pass |
| The edges of each range | crates/mu-cli/tests/world_config.rs: edges_of_each_range | covered | pass |
| A line break in a line field | crates/mu-cli/tests/world_config.rs: line_break_in_a_line_field | covered | pass |
| A password in a file | crates/mu-cli/tests/world_config.rs: password_in_a_file | covered | pass |
| A secret that is not a reference | crates/mu-cli/tests/world_config.rs: secret_that_is_not_a_reference | covered | pass |
| A value of the wrong type under login | crates/mu-cli/tests/world_config.rs: wrong_type_under_login | covered | pass |
| Show does not read the secret | crates/mu-cli/tests/world_config.rs: show_does_not_read_the_secret | covered | pass |
| A plugin sets a field of the connection | crates/mu-core/tests/world_config.rs: plugin_cannot_set_the_connection | covered | pass |
| A plugin sets a preference | crates/mu-core/tests/world_config.rs: plugin_can_set_a_preference | covered | pass |
| A world with no host or port | crates/mu-cli/tests/world_config.rs: world_with_no_host_or_port | covered | pass |
| A login name with no secret | crates/mu-cli/tests/world_config.rs: login_name_with_no_secret | covered | pass |
| A login secret with no name | crates/mu-cli/tests/world_config.rs: login_secret_with_no_name | covered | pass |
| Half a keepalive | crates/mu-cli/tests/world_config.rs: half_a_keepalive | covered | pass |
| A world that logs in by hand | crates/mu-cli/tests/world_config.rs: world_that_logs_in_by_hand | covered | pass |
| Sources of fields | crates/mu-cli/tests/world_config.rs: sources_of_fields | covered | pass |
| An unknown world | crates/mu-cli/tests/world_config.rs: unknown_world | covered | pass |
| A bad configuration stops the start | crates/mu-cli/tests/worlds.rs: bad_configuration_stops_the_start | covered | pass |
| A fake world with a configured id | crates/mu-cli/tests/worlds.rs: fake_world_with_a_configured_id | covered | pass |
| The secret is not read at the start | crates/mu-cli/tests/worlds.rs: server_does_not_read_a_configured_secret | covered | pass |
| Help lists the verbs | crates/mu-cli/tests/cli.rs: help_lists_verbs | covered | pass |
| Help for one verb | crates/mu-cli/tests/cli.rs: help_for_one_verb | covered | pass |
| Two worlds connect | crates/mu-cli/tests/worlds.rs: two_worlds_connect | covered | pass |
| A configured world waits | crates/mu-cli/tests/worlds.rs: configured_world_waits | covered | pass |
| A configured world cannot connect yet | crates/mu-cli/tests/worlds.rs: configured_world_cannot_connect_yet | covered | pass |

### Not run

None.

## Verification runs

- exit 0: `cargo nextest run -p mu-cli --test cli --locked -E 'test(=help_for_one_verb)'`
- exit 0: `cargo nextest run -p mu-cli --test cli --locked -E 'test(=help_lists_verbs)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=character_that_names_a_parent)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=character_with_no_game)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=charset_case)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=charset_the_engine_cannot_read)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=default_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=directory_under_home)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=disable_with_no_target)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=edges_of_each_range)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=empty_mu_config)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=entries_from_each_layer)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=entry_with_no_id)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=file_name_outside_the_id_rule)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=flag_wins_over_the_environment)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=game_with_no_type)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=global_default_reaches_each_world)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=global_preference_is_not_hidden_by_a_type)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=half_a_keepalive)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=id_twice_in_one_layer)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=last_layer_wins)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=layer_disables_an_inherited_rule)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=layer_replaces_an_inherited_rule)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=line_break_in_a_line_field)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=link_to_nothing_is_an_error)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=login_name_with_no_secret)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=login_secret_with_no_name)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=loop_of_types)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=missing_default_directory_is_quiet)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=missing_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=no_home_directory)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=password_in_a_file)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=port_out_of_range)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=relative_xdg_config_home)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=rule_and_script_with_one_id)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=secret_that_is_not_a_reference)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=show_does_not_read_the_secret)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=sources_of_fields)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=stray_file)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=table_merges_field_by_field)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=template_with_no_password)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=type_that_extends_itself)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=unknown_field)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=unknown_type)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=unknown_world)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=user_type_extends_built_in_type)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=user_type_with_a_built_in_name)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=world_that_logs_in_by_hand)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=world_with_no_host_or_port)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=worlds_from_character_files)'`
- exit 0: `cargo nextest run -p mu-cli --test world_config --locked -E 'test(=wrong_type_under_login)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=bad_configuration_stops_the_start)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=configured_world_cannot_connect_yet)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=configured_world_waits)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=fake_world_with_a_configured_id)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=server_does_not_read_a_configured_secret)'`
- exit 0: `cargo nextest run -p mu-cli --test worlds --locked -E 'test(=two_worlds_connect)'`
- exit 0: `cargo nextest run -p mu-core --test world_config --locked -E 'test(=built_in_types_hold_only_quirks)'`
- exit 0: `cargo nextest run -p mu-core --test world_config --locked -E 'test(=plugin_can_set_a_preference)'`
- exit 0: `cargo nextest run -p mu-core --test world_config --locked -E 'test(=plugin_cannot_set_the_connection)'`

## Dropped

None.

## Lens notes

- approach: Follow-up round since 9e9821d. Read each changed file of diff-files.txt except review.md: README.md, mu-cli config.rs, lib.rs, server.rs, world.rs, tests/world_config.rs, mu-core world/error.rs, tests/world_config.rs, and design.md.
- approach: The approach lens has no Bash, so it bounded the round by the changed files, not by changed lines.
- approach: The Context facts of design.md now say they describe the code before the change. Each one that the change does not move still holds at head: Source and Origin at the crate root of mu-core, has_line_break shared by secret.rs and session.rs, the TelnetSettings default of UTF-8 and 120 by 40, and the socket order in socket.rs. The listed moves also hold at head: world/ with is_part pub(crate), PathValue and parse_path in env_path.rs, toml and figment2 in mu-core, and mu_with through test_env.
- approach: The fixed rows R-87 (warning prefix), R-89 (README status), R-90 (stdout write policy) and R-91 (FieldError promise) did not come back at head. The warning goes through crate::warn as `mu <verb>: warning:`, `mu version` and `mu world` share crate::print, README points to the docs site and its first-run guide, and FieldError promises only that it holds no value under login.
- approach: server.rs follows the decision Configured worlds in the server: it resolves first, refuses an id shared by a fake and a configured world, then adds WorldSetup::without_transport. Both happen before the socket opens.
- approach: world.rs follows the decision mu world: it takes the id as text, resolves every world first, and fails with exit 1 and an error that names an unknown id.
- fit: The lens has no Bash tool, so it could not run git diff for the exact changed lines since 9e9821d. It read each file in diff-files.txt whole and put the changed text first in the evidence, judging from the round 9 fixes (R-87, R-89, R-90, R-91) and the commit subjects.
- fit: The round 9 fixes hold: config.rs warns through crate::warn with the `mu <verb>:` prefix (R-87), mu world and mu version both write stdout through crate::print (R-90), and README.md points at the docs site instead of copying the status sentences (R-89).
- fit: ConfigArgs follows the shape of SocketArgs: a pure function of the environment values (dir, like path) and a method that reads the process environment (worlds, like resolve), both on the shared helpers of env_path.rs.
- fit: The mu-cli tests use the shared ConfigDir, BASE_GAME and mu_command of tests/support, so the copies that R-68, R-74 and R-77 named are gone from the changed files.
- fit: built_in_types_hold_only_quirks lists the quirk fields by hand beside the Quirks struct. That repeats a production shape in a test on purpose, so the lens does not report it.
- invariants: I1: print never fails the verb on a BrokenPipe write of stdout, and always fails it with status 1 on any other write error \| enforced at crates/mu-cli/src/lib.rs:33-36 \| boundary test none (inv-1)
- invariants: I2: each warning of mu-cli is one write of the line `mu <verb>: warning: <message>` \| enforced at crates/mu-cli/src/lib.rs:26-29 \| boundary test missing_directory in crates/mu-cli/tests/world_config.rs:196, for the verb world only. The server caller passes the literal "server" at server.rs:62, and no test runs mu server with a missing --config.
- invariants: I3: the text of a FieldError never holds a value under login \| enforced at crates/mu-core/src/world/check.rs:47-60 and 102-110, where each message under login is fixed text \| boundary test password_in_a_file, secret_that_is_not_a_reference and wrong_type_under_login, which check that stderr does not hold the value
- invariants: I4: a built-in type sets only charset, login.template and keepalive, and sets at least one of them \| enforced at Quirks with deny_unknown_fields in crates/mu-core/src/world/layer.rs \| boundary test built_in_types_hold_only_quirks (the new loop at crates/mu-core/tests/world_config.rs:162-174) and quirks_refuse_other_fields
- invariants: I5: the missing-directory warning fires only when --config or MU_CONFIG names the directory, never for the XDG or HOME default \| enforced at crates/mu-cli/src/config.rs:50-51 \| boundary test missing_directory and missing_default_directory_is_quiet
- invariants: Follow-up round since 9e9821d: the changed code text is config.rs, lib.rs, server.rs, world.rs, error.rs and two test files. No changed file is a script, so no probe ran.
- invariants: Messages under login in check_fields and check_login are fixed strings, so the narrowed FieldError doc at error.rs:48 holds for every builder of FieldError at head.
- test-coverage: Test table_written_as_an_array, other_files_are_skipped and keepalive_of_a_built_in_type_resolves have no scenario; prior R-81 and R-83 are accepted, so not raised again.
- test-honesty: No disabled test, no test without an assertion, and no vacuous gate found in the two changed test files.
- test-honesty: Could not diff against the base (no git tool); I read both changed test files whole instead of only the changed lines.
- test-coverage gap: Unit-test bodies in resolve.rs, builtin.rs and dir.rs were not read; their names were matched only. Tests were not run.

## Ledger

| id | first | last | severity | scope | location | disposition | basis | override | claim |
|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The merge of fields uses `figment2`"],"Thus the errors that a user sees come from `toml`, with the file, the field and the line."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | app-1: The design shows the toml error to the user, and the Display of a toml error prints the whole source line, so the value of a password field in a character file reaches stderr.; inv-1: No task strips the source line from TOML errors, and toml_edit prints that line. A probe printed hunter2 for both login.password and a bad login.secret. |
| R-2 | 1 | 1 | blocker | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"Thus the error names that file and that field. The parse checks `login.secret` with"] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The design checks the login reference with the FromStr parse of the reference type, whose error keeps and prints the bad text, so a plain password in that field reaches stderr. |
| R-3 | 1 | 1 | blocker | in-change | ["artifact",["specs/world-config/spec.md","Requirement: A world has a place to connect"],"Without `login.name` and `login.secret`, a `login.template` SHALL NOT count."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | No decision produces the requirement "A world has a place to connect": the required host and port, the login pair, and a template that does not count alone. Only task 3.1 names the check. |
| R-4 | 1 | 1 | blocker | in-change | ["artifact",["design.md","Configured worlds in the server"],"Then it adds a `WorldSetup` with no transport and no password for each configured world."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | No decision refuses an id that is both a fake and a configured world; Portal::start keys worlds by id in a BTreeMap, so a duplicate would silently replace one sender. |
| R-5 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The layer seam and its trust"],"`TomlDirSource` reads the directory."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | No decision says how TomlDirSource maps file names to ids and refuses a name outside the id rule, a character with no game file, or a stray `.toml` file. |
| R-6 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The chain of a world"],"The chain starts at the character and the game. Next comes the type that the game `extends`"] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The chain decision covers only the loop and the built-in name; nothing makes `extends` required in a game, refused in global.toml and a character, or an unknown name an error. |
| R-7 | 1 | 1 | blocker | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"and the character set against the names of `Charset`"] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The design checks the character set against the names of Charset, which are `UTF-8` and `ISO-8859-1`, but the requirement makes tinymux set `iso-8859-1`; an exact check fails the built-in type. |
| R-8 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The merge of fields uses `figment2`"],"The resolver gives `figment2` only typed layers that `toml` parsed, through `Serialized`."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | app-9: The resolver in mu-core parses each file with `toml`, but the plan adds only `figment2` and `serde` to mu-core and to its ALLOWED_CRATES, so the boundary test refuses the parse path.; fit-4: The plan names two TOML readers: figment2 with its TOML feature, and the toml crate that parses each layer before Serialized; task 1.2 adds only the first to mu-core.; app-10: The TOML feature of figment2 serves nothing: the resolver passes figment2 only Serialized providers, so figment2 never reads a file. |
| R-9 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Context"],"The process helper in `crates/mu-cli/tests/support/process.rs` clears `MU_SOCKET` for each `mu` that a test starts."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | Only ServerBuilder clears MU_SOCKET; `mu_with` and `mu_on_pty` start `mu` with the inherited environment, so the cited helper does not cover each `mu` a test starts. |
| R-10 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The configuration directory"],"`mu-cli` gets `ConfigArgs`, with `--config` and the order `--config`, `MU_CONFIG`, `XDG_CONFIG_HOME/mu`, then `HOME/.config/mu`, on Linux and macOS alike."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | ConfigArgs copies the shape of SocketArgs: a second empty-means-unset value parser and a second relative-XDG filter, with no shared helper. |
| R-11 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The resolved world"],"The fake world keeps `with_default_login`, which fills the new fields with the defaults."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The defaults of charset, columns, rows and the login template get two new homes (the resolver's default layer and with_default_login) beside TelnetSettings::default and DEFAULT_LOGIN_TEMPLATE. |
| R-12 | 1 | 2 | should-fix | in-change | ["artifact",["design.md","The layer seam and its trust"],"pub struct Layer { pub key: LayerKey, pub source: Source, pub trust: Trust, pub fields: LayerFields }"] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The new Source type of a layer reuses the name of mu_core::Source, which the crate root already exports as the source of an event. |
| R-13 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Tests do not read the configuration of the developer"],"The process helper sets `MU_CONFIG` to an empty directory of the test for each `mu` that it starts, as it clears `MU_SOCKET`."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The design adds the MU_CONFIG scrub to ServerBuilder alone, while three other test launchers build their own environment and mu client starts a server that inherits it. |
| R-14 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","6. The server and the docs"],"Edit `docs/src/content/docs/decisions/world-type-profiles.md` to name `figment2` and say why."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The decision page calls the window width `width`, while the spec and TelnetSettings call it `columns`; task 6.2 edits the page but only for figment2. |
| R-15 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Each character file makes one world"],"A file name that is not a part of a world id SHALL be an error that names the file."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The file-name rule needs the world-id part check, which id.rs already has as the private is_part, and the design names no reuse. |
| R-16 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","`mu world`"],"`mu world show <id>` prints the resolved world as TOML. Each field ends with a comment `# <source>`"] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | mu world show writes TOML with a comment on each field, and the design names no TOML writer, which leaves hand-written quoting of values such as a login template. |
| R-17 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: A world has a place to connect"],"A world with `login.name` or `login.secret` SHALL have both."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The login pair check is tested with a name and no secret only. A secret with no name, the other side of the rule, has no test. |
| R-18 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: A loop of types"],"- **GIVEN** `types/a.toml` sets `extends = \\"b\\"`, and `types/b.toml` sets `extends = \\"a\\"`"] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The loop test uses two types. A type that extends itself, the loop of length one, has no test. |
| R-19 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: The configuration directory"],"An empty `MU_CONFIG` SHALL count as unset."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The spec says an empty MU_CONFIG counts as unset, and no scenario or test sets MU_CONFIG to the empty string. |
| R-20 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: The configuration directory"],"If that variable is not set, empty or relative, the directory SHALL be `.config/mu` under `HOME`."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The spec says a relative XDG_CONFIG_HOME is ignored. The tests cover only an empty value, not a relative one. |
| R-21 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","4. Rules, scripts and trust"],"Add `plugin_cannot_set_the_host` and `plugin_can_set_a_preference` to `crates/mu-core/tests/world_config.rs`"] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | Only the host is tested against the plugin trust check. port, tls and the nested login.name and login.secret, a different path through a table, are untested. |
| R-22 | 1 | 1 | should-fix | in-change | ["path","crates/mu-core/src/framer.rs",221] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The design checks charset against the names of Charset, which are upper case, while the spec writes iso-8859-1. No test pins the case rule. |
| R-23 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Rules and scripts accumulate"],"The array `disable` of a layer SHALL remove each inherited entry with an id that it names."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | Nothing states whether rules and scripts share one id space, or refuses a repeated id in one layer's array, yet disable and replace both act by id alone. |
| R-24 | 1 | 1 | should-fix | in-change | ["path","crates/mu-core/src/secret.rs",22] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The design parses login.secret with the from_str parse of the secret reference type, whose error Display prints the rejected text, so wrapping that error puts the password in stderr. |
| R-25 | 1 | 1 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",339] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | login.name, login.template and keepalive.command become user input written as a line, and nothing refuses CR or LF in them. Only the password has that check. A probe parsed a name with CR LF. |
| R-26 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: A shared plugin layer cannot set the connection"],"A layer from a shared plugin source SHALL NOT set `host`, `port`, `tls`, `login.name` or `login.secret`."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | A shared plugin may set login.template, and the template decides where the password goes, so a plugin can route the password to another target although it cannot set the secret. |
| R-27 | 1 | 2 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"If one field lacks it, the test `empty_layer_gives_figment_no_keys` fails."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | An empty layer has login and keepalive as None, so empty_layer_gives_figment_no_keys never serializes the nested fields. A missing skip on login.template or keepalive.seconds passes it. |
| R-28 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The resolved world"],"and `keepalive`, an `Option` of a command and a period."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | Nothing checks after the merge that keepalive has both a command and seconds. A layer that sets one half yields a keepalive the resolved type cannot hold. |
| R-29 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"It also checks the ranges of `columns`, `rows`, `port` and `keepalive.seconds`."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | inv-7: The design checks ranges of columns and rows but no artifact names the range. NAWS sends them as u16, so 0 and 65536 are the edges.; inv-8: No artifact names the range of keepalive.seconds, so a zero period is accepted as a keepalive that #54 then has to run. |
| R-30 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: A port out of range"],"- **GIVEN** the game `alpha` sets `port = 70000`"] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The only port test uses 70000, far past the limit. Port 0, 65535 and 65536 are untested, and a u16 accepts 0. |
| R-31 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Rules and scripts accumulate"],"the game has a rule `hl-page` with another pattern"] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The scenario 'A layer replaces an inherited rule' names no pattern for the game's hl-page rule, so its THEN cannot be checked. |
| R-32 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Layers merge from global to character"],"the game `alpha` sets `login.template`, and `characters/alpha/aria.toml` sets only `login.name` and `login.secret`"] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The scenario 'A table merges field by field' gives no values for login.template, login.name or login.secret, so its THEN cannot be checked. |
| R-33 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The directory and the layers"],"In `crates/mu-cli/tests/cli.rs`, make `help_lists_verbs` expect the four verbs."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The scenario 'Help for one verb' is in the delta but no task names it or its test. |
| R-34 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","The layer seam and its trust"],"pub trait LayerSource { fn layers(&self) -> Result<Vec<Layer>, ConfigError>; }"] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | LayerSource::layers takes no input, but the Lua decision lets configuration Lua read its parent layer, so the Lua change must reshape the seam the design says it will not touch. |
| R-35 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","Rules and scripts accumulate in the resolver"],"A replace keeps the place of the old entry."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | The accumulation of rules over any sequence of layers has invariants over an unbounded domain: ids stay unique and order follows first insertion, minus disabled ids. |
| R-36 | 1 | 1 | suggestion | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Show names the source of each value"],"`mu world show <id>` SHALL print the resolved world as TOML."] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | mu world show must print TOML that parses for any string value, including rule tables kept as values with quotes, hash signs and newlines. |
| R-37 | 1 | 3 | should-fix | in-change | ["artifact",["review","lens-status"],"approach partial"] | fixed | d480822aa96f2e98c97aa8da76f641e75423ca0e |  | lens approach reported partial, gaps: 1 |
| R-38 | 2 | 2 | blocker | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"Under `login`, it also drops the message of `toml`, because a message of `serde` can quote the value. In its place, it says that the value is not valid."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The THEN of 'A password in a file' needs login.password in stderr. The design drops the toml message under login, and the toml error has no public key path, so nothing in the design produces that name. |
| R-39 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"A `ConfigError` of a parse keeps the file, the key path, the line and the column of the `toml` error."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | app-1: The design keeps the key path of the toml error, but toml 1.2.1 has no public accessor for it: the keys field is private, and Display prints it only when the error has no span.; inv-1: The design drops the toml message under login by its key path, but toml 1.2.1 keeps the key path private; Error exposes only message() and span(). |
| R-40 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Tests do not read the configuration of the developer"],"One base launcher in `crates/mu-cli/tests/support/` builds the `Command` of each `mu` that a test starts."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The base launcher builds one Command, but the four launchers use three command types: std, tokio and the CommandBuilder of portable_pty, which cannot start from a std Command. |
| R-41 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"`login.name`, `login.template` and `keepalive.command` hold no CR and no LF, because the session writes each one as a line."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | fit-3: The CR and LF check of login.name, login.template and keepalive.command names no helper, while has_line_break in framer.rs is the one check that secret.rs and session.rs already share.; inv-9: The Login doc says the template holds {login} and {password}, and the template is now user input. Nothing refuses a template without {password}, so a typo sends no password. |
| R-42 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","6. The server and the docs"],"to name `figment2` and say why, and to call the window width `columns`, as the spec does."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The decision page states the plugin rule without login.template, which the spec now adds, and task 6.2 edits the page only for figment2 and columns, so the two statements of the rule disagree. |
| R-43 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: Half a keepalive"],"the game `alpha` sets `keepalive.command = \\"idle\\"` and no layer sets `keepalive.seconds`"] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The keepalive pair rule is tested only with a command and no seconds. Seconds with no command, the other side of the rule, has no test. |
| R-44 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: A world has a place to connect"],"After the merge, each world SHALL have `host` and `port`."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | Each world must have a host and a port, but only a missing port is tested. A world with no host has no test. |
| R-45 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"`ConfigError` drops the source snippet, the message under `login`, and the text of `BadSecretRef`. The two tests above check stderr."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | No test passes a value of the wrong type under login. A probe of toml 1.2.1 showed that serde quotes the value in its message, for an integer login.secret and for login set to a string. |
| R-46 | 2 | 3 | should-fix | in-change | ["artifact",["design.md","`mu world`"],"`show` writes each line as `<key> = <value> # <source>`. The `Display` of `toml::Value` writes each value, so the `toml` crate quotes and escapes each string."] | fixed | ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | show writes the keys of a rule or script entry itself, unquoted. A probe printed `two words = "x"` and `a#b = 1`, and neither parsed back as TOML. |
| R-47 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Built-in types set only quirks"],"A built-in type SHALL set only quirk fields: `charset`, `login.template` and `keepalive`."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | No test feeds Quirks a field that is not a quirk; built_in_types_hold_only_quirks reads only valid files, so a Quirks or nested login struct without deny_unknown_fields passes. |
| R-48 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Rules and scripts accumulate"],"Each entry SHALL have an `id`."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The rule that each entry has an id has no test with an entry that lacks id, or has an id that is not a string. |
| R-49 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The configuration directory"],"An empty or relative `XDG_CONFIG_HOME` is ignored, as the XDG Base Directory specification asks."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | Nothing covers an unset, empty or relative HOME. The absolute-path check covers only the XDG base, so an empty HOME yields the relative path .config/mu under the cwd. |
| R-50 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The walk of the directory"],"Any other file that ends in `.toml` is an error that names its path."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The stray-file rule covers every depth, but the only test puts worlds.toml at the root. A .toml file one level deeper in a known directory goes untested. |
| R-51 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: A plugin sets a field of the connection"],"a layer from a shared plugin source named `colors` that sets one of `host`, `port`, `tls`, `login.name`, `login.secret` and `login.template`"] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The GIVEN of the scenario 'A plugin sets a field of the connection' names six fields with 'one of' and gives no value for any of them, so the input of the THEN is not concrete. |
| R-52 | 3 | 3 | blocker | in-change | ["artifact",["design.md","`mu world`"],"`show` resolves all worlds first, so a bad file fails `show` as it fails the server."] | fixed | ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | No decision produces the rule that show of an id that is not a configured world is an error that names the id; the mu world decision covers only the print of a resolved world and a bad file. |
| R-53 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"`ALLOWED_CRATES` in the boundary test, with `figment2`, `serde` and `toml`, and `cargo deny check`."] | fixed | ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | The enforcement row of the crate rule says ALLOWED_CRATES of mu-core gains figment2, serde and toml, but task 1.2 and the proposal add serde_path_to_error as a fourth crate that the parse needs. |
| R-54 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Checked facts"],"With no features, `figment2` 0.11.5 depends only on `serde` and `uncased`."] | fixed | ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | The checked fact says that with no features figment2 0.11.5 depends only on serde and uncased, but its manifest also has the build dependency version_check and the target dependency atomic on 32-bit targets. |
| R-55 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"`charset` against `Charset::name`, with no regard to case. Thus `iso-8859-1` and `ISO-8859-1` are the same name."] | fixed | ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | The parse matches a charset name against Charset::name with no regard to case, which charset_answer in the framer already does inline; the change adds a second copy of that match. |
| R-56 | 3 | 3 | should-fix | in-change | ["artifact",["tasks.md","3. The merge and `mu world show`"],"It renders a resolved world with any strings and parses the output back to the same values."] | fixed | ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | The property test show_output_parses_back generates keys and string values only, so an entry with a nested table or an array of tables, the case that breaks the writer, never reaches it. |
| R-57 | 3 | 4 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"The tests `loop_of_types` and `type_that_extends_itself`."] | fixed | 06cbadb07a43996a0f60b3f15e910fe730138b5c |  | The loop check runs only on the chain of a world, which starts at a character and a game. The scenario A loop of types has no game, so types a and b are never walked and the error the scenario requires never fires; an unknown extends in an unused type passes too. |
| R-58 | 3 | 4 | should-fix | in-change | ["artifact",["design.md","Built-in types and the quirk schema"],"The resolver reads them through a separate `Quirks` struct, which holds only `charset`, `login.template` and `keepalive`."] | fixed | 06cbadb07a43996a0f60b3f15e910fe730138b5c |  | The value checks ({password} in login.template, no CR or LF in the template and keepalive.command, the range of keepalive.seconds, the charset name) belong to the parse of LayerFile. Built-in types parse through the separate Quirks struct, and no decision or test runs those checks on it. |
| R-59 | 3 | 3 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: A world with no host or port"],"- **GIVEN** no layer of `alpha/aria` sets `host`, or no layer sets `port`"] | fixed | ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | The check after the merge tests only that host is set. host = "" passes it and the parse, and no test passes the empty host, so a world with no place to connect resolves. |
| R-60 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"No id repeats in one array of one layer."] | fixed | ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | The parse refuses a repeat only within one array, and the only cross-kind test puts the rule and the script in two layers. A rule and a script with one id in the same layer is not inherited, and no test passes it. |
| R-61 | 4 | 4 | blocker | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"`charset` with `Charset::from_name`, a new function of `framer.rs` that matches a name with no regard to case."] | fixed | 06cbadb07a43996a0f60b3f15e910fe730138b5c |  | app-1: The THEN of 'A user type extends a built-in type' needs show to print charset = "iso-8859-1". The resolved charset is a Charset whose only name is ISO-8859-1, and the design keeps no written text.; fit-1: The parse checks charset with Charset::from_name but the design does not say that the parse keeps the Charset or that ResolvedWorld.charset is a Charset. If the resolved world keeps the text, world-sessions (#54) must call from_name a second time to fill TelnetSettings.charset. If it keeps a Charset, show prints Charset::name(), which is ISO-8859-1, while the spec expects the lower-case text of the built-in file. |
| R-62 | 4 | 4 | blocker | in-change | ["artifact",["design.md","`mu world`"],"`show` writes each field of the world as `<key> = <value> # <source>`. The keys of these fields are fixed names"] | fixed | 06cbadb07a43996a0f60b3f15e910fe730138b5c |  | The THEN of 'A table merges field by field' needs the three login lines under a login table. The show decision writes each field as one key line and creates no table header and no order of tables after top-level keys. |
| R-63 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","The chain of a world"],"Before it builds any chain, it also walks from each file in `types/`, so a type that no game uses is checked too."] | fixed | 06cbadb07a43996a0f60b3f15e910fe730138b5c |  | The new walk starts from each file in types/ only, so a game with no character file whose extends names an unknown type is never walked, and the spec's unknown-name error never fires. |
| R-64 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"`charset_answer` of the framer moves to it, so the match has one copy."] | fixed | 06cbadb07a43996a0f60b3f15e910fe730138b5c |  | charset_answer moves onto Charset::from_name, a text match, but the server list is raw bytes. No test passes a non-UTF-8 name before a valid one, so a refactor that rejects the whole list passes. |
| R-65 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","`mu world`"],"An id that is not a configured world is an error that names the id, and the exit status is 1."] | fixed | 06cbadb07a43996a0f60b3f15e910fe730138b5c |  | show must exit 1 for an id that is not a configured world. The only test uses a well-formed id, and a malformed id such as Alpha/x would stop in a clap value parser with exit 2, as --fake-world does. |
| R-66 | 4 | 4 | suggestion | in-change | ["artifact",["design.md","Built-in types and the quirk schema"],"The resolver turns each `Quirks` into a `LayerFields` and runs the value checks of the parse on it"] | fixed | 06cbadb07a43996a0f60b3f15e910fe730138b5c |  | The value checks run in two places: the TOML parse of each file, and the resolver for each built-in Quirks. A Layer from any other LayerSource, such as the test plugin layer now and the Lua source that the proposal names next, reaches the merge with no value check, so the Lua change must add a third call site or move the checks into the resolver. |
| R-67 | 4 | 4 | should-fix | in-change | ["artifact",["review","lens-status"],"spec-lint partial"] | fixed | 06cbadb07a43996a0f60b3f15e910fe730138b5c |  | lens spec-lint reported partial, gaps: 1 |
| R-68 | 5 | 5 | blocker | in-change | ["path","crates/mu-cli/tests/world_config.rs",123] | fixed | 57cef27f7b261cbfef5341cc5ee447fdfbb0f465 |  | The loop that sets or removes each (name, Option<value>) on a std Command gains two more copies (apply_test_env, list_in) beside the ones in mu_with and ServerBuilder. list_in is mu_with plus a current_dir. |
| R-69 | 5 | 5 | should-fix | in-change | ["artifact",["design.md","Checked facts"],"`Cargo.lock` holds `serde`, `atomic` and `version_check`, but not `uncased`."] | fixed | d33446bd67c43adbcf24a48276650f17685c5045 |  | The checked fact says Cargo.lock does not hold uncased, but at head Cargo.lock holds uncased 0.9.10 as a dependency of figment2. |
| R-70 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/world/dir.rs",114] | fixed | d33446bd67c43adbcf24a48276650f17685c5045 |  | app-2: The walk matches the extension with no regard to case, so a file such as notes.TOML is read as a configuration path and fails as a stray file, while the design says such a file is not read.; inv-3: The walk picks files by Path::extension, which is None for a file named exactly .toml, so games/.toml or characters/alpha/.toml is skipped in silence instead of failing as a stray file. A probe printed .toml ext=None. |
| R-71 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/world/dir.rs",133] | fixed | d33446bd67c43adbcf24a48276650f17685c5045 |  | The walk of the directory writes the rule of an id part out twice more as text, beside the text of BadWorldId, so a change of is_part leaves three messages to edit. |
| R-72 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/world/dir.rs",174] | fixed | d33446bd67c43adbcf24a48276650f17685c5045 |  | The check that a character has its game file runs twice with the same message: check_games in TomlDirSource, and Index::chain in the resolver, which covers every LayerSource. |
| R-73 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/world/resolve.rs",22] | fixed | 20a7c800a89338b26c7d601379371bf5755a2e0f |  | FIELD_PATHS says it holds the fields in the order of mu world show, but render writes its own list of the same paths and order. A field added to one list is silently missing from the other. |
| R-74 | 5 | 5 | should-fix | in-change | ["path","crates/mu-cli/tests/world_config.rs",133] | fixed | 57cef27f7b261cbfef5341cc5ee447fdfbb0f465 |  | Four new helpers write the base directory of the spec: Config::base, base_at and beta_at in world_config.rs, and configuration in worlds.rs. None of them is in tests/support. |
| R-75 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/world/parse.rs",26] | fixed | d33446bd67c43adbcf24a48276650f17685c5045 |  | The redaction of a parse error under login matches only the path login or login.<x>. A login written as an array gives the path login[N], and the serde message keeps an integer value. A probe with toml 1.2.1 and serde_path_to_error 0.1.20 printed: login[0]: invalid type: integer `123456`, expected a string. |
| R-76 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/world/layer.rs",164] | fixed | d33446bd67c43adbcf24a48276650f17685c5045 |  | A table field given as an array is not refused. The derived Deserialize of LoginFields accepts a sequence by field order, so login = ["connect {login} {password}", "Aria", "env:ARIA_PW"] parses as a full login, and a fourth element is dropped with no error. A probe with toml 1.2.1 returned ok for both, and for keepalive = ["idle", 60] and keepalive = ["idle", 60, 7]. |
| R-77 | 6 | 6 | should-fix | in-change | ["path","crates/mu-cli/tests/support/mod.rs",54] | fixed | e99f0369e6f0280a2208b72739b85fec052bb725 |  | mu_command is now the base launcher of a std Command with the test environment, but mu_on_pty and non_utf8_verb_is_usage_error still build the same Command::new plus apply_test_env by hand instead of calling it. |
| R-78 | 6 | 6 | should-fix | in-change | ["path","crates/mu-core/src/world/layer.rs",203] | fixed | e99f0369e6f0280a2208b72739b85fec052bb725 |  | The new table guard on the login and keepalive fields of Quirks has no test that passes an array, so removing it from Quirks passes every test. |
| R-79 | 6 | 6 | should-fix | in-change | ["path","crates/mu-core/src/world/dir.rs",108] | fixed | e99f0369e6f0280a2208b72739b85fec052bb725 |  | The changed name predicate of the walk has no test for the other side: a near-miss name such as alpha.toml.bak or README.md beside the base is never shown to be skipped, so a predicate that reads every file passes. |
| R-80 | 6 | 6 | suggestion | deviates-design | ["artifact",["design.md","The walk of the directory"],"The resolver makes this check, so it holds for each source of layers."] | accepted | ticket: https://github.com/seanb4t/mu-portal/issues/85; reason: Only TomlDirSource checks which layer kinds may set extends: a game must name a type, and global, character and plugin layers must not. The resolver does not check it, so a later layer source, such as the Lua source, can give a game with no type or a character with extends, and nothing refuses it. |  | The game-file check moved to the resolver so that it holds for each source, but the check of extends by layer kind stays in TomlDirSource. A Lua source, which the proposal names next, gets a game with no type, or extends on a character or plugin layer, ignored in silence. |
| R-81 | 6 | 6 | note | in-change | ["path","crates/mu-cli/tests/world_config.rs",391] | accepted | ticket: none; reason: The wrong-type requirement of the spec already covers a table written as an array, and the test names the case; a scenario of its own adds no new behavior. |  | table_written_as_an_array has no scenario and no task; the spec states the wrong-type rule but gives no scenario for a table written as an array. |
| R-82 | 7 | 7 | should-fix | in-change | ["path","crates/mu-core/tests/world_config.rs",170] | fixed | ddaa5afc330334edb7cb9968a167036b142bd4e1 |  | The new test rows refuse login and keepalive written as arrays, which only the table guard of layer.rs produces. design.md names no such guard and says the typed struct alone fails a bad type, which the derived Deserialize does not do for an array. |
| R-83 | 7 | 7 | note | in-change | ["path","crates/mu-cli/tests/world_config.rs",253] | accepted | ticket: none; reason: The test covers the skip side of the file-name rule of the design; a scenario of its own adds no new behavior, as with R-81. |  | other_files_are_skipped has no scenario and no task that names it. |
| R-84 | 7 | 9 | should-fix | in-change | ["artifact",["review","not-run"],"Each built-in type reads as quirks"] | refuted | The row names two tests, built_in_types_hold_only_quirks and quirks_refuse_other_fields; round 9 ran each one alone at the head and both passed, so the row is verified though no one command holds both names. |  | covered row 'Each built-in type reads as quirks' has no verification run |
| R-85 | 7 | 9 | should-fix | in-change | ["artifact",["review","not-run"],"A port out of range"] | refuted | The row names two tests, port_out_of_range and edges_of_each_range; round 9 ran each one alone at the head and both passed, so the row is verified though no one command holds both names. |  | covered row 'A port out of range' has no verification run |
| R-86 | 7 | 7 | should-fix | in-change | ["artifact",["review","lens-status"],"test-honesty partial"] | accepted | ticket: none; reason: The lens had no diff tool. The only hunk it did not read is the switch of cli.rs to mu_command, which the fit and invariants lenses checked in this round. |  | lens test-honesty reported partial, gaps: 1 |
| R-87 | 9 | 9 | should-fix | in-change | ["path","crates/mu-cli/src/config.rs",57] | fixed | 15f3f9c4b769faac2a53bc076fd398fde0ed9132 |  | The missing-directory warning writes its own stderr line with the prefix `mu:`, while every other line of mu-cli goes through crate::fail as `mu <verb>:`, and the design names no reason. |
| R-88 | 9 | 9 | should-fix | in-change | ["path","crates/mu-cli/src/config.rs",52] | accepted | ticket: none; reason: TomlDirSource documents a missing root as an empty configuration, which the server needs; the CLI check only names a directory that the user chose, so the two checks answer different questions. |  | Two mechanisms decide that the configuration directory is missing: try_exists in ConfigArgs::worlds for the warning, and the NotFound branch of fs::metadata in TomlDirSource::layers for the empty result. |
| R-89 | 9 | 9 | should-fix | in-change | ["path","README.md",7] | fixed | ae74e12107949e339c1da2fee2b929154a0eed33 |  | The change writes the same two status sentences about mu world and the configured worlds into both README.md and the docs index, so the status paragraph keeps two copies to edit at each step. |
| R-90 | 9 | 9 | should-fix | in-change | ["path","crates/mu-cli/src/world.rs",63] | fixed | 15f3f9c4b769faac2a53bc076fd398fde0ed9132 |  | Two verbs write stdout under two policies: mu world fails on every write error but a broken pipe, while mu version drops every write error. A failed print has two meanings in one binary. |
| R-91 | 9 | 9 | should-fix | in-change | ["path","crates/mu-core/src/world/error.rs",48] | fixed | aee5d3543f665a94b13df888e04019d66a682bad |  | The new public FieldError states that its text never holds the value, but check_ids builds a FieldError whose message holds the value of rules[N].id, as the spec requires. |
| R-92 | 10 | 10 | should-fix | in-change | ["path","crates/mu-cli/src/lib.rs",28] |  |  |  | The new warn in lib.rs copies fail line for line: the same one-write stderr call, the same format prefix and the same comment on a closed stderr. Its doc even says it writes as fail does, so a change to the policy of fail must be made twice. |
| R-93 | 10 | 10 | should-fix | in-change | ["path","crates/mu-cli/src/lib.rs",35] |  |  |  | The new print helper splits stdout write errors in two: BrokenPipe returns success, and any other kind fails the verb with status 1. No test passes either side, so a helper that swallows all errors or fails on BrokenPipe still passes every test. |

## Reviewed files

| path | content |
|---|---|
| Cargo.lock | 85dea26426bf3179a36b7f0680d5b9e2f4b480da |
| Cargo.toml | dbddeb394fcd583bec10dd4dea0887e09b0dc91c |
| README.md | 73ac090abc1755f04b8c0477ea43a7bbca2789b0 |
| crates/mu-cli/Cargo.toml | 14c5317c20fd2a10e7d1bdacfcc727c1a0177f45 |
| crates/mu-cli/src/config.rs | 3bcdbe020f1c11b0e2c55a08b0e3754fb7fc2b03 |
| crates/mu-cli/src/env_path.rs | ce12a6e2c9b2272d34c3583dedd0b64db74963cb |
| crates/mu-cli/src/lib.rs | ca891976e4264930c147d7e93c8e678a6751e985 |
| crates/mu-cli/src/server.rs | a90fbe9da0341bc0ecb8efe97646a20e69259317 |
| crates/mu-cli/src/socket.rs | a47a169ecaddfc326fd182ec5f2d8e0ccf087a4d |
| crates/mu-cli/src/world.rs | 936e3e584ced8f94f33f88afd79c1723eb6a4797 |
| crates/mu-cli/tests/cli.rs | 56f0e1ebea1a711bacb5d6692a49de9bf2166e5d |
| crates/mu-cli/tests/support/mod.rs | 99c40b503b007706540b5798d95a764df7a53041 |
| crates/mu-cli/tests/support/process.rs | 3f9aa433437518eed9a345c1449fd84c602ad6a0 |
| crates/mu-cli/tests/workspace_boundary.rs | 1440c238a0611f6c7e676ef0bfaf6231f13458df |
| crates/mu-cli/tests/world_config.rs | 0b1e674b0954318b09d9e8fb1872e39ef402e7c3 |
| crates/mu-cli/tests/worlds.rs | e5be3d173d2668cbcd5158b482138a1388214d51 |
| crates/mu-core/Cargo.toml | 8f781dfd5083f9a5721fe2aec7171959e126a41d |
| crates/mu-core/src/error.rs | 7cb76b78df8ebf5e14ec0a7d28fd2deaa7c9dc2c |
| crates/mu-core/src/framer.rs | 7fa4d779ba082dae3ce09b00da9585215c562036 |
| crates/mu-core/src/id.rs | f1a855b11cc03efe28d5bf178e8aeb1b962d066b |
| crates/mu-core/src/lib.rs | 0a356dc85058cef4ff11b4239a926a937b4bc17f |
| crates/mu-core/src/portal.rs | dab323ae37f095b4fbb549cd2c5cb881566b19d6 |
| crates/mu-core/src/session.rs | 0bbdfbe2e9a87ba5913cd11573da5b5ccc57b20e |
| crates/mu-core/src/world.rs | absent |
| crates/mu-core/src/world/builtin.rs | 5cea16b4b29b3589840e1f9d9a0f1a11c2ed67ae |
| crates/mu-core/src/world/check.rs | e462d0feab84c5d1b1e2877fa5948ce7e6adac91 |
| crates/mu-core/src/world/dir.rs | 20287c6ee40525b36361ecd0b45cf55b4ab58556 |
| crates/mu-core/src/world/error.rs | cc67258cca04cce3c18fe345353c0ce0997fd38a |
| crates/mu-core/src/world/layer.rs | 09bc4403435c121f6caa154412c809e72a0b7c8a |
| crates/mu-core/src/world/mod.rs | 99a3e35c84368ddece94398969325d1e66eb6584 |
| crates/mu-core/src/world/parse.rs | 9d4adb82ff509e32b388b5dd0228e8dec189f58c |
| crates/mu-core/src/world/resolve.rs | 0fe25c9407d995ef131ff0cf408b2c137a164351 |
| crates/mu-core/src/world/types/aresmush.toml | 666a06fe695a675f55eb1ddcd3c4d1fb2ff894d7 |
| crates/mu-core/src/world/types/evennia.toml | 9027f548149e9e29dae6ca340075d30a333ffce9 |
| crates/mu-core/src/world/types/tinymux.toml | 666d779f084ff6b3fdc5d57daa1bb80353a4f8e9 |
| crates/mu-core/tests/portal.rs | 5abc80d5c5be261a7b0659037dddee526d746e1f |
| crates/mu-core/tests/support/mod.rs | 058e7142a316a29729ed95f3c72d767a39b5fd24 |
| crates/mu-core/tests/telnet_framer.rs | f923c53e925f4630a98a3e3196459f3e6c20f8f6 |
| crates/mu-core/tests/world_config.rs | 53a91f0e887d8d79fa87fa21e53306793e0021d0 |
| crates/mu-server/src/convert.rs | eede0280e35635c3cec66cb24f2aae5a4be79049 |
| docs/src/content/docs/decisions/world-type-profiles.md | 3ff4875d18199e9f48f62dd4f890237ccfc1b088 |
| docs/src/content/docs/guides/configuration.md | 20d43133f96d99877bd99d7faca91925560b8ad3 |
| docs/src/content/docs/guides/first-run.md | 093ff92cc5251cbdedaa49094702e005f8ec4beb |
| docs/src/content/docs/index.md | 1f193b4c1e37c4d37172956aa3252753cfc49029 |
| openspec/changes/world-config/.openspec.yaml | 578c89ac91eda0589930783c384ddbe6201a0568 |
| openspec/changes/world-config/design.md | b8182a7d44ec81cff109e877b2622cf1d968a680 |
| openspec/changes/world-config/proposal.md | 55326dcea67d9aa8b7159b77ec0735bb02b94c94 |
| openspec/changes/world-config/specs/cli-entrypoint/spec.md | 04cc2235ead3806c9548bfec33afa5c9df5a8deb |
| openspec/changes/world-config/specs/world-config/spec.md | 6db8a1f9101e6a234709cf61f98a9e04ab194629 |
| openspec/changes/world-config/specs/world-sessions/spec.md | efc5afbb2c359e8e0a1c7257b15fc38bed9a7319 |
| openspec/changes/world-config/tasks.md | sha256:9aa20aaaffbbf71dfc6046b8c155b55ccf766a9c5db32e8291dc1f309dd2cd01 |

## Plan review (2026-11-17)

Verdict changes-requested at head 370a2621585b22b9a902ca04813279ce01fe8c00.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-61 | approach,fit | blocker | in-change | app-1: The THEN of 'A user type extends a built-in type' needs show to print charset = "iso-8859-1". The resolved charset is a Charset whose only name is ISO-8859-1, and the design keeps no written text.; fit-1: The parse checks charset with Charset::from_name but the design does not say that the parse keeps the Charset or that ResolvedWorld.charset is a Charset. If the resolved world keeps the text, world-sessions (#54) must call from_name a second time to fill TelnetSettings.charset. If it keeps a Charset, show prints Charset::name(), which is ISO-8859-1, while the spec expects the lower-case text of the built-in file. | design.md > Layers are parsed alone, then merged — "`charset` with `Charset::from_name`, a new function of `framer.rs` that matches a name with no regard to case." | app-1: State in 'The resolved world' and '`mu world`' how show writes the charset (for example a lower-case name), or change the THEN to the name that Charset::name gives.; fit-1: State in 'Layers are parsed alone, then merged' that the parse yields a Charset through from_name, and in 'The resolved world' that charset is a Charset and columns and rows are u16, as TelnetSettings holds them. Then say what show prints for charset (Charset::name or the text of the layer) and align the scenario with it. |  | severity? should-fix 0.85 |
| R-62 | approach | blocker | in-change | The THEN of 'A table merges field by field' needs the three login lines under a login table. The show decision writes each field as one key line and creates no table header and no order of tables after top-level keys. | design.md > `mu world` — "`show` writes each field of the world as `<key> = <value> # <source>`. The keys of these fields are fixed names" | Say in '`mu world`' that show writes the top-level fields first, then a [login] and a [keepalive] header with their fields, then the rule and script documents. |  | scope? deviates-design 0.68 |
| R-57 | invariants | should-fix | in-change | The walk from each type now enforces the unknown-name rule for a type no game uses, but the only unknown-name test puts the bad name in a game, so an unused type with an unknown extends is never tested. | design.md > The chain of a world — "A name in `extends` that is neither a built-in type nor a file in `types/` is an error that names the file and the name." | Make unknown_type a table test over two cases: the game alpha extends nosuch, and types/x.toml extends nosuch while no game extends x. Both expect stderr to name the file and nosuch. | regression |  |
| R-58 | invariants | should-fix | in-change | Value checks on the Quirks path are tested only with the three valid shipped files. No test passes a bad quirk value, so a converter that drops a field or skips a check still passes. | design.md > Built-in types and the quirk schema — "The resolver turns each `Quirks` into a `LayerFields` and runs the value checks of the parse on it: the character set, t" | In task 5.1, add a table test that loads quirk text through the built-in loader. Use charset = "koi8-r", a template with no {password}, keepalive.seconds = 0, and a keepalive.command with an LF. Each case must fail and name the field. | regression |  |
| R-63 | invariants | should-fix | in-change | The new walk starts from each file in types/ only, so a game with no character file whose extends names an unknown type is never walked, and the spec's unknown-name error never fires. | design.md > The chain of a world — "Before it builds any chain, it also walks from each file in `types/`, so a type that no game uses is checked too." | In 'The chain of a world', also walk from each file in games/ before any chain is built. Add a case to unknown_type where games/beta.toml sets extends = "nosuch" and has no character file. |  |  |
| R-64 | invariants | should-fix | in-change | charset_answer moves onto Charset::from_name, a text match, but the server list is raw bytes. No test passes a non-UTF-8 name before a valid one, so a refactor that rejects the whole list passes. | design.md > Layers are parsed alone, then merged — "`charset_answer` of the framer moves to it, so the match has one copy." | In task 2.1, state whether from_name takes bytes or text. Add a framer test where the list ;\\xff;utf-8 expects ACCEPTED utf-8. |  |  |
| R-65 | invariants | should-fix | in-change | show must exit 1 for an id that is not a configured world. The only test uses a well-formed id, and a malformed id such as Alpha/x would stop in a clap value parser with exit 2, as --fake-world does. | design.md > `mu world` — "An id that is not a configured world is an error that names the id, and the exit status is 1." | State in the design whether a malformed id is a usage error (exit 2) or an unknown world (exit 1). Make unknown_world a table test over alpha/nobody and a malformed id. |  |  |
| R-67 | spec-lint | should-fix | in-change | lens spec-lint reported partial, gaps: 1 | review > lens-status — "spec-lint partial" | rerun the lens or accept the gap in the review |  |  |
| R-66 | fit | suggestion | in-change | The value checks run in two places: the TOML parse of each file, and the resolver for each built-in Quirks. A Layer from any other LayerSource, such as the test plugin layer now and the Lua source that the proposal names next, reaches the merge with no value check, so the Lua change must add a third call site or move the checks into the resolver. | design.md > Built-in types and the quirk schema — "The resolver turns each `Quirks` into a `LayerFields` and runs the value checks of the parse on it" | Run the value checks once in the resolver, on the LayerFields of every Layer from every LayerSource, beside the trust check, and name the file through its LayerOrigin. The TOML parse then keeps only the serde errors, and the Quirks path, the plugin layer and a later Lua source share one call site. |  |  |

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| The flag wins over the environment | 2.3 | covered |  |
| The default directory | 2.3 | covered |  |
| The directory under the home directory | 2.3 | covered |  |
| An empty MU_CONFIG | 2.3 | covered |  |
| A relative XDG_CONFIG_HOME | 2.3 | covered |  |
| No home directory | 2.3 | covered |  |
| A missing directory | 2.3 | covered |  |
| Worlds from character files | 2.3 | covered |  |
| A character with no game | 2.3 | covered |  |
| A file name outside the id rule | 2.3 | covered |  |
| A stray file | 2.3 | covered |  |
| The last layer wins | 3.2 | covered |  |
| A global default reaches each world | 3.2 | covered |  |
| A table merges field by field | 3.2 | covered |  |
| A user type extends a built-in type | 3.2 | covered |  |
| An unknown type | 3.2 | covered |  |
| A game with no type | 3.2 | covered |  |
| A loop of types | 3.2 | covered |  |
| A type that extends itself | 3.2 | covered |  |
| A character that names a parent | 3.2 | covered |  |
| Entries from each layer | 4.1 | covered |  |
| A layer disables an inherited rule | 4.1 | covered |  |
| A layer replaces an inherited rule | 4.1 | covered |  |
| An entry with no id | 4.1 | covered |  |
| A disable with no target | 4.1 | covered |  |
| An id twice in one layer | 4.1 | covered |  |
| A rule and a script with one id | 4.1 | covered |  |
| Each built-in type reads as quirks | 5.1 | covered |  |
| A global preference is not hidden by a type | 5.1 | covered |  |
| A user type with a built-in name | 5.1 | covered |  |
| An unknown field | 2.3 | covered |  |
| A port out of range | 2.3 | covered |  |
| A template with no password | 2.3 | covered |  |
| The case of a character set | 2.3 | covered |  |
| A character set that the engine cannot read | 2.3 | covered |  |
| The edges of each range | 2.3 | covered |  |
| A line break in a line field | 2.3 | covered |  |
| A password in a file | 2.3 | covered |  |
| A secret that is not a reference | 2.3 | covered |  |
| A value of the wrong type under login | 2.3 | covered |  |
| Show does not read the secret | 3.2 | covered |  |
| A plugin sets a field of the connection | 4.2 | covered |  |
| A plugin sets a preference | 4.2 | covered |  |
| A world with no host or port | 3.2 | covered |  |
| A login name with no secret | 3.2 | covered |  |
| A login secret with no name | 3.2 | covered |  |
| Half a keepalive | 3.2 | covered |  |
| A world that logs in by hand | 3.2 | covered |  |
| Sources of fields | 3.2 | covered |  |
| An unknown world | 3.2 | covered |  |
| A bad configuration stops the start | 6.1 | covered |  |
| A fake world with a configured id | 6.1 | covered |  |
| The secret is not read at the start | 6.1 | covered |  |
| Help lists the verbs | 2.3 | covered |  |
| Help for one verb | 2.3 | covered |  |
| Two worlds connect | 6.1 | covered |  |
| A configured world waits | 6.1 | covered |  |
| A configured world cannot connect yet | 6.1 | covered |  |

### Not run

Plan phase: no verification runs.
