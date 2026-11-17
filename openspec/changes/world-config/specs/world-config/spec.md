# Spec Delta

## Purpose

Defines the configuration of the worlds: where its files live, how its layers merge into one resolved world, what each layer can set, and how the owner sees the result.

## ADDED Requirements

### Requirement: The configuration directory
The flag `--config <dir>` of `mu server` and `mu world`, and the environment variable `MU_CONFIG`, SHALL name the configuration directory. The flag SHALL win. An empty `MU_CONFIG` SHALL count as unset. By default, the directory SHALL be `mu` under `XDG_CONFIG_HOME`. If that variable is not set, empty or relative, the directory SHALL be `.config/mu` under `HOME`. A missing directory SHALL hold no worlds and SHALL NOT be an error.

#### Scenario: The flag wins over the environment
- **GIVEN** `MU_CONFIG` is `<dir>/env` with the world `alpha/aria`, and `<dir>/flag` holds the world `beta/bryn`
- **WHEN** the user runs `mu world list --config <dir>/flag`
- **THEN** stdout is `beta/bryn` and a line end
- **AND** the exit status is 0

#### Scenario: The default directory
- **GIVEN** `XDG_CONFIG_HOME` is `<dir>`, `<dir>/mu` holds the world `alpha/aria`, and neither `MU_CONFIG` nor `--config` is given
- **WHEN** the user runs `mu world list`
- **THEN** stdout is `alpha/aria` and a line end

#### Scenario: The directory under the home directory
- **GIVEN** `XDG_CONFIG_HOME` is empty, `HOME` is `<dir>`, and `<dir>/.config/mu` holds the world `alpha/aria`
- **WHEN** the user runs `mu world list`
- **THEN** stdout is `alpha/aria` and a line end

#### Scenario: A missing directory
- **WHEN** the user runs `mu world list --config <dir>/none`
- **THEN** stdout is empty
- **AND** the exit status is 0

### Requirement: Each character file makes one world
The directory SHALL hold `global.toml`, `types/<name>.toml`, `games/<game>.toml` and `characters/<game>/<character>.toml`, and each file SHALL be optional. Each character file SHALL make the world `<game>/<character>`, and its game SHALL be `games/<game>.toml`. A file name that is not a part of a world id SHALL be an error that names the file. So SHALL a character with no game file, and any other file that ends in `.toml`.

#### Scenario: Worlds from character files
- **GIVEN** a directory with `games/alpha.toml`, `games/beta.toml`, `characters/alpha/aria.toml`, `characters/alpha/ann.toml` and `characters/beta/bryn.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stdout holds the lines `alpha/ann`, `alpha/aria` and `beta/bryn`, in that order

#### Scenario: A character with no game
- **GIVEN** a directory with `characters/gamma/gil.toml` and no `games/gamma.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `characters/gamma/gil.toml` and `games/gamma.toml`
- **AND** the exit status is 1

#### Scenario: A file name outside the id rule
- **GIVEN** a directory with `games/Alpha.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/Alpha.toml`
- **AND** the exit status is 1

#### Scenario: A stray file
- **GIVEN** a directory with `worlds.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `worlds.toml`
- **AND** the exit status is 1

### Requirement: Layers merge from global to character
The layers of a world SHALL merge in this order: the defaults, `global.toml`, the type chain from the most general type to the most specific, the game and the character. A field SHALL take its value from the last layer that sets it. A table SHALL merge field by field.

#### Scenario: The last layer wins
- **GIVEN** `global.toml` sets `columns = 100`, the game `alpha` sets `columns = 90`, and `characters/alpha/aria.toml` sets `columns = 80`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows `columns = 80`

#### Scenario: A global default reaches each world
- **GIVEN** `global.toml` sets `rows = 50`, and no other layer sets `rows`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows `rows = 50`

#### Scenario: A table merges field by field
- **GIVEN** the game `alpha` sets `login.template`, and `characters/alpha/aria.toml` sets only `login.name` and `login.secret`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows the template of the game, and the name and the secret of the character

### Requirement: A layer names its parent with extends
A game file SHALL name its type with `extends`. A user type file can name another type with `extends`. The value SHALL be the name of a built-in type or of a user type. A type with no `extends` SHALL come right after `global.toml`. `global.toml` and a character file SHALL NOT hold `extends`. An unknown name, a game with no `extends` and a loop SHALL each be an error that names the files.

#### Scenario: A user type extends a built-in type
- **GIVEN** `types/rp.toml` sets `extends = "tinymux"` and `columns = 78`, and the game `alpha` sets `extends = "rp"`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows the character set of `tinymux` and `columns = 78`

#### Scenario: An unknown type
- **GIVEN** the game `alpha` sets `extends = "nosuch"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `nosuch`
- **AND** the exit status is 1

