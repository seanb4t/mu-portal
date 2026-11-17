---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-17T13:29:32+00:00
head: 11c17ba73571a89b109c3b521bf7545997170757
round: 2
next_id: 52
lenses:
  spec-lint: ok
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-38 | approach | blocker | in-change | The THEN of 'A password in a file' needs login.password in stderr. The design drops the toml message under login, and the toml error has no public key path, so nothing in the design produces that name. | design.md > Layers are parsed alone, then merged — "Under `login`, it also drops the message of `toml`, because a message of `serde` can quote the value. In its place, it s" | Name the step that builds the field path of an error under login without the message, for example: read the key at span() in the source, and the table that holds it, then write login.<key>. Also say how the error knows that it is under login. |  | severity? should-fix 0.70 |
| R-12 | approach,fit | should-fix | in-change | app-3: The design renames the layer source to Origin to avoid a clash in the crate root, but the crate root of mu-core already exports command::Origin, the maker of a command.; fit-1: The rename that fixed R-12 picks Origin, which the crate root of mu-core already exports as the origin of a command, so the name again means two things. | design.md > The layer seam and its trust — "It has its own name, so `Source` keeps one meaning in the crate root." | app-3: Pick a name that the crate root does not export, such as LayerOrigin or FieldSource, and use it in the seam, the sources map and the tasks.; fit-1: Give the source of a layer or a field a name that no export of crates/mu-core/src/lib.rs uses, and update the design, task 2.1 and the sources map with it. | regression |  |
| R-39 | approach,invariants | should-fix | in-change | app-1: The design keeps the key path of the toml error, but toml 1.2.1 has no public accessor for it: the keys field is private, and Display prints it only when the error has no span.; inv-1: The design drops the toml message under login by its key path, but toml 1.2.1 keeps the key path private; Error exposes only message() and span(). | design.md > Layers are parsed alone, then merged — "A `ConfigError` of a parse keeps the file, the key path, the line and the column of the `toml` error." | app-1: State how the parse gets the key path, for example from span() against the parsed document, or from checks of its own that name the field. Say that line and column come from span() and the source text. A u16 port of 70000 fails in serde with a message that does not name port.; inv-1: Name how the parse knows an error is under login. Either deserialize `login` as a `toml::Value` and check its shape in project code, so serde builds no message from a login value, or redact each error whose span() lies inside the login table. Compute line and column from span(). |  |  |
| R-40 | fit | should-fix | in-change | The base launcher builds one Command, but the four launchers use three command types: std, tokio and the CommandBuilder of portable_pty, which cannot start from a std Command. | design.md > Tests do not read the configuration of the developer — "One base launcher in `crates/mu-cli/tests/support/` builds the `Command` of each `mu` that a test starts." | Make the shared base the list of variable changes, in the (name, Option<value>) shape that ServerBuilder and mu_with already take, and let each launcher apply it to its own command type; say so in the decision and task 2.2. |  |  |
| R-41 | fit,invariants | should-fix | in-change | fit-3: The CR and LF check of login.name, login.template and keepalive.command names no helper, while has_line_break in framer.rs is the one check that secret.rs and session.rs already share.; inv-9: The Login doc says the template holds {login} and {password}, and the template is now user input. Nothing refuses a template without {password}, so a typo sends no password. | design.md > Layers are parsed alone, then merged — "`login.name`, `login.template` and `keepalive.command` hold no CR and no LF, because the session writes each one as a li" | fit-3: Name crate::framer::has_line_break as the check in the decision and in task 2.1, so the parse and the session refuse the same characters.; inv-9: Add a parse check that login.template holds `{password}` (and `{login}`, if the spec wants it), with a scenario and a test of a template that lacks it, or drop the claim from the Login doc. |  |  |
| R-42 | fit | should-fix | in-change | The decision page states the plugin rule without login.template, which the spec now adds, and task 6.2 edits the page only for figment2 and columns, so the two statements of the rule disagree. | tasks.md > 6. The server and the docs — "to name `figment2` and say why, and to call the window width `columns`, as the spec does." | In task 6.2, make the page name the login template in its plugin sentence, or make it point to the user docs page for the list of fields that a plugin cannot set. |  |  |
| R-27 | invariants | should-fix | in-change | A missing skip on login.name or keepalive.command passes all three key tests, because each nested test sets exactly the field it would check. | design.md > Enforcement of each rule — "If one field lacks it, `empty_layer_gives_figment_no_keys`, `login_name_only_gives_one_key` or `keepalive_command_only_g" | Add login_secret_only_gives_one_key and keepalive_seconds_only_gives_one_key, or one table test that serializes each nested field alone and expects one key. | regression |  |
| R-43 | invariants | should-fix | in-change | The keepalive pair rule is tested only with a command and no seconds. Seconds with no command, the other side of the rule, has no test. | specs/world-config/spec.md > Scenario: Half a keepalive — "the game `alpha` sets `keepalive.command = "idle"` and no layer sets `keepalive.seconds`" | Make half_a_keepalive a table test with both halves: command only, which names keepalive.seconds, and seconds only, which names keepalive.command. |  |  |
| R-44 | invariants | should-fix | in-change | Each world must have a host and a port, but only a missing port is tested. A world with no host has no test. | specs/world-config/spec.md > Requirement: A world has a place to connect — "After the merge, each world SHALL have `host` and `port`." | Add world_with_no_host, or make world_with_no_port a table test over host and port, each naming the world and the field. |  |  |
| R-45 | invariants | should-fix | in-change | No test passes a value of the wrong type under login. A probe of toml 1.2.1 showed that serde quotes the value in its message, for an integer login.secret and for login set to a string. | design.md > Enforcement of each rule — "`ConfigError` drops the source snippet, the message under `login`, and the text of `BadSecretRef`. The two tests above c" | Add cases to password_in_a_file or a new table test: login.secret set to an integer, and login set to a string, each asserting that stderr names the file and lacks the value. The second covers the key path `login` itself, not only paths under `login.`. |  |  |
| R-46 | invariants | should-fix | in-change | show writes the keys of a rule or script entry itself, unquoted. A probe printed `two words = "x"` and `a#b = 1`, and neither parsed back as TOML. | design.md > `mu world` — "`show` writes each line as `<key> = <value> # <source>`. The `Display` of `toml::Value` writes each value, so the `toml`" | Write each key through the toml crate as well (for example, render each entry as a `toml::Table`, or quote keys with `toml_writer`). Make show_output_parses_back generate entry keys from any string, including space, `#`, `=` and `.`. |  |  |
| R-47 | invariants | should-fix | in-change | No test feeds Quirks a field that is not a quirk; built_in_types_hold_only_quirks reads only valid files, so a Quirks or nested login struct without deny_unknown_fields passes. | specs/world-config/spec.md > Requirement: Built-in types set only quirks — "A built-in type SHALL set only quirk fields: `charset`, `login.template` and `keepalive`." | Add a test that parses `columns = 80` and `[login] name = "x"` through Quirks and expects an error for each. |  |  |
| R-48 | invariants | should-fix | in-change | The rule that each entry has an id has no test with an entry that lacks id, or has an id that is not a string. | specs/world-config/spec.md > Requirement: Rules and scripts accumulate — "Each entry SHALL have an `id`." | Add a scenario and the test entry_with_no_id: a rule with no id, and one with `id = 1`, each failing with stderr naming the file and `id`. |  |  |
| R-49 | invariants | should-fix | in-change | Nothing covers an unset, empty or relative HOME. The absolute-path check covers only the XDG base, so an empty HOME yields the relative path .config/mu under the cwd. | design.md > The configuration directory — "An empty or relative `XDG_CONFIG_HOME` is ignored, as the XDG Base Directory specification asks." | State the rule for an unset, empty or relative HOME (for example, no default directory and so no worlds, or an error). Pass HOME through the absolute-path helper of env_path.rs, and add a test with HOME empty. |  |  |
| R-50 | invariants | should-fix | in-change | The stray-file rule covers every depth, but the only test puts worlds.toml at the root. A .toml file one level deeper in a known directory goes untested. | design.md > The walk of the directory — "Any other file that ends in `.toml` is an error that names its path." | Make stray_file a table test that adds `characters/alpha.toml`, `games/old/alpha.toml` and `characters/alpha/aria/x.toml`, each named in stderr. |  |  |
| R-51 | spec-lint | should-fix | in-change | The GIVEN of the scenario 'A plugin sets a field of the connection' names six fields with 'one of' and gives no value for any of them, so the input of the THEN is not concrete. | specs/world-config/spec.md > Scenario: A plugin sets a field of the connection — "a layer from a shared plugin source named `colors` that sets one of `host`, `port`, `tls`, `login.name`, `login.secret` " | Name one value for each of the six fields in the GIVEN, or state that the scenario runs once per field with the value given there, so that the THEN names a concrete field. |  |  |

