# Spec Delta

## Purpose

Defines the journal of events that the server keeps for all worlds, and the one event stream through which a client reads it and resumes.

## ADDED Requirements

### Requirement: One sequence for all worlds
The server SHALL give each event a sequence number. The numbers SHALL start at 1 and grow by 1 for each event, across all worlds, with no gap. Each event SHALL carry the id of its world, its kind, its source and its text.

#### Scenario: Events of two worlds share one sequence
- **GIVEN** a server with the fake worlds `alpha/aria` and `beta/bryn`
- **WHEN** a client sends `one` to `alpha/aria` and `two` to `beta/bryn`, and reads the stream from sequence 1
- **THEN** the sequence numbers of the events are 1, 2, 3 and so on, with no gap and no repeat
- **AND** each event names `alpha/aria` or `beta/bryn` as its world

### Requirement: Each event names its source
Each event SHALL have exactly one source. The sources are the game, the player, a script, the system and a credential. A line from the game SHALL have the source game. A line that a client sends SHALL have the source player. A connect or a disconnect SHALL have the source system. A login line SHALL have the source credential.

#### Scenario: Sources of a session
- **GIVEN** a server with the fake world `alpha/aria`
- **WHEN** a client sends the line `hello` to `alpha/aria` and reads the stream
- **THEN** the event of the connect has the source system
- **AND** the event of the login line has the source credential
- **AND** the event of the sent line `hello` has the source player
- **AND** the event of the echo of `hello` from the fake world has the source game

### Requirement: The journal never holds a password
The server SHALL send the real password to the game. Before any event of a world reaches the journal, the server SHALL replace each occurrence of the password of that world in its text with `****`. If the result still holds the password, the whole text SHALL become `****`. No event, and no line that the server writes to stderr or to its log, SHALL contain the password.

#### Scenario: The password is replaced
- **GIVEN** the environment variable `ALPHA_PW` holds `s3cret-pw`, and the fake world `alpha/aria` reads its password from it
- **WHEN** the world connects and logs in
- **THEN** the fake world reports that the login succeeded
- **AND** the stream holds a login event with the text `connect aria ****`
- **AND** no event in the stream contains `s3cret-pw`
- **AND** the stderr of the server does not contain `s3cret-pw`

#### Scenario: A typed password is masked
- **GIVEN** the fake world `alpha/aria` reads its password `s3cret-pw` from `ALPHA_PW`
- **WHEN** a client sends `my pw is s3cret-pw` to `alpha/aria`
- **THEN** the stream holds a player event with the text `my pw is ****`
- **AND** the stream holds a game event with the text `You say, "my pw is ****"`
- **AND** no event in the stream contains `s3cret-pw`

#### Scenario: A password with a star
- **GIVEN** the password of the fake world `alpha/aria` is `a*`
- **WHEN** a client sends `aa*` to `alpha/aria`
- **THEN** the player event of the line has the text `****`
- **AND** no event in the stream contains `a*`

### Requirement: A stream starts from a sequence number
The `Subscribe` call SHALL take the first sequence number the client wants. The stream SHALL send every recorded event from that number in order, and then each new event as it arrives. 0 and 1 SHALL both mean the start of the journal. The first message of each stream SHALL carry the id of the server instance and the last sequence number at the time of the call.

#### Scenario: A stream from the middle
- **GIVEN** the journal holds events 1 to 6
- **WHEN** a client subscribes from sequence 4
- **THEN** the first message carries the id of the server instance and the last sequence number 6
- **AND** the stream then sends the events 4, 5 and 6 in order
- **AND** the next event that the server records arrives on the same stream as event 7

#### Scenario: A stream from 0
- **GIVEN** the journal holds events 1 to 6
- **WHEN** a client subscribes from sequence 0
- **THEN** the stream sends the events 1 to 6 in order after the first message

#### Scenario: A stream from the next number
- **GIVEN** the journal holds events 1 to 6
- **WHEN** a client subscribes from sequence 7
- **THEN** the first message carries the last sequence number 6
- **AND** the next event on the stream is the next event that the server records, as event 7

### Requirement: A stream past the end ends
If the first sequence number of `Subscribe` is greater than the last sequence number plus 1, the stream SHALL send the first message and then end with the code `out_of_range`. Thus a client that resumes after a restart reads the new server instance id first.

#### Scenario: A stream past the end
- **GIVEN** the journal holds events 1 to 6
- **WHEN** a client subscribes from sequence 8
- **THEN** the first message carries the id of the server instance and the last sequence number 6
- **AND** the stream then ends with the code `out_of_range`

### Requirement: A client resumes without a gap
A client that loses its stream SHALL subscribe again from the sequence number after the last event it read. The events of both streams together SHALL hold each sequence number exactly once and in order. A new server instance id SHALL tell the client that the server restarted. In that case the client SHALL drop its events and subscribe from sequence 1.

#### Scenario: Resume after a cut connection
- **GIVEN** a client reads the stream through a relay
- **WHEN** the relay cuts the connection, the server records two events, and the client resumes
- **THEN** the client reads each sequence number once, in order, with no gap

#### Scenario: Resume after a restart of the server
- **GIVEN** a client read events 1 to 5 from a server
- **WHEN** the server restarts and the client resumes
- **THEN** the first message of the new stream carries a different server instance id
- **AND** the client shows only the events of the new server instance

### Requirement: A slow subscriber cannot block the server
If a subscriber falls too far behind the live events, the server SHALL end its stream with the code `resource_exhausted`. The other subscribers and the worlds SHALL go on without a pause.

#### Scenario: A subscriber that does not read
- **GIVEN** a journal whose live buffer holds 4 events, one subscriber that reads nothing yet, and a second subscriber that reads
- **WHEN** the journal records 5 events, and the first subscriber then reads
- **THEN** the first read of the first subscriber gives a lag
- **AND** the stream of that subscriber ends with the code `resource_exhausted`
- **AND** the second subscriber reads every event, in order

#### Scenario: A subscriber at the edge of the buffer
- **GIVEN** a journal whose live buffer holds 4 events, and one subscriber that reads nothing yet
- **WHEN** the journal records 4 events, and the subscriber then reads
- **THEN** the subscriber reads the events 1 to 4 in order
