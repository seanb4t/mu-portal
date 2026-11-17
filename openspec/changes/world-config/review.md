---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-17T13:41:41+00:00
head: d480822aa96f2e98c97aa8da76f641e75423ca0e
round: 3
next_id: 61
lenses:
  spec-lint: ok
  approach: partial
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
---
# Review

## Incomplete lenses

- approach: partial, gaps: 1
  - The citation of the XDG Base Directory specification, version 0.8, sections 2 and 3, is not checked: this lens has no web access.

The verdict covers only what the lenses checked. An incomplete lens keeps it from `approve`: the verdict is `changes-requested` until the lens is rerun or the gaps are accepted.

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-52 | approach | blocker | in-change | No decision produces the rule that show of an id that is not a configured world is an error that names the id; the mu world decision covers only the print of a resolved world and a bad file. | design.md > `mu world` — "`show` resolves all worlds first, so a bad file fails `show` as it fails the server." | Add to the mu world decision that show looks the id up among the resolved worlds, and that a missing id fails with a message that names it and the exit status 1. |  | severity? should-fix 0.72 |
| R-37 | approach | should-fix | in-change | lens approach reported partial, gaps: 1 | review > lens-status — "approach partial" | rerun the lens or accept the gap in the review | regression |  |
| R-46 | approach,fit,invariants | should-fix | in-change | fit-2: show writes the [[rules]] and [[scripts]] headers itself and puts toml::to_string of each entry under them; a sub-table of an entry then comes out as a root [name] table, so the output does not parse back.; app-2: The design says toml::to_string of an entry, written under its comment line, gives TOML that parses back. The document serializer of toml 1.2.1 writes a nested table of the entry as a root header such as [style], so after [[rules]] it leaves the entry, and two such entries fail to parse.; inv-1: show writes each rule entry with toml::to_string under a [[rules]] line, and toml 1.2.1 writes a nested table of an entry as a top-level [style] header. A probe reparsed one such rule with style at the root, and two such rules failed with duplicate key. | design.md > `mu world` — "Each entry of `rules` and `scripts` is a `toml::Table`, and `toml::to_string` writes it under its comment line." | fit-2: Let the toml crate write the header too: serialize each entry as a table {rules = [entry]} (or {scripts = [entry]}) under its comment line, or build the whole output as a toml_edit document with the sources as comments. Name the choice in the decision, and make show_output_parses_back generate entries with sub-tables.; app-2: Write each entry with its parent key, for example toml::to_string of a table {rules = [entry]}, so that the headers come out as [[rules]] and [rules.style]; or write each entry as an inline table. Make show_output_parses_back generate nested tables and arrays of tables in an entry.; inv-1: Serialize the entries through a wrapper table {rules = [entries]} so toml writes [rules.style], or write each entry as an inline table, and say which in the decision `mu world`. | regression | severity? blocker 0.78 |
| R-53 | approach | should-fix | in-change | The enforcement row of the crate rule says ALLOWED_CRATES of mu-core gains figment2, serde and toml, but task 1.2 and the proposal add serde_path_to_error as a fourth crate that the parse needs. | design.md > Enforcement of each rule — "`ALLOWED_CRATES` in the boundary test, with `figment2`, `serde` and `toml`, and `cargo deny check`." | Name serde_path_to_error in the enforcement row, so that the row lists the four crates of task 1.2. |  |  |
| R-54 | approach | should-fix | in-change | The checked fact says that with no features figment2 0.11.5 depends only on serde and uncased, but its manifest also has the build dependency version_check and the target dependency atomic on 32-bit targets. | design.md > Checked facts — "With no features, `figment2` 0.11.5 depends only on `serde` and `uncased`." | Name version_check and atomic in the fact, and say that Cargo.lock already holds both, so that the lock still gains only figment2, uncased and serde_path_to_error. |  |  |
| R-55 | fit | should-fix | in-change | The parse matches a charset name against Charset::name with no regard to case, which charset_answer in the framer already does inline; the change adds a second copy of that match. | design.md > Layers are parsed alone, then merged — "`charset` against `Charset::name`, with no regard to case. Thus `iso-8859-1` and `ISO-8859-1` are the same name." | Add Charset::from_name in framer.rs, which maps a name to a Charset with no regard to case. Name it in the decision, use it in the layer parse, and make charset_answer use it too. |  |  |
| R-56 | invariants | should-fix | in-change | The property test show_output_parses_back generates keys and string values only, so an entry with a nested table or an array of tables, the case that breaks the writer, never reaches it. | tasks.md > 3. The merge and `mu world show` — "It renders a resolved world with any strings and parses the output back to the same values." | Make the generator of show_output_parses_back produce entries with nested tables, arrays of tables and two entries that share a nested key. |  |  |
| R-57 | invariants | should-fix | in-change | The loop check runs only on the chain of a world, which starts at a character and a game. The scenario A loop of types has no game, so types a and b are never walked and the error the scenario requires never fires; an unknown extends in an unused type passes too. | design.md > Enforcement of each rule — "The tests `loop_of_types` and `type_that_extends_itself`." | Walk the extends chain from every type file as well as from each world, and keep loop_of_types with no game, or add a game that extends a to the scenario. |  |  |
| R-58 | invariants | should-fix | in-change | The value checks ({password} in login.template, no CR or LF in the template and keepalive.command, the range of keepalive.seconds, the charset name) belong to the parse of LayerFile. Built-in types parse through the separate Quirks struct, and no decision or test runs those checks on it. | design.md > Built-in types and the quirk schema — "The resolver reads them through a separate `Quirks` struct, which holds only `charset`, `login.template` and `keepalive`" | Run the same value checks on Quirks, and make built_in_types_hold_only_quirks go through them, so a built-in template without {password} fails a test. |  |  |
| R-59 | invariants | should-fix | in-change | The check after the merge tests only that host is set. host = "" passes it and the parse, and no test passes the empty host, so a world with no place to connect resolves. | specs/world-config/spec.md > Scenario: A world with no host or port — "- **GIVEN** no layer of `alpha/aria` sets `host`, or no layer sets `port`" | State that an empty host is an error that names the file and host, check it in the parse, and add host = "" to the table of edges_of_each_range or its own test. |  | severity? blocker 0.78 |
| R-60 | invariants | should-fix | in-change | The parse refuses a repeat only within one array, and the only cross-kind test puts the rule and the script in two layers. A rule and a script with one id in the same layer is not inherited, and no test passes it. | design.md > Layers are parsed alone, then merged — "No id repeats in one array of one layer." | Make rule_and_script_with_one_id a table test that also puts the rule and the script in the same file, and name where that check runs. |  |  |

