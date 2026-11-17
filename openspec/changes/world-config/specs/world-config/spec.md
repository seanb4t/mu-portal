# Spec Delta

## Purpose

Defines the configuration of the worlds: where its files live, how its layers merge into one resolved world, what each layer can set, and how the owner sees the result.

Each scenario starts from a base directory `<dir>`, and names only what it changes. The base holds two files. `games/alpha.toml` sets `extends = "tinymux"`, `host = "mux.example.org"` and `port = 4201`. `characters/alpha/aria.toml` is empty. A scenario that removes a field or a file says so.

## ADDED Requirements

### Requirement: The configuration directory
The flag `--config <dir>` of `mu server` and `mu world` SHALL name the configuration directory, then `MU_CONFIG`. An empty `MU_CONFIG` SHALL count as unset. Next comes `mu` under `XDG_CONFIG_HOME`, then `.config/mu` under `HOME`. Each of these two SHALL count only with an absolute path. With none, there SHALL be no directory. A missing directory SHALL hold no worlds and SHALL NOT be an error. If `--config` or `MU_CONFIG` names a missing directory, stderr SHALL name it.

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

#### Scenario: An empty MU_CONFIG
- **GIVEN** `MU_CONFIG` is empty, `XDG_CONFIG_HOME` is `<dir>`, and `<dir>/mu` holds the world `alpha/aria`
- **WHEN** the user runs `mu world list`
- **THEN** stdout is `alpha/aria` and a line end

#### Scenario: A relative XDG_CONFIG_HOME
- **GIVEN** the current directory is `<dir>`, `XDG_CONFIG_HOME` is `xdg`, and `<dir>/xdg/mu` holds the world `beta/bryn`
- **AND** `HOME` is `<dir>/home`, and `<dir>/home/.config/mu` holds the world `alpha/aria`
- **WHEN** the user runs `mu world list`
- **THEN** stdout is `alpha/aria` and a line end

#### Scenario: No home directory
- **GIVEN** `MU_CONFIG` and `XDG_CONFIG_HOME` are not set, `HOME` is empty, and the current directory `<dir>` holds `.config/mu` with the world `alpha/aria`
- **WHEN** the user runs `mu world list`
- **THEN** stdout is empty
- **AND** the exit status is 0

#### Scenario: A missing directory
- **WHEN** the user runs `mu world list --config <dir>/none`
- **THEN** stdout is empty
- **AND** stderr names `<dir>/none`
- **AND** the exit status is 0

#### Scenario: A missing default directory
- **GIVEN** `XDG_CONFIG_HOME` is `<dir>`, `<dir>/mu` does not exist, and neither `MU_CONFIG` nor `--config` is given
- **WHEN** the user runs `mu world list`
- **THEN** stdout and stderr are empty
- **AND** the exit status is 0

### Requirement: Each character file makes one world
The directory SHALL hold `global.toml`, `types/<name>.toml`, `games/<game>.toml` and `characters/<game>/<character>.toml`, and each file SHALL be optional. Each character file SHALL make the world `<game>/<character>`, and its game SHALL be `games/<game>.toml`. A file name that is not a part of a world id SHALL be an error that names the file. So SHALL a character with no game file, and any other file that ends in `.toml`. So SHALL an entry that cannot be read, such as a link to nothing.

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
- **GIVEN** a directory with `games/Beta.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/Beta.toml`
- **AND** the exit status is 1

#### Scenario: A stray file
- **GIVEN** a directory with one of `worlds.toml`, `characters/alpha.toml`, `games/old/alpha.toml` and `characters/alpha/aria/x.toml`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names that path
- **AND** the exit status is 1

#### Scenario: A link to nothing
- **GIVEN** `games/beta` is a symbolic link to a path that does not exist
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/beta`
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
- **GIVEN** the game `alpha` sets `login.template = "co {login} {password}"`, and `characters/alpha/aria.toml` sets only `login.name = "Aria"` and `login.secret = "env:ARIA_PW"`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout has the lines `template = "co {login} {password}"`, `name = "Aria"` and `secret = "env:ARIA_PW"`, under the table `login`

### Requirement: A layer names its parent with extends
A game file SHALL name its type with `extends`. A user type file can name another type with `extends`. The value SHALL be the name of a built-in type or of a user type. A type with no `extends` SHALL come right after `global.toml`. `global.toml` and a character file SHALL NOT hold `extends`. An unknown name, a game with no `extends` and a loop SHALL each be an error that names the files.

#### Scenario: A user type extends a built-in type
- **GIVEN** `types/rp.toml` sets `extends = "tinymux"` and `columns = 78`, and the game `alpha` sets `extends = "rp"`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows `charset = "iso-8859-1"` and `columns = 78`

#### Scenario: An unknown type
- **GIVEN** the game `alpha` sets `extends = "nosuch"`
- **OR** `types/rp.toml` sets `extends = "nosuch"`, and no game extends `rp`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names the file that sets it and `nosuch`
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

#### Scenario: A type that extends itself
- **GIVEN** `types/a.toml` sets `extends = "a"`, and the game `alpha` sets `extends = "a"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `types/a.toml`
- **AND** the exit status is 1