#### Scenario: A game with no type
- **GIVEN** `games/alpha.toml` has no `extends`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `extends`
- **AND** the exit status is 1

#### Scenario: A loop of types
- **GIVEN** `types/a.toml` sets `extends = "b"`, and `types/b.toml` sets `extends = "a"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `types/a.toml` and `types/b.toml`
- **AND** the exit status is 1

#### Scenario: A character that names a parent
- **GIVEN** `characters/alpha/aria.toml` sets `extends = "beta"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `characters/alpha/aria.toml` and `extends`
- **AND** the exit status is 1

### Requirement: Rules and scripts accumulate
The arrays `rules` and `scripts` SHALL accumulate across the layers in merge order. Each entry SHALL have an `id`. The array `disable` of a layer SHALL remove each inherited entry with an id that it names. An entry with the id of an entry that is still inherited SHALL replace that entry in its place. A `disable` id that names no inherited entry SHALL be an error.

#### Scenario: Entries from each layer
- **GIVEN** `global.toml` has the rule `hl-page`, the game `alpha` has the rule `hl-ooc`, and the character has the rule `hl-aria`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout lists the rules `hl-page`, `hl-ooc` and `hl-aria`, in that order

#### Scenario: A layer disables an inherited rule
- **GIVEN** `global.toml` has the rules `hl-page` and `hl-ooc`, and the character sets `disable = ["hl-ooc"]`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout lists only the rule `hl-page`

#### Scenario: A layer replaces an inherited rule
- **GIVEN** `global.toml` has the rules `hl-page` and `hl-ooc`, and the game has a rule `hl-page` with another pattern
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout lists `hl-page` with the pattern of the game, then `hl-ooc`

#### Scenario: A disable with no target
- **GIVEN** the character sets `disable = ["nosuch"]`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `characters/alpha/aria.toml` and `nosuch`
- **AND** the exit status is 1

### Requirement: Built-in types set only quirks
The binary SHALL hold the built-in types `tinymux`, `aresmush` and `evennia`. A built-in type SHALL set only quirk fields: `charset`, `login.template` and `keepalive`. The built-in type `tinymux` SHALL set `charset = "iso-8859-1"`. A user type SHALL NOT have the name of a built-in type.

#### Scenario: Each built-in type reads as quirks
- **WHEN** a test reads each built-in type with the schema of the quirk fields, which refuses every other field
- **THEN** each read succeeds

#### Scenario: A global preference is not hidden by a type
- **GIVEN** `global.toml` sets `columns = 100`, and the game `alpha` extends `tinymux`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows `columns = 100`

#### Scenario: A user type with a built-in name
- **GIVEN** a directory with `types/tinymux.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `types/tinymux.toml` and `tinymux`
- **AND** the exit status is 1

### Requirement: Each field is known
Each layer SHALL set only the fields of a world. The connection fields are `host`, `port` and `tls`. The text fields are `charset`, `columns` and `rows`. The tables are `login` (`template`, `name`, `secret`) and `keepalive` (`command`, `seconds`). The arrays are `rules`, `scripts` and `disable`. A layer that allows it also sets `extends`. An unknown field, or a value of the wrong type or out of range, SHALL be an error that names the file and the field.

#### Scenario: An unknown field
- **GIVEN** the game `alpha` sets `hots = "mux.example.org"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `hots`
- **AND** the exit status is 1

#### Scenario: A port out of range
- **GIVEN** the game `alpha` sets `port = 70000`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `port`
- **AND** the exit status is 1

#### Scenario: A character set that the engine cannot read
- **GIVEN** the game `alpha` sets `charset = "koi8-r"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `charset`
- **AND** the exit status is 1

### Requirement: A password is a secret reference
The field `login.secret` SHALL be a secret reference of the form `env:<NAME>`. No field SHALL hold a password. Neither the resolution of the configuration nor `mu world` SHALL read the secret.

#### Scenario: A password in a file
- **GIVEN** the character sets `login.password = "hunter2"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `characters/alpha/aria.toml` and `login.password`
- **AND** stderr does not contain `hunter2`
- **AND** the exit status is 1

