# Spec Delta

## ADDED Requirements

### Requirement: The top row is the tabs block
In this spec, the top row SHALL be the block `tabs` of the layout. With the layout `classic`, it SHALL be the first row of the screen. The text of each world in it SHALL be the tab label of the world, which is the world id by default.

#### Scenario: The default layout puts the tabs first
- **GIVEN** the configuration directory of the server holds no `clients` directory, and the server has the fake world `alpha/aria`
- **WHEN** the user starts `mu client` in a terminal of 80 columns and 24 rows
- **THEN** row 1 of the screen shows `[alpha/aria]`