#### Scenario: A character that names a parent
- **GIVEN** `characters/alpha/aria.toml` sets `extends = "beta"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `characters/alpha/aria.toml` and `extends`
- **AND** the exit status is 1

### Requirement: Rules and scripts accumulate
The arrays `rules` and `scripts` SHALL accumulate across the layers in merge order. Each entry SHALL have an `id` that is a string. The array `disable` of a layer SHALL remove each inherited rule or script with an id that it names. An entry with the id of an entry that is still inherited SHALL replace that entry in its place. A `disable` id that names no inherited entry SHALL be an error.

#### Scenario: Entries from each layer
- **GIVEN** `global.toml` has the rule `hl-page`, the game `alpha` has the rule `hl-ooc`, and the character has the rule `hl-aria`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout lists the rules `hl-page`, `hl-ooc` and `hl-aria`, in that order

#### Scenario: A layer disables an inherited rule
- **GIVEN** `global.toml` has the rules `hl-page` and `hl-ooc`, and the character sets `disable = ["hl-ooc"]`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout lists only the rule `hl-page`

#### Scenario: A layer replaces an inherited rule
- **GIVEN** `global.toml` has the rule `hl-page` with `pattern = "pages"` and the rule `hl-ooc`, and the game has the rule `hl-page` with `pattern = "pages you"`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout lists `hl-page`, then `hl-ooc`
- **AND** the table of `hl-page` has the line `pattern = "pages you"`, and stdout does not have the line `pattern = "pages"`

#### Scenario: An entry with no id
- **GIVEN** the game `alpha` has a rule with no `id`, or a rule with `id = 1`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `id`
- **AND** the exit status is 1

#### Scenario: A disable with no target
- **GIVEN** the character sets `disable = ["nosuch"]`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `characters/alpha/aria.toml` and `nosuch`
- **AND** the exit status is 1

### Requirement: Rules and scripts have unique ids
Rules and scripts SHALL share one space of ids in a world. An id twice in one array of one layer SHALL be an error that names the file and the id. So SHALL a rule and a script with the same id.

#### Scenario: An id twice in one layer
- **GIVEN** the game `alpha` has two rules with the id `hl-page`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `hl-page`
- **AND** the exit status is 1

#### Scenario: A rule and a script with one id
- **GIVEN** `global.toml` has the rule `greet`, and the game `alpha` has the script `greet`
- **OR** the game `alpha` has both the rule `greet` and the script `greet`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `greet`
- **AND** the exit status is 1

### Requirement: Built-in types set only quirks
The binary SHALL hold the built-in types `tinymux`, `aresmush` and `evennia`. A built-in type SHALL set only quirk fields: `charset`, `login.template` and `keepalive`. The built-in type `tinymux` SHALL set `charset = "iso-8859-1"`. A name of a character set SHALL match with no regard to case. A user type SHALL NOT have the name of a built-in type.

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
Each layer SHALL set only the fields of a world. The connection fields are `host`, `port` and `tls`. The display fields are `charset`, `columns` and `rows`. The tables are `login` (`template`, `name`, `secret`) and `keepalive` (`command`, `seconds`). The arrays are `rules`, `scripts` and `disable`. A layer that allows it also sets `extends`. An unknown field, or a value of the wrong type or out of range, SHALL be an error that names the file and the field.

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

#### Scenario: A template with no password
- **GIVEN** the game `alpha` sets `login.template = "connect {login}"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `login.template`
- **AND** the exit status is 1

#### Scenario: The case of a character set
- **GIVEN** the games `alpha`, `beta` and `gamma` set `charset` to `ISO-8859-1`, `iso-8859-1` and `utf-8`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** the exit status is 0

