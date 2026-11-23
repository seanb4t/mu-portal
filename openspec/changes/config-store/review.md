---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-23T17:49:26+00:00
head: b50eb43641ff49cb15e6b753d2890a31dc64dbc1
round: 3
next_id: 62
lenses:
  spec-lint: ok
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
  verify: not-applicable
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-43 | approach | blocker | in-change | The scenario A new binding works without a restart needs config_changed to reach mu-tui, but mu-tui reads the stream only as ClientEvent items, and D5 adds only ClientEvent::Opened. No decision makes mu-client yield an item for config_changed; today Resume::next drops every message that is not a Hello or an Event. | design.md > D5. The API — "So the change adds `ClientEvent::Opened`, which `mu-client` sends for every `Hello`, the quiet one too." | In D5, add a ClientEvent for config_changed that Resume::next yields, name it in D8 as the source of the mu-tui message, and add it to task 7.1. |  |  | unmet-requirement |
| R-44 | approach | blocker | in-change | The requirement of mu config show gives it the flags --kind and --os with defaults, and the scenario Another OS runs --os linux, but no decision creates either flag. D6 gives mu config only --server and --device. | specs/config-store/spec.md > Requirement: mu config show prints one client configuration — "The default of `--kind` SHALL be `tui`. The default of `--os` SHALL be the OS of the machine." | In D6, give mu config show the flags --kind and --os, checked with mu-config, with the defaults tui and the OS of the machine, and name them in task 5.3. |  | severity? should-fix 0.75 | unmet-requirement |
| R-3 | invariants | blocker | in-change | The OS name rule (macos or linux) has no test anywhere: task 2.3 tests an unknown kind only, task 4.2 a layer of an unknown kind only, and task 5.1 a bad device in a call only. A bad kind or OS in GetClientConfig is never sent. | specs/config-store/spec.md > Requirement: The layer files of a client — "A kind SHALL be `tui`. An OS SHALL be `macos` or `linux`." | Add an_unknown_os to task 2.3, a_layer_for_an_unknown_os (clients/os/windows.toml) to task 4.2, and a_bad_kind_or_os_in_a_call to task 5.1, which calls GetClientConfig with kind tiu and with OS windows and expects an error naming each. | new-evidence | severity? should-fix 0.76 | unmet-requirement |
| R-45 | approach | should-fix | in-change | D2 says a read error outside clients/ stops the snapshot as today, and TomlDirSource becomes a snapshot. But today the scan never reads server.toml: an unreadable server.toml is left to its reader, and mu world list still succeeds. A snapshot that reads the bytes of server.toml and stops on its read error makes mu world list fail, against the non-goal that mu world reads the directory as before. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "Outside `clients/`, the error still stops the read, as today." | In D2, keep an unreadable server.toml in the snapshot as an unreadable file with its error, as under clients/, leave it to server_config, and list unreadable_server_file_makes_no_world in task 3.3. |  |  | wrong-behaviour |
| R-46 | fit | should-fix | in-change | D2 rebuilds in mu-server the figment2 block of the world merge: a Named provider per layer over a defaults layer, then find_metadata mapped back to a layer origin. resolve.rs already has that block, and the two copies can drift, for example in the find_value guard or the index-as-name trick. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "Each layer is a `Named` provider, the merge is `Figment::merge`, and the source of a field is `find_metadata`." | Extract the merge-with-sources block of world() in resolve.rs into one generic mu-core function over serializable layers and field paths. Have the world merge, D2 and task 4.3 call it. |  |  | drift |
| R-47 | fit | should-fix | in-change | D2 adds server_config::check for the reload. At start, server.toml is checked by server_config::read and then telemetry::export. Two checks of one file can drift: a value that telemetry::export refuses at start can pass the reload check. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "`mu-cli` gives the store a check function for it, `server_config::check`, which takes the bytes and returns the error of" | Make server_config::check the one check of server.toml: the parse plus the value checks of telemetry::export. Have the start path in server::run call it too, and state in D2 that start and reload share it. |  |  | drift |
| R-48 | fit | should-fix | in-change | Task 4.2 has the store in mu-server write its own stderr line for each bad layer. mu-cli already has report and warn, which write the line in one write and turn control characters into U+FFFD, so a file name under clients/ cannot send an escape sequence to the terminal. The design does not name them. | tasks.md > 4. The store — "It writes one stderr line for each bad layer." | Have the store return its bad layers and let mu-cli print each with crate::warn("server", ...). Or give the store a report callback, as D2 does for the server.toml check. Name the choice in D2. |  |  | wrong-behaviour |
| R-49 | fit | should-fix | in-change | Task 2.1 puts the echo type of a client layer in mu-config, but task 6.1 removes only ClientConfig from mu-tui. So the Echo enum of mu-tui, with its default Dim, stays beside the new type. The default of echo is then stated in two crates. | tasks.md > 2. The crate mu-config — "It holds the types of a client layer, `echo` and `[keys]`, with unknown fields refused." | Move Echo with its default into mu-config in task 2.1, and have mu-tui's Settings, model and update use mu_config::Echo. State this in D1. |  |  | drift |
| R-50 | invariants | should-fix | in-change | The connect_wait rule refuses 0 or less, a value that is not finite and one under 1 ns, but task 5.2 names only a_wait_of_zero. No test sends nan, inf, a negative value or 1e-10 through the query, and no task keeps the unit test that covers them today. | specs/client-bootstrap/spec.md > Requirement: The connect_wait option — "A value that is 0 or less, not finite, or less than 1 ns SHALL stop the command." | In task 5.2, state that the query parser passes the value through connect_wait of client.rs and keep a_wait_that_is_no_time_is_refused. Add a bootstrap test that runs connect_wait=nan, connect_wait=-1 and connect_wait=0.0000000001 and expects exit 1 naming connect_wait. |  |  | wrong-behaviour |
| R-51 | invariants | should-fix | in-change | The URL rule refuses a user, a password and a fragment, but task 5.2 names tests only for a scheme, a host, no double slash and a relative path. A fragment such as unix:///tmp/s.sock#x keeps a good path, so a missing check connects silently and no test fails. | specs/client-bootstrap/spec.md > Requirement: A URL names a Unix socket — "A URL with another scheme, a relative path, a host, a user, a password or a fragment SHALL stop the command before it co" | Add to task 5.2 the tests a_fragment (unix:///<dir>/s.sock#x), a_user (unix://u@/<dir>/s.sock) and a_password (unix://u:p@/<dir>/s.sock), each expecting exit 1 with the URL on stderr, and record their url-crate parse in task 1.1. |  |  | unmet-requirement |
| R-52 | invariants | should-fix | in-change | An empty MU_SERVER must count as unset, but no test sets MU_SERVER to the empty string. If the parser reads it as a URL, every command with MU_SERVER= stops with a URL error. | specs/client-bootstrap/spec.md > Requirement: The server URL — "An empty `MU_SERVER` SHALL count as unset." | Add the test an_empty_mu_server to bootstrap.rs: MU_SERVER is empty, XDG_RUNTIME_DIR names a served default socket, and mu config show --device laptop exits 0. Name it in task 5.2. |  |  | wrong-behaviour |
| R-53 | invariants | should-fix | in-change | An empty MU_DEVICE must count as unset and fall back to the host name, but the only empty-name test is an_empty_device_flag, which expects a refusal. Nothing tells the empty variable from the empty flag. | specs/client-bootstrap/spec.md > Requirement: The device name — "An empty `MU_DEVICE` SHALL count as unset." | Add the test an_empty_device_variable to bootstrap.rs: MU_DEVICE is empty, clients/devices/<host>.toml holds echo = "hide", and mu config show prints that source. Name it in task 5.2. |  |  | wrong-behaviour |
| R-54 | invariants | should-fix | in-change | A bad client layer must never stop the server, but the only walk-error test under clients/ is a broken link. A directory that stats but cannot be listed (clients/ itself, or clients/devices/ with mode 000) fails at read_dir. D2 does not say whether clients/ itself is under clients/, so the start can stop. | crates/mu-core/src/world/dir.rs:150 — "fs::read_dir(directory).map_err(\|error\| ConfigError::new(&place, error.to_string()))?;" | State in D2 that a read_dir error of clients/ or of a directory under it is a bad client layer. Add to task 3.1 the unit tests an_unreadable_clients_directory_is_kept and an_unreadable_devices_directory_is_kept (mode 000, skipped as root), and assert that the server starts. |  |  | wrong-behaviour |
| R-55 | invariants | should-fix | in-change | A reload must apply the good client layers even when a file is bad, but D2 keeps a walk error outside clients/ fatal to the snapshot. A link to nothing in games/ then stops the reload before clients/ is read, and no good layer applies. The only bad-world reload test uses a parse error. | specs/config-store/spec.md > Requirement: mu config reload reads the directory again — "The good layers SHALL apply in both cases." | Have D2 say that at a reload a walk error outside clients/ is a bad world file to report and the walk goes on, so clients/ is still read. Add the test a_reload_with_a_link_to_nothing_in_games to task 5.3: exit 1, the error names the link, and a new clients/global.toml applies. |  |  | unmet-requirement |
| R-56 | approach | suggestion | in-change | The Context says mu-tui parses client.toml. At head mu-tui holds the type ClientConfig, and mu-cli reads and parses the file through config_file::read. | design.md > Context — "`mu-tui` parses `client.toml`." | Say in the Context that mu-tui holds the types of client.toml and mu-cli parses the file. |  | scope? deviates-design 0.73 |  |
| R-57 | fit | suggestion | in-change | The GetClientConfig reply of D5 carries a separate echo_source field. The existing pattern keeps sources in one map keyed by field path. #172 adds the layout to the client configuration, so each new field there needs its own source field in the proto. | design.md > D5. The API — "-> revision, echo, echo_source, bindings[{key, entry, source}], notices[{path, reason}]" | Give the reply one sources map from field path to source, as ResolvedWorld.sources does. Then #172 adds only its value fields. |  | scope? deviates-design 0.67 |  |
| R-58 | fit | suggestion | in-change | Task 5.2 adds a parser of connect_wait. mu-cli already has connect_wait(), which applies the same rule: not finite, or under 1 ns, is no wait. It also has shown() for the time in the message. Neither D6 nor the task names them. | tasks.md > 5. The API and the bootstrap values — "Add to `mu-cli` the parser of the server URL and of `connect_wait` (design.md D6)." | Name connect_wait() and shown() in D6 as the check and the print of the option value. Have the URL parser call them. |  |  |  |
| R-59 | invariants | suggestion | in-change | The merge needs every canonical key name to hold no dot, because figment2 splits a path at each '.'. That rule covers every key name. Only the unit test for '.' checks it, while key_names_read_back already generates '.' and does not assert it. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "`figment2` splits a key path at each `.` and has no escape, so no canonical name holds a dot." | When key_names_read_back moves to mu-config (task 2.1), add prop_assert!(!written.contains('.')) after the canonical name is written. It uses proptest, as crates/mu-tui/tests/keymap.rs does. |  | severity? should-fix 0.73 |  |
| R-60 | invariants | suggestion | in-change | D4 says the framing gives two different layer lists two different inputs. That rule covers every list, but task 4.4 tests one hand-made pair. | design.md > D4. The revision is a hash of the layers that apply — "Each entry is framed, so no two lists give the same input" | Add a proptest in mu-server that decodes the framed input back to the list of (path, tag, bytes or error kind) and asserts equality, after the pattern of crates/mu-tui/tests/keymap.rs. Add proptest to the mu-server row of ALLOWED_CRATES in task 4.1. |  |  |  |
| R-61 | spec-lint | suggestion | in-change | D2 states as settled that figment2 merges a nested table key by key, which the design lists as an unchecked third-party fact, and task 4.3 builds on it. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "A nested table merges key by key in `figment2`, so `[keys]` gets its rule from the library." | Mark the D2 sentence as conditional on task 1.4, or record the result of 1.4 in D2 before task 4.3 starts. |  |  |  |

