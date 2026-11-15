## ADDED Requirements

### Requirement: Each spike has a decision record
Each crate of the `spikes/` workspace SHALL have one decision page in `docs/src/content/docs/decisions/`. Its Source section SHALL name the crate. The page SHALL have the sections Decision, Reason, Not tested and Source, in that order. The Source section SHALL name the issue and the pull request of the spike.

#### Scenario: Each spike crate has a decision page
- **WHEN** a reader lists the member crates of the `spikes/` workspace and reads the Source section of each decision page
- **THEN** each crate is named as the path `spikes/<crate>`, or as a path under it, in the Source section of exactly one decision page
- **AND** a path whose last part only starts with the crate name, such as `spikes/<crate>-v2`, does not count as a name of that crate

#### Scenario: The decision page of a spike has the sections in order
- **WHEN** a reader lists the second-level headings of the decision page of a spike crate
- **THEN** the headings are Decision, Reason, Not tested and Source, in that order

#### Scenario: The Source section names the issue and the pull request
- **WHEN** a reader reads the Source section of the decision page of a spike crate
- **THEN** the section holds the text `issue <n>` and the text `pull request <n>`, each with a number

### Requirement: Production code includes no spike file
No Rust file in `crates/` SHALL include a file from `spikes/` at build time.

#### Scenario: No include of a spike file
- **WHEN** a reader searches each Rust file of a member crate of the main workspace for the macros `include!`, `include_str!` and `include_bytes!`, also inside the body of another macro, and for each attribute that sets `path`, also inside `cfg_attr`
- **THEN** no joined string argument of a match has `spikes` as a path component
- **AND** a file that cannot be read as Rust tokens is reported, not skipped

### Requirement: The design rule sends a spike change to the spikes page
The `design` rules in `openspec/config.yaml` SHALL hold one rule for a change that builds on a spike. The rule SHALL send the design to the part "Using a spike" of the spikes page.

#### Scenario: The design rules name the spikes page
- **WHEN** a reader reads the `rules.design` list in `openspec/config.yaml`
- **THEN** one rule holds both the text `builds on a spike` and the text `Using a spike`