## Repeats

None.

## Out of bound

None.

## Accepted

None.

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 24231853aa33a631c42924f6d4fbd71382a53c2a | 43d710b636d358e848719344785e9f0dd0ec877ee969ec935c24da9f81cc5acb | changes-requested | 37 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 11c17ba73571a89b109c3b521bf7545997170757 | e7474d5caadfca061fb8aab67156e20ee6fd8b966a8e8353ca2598c592d7e29e | changes-requested | 14 | 0 | 2 | 0 |  |

## Judge

judge: 32 questions, 0.001 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| The flag wins over the environment | 2.3 | covered |  |
| The default directory | 2.3 | covered |  |
| The directory under the home directory | 2.3 | covered |  |
| An empty MU_CONFIG | 2.3 | covered |  |
| A relative XDG_CONFIG_HOME | 2.3 | covered |  |
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
| A disable with no target | 4.1 | covered |  |
| An id twice in one layer | 4.1 | covered |  |
| A rule and a script with one id | 4.1 | covered |  |
| Each built-in type reads as quirks | 5.1 | covered |  |
| A global preference is not hidden by a type | 5.1 | covered |  |
| A user type with a built-in name | 5.1 | covered |  |
| An unknown field | 2.3 | covered |  |
| A port out of range | 2.3 | covered |  |
| The case of a character set | 2.3 | covered |  |
| A character set that the engine cannot read | 2.3 | covered |  |
| The edges of each range | 2.3 | covered |  |
| A line break in a line field | 2.3 | covered |  |
| A password in a file | 2.3 | covered |  |
| A secret that is not a reference | 2.3 | covered |  |
| Show does not read the secret | 3.2 | covered |  |
| A plugin sets a field of the connection | 4.2 | covered |  |
| A plugin sets a preference | 4.2 | covered |  |
| A world with no port | 3.2 | covered |  |
| A login name with no secret | 3.2 | covered |  |
| A login secret with no name | 3.2 | covered |  |
| Half a keepalive | 3.2 | covered |  |
| A world that logs in by hand | 3.2 | covered |  |
| Sources of fields | 3.2 | covered |  |
| An unknown world | 3.2 | covered |  |
| A bad configuration stops the start | 6.1 | covered |  |
| A fake world with a configured id | 6.1 | covered |  |
| The secret is not read at the start | 6.1 | covered |  |
| Two worlds connect | 6.1 | covered |  |
| A configured world waits | 6.1 | covered |  |
| A configured world cannot connect yet | 6.1 | covered |  |
| Help lists the verbs | 2.3 | covered |  |
| Help for one verb | 2.3 | covered |  |

