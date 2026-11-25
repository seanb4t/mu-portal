# Spec Delta

## Purpose

Adds the verb `layout` to the verbs of the one binary.

## MODIFIED Requirements

### Requirement: One binary with verbs
The product SHALL ship as one executable named `mu`. Each function of the product SHALL be a verb of that executable. The verbs in this capability are `client`, `config`, `layout`, `server`, `version` and `world`. Each verb SHALL have a one-line description in the help output.

#### Scenario: Help lists the verbs
- **WHEN** the user runs `mu --help`
- **THEN** stdout contains a line that starts with `Usage: mu`
- **AND** the commands section of stdout lists exactly six verbs: `client`, `config`, `layout`, `server`, `version` and `world`
- **AND** the exit status is 0

#### Scenario: Help for one verb
- **WHEN** the user runs `mu server --help`
- **THEN** stdout contains the text `Usage: mu server`
- **AND** stdout contains the one-line description of the `server` verb that `mu --help` shows
- **AND** the exit status is 0
