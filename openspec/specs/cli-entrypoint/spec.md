# cli-entrypoint Specification

## Purpose
Defines the command grammar of the single `mu` binary, so that every function of the product is reached through one program with verbs.

## Requirements

### Requirement: One binary with verbs
The product SHALL ship as one executable named `mu`. Each function of the product SHALL be a verb of that executable. The verbs in this capability are `client`, `server` and `version`. Each verb SHALL have a one-line description in the help output.

#### Scenario: Help lists the verbs
- **WHEN** the user runs `mu --help`
- **THEN** stdout contains a line that starts with `Usage: mu`
- **AND** the commands section of stdout lists exactly three verbs: `client`, `server` and `version`
- **AND** the exit status is 0

#### Scenario: Help for one verb
- **WHEN** the user runs `mu server --help`
- **THEN** stdout contains the text `Usage: mu server`
- **AND** stdout contains the one-line description of the `server` verb that `mu --help` shows
- **AND** the exit status is 0

### Requirement: The client is the default verb
If the user gives no verb, `mu` SHALL behave exactly as `mu client` behaves.

#### Scenario: No verb
- **WHEN** the user runs `mu` with no arguments
- **THEN** stdout, stderr and the exit status are the same as those of `mu client`

### Requirement: Version output
`mu` SHALL report its name and version through the `version` verb and through the `--version` flag. Both SHALL write the same single line to stdout. The line SHALL be `mu`, one space, and the package version.

#### Scenario: Version verb
- **WHEN** the user runs `mu version`
- **THEN** stdout is exactly one line: `mu`, one space, and the package version
- **AND** the exit status is 0

#### Scenario: Version flag
- **WHEN** the user runs `mu --version`
- **THEN** stdout is byte for byte the same as the stdout of `mu version`
- **AND** the exit status is 0

### Requirement: Unknown verb is a usage error
If the user gives a verb that does not exist, `mu` SHALL write an error to stderr that names the verb. The exit status SHALL be 2. Nothing SHALL be written to stdout.

#### Scenario: Unknown verb
- **WHEN** the user runs `mu frobnicate`
- **THEN** stderr contains the text `frobnicate`
- **AND** stdout is empty
- **AND** the exit status is 2
