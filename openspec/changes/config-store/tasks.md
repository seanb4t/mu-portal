# Tasks: config-store

Test files: `crates/mu-cli/tests/config_store.rs` (new) holds the tests of `mu config` and of the server start with client layers. `crates/mu-cli/tests/bootstrap.rs` (new) holds the tests of the URL and the device name. `crates/mu-cli/tests/client.rs` holds the tests of the `mu` binary in a terminal. `crates/mu-cli/tests/world_config.rs` holds the tests of the configuration directory. `crates/mu-cli/tests/server.rs` holds the tests of `mu server`. The one-test command is in `openspec/config.yaml`.

## 1. Facts to check first

- [ ] 1.1 Read the current documentation of the `url` crate. Parse `unix:///tmp/a%20b/s.sock`, `unix:s.sock`, `unix://host/tmp/s.sock` and `unix:///tmp/s.sock?connect_wait=1&connect_wait=2` in a scratch test. Record the result of each one in design.md D6, and the call that gives the decoded path. Make sure that D6 names each call that the parser of task 5.1 uses.
- [ ] 1.2 Read the current documentation of `rustix::system::uname` and its feature flags. Make sure that design.md D6 names the feature, and states what `nodename` returns on macOS and on Linux.
- [ ] 1.3 Read the current documentation of `sha2` for a hash over several byte slices. Make sure that design.md D4 names the calls.

## 2. The crate mu-config

- [ ] 2.1 Load the `rust-best-practices` skill. Add the crate `crates/mu-config` (design.md D1). It holds the types of a client layer, `echo` and `[keys]`, with unknown fields refused. It also holds the neutral key name with its parser, and the table of entry names. Move the grammar and its unit tests out of `crates/mu-tui/src/keymap.rs`. Add `mu-config` to `ALLOWED_EDGES` and `ALLOWED_CRATES` in `workspace_boundary.rs`. Make sure that `cargo nextest run -p mu-cli --test workspace_boundary --locked` passes, and that the moved unit tests pass in `mu-config`.
- [ ] 2.2 Add the check of one layer in `mu-config`: each case of the requirement "A bad binding makes its layer bad", and each case of "The fields of a client layer". The error names the binding or the field. If TOML gives a line, the error names it too. Make sure that the unit tests `two_names_of_one_key_in_one_layer`, `an_uppercase_key`, `a_control_character`, `shift_with_a_digit`, `an_unknown_entry`, `an_unknown_key_name`, `an_entry_that_needs_an_argument`, `a_server_table`, `an_unknown_field` and `not_toml_names_the_line` pass.
- [ ] 2.3 Make `mu-tui` build its keymap from the neutral key names of `mu-config`, and map each one to a `termina` chord. Add the test `the_entry_table_matches_the_registry` in `mu-tui`. If an entry is added to only one of the two lists, the test fails. Make sure of that, and that every existing test of `crates/mu-tui` passes.
- [ ] 2.4 Update `architecture/model.c4` with the crate `mu-config` and its edges. Make sure that `mise run docs:check` passes.

## 3. The directory scan

- [ ] 3.1 In `crates/mu-core/src/world/dir.rs`, skip the directory `clients` and drop `client.toml` from the top-level files (design.md D3). The error of a leftover `client.toml` says to move it to `clients/global.toml`. Make sure that the tests `the_file_of_the_client`, `a_bad_client_layer_does_not_stop_the_worlds` and `a_leftover_client_toml` in `world_config.rs` pass, and that every other test in `world_config.rs` passes.

## 4. The store

- [ ] 4.1 Add the store to `mu-server` (design.md D2). It takes the snapshot and resolves the worlds through `mu_core::world::resolve`. It checks each client layer through `mu-config`, and writes one stderr line for each bad layer. Move the call of `resolve` out of `mu-cli`. A bad world still stops the start. Make sure that the tests `a_bad_device_layer_at_start`, `a_file_that_is_not_a_layer` and `a_layer_for_an_unknown_kind` in `config_store.rs` pass, and that every test in `server.rs` and `worlds.rs` passes.
- [ ] 4.2 Add the resolution of one client. It merges the layers in order, field by field and key by key. Two names of one key across layers count as one key. It skips a bad layer, and gives the sources and the notices. Add the revision of design.md D4. Make sure that the unit tests `the_device_wins`, `keys_merge_key_by_key`, `two_names_across_layers_are_one_key`, `a_bad_layer_adds_nothing`, `the_revision_ignores_other_devices` and `a_bad_layer_changes_the_revision` in `mu-server` pass.
- [ ] 4.3 Add the reload to the store (design.md D2). It takes a new snapshot and swaps the client part as one value. It lists the world and `server.toml` changes that apply at restart. One mutex serializes reloads. Make sure that the unit tests `a_reload_swaps_the_client_layers`, `a_reload_keeps_the_worlds` and `a_reload_reports_a_bad_world_file` in `mu-server` pass.

## 5. The API and the bootstrap values

