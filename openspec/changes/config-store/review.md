---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-23T17:36:44+00:00
head: 5cbf017513e9ae12df877428ff4d90c1dc61b993
round: 2
next_id: 43
lenses:
  spec-lint: ok
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
  verify: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-1 | verify | blocker | in-change | PARTIAL: the fix 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 of R-1 did not hold. A config_changed sent while a client's stream is lost is never seen: the stream resumes silently, and no decision has the client fetch GetClientConfig again on Resumed or Reset. | crates/mu-client/src/lib.rs:218 — "/// The next `Hello` follows a reset that this client made, so it yields nothing." | fix it again | regression | severity? should-fix 0.66 | wrong-behaviour |
| R-33 | test-coverage | should-fix | in-change | delta scenario 'No double slash' has no matrix row | review > absent-row — "No double slash" | add the row with its covering task or test |  |  | unmet-requirement |
| R-34 | test-coverage | should-fix | in-change | delta scenario 'An option twice' has no matrix row | review > absent-row — "An option twice" | add the row with its covering task or test |  |  | unmet-requirement |
| R-35 | test-coverage | should-fix | in-change | delta scenario 'A name of 64 characters' has no matrix row | review > absent-row — "A name of 64 characters" | add the row with its covering task or test |  |  | unmet-requirement |
| R-36 | test-coverage | should-fix | in-change | delta scenario 'A name of 65 characters' has no matrix row | review > absent-row — "A name of 65 characters" | add the row with its covering task or test |  |  | unmet-requirement |
| R-37 | test-coverage | should-fix | in-change | delta scenario 'An empty device flag' has no matrix row | review > absent-row — "An empty device flag" | add the row with its covering task or test |  |  | unmet-requirement |
| R-38 | test-coverage | should-fix | in-change | delta scenario 'The socket flag' has no matrix row | review > absent-row — "The socket flag" | add the row with its covering task or test |  |  | unmet-requirement |
| R-39 | test-coverage | should-fix | in-change | delta scenario 'A device file with a bad name' has no matrix row | review > absent-row — "A device file with a bad name" | add the row with its covering task or test |  |  | unmet-requirement |
| R-40 | test-coverage | should-fix | in-change | delta scenario 'A bad device name in a call' has no matrix row | review > absent-row — "A bad device name in a call" | add the row with its covering task or test |  |  | unmet-requirement |
| R-41 | test-coverage | should-fix | in-change | delta scenario 'A reload while the stream is down' has no matrix row | review > absent-row — "A reload while the stream is down" | add the row with its covering task or test |  |  | unmet-requirement |
| R-42 | test-coverage | should-fix | in-change | delta scenario 'A game named clients' has no matrix row | review > absent-row — "A game named clients" | add the row with its covering task or test |  |  | unmet-requirement |
| R-10 | verify | should-fix | in-change | D2 takes the source of each binding from figment2 find_metadata, which splits its key path on '.'. The key grammar accepts '.' as a key, and its canonical name keeps the dot, so a binding of '.' or 'alt+.' gets no source or the source of the whole [keys] table. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "Each layer is a `Named` provider, the merge is `Figment::merge`, and the source of a field is `find_metadata`." | In D2, take the source of a binding from the [keys] dict of each layer, not from find_metadata on a dotted path, or name the escape of a key name that holds a dot. Add a test of a binding of alt+. to task 4.3. | regression |  | wrong-behaviour |
| R-12 | verify | should-fix | in-change | D2 builds DirSnapshot with the walk of TomlDirSource, which fails the whole read on any entry it cannot stat. So a link to nothing under clients/ stops the start, where a bad client layer must be skipped. D3 also puts the skip of clients/ in the same scan, and neither says the snapshot still keeps the clients/ files. | design.md > D2. The store lives in `mu-server` and keeps a snapshot — "It walks the directory with the walk and the top-file rule of `TomlDirSource`, and keeps each relative path with its byt" | State in D2 that the snapshot walks clients/ in full and keeps an entry that cannot be stat'ed or read as an unreadable file (tag 1 of D4), and that the skip of D3 applies to the world layers, not to the snapshot walk. Add a test of a link to nothing under clients/devices to task 4.2. | regression |  | unmet-requirement |
| R-18 | verify | should-fix | in-change | PARTIAL: the fix 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 of R-18 did not hold. A config_changed sent while a client's stream is lost and waiting to resubscribe, or a change of client layers across a server restart, is never replayed, so the client keeps an old revision until the next reload. | crates/mu-client/src/lib.rs:284 — "self.quiet_hello = true;" | fix it again | regression |  | wrong-behaviour |
| R-32 | verify | suggestion | in-change | PARTIAL: the fix 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 of R-32 did not hold. The rule that two names of one key are one key is order independence over all modifier sets, and the merge relies on it. D1 allows mu-config only serde and toml, so the existing proptest cannot move with the grammar. | design.md > D1. A new crate, `mu-config`, holds the client fields and the key grammar — "Its only allowed third-party crate is `serde`." | fix it again | regression | severity? should-fix 0.64 | next-change |

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

