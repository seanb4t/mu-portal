---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-23T17:20:23+00:00
head: 00dc0995a2d309a82053d7d5fe23f0472b5b4e84
round: 1
next_id: 33
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
| R-1 | approach | blocker | in-change | A config_changed sent while a client's stream is lost is never seen: the stream resumes silently, and no decision has the client fetch GetClientConfig again on Resumed or Reset. | design.md > D5. The API — "SubscribeResponse.config_changed {}     (not in the journal, no sequence number)" | In D8, have the client call GetClientConfig after each Resumed and Reset event as well as on config_changed, and name the new ClientEvent variant that mu-client yields for config_changed. |  | severity? should-fix 0.80 | unmet-requirement |
| R-2 | approach | blocker | in-change | The reload must report a bad server.toml, but D2 keeps the server.toml parser in mu-cli and the store holds only its bytes, and mu-server cannot depend on mu-cli. No decision checks server.toml at reload. | specs/config-store/spec.md > Requirement: mu config reload reads the directory again — "The command SHALL print one line for each bad file to stderr." | Decide in D2 how a reload checks server.toml: move ServerFile to a crate mu-server can reach, pass a check function from mu-cli into the store, or state that a reload does not check server.toml and amend the requirement. |  |  | unmet-requirement |
| R-3 | approach,invariants | blocker | in-change | app-3: The server must check device names (layer file names, and the device of GetClientConfig), but the design puts the device-name rule only in mu-cli. D1 gives mu-config the key grammar and entry names, not the device, kind or OS names.; inv-6: The server-side check of the kind, the OS and the device of GetClientConfig has no task and no test. Only the CLI checks the device name, and remote callers reach the API directly. | specs/config-store/spec.md > Requirement: GetClientConfig resolves one client — "A bad name SHALL be an error that names it." | app-3: Add the device-name rule, the kind list and the OS names to mu-config in D1, used by mu-cli for the flag and by the store for layer names and GetClientConfig.; inv-6: Add the check to task 4.2 or 5.1, with a mu-server test that refuses the kind tiu, the OS windows, the device my laptop and a 65-character device, each error naming the value. |  |  | unmet-requirement |
| R-4 | approach | blocker | in-change | The connect_wait requirement and the scenario A short wait apply to mu config show, but only D7 (mu client) applies connect_wait. D6 gives mu config the flags and nothing about the wait. | specs/client-bootstrap/spec.md > Requirement: The connect_wait option — "The query option `connect_wait` SHALL be the longest time, in seconds, that a client waits for a server to answer." | State in D6 that each call of mu config show and mu config reload waits at most connect_wait, and that the timeout message names the socket and the wait. |  | severity? should-fix 0.84 | unmet-requirement |
| R-5 | invariants | blocker | in-change | The rule of workspace_boundary.rs refuses sha2 in mu-server: its ALLOWED_CRATES row has no sha2, and no task adds it or a workspace dependency entry. | crates/mu-cli/tests/workspace_boundary.rs:70 — ""mu-server"," | Add to a task of section 4 the entry of sha2 in [workspace.dependencies] and in the mu-server row of ALLOWED_CRATES, and name it in the Enforcement table. |  | scope? beyond-scope 0.85; severity? should-fix 0.69 | unmet-requirement |
| R-6 | invariants | blocker | in-change | The rule of workspace_boundary.rs refuses url in mu-cli: its ALLOWED_CRATES row has no url, and no task adds it or a workspace dependency entry. | crates/mu-cli/tests/workspace_boundary.rs:150 — ""vt100"," | Add to task 5.2 the entry of url in [workspace.dependencies] and in the mu-cli row of ALLOWED_CRATES. |  | severity? should-fix 0.73 | unmet-requirement |
| R-7 | approach | should-fix | in-change | D2 resolves the worlds from the in-memory snapshot with mu_core::world::resolve, as mu-cli does. At head, resolve reads through a LayerSource, and the only source, TomlDirSource, reads the disk itself. So the worlds and the snapshot come from two reads, and a file edited between them is compared against bytes that never ran. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "From the snapshot it resolves the worlds with `mu_core::world::resolve`, as `mu-cli` does today." | Name in D2/D3 a LayerSource over the snapshot in mu-core (path and bytes), or state that the store resolves from disk and record the double read as a risk. |  |  | wrong-behaviour |
| R-8 | approach | should-fix | in-change | D3 says the existing stray-file rule yields an error that points to clients/global.toml with no new path. At head the stray-file error is the fixed list of layer files, so the pointer the Migration Plan promises needs a new branch. | design.md > D3. The world scan skips `clients/` and rejects `client.toml` — "This uses the existing stray-file rule, so no new error path is needed." | Say in D3 that key_of gets a branch for a top-level client.toml whose reason names clients/global.toml, or drop the new path from the Migration Plan. |  |  | wrong-behaviour |
| R-9 | approach | should-fix | in-change | The Migration Plan moves client.toml to clients/global.toml with no rollback step. The binary at head treats any file under clients/ as a stray file, so after the move a downgrade stops mu server and mu world. | design.md > Migration Plan — "The user moves the content of `client.toml` to `clients/global.toml` and drops the table `[server]`." | Add a rollback step: to go back, move clients/global.toml to client.toml, add any [server] table again, and remove the clients directory. |  | severity? blocker 0.75 | wrong-behaviour |
| R-10 | fit | should-fix | in-change | The store hand-writes a layer merge with a source per field, which figment2 already does for the world layers; the design does not name figment2 or say why it does not fit, and #172 adds more fields to that merge. | tasks.md > 4. The store — "It merges the layers in order, field by field and key by key." | In D2, name figment2 (Named providers, merge, find_metadata) as the alternative for the client merge and either use it, or state the reason it does not fit (such as key names with a dot in find_metadata paths) and how #172 adds a field. |  | scope? deviates-design 0.62 | next-change |
| R-11 | fit | should-fix | in-change | mu-config gets its own TOML parse that names the line of an error, a copy of mu_core::world::parse_toml and its line-and-column helper, which also keeps the rule never to quote the line. | tasks.md > 2. The crate mu-config — "The error names the binding or the field. If TOML gives a line, the error names it too." | Parse each client layer in the store of mu-server with mu_core::world::parse_toml into the serde types of mu-config, and keep in mu-config only the types and the check of a parsed layer. |  |  | drift |
| R-12 | fit | should-fix | in-change | The store adds a second reader of the configuration directory (its own walk and sort into worlds, server.toml and clients/) beside TomlDirSource, whose walk and top-file rule read the same tree; resolve takes only a LayerSource, and the design does not say how the snapshot reaches it. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "At the start, the store reads every `.toml` file under the configuration directory into a snapshot: each relative path w" | Give mu-core one reader: TomlDirSource takes the snapshot of path and bytes and is the LayerSource over it, and the store and mu world use that reader, so the rule of which file is a world, a top file or a client layer lives in dir.rs alone. |  |  | drift |
| R-13 | fit,invariants | should-fix | in-change | fit-4: D1 keeps two tables of the entries, one in mu-config with the entries that need an argument and the registry in mu-tui with its own required flag; the guard test compares names only, so the argument flag can drift.; inv-5: The parity test compares entry names only. If the needs-an-argument flag differs between mu-config and the registry, the server accepts a binding to an entry that refuses to run without its argument. | design.md > D1. A new crate, `mu-config`, holds the client fields and the key grammar — "It also holds the table of entry names, with the entries that need an argument." | fit-4: Make the mu-config table the one source of each entry name and its argument rule, and have the registry of mu-tui map each mu-config entry to its description and its build function, for example by an exhaustive match on a mu-config enum.; inv-5: Make the_entry_table_matches_the_registry compare each name with its required-argument flag, and check in task 2.3 that it fails when only the flag differs. |  | severity? blocker 0.81 | drift |
| R-14 | fit | should-fix | in-change | The binding check of ClientConfig::keymap in mu-tui (unknown entry, entry that needs an argument, two names of one key) stays, and task 2.2 adds the same check in mu-config; no task removes the mu-tui copy or ClientConfig. | crates/mu-tui/src/config.rs:102 — "pub fn keymap(&self) -> Result<Keymap, KeyError> {" | Add to task 2.3 or 6.1 the removal of ClientConfig, ServerTable and the check in ClientConfig::keymap from crates/mu-tui/src/config.rs, so the client builds its keymap from the checked bindings that GetClientConfig returns. |  |  | drift |
| R-15 | invariants | should-fix | in-change | A reload between the GetClientConfig call at the start and the first subscribe of the stream is never seen by that client: config_changed goes only to open streams, is not journaled, and the stream opens after the screen and the terminal probe. | design.md > D7. The start of `mu client` — "Make the first list and the call `GetClientConfig`, both within one wait of `connect_wait`." | In D8, make the client call GetClientConfig on each Hello of its stream as well as on config_changed, and add a client.rs test that reloads after the start call and before the first subscribe. |  |  | wrong-behaviour |
| R-16 | invariants | should-fix | in-change | None of the listed refusals catches unix:/tmp/s.sock, which has no authority. The spec requires unix:// before the path, and task 1.1 does not parse this form. | specs/client-bootstrap/spec.md > Requirement: A URL names a Unix socket — "A server URL SHALL be `unix://`, then an absolute path, then an optional query." | Add unix:/tmp/s.sock to the parses of task 1.1, decide whether D6 refuses it (no authority), and add the matching bootstrap.rs test. |  |  | unmet-requirement |
| R-17 | invariants | should-fix | in-change | D3 skips the directory clients without saying top level only. The scan recurses, and no test covers a game named clients, so characters/clients/<c>.toml could vanish with no error. | design.md > D3. The world scan skips `clients/` and rejects `client.toml` — "drops `client.toml` from the top-level files and skips the directory `clients`." | Say top level in D3, and add to task 3.1 a world_config.rs test with games/clients.toml and characters/clients/cy.toml that lists clients/cy. |  |  | wrong-behaviour |
| R-18 | invariants | should-fix | in-change | A config_changed sent while a client's stream is lost and waiting to resubscribe, or a change of client layers across a server restart, is never replayed, so the client keeps an old revision until the next reload. | crates/mu-client/src/lib.rs:46 — "/// The stream ended. The client subscribes again by itself." | Make the client fetch GetClientConfig again on ClientEvent::Resumed and ClientEvent::Reset (the revision compare keeps it cheap), state it in D8, and add a test that reloads while the stream is down. |  |  | wrong-behaviour |
| R-19 | invariants | should-fix | in-change | D4 hashes each path with its bytes but names no framing, so removing one layer and appending its path and bytes to the previous layer gives the same input and the same revision. | design.md > D4. The revision is a hash of the layers that apply — "Its input is the sorted list of the layer files that exist for the kind, the OS and the device of the client, each path " | State in D4 a length prefix (for example a u64 of each length) before each path and each byte string, and add a mu-server unit test that a removed layer always changes the revision. |  |  | unmet-requirement |
| R-20 | invariants | should-fix | in-change | D4 does not say what the hash takes for a layer that cannot be read. If it takes no bytes, an empty layer that turns unreadable keeps its revision, and the client then shows no notice for it. | design.md > D4. The revision is a hash of the layers that apply — "A bad layer counts too, so a layer that turns bad changes the revision." | Name in D4 a distinct input for an unreadable layer, and add a unit test that an empty layer and the same path at mode 000 give different revisions. |  |  | unmet-requirement |
| R-21 | invariants | should-fix | in-change | No task checks the name of a devices/<name>.toml file against the device grammar, and no test covers one. On a case-insensitive file system, devices/Laptop.toml then applies to no device and gives no notice. | specs/config-store/spec.md > Requirement: The layer files of a client — "A device SHALL be a valid device name." | In task 4.1, check each device file name with the same function as the CLI device grammar, and add a config_store.rs test with clients/devices/Laptop.toml that expects a stderr line naming it. |  |  | wrong-behaviour |
| R-22 | invariants | should-fix | in-change | The device name length rule of 1 to 64 has no test at 64, at 65 or for an empty --device. The only bad-name test is a name with a space. | specs/client-bootstrap/spec.md > Requirement: The device name — "A device name SHALL have 1 to 64 characters from `a` to `z`, `0` to `9`, `-`, `_` and `.`." | Add bootstrap.rs or unit tests: a 64-character name is accepted, a 65-character name and --device with an empty value are refused with exit status 1. |  |  | unmet-requirement |
| R-23 | invariants | should-fix | in-change | The refusal of a query option that appears twice has no test. Task 1.1 only parses it in a scratch test. | specs/client-bootstrap/spec.md > Requirement: The connect_wait option — "So SHALL a query option with any other name, and an option that is there twice." | Add a bootstrap.rs test with ?connect_wait=1&connect_wait=2 that expects exit status 1 and connect_wait on stderr. |  |  | unmet-requirement |
| R-24 | spec-lint | should-fix | in-change | The proposal marks three changes BREAKING, but no delta spec carries a BREAKING marker. | proposal.md > What Changes — "**BREAKING** `client.toml` is removed. `mu client` no longer takes `--config` and reads no configuration file." | Add the BREAKING marker to the REMOVED and MODIFIED requirements that carry the proposal's BREAKING items, or drop the marker from the proposal. |  |  | next-change |
| R-25 | spec-lint | should-fix | in-change | The requirement 'The configuration directory' has no task line that names its subject; only the word 'directory' matches. Its new scenarios are named in tasks 3.1 and 6.1 only. | specs/world-config/spec.md > Requirement: The configuration directory — "The flag `--config <dir>` of `mu server` and `mu world` SHALL name the configuration directory" | Name the subject in a task line, such as the flag change of mu server and mu world, or a task for the MU_CONFIG resolution. |  |  | unmet-requirement |
| R-26 | test-coverage | should-fix | in-change | Five scenarios of the modified requirement 'Connect and disconnect a world' are named by no task; only the full-suite run in 9.2 reaches them. | tasks.md > 6.2 — "`a_key_bound_to_connect_with_no_world_in_focus`" | In task 6.1 or 6.2 name the client.rs tests for Connect the world in focus, Disconnect a named world, Connect with no world in focus, A trailing space is no argument and Disconnect an unknown world, since their GIVEN steps now run on the URL bootstrap. |  |  | unmet-requirement |
| R-27 | test-coverage | should-fix | in-change | The eight terminal-client scenarios are covered only by 'the tests of the requirement still pass'; no test is named per scenario. | tasks.md > 6.3 — "Make sure that the tests of the requirement "The top row shows the first list" in `crates/mu-tui/tests/app.rs` still pas" | Name the app.rs test for each scenario (Before the first list ... The first list fails) in task 6.3, and say which ones change with connect_wait. |  |  | unmet-requirement |
| R-28 | test-coverage | should-fix | in-change | Seven world-config scenarios (flag wins, default dir, XDG, empty MU_CONFIG, no HOME, missing dir, character files and others) are covered only by 'every other test in world_config.rs passes'. | tasks.md > 3.1 — "and that every other test in `world_config.rs` passes" | In task 3.1 list the world_config.rs test for each of the modified world-config scenarios, or state which existing tests stay unchanged. |  |  | unmet-requirement |
| R-29 | test-coverage | should-fix | in-change | The BREAKING removal of --socket and --config from mu client has no scenario; only MU_SOCKET and the client takes the flag are covered. | proposal.md > Bootstrap — "`mu client` no longer takes `--config` and reads no configuration file." | Add a scenario to client-bootstrap that mu client --socket <path> is refused with a message naming --server, and name its test in task 6.1. |  |  | unmet-requirement |
| R-30 | approach | suggestion | in-change | D5 keeps ListWorlds.config_dir for a remote client or MCP server, but no requirement needs it and the proposal puts the remote client out of scope. | design.md > D5. The API — "`ListWorlds.config_dir` stays. `mu config show` does not need it, but a remote client or an MCP server can use it to nam" | Tie the field to a requirement, or leave the choice to #175 and say so in D5. |  |  |  |
| R-31 | fit | suggestion | in-change | mu config show prints TOML with a source comment on each line, the same form that render in world.rs writes for mu world show; the two vary only in their rows. | tasks.md > 5. The API and the bootstrap values — "`show` prints the TOML with a source comment on each field and binding, and the revision on its first line." | Write one helper in mu-cli that renders rows of table, key, TOML value and source, with the rows as its parameter, and call it from mu world show and mu config show. |  |  |  |
| R-32 | invariants | suggestion | in-change | The rule that two names of one key are one key is order independence over all modifier sets, and the merge relies on it. D1 allows mu-config only serde and toml, so the existing proptest cannot move with the grammar. | crates/mu-tui/tests/keymap.rs:260 — "fn key_names_read_back(" | Keep a proptest (proptest 1.12, as in crates/mu-tui/tests/keymap.rs) over the neutral key value of mu-config: allow proptest as a dev crate of mu-config, or name in task 2.1 that key_names_read_back stays in mu-tui and runs through the mu-config parser. |  |  |  |

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

