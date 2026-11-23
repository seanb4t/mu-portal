# Spec Delta

## ADDED Requirements

### Requirement: The fields of a client layer
A client layer SHALL hold only the field `echo` and the table `[keys]`. Each SHALL be optional. An unknown field or table SHALL make the layer bad. So SHALL a value of the wrong type. The notice SHALL name the file and the field.

#### Scenario: A server table
- **GIVEN** `clients/global.toml` holds `[server]` with `connect_wait_seconds = 2`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `server`

#### Scenario: An unknown field
- **GIVEN** `clients/global.toml` holds `colour = "red"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `colour`

#### Scenario: Not TOML
- **GIVEN** `clients/global.toml` holds `echo = `, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and line 1

#### Scenario: A file that cannot be read
- **GIVEN** the test runs as a user that is not root, `<dir>/clients/global.toml` exists with mode 000, and a server serves `<dir>` on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml`
- **AND** the exit status is 0

### Requirement: A bad binding makes its layer bad
A binding of `[keys]` SHALL make its layer bad in each of these cases:

- an unknown key or an unknown entry
- an entry that needs an argument
- a repeated modifier
- `shift` with a character that is not a letter
- a control character
- two names of one key in the same layer

The terminal reports Shift and such a character as the shifted character, such as `!`. The notice SHALL name the file and the binding.

#### Scenario: Two names of one key
- **GIVEN** `clients/global.toml` holds `[keys]` with `"ctrl+shift+z" = "redo"` and `"shift+ctrl+z" = "undo"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr contains `clients/global.toml`, `ctrl+shift+z` and `shift+ctrl+z`

#### Scenario: An uppercase key
- **GIVEN** `clients/global.toml` holds `[keys]` with `"ctrl+N" = "quit"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr contains `clients/global.toml` and `ctrl+N`

#### Scenario: A control character
- **GIVEN** `clients/global.toml` holds `[keys]` with `"alt+\u007f" = "quit"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml`

#### Scenario: Shift with a digit
- **GIVEN** `clients/global.toml` holds `[keys]` with `"shift+1" = "quit"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr contains `clients/global.toml` and `shift+1`

#### Scenario: An unknown entry
- **GIVEN** `clients/global.toml` holds `[keys]` with `"alt+n" = "fly"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr contains `clients/global.toml`, `alt+n` and `fly`

#### Scenario: An unknown key name
- **GIVEN** `clients/global.toml` holds `[keys]` with `"hyper+n" = "quit"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr contains `clients/global.toml` and `hyper+n`

#### Scenario: An entry that needs an argument
- **GIVEN** `clients/global.toml` holds `[keys]` with `"alt+f" = "focus"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr contains `clients/global.toml`, `alt+f` and `focus`

#### Scenario: A binding of Shift+Tab
- **GIVEN** `clients/global.toml` holds `[keys]` with `"shift+tab" = "focus-prev"`, and a server serves that directory with the fake worlds `alpha/aria`, `beta/bryn` and `gamma/gil`
- **WHEN** the user starts `mu client` and presses Shift+Tab
- **THEN** the top row shows `[gamma/gil]`

#### Scenario: A binding of Ctrl+Shift+Z
- **GIVEN** `clients/global.toml` holds `[keys]` with `"ctrl+shift+z" = "none"`, and a server serves that directory with the fake world `alpha/aria`
- **WHEN** the user starts `mu client`, types `h`, presses Ctrl+Z, and presses Ctrl+Shift+Z
- **THEN** the editor is empty

## MODIFIED Requirements

### Requirement: The keys table binds a key to an entry
The table `[keys]` of a client layer SHALL map a key to the name of an entry of the registry, or to `"none"`. A key SHALL be modifiers from `ctrl`, `alt`, `shift` and `super`, then one key, joined with `+`. A key is one character that is not an uppercase letter, not a control character and not `+`, the name `plus`, or a name of a special key. Each binding SHALL replace the default binding of that key. A key bound to `"none"` SHALL do nothing. The keys that no layer names SHALL keep their defaults.

#### Scenario: A new binding
- **GIVEN** `clients/global.toml` holds `[keys]` with `"alt+n" = "focus-next"`, and a server serves that directory with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client` and presses Alt+N
- **THEN** the top row shows `[beta/bryn]`

#### Scenario: The default keys stay
- **GIVEN** `clients/global.toml` holds `[keys]` with `"alt+n" = "focus-next"`, and a server serves that directory with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client` and presses Ctrl+N
- **THEN** the top row shows `[beta/bryn]`

#### Scenario: A removed binding
- **GIVEN** `clients/global.toml` holds `[keys]` with `"ctrl+n" = "none"`, and a server serves that directory with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client` and presses Ctrl+N
- **THEN** the top row shows `[alpha/aria]`

#### Scenario: A binding replaces a default
- **GIVEN** `clients/global.toml` holds `[keys]` with `"ctrl+n" = "focus-prev"`, and a server serves that directory with the fake worlds `alpha/aria`, `beta/bryn` and `gamma/gil`
- **WHEN** the user starts `mu client` and presses Ctrl+N
- **THEN** the top row shows `[gamma/gil]`

#### Scenario: None on a key with a modifier
- **GIVEN** `clients/global.toml` holds `[keys]` with `"alt+enter" = "none"`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `hi` and presses Alt+Enter
- **THEN** the server records no player event
- **AND** the editor holds `hi`

### Requirement: The echo field sets the echo line
The field `echo` SHALL be one of `show`, `dim` and `hide`, and its default SHALL be `dim`. With `show`, the client SHALL show the echo line of a send with no dim attribute. With `dim`, it SHALL show it with the dim attribute. With `hide`, it SHALL NOT show it.

#### Scenario: The default dims
- **GIVEN** the configuration directory of the server holds no `clients` directory, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `hello` and presses Enter
- **THEN** the cells of `> hello` are dim

#### Scenario: Show
- **GIVEN** `clients/global.toml` holds `echo = "show"`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `hello` and presses Enter
- **THEN** the screen shows `> hello`, and its cells are not dim

#### Scenario: Hide
- **GIVEN** `clients/global.toml` holds `echo = "hide"`, and the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `hello` and presses Enter
- **THEN** the screen shows `You say, "hello"`
- **AND** the screen shows no row that starts with `> `

#### Scenario: A bad echo value
- **GIVEN** `clients/global.toml` holds `echo = "loud"`, and a server serves that directory on `<dir>/s.sock`
- **WHEN** the user runs `mu config show --server unix://<dir>/s.sock --device laptop`
- **THEN** stderr holds one line that contains `clients/global.toml` and `echo`

## REMOVED Requirements

### Requirement: The client reads client.toml
**Reason**: **BREAKING** The server owns the client configuration (#169). A local file brings back drift between the clients of one server.
**Migration**: Move the content of `client.toml` to `clients/global.toml` in the configuration directory of the server, without the table `[server]`.

### Requirement: A bad client.toml stops the client
**Reason**: The client reads no file. A bad client layer is skipped on the server, and the client shows a notice. See the `config-store` spec.
**Migration**: None. Read the notice on the screen, or run `mu config show`.

### Requirement: A bad binding stops the client
**Reason**: A bad binding now makes its layer bad, and the server skips that layer. See the requirement "A bad binding makes its layer bad".
**Migration**: None.

### Requirement: The server table sets the wait for a server
**Reason**: **BREAKING** The client needs the wait before a server answers, so it cannot come from the server.
**Migration**: Use the option `connect_wait` of the server URL, such as `--server "unix:///path/s.sock?connect_wait=2"`.
