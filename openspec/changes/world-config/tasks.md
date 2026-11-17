# Tasks

## 1. Sources and dependencies

- [x] 1.1 Read the sources of each fact that the design uses:
  - the XDG Base Directory specification, on a relative `XDG_CONFIG_HOME`
  - the documentation or the source of TinyMUX, AresMUSH and Evennia: the login command, and a command that the server accepts with no effect
  - the `>` lines of the transcripts in `crates/mu-core/tests/fixtures/telnet/`

  Fill the table "Checked facts" of the design with one row for each fact and its source. Make sure that each difference from the sources is an edit of the design that its row names.
- [x] 1.2 Add `figment2` with `default-features = false`, and `serde`, to `[workspace.dependencies]` in `Cargo.toml`. Add `serde_path_to_error` to `[workspace.dependencies]` too. Add the three, and `toml`, to the dependencies of `mu-core`. Add all four to the list of `mu-core` in `ALLOWED_CRATES` of `crates/mu-cli/tests/workspace_boundary.rs`. Make sure that `cargo deny check` and `cargo nextest run -p mu-cli --test workspace_boundary --locked` pass, and that Renovate reads both new pins from `Cargo.toml`. Make sure that `Cargo.lock` gains only `figment2`, `uncased` and `serde_path_to_error`.

## 2. The directory and the layers

Load the `rust-best-practices` skill before this group.

- [x] 2.1 Make `world.rs` the module `world/` of `mu-core`. Add the types of the design:
  - `Layer`, `LayerKey`, `Trust`, `LayerOrigin` and `LayerSource`
  - `LayerFields` and `LayerFile`
  - `Quirks`, `WorldDefaults` and `ConfigError`

  Do the checks of the decision "Layers are parsed alone, then merged". Run the parse through `serde_path_to_error`, and build `ConfigError` from the file, the path and the line and column of the span. Never keep the `Display` of a `toml` error, and drop the message under `login`. Map `BadSecretRef` to a `ConfigError` that drops its text. Add `Charset::from_name` to `framer.rs`, and make `charset_answer` use it. Add a framer test with a server list that holds a name that is not UTF-8 before `UTF-8`, and expect the answer `ACCEPTED UTF-8`. Make `is_part` `pub(crate)`, and add `TomlDirSource`, which walks the directory as the design describes. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [x] 2.2 Move the empty-value parser and the XDG lookup of `crates/mu-cli/src/socket.rs` to `crates/mu-cli/src/env_path.rs`, and make `SocketArgs` use them. Add the shared list of changes to the environment to `crates/mu-cli/tests/support/`. Make `ServerBuilder`, `PtyClient`, `mu_on_pty`, `mu_with` and `non_utf8_verb_is_usage_error` apply it. Make sure that `cargo nextest run -p mu-cli --locked` passes with no change to a test.
- [ ] 2.3 Add `ConfigArgs` to `mu-cli` on the helpers of `env_path.rs`, as the requirement "The configuration directory" states. Add the verb `world` with `mu world list`. Make the shared list set `MU_CONFIG` to an empty directory of the test, and add the test `helper_points_mu_config_at_an_empty_directory`. Add `crates/mu-cli/tests/world_config.rs` with a helper that writes the base directory of the spec, and with these tests:
  - `flag_wins_over_the_environment`, `default_directory`, `directory_under_home`, `empty_mu_config`, `relative_xdg_config_home`, `no_home_directory` and `missing_directory`, for the seven scenarios of the configuration directory
  - `worlds_from_character_files`, `character_with_no_game`, `file_name_outside_the_id_rule` and `stray_file`, for the four scenarios of the character files. Make `stray_file` a table test over the four paths of its scenario
  - `unknown_field`, `port_out_of_range`, `charset_case` and `charset_the_engine_cannot_read`, for the four scenarios of "Each field is known"
  - `edges_of_each_range`, `line_break_in_a_line_field` and `template_with_no_password`, for the three scenarios. The first two are table tests over each field and value of "Each value is in its range"
  - `password_in_a_file`, `secret_that_is_not_a_reference` and `wrong_type_under_login`, for the scenarios of the same names

  In `crates/mu-cli/tests/cli.rs`, make `help_lists_verbs` expect the four verbs, and make sure that `help_for_one_verb` passes with the verb `world`. Make sure that `cargo nextest run -p mu-cli --locked` passes.

## 3. The merge and `mu world show`