## Judge

judge: 60 questions, 0.003 USD, stopped: cap of 60 questions

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
| A relative path | 5.2 | covered |  |
| A wait of zero | 5.2 | covered |  |
| An unknown option | 5.2 | covered |  |
| A short wait | 5.2 | covered |  |
| The host name | 5.2 | covered |  |
| A bad name | 5.2 | covered |  |
| The OS layer applies | 6.2 | covered |  |
| Connect the world in focus |  | partial |  |
| Disconnect a named world |  | partial |  |
| A key bound to connect with no world in focus | 6.2 | covered |  |
| Connect with no world in focus |  | partial |  |
| A trailing space is no argument |  | partial |  |
| Disconnect an unknown world |  | partial |  |
| A server table | 2.2, 5.4 | covered |  |
| An unknown field | 2.2, 5.4 | covered |  |
| Not TOML | 2.2, 5.4 | covered |  |
| A file that cannot be read | 5.4 | covered |  |
| Two names of one key | 2.2, 5.4 | covered |  |
| An uppercase key | 2.2, 5.4 | covered |  |
| A control character | 2.2, 5.4 | covered |  |
| Shift with a digit | 2.2, 5.4 | covered |  |
| An unknown entry | 2.2, 5.4 | covered |  |
| An unknown key name | 2.2, 5.4 | covered |  |
| An entry that needs an argument | 2.2, 5.4 | covered |  |
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
| A bad device layer at start | 4.1 | covered |  |
| A file that is not a layer | 4.1 | covered |  |
| A layer for an unknown kind | 4.1 | covered |  |
| No clients directory | 5.3 | covered |  |
| A device with no file | 5.3 | covered |  |
| The device wins | 4.2, 5.3 | covered |  |
| Keys merge key by key | 4.2, 5.3 | covered |  |
| Two names of one key in two layers | 5.3 | covered |  |
| One bad binding skips its whole layer | 5.3 | covered |  |
| A notice for a bad layer that applies | 5.3 | covered |  |
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
| Before the first list | 6.3 | partial |  |
| A send before the first list | 6.3 | partial |  |
| Text typed before the first list | 6.3 | partial |  |
| An event before the first list | 6.3 | partial |  |
| A focus key before the first list | 6.3 | partial |  |
| A failed first list after an event | 6.3 | partial |  |
| A notice before the first list | 6.3 | partial |  |
| The first list fails | 6.3 | partial |  |
| The flag wins over the environment | 3.1 | partial |  |
| The default directory | 3.1 | partial |  |
| The directory under the home directory | 3.1 | partial |  |
| An empty MU_CONFIG | 3.1 | partial |  |
| A relative XDG_CONFIG_HOME | 3.1 | partial |  |
| No home directory | 3.1 | partial |  |
| A missing directory | 3.1 | partial |  |
| A missing default directory | 3.1 | partial |  |
| The client warns of a missing directory | 6.1 | covered |  |
| The client takes the flag | 6.1 | covered |  |
| Worlds from character files | 3.1 | partial |  |
| A character with no game | 3.1 | partial |  |
| A file name outside the id rule | 3.1 | partial |  |
| A stray file | 3.1 | partial |  |
| A link to nothing | 3.1 | partial |  |
| The file of the server | 3.1 | partial |  |
| A character named server | 3.1 | partial |  |
| The file of the client | 3.1 | covered |  |
| A bad client layer does not stop the worlds | 3.1 | covered |  |
| A leftover client.toml | 3.1 | covered |  |

