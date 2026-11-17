---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-17T13:17:17+00:00
head: 24231853aa33a631c42924f6d4fbd71382a53c2a
round: 1
next_id: 38
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
  - The facts about figment2 (its API, its TOML reader toml_edit 0.25, and the claim that it adds only figment2 and uncased to Cargo.lock) are not checked: no source of figment2 is in the local registry and this lens has no network tool. Task 1.1 owns them.

The verdict covers only what the lenses checked. An incomplete lens keeps it from `approve`: the verdict is `changes-requested` until the lens is rerun or the gaps are accepted.

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-1 | approach,invariants | blocker | in-change | app-1: The design shows the toml error to the user, and the Display of a toml error prints the whole source line, so the value of a password field in a character file reaches stderr.; inv-1: No task strips the source line from TOML errors, and toml_edit prints that line. A probe printed hunter2 for both login.password and a bad login.secret. | design.md > The merge of fields uses `figment2` — "Thus the errors that a user sees come from `toml`, with the file, the field and the line." | app-1: Decide in design.md how a parse error of a file is printed without its source line (for example the message and the key path only), and name that as the means of the goal that no error prints a secret value.; inv-1: Make ConfigError keep the file, the key path, the line and the column of a TOML error and drop its source snippet, for every layer and every field. State this in the design, and give tasks 2.1 an explicit redaction step that password_in_a_file and secret_that_is_not_a_reference exercise. |  |  |
| R-2 | approach | blocker | in-change | The design checks the login reference with the FromStr parse of the reference type, whose error keeps and prints the bad text, so a plain password in that field reaches stderr. | design.md > Layers are parsed alone, then merged — "Thus the error names that file and that field. The parse checks `login.secret` with" | State in the decision that the layer parse maps the parse error of the reference to an error that names the file and the field, and drops the text that the error holds. |  |  |
| R-3 | approach | blocker | in-change | No decision produces the requirement "A world has a place to connect": the required host and port, the login pair, and a template that does not count alone. Only task 3.1 names the check. | specs/world-config/spec.md > Requirement: A world has a place to connect — "Without `login.name` and `login.secret`, a `login.template` SHALL NOT count." | Add a decision for the check after the merge: which fields are required, that `login` is `None` when neither name nor secret is set even if a type sets a template, and that the error names the world id and the field. |  | severity? should-fix 0.63 |
| R-4 | approach | blocker | in-change | No decision refuses an id that is both a fake and a configured world; Portal::start keys worlds by id in a BTreeMap, so a duplicate would silently replace one sender. | design.md > Configured worlds in the server — "Then it adds a `WorldSetup` with no transport and no password for each configured world." | Add to the decision where `mu server` checks the fake ids against the configured ids, before the socket opens, and the error that names the id. |  |  |
| R-5 | approach | blocker | in-change | No decision says how TomlDirSource maps file names to ids and refuses a name outside the id rule, a character with no game file, or a stray `.toml` file. | design.md > The layer seam and its trust — "`TomlDirSource` reads the directory." | Add a decision for the walk of the directory: the paths it accepts, the parse of each stem with the rule of `WorldId`, and the errors for a stray file and a character with no game. |  | severity? should-fix 0.67 |
| R-6 | approach | blocker | in-change | The chain decision covers only the loop and the built-in name; nothing makes `extends` required in a game, refused in global.toml and a character, or an unknown name an error. | design.md > The chain of a world — "The chain starts at the character and the game. Next comes the type that the game `extends`" | State in the decision which layer kinds accept `extends` (for example a separate parse struct for each kind) and the errors for a missing, a forbidden and an unknown `extends`. |  | severity? should-fix 0.73 |
| R-7 | approach | blocker | in-change | The design checks the character set against the names of Charset, which are `UTF-8` and `ISO-8859-1`, but the requirement makes tinymux set `iso-8859-1`; an exact check fails the built-in type. | design.md > Layers are parsed alone, then merged — "and the character set against the names of `Charset`" | State the accepted spellings in the decision: a case-insensitive match of `Charset::name`, or a serde rename to lower-case names. |  |  |
| R-8 | approach,fit | blocker | in-change | app-9: The resolver in mu-core parses each file with `toml`, but the plan adds only `figment2` and `serde` to mu-core and to its ALLOWED_CRATES, so the boundary test refuses the parse path.; fit-4: The plan names two TOML readers: figment2 with its TOML feature, and the toml crate that parses each layer before Serialized; task 1.2 adds only the first to mu-core.; app-10: The TOML feature of figment2 serves nothing: the resolver passes figment2 only Serialized providers, so figment2 never reads a file. | design.md > The merge of fields uses `figment2` — "The resolver gives `figment2` only typed layers that `toml` parsed, through `Serialized`." | app-9: Add `toml` (already in [workspace.dependencies]) to the dependencies of mu-core and to its ALLOWED_CRATES row, in the enforcement table and task 1.2.; fit-4: Pick the workspace toml crate as the one reader: drop the TOML feature of figment2 from the proposal, and add toml to mu-core and its ALLOWED_CRATES row in task 1.2.; app-10: Drop the TOML feature of figment2, or name the need for it; then correct the risk line on toml_edit. |  |  |
| R-9 | approach | should-fix | in-change | Only ServerBuilder clears MU_SOCKET; `mu_with` and `mu_on_pty` start `mu` with the inherited environment, so the cited helper does not cover each `mu` a test starts. | design.md > Context — "The process helper in `crates/mu-cli/tests/support/process.rs` clears `MU_SOCKET` for each `mu` that a test starts." | Correct the fact, and name in "Tests do not read the configuration of the developer" each helper that sets MU_CONFIG, including `mu_with` in support/mod.rs, which the new `mu world` tests use. |  |  |
| R-37 | approach | should-fix | in-change | lens approach reported partial, gaps: 1 | review > lens-status — "approach partial" | rerun the lens or accept the gap in the review |  |  |
| R-10 | fit | should-fix | in-change | ConfigArgs copies the shape of SocketArgs: a second empty-means-unset value parser and a second relative-XDG filter, with no shared helper. | design.md > The configuration directory — "`mu-cli` gets `ConfigArgs`, with `--config` and the order `--config`, `MU_CONFIG`, `XDG_CONFIG_HOME/mu`, then `HOME/.con" | Move the empty-as-unset path value (SocketValue, parse_socket) and the absolute-only XDG base lookup into one helper that SocketArgs and ConfigArgs both use, and name it in the decision. |  |  |
| R-11 | fit | should-fix | in-change | The defaults of charset, columns, rows and the login template get two new homes (the resolver's default layer and with_default_login) beside TelnetSettings::default and DEFAULT_LOGIN_TEMPLATE. | design.md > The resolved world — "The fake world keeps `with_default_login`, which fills the new fields with the defaults." | State in the design that the default layer and with_default_login both read one defaults value built from TelnetSettings::default() and DEFAULT_LOGIN_TEMPLATE, so no number or template is written twice. |  |  |
| R-12 | fit | should-fix | in-change | The new Source type of a layer reuses the name of mu_core::Source, which the crate root already exports as the source of an event. | design.md > The layer seam and its trust — "pub struct Layer { pub key: LayerKey, pub source: Source, pub trust: Trust, pub fields: LayerFields }" | Give the source of a field its own name, such as FieldSource or Origin, in the design and in task 2.1, so the crate root keeps one meaning for Source. |  |  |
| R-13 | fit | should-fix | in-change | The design adds the MU_CONFIG scrub to ServerBuilder alone, while three other test launchers build their own environment and mu client starts a server that inherits it. | design.md > Tests do not read the configuration of the developer — "The process helper sets `MU_CONFIG` to an empty directory of the test for each `mu` that it starts, as it clears `MU_SOC" | Add one base launcher in tests/support that sets the isolated environment (no MU_SOCKET, MU_CONFIG at an empty directory), and make ServerBuilder, PtyClient, mu_on_pty and mu_with start from it. |  |  |
| R-14 | fit | should-fix | in-change | The decision page calls the window width `width`, while the spec and TelnetSettings call it `columns`; task 6.2 edits the page but only for figment2. | tasks.md > 6. The server and the docs — "Edit `docs/src/content/docs/decisions/world-type-profiles.md` to name `figment2` and say why." | Make task 6.2 also replace `width` with `columns` on the decision page. |  |  |
| R-15 | fit | should-fix | in-change | The file-name rule needs the world-id part check, which id.rs already has as the private is_part, and the design names no reuse. | specs/world-config/spec.md > Requirement: Each character file makes one world — "A file name that is not a part of a world id SHALL be an error that names the file." | State in the design that TomlDirSource checks game, character and type file names with is_part, made pub(crate), or by parsing a WorldId. |  |  |
| R-16 | fit | should-fix | in-change | mu world show writes TOML with a comment on each field, and the design names no TOML writer, which leaves hand-written quoting of values such as a login template. | design.md > `mu world` — "`mu world show <id>` prints the resolved world as TOML. Each field ends with a comment `# <source>`" | Name the writer in the decision: format each value with the toml crate, or build a toml_edit document with a suffix comment on each key, and add that crate to mu-cli. |  |  |
| R-17 | invariants | should-fix | in-change | The login pair check is tested with a name and no secret only. A secret with no name, the other side of the rule, has no test. | specs/world-config/spec.md > Requirement: A world has a place to connect — "A world with `login.name` or `login.secret` SHALL have both." | Add a scenario and the test login_secret_with_no_name that expects stderr to name alpha/aria and login.name. |  |  |
| R-18 | invariants | should-fix | in-change | The loop test uses two types. A type that extends itself, the loop of length one, has no test. | specs/world-config/spec.md > Scenario: A loop of types — "- **GIVEN** `types/a.toml` sets `extends = "b"`, and `types/b.toml` sets `extends = "a"`" | Add a test type_that_extends_itself with types/a.toml setting extends = "a", expecting stderr to name types/a.toml. |  |  |
| R-19 | invariants | should-fix | in-change | The spec says an empty MU_CONFIG counts as unset, and no scenario or test sets MU_CONFIG to the empty string. | specs/world-config/spec.md > Requirement: The configuration directory — "An empty `MU_CONFIG` SHALL count as unset." | Add a scenario and a test with MU_CONFIG set to empty and XDG_CONFIG_HOME pointing at a directory with a world, expecting that world. |  |  |
| R-20 | invariants | should-fix | in-change | The spec says a relative XDG_CONFIG_HOME is ignored. The tests cover only an empty value, not a relative one. | specs/world-config/spec.md > Requirement: The configuration directory — "If that variable is not set, empty or relative, the directory SHALL be `.config/mu` under `HOME`." | Add a test with XDG_CONFIG_HOME set to a relative path that holds a world, run from its parent, expecting the world under HOME instead. |  |  |
| R-21 | invariants | should-fix | in-change | Only the host is tested against the plugin trust check. port, tls and the nested login.name and login.secret, a different path through a table, are untested. | tasks.md > 4. Rules, scripts and trust — "Add `plugin_cannot_set_the_host` and `plugin_can_set_a_preference` to `crates/mu-core/tests/world_config.rs`" | Make plugin_cannot_set_the_host a table test over all five fields, each expecting an error that names colors and the field. |  |  |
| R-22 | invariants | should-fix | in-change | The design checks charset against the names of Charset, which are upper case, while the spec writes iso-8859-1. No test pins the case rule. | crates/mu-core/src/framer.rs:221 — "Self::Latin1 => "ISO-8859-1"," | State that the charset name is matched case-insensitively, and add a test that accepts ISO-8859-1, iso-8859-1 and utf-8 in a user layer. |  |  |
| R-23 | invariants | should-fix | in-change | Nothing states whether rules and scripts share one id space, or refuses a repeated id in one layer's array, yet disable and replace both act by id alone. | specs/world-config/spec.md > Requirement: Rules and scripts accumulate — "The array `disable` of a layer SHALL remove each inherited entry with an id that it names." | State whether disable acts on rules, scripts or both, refuse a repeated id within one array of a layer with an error that names the file and the id, and test both. |  |  |
| R-24 | invariants | should-fix | in-change | The design parses login.secret with the from_str parse of the secret reference type, whose error Display prints the rejected text, so wrapping that error puts the password in stderr. | crates/mu-core/src/secret.rs:22 — "#[error("{0:?} is not a secret reference of the form env:<NAME>")]" | Have the layer parse map BadSecretRef to a ConfigError that names the file and login.secret only, never its source or Display, and say so in task 2.1. |  | severity? blocker 0.69 |
| R-25 | invariants | should-fix | in-change | login.name, login.template and keepalive.command become user input written as a line, and nothing refuses CR or LF in them. Only the password has that check. A probe parsed a name with CR LF. | crates/mu-core/src/session.rs:339 — "self.write_line(&login.line(&password), depth).await?;" | Refuse CR and LF in login.name, login.template and keepalive.command at the layer parse, with an error that names the file and the field. Add a spec scenario and a test for each field. |  | severity? blocker 0.81 |
| R-26 | invariants | should-fix | in-change | A shared plugin may set login.template, and the template decides where the password goes, so a plugin can route the password to another target although it cannot set the secret. | specs/world-config/spec.md > Requirement: A shared plugin layer cannot set the connection — "A layer from a shared plugin source SHALL NOT set `host`, `port`, `tls`, `login.name` or `login.secret`." | Add login.template to the fields a shared plugin layer cannot set, or refuse a plugin template that holds {password}. Add a test plugin_cannot_set_the_login_template. |  | severity? blocker 0.61 |
| R-27 | invariants | should-fix | in-change | An empty layer has login and keepalive as None, so empty_layer_gives_figment_no_keys never serializes the nested fields. A missing skip on login.template or keepalive.seconds passes it. | design.md > Enforcement of each rule — "If one field lacks it, the test `empty_layer_gives_figment_no_keys` fails." | Add a unit test that serializes a layer that sets only login.name, and one that sets only keepalive.command, and asserts the exact set of keys. Run the removal check on a nested field. |  |  |
| R-28 | invariants | should-fix | in-change | Nothing checks after the merge that keepalive has both a command and seconds. A layer that sets one half yields a keepalive the resolved type cannot hold. | design.md > The resolved world — "and `keepalive`, an `Option` of a command and a period." | Add to the post-merge check that keepalive has both command and seconds or neither, with an error that names the world and the missing field. Add a spec scenario and a test. |  | severity? blocker 0.70 |
| R-29 | invariants | should-fix | in-change | inv-7: The design checks ranges of columns and rows but no artifact names the range. NAWS sends them as u16, so 0 and 65536 are the edges.; inv-8: No artifact names the range of keepalive.seconds, so a zero period is accepted as a keepalive that #54 then has to run. | design.md > Layers are parsed alone, then merged — "It also checks the ranges of `columns`, `rows`, `port` and `keepalive.seconds`." | inv-7: State the range of columns and rows in the requirement Each field is known, such as 1 to 65535, and test 0 and 65536 for each.; inv-8: Name the range of keepalive.seconds in the spec, with zero refused, and add a test keepalive_of_zero_seconds. |  |  |
| R-30 | invariants | should-fix | in-change | The only port test uses 70000, far past the limit. Port 0, 65535 and 65536 are untested, and a u16 accepts 0. | specs/world-config/spec.md > Scenario: A port out of range — "- **GIVEN** the game `alpha` sets `port = 70000`" | Test port = 0 refused, 65535 accepted and 65536 refused, and state the range 1 to 65535 in the requirement. |  |  |
| R-31 | spec-lint | should-fix | in-change | The scenario 'A layer replaces an inherited rule' names no pattern for the game's hl-page rule, so its THEN cannot be checked. | specs/world-config/spec.md > Rules and scripts accumulate — "the game has a rule `hl-page` with another pattern" | Give the game's pattern as a literal in the GIVEN, and the exact printed line of hl-page in the THEN. |  |  |
| R-32 | spec-lint | should-fix | in-change | The scenario 'A table merges field by field' gives no values for login.template, login.name or login.secret, so its THEN cannot be checked. | specs/world-config/spec.md > Layers merge from global to character — "the game `alpha` sets `login.template`, and `characters/alpha/aria.toml` sets only `login.name` and `login.secret`" | Give the template, the name and the secret as literal values in the GIVEN, and the expected login lines of stdout in the THEN. |  |  |
| R-33 | test-coverage | should-fix | in-change | The scenario 'Help for one verb' is in the delta but no task names it or its test. | tasks.md > 2. The directory and the layers — "In `crates/mu-cli/tests/cli.rs`, make `help_lists_verbs` expect the four verbs." | In task 2.2, name `help_for_one_verb` and check it still passes with the fourth verb `world` in the help output. |  |  |
| R-34 | fit | suggestion | in-change | LayerSource::layers takes no input, but the Lua decision lets configuration Lua read its parent layer, so the Lua change must reshape the seam the design says it will not touch. | design.md > The layer seam and its trust — "pub trait LayerSource { fn layers(&self) -> Result<Vec<Layer>, ConfigError>; }" | Either give the seam the merged parent fields of the layer it builds, or drop the no-change goal and name the Lua change as the one that widens LayerSource. |  |  |
| R-35 | invariants | suggestion | in-change | The accumulation of rules over any sequence of layers has invariants over an unbounded domain: ids stay unique and order follows first insertion, minus disabled ids. | design.md > Rules and scripts accumulate in the resolver — "A replace keeps the place of the old entry." | Add a proptest (mu-core already uses proptest, see crates/mu-core/src/id.rs) that generates layer sequences of rules and disables and checks unique ids and the order against a simple model. |  |  |
| R-36 | invariants | suggestion | in-change | mu world show must print TOML that parses for any string value, including rule tables kept as values with quotes, hash signs and newlines. | specs/world-config/spec.md > Requirement: Show names the source of each value — "`mu world show <id>` SHALL print the resolved world as TOML." | Add a proptest, after the pattern of crates/mu-core/src/id.rs, that renders a resolved world with arbitrary strings and checks that the output parses and gives back the same values. |  |  |

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

## Judge

judge: 60 questions, 0.003 USD, stopped: cap of 60 questions

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| The flag wins over the environment | 2.2 | covered |  |
| The default directory | 2.2 | covered |  |
| The directory under the home directory | 2.2 | covered |  |
| A missing directory | 2.2 | covered |  |
| Worlds from character files | 2.2 | covered |  |
| A character with no game | 2.2 | covered |  |
| A file name outside the id rule | 2.2 | covered |  |
| A stray file | 2.2 | covered |  |
| The last layer wins | 3.2 | covered |  |
| A global default reaches each world | 3.2 | covered |  |
| A table merges field by field | 3.2 | covered |  |
| A user type extends a built-in type | 3.2 | covered |  |
| An unknown type | 3.2 | covered |  |
| A game with no type | 3.2 | covered |  |
| A loop of types | 3.2 | covered |  |
| A character that names a parent | 3.2 | covered |  |
| Entries from each layer | 4.1 | covered |  |
| A layer disables an inherited rule | 4.1 | covered |  |
| A layer replaces an inherited rule | 4.1 | covered |  |
| A disable with no target | 4.1 | covered |  |
| Each built-in type reads as quirks | 5.1 | covered |  |
| A global preference is not hidden by a type | 5.1 | covered |  |
| A user type with a built-in name | 5.1 | covered |  |
| An unknown field | 2.2 | covered |  |
| A port out of range | 2.2 | covered |  |
| A character set that the engine cannot read | 2.2 | covered |  |
| A password in a file | 2.2 | covered |  |
| A secret that is not a reference | 2.2 | covered |  |
| Show does not read the secret | 3.2 | covered |  |
| A plugin sets the host | 4.2 | covered |  |
| A plugin sets a preference | 4.2 | covered |  |
| A world with no port | 3.2 | covered |  |
| A login name with no secret | 3.2 | covered |  |
| A world that logs in by hand | 3.2 | covered |  |
| Sources of fields | 3.2 | covered |  |
| An unknown world | 3.2 | covered |  |
| A bad configuration stops the start | 6.1 | covered |  |
| A fake world with a configured id | 6.1 | covered |  |
| The secret is not read at the start | 6.1 | covered |  |
| Help lists the verbs | 2.2 | covered |  |
| Help for one verb | 2.2 | partial |  |
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

- approach: Checked and holding: ResolvedWorld and Login in crates/mu-core/src/world.rs, WorldSetup in portal.rs, SecretRef in secret.rs, TelnetSettings default 120 by 40, the socket order in socket.rs, convert.rs mapping NotConnected to FailedPrecondition, the decision pages, the telnet fixtures, help_lists_verbs and two_worlds_connect.
- approach: The rejection of schematic holds: deny.toml allows no MPL-2.0 license.
- approach: No open question section, and the migration plan has no deployed configuration, so no-rollback does not apply.
- fit: prior.json is empty, so this is a first round and no finding carries prior.
- fit: The library rows of openspec/config.yaml were read; no row names a configuration or XDG helper beyond the socket module.
- invariants: I1: A layer sets only known fields \| enforced at deny_unknown_fields on LayerFields (task 2.1) \| boundary test unknown_field
- invariants: I2: An error never holds a secret value \| enforced at none: toml_edit error Display prints the source line, BadSecretRef Display prints the text \| boundary test password_in_a_file, secret_that_is_not_a_reference (both will fail as designed)
- invariants: I3: login.secret is env:<NAME> \| enforced at the from_str parse in secret.rs:28 \| boundary test secret_that_is_not_a_reference
- invariants: I4: port is in range \| enforced at the layer parse (u16 plus range check, task 2.1) \| boundary test port_out_of_range at 70000 only
- invariants: I5: columns, rows, keepalive.seconds are in range \| enforced at the layer parse, range unnamed \| boundary test none
- invariants: I6: charset is a Charset name \| enforced at the layer parse \| boundary test charset_the_engine_cannot_read (negative only)
- invariants: I7: A login line holds no CR or LF \| enforced at secret.rs:178 for the password only \| boundary test none for name, template, keepalive.command
- invariants: I8: A layer that does not set a field keeps the earlier value \| enforced at skip_serializing_if on each optional field \| boundary test empty_layer_gives_figment_no_keys (top level only), table_merges_field_by_field (login.template only)
- invariants: I9: A loop in extends fails \| enforced at the chain walk with a seen set \| boundary test loop_of_types (length two)
- invariants: I10: A user type never has a built-in name \| enforced at the resolver \| boundary test user_type_with_a_built_in_name
- invariants: I11: A built-in type sets only quirks \| enforced at Quirks with deny_unknown_fields \| boundary test built_in_types_hold_only_quirks
- invariants: I12: A shared plugin layer never sets host, port, tls, login.name, login.secret \| enforced at the trust check (task 4.2) \| boundary test plugin_cannot_set_the_host (host only)
- invariants: I13: After the merge each world has host and port, and the login pair is both or neither \| enforced at the post-merge check (task 3.1) \| boundary test world_with_no_port, login_name_with_no_secret (one side)
- invariants: I14: keepalive is both command and seconds or neither \| enforced at none \| boundary test none
- invariants: I15: A disable id names an inherited entry \| enforced at the accumulation \| boundary test disable_with_no_target
- invariants: I16: Resolved rule and script ids are unique and keep first-insertion order \| enforced at the accumulation \| boundary test layer_replaces_an_inherited_rule
- invariants: I17: Configuration directory order flag, MU_CONFIG (empty is unset), XDG_CONFIG_HOME (empty or relative ignored), HOME \| enforced at ConfigArgs (task 2.2) \| boundary test four scenario tests, none for empty MU_CONFIG or relative XDG_CONFIG_HOME
- invariants: I18: mu world list is in WorldId order (game, then character) \| enforced at Ord for WorldId (id.rs:78) \| boundary test the sort test in id.rs and worlds_from_character_files
- invariants: I19: A bad configuration leaves no socket \| enforced at resolve before bind (task 6.1) \| boundary test bad_configuration_stops_the_start
- invariants: I20: An id is never both fake and configured \| enforced at mu server start (task 6.1) \| boundary test fake_world_with_a_configured_id
- invariants: I21: No resolution reads a secret \| enforced at the resolver taking no SecretStore \| boundary test show_does_not_read_the_secret, server_does_not_read_a_configured_secret
- invariants: I22: Each PortalError maps to one code \| enforced at the exhaustive match in convert.rs:13 \| boundary test portal_errors_map_to_codes, which task 6.1 does not extend with NoTransport
- invariants: I23: The modified requirements drop no scenario: cli-entrypoint keeps both scenarios, and world-sessions keeps Two worlds connect. No weakened finding.
- invariants: Probe: a toml_edit 0.25.19 deserialize under the handoff directory (probe/) printed the line with hunter2 for login.password and for a bad login.secret, and accepted login.name with CR LF.
- invariants: global.toml holding extends has no test; only the character side is tested. Left as a note.
- invariants: The built-in quirk values are not in the repository yet; task 1.1 fills them, so they were not checked here.
- spec-lint: Scenario shape: every requirement of the three delta specs has a scenario, and every scenario has a WHEN or GIVEN and a THEN.
- spec-lint: Requirement subjects: each requirement has a task line with two or more distinctive words, so no untasked-requirement finding.
- spec-lint: Build targets: mise tasks check and docs:check exist; packages mu-core, mu-cli and mu-server exist; the test files and the docs page named by tasks exist; review_lint.py exists at the path given in task 7.1.
- spec-lint: BREAKING: no marker in proposal.md or in the delta specs, so no breaking-mismatch.
- spec-lint: Open questions: design.md has no open-questions section, so no assumed-answer finding.
- spec-lint: Design 'Checked facts' rows are still 'To check:' placeholders. Task 5.1 takes the built-in type values from that table, so the values are not yet fixed. This is not a form defect under the categories of this lens, and it is recorded here only.
- test-coverage: Only one pass was written: the change is small enough to read in full before the first write.

## Ledger

| id | first | last | severity | scope | location | disposition | basis | override | claim |
|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The merge of fields uses `figment2`"],"Thus the errors that a user sees come from `toml`, with the file, the field and the line."] |  |  |  | app-1: The design shows the toml error to the user, and the Display of a toml error prints the whole source line, so the value of a password field in a character file reaches stderr.; inv-1: No task strips the source line from TOML errors, and toml_edit prints that line. A probe printed hunter2 for both login.password and a bad login.secret. |
| R-2 | 1 | 1 | blocker | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"Thus the error names that file and that field. The parse checks `login.secret` with"] |  |  |  | The design checks the login reference with the FromStr parse of the reference type, whose error keeps and prints the bad text, so a plain password in that field reaches stderr. |
| R-3 | 1 | 1 | blocker | in-change | ["artifact",["specs/world-config/spec.md","Requirement: A world has a place to connect"],"Without `login.name` and `login.secret`, a `login.template` SHALL NOT count."] |  |  |  | No decision produces the requirement "A world has a place to connect": the required host and port, the login pair, and a template that does not count alone. Only task 3.1 names the check. |
| R-4 | 1 | 1 | blocker | in-change | ["artifact",["design.md","Configured worlds in the server"],"Then it adds a `WorldSetup` with no transport and no password for each configured world."] |  |  |  | No decision refuses an id that is both a fake and a configured world; Portal::start keys worlds by id in a BTreeMap, so a duplicate would silently replace one sender. |
| R-5 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The layer seam and its trust"],"`TomlDirSource` reads the directory."] |  |  |  | No decision says how TomlDirSource maps file names to ids and refuses a name outside the id rule, a character with no game file, or a stray `.toml` file. |
| R-6 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The chain of a world"],"The chain starts at the character and the game. Next comes the type that the game `extends`"] |  |  |  | The chain decision covers only the loop and the built-in name; nothing makes `extends` required in a game, refused in global.toml and a character, or an unknown name an error. |
| R-7 | 1 | 1 | blocker | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"and the character set against the names of `Charset`"] |  |  |  | The design checks the character set against the names of Charset, which are `UTF-8` and `ISO-8859-1`, but the requirement makes tinymux set `iso-8859-1`; an exact check fails the built-in type. |
| R-8 | 1 | 1 | blocker | in-change | ["artifact",["design.md","The merge of fields uses `figment2`"],"The resolver gives `figment2` only typed layers that `toml` parsed, through `Serialized`."] |  |  |  | app-9: The resolver in mu-core parses each file with `toml`, but the plan adds only `figment2` and `serde` to mu-core and to its ALLOWED_CRATES, so the boundary test refuses the parse path.; fit-4: The plan names two TOML readers: figment2 with its TOML feature, and the toml crate that parses each layer before Serialized; task 1.2 adds only the first to mu-core.; app-10: The TOML feature of figment2 serves nothing: the resolver passes figment2 only Serialized providers, so figment2 never reads a file. |
| R-9 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Context"],"The process helper in `crates/mu-cli/tests/support/process.rs` clears `MU_SOCKET` for each `mu` that a test starts."] |  |  |  | Only ServerBuilder clears MU_SOCKET; `mu_with` and `mu_on_pty` start `mu` with the inherited environment, so the cited helper does not cover each `mu` a test starts. |
| R-10 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The configuration directory"],"`mu-cli` gets `ConfigArgs`, with `--config` and the order `--config`, `MU_CONFIG`, `XDG_CONFIG_HOME/mu`, then `HOME/.config/mu`, on Linux and macOS alike."] |  |  |  | ConfigArgs copies the shape of SocketArgs: a second empty-means-unset value parser and a second relative-XDG filter, with no shared helper. |
| R-11 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The resolved world"],"The fake world keeps `with_default_login`, which fills the new fields with the defaults."] |  |  |  | The defaults of charset, columns, rows and the login template get two new homes (the resolver's default layer and with_default_login) beside TelnetSettings::default and DEFAULT_LOGIN_TEMPLATE. |
| R-12 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The layer seam and its trust"],"pub struct Layer { pub key: LayerKey, pub source: Source, pub trust: Trust, pub fields: LayerFields }"] |  |  |  | The new Source type of a layer reuses the name of mu_core::Source, which the crate root already exports as the source of an event. |
| R-13 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Tests do not read the configuration of the developer"],"The process helper sets `MU_CONFIG` to an empty directory of the test for each `mu` that it starts, as it clears `MU_SOCKET`."] |  |  |  | The design adds the MU_CONFIG scrub to ServerBuilder alone, while three other test launchers build their own environment and mu client starts a server that inherits it. |
| R-14 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","6. The server and the docs"],"Edit `docs/src/content/docs/decisions/world-type-profiles.md` to name `figment2` and say why."] |  |  |  | The decision page calls the window width `width`, while the spec and TelnetSettings call it `columns`; task 6.2 edits the page but only for figment2. |
| R-15 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Each character file makes one world"],"A file name that is not a part of a world id SHALL be an error that names the file."] |  |  |  | The file-name rule needs the world-id part check, which id.rs already has as the private is_part, and the design names no reuse. |
| R-16 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","`mu world`"],"`mu world show <id>` prints the resolved world as TOML. Each field ends with a comment `# <source>`"] |  |  |  | mu world show writes TOML with a comment on each field, and the design names no TOML writer, which leaves hand-written quoting of values such as a login template. |
| R-17 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: A world has a place to connect"],"A world with `login.name` or `login.secret` SHALL have both."] |  |  |  | The login pair check is tested with a name and no secret only. A secret with no name, the other side of the rule, has no test. |
| R-18 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: A loop of types"],"- **GIVEN** `types/a.toml` sets `extends = \\"b\\"`, and `types/b.toml` sets `extends = \\"a\\"`"] |  |  |  | The loop test uses two types. A type that extends itself, the loop of length one, has no test. |
| R-19 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: The configuration directory"],"An empty `MU_CONFIG` SHALL count as unset."] |  |  |  | The spec says an empty MU_CONFIG counts as unset, and no scenario or test sets MU_CONFIG to the empty string. |
| R-20 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: The configuration directory"],"If that variable is not set, empty or relative, the directory SHALL be `.config/mu` under `HOME`."] |  |  |  | The spec says a relative XDG_CONFIG_HOME is ignored. The tests cover only an empty value, not a relative one. |
| R-21 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","4. Rules, scripts and trust"],"Add `plugin_cannot_set_the_host` and `plugin_can_set_a_preference` to `crates/mu-core/tests/world_config.rs`"] |  |  |  | Only the host is tested against the plugin trust check. port, tls and the nested login.name and login.secret, a different path through a table, are untested. |
| R-22 | 1 | 1 | should-fix | in-change | ["path","crates/mu-core/src/framer.rs",221] |  |  |  | The design checks charset against the names of Charset, which are upper case, while the spec writes iso-8859-1. No test pins the case rule. |
| R-23 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Rules and scripts accumulate"],"The array `disable` of a layer SHALL remove each inherited entry with an id that it names."] |  |  |  | Nothing states whether rules and scripts share one id space, or refuses a repeated id in one layer's array, yet disable and replace both act by id alone. |
| R-24 | 1 | 1 | should-fix | in-change | ["path","crates/mu-core/src/secret.rs",22] |  |  |  | The design parses login.secret with the from_str parse of the secret reference type, whose error Display prints the rejected text, so wrapping that error puts the password in stderr. |
| R-25 | 1 | 1 | should-fix | in-change | ["path","crates/mu-core/src/session.rs",339] |  |  |  | login.name, login.template and keepalive.command become user input written as a line, and nothing refuses CR or LF in them. Only the password has that check. A probe parsed a name with CR LF. |
| R-26 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Requirement: A shared plugin layer cannot set the connection"],"A layer from a shared plugin source SHALL NOT set `host`, `port`, `tls`, `login.name` or `login.secret`."] |  |  |  | A shared plugin may set login.template, and the template decides where the password goes, so a plugin can route the password to another target although it cannot set the secret. |
| R-27 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"If one field lacks it, the test `empty_layer_gives_figment_no_keys` fails."] |  |  |  | An empty layer has login and keepalive as None, so empty_layer_gives_figment_no_keys never serializes the nested fields. A missing skip on login.template or keepalive.seconds passes it. |
| R-28 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The resolved world"],"and `keepalive`, an `Option` of a command and a period."] |  |  |  | Nothing checks after the merge that keepalive has both a command and seconds. A layer that sets one half yields a keepalive the resolved type cannot hold. |
| R-29 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Layers are parsed alone, then merged"],"It also checks the ranges of `columns`, `rows`, `port` and `keepalive.seconds`."] |  |  |  | inv-7: The design checks ranges of columns and rows but no artifact names the range. NAWS sends them as u16, so 0 and 65536 are the edges.; inv-8: No artifact names the range of keepalive.seconds, so a zero period is accepted as a keepalive that #54 then has to run. |
| R-30 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Scenario: A port out of range"],"- **GIVEN** the game `alpha` sets `port = 70000`"] |  |  |  | The only port test uses 70000, far past the limit. Port 0, 65535 and 65536 are untested, and a u16 accepts 0. |
| R-31 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Rules and scripts accumulate"],"the game has a rule `hl-page` with another pattern"] |  |  |  | The scenario 'A layer replaces an inherited rule' names no pattern for the game's hl-page rule, so its THEN cannot be checked. |
| R-32 | 1 | 1 | should-fix | in-change | ["artifact",["specs/world-config/spec.md","Layers merge from global to character"],"the game `alpha` sets `login.template`, and `characters/alpha/aria.toml` sets only `login.name` and `login.secret`"] |  |  |  | The scenario 'A table merges field by field' gives no values for login.template, login.name or login.secret, so its THEN cannot be checked. |
| R-33 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The directory and the layers"],"In `crates/mu-cli/tests/cli.rs`, make `help_lists_verbs` expect the four verbs."] |  |  |  | The scenario 'Help for one verb' is in the delta but no task names it or its test. |
| R-34 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","The layer seam and its trust"],"pub trait LayerSource { fn layers(&self) -> Result<Vec<Layer>, ConfigError>; }"] |  |  |  | LayerSource::layers takes no input, but the Lua decision lets configuration Lua read its parent layer, so the Lua change must reshape the seam the design says it will not touch. |
| R-35 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","Rules and scripts accumulate in the resolver"],"A replace keeps the place of the old entry."] |  |  |  | The accumulation of rules over any sequence of layers has invariants over an unbounded domain: ids stay unique and order follows first insertion, minus disabled ids. |
| R-36 | 1 | 1 | suggestion | in-change | ["artifact",["specs/world-config/spec.md","Requirement: Show names the source of each value"],"`mu world show <id>` SHALL print the resolved world as TOML."] |  |  |  | mu world show must print TOML that parses for any string value, including rule tables kept as values with quotes, hash signs and newlines. |
| R-37 | 1 | 1 | should-fix | in-change | ["artifact",["review","lens-status"],"approach partial"] |  |  |  | lens approach reported partial, gaps: 1 |