#### Scenario: A secret that is not a reference
- **GIVEN** the character sets `login.secret = "hunter2"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `characters/alpha/aria.toml` and `login.secret`
- **AND** stderr does not contain `hunter2`
- **AND** the exit status is 1

#### Scenario: Show does not read the secret
- **GIVEN** the character sets `login.secret = "env:ARIA_PW"`, and `ARIA_PW` is not set
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows `env:ARIA_PW`
- **AND** the exit status is 0

### Requirement: A shared plugin layer cannot set the connection
A layer from a shared plugin source SHALL NOT set `host`, `port`, `tls`, `login.name` or `login.secret`. Such a layer SHALL be an error that names the source and the field.

#### Scenario: A plugin sets the host
- **GIVEN** a layer from a shared plugin source named `colors` that sets `host`
- **WHEN** the configuration resolves the world `alpha/aria` with that layer
- **THEN** the resolution fails with an error that names `colors` and `host`

#### Scenario: A plugin sets a preference
- **GIVEN** a layer from a shared plugin source named `colors` that sets `columns = 72`
- **WHEN** the configuration resolves the world `alpha/aria` with that layer after the game
- **THEN** the resolved world has `columns = 72`

### Requirement: A world has a place to connect
After the merge, each world SHALL have `host` and `port`. A world with `login.name` or `login.secret` SHALL have both. A world with neither SHALL log in by hand. Without `login.name` and `login.secret`, a `login.template` SHALL NOT count. A missing field SHALL be an error that names the world and the field.

#### Scenario: A world with no port
- **GIVEN** no layer of `alpha/aria` sets `port`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `alpha/aria` and `port`
- **AND** the exit status is 1

#### Scenario: A login name with no secret
- **GIVEN** the character sets `login.name = "Aria"` and no layer sets `login.secret`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `alpha/aria` and `login.secret`
- **AND** the exit status is 1

#### Scenario: A world that logs in by hand
- **GIVEN** no layer of `alpha/aria` sets `login.name` or `login.secret`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows no login
- **AND** the exit status is 0

### Requirement: Show names the source of each value
`mu world show <id>` SHALL print the resolved world as TOML. Each field SHALL carry a comment that names its source: the path of a file relative to the configuration directory, `built-in <type>`, or `default`. Each rule and each script SHALL carry the source of its layer. An id that is not a configured world SHALL be an error that names the id.

#### Scenario: Sources of fields
- **GIVEN** the game `alpha` extends `tinymux` and sets `host`, and no layer sets `rows`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout parses as TOML
- **AND** the line of `host` names `games/alpha.toml`
- **AND** the line of `charset` names `built-in tinymux`
- **AND** the line of `rows` names `default`

#### Scenario: An unknown world
- **WHEN** the user runs `mu world show alpha/nobody --config <dir>`
- **THEN** stderr names `alpha/nobody`
- **AND** the exit status is 1

### Requirement: The server holds the configured worlds
At the start, `mu server` SHALL resolve the configuration directory. A bad configuration SHALL stop the start with an error that names the file, and the exit status SHALL be 1. A world id that is both a configured world and a `--fake-world` SHALL be an error that names the id. The server SHALL NOT read the secret of a configured world at the start.

#### Scenario: A bad configuration stops the start
- **GIVEN** the game `alpha` sets `port = 70000`
- **WHEN** the user runs `mu server --socket <dir>/s.sock --config <dir>/conf`
- **THEN** stderr names `games/alpha.toml`
- **AND** no file `<dir>/s.sock` exists
- **AND** the exit status is 1

#### Scenario: A fake world with a configured id
- **GIVEN** the configuration holds the world `alpha/aria`
- **WHEN** the user runs `mu server --socket <dir>/s.sock --config <dir>/conf --fake-world alpha/aria=env:PW`
- **THEN** stderr names `alpha/aria`
- **AND** the exit status is 1

#### Scenario: The secret is not read at the start
- **GIVEN** the configuration holds the world `alpha/aria` with `login.secret = "env:ARIA_PW"`, and `ARIA_PW` is not set
- **WHEN** the user runs `mu server --socket <dir>/s.sock --config <dir>/conf`
- **THEN** an API call to list the worlds on `<dir>/s.sock` succeeds
