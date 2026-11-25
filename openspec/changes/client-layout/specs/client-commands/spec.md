# Spec Delta

## Purpose

Widens the argument of an entry: the entries of the layout take text, the id of a block or the name of a layout.

## MODIFIED Requirements

### Requirement: One registry names each entry
The client SHALL have one registry of entries. Each entry SHALL have a unique name in lowercase words joined with `-`, and a description of one line. Its argument SHALL be none, an optional world, a required world, text, the id of a block of the layout, or the name of a layout. A key of the keymap SHALL reach the client only through an entry. A command SHALL reach the client only through an entry. The help SHALL list the entries from the same registry.

#### Scenario: Help lists the entries
- **GIVEN** the client runs
- **WHEN** the user types `/help` and presses Enter
- **THEN** the screen shows a row that starts with `-- ` and holds `connect`
- **AND** the screen shows a row that starts with `-- ` and holds `focus-next`
- **AND** the server records no player event

#### Scenario: An entry takes the id of a block
- **GIVEN** the layout in use holds the lines `output`, `editor below main` and `editor as scratch right of editor`
- **WHEN** the client lists the values of the argument of the entry `pane-hide`
- **THEN** the values are `main`, `editor` and `scratch`