### Not run

Plan phase: no verification runs.

## Verification runs

None.

## Dropped

None.

## Lens notes

- approach: Checked and holding at head: figment2 0.11.5 merge/extract/find_metadata at figment.rs 348/605/940 and Named::new at named.rs 39; with no features figment2 adds only figment2 and uncased to Cargo.lock (atomic and version_check are already there).
- approach: Checked and holding: is_part is private at id.rs 48; Charset::name gives UTF-8 and ISO-8859-1; TelnetSettings defaults 120 by 40; SocketValue and parse_socket read empty as none; mu-cli depends on toml and mu-core on no TOML crate; the four launchers and their MU_SOCKET handling; mu client spawns the server with the inherited environment.
- approach: Checked and holding: deny.toml allows no MPL-2.0, so the rejection of schematic for option-ext holds; the Display of toml::Value and Table exists under the default display feature of toml 1.2.1.
- fit: The Context of design.md says mu-cli already depends on toml, but crates/mu-cli/Cargo.toml line 38 has toml under [dev-dependencies] only. If mu world show renders with toml::Value in mu-cli, task 1.2 must also add toml to the [dependencies] of mu-cli. ALLOWED_CRATES already permits it for mu-cli.
- fit: design.md says GlobalFile, TypeFile, GameFile and CharacterFile share LayerFields under deny_unknown_fields. The serde documentation states that deny_unknown_fields is not supported with flatten, so if the four share it through flatten, unknown fields pass. This fact was not checked against a local copy of serde here.
- fit: crates/mu-cli/tests/cli.rs line 112 builds a fifth Command of mu, outside the four launchers that the design lists. It runs a usage error and reads no configuration, and the design leaves such a test to review.
- invariants: I1: No ConfigError text holds a secret value \| enforced at ConfigError (drop snippet, message under login, BadSecretRef text), design only; toml 1.2.1 keeps the key path private \| boundary test password_in_a_file, secret_that_is_not_a_reference (no wrong-type case) -> inv-1, inv-2
- invariants: I2: port, columns and rows are 1 to 65535 \| enforced at the layer parse (a probe with NonZeroU16 refused 0 and 65536 with keys port) \| boundary test edges_of_each_range (0, 1, top, top+1)
- invariants: I3: keepalive.seconds is 1 to 86400 \| enforced at the layer parse \| boundary test edges_of_each_range
- invariants: I4: login.name, login.template and keepalive.command hold no CR or LF \| enforced at the layer parse, and again at session write_line (crates/mu-core/src/session.rs:426) \| boundary test line_break_in_a_line_field
- invariants: I5: login.name and login.secret are both set or neither \| enforced at the check after the merge \| boundary test login_name_with_no_secret, login_secret_with_no_name, world_that_logs_in_by_hand
- invariants: I6: keepalive.command and keepalive.seconds are both set or neither \| enforced at the check after the merge \| boundary test half_a_keepalive (one side only) -> inv-10
- invariants: I7: each world has host and port \| enforced at the check after the merge \| boundary test world_with_no_port (port only) -> inv-11
- invariants: I8: ids are unique in one array and across rules and scripts \| enforced at the parse and the resolver \| boundary test id_twice_in_one_layer, rule_and_script_with_one_id, property accumulation_matches_a_model
- invariants: I9: each entry has an id \| enforced at the parse (implied by the entry type) \| boundary test none -> inv-6
- invariants: I10: a disable id names an inherited entry \| enforced at the resolver \| boundary test disable_with_no_target
- invariants: I11: a shared plugin layer sets none of six connection fields \| enforced at the trust check \| boundary test plugin_cannot_set_the_connection over the six fields
- invariants: I12: a built-in type sets only quirks \| enforced at Quirks with deny_unknown_fields \| boundary test built_in_types_hold_only_quirks (positive only) -> inv-5
- invariants: I13: mu world show prints TOML that parses \| enforced at toml::Value Display for values; keys of entries none \| property test show_output_parses_back (values) -> inv-3
- invariants: I14: a layer that does not set a field never hides an earlier value \| enforced at skip_serializing_if on each optional field \| boundary test three key tests, nested gap -> inv-4
- invariants: I15: the extends chain is acyclic and each name is known and unshadowed \| enforced at the resolver walk \| boundary test loop_of_types, type_that_extends_itself, unknown_type, user_type_with_a_built_in_name
- invariants: I16: each file name is a part of a world id \| enforced at TomlDirSource with is_part \| boundary test file_name_outside_the_id_rule
- invariants: I17: no id is both fake and configured \| enforced at mu server before the socket opens \| boundary test fake_world_with_a_configured_id
- invariants: I18: the configuration directory is absolute \| enforced at the env_path XDG helper; HOME none \| boundary test relative_xdg_config_home, empty_mu_config -> inv-7
- invariants: I19: no resolution reads a secret \| enforced at the resolver signature (no SecretStore) \| boundary test show_does_not_read_the_secret, server_does_not_read_a_configured_secret
- invariants: I20: a charset name matches with no regard to case \| enforced at the parse against Charset::name \| boundary test charset_case, charset_the_engine_cannot_read
- invariants: I21: a template holds {login} and {password} \| enforced at none \| boundary test none -> inv-9
- invariants: Probe: toml 1.2.1 error messages for a syntax error on the secret line (unterminated string, bare word) held no value, and those errors carry no key path at all.
- invariants: Probe: columns or rows of 255 or 65535 put 0xFF in NAWS; send_subnegotiation escapes IAC (crates/mu-core/src/telnet/parser.rs:252), so the upper edge is safe on the wire.
- invariants: Probe sources and output are under probe-toml/ in the handoff directory.
- invariants: inv-1 concerns the mechanism of the fix of R-1; the leak of R-1 itself was not reproduced at head, so inv-1 carries no prior.
- spec-lint: Follow-up round: the tools here have no git diff, so the changed lines of design.md, proposal.md, spec.md and tasks.md could not be identified. Findings are judged on content; the in-bound status of spe-1 depends on whether its scenario changed since the base.
- spec-lint: Checked with no finding: every requirement of the world-config delta has at least one scenario with a WHEN or GIVEN and a THEN; every requirement and scenario named in tasks.md 2.3 to 6.1 has a task that names it, with matching counts; BREAKING appears nowhere in proposal.md or the delta specs, so no breaking-mismatch; the build targets mise run check, mise run docs:check, the crates mu-core, mu-cli and mu-server, and the test files worlds.rs, cli.rs and workspace_boundary.rs exist.
- spec-lint: A grep over the change root returned one line of review.md. That file was not read and was not used.
- test-coverage: Task 4.1 adds property test accumulation_matches_a_model to mu-core, but its command runs only mu-cli --test world_config; likewise 3.2 show_output_parses_back has no named crate. No scenario depends on them.
- test-coverage: No BREAKING requirement in the delta specs.

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
| R-37 | 1 | 1 | should-fix | in-change | ["artifact",["review","lens-status"],"approach partial"] | fixed | 57909dd67a277600fc26651a7800440885ed39c3 |  | lens approach reported partial, gaps: 1 |
| R-38 | 2 | 2 | blocker | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"Under `login`, it also drops the message of `toml`, because a message of `serde` can quote the value. In its place, it says that the value is not valid."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The THEN of 'A password in a file' needs login.password in stderr. The design drops the toml message under login, and the toml error has no public key path, so nothing in the design produces that name. |
| R-39 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"A `ConfigError` of a parse keeps the file, the key path, the line and the column of the `toml` error."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | app-1: The design keeps the key path of the toml error, but toml 1.2.1 has no public accessor for it: the keys field is private, and Display prints it only when the error has no span.; inv-1: The design drops the toml message under login by its key path, but toml 1.2.1 keeps the key path private; Error exposes only message() and span(). |
| R-40 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Tests do not read the configuration of the developer"],"One base launcher in `crates/mu-cli/tests/support/` builds the `Command` of each `mu` that a test starts."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The base launcher builds one Command, but the four launchers use three command types: std, tokio and the CommandBuilder of portable_pty, which cannot start from a std Command. |
| R-41 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"`login.name`, `login.template` and `keepalive.command` hold no CR and no LF, because the session writes each one as a line."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | fit-3: The CR and LF check of login.name, login.template and keepalive.command names no helper, while has_line_break in framer.rs is the one check that secret.rs and session.rs already share.; inv-9: The Login doc says the template holds {login} and {password}, and the template is now user input. Nothing refuses a template without {password}, so a typo sends no password. |
| R-42 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","6. The server and the docs"],"to name `figment2` and say why, and to call the window width `columns`, as the spec does."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The decision page states the plugin rule without login.template, which the spec now adds, and task 6.2 edits the page only for figment2 and columns, so the two statements of the rule disagree. |
| R-43 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: Half a keepalive"],"the game `alpha` sets `keepalive.command = \\"idle\\"` and no layer sets `keepalive.seconds`"] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The keepalive pair rule is tested only with a command and no seconds. Seconds with no command, the other side of the rule, has no test. |
| R-44 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: A world has a place to connect"],"After the merge, each world SHALL have `host` and `port`."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | Each world must have a host and a port, but only a missing port is tested. A world with no host has no test. |
| R-45 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"`ConfigError` drops the source snippet, the message under `login`, and the text of `BadSecretRef`. The two tests above check stderr."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | No test passes a value of the wrong type under login. A probe of toml 1.2.1 showed that serde quotes the value in its message, for an integer login.secret and for login set to a string. |
| R-46 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","`mu world`"],"`show` writes each line as `<key> = <value> # <source>`. The `Display` of `toml::Value` writes each value, so the `toml` crate quotes and escapes each string."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | show writes the keys of a rule or script entry itself, unquoted. A probe printed `two words = "x"` and `a#b = 1`, and neither parsed back as TOML. |
| R-47 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Built-in types set only quirks"],"A built-in type SHALL set only quirk fields: `charset`, `login.template` and `keepalive`."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | No test feeds Quirks a field that is not a quirk; built_in_types_hold_only_quirks reads only valid files, so a Quirks or nested login struct without deny_unknown_fields passes. |
| R-48 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Rules and scripts accumulate"],"Each entry SHALL have an `id`."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The rule that each entry has an id has no test with an entry that lacks id, or has an id that is not a string. |
| R-49 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The configuration directory"],"An empty or relative `XDG_CONFIG_HOME` is ignored, as the XDG Base Directory specification asks."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | Nothing covers an unset, empty or relative HOME. The absolute-path check covers only the XDG base, so an empty HOME yields the relative path .config/mu under the cwd. |
| R-50 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The walk of the directory"],"Any other file that ends in `.toml` is an error that names its path."] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The stray-file rule covers every depth, but the only test puts worlds.toml at the root. A .toml file one level deeper in a known directory goes untested. |
| R-51 | 2 | 2 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: A plugin sets a field of the connection"],"a layer from a shared plugin source named `colors` that sets one of `host`, `port`, `tls`, `login.name`, `login.secret` and `login.template`"] | fixed | 6baba79166e2abbee936962c92ee234b27629d89 |  | The GIVEN of the scenario 'A plugin sets a field of the connection' names six fields with 'one of' and gives no value for any of them, so the input of the THEN is not concrete. |
