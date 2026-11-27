# Spec Delta

## ADDED Requirements

### Requirement: Each event has a unique id
Each event SHALL carry the field `id`. The `id` SHALL be a ULID, and the server SHALL make it at the record of the event. No two events SHALL have the same id, also across a restart of the server. A stream from an earlier sequence number SHALL carry the same id for each event as the live stream. The sequence number SHALL stay the order and the point of resume.

#### Scenario: Ids in one stream
- **GIVEN** a server runs with the fake world `alpha/aria`
- **WHEN** a client sends `one`, `two` and `three` to `alpha/aria` and reads the stream
- **THEN** each event on the stream has an `id` of 26 characters
- **AND** no two events on the stream have the same `id`

#### Scenario: Ids after a restart
- **GIVEN** a client read the ids of the events of a server, and the server then restarted with the same fake world `alpha/aria`
- **WHEN** the client reads the stream from sequence 1
- **THEN** no event on the new stream has an `id` that the client read before

### Requirement: Each event carries its streams
Each event SHALL carry the field `streams`, with 1 to 8 stream names. An event that no rule sends elsewhere SHALL have the one stream `main`. The journal SHALL keep the streams, so a stream from an earlier sequence number SHALL carry the same streams as the live stream.

#### Scenario: A live event and a replay
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^You say, "\[Public\]'` and `stream = ["public", "main"]`
- **AND** a server runs with `--config <dir>` and the fake world `alpha/aria`
- **WHEN** a client sends `[Public] hi` to `alpha/aria`, and a second client then reads the stream from sequence 1
- **THEN** both clients get the game event `You say, "[Public] hi"` with the streams `public` and `main`, and with the same `id`
- **AND** both clients get the game event `Welcome, aria.` with the one stream `main`

#### Scenario: No event without a stream
- **GIVEN** a server runs with the fake world `alpha/aria`
- **WHEN** a client sends `hi` and `@drop` to `alpha/aria`, and reads the stream
- **THEN** each event on the stream has at least one stream