- [ ] 5.1 Add `GetClientConfig`, `ReloadConfig` and `SubscribeResponse.config_changed` to `crates/mu-proto/proto/mu/v1/portal.proto`, serve them in `mu-server`, and add the calls to `mu-client` (design.md D5). A reload that changes a client layer sends `config_changed` on every stream, and the journal gets no event. Make sure that the test `a_reload_sends_config_changed` in `mu-server` passes, and that `cargo nextest run -p mu-server --locked` passes.
- [ ] 5.2 Add to `mu-cli` the parser of the server URL and of `connect_wait` (design.md D6). Add the device name, with its default from the host name. Make sure that these tests in `bootstrap.rs` pass: `the_flag_wins`, `the_variable`, `the_default`, `mu_socket_does_not_count`, `a_path_with_a_space`, `another_scheme`, `a_host`, `a_relative_path`, `a_wait_of_zero`, `an_unknown_option`, `a_short_wait`, `the_device_variable`, `the_device_flag_wins`, `the_host_name` and `a_bad_name`.
- [ ] 5.3 Add the verb `mu config` with `show` and `reload` to `crates/mu-cli/src/lib.rs`. `show` prints the TOML with a source comment on each field and binding, and the revision on its first line. Neither starts a server. Make sure that these tests in `config_store.rs` pass: `no_clients_directory`, `a_device_with_no_file`, `the_device_wins`, `keys_merge_key_by_key`, `two_names_of_one_key_in_two_layers`, `one_bad_binding_skips_its_whole_layer`, `a_notice_for_a_bad_layer_that_applies`, `no_notice_for_a_bad_layer_of_another_device`, `the_same_files_give_the_same_revision`, `a_changed_layer_gives_a_new_revision`, `a_layer_of_another_device_leaves_the_revision`, `no_server`, `another_os`, `a_reload_applies_a_client_layer`, `a_reload_with_a_bad_layer`, `a_reload_does_not_apply_a_world_change` and `a_reload_with_a_bad_world_file`.
- [ ] 5.4 Make sure that the scenarios of the `client-config` spec that use `mu config show` pass, as these tests in `config_store.rs`: `a_server_table`, `an_unknown_field`, `not_toml`, `a_file_that_cannot_be_read`, `two_names_of_one_key`, `an_uppercase_key`, `a_control_character`, `shift_with_a_digit`, `an_unknown_entry`, `an_unknown_key_name`, `an_entry_that_needs_an_argument` and `a_bad_echo_value`. As root the file opens, so `a_file_that_cannot_be_read` then skips.

## 6. The start of the client

- [ ] 6.1 Replace `--socket` and `MU_SOCKET` of `mu client` with the URL (design.md D7). Remove `--config`, `client.toml` and the directory compare. Get the client configuration before the screen opens. Remove `config_file::read` if nothing else uses it. Make sure that these tests in `client.rs` pass: `a_client_with_no_server_starts_one`, `no_server_answers_in_time`, `two_clients_start_at_the_same_time`, `a_server_that_closes_the_connection`, `a_server_that_does_not_answer`, `the_started_server_finds_the_default_directory`, `the_started_server_inherits_mu_config`, `no_terminal`, `no_terminal_on_stdin`, `the_client_warns_of_a_missing_directory` and `the_client_takes_the_flag`. Make sure that the tests of the removed requirement "The client notes a server with another directory" are gone.
- [ ] 6.2 Show a notice for each bad layer at attach. Make sure that these tests in `client.rs` pass: `a_bad_device_layer_at_attach`, `the_os_layer_applies`, `a_new_binding`, `the_default_keys_stay`, `a_removed_binding`, `a_binding_replaces_a_default`, `none_on_a_key_with_a_modifier`, `a_binding_of_shift_tab`, `a_binding_of_ctrl_shift_z`, `the_default_dims`, `show`, `hide` and `a_key_bound_to_connect_with_no_world_in_focus`.
- [ ] 6.3 Make sure that the tests of the requirement "The top row shows the first list" in `crates/mu-tui/tests/app.rs` still pass, and that the requirement text names `connect_wait`.

## 7. A live change in the client

- [ ] 7.1 Add the message for `config_changed` to `mu-tui`. Its effect calls `GetClientConfig`, and a new revision replaces the keymap and the echo setting (design.md D8). Each new notice shows once. Make sure that `a_new_binding_works_without_a_restart` and `a_bad_layer_after_a_reload` in `client.rs` pass, and that the unit test `the_same_revision_changes_nothing` in `crates/mu-tui/tests/update.rs` passes.

## 8. Documentation

- [ ] 8.1 Update `docs/src/content/docs/guides/client.md` and `guides/configuration.md`: the URL, `connect_wait`, the device name, the layers in `clients/`, `mu config show` and `mu config reload`, and the move of `client.toml` to `clients/global.toml`. Update `architecture/terminal-client.mdx` and `decisions/terminal-client-architecture.md` where they name `client.toml` or the directory notice. Make sure that `rg -n 'client\.toml|reads its worlds from' docs/src` lists only the migration note, and that `mise run docs:check` passes.
- [ ] 8.2 Add the decision page `docs/src/content/docs/decisions/configuration-ownership.md` for #169: the server owns all configuration, the bootstrap values, the layer order, the rule for a bad client layer, and the plan to writes through the API (#173). Make sure that `mise run docs:check` passes.

## 9. Review and integration

- [ ] 9.1 Run `/sb:openspec-review config-store implementation`. Make sure that the verdict is not changes-requested.
- [ ] 9.2 Run `mise run check` and `cargo nextest run --workspace --locked`. Make sure that both pass.
- [ ] 9.3 Run `/sb:suite-review` on the branch, and fix or triage each blocker and major finding. Make sure that the pull request body says `Fixes #169`.

## Workflow follow-up

- Archive the change after it merges, with an implementation-phase review.md whose verdict is not changes-requested.
- Then propose `client-layout` (#172).
