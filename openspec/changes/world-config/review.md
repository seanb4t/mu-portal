---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-17T14:06:29+00:00
head: 370a2621585b22b9a902ca04813279ce01fe8c00
round: 4
next_id: 68
lenses:
  spec-lint: partial
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
---
# Review

## Incomplete lenses

- spec-lint: partial, gaps: 1
  - Follow-up bound not verified: the changed lines between the since commit d480822 and head were not computed, because this lens has no git tool. Findings are reported with the changed text first where the evidence is known, and the merge applies the bound.

The verdict covers only what the lenses checked. An incomplete lens keeps it from `approve`: the verdict is `changes-requested` until the lens is rerun or the gaps are accepted.

## Findings

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

## Repeats

None.

## Out of bound

| lens | severity | claim | evidence |
|---|---|---|---|
| spec-lint | should-fix | Many show and list scenarios give a GIVEN with no host and no port, but the post-merge rule makes both mandatory and no default supplies a port, so their exit-0 and stdout THENs cannot hold. | specs/world-config/spec.md > Scenario: Worlds from character files — "a directory with `games/alpha.toml`, `games/beta.toml`, `characters/alpha/aria.toml`" |
| spec-lint | should-fix | The GIVEN of two secret scenarios sets login.secret with no login.name, which the requirement 'A world has a place to connect' rejects, so their exit-0 THEN contradicts the spec. | specs/world-config/spec.md > Show does not read the secret — "the character sets `login.secret" |
| spec-lint | should-fix | Character-file scenarios such as 'A password in a file' give a character with no game file, which the requirement makes an error, so the THEN that stderr names login.password cannot be produced as written. | specs/world-config/spec.md > A password in a file — "the character sets `login.password" |

## Accepted

None.

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 24231853aa33a631c42924f6d4fbd71382a53c2a | 43d710b636d358e848719344785e9f0dd0ec877ee969ec935c24da9f81cc5acb | changes-requested | 37 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 11c17ba73571a89b109c3b521bf7545997170757 | e7474d5caadfca061fb8aab67156e20ee6fd8b966a8e8353ca2598c592d7e29e | changes-requested | 14 | 0 | 2 | 0 |  |
| 3 | plan | follow-up | d480822aa96f2e98c97aa8da76f641e75423ca0e | 8ef3974cf52d28db4892f08142a7378b4c978d2c3b049fbd071714cacb8076f9 | changes-requested | 9 | 0 | 2 | 1 |  |
| 4 | plan | follow-up | 370a2621585b22b9a902ca04813279ce01fe8c00 | 7044b6b91f696a97397d08c9e3eb4a5f67cb2f250036e146010936a74d956b8c | changes-requested | 7 | 0 | 2 | 3 | operator decision |

## Judge

judge: 16 questions, 0.001 USD

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

