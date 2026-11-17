# Design

## Context

See proposal.md, section Why. These are the facts of the code before this change, which shape the design. The change moves some of them:

- `world.rs` becomes the module `world/`, and `is_part` becomes `pub(crate)`.
- `SocketValue` and `parse_socket` become `PathValue` and `parse_path` in `env_path.rs`.
- `mu-core` depends on `toml` and `figment2`.
- `mu_with` points `MU_CONFIG` at an empty directory.

The facts before the change:

- `ResolvedWorld` in `crates/mu-core/src/world.rs` holds only the id and an optional `Login`. `mu server` builds it in `crates/mu-cli/src/server.rs` from each `--fake-world` flag, with `DEFAULT_LOGIN_TEMPLATE` and the character part of the id as the login name.
- `WorldSetup` in `crates/mu-core/src/portal.rs` holds the world, the password, the transport, the framer, the hooks and the handlers. The portal connects every world at the start.
- `SecretRef` in `crates/mu-core/src/secret.rs` parses `env:<NAME>`, and `load_password` reads it. The parse is the only check that a text is a reference.
- `TelnetSettings` in `crates/mu-core/src/framer.rs` holds the character set (`Charset::Utf8` or `Charset::Latin1`), the window size and the option policy. Its default is UTF-8 and 120 by 40.
- The socket path rules in `crates/mu-cli/src/socket.rs` read `--socket`, then `MU_SOCKET`, then `XDG_RUNTIME_DIR`. The configuration directory follows the same shape.
- Four test launchers start `mu`. `ServerBuilder` and `PtyClient` in `crates/mu-cli/tests/support/process.rs` clear `MU_SOCKET`. `mu_on_pty` in the same file and `mu_with` in `crates/mu-cli/tests/support/mod.rs` pass the inherited environment.
- `is_part` in `crates/mu-core/src/id.rs` checks one part of a world id. It is private.
- `SocketValue` and `parse_socket` in `crates/mu-cli/src/socket.rs` read an empty value as none.
- The crate root of `mu-core` exports `Source`, the source of an event.
- `mu-cli` has `toml` only as a dev-dependency. `mu-core` depends on no TOML crate.
- The crate root of `mu-core` also exports `Origin`, the maker of a command.
- `has_line_break` in `crates/mu-core/src/framer.rs` is the check for CR and LF that `secret.rs` and `session.rs` share.
- The decision page `world-type-profiles.md` names `figment`. The page `telnet-engine.md` records the quirks that the spike of issue 7 found (pull request #32).

## Goals / Non-Goals

**Goals:**

- One resolver that turns a set of layers into one `ResolvedWorld` per world, with the source of each field.
- A layer seam that a Lua source can implement later. A Lua source that reads its parent layer widens `LayerSource` in its own change.
- Errors that name the file and the field, and never a secret value.

**Non-Goals:**

- A schema version in the files. The files have no users yet.
- Kept comments or order in a file that the client edits. Nothing writes the files.
- Plugin layers from disk, and the place of a plugin layer in a real chain. A test gives the resolver a plugin layer at a fixed place.
- Environment variables that override single fields, such as `MU_COLUMNS`.
- The system directories of `XDG_CONFIG_DIRS`, such as `/etc/xdg/mu`. The XDG Base Directory specification lists them after `XDG_CONFIG_HOME`, but a system directory needs its own layer and trust level. Issue #84 tracks this gap.

## Decisions

### Layers are parsed alone, then merged

The resolver reads each file with the `toml` crate into a typed struct with `deny_unknown_fields`, before any merge. A bad field fails in its own file, and so does a bad type or a value out of range. Thus the error names that file and that field. The derived `Deserialize` of a struct also reads an array, by the order of its fields, and drops extra elements. Thus the tables `login` and `keepalive` go through a guard, `table` in `layer.rs`, that accepts only a table. An array there fails as a bad type of that field.

Each file parses into one struct, `LayerFile`, which holds the fields of `LayerFields` and an optional `extends`. The struct does not use `#[serde(flatten)]`, because `deny_unknown_fields` does not work with it. The walk of the directory then checks `extends` for the kind of the file. A game requires it, and a type accepts it. `global.toml` and a character file refuse it, with an error that names the file and `extends`.

The resolver checks these values on every layer, before the trust check and the merge. It does so for each source: a file, a built-in type, a plugin, and later Lua. Thus the checks have one place. An error names the origin of the layer and the field.

- `login.secret` with `SecretRef::from_str`. The parse maps a `BadSecretRef` to a `ConfigError` that names the file and `login.secret`. The error never holds the text of `BadSecretRef` or its `Display`, because that text is the rejected value.
- `charset` with `Charset::from_name`, a new function of `framer.rs` that matches a name with no regard to case. `charset_answer` of the framer moves to it, so the match has one copy. Thus `iso-8859-1` and `ISO-8859-1` are the same name.
- `port`, `columns` and `rows` from 1 to 65535, and `keepalive.seconds` from 1 to 86400.
- `login.name`, `login.template` and `keepalive.command` hold no CR and no LF, because the session writes each one as a line. The check uses `has_line_break` of `framer.rs`, which `secret.rs` and `session.rs` already share.
- `login.template` holds `{password}`. Without it, the login sends no password.
- Each entry of `rules` and `scripts` has an `id` that is a string.
- No id repeats in one array of one layer.

The parse runs the deserializer of `toml` through `serde_path_to_error`. That crate gives the path of the field that failed, such as `login.password` or `port`, for every error of `serde`. The `toml` error keeps its own path private, so the project needs this crate to name the field. A probe on 2026-11-17 used `toml` 1.2.1. It gave the path, the span and the message for three errors. They were an unknown field, a value out of range, and a value of the wrong type under `login`.

A `ConfigError` of a parse holds the file, the path, and the line and the column that the span gives. It never holds the `Display` of the `toml` error, because that text quotes the source line, and the line can hold a password. For a path outside `login`, it keeps the message of `toml`. For a path under `login`, it says only that the value is not valid, because a message of `serde` can quote the value.

Alternative: merge first and check the merged result. The error then names the field but not always the file that broke it, and a value that a later layer hides goes unchecked.

### The defaults

One `WorldDefaults` value holds the defaults of a world. It comes from `TelnetSettings::default()` and `DEFAULT_LOGIN_TEMPLATE`. The default layer of the resolver and `with_default_login` both read it. Thus no default number or template is written twice.

### The merge of fields uses `figment2`

The resolver turns each layer into one provider, named by its source, and merges them with `merge` in layer order. It reads the merged fields with `extract`, and the source of each field with `find_metadata`. Only the fields go to `figment2`. The arrays `rules`, `scripts` and `disable` stay out, because `merge` replaces an array and `admerge` cannot remove an entry by id.

`figment2` is a fork of `figment` with the same API. It carries the open pull requests of `figment`. The project uses it with no features, because the `toml` crate reads the files. The decision page names `figment`, so task 6.2 edits the page.

The research on 2026-11-17 compared these crates. The hard need is the source of each merged field, because `mu world show` prints it.

| Crate | Last release | Source of each field | Fit |
|---|---|---|---|
| `figment` | 2024-05, 0.10.19 | Yes, `find_metadata` | In 2026-04 the author promised more work, but no release followed. It depends on `toml` 0.8. |
| `figment2` | 2026-04, 0.11.5 | Yes, the same API | One maintainer. Foundry moved to it in 2026-01. It does only the merge, which the resolver lacks. |
| `config-rs` | 2026-09, 0.15.27 | Only in some errors | The most used crate, but a tree of values with no typed layers and no source for each merged value. |
| `schematic` | 2026-11, 0.21.1 | No. The loader keeps the layers, and the caller searches them. | Typed partial layers, merge functions and validation. Its `extends` names files or URLs, not types, and it brings `miette` and `garde`. |
| `confique` | 2025-10, 0.4.0 | No | Typed layers with `with_fallback`, one maintainer, and `toml` 0.9. |

A debate on 2026-11-17 between two agents, under a protocol fixed before it began, kept `figment2`. The scores were 4.00 for `figment2` and 3.40 for `schematic`. `schematic` fails the license policy of `deny.toml`, because it brings `option-ext` under MPL-2.0. The maintenance of `schematic` is better, by less than the two points that the protocol needs to decide alone.

The resolver gives `figment2` only typed layers that `toml` parsed, through `Serialized`. Thus the errors that a user sees come from the parse of each layer. A `None` field in `Serialized` hides the value of an earlier layer. Thus each optional field skips `None` in serialization.

Upstream `figment` has the same API on its main branch since 2026-10, but no release. When it releases, a rename of the package in `Cargo.toml` moves the project back to it.

Alternatives: `figment`, with no release in two years and an old `toml`. `schematic`, which is the best second choice. It replaces the typed parse of each layer, but the resolver then writes the source search that `figment2` gives. A merge of `toml::Value` tables that the project writes, which breaks the rule to reuse open source first.

### Rules and scripts accumulate in the resolver

The resolver walks the layers in merge order. For each layer, it first removes each entry that `disable` names, then appends or replaces each entry of `rules` and `scripts` by id. A replace keeps the place of the old entry. An entry is an `id` and the rest of its TOML table, kept as a value. Each entry records the source of the layer that added it. A `disable` id with no inherited entry is an error, so a typo does not pass in silence.

Rules and scripts share one space of ids in a world. Thus a `disable` id names one entry, a rule or a script. A rule and a script with the same id is an error that names the file of the later one.

### The chain of a world

The chain starts at the character and the game. Next comes the type that the game `extends`, and each type that a type `extends`, up to a type with no `extends`. Then come `global.toml` and the defaults. The resolver walks this chain with a set of the types it saw. Before it builds any chain, it also walks from each file in `games/` and in `types/`. Thus a game with no character and a type that no game uses are checked too. Thus a loop fails with the list of the files in it, and so does a loop with no game. A type that extends itself is a loop of one file. A name in `extends` that is neither a built-in type nor a file in `types/` is an error that names the file and the name. A user type shadows nothing: a user type with a built-in name is an error. Thus a name in `extends` has one meaning.

### The walk of the directory

`TomlDirSource` accepts these paths under the configuration directory, and no others:

- `global.toml`
- `types/<name>.toml`
- `games/<game>.toml`
- `characters/<game>/<character>.toml`

It checks each name, game and character with `is_part`, which becomes `pub(crate)`. Thus a file name and a world id follow one rule. Any other file whose name ends in `.toml`, in any case, is an error that names its path. Thus `games/beta.TOML` and `games/.toml` are errors, and are not skipped. A character with no file `games/<game>.toml` is an error that names both paths. The resolver makes this check, so it holds for each source of layers. Files whose names do not end in `.toml` are not read. The walk follows links, and an entry that it cannot read, such as a link to nothing, is an error that names the entry. It is not skipped, because a skipped link to a missing directory of games hides that directory.

### Built-in types and the quirk schema

The built-in types live in `crates/mu-core/src/world/types/<name>.toml`, and `include_str!` puts them in the binary. The resolver reads them through a separate `Quirks` struct, which holds only `charset`, `login.template` and `keepalive`. `Quirks` and its nested table of `login` both have `deny_unknown_fields`, so `[login] name` fails too. Thus a built-in type that sets `columns` fails each start and each test. The resolver turns each `Quirks` into a `LayerFields`, so the value checks run on it as on any other layer. The test `built_in_types_hold_only_quirks` reads each file through `Quirks` and those checks.

Task 1.1 fills the values from sources. TinyMUX gets `charset = "iso-8859-1"`, from the decision page of the telnet engine. The login template and the keepalive of each type come from the documentation or the source of each server. If a source names a command that the server accepts with no effect, the type sets a keepalive. Otherwise it sets none.

### The layer seam and its trust

```rust
pub enum LayerKey { Global, Type(String), Game(String), Character(WorldId), Plugin(String) }
pub enum Trust { Owner, SharedPlugin }
pub struct Layer { pub key: LayerKey, pub origin: LayerOrigin, pub trust: Trust, pub extends: Option<String>, pub fields: LayerFields }
pub trait LayerSource { fn layers(&self) -> Result<Vec<Layer>, ConfigError>; }
```

`LayerOrigin` is the source of a layer or a field: a file, a built-in type, a plugin or the default. The crate root already exports `Source` and `Origin`, so the new type takes a name that neither has.

The resolver takes the layers of each source, builds the chain of each world, and checks trust before the merge. A layer with `Trust::SharedPlugin` that sets `host`, `port`, `tls`, `login.name`, `login.secret` or `login.template` fails with the plugin name and the field. The template decides where the password goes, so a plugin cannot set it. In this change, only tests build a plugin layer, and the resolver puts it between the game and the character. A Lua source later returns `Layer` values of its own.

### The resolved world

`ResolvedWorld` gains the connection fields `host`, `port` and `tls`. It gains `charset`, `columns`, `rows`, and `keepalive`, an `Option` of a command and a period. It also gains `rules`, `scripts` and `sources`, a map from a field path to its `LayerOrigin`. `Login.template` comes from the merge, and `Login.name` comes from `login.name`. The fake world keeps `with_default_login`, which fills the new fields from `WorldDefaults`.

### The check after the merge

After the merge, the resolver checks each world. Each error names the world id and the field.

- `host` and `port` are set, and `host` is not empty.
- `login.name` and `login.secret` are both set, or neither is. With neither, `login` is `None`, and a `login.template` from any layer does not count.
- `keepalive.command` and `keepalive.seconds` are both set, or neither is.

### The configuration directory

A new module `crates/mu-cli/src/env_path.rs` holds two helpers. One parses a value and reads an empty value as none. The other looks up an XDG base and accepts only an absolute path. `SocketArgs` moves to them, and `ConfigArgs` uses them. `mu-cli` gets `ConfigArgs`, with `--config` and the order `--config`, `MU_CONFIG`, `XDG_CONFIG_HOME/mu`, then `HOME/.config/mu`, on Linux and macOS alike. An empty or relative `XDG_CONFIG_HOME` is ignored, as the XDG Base Directory specification asks. `HOME` goes through the same check. If `HOME` is unset, empty or relative, there is no default directory, and the configuration holds no worlds. macOS has no other convention that a terminal user expects for a CLI tool. A missing directory is an empty configuration, so `mu server` with no configuration still starts. If `--config` or `MU_CONFIG` names the missing directory, a warning on stderr names it. The user chose that path, so a typo there must not give no worlds and no message.

### `mu world`

`mu world list` prints the ids in the order of `WorldId`, one on each line. `mu world show <id>` prints the resolved world as TOML. Each field ends with a comment `# <source>`, and each `[[rules]]` and `[[scripts]]` table has a comment line with its source above it. `show` resolves all worlds first, so a bad file fails `show` as it fails the server. Then it looks up the id. `show` takes the id as text, not through the parser of `WorldId`, so a malformed id also gives exit status 1. An id that is not a configured world is an error that names the id, and the exit status is 1. The verb reads files only, and needs no server and no secret.

`show` writes the world in a fixed order. First come `host`, `port`, `tls`, `charset`, `columns` and `rows`. Then come the table `[login]` with `template`, `name` and `secret`, and the table `[keepalive]` with `command` and `seconds`. A table that the world does not have is left out. The rules and the scripts come last. `show` writes each field as `<key> = <value> # <source>`. It writes the character set in lower case, such as `iso-8859-1`, as a file sets it. `ResolvedWorld.charset` is a `Charset`, so world-sessions (#54) reads no name again. The keys of these fields are fixed names, and the `Display` of `toml::Value` writes each value. For each entry of `rules` and `scripts`, `show` builds a document that holds only that entry, such as `{ rules = [entry] }`. `toml::to_string` writes the document under the comment line of the entry. Thus the `toml` crate writes the header `[[rules]]`, each nested table under it, and each key and each value. A row of such documents is valid TOML, because a nested table after `[[rules]]` belongs to the last entry. The project writes no quoting of its own. `mu-cli` takes `toml` as a normal dependency for this.

### Configured worlds in the server

`WorldSetup` gets a transport of `Option<Arc<dyn Transport>>`. A world with no transport starts `disconnected` and does not connect at the start. `Connect` on it fails with a new `PortalError::NoTransport(WorldId)`, which `convert.rs` maps to `FailedPrecondition`. `mu server` resolves the configuration before it opens the socket, so a bad file leaves no socket. Next it compares the ids of `--fake-world` with the configured ids. A shared id is an error that names the id, and the server stops before it opens the socket. Then it adds a `WorldSetup` with no transport and no password for each configured world. world-sessions (#54) gives these worlds a telnet transport.

### Tests do not read the configuration of the developer

The launchers use three types of command: the one of `std`, the one of `tokio`, and `CommandBuilder` of `portable_pty`. Thus the shared part is one list of changes to the environment, in the shape `(name, Option<value>)` that `ServerBuilder` and `mu_with` already take. The list removes `MU_SOCKET` and sets `MU_CONFIG` to an empty directory of the test. `ServerBuilder`, `PtyClient`, `mu_on_pty`, `mu_with` and the `Command` of `non_utf8_verb_is_usage_error` in `cli.rs` each apply it to their own command. A `mu client` that starts a server passes this environment to it. A test that needs a configuration writes one to its temporary directory and passes `--config` or `MU_CONFIG`.

### Using the spike

This change uses one fact of the spike of issue 7, so it follows "Using a spike" on `docs/src/content/docs/contributing/spikes.md`:

1. The decision page is `docs/src/content/docs/decisions/telnet-engine.md`, and the spike merged in pull request #32.
2. The one review finding of #32, the TTABLE prefix, belongs to the telnet engine and does not touch this change.
3. Of the items under Not tested, the cause of the TinyMUX stall after `ACCEPTED UTF-8` stays a non-goal. This change gives TinyMUX ISO-8859-1. The other items belong to the telnet engine or to world-sessions.
4. No spike code is copied. The built-in types are new files.
5. No new evidence is copied. The transcripts are already in `crates/mu-core/tests/fixtures/telnet/`.

### Enforcement of each rule

| Rule | Enforced by |
|---|---|
| A built-in type sets only quirks | The `Quirks` struct with `deny_unknown_fields`, the test `built_in_types_hold_only_quirks`, and the test `quirks_refuse_other_fields`. |
| A shared plugin cannot set the connection | The trust check in the resolver, the table test `plugin_cannot_set_the_connection` over the six fields, and `plugin_can_set_a_preference`. |
| No file holds a password | `deny_unknown_fields` and the `SecretRef` parse, and the tests `password_in_a_file` and `secret_that_is_not_a_reference`, which also check that stderr does not hold the value. |
| No error prints a secret value | `ConfigError` holds no `Display` of a `toml` error, no message under `login`, and no text of `BadSecretRef`. The tests `password_in_a_file`, `secret_that_is_not_a_reference` and `wrong_type_under_login` check stderr. |
| No line field holds CR or LF | The parse of each layer, and the test `line_break_in_a_line_field`. |
| Rules and scripts keep unique ids in the order of their first layer | The parse and the resolver, and the property test `accumulation_matches_a_model`. |
| `mu world show` prints TOML that parses | The `toml` crate writes each entry as a document. The property test `show_output_parses_back` generates keys and values from any string, and nested tables. |
| No resolution reads a secret | The resolver takes no `SecretStore`, and the tests `show_does_not_read_the_secret` and `server_does_not_read_a_configured_secret`. |
| A layer that does not set a field keeps the value of an earlier layer | `skip_serializing_if = "Option::is_none"` on each optional field of `LayerFields`, `Login` and `Keepalive`. If one field lacks it, `empty_layer_gives_figment_no_keys` or `each_nested_field_alone_gives_one_key` fails. The second is a table test over each field of `login` and `keepalive`. |
| A loop in `extends` fails | The tests `loop_of_types` and `type_that_extends_itself`. |
| `mu-core` takes only allowed crates | `ALLOWED_CRATES` in the boundary test, with `figment2`, `serde`, `serde_path_to_error` and `toml`, and `cargo deny check`. |
| A test never reads the configuration of the developer | The shared list of changes to the environment, and the test `helper_points_mu_config_at_an_empty_directory`. Review covers a test that builds a `Command` of its own. |
| Each fact about `figment2`, XDG and the servers holds | Task 1.1 reads the sources and fills the table of checked facts. |

## Risks / Trade-offs

- [`figment2` has one maintainer. Its last release is from 2026-04.] → `figment2` sits behind the resolver. A swap to `figment` or `schematic` touches one module. With no features, it adds only `figment2` and `uncased` to `Cargo.lock`.
- [The `sources` map uses field paths as text. A renamed field can leave a stale path.] → The test `each_field_has_a_source` checks that each field of a resolved world has one entry.
- [Before #54, a client shows configured worlds that cannot connect.] → `ListWorlds` shows it `disconnected`, and `Connect` names the world in its error. #54 is next in the plan.
- [The default directory on macOS is `~/.config/mu`, not `~/Library/Application Support`] → This matches the CLI tools a terminal user runs. `MU_CONFIG` moves it.

## Migration Plan

None. No user has a configuration, and the `--fake-world` flag keeps its behavior.

## Checked facts

Task 1.1 fills this table with each fact that the design uses, with its source.

| Fact | Source |
|---|---|
| `figment2` 0.11.5 has `Figment::merge`, `Figment::extract` and `Figment::find_metadata`, and `providers::Named::new` gives a provider a name. | The source of `figment2` 0.11.5: `src/figment.rs` lines 348, 605 and 940, and `src/providers/named.rs` line 39. The debate projects ran each of them. |
| With no features, `figment2` 0.11.5 depends on `serde` and `uncased`, on `atomic` for targets of 32 bits or less, and on `version_check` to build. `toml_edit` 0.25 comes only with its TOML feature. | `Cargo.toml` of `figment2` 0.11.5. Before the change, `Cargo.lock` held `serde`, `atomic` and `version_check`, but not `uncased`. The change added only `figment2`, `uncased` and `serde_path_to_error`. |
| A relative path in `XDG_CONFIG_HOME` is invalid and is ignored. If the variable is unset or empty, the base is `$HOME/.config`. | XDG Base Directory specification, version 0.8, sections 2 and 3: https://specifications.freedesktop.org/basedir-spec/latest/ |
| All three servers log in with `connect <name> <password>`, so each built-in type sets `login.template = "connect {login} {password}"`. | The `>` lines of `tinymux-latin1.txt`, `aresmush.txt` and `evennia.txt` in `crates/mu-core/tests/fixtures/telnet/`. |
| TinyMUX accepts the socket command `IDLE`, which does nothing and does not reset the idle time. Its help suggests to send it about once a minute. The type `tinymux` sets `keepalive = { command = "IDLE", seconds = 60 }`. | https://www.tinymux.org/wiki/flags/idle/ and https://www.tinymux.org/wiki/commands/socket-command/ |
| Evennia accepts `idle` with no effect except the idle timer, whatever `IDLE_COMMAND` is. The type `evennia` sets `keepalive = { command = "idle", seconds = 60 }`. | `evennia/server/inputfuncs.py`, function `text`, and `IDLE_COMMAND` in `settings_default.py`: https://www.evennia.com/docs/latest/Setup/Settings-Default.html |
| AresMUSH names no command that a client sends with no effect. Its keepalive runs in the server and the player turns it off with `keepalive off`. The type `aresmush` sets no keepalive. | https://aresmush.com/clients.html |
| TinyMUX takes no input after the client accepts UTF-8 through CHARSET, so it needs ISO-8859-1. | `docs/src/content/docs/decisions/telnet-engine.md`. |