## Repeats

None.

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-30 | suggestion | deferred | Suggestions with no behavior change: tie ListWorlds.config_dir to a requirement or drop it, and share one TOML-with-sources renderer between mu world show and mu config show | https://github.com/seanb4t/mu-portal/issues/176 |
| R-31 | suggestion | deferred | Suggestions with no behavior change: tie ListWorlds.config_dir to a requirement or drop it, and share one TOML-with-sources renderer between mu world show and mu config show | https://github.com/seanb4t/mu-portal/issues/176 |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 00dc0995a2d309a82053d7d5fe23f0472b5b4e84 | 4619aaac294581f63428f0fa799a3ecda7f669c87b0c1f49b8766fbac1c6cd30 | changes-requested | 32 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 5cbf017513e9ae12df877428ff4d90c1dc61b993 | 1388d98570750bcc4098ddf6edab3aefbc494dd56f7ce6a9dc12248d5696f2c1 | changes-requested | 10 | 0 | 5 | 24 | 0 |  |
| 3 | plan | full | b50eb43641ff49cb15e6b753d2890a31dc64dbc1 | 97ab398377b3f8615fb16eead2f9c6064308a922f648cf1e647d26efa5df8a8c | changes-requested | 19 | 0 | 0 | 0 | 0 | new scenarios from rounds 1 and 2 need coverage matrix rows (R-33 to R-42) |

