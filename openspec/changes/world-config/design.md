# Design

## Context

See proposal.md, section Why. These are the facts of the code that shape the design:

- `ResolvedWorld` in `crates/mu-core/src/world.rs` holds only the id and an optional `Login`. `mu server` builds it in `crates/mu-cli/src/server.rs` from each `--fake-world` flag, with `DEFAULT_LOGIN_TEMPLATE` and the character part of the id as the login name.
- `WorldSetup` in `crates/mu-core/src/portal.rs` holds the world, the password, the transport, the framer, the hooks and the handlers. The portal connects every world at the start.
- `SecretRef` in `crates/mu-core/src/secret.rs` parses `env:<NAME>`, and `load_password` reads it. The parse is the only check that a text is a reference.
- `TelnetSettings` in `crates/mu-core/src/framer.rs` holds the character set (`Charset::Utf8` or `Charset::Latin1`), the window size and the option policy. Its default is UTF-8 and 120 by 40.
- The socket path rules in `crates/mu-cli/src/socket.rs` read `--socket`, then `MU_SOCKET`, then `XDG_RUNTIME_DIR`. The configuration directory follows the same shape.
- The process helper in `crates/mu-cli/tests/support/process.rs` clears `MU_SOCKET` for each `mu` that a test starts.
- The decision page `world-type-profiles.md` names `figment`. The page `telnet-engine.md` records the quirks that the spike of issue 7 found (pull request #32).

## Goals / Non-Goals

**Goals:**

- One resolver that turns a set of layers into one `ResolvedWorld` per world, with the source of each field.
- A layer seam that a Lua source can implement later with no change to the resolver.
- Errors that name the file and the field, and never a secret value.

**Non-Goals:**

- A schema version in the files. The files have no users yet.
- Kept comments or order in a file that the client edits. Nothing writes the files.
- Plugin layers from disk, and the place of a plugin layer in a real chain. A test gives the resolver a plugin layer at a fixed place.
- Environment variables that override single fields, such as `MU_COLUMNS`.

## Decisions

### Layers are parsed alone, then merged

The resolver reads each file into a typed `Layer` with `serde` and `deny_unknown_fields`, before any merge. A bad field fails in its own file, and so does a bad type or a value out of range. Thus the error names that file and that field. The parse checks `login.secret` with `SecretRef::from_str`, and the character set against the names of `Charset`. It also checks the ranges of `columns`, `rows`, `port` and `keepalive.seconds`.

Alternative: merge first and check the merged result. The error then names the field but not always the file that broke it, and a value that a later layer hides goes unchecked.

### The merge of fields uses `figment2`

The resolver turns each layer into one provider, named by its source, and merges them with `merge` in layer order. It reads the merged fields with `extract`, and the source of each field with `find_metadata`. Only the fields go to `figment2`. The arrays `rules`, `scripts` and `disable` stay out, because `merge` replaces an array and `admerge` cannot remove an entry by id.

`figment2` is a fork of `figment` with the same API. It carries the open pull requests of `figment` and reads TOML with `toml_edit` 0.25. The decision page names `figment`, so task 6.2 edits the page.

The research on 2026-11-17 compared these crates. The hard need is the source of each merged field, because `mu world show` prints it.

| Crate | Last release | Source of each field | Fit |
|---|---|---|---|
| `figment` | 2024-05, 0.10.19 | Yes, `find_metadata` | In 2026-04 the author promised more work, but no release followed. It depends on `toml` 0.8. |
| `figment2` | 2026-04, 0.11.5 | Yes, the same API | One maintainer. Foundry moved to it in 2026-01. It does only the merge, which the resolver lacks. |
| `config-rs` | 2026-09, 0.15.27 | Only in some errors | The most used crate, but a tree of values with no typed layers and no source for each merged value. |
| `schematic` | 2026-11, 0.21.1 | No. The loader keeps the layers, and the caller searches them. | Typed partial layers, merge functions and validation. Its `extends` names files or URLs, not types, and it brings `miette` and `garde`. |
| `confique` | 2025-10, 0.4.0 | No | Typed layers with `with_fallback`, one maintainer, and `toml` 0.9. |

A debate on 2026-11-17 between two agents, under a protocol fixed before it began, kept `figment2`. The scores were 4.00 for `figment2` and 3.40 for `schematic`. `schematic` fails the license policy of `deny.toml`, because it brings `option-ext` under MPL-2.0. The maintenance of `schematic` is better, by less than the two points that the protocol needs to decide alone.

The resolver gives `figment2` only typed layers that `toml` parsed, through `Serialized`. Thus the errors that a user sees come from `toml`, with the file, the field and the line. A `None` field in `Serialized` hides the value of an earlier layer. Thus each optional field skips `None` in serialization.

Upstream `figment` has the same API on its main branch since 2026-10, but no release. When it releases, a rename of the package in `Cargo.toml` moves the project back to it.

Alternatives: `figment`, with no release in two years and an old `toml`. `schematic`, which is the best second choice. It replaces the typed parse of each layer, but the resolver then writes the source search that `figment2` gives. A merge of `toml::Value` tables that the project writes, which breaks the rule to reuse open source first.

### Rules and scripts accumulate in the resolver

The resolver walks the layers in merge order. For each layer, it first removes each entry that `disable` names, then appends or replaces each entry of `rules` and `scripts` by id. A replace keeps the place of the old entry. An entry is an `id` and the rest of its TOML table, kept as a value. Each entry records the source of the layer that added it. A `disable` id with no inherited entry is an error, so a typo does not pass in silence.

### The chain of a world

The chain starts at the character and the game. Next comes the type that the game `extends`, and each type that a type `extends`, up to a type with no `extends`. Then come `global.toml` and the defaults. The resolver walks this chain with a set of the types it saw, so a loop fails with the list of the files in it. A user type shadows nothing: a user type with a built-in name is an error. Thus a name in `extends` has one meaning.

### Built-in types and the quirk schema

The built-in types live in `crates/mu-core/src/world/types/<name>.toml`, and `include_str!` puts them in the binary. The resolver reads them through a separate `Quirks` struct with `deny_unknown_fields`, which holds only `charset`, `login.template` and `keepalive`. Thus a built-in type that sets `columns` fails each start and each test. The test `built_in_types_hold_only_quirks` reads each file through `Quirks`.

Task 1.1 fills the values from sources. TinyMUX gets `charset = "iso-8859-1"`, from the decision page of the telnet engine. The login template and the keepalive of each type come from the documentation or the source of each server. If a source names a command that the server accepts with no effect, the type sets a keepalive. Otherwise it sets none.

### The layer seam and its trust

```rust
pub enum LayerKey { Global, Type(String), Game(String), Character(WorldId), Plugin(String) }
pub enum Trust { Owner, SharedPlugin }
pub struct Layer { pub key: LayerKey, pub source: Source, pub trust: Trust, pub fields: LayerFields }
pub trait LayerSource { fn layers(&self) -> Result<Vec<Layer>, ConfigError>; }
```

`TomlDirSource` reads the directory. The resolver takes the layers of each source, builds the chain of each world, and checks trust before the merge. A layer with `Trust::SharedPlugin` that sets `host`, `port`, `tls`, `login.name` or `login.secret` fails with the plugin name and the field. In this change, only tests build a plugin layer, and the resolver puts it between the game and the character. A Lua source later returns `Layer` values of its own.

### The resolved world

`ResolvedWorld` gains the connection fields `host`, `port` and `tls`. It gains `charset`, `columns`, `rows`, and `keepalive`, an `Option` of a command and a period. It also gains `rules`, `scripts` and `sources`, a map from a field path to its `Source`. `Login.template` comes from the merge, and `Login.name` comes from `login.name`. The fake world keeps `with_default_login`, which fills the new fields with the defaults.

### The configuration directory

`mu-cli` gets `ConfigArgs`, with `--config` and the order `--config`, `MU_CONFIG`, `XDG_CONFIG_HOME/mu`, then `HOME/.config/mu`, on Linux and macOS alike. An empty or relative `XDG_CONFIG_HOME` is ignored, as the XDG Base Directory specification asks. macOS has no other convention that a terminal user expects for a CLI tool. A missing directory is an empty configuration, so `mu server` with no configuration still starts.

### `mu world`

`mu world list` prints the ids in the order of `WorldId`, one on each line. `mu world show <id>` prints the resolved world as TOML. Each field ends with a comment `# <source>`, and each `[[rules]]` and `[[scripts]]` table has a comment line with its source above it. `show` resolves all worlds first, so a bad file fails `show` as it fails the server. The verb reads files only, and needs no server and no secret.

### Configured worlds in the server

`WorldSetup` gets a transport of `Option<Arc<dyn Transport>>`. A world with no transport starts `disconnected` and does not connect at the start. `Connect` on it fails with a new `PortalError::NoTransport(WorldId)`, which `convert.rs` maps to `FailedPrecondition`. `mu server` resolves the configuration before it opens the socket, so a bad file leaves no socket. Then it adds a `WorldSetup` with no transport and no password for each configured world. world-sessions (#54) gives these worlds a telnet transport.

### Tests do not read the configuration of the developer

The process helper sets `MU_CONFIG` to an empty directory of the test for each `mu` that it starts, as it clears `MU_SOCKET`. A test that needs a configuration writes one to its temporary directory and passes `--config` or `MU_CONFIG`.

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
| A built-in type sets only quirks | The `Quirks` struct with `deny_unknown_fields`, and the test `built_in_types_hold_only_quirks`. |
| A shared plugin cannot set the connection | The trust check in the resolver, and the tests `plugin_cannot_set_the_host` and `plugin_can_set_a_preference`. |
| No file holds a password | `deny_unknown_fields` and the `SecretRef` parse, and the tests `password_in_a_file` and `secret_that_is_not_a_reference`, which also check that stderr does not hold the value. |
| No resolution reads a secret | The resolver takes no `SecretStore`, and the tests `show_does_not_read_the_secret` and `server_does_not_read_a_configured_secret`. |
| A layer that does not set a field keeps the value of an earlier layer | `skip_serializing_if = "Option::is_none"` on each optional field of `LayerFields`. If one field lacks it, the test `empty_layer_gives_figment_no_keys` fails. |
| A loop in `extends` fails | The test `loop_of_types`. |
| `mu-core` takes only allowed crates | `ALLOWED_CRATES` in the boundary test, with `figment2` and `serde`, and `cargo deny check`. |
| A test never reads the configuration of the developer | The process helper, and the test `helper_points_mu_config_at_an_empty_directory`. Review covers a test that builds a `Command` of its own. |
| Each fact about `figment2`, XDG and the servers holds | Task 1.1 reads the sources and fills the table of checked facts. |

## Risks / Trade-offs

- [`figment2` has one maintainer. Its last release is from 2026-04.] → `figment2` sits behind the resolver. A swap to `figment` or `schematic` touches one module. It adds only `figment2` and `uncased` to `Cargo.lock`. Its TOML reader, `toml_edit`, is already there.
- [The `sources` map uses field paths as text. A renamed field can leave a stale path.] → The test `each_field_has_a_source` checks that each field of a resolved world has one entry.
- [Before #54, a client shows configured worlds that cannot connect.] → `ListWorlds` shows it `disconnected`, and `Connect` names the world in its error. #54 is next in the plan.
- [The default directory on macOS is `~/.config/mu`, not `~/Library/Application Support`] → This matches the CLI tools a terminal user runs. `MU_CONFIG` moves it.

## Migration Plan

None. No user has a configuration, and the `--fake-world` flag keeps its behavior.

## Checked facts

Task 1.1 fills this table with each fact that the design uses, with its source.

| Fact | Source |
|---|---|
| `figment2` `merge` uses the incoming value and recurses into dictionaries. `find_metadata` gives the metadata of the provider that set a key. A provider can carry a name that `Metadata` keeps. | To check: docs.rs, `figment2`. |
| A relative path in `XDG_CONFIG_HOME` is invalid and is ignored. | To check: XDG Base Directory specification. |
| The login command and a no-op command of TinyMUX, AresMUSH and Evennia. | To check: the documentation or the source of each server, and the `>` lines of the transcripts. |
| TinyMUX takes no input after the client accepts UTF-8 through CHARSET, so it needs ISO-8859-1. | `docs/src/content/docs/decisions/telnet-engine.md`. |