## Judge

judge: 10 questions, 0.000 USD

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

- verify: R-1 and R-18 are PARTIAL: after a server restart, mu-client drops the stream that opened past event 1, yields Reset, subscribes again from 1 and keeps the next Hello quiet. A config_changed sent in that gap, after the fetch that the Reset starts, is never seen, against D5's claim that every Hello makes the client compare.
- verify: R-32 is PARTIAL: D1 and task 2.1 move key_names_read_back with proptest as a dev-dependency, but give mu-config serde as its only crate. ALLOWED_CRATES in workspace_boundary.rs applies to every dependency table, dev-dependencies too, so the check of task 2.1 fails unless proptest joins the mu-config row.
- verify: R-9, R-30 and R-31 are closed rows (refuted or deferred) and have no basis, so this round gives them no fixes entry.
- verify: The fix commit 63f4f6c also rewrote review.md lines 1 to 269. This agent does not read review.md, so those lines were not checked.
- verify: Every test name that tasks 3.3, 6.3 and 6.4 cite exists at head in crates/mu-cli/tests/world_config.rs, client.rs and crates/mu-tui/tests/app.rs. The_socket_flag and the new config_store.rs and bootstrap.rs tests do not exist yet; the tasks add them.
- verify: mu_core::world::parse_toml is generic over T: DeserializeOwned at head, so D2 can parse a client layer into the types of mu-config.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | fixed_after | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | wrong-behaviour | ["artifact",["design.md","D5. The API"],"SubscribeResponse.config_changed {} (not in the journal, no sequence number)"] | fixed | 30253ffee5f0416968dc01cb22997ee738fa1972 | 2 |  | severity? should-fix 0.66 | A config_changed sent while a client's stream is lost is never seen: the stream resumes silently, and no decision has the client fetch GetClientConfig again on Resumed or Reset. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: mu config reload reads the directory again"],"The command SHALL print one line for each bad file to stderr."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | The reload must report a bad server.toml, but D2 keeps the server.toml parser in mu-cli and the store holds only its bytes, and mu-server cannot depend on mu-cli. No decision checks server.toml at reload. |
| R-3 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/config-store/spec.md","Requirement: GetClientConfig resolves one client"],"A bad name SHALL be an error that names it."] | resolved | resolved in round 2 from fixed 63f4f6c562f29dfab27ce9a7f67bac1cd6a26565 | 1 |  |  | app-3: The server must check device names (layer file names, and the device of GetClientConfig), but the design puts the device-name rule only in mu-cli. D1 gives mu-config the key grammar and entry names, not the device, kind or OS names.; inv-6: The server-side check of the kind, the OS and the device of GetClientConfig has no task and no test. Only the CLI checks the device name, and remote callers reach the API directly. |
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
