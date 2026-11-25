# Spec Delta

## Purpose

Gives each part of a world id a limit of 64 characters, so that a world id is a key of `[world-layouts]` of a known length.

## MODIFIED Requirements

### Requirement: A world id names one character on one game
Each world SHALL have an id of the form `<game>/<character>`. Each id SHALL have exactly one slash. Each part SHALL hold 1 to 64 characters, only the ASCII letters `a` to `z`, the digits and hyphens, and SHALL start with a letter. Each command to a world SHALL name its id, and each event SHALL carry one.

#### Scenario: A bad world id is refused
- **WHEN** the user runs `mu server --socket <dir>/s.sock --fake-world Alpha_1=env:PW`
- **THEN** stderr contains `Alpha_1`
- **AND** the exit status is 2

#### Scenario: Ids outside the rule
- **WHEN** the engine parses `-alpha/aria`, `alpha`, `alpha/b/c`, `alpha/`, `1alpha/aria`, `alpha/arïa`, `Alpha/aria` and `alpha/a_b`
- **THEN** each parse fails

#### Scenario: Ids inside the rule
- **WHEN** the engine parses `alpha/aria`, `alpha-2/bea` and `a/b-9`
- **THEN** each parse succeeds, and the display of each id is the text it came from

#### Scenario: A part of 65 characters
- **WHEN** the engine parses an id whose game part has 65 characters, and an id whose two parts have 64 characters each
- **THEN** the first parse fails, and the second succeeds