- [ ] 3.1 Add the chain of each world, as the design describes. Merge the fields with `figment2`, and keep the source of each field. Grow `ResolvedWorld` and `Login` with the new fields and `sources`. Make the default layer and `with_default_login` read `WorldDefaults`. Add the check after the merge, as the design describes. Add `crates/mu-core/tests/world_config.rs` with `each_field_has_a_source`. Add the unit tests `empty_layer_gives_figment_no_keys` and `each_nested_field_alone_gives_one_key`, a table test over each field of `login` and `keepalive`. Remove `skip_serializing_if` from one field of `LayerFields` and from one nested field, and make sure that a test fails each time. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 3.2 Add `mu world show <id>`, which prints TOML with a source comment on each field, as the design describes. Add `toml` to the normal dependencies of `mu-cli`. Add the property test `show_output_parses_back` to `mu-cli`. It renders a resolved world with any strings and parses the output back to the same values. It generates the keys of entries from any string, with spaces, `#`, `=` and `.`. It also generates values that are nested tables and arrays of tables. Add these tests to `crates/mu-cli/tests/world_config.rs`:
  - `last_layer_wins`, `global_default_reaches_each_world` and `table_merges_field_by_field`, for the three scenarios of the merge
  - `user_type_extends_built_in_type`, `unknown_type`, `game_with_no_type`, `loop_of_types`, `type_that_extends_itself` and `character_that_names_a_parent`, for the six scenarios of `extends`. In `loop_of_types`, no game extends `a` or `b`. Make `unknown_type` a table test over a game and a type that no game uses
  - `world_with_no_host_or_port`, `login_name_with_no_secret`, `login_secret_with_no_name`, `half_a_keepalive` and `world_that_logs_in_by_hand`, for the five scenarios of "A world has a place to connect". Make `world_with_no_host_or_port` a table test over no host, an empty host and no port
  - `sources_of_fields` and `unknown_world`, for the two scenarios of `show`. Make `unknown_world` a table test over `alpha/nobody` and `Alpha/x`
  - `show_does_not_read_the_secret`

  Make sure that `cargo nextest run -p mu-cli --test world_config --locked` passes.

## 4. Rules, scripts and trust

- [ ] 4.1 Add the accumulation of `rules` and `scripts`, as the design describes. A `disable` removes an entry, and an entry with a known id replaces it. Rules and scripts share one space of ids. Print the entries in `mu world show`. Add `entries_from_each_layer`, `layer_disables_an_inherited_rule`, `layer_replaces_an_inherited_rule` and `disable_with_no_target` to `crates/mu-cli/tests/world_config.rs`, for the four scenarios of "Rules and scripts accumulate". Add `entry_with_no_id` for the scenario of that name. Add `id_twice_in_one_layer` and `rule_and_script_with_one_id`, for the two scenarios of "Rules and scripts have unique ids". Make `rule_and_script_with_one_id` a table test over two layers and one layer. Add the property test `accumulation_matches_a_model` to `mu-core`, after the pattern of the property tests in `crates/mu-core/src/id.rs`. Make sure that `cargo nextest run -p mu-core --locked` passes too. Make sure that `cargo nextest run -p mu-cli --test world_config --locked` passes.
- [ ] 4.2 Add the trust check for a layer with `Trust::SharedPlugin`. Add the table test `plugin_cannot_set_the_connection` over the six fields. Add it and `plugin_can_set_a_preference` to `crates/mu-core/tests/world_config.rs`, with a test `LayerSource` that gives a plugin layer named `colors`. Make sure that `cargo nextest run -p mu-core --test world_config --locked` passes.

## 5. Built-in types

- [ ] 5.1 Add `crates/mu-core/src/world/types/tinymux.toml`, `aresmush.toml` and `evennia.toml` with the values of the table "Checked facts", and load them with `include_str!` through `Quirks`. Run the value checks on each built-in type through the same path as other layers. Add the unit test `bad_quirk_fails_the_value_checks`, which gives the resolver a `Quirks` with a template with no `{password}`. Refuse a user type with a built-in name. Add `built_in_types_hold_only_quirks` and `quirks_refuse_other_fields` to `crates/mu-core/tests/world_config.rs`, and `global_preference_is_not_hidden_by_a_type` and `user_type_with_a_built_in_name` to `crates/mu-cli/tests/world_config.rs`. Make sure that `cargo nextest run -p mu-core --test world_config --locked` and `cargo nextest run -p mu-cli --test world_config --locked` pass.

## 6. The server and the docs

- [ ] 6.1 Make the transport of `WorldSetup` optional, add `PortalError::NoTransport` and map it to `FailedPrecondition` in `crates/mu-server/src/convert.rs`. Make `mu server` resolve the configuration before it opens the socket. Before the socket opens, refuse an id that is both fake and configured. Add each configured world with no transport and no password. Add these tests to `crates/mu-cli/tests/worlds.rs`:
  - `configured_world_waits` and `configured_world_cannot_connect_yet`, for the two new scenarios of "Each world connects and logs in at the start". `two_worlds_connect` keeps passing.
  - `bad_configuration_stops_the_start`, `fake_world_with_a_configured_id` and `server_does_not_read_a_configured_secret`, for the three scenarios of "The server holds the configured worlds"

  Make sure that `cargo nextest run -p mu-cli --test worlds --locked` and `cargo nextest run -p mu-server --locked` pass.
- [ ] 6.2 Edit `docs/src/content/docs/decisions/world-type-profiles.md` to name `figment2` and say why, and to call the window width `columns`, as the spec does. Make the plugin sentence of the page name the login template. Add a page on the files of the configuration to the user docs, with the layout, the layers, `extends`, `disable` and `mu world show`. Follow the `simple-english` skill. Make sure that `mise run docs:check` passes.

## 7. Checks

- [ ] 7.1 Run `mise run check` and `uv run ~/.agents/plugins/sb/skills/review-lint/scripts/review_lint.py --base origin/main`. Make sure that both pass.

## 8. Review

- [ ] 8.1 Run `/sb:openspec-review world-config implementation`. Make sure that the verdict in `review.md` is not changes-requested.

## Workflow follow-up

- After the merge, archive the change in its own pull request.
