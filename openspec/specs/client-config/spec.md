# client-config Specification

## Purpose
Defines the fields of a client layer in `clients/` on the server: the keys and the echo line, and when a binding makes its layer bad.

## Requirements

### Requirement: The client reads client.toml
The client SHALL read `client.toml` at the top level of the configuration directory, which it finds by the rule of the requirement "The configuration directory". A missing file, or no directory, SHALL give the defaults and SHALL NOT be an error. The client SHALL read the file once, before it opens the screen.

#### Scenario: A file that sets the echo
- **GIVEN** `<dir>/client.toml` holds `echo = "show"`, and a server with the fake world `alpha/aria`
- **WHEN** the user starts `mu client --config <dir>`, types `hello` and presses Enter
- **THEN** the screen shows `> hello`, and its cells are not dim

#### Scenario: A file that sets dim
- **GIVEN** `<dir>/client.toml` holds `echo = "dim"`, and a server with the fake world `alpha/aria`
- **WHEN** the user starts `mu client --config <dir>`, types `hello` and presses Enter
- **THEN** the cells of `> hello` are dim

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

#### Scenario: A file that cannot be read
- **GIVEN** the test runs as a user that is not root, and `<dir>/client.toml` exists with mode 000
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml`
- **AND** the exit status is 1

#### Scenario: Not TOML
- **GIVEN** `<dir>/client.toml` holds `echo = `
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml` and line 1
- **AND** the exit status is 1

### Requirement: The keys table binds a key to an entry
The table `[keys]` SHALL map a key to the name of an entry of the registry, or to `"none"`. A key SHALL be modifiers from `ctrl`, `alt`, `shift` and `super`, then one key, joined with `+`. A key is one character that is not an uppercase letter, not a control character and not `+`, the name `plus`, or a name of a special key. Each binding SHALL replace the default binding of that key. A key bound to `"none"` SHALL do nothing. The keys that the table does not name SHALL keep their defaults.

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

#### Scenario: A binding replaces a default
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"ctrl+n" = "focus-prev"`, and a server with the fake worlds `alpha/aria`, `beta/bryn` and `gamma/gil`
- **WHEN** the user starts `mu client --config <dir>` and presses Ctrl+N
- **THEN** the top row shows `[gamma/gil]`

#### Scenario: None on a key with a modifier
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"alt+enter" = "none"`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `hi` and presses Alt+Enter
- **THEN** the server records no player event
- **AND** the editor holds `hi`

### Requirement: A bad binding stops the client
A binding of `[keys]` SHALL stop the client as a bad file in each of these cases: an unknown key, an unknown entry, an entry that needs an argument, a repeated modifier, `shift` with a character that is not a letter, a control character, and two names of one key. The terminal reports Shift and such a character as the shifted character, such as `!`.

#### Scenario: Two names of one key
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"ctrl+shift+z" = "redo"` and `"shift+ctrl+z" = "undo"`
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml`, `ctrl+shift+z` and `shift+ctrl+z`
- **AND** the exit status is 1

#### Scenario: A binding of Shift+Tab
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"shift+tab" = "focus-prev"`, and a server with the fake worlds `alpha/aria`, `beta/bryn` and `gamma/gil`
- **WHEN** the user starts `mu client --config <dir>` and presses Shift+Tab
- **THEN** the top row shows `[gamma/gil]`

#### Scenario: An uppercase key
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"ctrl+N" = "quit"`
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml` and `ctrl+N`
- **AND** the exit status is 1

#### Scenario: A control character
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"alt+\u007f" = "quit"`
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml`
- **AND** the exit status is 1

#### Scenario: Shift with a digit
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"shift+1" = "quit"`
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** stderr names `client.toml` and `shift+1`
- **AND** the exit status is 1

#### Scenario: A binding of Ctrl+Shift+Z
- **GIVEN** `<dir>/client.toml` holds `[keys]` with `"ctrl+shift+z" = "none"`
- **WHEN** the user types `h`, presses Ctrl+Z, and presses Ctrl+Shift+Z
- **THEN** the editor is empty

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

### Requirement: The server table sets the wait for a server
The field `connect_wait_seconds` of the table `[server]` SHALL be the longest time, in seconds, that `mu client` waits for a server to answer. It SHALL allow a fraction, and its default SHALL be 5. A value that is 0 or less, that is not finite, or that is less than 1 ns SHALL stop the client before the screen opens, with exit status 1 and an error on stderr that names `client.toml` and `server.connect_wait_seconds`. The requirement "The client starts a server" gives the waits that it bounds.

#### Scenario: A wait of zero
- **GIVEN** `<dir>/client.toml` holds `[server]` with `connect_wait_seconds = 0`
- **WHEN** the user starts `mu client --config <dir>`
- **THEN** the exit status is 1, the client does not enter the alternate screen, and stderr contains `client.toml` and `server.connect_wait_seconds`

### Requirement: The echo field sets the echo line
The field `echo` SHALL be one of `show`, `dim` and `hide`, and its default SHALL be `dim`. With `show`, the client SHALL show the echo line of a send with no dim attribute. With `dim`, it SHALL show it with the dim attribute. With `hide`, it SHALL NOT show it.

#### Scenario: The default dims
- **GIVEN** `<dir>` holds no `client.toml`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `hello` and presses Enter
- **THEN** the cells of `> hello` are dim

#### Scenario: Hide
- **GIVEN** `<dir>/client.toml` holds `echo = "hide"`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `hello` and presses Enter
- **THEN** the screen shows `You say, "hello"`
- **AND** the screen shows no row that starts with `> `
