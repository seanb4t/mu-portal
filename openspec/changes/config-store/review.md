---
verdict: approve-with-notes
phase: plan
reviewed_at: 2026-11-23T17:55:49+00:00
head: fa7bcb23a3adf12bfe8cbf119449d5f4810ca2a6
round: 4
next_id: 82
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
| R-62 | test-coverage | should-fix | in-change | delta scenario 'The server flag wins' has no matrix row | review > absent-row — "The server flag wins" | add the row with its covering task or test |  |  | unmet-requirement |
| R-63 | test-coverage | should-fix | in-change | delta scenario 'The server variable' has no matrix row | review > absent-row — "The server variable" | add the row with its covering task or test |  |  | unmet-requirement |
| R-64 | test-coverage | should-fix | in-change | delta scenario 'An empty server variable' has no matrix row | review > absent-row — "An empty server variable" | add the row with its covering task or test |  |  | unmet-requirement |
| R-65 | test-coverage | should-fix | in-change | delta scenario 'A user' has no matrix row | review > absent-row — "A user" | add the row with its covering task or test |  |  | unmet-requirement |
| R-66 | test-coverage | should-fix | in-change | delta scenario 'A password' has no matrix row | review > absent-row — "A password" | add the row with its covering task or test |  |  | unmet-requirement |
| R-67 | test-coverage | should-fix | in-change | delta scenario 'A fragment' has no matrix row | review > absent-row — "A fragment" | add the row with its covering task or test |  |  | unmet-requirement |
| R-68 | test-coverage | should-fix | in-change | delta scenario 'A negative wait' has no matrix row | review > absent-row — "A negative wait" | add the row with its covering task or test |  |  | unmet-requirement |
| R-69 | test-coverage | should-fix | in-change | delta scenario 'A wait that is not a number' has no matrix row | review > absent-row — "A wait that is not a number" | add the row with its covering task or test |  |  | unmet-requirement |
| R-70 | test-coverage | should-fix | in-change | delta scenario 'An infinite wait' has no matrix row | review > absent-row — "An infinite wait" | add the row with its covering task or test |  |  | unmet-requirement |
| R-71 | test-coverage | should-fix | in-change | delta scenario 'A wait under one nanosecond' has no matrix row | review > absent-row — "A wait under one nanosecond" | add the row with its covering task or test |  |  | unmet-requirement |
| R-72 | test-coverage | should-fix | in-change | delta scenario 'The device variable' has no matrix row | review > absent-row — "The device variable" | add the row with its covering task or test |  |  | unmet-requirement |
| R-73 | test-coverage | should-fix | in-change | delta scenario 'The device flag wins' has no matrix row | review > absent-row — "The device flag wins" | add the row with its covering task or test |  |  | unmet-requirement |
| R-74 | test-coverage | should-fix | in-change | delta scenario 'An empty device variable' has no matrix row | review > absent-row — "An empty device variable" | add the row with its covering task or test |  |  | unmet-requirement |
| R-75 | test-coverage | should-fix | in-change | delta scenario 'An unlistable clients directory' has no matrix row | review > absent-row — "An unlistable clients directory" | add the row with its covering task or test |  |  | unmet-requirement |
| R-76 | test-coverage | should-fix | in-change | delta scenario 'A control character in a layer name' has no matrix row | review > absent-row — "A control character in a layer name" | add the row with its covering task or test |  |  | unmet-requirement |
| R-77 | test-coverage | should-fix | in-change | delta scenario 'A bad kind in a call' has no matrix row | review > absent-row — "A bad kind in a call" | add the row with its covering task or test |  |  | unmet-requirement |
| R-78 | test-coverage | should-fix | in-change | delta scenario 'A bad OS in a call' has no matrix row | review > absent-row — "A bad OS in a call" | add the row with its covering task or test |  |  | unmet-requirement |
| R-79 | test-coverage | should-fix | in-change | delta scenario 'An unknown OS flag' has no matrix row | review > absent-row — "An unknown OS flag" | add the row with its covering task or test |  |  | unmet-requirement |
| R-80 | test-coverage | should-fix | in-change | delta scenario 'A reload with a link to nothing in games' has no matrix row | review > absent-row — "A reload with a link to nothing in games" | add the row with its covering task or test |  |  | unmet-requirement |
| R-81 | test-coverage | should-fix | in-change | delta scenario 'An unreadable server file leaves world list' has no matrix row | review > absent-row — "An unreadable server file leaves world list" | add the row with its covering task or test |  |  | unmet-requirement |
| R-60 | verify | suggestion | in-change | PARTIAL: the fix cd948b212f17afd387a0e0119bb305c81e031977 of R-60 did not hold. D4 says the framing gives two different layer lists two different inputs. That rule covers every list, but task 4.4 tests one hand-made pair. | tasks.md > 4. The store — "which generates two different lists of paths and bytes and asserts two different framed inputs." | fix it again | regression | severity? should-fix 0.63 | false-test |

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
| 4 | plan | follow-up | fa7bcb23a3adf12bfe8cbf119449d5f4810ca2a6 | a89e0842f57fda193535b426370b97a26f5a2e39b65523018c521173c0578fa6 | approve-with-notes | 20 | 0 | 1 | 19 | 0 |  |

## Judge

judge: 2 questions, 0.000 USD

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

- verify: The fix of R-44 put the scenario 'An unknown OS flag' (mu config show --os windows) under the requirement 'mu config reload reads the directory again' (config-store spec line 167), whose text says nothing about --os; it belongs under 'mu config show prints one client configuration'.
- verify: D2 says a file name under clients/ cannot send an escape sequence to the terminal because mu-cli writes the store notices through warn. That covers the start only: task 5.3 does not say that mu config show and mu config reload print the notices of the API reply through report or warn.
- verify: R-60 is partial: the property test of task 4.4 generates lists of paths and bytes only, so an entry with tag 1 (an unreadable file with its error text) is covered by the one unit test an_unreadable_layer_changes_the_revision. An implementation without the tag byte collides a readable file whose bytes equal an error text with that unreadable file, and every named test still passes.
- verify: The R-61 fact is checked: figment2 0.11.5 is in Cargo.lock, and its src/coalesce.rs merges two dicts recursively under Order::Merge.
- verify: The existing connect_wait of crates/mu-cli/src/client.rs refuses NaN, infinity, 0 and below, and under 1 ns with an Err, so the reuse of R-58 agrees with the connect_wait requirement.

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
