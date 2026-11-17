# Tasks

## 1. Sources and dependencies

- [ ] 1.1 Read the sources of each fact that the design uses:
  - the current documentation of `figment2`, with Context7 or docs.rs: `Figment::merge`, `find_metadata`, `Metadata` names, `Serialized` providers and `extract`
  - the XDG Base Directory specification, on a relative `XDG_CONFIG_HOME`
  - the documentation or the source of TinyMUX, AresMUSH and Evennia: the login command, and a command that the server accepts with no effect
  - the `>` lines of the transcripts in `crates/mu-core/tests/fixtures/telnet/`

  Fill the table "Checked facts" of the design with one row for each fact and its source. Make sure that each difference from the sources is an edit of the design that its row names.
- [ ] 1.2 Add `figment2` and `serde` to `[workspace.dependencies]` in `Cargo.toml`, and to the dependencies of `mu-core`. Add both to the list of `mu-core` in `ALLOWED_CRATES` of `crates/mu-cli/tests/workspace_boundary.rs`. Make sure that `cargo deny check` and `cargo nextest run -p mu-cli --test workspace_boundary --locked` pass, and that Renovate reads both pins from `Cargo.toml`.

## 2. The directory and the layers

Load the `rust-best-practices` skill before this group.

- [ ] 2.1 Make `world.rs` the module `world/` of `mu-core`. Add the types of the design: `Layer`, `LayerKey`, `Trust`, `Source`, `LayerFields`, `Quirks`, `LayerSource` and `ConfigError`. Parse each layer with `deny_unknown_fields` and the `SecretRef` parse. Check the names of `Charset` and the range of each number. Make each error name the file and the field, and never print a field value of `login`. Add `TomlDirSource`, which reads the layout of the requirement "Each character file makes one world". Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 2.2 Add `ConfigArgs` to `mu-cli` with `--config`, `MU_CONFIG`, `XDG_CONFIG_HOME` and `HOME`, as the requirement "The configuration directory" states. Add the verb `world` with `mu world list`. Make the process helper in `crates/mu-cli/tests/support/process.rs` set `MU_CONFIG` to an empty directory of the test, and add the test `helper_points_mu_config_at_an_empty_directory`. Add `crates/mu-cli/tests/world_config.rs` with these tests:
  - `flag_wins_over_the_environment`, `default_directory`, `directory_under_home` and `missing_directory`, for the four scenarios of the configuration directory
  - `worlds_from_character_files`, `character_with_no_game`, `file_name_outside_the_id_rule` and `stray_file`, for the four scenarios of the character files
  - `unknown_field`, `port_out_of_range` and `charset_the_engine_cannot_read`, for the three scenarios of "Each field is known"
  - `password_in_a_file` and `secret_that_is_not_a_reference`, for the scenarios of the same names

  In `crates/mu-cli/tests/cli.rs`, make `help_lists_verbs` expect the four verbs. Make sure that `cargo nextest run -p mu-cli --locked` passes.

## 3. The merge and `mu world show`

- [ ] 3.1 Add the chain of each world, as the design describes. Merge the fields with `figment2`, and keep the source of each field. Grow `ResolvedWorld` and `Login` with the new fields and `sources`, and keep `with_default_login` for the fake world. Add the check of `host`, `port` and the login pair after the merge. Add `crates/mu-core/tests/world_config.rs` with `each_field_has_a_source`. Add the unit test `empty_layer_gives_figment_no_keys`. Remove `skip_serializing_if` from one optional field, and make sure that the test fails. Make sure that `cargo nextest run -p mu-core --locked` passes.
- [ ] 3.2 Add `mu world show <id>`, which prints TOML with a source comment on each field, as the design describes. Add these tests to `crates/mu-cli/tests/world_config.rs`:
  - `last_layer_wins`, `global_default_reaches_each_world` and `table_merges_field_by_field`, for the three scenarios of the merge
  - `user_type_extends_built_in_type`, `unknown_type`, `game_with_no_type`, `loop_of_types` and `character_that_names_a_parent`, for the five scenarios of `extends`
  - `world_with_no_port`, `login_name_with_no_secret` and `world_that_logs_in_by_hand`, for the three scenarios of "A world has a place to connect"
  - `sources_of_fields` and `unknown_world`, for the two scenarios of `show`
  - `show_does_not_read_the_secret`

  Make sure that `cargo nextest run -p mu-cli --test world_config --locked` passes.

## 4. Rules, scripts and trust

- [ ] 4.1 Add the accumulation of `rules` and `scripts`, as the design describes. A `disable` removes an entry, and an entry with a known id replaces it. Print the entries in `mu world show`. Add `entries_from_each_layer`, `layer_disables_an_inherited_rule`, `layer_replaces_an_inherited_rule` and `disable_with_no_target` to `crates/mu-cli/tests/world_config.rs`, for the four scenarios of "Rules and scripts accumulate". Make sure that `cargo nextest run -p mu-cli --test world_config --locked` passes.
- [ ] 4.2 Add the trust check for a layer with `Trust::SharedPlugin`. Add `plugin_cannot_set_the_host` and `plugin_can_set_a_preference` to `crates/mu-core/tests/world_config.rs`, with a test `LayerSource` that gives a plugin layer named `colors`. Make sure that `cargo nextest run -p mu-core --test world_config --locked` passes.

## 5. Built-in types

- [ ] 5.1 Add `crates/mu-core/src/world/types/tinymux.toml`, `aresmush.toml` and `evennia.toml` with the values of the table "Checked facts", and load them with `include_str!` through `Quirks`. Refuse a user type with a built-in name. Add `built_in_types_hold_only_quirks` to `crates/mu-core/tests/world_config.rs`, and `global_preference_is_not_hidden_by_a_type` and `user_type_with_a_built_in_name` to `crates/mu-cli/tests/world_config.rs`. Make sure that `cargo nextest run -p mu-core --test world_config --locked` and `cargo nextest run -p mu-cli --test world_config --locked` pass.

## 6. The server and the docs

- [ ] 6.1 Make the transport of `WorldSetup` optional, add `PortalError::NoTransport` and map it to `FailedPrecondition` in `crates/mu-server/src/convert.rs`. Make `mu server` resolve the configuration before it opens the socket. Refuse an id that is both fake and configured. Add each configured world with no transport and no password. Add these tests to `crates/mu-cli/tests/worlds.rs`:
  - `configured_world_waits` and `configured_world_cannot_connect_yet`, for the two new scenarios of "Each world connects and logs in at the start". `two_worlds_connect` keeps passing.
  - `bad_configuration_stops_the_start`, `fake_world_with_a_configured_id` and `server_does_not_read_a_configured_secret`, for the three scenarios of "The server holds the configured worlds"

  Make sure that `cargo nextest run -p mu-cli --test worlds --locked` and `cargo nextest run -p mu-server --locked` pass.
- [ ] 6.2 Edit `docs/src/content/docs/decisions/world-type-profiles.md` to name `figment2` and say why. Add a page on the files of the configuration to the user docs, with the layout, the layers, `extends`, `disable` and `mu world show`. Follow the `simple-english` skill. Make sure that `mise run docs:check` passes.

## 7. Checks

- [ ] 7.1 Run `mise run check` and `uv run ~/.agents/plugins/sb/skills/review-lint/scripts/review_lint.py --base origin/main`. Make sure that both pass.

## 8. Review

- [ ] 8.1 Run `/sb:openspec-review world-config implementation`. Make sure that the verdict in `review.md` is not changes-requested.

## Workflow follow-up

- After the merge, archive the change in its own pull request.
