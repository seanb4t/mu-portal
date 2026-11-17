# Proposal

## Why

Today a world exists only through the hidden `--fake-world` flag of `mu server`. Its id gives the login name, and the code fixes the login template. The servers that Phase 1 targets differ in small ways. TinyMUX needs ISO-8859-1, and each family has its own login and keepalive. world-sessions (#54) cannot connect a real game until each world has a configuration that holds those differences. This change builds that configuration. Issue #53 tracks it, and it is step 3 of the Phase 1 plan in #61.

## What Changes

- World configuration in TOML files under one configuration directory. The flag `--config <dir>` and the variable `MU_CONFIG` name the directory. By default, the directory is `mu` under `XDG_CONFIG_HOME`. If that variable is not set, the directory is `~/.config/mu`. The layout is `global.toml`, `types/<name>.toml`, `games/<game>.toml` and `characters/<game>/<character>.toml`. Each character file makes one world, `<game>/<character>`.
- Five layers: global, built-in type, user type, game and character. A game names its type with `extends`, and a user type can name another type. A character belongs to the game of its directory. A loop in `extends` is an error that names each file in the loop.
- The merge: a field takes the value of the last layer that sets it. Rules and scripts accumulate in layer order, and `disable = ["<id>"]` removes an inherited entry by its id. Nothing runs rules or scripts in this change. They are data that #59 and #60 will use.
- `ResolvedWorld` grows to hold the merged world: the host, the port, TLS, the login, the character set, the window size, the keepalive, the rules and the scripts. It records the source of each field: a file, a built-in type or the default.
- Built-in types `tinymux`, `aresmush` and `evennia` ship inside the binary. A built-in type can set only quirk fields: the character set, the login template and the keepalive. A test reads each built-in type through the quirk schema.
- A layer-source seam with a TOML implementation, so that a Lua source can come later. A source has a trust level. A layer of a shared plugin cannot set the host, the port, TLS, the login name or the secret.
- A password is a secret reference `env:<NAME>`, never a value in a file.
- A new verb, `mu world`, with `mu world list` and `mu world show <id>`. `show` prints the merged world and the source of each field. It reads the files directly and needs no server.
- `mu server` reads the configuration directory at start. It refuses to start on a bad configuration and names the file. `ListWorlds` lists each configured world in the state `disconnected`. A call to `Connect` on a configured world fails, because no transport for a real game exists yet. The `--fake-world` flag stays, and an id that is both fake and configured is an error.
- The configuration library is `figment2`, the maintained fork of `figment`. The world-type-profiles decision page names `figment`, so this change updates that page.

## Not in this change

- Connections to real games: TCP, the login, the keepalive timer and reconnect. world-sessions (#54) uses the resolved world for these.
- TLS. The field `tls` is read and resolved, but #58 builds the connection.
- Lua layer sources and plugin files on disk. The seam and its trust level exist, and only tests use a plugin source.
- Rule and script engines, and any meaning of the fields of a rule or a script beyond its `id`.
- A reload of the configuration while the server runs. The server reads it once at start.
- Edits of the configuration through the API or the client.
- Secret stores other than the environment.
- Built-in types for PennMUSH and RhostMUSH. Phase 1 targets TinyMUX, AresMUSH and Evennia.

## Capabilities

### New Capabilities

- `world-config`: the configuration of the worlds. It covers where the files live, how the layers merge into one resolved world, what each layer can set, and how `mu world` shows the result.

### Modified Capabilities

- `cli-entrypoint`: the verbs are `client`, `server`, `version` and `world`.
- `world-sessions`: the server also holds the worlds of the configuration. They start `disconnected` and cannot connect yet. Only the fake worlds connect at the start.

## Impact

- Code: a new module `world` in `mu-core` for the layers, the merge and the built-in types, with the type files in `crates/mu-core/src/world/types/`. `ResolvedWorld` and `Login` grow. `mu-cli` gains the `world` verb and the `--config` flag. `mu-server` gains the configured worlds that have no transport.
- API: no change to `portal.proto`. `ListWorlds` returns more worlds, and `Connect` returns `failed-precondition` for a configured world.
- Dependencies: `figment2` with its TOML feature, through `[workspace.dependencies]`. `ALLOWED_CRATES` of `mu-core` in the boundary test gains it.
- Tests: each test that starts `mu server` points `MU_CONFIG` at an empty directory, so that the configuration of the developer never leaks in.
- Docs: the world-type-profiles decision page names `figment2`, and a new page in the user docs describes the files.