- approach: No git tool was available, so the diff since d480822 was not read. The bound was approximated from the round-3 rows R-53 to R-60 of prior.json; the merge decides which findings fall out of bound.
- approach: The fixes of R-53 to R-60 hold at head: ALLOWED_CRATES row names four crates, the figment2 manifest matches the checked-facts row, charset_answer at framer.rs:362 is the one case-blind match to move, the loop walk covers unused types, Quirks runs the value checks, the empty host is refused, and a same-layer rule and script with one id is refused.
- approach: Cited facts verified at head: is_part private in id.rs:48, Source and Origin exported from lib.rs, has_line_break shared by secret.rs and session.rs, the five launch sites of mu in the tests, client.rs start_server inherits the environment, toml 1.2.1 and atomic, version_check, itoa, serde_core in Cargo.lock with no uncased, figment2 0.11.5 merge/extract/find_metadata at lines 348/605/940 and Named::new at named.rs:39.
- approach: serde_path_to_error 0.1.20 de.rs:1513 adds the captured key to the path of a key error, so an unknown field under login yields the full dotted path that the scenario needs.
- approach: The requirement 'Rules and scripts accumulate' says an entry with an inherited id replaces it, while 'Rules and scripts have unique ids' and the design make a rule and a script with one id an error. The design picks the error; the wording of the first requirement is a matter for the spec-lint lens.
- fit: The bound of this follow-up round was inferred from the rows of prior.json fixed since the base (R-55 to R-60) and the four changed artifact files; no line diff was read, because the lens has no git access.
- fit: The repository rows of openspec/config.yaml name TreeCopy in crates/mu-cli/tests/support/mod.rs as a test helper. TreeCopy::append_file already creates a file and its parent directories in a temporary directory, which the many tests of crates/mu-cli/tests/world_config.rs need; the plan names no helper for this, so the implementation phase should check that the tests reuse it rather than add a new one.
- fit: Charset::from_name with one caller in charset_answer and one in the parse, has_line_break, is_part, the env_path.rs helpers, WorldDefaults and toml::to_string for show each reuse an existing helper or library, and no duplication was found in the changed decisions.
- invariants: I1: Every extends name resolves to a built-in or a user type \| enforced at the chain walk from each world and each file in types/ (design 'The chain of a world'); games with no character are not walked \| boundary test unknown_type (game only), see inv-1 and inv-2
- invariants: I2: No loop in extends, including a loop that no game reaches \| enforced at the walk from each file in types/ \| boundary test loop_of_types (no game), type_that_extends_itself
- invariants: I3: A charset name matches with no regard to case \| enforced at Charset::from_name (new), shared with charset_answer \| boundary test charset_case, charset_the_engine_cannot_read; framer side charset_is_offered (lower-case utf-8) at crates/mu-core/tests/telnet_framer.rs:321; non-UTF-8 list name untested, see inv-4
- invariants: I4: Built-in types pass the value checks of a layer \| enforced at the Quirks to LayerFields conversion and checks \| boundary test none with a bad value, see inv-3
- invariants: I5: After the merge, host is set and not empty, and port is set \| enforced at the check after the merge \| boundary test world_with_no_host_or_port table over no host, empty host, no port
- invariants: I6: Rules and scripts share one id space, including in one layer \| enforced at the resolver accumulation \| boundary test rule_and_script_with_one_id table over two layers and one layer
- invariants: I7: mu world show prints TOML that parses back to the same values \| enforced at toml::to_string of a one-entry document per entry \| boundary test show_output_parses_back with nested tables and arrays of tables. A probe of toml 1.2.1 (inv-probe under the handoff dir) parsed back three edge documents: a nested table followed by a second [[rules]], odd keys with a newline value and an empty nested table, and an array of empty tables. All entries came back equal.
- invariants: I8: show of an id that is not configured exits 1 and names the id \| enforced at the lookup after resolve \| boundary test unknown_world (well-formed id only), see inv-5
- invariants: I9: Cargo.lock gains only figment2, uncased and serde_path_to_error \| enforced at task 1.2 check \| checked: Cargo.lock at head already holds atomic 0.6.1, version_check 0.9.5, itoa and serde_core, which match the requirements of figment2 0.11.5 and serde_path_to_error 0.1.20
- invariants: Follow-up round: findings cite the changed paragraphs of design.md 'The chain of a world', 'Layers are parsed alone, then merged', 'Built-in types and the quirk schema' and '`mu world`'. No change since the base weakens a check.
- spec-lint: All 14 requirements of the three delta specs have at least one scenario, and every scenario has a WHEN or GIVEN and a THEN. No malformed-scenario finding.
- spec-lint: Every task group names a command, test, file or observable. Every requirement heading matches two or more distinctive words in a task line. No untasked-requirement and no unverifiable-task finding.
- spec-lint: No BREAKING word appears in proposal.md or in the delta specs, so there is no breaking-mismatch finding.
- spec-lint: The build targets in tasks.md exist: mise tasks check and docs:check in mise.toml, the crates under crates/ including mu-server, and the test files workspace_boundary.rs, cli.rs and worlds.rs. The new test files named by tasks are created by the tasks and are not unknown targets.
- spec-lint: No open question in design.md, so no assumed-answer finding.
- spec-lint: The per-scenario table of the coverage lens is not produced by this lens; matrix is empty.
- spec-lint: A Grep for BREAKING matched one line in review.md. That line was not read as input and is not used.
- test-coverage: No BREAKING requirement in the delta specs. design.md read only through prior.json and the specs and tasks; every scenario maps to a task with a named test and a nextest command.

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
| R-57 | 3 | 4 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"The tests `loop_of_types` and `type_that_extends_itself`."] |  | regressed from fixed ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | The loop check runs only on the chain of a world, which starts at a character and a game. The scenario A loop of types has no game, so types a and b are never walked and the error the scenario requires never fires; an unknown extends in an unused type passes too. |
| R-58 | 3 | 4 | should-fix | in-change | ["artifact",["design.md","Built-in types and the quirk schema"],"The resolver reads them through a separate `Quirks` struct, which holds only `charset`, `login.template` and `keepalive`."] |  | regressed from fixed ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | The value checks ({password} in login.template, no CR or LF in the template and keepalive.command, the range of keepalive.seconds, the charset name) belong to the parse of LayerFile. Built-in types parse through the separate Quirks struct, and no decision or test runs those checks on it. |
| R-59 | 3 | 3 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: A world with no host or port"],"- **GIVEN** no layer of `alpha/aria` sets `host`, or no layer sets `port`"] | fixed | ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | The check after the merge tests only that host is set. host = "" passes it and the parse, and no test passes the empty host, so a world with no place to connect resolves. |
| R-60 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"No id repeats in one array of one layer."] | fixed | ec957596a2ed1627f0788f6a05bc25123751bbb6 |  | The parse refuses a repeat only within one array, and the only cross-kind test puts the rule and the script in two layers. A rule and a script with one id in the same layer is not inherited, and no test passes it. |
| R-61 | 4 | 4 | blocker | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"`charset` with `Charset::from_name`, a new function of `framer.rs` that matches a name with no regard to case."] |  |  |  | app-1: The THEN of 'A user type extends a built-in type' needs show to print charset = "iso-8859-1". The resolved charset is a Charset whose only name is ISO-8859-1, and the design keeps no written text.; fit-1: The parse checks charset with Charset::from_name but the design does not say that the parse keeps the Charset or that ResolvedWorld.charset is a Charset. If the resolved world keeps the text, world-sessions (#54) must call from_name a second time to fill TelnetSettings.charset. If it keeps a Charset, show prints Charset::name(), which is ISO-8859-1, while the spec expects the lower-case text of the built-in file. |
| R-62 | 4 | 4 | blocker | in-change | ["artifact",["design.md","`mu world`"],"`show` writes each field of the world as `<key> = <value> # <source>`. The keys of these fields are fixed names"] |  |  |  | The THEN of 'A table merges field by field' needs the three login lines under a login table. The show decision writes each field as one key line and creates no table header and no order of tables after top-level keys. |
| R-63 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","The chain of a world"],"Before it builds any chain, it also walks from each file in `types/`, so a type that no game uses is checked too."] |  |  |  | The new walk starts from each file in types/ only, so a game with no character file whose extends names an unknown type is never walked, and the spec's unknown-name error never fires. |
| R-64 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"`charset_answer` of the framer moves to it, so the match has one copy."] |  |  |  | charset_answer moves onto Charset::from_name, a text match, but the server list is raw bytes. No test passes a non-UTF-8 name before a valid one, so a refactor that rejects the whole list passes. |
| R-65 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","`mu world`"],"An id that is not a configured world is an error that names the id, and the exit status is 1."] |  |  |  | show must exit 1 for an id that is not a configured world. The only test uses a well-formed id, and a malformed id such as Alpha/x would stop in a clap value parser with exit 2, as --fake-world does. |
| R-66 | 4 | 4 | suggestion | in-change | ["artifact",["design.md","Built-in types and the quirk schema"],"The resolver turns each `Quirks` into a `LayerFields` and runs the value checks of the parse on it"] |  |  |  | The value checks run in two places: the TOML parse of each file, and the resolver for each built-in Quirks. A Layer from any other LayerSource, such as the test plugin layer now and the Lua source that the proposal names next, reaches the merge with no value check, so the Lua change must add a third call site or move the checks into the resolver. |
| R-67 | 4 | 4 | should-fix | in-change | ["artifact",["review","lens-status"],"spec-lint partial"] |  |  |  | lens spec-lint reported partial, gaps: 1 |