## Repeats

None.

## Out of bound

| lens | severity | claim | evidence |
|---|---|---|---|
| spec-lint | should-fix | The THEN of 'A user type extends a built-in type' names the character set of tinymux, but no field name and no value, so the charset line of stdout is not pinned. | specs/world-config/spec.md > Scenario: A user type extends a built-in type — "THEN** stdout shows the character set of `tinymux` and `columns = 78`" |

## Accepted

None.

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 24231853aa33a631c42924f6d4fbd71382a53c2a | 43d710b636d358e848719344785e9f0dd0ec877ee969ec935c24da9f81cc5acb | changes-requested | 37 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 11c17ba73571a89b109c3b521bf7545997170757 | e7474d5caadfca061fb8aab67156e20ee6fd8b966a8e8353ca2598c592d7e29e | changes-requested | 14 | 0 | 2 | 0 |  |
| 3 | plan | follow-up | d480822aa96f2e98c97aa8da76f641e75423ca0e | 8ef3974cf52d28db4892f08142a7378b4c978d2c3b049fbd071714cacb8076f9 | changes-requested | 9 | 0 | 2 | 1 |  |

## Judge

judge: 20 questions, 0.001 USD

## Coverage matrix

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

## Verification runs

None.

## Dropped

None.

## Lens notes