## Judge

judge: 40 questions, 0.002 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| The flag wins | 5.2 | covered |  |
| The variable | 5.2 | covered |  |
| The default | 5.2 | covered |  |
| MU_SOCKET does not count | 5.2 | covered |  |
| A path with a space | 5.2 | covered |  |
| Another scheme | 5.2 | covered |  |
| A host | 5.2 | covered |  |
| No double slash | 5.2 | covered |  |
| A relative path | 5.2 | covered |  |
| A wait of zero | 5.2 | covered |  |
| An unknown option | 5.2 | covered |  |
| An option twice | 5.2 | covered |  |
| A short wait | 5.2 | covered |  |
| The host name | 5.2 | covered |  |
| A bad name | 5.2 | covered |  |
| A name of 64 characters | 5.2 | covered |  |
| A name of 65 characters | 5.2 | covered |  |
| An empty device flag | 5.2 | covered |  |
| The OS layer applies | 6.2 | covered |  |
| The socket flag | 6.1 | covered |  |
| Connect the world in focus | 6.3 | covered |  |
| Disconnect a named world | 6.3 | covered |  |
| A key bound to connect with no world in focus | 6.3 | covered |  |
| Connect with no world in focus | 6.3 | covered |  |
| A trailing space is no argument | 6.3 | covered |  |
| Disconnect an unknown world | 6.3 | covered |  |
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
| The default dims | 6.2 | covered |  |
| Show | 6.2 | covered |  |
| Hide | 6.2 | covered |  |
| A bad echo value | 5.4 | covered |  |
| A bad device layer at start | 4.2 | covered |  |
| A file that is not a layer | 4.2 | covered |  |
| A device file with a bad name | 4.2 | covered |  |
| A link to nothing under clients | 4.2 | covered |  |
| A binding of a dot | 5.3 | covered |  |
| A layer for an unknown kind | 4.2 | covered |  |
| No clients directory | 5.3 | covered |  |
| A device with no file | 5.3 | covered |  |
| The device wins | 5.3 | covered |  |
| Keys merge key by key | 5.3 | covered |  |
| Two names of one key in two layers | 5.3 | covered |  |
| One bad binding skips its whole layer | 5.3 | covered |  |
| A notice for a bad layer that applies | 5.3 | covered |  |
| A bad device name in a call | 5.1 | covered |  |
| No notice for a bad layer of another device | 5.3 | covered |  |
| The same files give the same revision | 5.3 | covered |  |
| A changed layer gives a new revision | 5.3 | covered |  |
| A layer of another device leaves the revision | 5.3 | covered |  |
| No server | 5.3 | covered |  |
| Another OS | 5.3 | covered |  |
| A reload applies a client layer | 5.3 | covered |  |
| A reload with a bad layer | 5.3 | covered |  |
| A reload does not apply a world change | 5.3 | covered |  |
| A reload with a bad world file | 5.3 | covered |  |
| A new binding works without a restart | 7.1 | covered |  |
| A reload while the stream is down | 7.1 | covered |  |
| A reload across a server restart | 7.1 | covered |  |
| A bad layer after a reload | 7.1 | covered |  |
| A bad device layer at attach | 6.2 | covered |  |
| A client with no server starts one | 6.1 | covered |  |
| No server answers in time | 6.1 | covered |  |
| Two clients start at the same time | 6.1 | covered |  |
| A server that closes the connection | 6.1 | covered |  |
| A server that does not answer | 6.1 | covered |  |
| The started server finds the default directory | 6.1 | covered |  |
| The started server inherits MU_CONFIG | 6.1 | covered |  |
| No terminal | 6.1 | covered |  |
| No terminal on stdin | 6.1 | covered |  |
| Before the first list | 6.4 | covered |  |
| A send before the first list | 6.4 | covered |  |
| Text typed before the first list | 6.4 | covered |  |
| An event before the first list | 6.4 | covered |  |
| A focus key before the first list | 6.4 | covered |  |
| A failed first list after an event | 6.4 | covered |  |
| A notice before the first list | 6.4 | covered |  |
| The first list fails | 6.4 | covered |  |
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
| A link to nothing | 3.3 | covered |  |
| The file of the server | 3.3 | covered |  |
| A character named server | 3.3 | covered |  |
| The file of the client | 3.2 | covered |  |
| A game named clients | 3.2 | covered |  |
| A bad client layer does not stop the worlds | 3.2 | covered |  |
| A leftover client.toml | 3.2 | covered |  |