#### Scenario: A character set that the engine cannot read
- **GIVEN** the game `alpha` sets `charset = "koi8-r"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and `charset`
- **AND** the exit status is 1

### Requirement: Each value is in its range
The range of `port`, `columns` and `rows` SHALL be 1 to 65535. The range of `keepalive.seconds` SHALL be 1 to 86400. `login.name`, `login.template` and `keepalive.command` SHALL hold no CR and no LF. `login.template` SHALL hold `{password}`. A value outside its range SHALL be an error that names the file and the field.

#### Scenario: The edges of each range
- **GIVEN** a field of `port`, `columns`, `rows` and `keepalive.seconds`, and for each field a value at its edges: 0, 1 and the top of its range, and one more than the top
- **WHEN** the user runs `mu world list --config <dir>` with the game `alpha` that sets that field to that value
- **THEN** the exit status is 0 for 1 and the top of the range
- **AND** for 0 and one more than the top, stderr names `games/alpha.toml` and the field, and the exit status is 1

#### Scenario: A line break in a line field
- **GIVEN** the game `alpha` sets one of `login.name`, `login.template` and `keepalive.command` to a value with a CR or an LF
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml` and the field
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

#### Scenario: A value of the wrong type under login
- **GIVEN** the character sets `login.secret = 5`, or sets `login` to the string `"hunter2"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `characters/alpha/aria.toml` and the field, `login.secret` or `login`
- **AND** stderr does not contain the value
- **AND** the exit status is 1

#### Scenario: Show does not read the secret
- **GIVEN** the character sets `login.name = "Aria"` and `login.secret = "env:ARIA_PW"`, and `ARIA_PW` is not set
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout shows `env:ARIA_PW`
- **AND** the exit status is 0

### Requirement: A shared plugin layer cannot set the connection
A layer from a shared plugin source SHALL NOT set `host`, `port`, `tls`, `login.name`, `login.secret` or `login.template`. Such a layer SHALL be an error that names the source and the field.

#### Scenario: A plugin sets a field of the connection
- **GIVEN** a layer from a shared plugin source named `colors` that sets one field of this list: `host = "evil.example.org"`, `port = 4201`, `tls = true`, `login.name = "Eve"`, `login.secret = "env:EVE_PW"` or `login.template = "page eve={password}"`
- **WHEN** the configuration resolves the world `alpha/aria` with that layer
- **THEN** the resolution fails with an error that names `colors` and the field

#### Scenario: A plugin sets a preference
- **GIVEN** a layer from a shared plugin source named `colors` that sets `columns = 72`
- **WHEN** the configuration resolves the world `alpha/aria` with that layer after the game
- **THEN** the resolved world has `columns = 72`

### Requirement: A world has a place to connect
After the merge, each world SHALL have `host` and `port`, and `host` SHALL NOT be empty. A world with `login.name` or `login.secret` SHALL have both. A world with neither SHALL log in by hand. Without `login.name` and `login.secret`, a `login.template` SHALL NOT count. A world with `keepalive.command` or `keepalive.seconds` SHALL have both. A missing field SHALL be an error that names the world and the field.

#### Scenario: A world with no host or port
- **GIVEN** no layer of `alpha/aria` sets `host`, or the last layer that sets it sets `host = ""`, or no layer sets `port`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `alpha/aria` and the missing field
- **AND** the exit status is 1

#### Scenario: A login name with no secret
- **GIVEN** the character sets `login.name = "Aria"` and no layer sets `login.secret`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `alpha/aria` and `login.secret`
- **AND** the exit status is 1

#### Scenario: A login secret with no name
- **GIVEN** the character sets `login.secret = "env:ARIA_PW"` and no layer sets `login.name`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `alpha/aria` and `login.name`
- **AND** the exit status is 1

#### Scenario: Half a keepalive
- **GIVEN** the game `alpha` sets `extends = "aresmush"`, which sets no keepalive
- **AND** the game sets only `keepalive.command = "idle"`, or only `keepalive.seconds = 60`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `alpha/aria` and the half that is missing, `keepalive.seconds` or `keepalive.command`
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
- **WHEN** the user runs `mu world show alpha/nobody --config <dir>`, or `mu world show Alpha/x --config <dir>`
- **THEN** stderr names the id
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
- **GIVEN** the character sets `login.name = "Aria"` and `login.secret = "env:ARIA_PW"`, and `ARIA_PW` is not set
- **WHEN** the user runs `mu server --socket <dir>/s.sock --config <dir>/conf`
- **THEN** an API call to list the worlds on `<dir>/s.sock` succeeds