### Not run

Plan phase: no verification runs.

## Verification runs

None.

## Dropped

None.

## Lens notes

- approach: D4 and D6 bring sha2 into mu-server and url into mu-cli. At head, neither is in ALLOWED_CRATES of workspace_boundary.rs or in [workspace.dependencies], and task 2.1 edits the crate rules for mu-config only. Cargo.lock does hold url 2.5.9 and sha2 0.10.9.
- approach: Task 4.1 moves resolve out of mu-cli, but mu-cli still builds the WorldSetups (TcpTransport, secrets, fake worlds from mu-fake-server, which mu-server cannot depend on). D2 does not say how the store gives mu-cli the resolved worlds before the socket opens.
- approach: Context says mu-tui parses client.toml. At head mu-tui holds the serde types, and mu-cli reads and parses the file (crates/mu-tui/src/config.rs lines 1-2).
- approach: Verified at head: Chord::parse in keymap.rs, the entry registry in registry.rs, mu_tui::run(client, settings, notices), rustix::system behind the system feature in rustix 1.1.6, rustix as a workspace dependency, url and sha2 in Cargo.lock, and the TOP_FILES scan in dir.rs.
- approach: The third-party facts that design.md lists as unchecked (url parsing, nodename output, sha2 calls) are checked by tasks 1.1 to 1.3 and are not judged here.
- fit: D6 and D4 name url and sha2 as crates already in Cargo.lock, but no task adds them to [workspace.dependencies] or to ALLOWED_CRATES of mu-cli and mu-server; the boundary test will catch it at implementation.
- fit: The existing connect_wait check in crates/mu-cli/src/client.rs:56 states the same rule as the URL option; task 5.2 should call it rather than write a second one.
- fit: The device name rule differs from PART_RULE in crates/mu-core/src/id.rs:47 on purpose (host names), so it is not a copy.
- fit: D8 fits the existing Msg and Effect pattern of crates/mu-tui/src/update.rs, and D7 reuses the existing notices of mu_tui::run.
- invariants: I1: A bad client layer never stops the server \| enforced at the store of task 4.1 (planned) \| boundary test a_bad_device_layer_at_start, a_file_that_is_not_a_layer, a_layer_for_an_unknown_kind
- invariants: I2: No field or binding of a bad layer applies \| enforced at the resolution of task 4.2 (planned) \| boundary test a_bad_layer_adds_nothing, one_bad_binding_skips_its_whole_layer
- invariants: I3: Two names of one key are one key, in one layer and across layers \| enforced at the neutral key value of mu-config (planned) \| boundary test two_names_of_one_key_in_one_layer, two_names_across_layers_are_one_key; property see inv-12
- invariants: I4: The revision changes on any change, addition or removal of a layer that applies, and is stable across a restart \| enforced at D4 (planned) \| boundary test the_same_files_give_the_same_revision, a_changed_layer_gives_a_new_revision; removal and framing see inv-3, unreadable see inv-4
- invariants: I5: GetClientConfig sees the old or the new layers, never a mix \| enforced at the single swap of D2 \| boundary test none; a concurrency test is hard and the swap of one value enforces it by construction
- invariants: I6: Every attached client converges to its current revision \| enforced at config_changed of D5 \| boundary test a_new_binding_works_without_a_restart; gaps see inv-1 and inv-2
- invariants: I7: The entry table of mu-config equals the registry, with the argument flag \| enforced at the_entry_table_matches_the_registry (planned) \| boundary test none for the flag, see inv-5
- invariants: I8: A device name has 1 to 64 characters of [a-z0-9-_.] \| enforced at the parser of task 5.2 (planned), not on the server \| boundary test a_bad_name only, see inv-6, inv-7 and inv-8
- invariants: I9: connect_wait is finite, at least 1 ns, given once, and no other option exists \| enforced at connect_wait in crates/mu-cli/src/client.rs:56 (try_from_secs_f64, refuses f64::MAX) \| boundary test a_wait_that_is_no_time_is_refused at client.rs:264 must stay when the URL parser replaces the file read; repeated option see inv-9
- invariants: I10: A server URL is unix:// with an absolute path, no host, user, password or fragment \| enforced at the parser of task 5.2 (planned) \| boundary test another_scheme, a_host, a_relative_path; unix:/ form see inv-10
- invariants: I11: The world scan ignores the top-level clients directory and refuses client.toml \| enforced at dir.rs per D3 (planned) \| boundary test the_file_of_the_client, a_leftover_client_toml; nested name see inv-11
- invariants: I12: A reload never applies a world or server.toml change \| enforced at the reload of D2 \| boundary test a_reload_does_not_apply_a_world_change, a_reload_with_a_bad_world_file
- invariants: I13: mu config show and reload never start a server \| enforced at D6 \| boundary test no_server asserts no socket file
- invariants: I14: Each member uses only its allowed crates \| enforced at workspace_boundary.rs ALLOWED_CRATES \| see inv-13 and inv-14
- invariants: The existing top files match in any case and by same-file (dir.rs:111). D3 does not say whether the clients directory follows that rule, so a Clients directory on macOS would be a stray-file error for the world scan.
- invariants: No removed or modified requirement drops a check: the cases of the removed requirement A bad binding stops the client, and the wait rule of the removed server table, all reappear in the added requirements.
- spec-lint: The requirement 'A bad binding makes its layer bad' (config-store) lists a repeated modifier as a bad case, and no scenario and no task line names it.
- spec-lint: The scenario 'A binding of Shift+Tab' sits under the requirement 'A bad binding makes its layer bad', but it tests a valid binding.
- spec-lint: Build targets checked: cargo packages mu-cli, mu-server, mu-tui, mu-core and mu-proto exist; mise tasks check and docs:check exist; the test files and docs pages named in tasks exist or are marked new. No unknown-target finding.
- spec-lint: Every task line names a command, a test, a file or an observable result. Every requirement has at least one scenario with WHEN or GIVEN and THEN.
- test-coverage: Tasks name tests and rely on the one-test command of openspec/config.yaml as the verification command; only 2.1 writes a literal command. Graded covered on that basis.
- test-coverage: The scenario headings 'The variable' and 'The flag wins' occur twice in client-bootstrap; the matrix has one row each.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | fixed_after | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D5. The API"],"SubscribeResponse.config_changed {} (not in the journal, no sequence number)"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  | severity? should-fix 0.80 | A config_changed sent while a client's stream is lost is never seen: the stream resumes silently, and no decision has the client fetch GetClientConfig again on Resumed or Reset. |
| R-2 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu config reload reads the directory again"],"The command SHALL print one line for each bad file to stderr."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The reload must report a bad server.toml, but D2 keeps the server.toml parser in mu-cli and the store holds only its bytes, and mu-server cannot depend on mu-cli. No decision checks server.toml at reload. |
| R-3 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: GetClientConfig resolves one client"],"A bad name SHALL be an error that names it."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | app-3: The server must check device names (layer file names, and the device of GetClientConfig), but the design puts the device-name rule only in mu-cli. D1 gives mu-config the key grammar and entry names, not the device, kind or OS names.; inv-6: The server-side check of the kind, the OS and the device of GetClientConfig has no task and no test. Only the CLI checks the device name, and remote callers reach the API directly. |
| R-4 | 1 | 1 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The connect_wait option"],"The query option `connect_wait` SHALL be the longest time, in seconds, that a client waits for a server to answer."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  | severity? should-fix 0.84 | The connect_wait requirement and the scenario A short wait apply to mu config show, but only D7 (mu client) applies connect_wait. D6 gives mu config the flags and nothing about the wait. |
| R-5 | 1 | 1 | blocker | in-change | unmet-requirement | ["path","crates/mu-cli/tests/workspace_boundary.rs",70] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  | scope? beyond-scope 0.85; severity? should-fix 0.69 | The rule of workspace_boundary.rs refuses sha2 in mu-server: its ALLOWED_CRATES row has no sha2, and no task adds it or a workspace dependency entry. |
| R-6 | 1 | 1 | blocker | in-change | unmet-requirement | ["path","crates/mu-cli/tests/workspace_boundary.rs",150] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  | severity? should-fix 0.73 | The rule of workspace_boundary.rs refuses url in mu-cli: its ALLOWED_CRATES row has no url, and no task adds it or a workspace dependency entry. |
| R-7 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"From the snapshot it resolves the worlds with `mu_core::world::resolve`, as `mu-cli` does today."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | D2 resolves the worlds from the in-memory snapshot with mu_core::world::resolve, as mu-cli does. At head, resolve reads through a LayerSource, and the only source, TomlDirSource, reads the disk itself. So the worlds and the snapshot come from two reads, and a file edited between them is compared against bytes that never ran. |
| R-8 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. The world scan skips `clients/` and rejects `client.toml`"],"This uses the existing stray-file rule, so no new error path is needed."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | D3 says the existing stray-file rule yields an error that points to clients/global.toml with no new path. At head the stray-file error is the fixed list of layer files, so the pointer the Migration Plan promises needs a new branch. |
| R-9 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","Migration Plan"],"The user moves the content of `client.toml` to `clients/global.toml` and drops the table `[server]`."] | refuted | mu-portal has no release (engram 3j32zhc4bj): breaking changes to configuration are fine and no downgrade path is owed |  |  | severity? blocker 0.75 | The Migration Plan moves client.toml to clients/global.toml with no rollback step. The binary at head treats any file under clients/ as a stray file, so after the move a downgrade stops mu server and mu world. |
| R-10 | 1 | 1 | should-fix | in-change | next-change | ["artifact",["tasks.md","4. The store"],"It merges the layers in order, field by field and key by key."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  | scope? deviates-design 0.62 | The store hand-writes a layer merge with a source per field, which figment2 already does for the world layers; the design does not name figment2 or say why it does not fit, and #172 adds more fields to that merge. |
| R-11 | 1 | 1 | should-fix | in-change | drift | ["artifact",["tasks.md","2. The crate mu-config"],"The error names the binding or the field. If TOML gives a line, the error names it too."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | mu-config gets its own TOML parse that names the line of an error, a copy of mu_core::world::parse_toml and its line-and-column helper, which also keeps the rule never to quote the line. |
| R-12 | 1 | 1 | should-fix | in-change | drift | ["artifact",["design.md","D2. The store lives in `mu-server` and keeps a snapshot"],"At the start, the store reads every `.toml` file under the configuration directory into a snapshot: each relative path with its bytes."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The store adds a second reader of the configuration directory (its own walk and sort into worlds, server.toml and clients/) beside TomlDirSource, whose walk and top-file rule read the same tree; resolve takes only a LayerSource, and the design does not say how the snapshot reaches it. |
| R-13 | 1 | 1 | should-fix | in-change | drift | ["artifact",["design.md","D1. A new crate, `mu-config`, holds the client fields and the key grammar"],"It also holds the table of entry names, with the entries that need an argument."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  | severity? blocker 0.81 | fit-4: D1 keeps two tables of the entries, one in mu-config with the entries that need an argument and the registry in mu-tui with its own required flag; the guard test compares names only, so the argument flag can drift.; inv-5: The parity test compares entry names only. If the needs-an-argument flag differs between mu-config and the registry, the server accepts a binding to an entry that refuses to run without its argument. |
| R-14 | 1 | 1 | should-fix | in-change | drift | ["path","crates/mu-tui/src/config.rs",102] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The binding check of ClientConfig::keymap in mu-tui (unknown entry, entry that needs an argument, two names of one key) stays, and task 2.2 adds the same check in mu-config; no task removes the mu-tui copy or ClientConfig. |
| R-15 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7. The start of `mu client`"],"Make the first list and the call `GetClientConfig`, both within one wait of `connect_wait`."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | A reload between the GetClientConfig call at the start and the first subscribe of the stream is never seen by that client: config_changed goes only to open streams, is not journaled, and the stream opens after the screen and the terminal probe. |
| R-16 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-bootstrap/spec.md","Requirement: A URL names a Unix socket"],"A server URL SHALL be `unix://`, then an absolute path, then an optional query."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | None of the listed refusals catches unix:/tmp/s.sock, which has no authority. The spec requires unix:// before the path, and task 1.1 does not parse this form. |
| R-17 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. The world scan skips `clients/` and rejects `client.toml`"],"drops `client.toml` from the top-level files and skips the directory `clients`."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | D3 skips the directory clients without saying top level only. The scan recurses, and no test covers a game named clients, so characters/clients/<c>.toml could vanish with no error. |
| R-18 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-client/src/lib.rs",46] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | A config_changed sent while a client's stream is lost and waiting to resubscribe, or a change of client layers across a server restart, is never replayed, so the client keeps an old revision until the next reload. |
| R-19 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D4. The revision is a hash of the layers that apply"],"Its input is the sorted list of the layer files that exist for the kind, the OS and the device of the client, each path with its bytes."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | D4 hashes each path with its bytes but names no framing, so removing one layer and appending its path and bytes to the previous layer gives the same input and the same revision. |
| R-20 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","D4. The revision is a hash of the layers that apply"],"A bad layer counts too, so a layer that turns bad changes the revision."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | D4 does not say what the hash takes for a layer that cannot be read. If it takes no bytes, an empty layer that turns unreadable keeps its revision, and the client then shows no notice for it. |
| R-21 | 1 | 1 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/config-store/spec.md","Requirement: The layer files of a client"],"A device SHALL be a valid device name."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | No task checks the name of a devices/<name>.toml file against the device grammar, and no test covers one. On a case-insensitive file system, devices/Laptop.toml then applies to no device and gives no notice. |
| R-22 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The device name"],"A device name SHALL have 1 to 64 characters from `a` to `z`, `0` to `9`, `-`, `_` and `.`."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The device name length rule of 1 to 64 has no test at 64, at 65 or for an empty --device. The only bad-name test is a name with a space. |
| R-23 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-bootstrap/spec.md","Requirement: The connect_wait option"],"So SHALL a query option with any other name, and an option that is there twice."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The refusal of a query option that appears twice has no test. Task 1.1 only parses it in a scratch test. |
| R-24 | 1 | 1 | should-fix | in-change | next-change | ["artifact",["proposal.md","What Changes"],"**BREAKING** `client.toml` is removed. `mu client` no longer takes `--config` and reads no configuration file."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The proposal marks three changes BREAKING, but no delta spec carries a BREAKING marker. |
| R-25 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Requirement: The configuration directory"],"The flag `--config <dir>` of `mu server` and `mu world` SHALL name the configuration directory"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The requirement 'The configuration directory' has no task line that names its subject; only the word 'directory' matches. Its new scenarios are named in tasks 3.1 and 6.1 only. |
| R-26 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","6.2"],"`a_key_bound_to_connect_with_no_world_in_focus`"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | Five scenarios of the modified requirement 'Connect and disconnect a world' are named by no task; only the full-suite run in 9.2 reaches them. |
| R-27 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","6.3"],"Make sure that the tests of the requirement \\"The top row shows the first list\\" in `crates/mu-tui/tests/app.rs` still pass"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The eight terminal-client scenarios are covered only by 'the tests of the requirement still pass'; no test is named per scenario. |
| R-28 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","3.1"],"and that every other test in `world_config.rs` passes"] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | Seven world-config scenarios (flag wins, default dir, XDG, empty MU_CONFIG, no HOME, missing dir, character files and others) are covered only by 'every other test in world_config.rs passes'. |
| R-29 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["proposal.md","Bootstrap"],"`mu client` no longer takes `--config` and reads no configuration file."] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The BREAKING removal of --socket and --config from mu client has no scenario; only MU_SOCKET and the client takes the flag are covered. |
| R-30 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","D5. The API"],"`ListWorlds.config_dir` stays. `mu config show` does not need it, but a remote client or an MCP server can use it to name the directory."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/176; reason: Suggestions with no behavior change: tie ListWorlds.config_dir to a requirement or drop it, and share one TOML-with-sources renderer between mu world show and mu config show |  |  |  | D5 keeps ListWorlds.config_dir for a remote client or MCP server, but no requirement needs it and the proposal puts the remote client out of scope. |
| R-31 | 1 | 1 | suggestion | in-change |  | ["artifact",["tasks.md","5. The API and the bootstrap values"],"`show` prints the TOML with a source comment on each field and binding, and the revision on its first line."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/176; reason: Suggestions with no behavior change: tie ListWorlds.config_dir to a requirement or drop it, and share one TOML-with-sources renderer between mu world show and mu config show |  |  |  | mu config show prints TOML with a source comment on each line, the same form that render in world.rs writes for mu world show; the two vary only in their rows. |
| R-32 | 1 | 1 | suggestion | in-change |  | ["path","crates/mu-tui/tests/keymap.rs",260] | fixed | 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The rule that two names of one key are one key is order independence over all modifier sets, and the merge relies on it. D1 allows mu-config only serde and toml, so the existing proptest cannot move with the grammar. |