### Not run

Plan phase: no verification runs.

## Verification runs

None.

## Dropped

None.

## Lens notes

- approach: Prior rows R-1, R-10, R-12, R-18 and R-32 (fixed) were checked: D5 now fetches after each Hello through ClientEvent::Opened, D2 names figment2 and DirSnapshot as a LayerSource, and D1 moves the proptest with proptest as a dev-dependency. None is raised again; app-1 is a separate gap in the path of config_changed.
- approach: Cited facts that hold at head: the ALLOWED_EDGES and ALLOWED_CRATES rows of workspace_boundary.rs, the top files and stray-file error of dir.rs, the walk that stops on a stat error, parse_toml, resolve over LayerSource, figment2 Named and find_metadata in resolve.rs, server.toml read before serve_worlds, ListWorlds.config_dir, the Reset, Resumed and quiet Hello of mu-client, url 2.5.9 and sha2 0.10.9 in Cargo.lock, rustix 1.1.6 with uname and nodename under the system feature.
- approach: D5 relies on the stream being open at Hello. event_stream at head yields the Hello first and then subscribes nothing else; D5 does not say that each stream subscribes to the config_changed broadcast before it yields its Hello. This ordering is left to the invariants lens.
- fit: D6 picks the url crate and does not name http::Uri as a rejected alternative, although http is already in the mu-cli row of ALLOWED_CRATES. Task 1.1 could record why it does not parse unix:///path.
- fit: R-31 (the show render that mu world show also has) is deferred and is not raised again: this round has no new evidence for it.
- fit: The repository rows of the library-preference table come from openspec/config.yaml context. The test-helper and command-definition rows were checked, and none conflicts with the plan.
- invariants: I1: A device name is always 1 to 64 characters of a-z, 0-9, -, _ and . \| enforced at mu-config names check (task 2.3), called by mu-cli (5.2) and GetClientConfig (5.1) \| boundary test a_device_name_of_64, a_device_name_of_65, an_empty_device_name, a_name_of_64_characters, a_name_of_65_characters, an_empty_device_flag
- invariants: I2: connect_wait is never 0 or less, not finite, or under 1 ns \| enforced at connect_wait in crates/mu-cli/src/client.rs:56 today, and the new query parser of task 5.2 \| boundary test a_wait_of_zero only (inv-1). Probe: nan, inf and 1e400 parse as f64 and connect_wait refuses them, an empty value fails the f64 parse, and 1e19 s is accepted and tokio::time::timeout takes it
- invariants: I3: A URL with a user, a password or a fragment never connects \| enforced at the URL parser of task 5.2 \| boundary test none (inv-2)
- invariants: I4: An empty MU_SERVER or MU_DEVICE counts as unset \| enforced at the bootstrap parser of task 5.2 \| boundary test none (inv-3, inv-4)
- invariants: I5: A query option never appears twice \| enforced at the URL parser of task 5.2 \| boundary test an_option_twice
- invariants: I6: A kind is always tui and an OS is always macos or linux \| enforced at mu-config (task 2.3), the store (4.2) and GetClientConfig (5.1) \| boundary test an_unknown_kind and a_layer_for_an_unknown_kind; none for an OS (inv-5)
- invariants: I7: A bad client layer never stops the server \| enforced at DirSnapshot (task 3.1) and the store (4.2) \| boundary test a_broken_link_under_clients_is_kept, a_link_to_nothing_under_clients, a_bad_device_layer_at_start; none for an unlistable directory (inv-6)
- invariants: I8: The good client layers always apply at a reload, also with a bad file \| enforced at the reload (task 4.5), except a walk error outside clients/ (inv-7) \| boundary test a_reload_with_a_bad_layer, a_reload_with_a_bad_world_file (parse error only)
- invariants: I9: No canonical key name holds a dot \| enforced at the canonical name of mu-config (task 2.1) \| boundary test a_dot_key_has_the_canonical_name_period, a_binding_of_a_dot; property candidate inv-8
- invariants: I10: Two names of one key are one key, in one layer and across layers \| enforced at the canonical name before the figment2 merge (task 4.3) \| boundary test two_names_of_one_key_in_one_layer, two_names_across_layers_are_one_key, and key_names_read_back (shuffled modifiers)
- invariants: I11: Two different lists of layers never give the same revision input; the revision is the same across a restart \| enforced at the framing of D4 (task 4.4) \| boundary test the_revision_frames_each_layer, an_unreadable_layer_changes_the_revision, the_same_files_give_the_same_revision; property candidate inv-9
- invariants: I12: GetClientConfig never sees a mix of old and new layers; reloads are serialized \| enforced at the swap of one value under one mutex (task 4.5) \| boundary test none, which is a concurrency property with no edge value
- invariants: I13: Each Hello, the quiet one too, makes the client fetch its configuration \| enforced at ClientEvent::Opened (task 7.1); at head hello() in crates/mu-client/src/lib.rs:273 returns Resumed, Reset or nothing \| boundary test opened_after_each_hello, opened_after_the_quiet_hello
- invariants: I14: config_changed is never journaled \| enforced at the server broadcast (task 5.1) \| boundary test a_reload_sends_config_changed
- invariants: I15: clients is skipped at the top level only \| enforced at dir.rs walk (task 3.2) \| boundary test a_game_named_clients. The top-file rule at head matches server.toml in any case (dir.rs:120). D3 gives clients/ no such rule, so on a case-insensitive file system Clients/ becomes stray world files, which stop the start with an error that names them
- invariants: I16: The entry table of mu-config and the registry of mu-tui name the same entries with the same argument flag \| enforced at the_entry_table_matches_the_registry (task 2.4) \| boundary test that test
- invariants: mu_core::world::parse_toml (crates/mu-core/src/world/parse.rs:43) takes &str, so the store must turn snapshot bytes into UTF-8, and non-UTF-8 bytes must be a bad layer. No test covers a client layer that is not UTF-8. It keeps the serde message, which can quote a value, unless Hidden covers the field, so D2's 'never to quote a value' holds only with Hidden::All
- invariants: The mu-server row of ALLOWED_CRATES has no serde. A serde derive in the store fails workspace_boundary unless only types of mu-config are used.
- spec-lint: Every requirement of the seven delta specs has at least one scenario, and every scenario has a WHEN or GIVEN line and a THEN line.
- spec-lint: Every THEN step names a status, an exit code, a stderr or stdout line, a file, a rendered cell or a server record. No THEN restates its WHEN.
- spec-lint: Each requirement heading has a task line with two or more distinctive words of its subject, checked on the artifact text.
- spec-lint: BREAKING appears in the proposal and in the delta specs (client-bootstrap, client-config, server-process). No mismatch.
- spec-lint: Build targets exist: mise tasks check and docs:check, the cargo commands in tasks.md, the test target workspace_boundary in mu-cli, and the crate mu-server. The new targets config_store.rs, bootstrap.rs and the crate mu-config are created by the change.
- spec-lint: All tasks name a command, a test, a file or an observable result.
- test-coverage: Verification command is named once in tasks.md line 3 (one-test command in openspec/config.yaml), so each task that lists test names counts as covered.
- test-coverage: The scenario headings The variable and The flag wins each appear twice in client-bootstrap (URL and device); each has one row, and task 5.2 lists the_variable, the_flag_wins, the_device_variable and the_device_flag_wins.
- test-coverage: Every BREAKING requirement has a scenario: The socket flag, The client warns of a missing directory (mu client --config), A leftover client.toml.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | fixed_after | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D5. The API"],"SubscribeResponse.config_changed {} (not in the journal, no sequence number)"] | fixed | 30253ffee5f0416968dc01cb22997ee738fa1972 | 2 |  | severity? should-fix 0.66 | A config_changed sent while a client's stream is lost is never seen: the stream resumes silently, and no decision has the client fetch GetClientConfig again on Resumed or Reset. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu config reload reads the directory again"],"The command SHALL print one line for each bad file to stderr."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The reload must report a bad server.toml, but D2 keeps the server.toml parser in mu-cli and the store holds only its bytes, and mu-server cannot depend on mu-cli. No decision checks server.toml at reload. |
| R-3 | 1 | 3 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: GetClientConfig resolves one client"],"A bad name SHALL be an error that names it."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  | severity? should-fix 0.76 | app-3: The server must check device names (layer file names, and the device of GetClientConfig), but the design puts the device-name rule only in mu-cli. D1 gives mu-config the key grammar and entry names, not the device, kind or OS names.; inv-6: The server-side check of the kind, the OS and the device of GetClientConfig has no task and no test. Only the CLI checks the device name, and remote callers reach the API directly. |
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
| R-43 | 3 | 3 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D5. The API"],"So the change adds `ClientEvent::Opened`, which `mu-client` sends for every `Hello`, the quiet one too."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | The scenario A new binding works without a restart needs config_changed to reach mu-tui, but mu-tui reads the stream only as ClientEvent items, and D5 adds only ClientEvent::Opened. No decision makes mu-client yield an item for config_changed; today Resume::next drops every message that is not a Hello or an Event. |
| R-44 | 3 | 3 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu config show prints one client configuration"],"The default of `--kind` SHALL be `tui`. The default of `--os` SHALL be the OS of the machine."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  | severity? should-fix 0.75 | The requirement of mu config show gives it the flags --kind and --os with defaults, and the scenario Another OS runs --os linux, but no decision creates either flag. D6 gives mu config only --server and --device. |
| R-45 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"Outside `clients/`, the error still stops the read, as today."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | D2 says a read error outside clients/ stops the snapshot as today, and TomlDirSource becomes a snapshot. But today the scan never reads server.toml: an unreadable server.toml is left to its reader, and mu world list still succeeds. A snapshot that reads the bytes of server.toml and stops on its read error makes mu world list fail, against the non-goal that mu world reads the directory as before. |
| R-46 | 3 | 3 | should-fix | in-change | drift | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"Each layer is a `Named` provider, the merge is `Figment::merge`, and the source of a field is `find_metadata`."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | D2 rebuilds in mu-server the figment2 block of the world merge: a Named provider per layer over a defaults layer, then find_metadata mapped back to a layer origin. resolve.rs already has that block, and the two copies can drift, for example in the find_value guard or the index-as-name trick. |
| R-47 | 3 | 3 | should-fix | in-change | drift | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"`mu-cli` gives the store a check function for it, `server_config::check`, which takes the bytes and returns the error of the file."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | D2 adds server_config::check for the reload. At start, server.toml is checked by server_config::read and then telemetry::export. Two checks of one file can drift: a value that telemetry::export refuses at start can pass the reload check. |
| R-48 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["tasks.md","4. The store"],"It writes one stderr line for each bad layer."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | Task 4.2 has the store in mu-server write its own stderr line for each bad layer. mu-cli already has report and warn, which write the line in one write and turn control characters into U+FFFD, so a file name under clients/ cannot send an escape sequence to the terminal. The design does not name them. |
| R-49 | 3 | 3 | should-fix | in-change | drift | ["artifact",["tasks.md","2. The crate mu-config"],"It holds the types of a client layer, `echo` and `[keys]`, with unknown fields refused."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | Task 2.1 puts the echo type of a client layer in mu-config, but task 6.1 removes only ClientConfig from mu-tui. So the Echo enum of mu-tui, with its default Dim, stays beside the new type. The default of echo is then stated in two crates. |
| R-50 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The connect_wait option"],"A value that is 0 or less, not finite, or less than 1 ns SHALL stop the command."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | The connect_wait rule refuses 0 or less, a value that is not finite and one under 1 ns, but task 5.2 names only a_wait_of_zero. No test sends nan, inf, a negative value or 1e-10 through the query, and no task keeps the unit test that covers them today. |
| R-51 | 3 | 3 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-bootstrap/spec.md","Requirement: A URL names a Unix socket"],"A URL with another scheme, a relative path, a host, a user, a password or a fragment SHALL stop the command before it connects."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | The URL rule refuses a user, a password and a fragment, but task 5.2 names tests only for a scheme, a host, no double slash and a relative path. A fragment such as unix:///tmp/s.sock#x keeps a good path, so a missing check connects silently and no test fails. |
| R-52 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The server URL"],"An empty `MU_SERVER` SHALL count as unset."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | An empty MU_SERVER must count as unset, but no test sets MU_SERVER to the empty string. If the parser reads it as a URL, every command with MU_SERVER= stops with a URL error. |
| R-53 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The device name"],"An empty `MU_DEVICE` SHALL count as unset."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | An empty MU_DEVICE must count as unset and fall back to the host name, but the only empty-name test is an_empty_device_flag, which expects a refusal. Nothing tells the empty variable from the empty flag. |
| R-54 | 3 | 3 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-core/src/world/dir.rs",150] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | A bad client layer must never stop the server, but the only walk-error test under clients/ is a broken link. A directory that stats but cannot be listed (clients/ itself, or clients/devices/ with mode 000) fails at read_dir. D2 does not say whether clients/ itself is under clients/, so the start can stop. |
| R-55 | 3 | 3 | should-fix | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu config reload reads the directory again"],"The good layers SHALL apply in both cases."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | A reload must apply the good client layers even when a file is bad, but D2 keeps a walk error outside clients/ fatal to the snapshot. A link to nothing in games/ then stops the reload before clients/ is read, and no good layer applies. The only bad-world reload test uses a parse error. |
| R-56 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","Context"],"`mu-tui` parses `client.toml`."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  | scope? deviates-design 0.73 | The Context says mu-tui parses client.toml. At head mu-tui holds the type ClientConfig, and mu-cli reads and parses the file through config_file::read. |
| R-57 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","D5. The API"],"-> revision, echo, echo_source, bindings[{key, entry, source}], notices[{path, reason}]"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  | scope? deviates-design 0.67 | The GetClientConfig reply of D5 carries a separate echo_source field. The existing pattern keeps sources in one map keyed by field path. #172 adds the layout to the client configuration, so each new field there needs its own source field in the proto. |
| R-58 | 3 | 3 | suggestion | in-change |  | ["artifact",["tasks.md","5. The API and the bootstrap values"],"Add to `mu-cli` the parser of the server URL and of `connect_wait` (design.md D6)."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | Task 5.2 adds a parser of connect_wait. mu-cli already has connect_wait(), which applies the same rule: not finite, or under 1 ns, is no wait. It also has shown() for the time in the message. Neither D6 nor the task names them. |
| R-59 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"`figment2` splits a key path at each `.` and has no escape, so no canonical name holds a dot."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  | severity? should-fix 0.73 | The merge needs every canonical key name to hold no dot, because figment2 splits a path at each '.'. That rule covers every key name. Only the unit test for '.' checks it, while key_names_read_back already generates '.' and does not assert it. |
| R-60 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","D4. The revision is a hash of the layers that apply"],"Each entry is framed, so no two lists give the same input"] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | D4 says the framing gives two different layer lists two different inputs. That rule covers every list, but task 4.4 tests one hand-made pair. |
| R-61 | 3 | 3 | suggestion | in-change |  | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"A nested table merges key by key in `figment2`, so `[keys]` gets its rule from the library."] | fixed | cd948b212f17afd387a0e0119bb305c81e031977 | 3 |  |  | D2 states as settled that figment2 merges a nested table key by key, which the design lists as an unchecked third-party fact, and task 4.3 builds on it. |