- approach: No git access in this lens, so the bound of the follow-up round is judged from the headings whose text the fixed round-2 rows of prior.json and the head commit name.
- approach: Verified at head: has_line_break in framer.rs shared by secret.rs and session.rs; Source and Origin exported by the mu-core root; Charset::name gives UTF-8 and ISO-8859-1; is_part private in id.rs; SocketValue and parse_socket; the five launch points of mu in the tests; toml 1.2.1 keeps the key path of its error private and has Deserializer::parse; figment2 0.11.5 merge, extract, find_metadata and Named::new at the cited lines; Cargo.lock holds serde, atomic and version_check but not uncased or serde_path_to_error; deny.toml allows MIT and Apache-2.0.
- fit: No git diff was available to this lens, so the text changed since 11c17ba is inferred from the round-2 rows of prior.json; the bound of each finding rests on the heading of its first evidence item, and fit-2 also names the fixed row R-46.
- fit: The claim of fit-2 about the output of toml::to_string for a nested table comes from the documented behavior of the toml serializer and was not probed; a probe with two rules that each hold a sub-table decides it.
- fit: The fixed fit rows R-8, R-10, R-11, R-12, R-15, R-16 and R-41 (fit-3) hold at head: one TOML reader, the env_path.rs helpers, WorldDefaults, the name LayerOrigin, is_part as pub(crate), the toml writer and has_line_break.
- fit: The tests of tasks 2.3 to 6.1 write many configuration trees, and the plan names no helper for it; the library row names TreeCopy, whose append_file creates a file and its parents, but TreeCopy has no constructor for an empty tree.
- fit: The XDG lookup of env_path.rs is extracted from default_socket in socket.rs, so it reuses the repository helper; a crate such as etcetera is not named, and its fallback for an unset HOME differs from the spec.
- invariants: Follow-up round since 11c17ba: read every changed artifact; the deltas of cli-entrypoint and world-sessions hold no changed text.
- invariants: I1: No error prints a secret value \| enforced at ConfigError (no toml Display, no message under login, no BadSecretRef text) \| boundary test password_in_a_file, secret_that_is_not_a_reference, wrong_type_under_login
- invariants: Probe of I1 (toml 1.2.1, serde_path_to_error 0.1.20, under the handoff probe/ directory): paths were login.password, login and login.secret; eleven syntax errors under [login] gave messages that quote no value. port = "hunter2" keeps a message that quotes the value, which the design accepts outside login.
- invariants: I2: Each field error names the file and the field \| enforced at serde_path_to_error over toml::Deserializer::parse \| boundary test unknown_field, port_out_of_range
- invariants: I3: port, columns, rows are 1..65535 and keepalive.seconds 1..86400 \| enforced at the parse \| boundary test edges_of_each_range (0, 1, top, top+1)
- invariants: I4: login.name, login.template, keepalive.command hold no CR or LF \| enforced at has_line_break (framer.rs:28, checks CR and LF) \| boundary test line_break_in_a_line_field
- invariants: I5: login.template holds {password} \| enforced at the parse of LayerFile, none for Quirks (inv-4) \| boundary test template_with_no_password
- invariants: I6: charset matches Charset::name with no regard to case \| enforced at the parse \| boundary test charset_case (both cases), charset_the_engine_cannot_read
- invariants: I7: A built-in type sets only quirks \| enforced at Quirks with deny_unknown_fields \| boundary test quirks_refuse_other_fields
- invariants: I8: A shared plugin cannot set the six connection fields \| enforced at the trust check \| boundary test plugin_cannot_set_the_connection (table over six)
- invariants: I9: A layer that does not set a field keeps the earlier value \| enforced at skip_serializing_if \| boundary test empty_layer_gives_figment_no_keys, each_nested_field_alone_gives_one_key
- invariants: I10: Ids are unique in a world and keep the order of their first layer \| enforced at the parse and the resolver \| boundary test accumulation_matches_a_model, id_twice_in_one_layer, rule_and_script_with_one_id (cross-layer only, inv-6)
- invariants: I11: A disable names an inherited entry \| enforced at the resolver \| boundary test disable_with_no_target
- invariants: I12: A loop or unknown name in extends fails \| enforced at the chain walk from each world only (inv-3) \| boundary test loop_of_types, type_that_extends_itself, unknown_type
- invariants: I13: Each world has host and port, both or neither of the login pair and the keepalive pair \| enforced at the check after the merge \| boundary test both sides of each pair; empty host none (inv-5)
- invariants: I14: mu world show prints TOML that parses \| enforced at the toml writer, which fails for nested entry tables (inv-1) \| boundary test show_output_parses_back (strings only, inv-2). Probe: Display of toml::Value with newlines and quotes round-trips with a trailing comment.
- invariants: I15: File names follow the world id part rule \| enforced at is_part (id.rs:48) \| boundary test file_name_outside_the_id_rule, stray_file
- invariants: I16: No id is both fake and configured \| enforced at mu server before the socket \| boundary test fake_world_with_a_configured_id
- invariants: I17: A test never reads the developer configuration \| enforced at the shared environment list \| boundary test helper_points_mu_config_at_an_empty_directory. Grep found exactly five builders of CARGO_BIN_EXE_mu in tests, all five named by task 2.2.
- invariants: I18: XDG_CONFIG_HOME and HOME count only when absolute \| enforced at env_path.rs \| boundary test relative_xdg_config_home, no_home_directory (empty HOME), empty_mu_config
- invariants: I19: No resolution reads a secret \| enforced at the resolver taking no SecretStore \| boundary test show_does_not_read_the_secret, server_does_not_read_a_configured_secret
- spec-lint: Form checked against every requirement of the three delta specs: each requirement has a scenario with a WHEN or GIVEN and a THEN. No BREAKING word appears in proposal.md or in the delta specs, so there is no breaking-mismatch.
- spec-lint: Every tracked task in tasks.md names a command, a test, a file or an observable result. Every requirement of the delta specs has a task that names its scenarios.
- spec-lint: Build targets checked: mise tasks check and docs:check exist in mise.toml; the crates mu-core, mu-cli and mu-server exist; the test files worlds.rs, cli.rs and workspace_boundary.rs exist; the review-lint script exists at the path in task 7.1.
- spec-lint: Count mismatch in tasks.md 2.3 (not a finding class): it says 'four scenarios' of 'Each field is known', but the spec has five, and the scenario 'A template with no password' is listed under 'three scenarios' of 'Each value is in its range', which has two. Every scenario is still named by some task.
- spec-lint: Process note: a Grep over the change root matched a line in review.md. I did not open review.md, and I used none of its content. The finding above comes from the spec text alone.
- test-coverage: Every scenario has a task that names a test and a nextest command. No finding in the changed text.

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
| R-37 | 1 | 3 | should-fix | in-change | ["artifact",["review","lens-status"],"approach partial"] |  | regressed from fixed 57909dd67a277600fc26651a7800440885ed39c3 |  | lens approach reported partial, gaps: 1 |
| R-38 | 2 | 2 | blocker | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"Under `login`, it also drops the message of `toml`, because a message of `serde` can quote the value. In its place, it says that the value is not valid."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The THEN of 'A password in a file' needs login.password in stderr. The design drops the toml message under login, and the toml error has no public key path, so nothing in the design produces that name. |
| R-39 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"A `ConfigError` of a parse keeps the file, the key path, the line and the column of the `toml` error."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | app-1: The design keeps the key path of the toml error, but toml 1.2.1 has no public accessor for it: the keys field is private, and Display prints it only when the error has no span.; inv-1: The design drops the toml message under login by its key path, but toml 1.2.1 keeps the key path private; Error exposes only message() and span(). |
| R-40 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Tests do not read the configuration of the developer"],"One base launcher in `crates/mu-cli/tests/support/` builds the `Command` of each `mu` that a test starts."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The base launcher builds one Command, but the four launchers use three command types: std, tokio and the CommandBuilder of portable_pty, which cannot start from a std Command. |
| R-41 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"`login.name`, `login.template` and `keepalive.command` hold no CR and no LF, because the session writes each one as a line."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | fit-3: The CR and LF check of login.name, login.template and keepalive.command names no helper, while has_line_break in framer.rs is the one check that secret.rs and session.rs already share.; inv-9: The Login doc says the template holds {login} and {password}, and the template is now user input. Nothing refuses a template without {password}, so a typo sends no password. |
| R-42 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","6. The server and the docs"],"to name `figment2` and say why, and to call the window width `columns`, as the spec does."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The decision page states the plugin rule without login.template, which the spec now adds, and task 6.2 edits the page only for figment2 and columns, so the two statements of the rule disagree. |
| R-43 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: Half a keepalive"],"the game `alpha` sets `keepalive.command = \\"idle\\"` and no layer sets `keepalive.seconds`"] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The keepalive pair rule is tested only with a command and no seconds. Seconds with no command, the other side of the rule, has no test. |
| R-44 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: A world has a place to connect"],"After the merge, each world SHALL have `host` and `port`."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | Each world must have a host and a port, but only a missing port is tested. A world with no host has no test. |
| R-45 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"`ConfigError` drops the source snippet, the message under `login`, and the text of `BadSecretRef`. The two tests above check stderr."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | No test passes a value of the wrong type under login. A probe of toml 1.2.1 showed that serde quotes the value in its message, for an integer login.secret and for login set to a string. |
| R-46 | 2 | 3 | should-fix | in-change | ["artifact",["design.md","`mu world`"],"`show` writes each line as `<key> = <value> # <source>`. The `Display` of `toml::Value` writes each value, so the `toml` crate quotes and escapes each string."] |  | regressed from fixed 6baba79166e2abbee936962c92ee234b27629d89 |  | show writes the keys of a rule or script entry itself, unquoted. A probe printed `two words = "x"` and `a#b = 1`, and neither parsed back as TOML. |
| R-47 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Built-in types set only quirks"],"A built-in type SHALL set only quirk fields: `charset`, `login.template` and `keepalive`."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | No test feeds Quirks a field that is not a quirk; built_in_types_hold_only_quirks reads only valid files, so a Quirks or nested login struct without deny_unknown_fields passes. |
| R-48 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Rules and scripts accumulate"],"Each entry SHALL have an `id`."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The rule that each entry has an id has no test with an entry that lacks id, or has an id that is not a string. |
| R-49 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The configuration directory"],"An empty or relative `XDG_CONFIG_HOME` is ignored, as the XDG Base Directory specification asks."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | Nothing covers an unset, empty or relative HOME. The absolute-path check covers only the XDG base, so an empty HOME yields the relative path .config/mu under the cwd. |
| R-50 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The walk of the directory"],"Any other file that ends in `.toml` is an error that names its path."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The stray-file rule covers every depth, but the only test puts worlds.toml at the root. A .toml file one level deeper in a known directory goes untested. |
| R-51 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: A plugin sets a field of the connection"],"a layer from a shared plugin source named `colors` that sets one of `host`, `port`, `tls`, `login.name`, `login.secret` and `login.template`"] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The GIVEN of the scenario 'A plugin sets a field of the connection' names six fields with 'one of' and gives no value for any of them, so the input of the THEN is not concrete. |
| R-52 | 3 | 3 | blocker | in-change | ["artifact",["design.md","`mu world`"],"`show` resolves all worlds first, so a bad file fails `show` as it fails the server."] |  |  |  | No decision produces the rule that show of an id that is not a configured world is an error that names the id; the mu world decision covers only the print of a resolved world and a bad file. |
| R-53 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"`ALLOWED_CRATES` in the boundary test, with `figment2`, `serde` and `toml`, and `cargo deny check`."] |  |  |  | The enforcement row of the crate rule says ALLOWED_CRATES of mu-core gains figment2, serde and toml, but task 1.2 and the proposal add serde_path_to_error as a fourth crate that the parse needs. |
| R-54 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Checked facts"],"With no features, `figment2` 0.11.5 depends only on `serde` and `uncased`."] |  |  |  | The checked fact says that with no features figment2 0.11.5 depends only on serde and uncased, but its manifest also has the build dependency version_check and the target dependency atomic on 32-bit targets. |
| R-55 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"`charset` against `Charset::name`, with no regard to case. Thus `iso-8859-1` and `ISO-8859-1` are the same name."] |  |  |  | The parse matches a charset name against Charset::name with no regard to case, which charset_answer in the framer already does inline; the change adds a second copy of that match. |
| R-56 | 3 | 3 | should-fix | in-change | ["artifact",["tasks.md","3. The merge and `mu world show`"],"It renders a resolved world with any strings and parses the output back to the same values."] |  |  |  | The property test show_output_parses_back generates keys and string values only, so an entry with a nested table or an array of tables, the case that breaks the writer, never reaches it. |
| R-57 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"The tests `loop_of_types` and `type_that_extends_itself`."] |  |  |  | The loop check runs only on the chain of a world, which starts at a character and a game. The scenario A loop of types has no game, so types a and b are never walked and the error the scenario requires never fires; an unknown extends in an unused type passes too. |
| R-58 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Built-in types and the quirk schema"],"The resolver reads them through a separate `Quirks` struct, which holds only `charset`, `login.template` and `keepalive`."] |  |  |  | The value checks ({password} in login.template, no CR or LF in the template and keepalive.command, the range of keepalive.seconds, the charset name) belong to the parse of LayerFile. Built-in types parse through the separate Quirks struct, and no decision or test runs those checks on it. |
| R-59 | 3 | 3 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: A world with no host or port"],"- **GIVEN** no layer of `alpha/aria` sets `host`, or no layer sets `port`"] |  |  |  | The check after the merge tests only that host is set. host = "" passes it and the parse, and no test passes the empty host, so a world with no place to connect resolves. |
| R-60 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"No id repeats in one array of one layer."] |  |  |  | The parse refuses a repeat only within one array, and the only cross-kind test puts the rule and the script in two layers. A rule and a script with one id in the same layer is not inherited, and no test passes it. |
