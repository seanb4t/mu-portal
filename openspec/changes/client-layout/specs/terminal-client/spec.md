# Spec Delta

## ADDED Requirements

### Requirement: The top row is the tabs block
In this spec, the top row SHALL be the block `tabs` of the layout. With the layout `classic`, it SHALL be the first row of the screen. The text of each world in it SHALL be the tab label of the world, which is the world id by default.

#### Scenario: The default layout puts the tabs first
- **GIVEN** the configuration directory of the server holds no `clients` directory, and the server has the fake world `alpha/aria`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** row 1 of the screen shows `[alpha/aria]`

## MODIFIED Requirements

### Requirement: The screen has three parts
The client SHALL use the alternate screen. With the layout `classic`, the top row SHALL list the worlds in order by game part and then by character part, with the world in focus in square brackets, or `no worlds` with no world. The bottom rows SHALL hold the pose editor, which takes one row with no text. The rows between SHALL show the output pane of the world in focus, at its bottom at the start. Another layout SHALL place these parts as its plan says.

#### Scenario: The first view
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** the top row shows `[alpha/aria]` and then `beta/bryn`
- **AND** the rows between show `Welcome, aria.`
- **AND** the screen shows no line of `beta/bryn`
- **AND** the editor takes only the bottom row

#### Scenario: The order of the worlds
- **GIVEN** a server with the fake worlds `alpha-2/cy`, `alpha/bea` and `alpha/aria`
- **WHEN** the user starts `mu client`
- **THEN** the top row shows `[alpha/aria]`, then `alpha/bea`, then `alpha-2/cy`

#### Scenario: No worlds
- **GIVEN** a server with no world
- **WHEN** the user starts `mu client`
- **THEN** the top row shows `no worlds`
