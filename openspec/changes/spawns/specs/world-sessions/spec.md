# Spec Delta

## ADDED Requirements

### Requirement: A fake world takes the rules of global.toml
If `mu server` runs with `--config <dir>`, each fake world SHALL take the rules of `global.toml` in that directory, after the `disable` of that file. A fake world SHALL take no other field of a layer. A bad rule in `global.toml` SHALL stop the start, as for a configured world.

#### Scenario: A fake world with a rule
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^You say, "\['` and `stream = "chat"`
- **AND** a server runs with `--config <dir>` and the fake world `alpha/aria`
- **WHEN** a client sends `[Public] hi` to `alpha/aria` and reads the stream
- **THEN** the game event `You say, "[Public] hi"` has the one stream `chat`

#### Scenario: A fake world with no rules
- **GIVEN** `global.toml` has no rule
- **AND** a server runs with `--config <dir>` and the fake world `alpha/aria`
- **WHEN** a client sends `[Public] hi` to `alpha/aria` and reads the stream
- **THEN** the game event `You say, "[Public] hi"` has the one stream `main`
