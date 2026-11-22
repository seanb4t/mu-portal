## Purpose

Defines the registry of what the player can ask the terminal client to do, and the client commands: a line that starts with `/` and goes to the client, not to the game.

## ADDED Requirements

### Requirement: One registry names each entry
The client SHALL have one registry of entries. Each entry SHALL have a unique name in lowercase words joined with `-`, and a description of one line. Its argument SHALL be none, an optional world or a required world. A key of the keymap SHALL reach the client only through an entry. A command SHALL reach the client only through an entry. The help SHALL list the entries from the same registry.

#### Scenario: Help lists the entries
- **GIVEN** the client runs
- **WHEN** the user types `/help` and presses Enter
- **THEN** the screen shows a row that starts with `-- ` and holds `connect`
- **AND** the screen shows a row that starts with `-- ` and holds `focus-next`
- **AND** the server records no player event

### Requirement: Each front end runs an entry the same way
A key and a command SHALL only choose an entry and its argument. The client SHALL run the entry the same way for each of them. An entry that acts on the text of the editor SHALL NOT run from a command, because the command is that text. A key can run every entry.

#### Scenario: A key and a command give the same focus
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, and the world in focus is `alpha/aria`
- **WHEN** the user presses Ctrl+N, or types `/focus-next` and presses Enter
- **THEN** in both cases the top row shows `[beta/bryn]`

#### Scenario: An editing entry is not a command
- **WHEN** the user types `/undo` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `not a command`
- **AND** the editor holds `/undo`

### Requirement: A line that starts with a slash is a command
A command is a text of one line that starts with `/` and not with `//`. Enter and Alt+Enter SHALL run a command and SHALL NOT send it to the game. The first word after `/` SHALL name an entry, and the rest SHALL be its argument. A command that runs SHALL clear the editor. A text of several lines SHALL NOT run a command.

#### Scenario: A command does not reach the game
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `/focus-next` and presses Enter
- **THEN** the server records no player event
- **AND** the editor is empty

#### Scenario: Several lines are not a command
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `/quit`, presses Ctrl+J, types `two` and presses Alt+Enter
- **THEN** the server records the player events `/quit` and `two` for `alpha/aria`
- **AND** the client is still running

### Requirement: Two slashes send one slash
If the text is one line that starts with `//`, the client SHALL send it without its first `/`.

#### Scenario: An escaped slash
- **GIVEN** the world in focus is the fake world `alpha/aria`
- **WHEN** the user types `//me waves` and presses Enter
- **THEN** the server records a player event `/me waves` for `alpha/aria`

### Requirement: A bad command keeps its text
A bad command names no entry, or an entry that does not run as a command. It can also lack a required world, or name a world that the client does not hold. It can also give an argument to an entry that takes none. An argument of only spaces SHALL count as no argument. For a bad command, the client SHALL show a line with the prefix `-- ` that says why. The text SHALL stay in the editor, and the client SHALL send nothing.

#### Scenario: An unknown command
- **WHEN** the user types `/fly` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `unknown command`
- **AND** the editor holds `/fly`

#### Scenario: A missing world
- **WHEN** the user types `/focus` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `needs a world`
- **AND** the editor holds `/focus`

#### Scenario: An argument to an entry that takes none
- **WHEN** the user types `/quit now` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `takes no argument`
- **AND** the editor holds `/quit now`
- **AND** the client is still running

#### Scenario: An unknown world
- **GIVEN** a server with the fake world `alpha/aria`
- **WHEN** the user types `/focus gamma/gil` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `gamma/gil`
- **AND** the editor holds `/focus gamma/gil`

### Requirement: Connect and disconnect a world
`/connect` SHALL ask the server to connect the world that it names, and `/disconnect` to disconnect it. With no world, each SHALL act on the world in focus. With no world and no world in focus, it SHALL be a bad command. If the server refuses, the client SHALL show the reason as a line with the prefix `-- `.

#### Scenario: Connect the world in focus
- **GIVEN** the world in focus is the fake world `alpha/aria`, and a test disconnected it through the API
- **WHEN** the user types `/connect` and presses Enter
- **THEN** the server connects `alpha/aria`
- **AND** the screen shows `Welcome, aria.` after the disconnect

#### Scenario: Disconnect a named world
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, and the world in focus is `alpha/aria`
- **WHEN** the user types `/disconnect beta/bryn` and presses Enter
- **THEN** the server reports `beta/bryn` as disconnected
- **AND** `alpha/aria` is still connected

#### Scenario: Connect with no world in focus
- **GIVEN** a server with no world
- **WHEN** the user types `/connect` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `no world`
- **AND** the editor holds `/connect`

#### Scenario: A trailing space is no argument
- **GIVEN** the world in focus is the fake world `alpha/aria`, and a test disconnected it through the API
- **WHEN** the user types `/connect ` with a trailing space and presses Enter
- **THEN** the server connects `alpha/aria`

#### Scenario: Disconnect an unknown world
- **GIVEN** a server with the fake world `alpha/aria`
- **WHEN** the user types `/disconnect gamma/gil` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and contains `gamma/gil`
- **AND** `alpha/aria` is still connected

### Requirement: Focus a world by name
`/focus <world>` SHALL give the focus to that world.

#### Scenario: Focus by name
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`, and the world in focus is `alpha/aria`
- **WHEN** the user types `/focus beta/bryn` and presses Enter
- **THEN** the top row shows `[beta/bryn]`

### Requirement: Quit by command
`/quit` SHALL do what Ctrl+C does, under the requirement "Ctrl+C quits the client only". If a `/quit` armed the quit because texts wait, the keys that type the next command SHALL NOT cancel the arm. A second `/quit` SHALL then quit. Any other send or command SHALL cancel the arm.

#### Scenario: Quit
- **GIVEN** no text of a world waits for the server
- **WHEN** the user types `/quit` and presses Enter
- **THEN** the client exits with status 0
- **AND** the server is still running

#### Scenario: Quit twice while a text waits
- **GIVEN** a text of `alpha/aria` waits for the server
- **WHEN** the user types `/quit` and presses Enter
- **THEN** the screen shows a line that starts with `-- ` and holds the count of the texts that wait
- **WHEN** the user types `/quit` and presses Enter again
- **THEN** the client exits with status 0

#### Scenario: A command with no world in focus
- **GIVEN** a server with no world
- **WHEN** the user types `/help` and presses Enter
- **THEN** the screen shows a row that starts with `-- ` and holds `connect`
- **AND** the client is still running
