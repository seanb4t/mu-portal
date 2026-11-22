## Purpose

Defines the configuration file of the terminal client, `client.toml`: where the client finds it, how it checks it, and the fields that change keys and the echo line.

## ADDED Requirements

### Requirement: The client reads client.toml
The client SHALL read `client.toml` at the top level of the configuration directory, which it finds by the rule of the requirement "The configuration directory". A missing file, or no directory, SHALL give the defaults and SHALL NOT be an error. The client SHALL read the file once, before it opens the screen.

#### Scenario: A file that sets the echo
- **GIVEN** `<dir>/client.toml` holds `echo = "show"`, and a server with the fake world `alpha/aria`
- **WHEN** the user starts `mu client --config <dir>`, types `hello` and presses Enter
- **THEN** the screen shows `> hello` in the default style

#### Scenario: No file
- **GIVEN** `<dir>` holds no `client.toml`, and a server with the fake world `alpha/aria`
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** the top row shows `[alpha/aria]`

### Requirement: A bad client.toml stops the client
A bad file is one that the client cannot read or parse, or that holds an unknown key or a value out of its range. For a bad `client.toml`, the client SHALL NOT open the screen. It SHALL exit with status 1, and stderr SHALL name the file and the key or the line.

#### Scenario: An unknown key
- **GIVEN** `<dir>/client.toml` holds `colour = "red"`
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml` and `colour`
- **AND** the exit status is 1
- **AND** the terminal is not in the alternate screen

#### Scenario: A bad echo value
- **GIVEN** `<dir>/client.toml` holds `echo = "loud"`
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml` and `echo`
- **AND** the exit status is 1

#### Scenario: Not TOML
- **GIVEN** `<dir>/client.toml` holds `echo = `
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml` and line 1
- **AND** the exit status is 1

### Requirement: The keys table binds a key to an entry
The table `[keys]` SHALL map a key to the name of an entry of the registry, or to `"none"`. A key SHALL be lowercase modifier names from `ctrl`, `alt`, `shift` and `super`, then one key name, joined with `+`. Each binding SHALL replace the default binding of that key, and `"none"` SHALL remove it. The keys that the table does not name SHALL keep their defaults. An unknown key name, an unknown entry, or an entry that needs an argument SHALL stop the client as a bad file.

#### Scenario: A new binding
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"alt+n" = "focus-next"`, and a server with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client --config <dir>` and presses Alt+N
- **THEN** the top row shows `[beta/bryn]`

#### Scenario: The default keys stay
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"alt+n" = "focus-next"`, and a server with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client --config <dir>` and presses Ctrl+N
- **THEN** the top row shows `[beta/bryn]`

#### Scenario: A removed binding
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"ctrl+n" = "none"`, and a server with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client --config <dir>` and presses Ctrl+N
- **THEN** the top row shows `[alpha/aria]`

#### Scenario: An unknown entry
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"alt+n" = "fly"`
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml`, `alt+n` and `fly`
- **AND** the exit status is 1

#### Scenario: An unknown key name
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"hyper+n" = "quit"`
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml` and `hyper+n`
- **AND** the exit status is 1

#### Scenario: An entry that needs an argument
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"alt+f" = "focus"`
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml`, `alt+f` and `focus`
- **AND** the exit status is 1

### Requirement: The echo field sets the echo line
The field `echo` SHALL be one of `show`, `dim` and `hide`, and its default SHALL be `dim`. With `show`, the client SHALL show the echo line of a send in the default style. With `dim`, it SHALL show it with the dim attribute. With `hide`, it SHALL NOT show it.

#### Scenario: The default dims
- **GIVEN** `<dir>` holds no `client.toml`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `hello` and presses Enter
- **THEN** the cells of `> hello` are dim

#### Scenario: Hide
- **GIVEN** `<dir>/client.toml` holds `echo = "hide"`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `hello` and presses Enter
- **THEN** the screen shows `You say, "hello"`
- **AND** the screen shows no row that starts with `> `
