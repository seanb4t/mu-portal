# Spec Delta

## Purpose

Defines how a configured world reaches its game: the TCP connection, the read of the secret, the moment of the login, the keepalive and the reconnect after a drop.

## ADDED Requirements

### Requirement: A configured world connects over TCP
`Connect` on a configured world SHALL open a TCP connection to its `host` and `port`. The server SHALL resolve the host again at each attempt. A connect that does not end in 10 seconds SHALL fail. A failed connect SHALL record a disconnect event whose text starts with `the connect failed:` and names the cause.

#### Scenario: A world reaches its game
- **GIVEN** a fake game on TCP at `127.0.0.1:<port>`, and a configured world `alpha/aria` with that host and port and no login
- **WHEN** a client calls `Connect` for `alpha/aria`
- **THEN** the stream holds a connect event for `alpha/aria`
- **AND** `ListWorlds` returns `alpha/aria` in the state `connected`

#### Scenario: Nothing listens on the port
- **GIVEN** a configured world `alpha/aria` whose port on `127.0.0.1` has no listener, and `reconnect.mode = "off"`
- **WHEN** a client calls `Connect` for `alpha/aria`
- **THEN** the stream holds a disconnect event for `alpha/aria` whose text starts with `the connect failed:`
- **AND** `ListWorlds` returns `alpha/aria` in the state `disconnected`

#### Scenario: A connect that hangs
- **GIVEN** a world on a paused clock whose transport never ends its connect, and `reconnect.mode = "off"`
- **WHEN** the world connects, and the clock passes 10 seconds
- **THEN** the journal holds a disconnect event whose text starts with `the connect failed:` and names the timeout

### Requirement: A TLS world cannot connect yet
`Connect` on a world with `tls = true` SHALL fail with `failed_precondition`. The message SHALL name the world and TLS. The server SHALL record no event, open no connection and not reconnect.

#### Scenario: A TLS world
- **GIVEN** a configured world `alpha/aria` with `tls = true`
- **WHEN** a client calls `Connect` for `alpha/aria`
- **THEN** the call fails with `failed_precondition`, and the message contains `alpha/aria` and `TLS`
- **AND** the server records no event
- **AND** `ListWorlds` returns `alpha/aria` in the state `disconnected`

### Requirement: A configured world speaks telnet in its character set and window size
A configured world SHALL read and write the game through the telnet protocol. It SHALL decode and encode text in its `charset`, and SHALL report its `columns` and `rows` as its window size.

#### Scenario: A Latin-1 game
- **GIVEN** a configured world `alpha/aria` with `charset = "iso-8859-1"`, connected to a fake game on TCP
- **WHEN** the game sends the bytes `63 61 66 e9 0d 0a`, and a client sends `café` to `alpha/aria`
- **THEN** the stream holds a game event with the text `café`
- **AND** the game gets the bytes `63 61 66 e9 0d 0a`

#### Scenario: The window size of the world
- **GIVEN** a configured world `alpha/aria` with `columns = 100` and `rows = 30`, connected to a fake game on TCP
- **WHEN** the game sends `IAC DO NAWS`
- **THEN** the game gets `IAC WILL NAWS` and `IAC SB NAWS 0 100 0 30 IAC SE`

### Requirement: The secret is read at each connect
The server SHALL read the secret of a world at the start of each connect of that world, and not before. Each automatic attempt is a connect. If the secret of a `Connect` call cannot be read, the call SHALL fail with `failed_precondition`. The message SHALL name the variable, and the server SHALL record no event.

#### Scenario: A missing variable
- **GIVEN** a configured world `alpha/aria` with `login.secret = "env:ARIA_PW"`, and `ARIA_PW` is not set
- **WHEN** the server starts, and a client calls `Connect` for `alpha/aria`
- **THEN** the start succeeds
- **AND** the call fails with `failed_precondition`, and the message contains `ARIA_PW`
- **AND** the server records no event

#### Scenario: The login line hides the password
- **GIVEN** a configured world `alpha/aria` with `login.name = "Aria"`, `login.secret = "env:ARIA_PW"` and `ARIA_PW` set to `s3cret-pw`, connected to a fake game on TCP
- **WHEN** the world logs in
- **THEN** the game gets the login line with `s3cret-pw`
- **AND** no event of the journal contains `s3cret-pw`

### Requirement: A reconnect stops when the secret is gone
If the secret of an automatic attempt cannot be read, the world SHALL go to the state `disconnected`. It SHALL record a disconnect event that names the variable. It SHALL then make no more attempts.

#### Scenario: A secret that is gone at a reconnect
- **GIVEN** a world on a paused clock with a login, whose secret store gives the secret once and then fails
- **WHEN** the world connects, and the game closes the connection
- **THEN** the journal holds a disconnect event that names the variable
- **AND** the world is in the state `disconnected`
- **AND** the game saw exactly 1 connect

### Requirement: The login waits until the game is quiet
A configured world with a login SHALL wait for a quiet second before its login line. A quiet second is 1 second with no byte from the game, from the connect or from the last byte. The world SHALL send the line at the latest 10 seconds after the connect. Until the login line goes out, the world SHALL stay in the state `connecting`. In that time, it SHALL send the game only the replies of the telnet protocol. At the open of the connection, a world with no login SHALL go to the state `connected`.

#### Scenario: A banner and then quiet
- **GIVEN** a world with a login on a paused clock, and a fake game that sends a banner of three lines
- **WHEN** the world connects
- **THEN** the game gets the login line 1 second after the last line of the banner
- **AND** the game got no line before the login line

#### Scenario: A silent game
- **GIVEN** a world with a login on a paused clock, and a fake game that sends nothing
- **WHEN** the world connects
- **THEN** the game gets the login line 1 second after the connect

#### Scenario: A game that never goes quiet
- **GIVEN** a world with a login on a paused clock, and a fake game that sends one line each half second
- **WHEN** the world connects
- **THEN** the game gets the login line 10 seconds after the connect

#### Scenario: A send during the quiet second
- **GIVEN** a world with a login on a paused clock, and a fake game that sends nothing
- **WHEN** the world connects, and a client sends `hello` to the world half a second later
- **THEN** the call fails with `failed_precondition`
- **AND** the first line that the game gets is the login line
- **AND** the game gets no `hello`

#### Scenario: A world that logs in by hand
- **GIVEN** a configured world with no login, connected to a fake game on TCP on a paused clock
- **WHEN** 20 seconds pass
- **THEN** the game got no line

### Requirement: The keepalive keeps an idle world connected
A world with `keepalive.mode = "idle"` SHALL send `keepalive.command` each time `keepalive.seconds` pass with no write to the game. Each line written to the game SHALL start that time again. With `interval`, the world SHALL send the command every `keepalive.seconds`, whatever else it writes. With `off`, it SHALL send none. At the move of the world to the state `connected`, the time SHALL start. The journal SHALL NOT record a keepalive line.

#### Scenario: Idle
- **GIVEN** a world on a paused clock with `keepalive.mode = "idle"`, `keepalive.command = "IDLE"` and `keepalive.seconds = 60`, connected and logged in at the time 0
- **WHEN** a client sends `hello` at the time 30, and the clock reaches 200
- **THEN** the game gets `IDLE` at the times 90 and 150, and at no other time

#### Scenario: Interval
- **GIVEN** the same world with `keepalive.mode = "interval"`
- **WHEN** a client sends `hello` at the time 30, and the clock reaches 200
- **THEN** the game gets `IDLE` at the times 60, 120 and 180

#### Scenario: Off
- **GIVEN** the same world with `keepalive.mode = "off"`
- **WHEN** the clock reaches 200
- **THEN** the game gets no `IDLE`

#### Scenario: The journal holds no keepalive
- **GIVEN** the world with `keepalive.mode = "idle"` after the clock reached 200
- **WHEN** a test reads the journal of the world
- **THEN** no event contains `IDLE`

#### Scenario: No keepalive before the login
- **GIVEN** a world on a paused clock with a login, `keepalive.seconds = 1`, and a fake game that sends one line each half second
- **WHEN** the world connects
- **THEN** the first line that the game gets is the login line, 10 seconds after the connect

### Requirement: A world reconnects after a drop
With `reconnect.mode = "backoff"`, a world SHALL connect again after a drop that the player did not ask for, and after a failed connect. A drop that the player asked for is a `Disconnect` call or a disconnect from a handler. The world SHALL NOT reconnect after such a drop. It SHALL NOT reconnect after a TLS refusal or a failed read of the secret. It SHALL NOT reconnect at the stop of the server. With `reconnect.mode = "off"`, the world SHALL NOT reconnect.

#### Scenario: The game closes the connection
- **GIVEN** a configured world `alpha/aria` with the default reconnect, connected to a fake game on TCP
- **WHEN** the game closes the connection
- **THEN** `ListWorlds` returns `alpha/aria` in the state `reconnecting`
- **AND** the stream later holds a connect event for `alpha/aria`

#### Scenario: The player disconnects
- **GIVEN** a configured world `alpha/aria` with the default reconnect, connected
- **WHEN** a client calls `Disconnect` for `alpha/aria`
- **THEN** `ListWorlds` returns `alpha/aria` in the state `disconnected`
- **AND** the fake game gets no new connection

#### Scenario: Reconnect off
- **GIVEN** a configured world `alpha/aria` with `reconnect.mode = "off"`, connected
- **WHEN** the game closes the connection
- **THEN** `ListWorlds` returns `alpha/aria` in the state `disconnected`

### Requirement: The wait before each attempt grows
The wait before attempt `k`, from 1, SHALL be `min(initial * 2^(k-1), max)` seconds, plus a random extra time. The extra time SHALL be at least 0 and at most `min(0.05 * k, 0.20)` times that wait. The wait SHALL be defined for every `k`, with no overflow.

#### Scenario: The waits of a game that refuses every connect
- **GIVEN** a world on a paused clock with `reconnect.initial = 1`, `reconnect.max = 60` and `reconnect.attempts = 10`, whose game refuses every connect
- **WHEN** the world connects and the clock runs until the world stops
- **THEN** the wait before attempt `k` is at least `min(2^(k-1), 60)` seconds and at most 1 + `min(0.05 * k, 0.20)` times that

#### Scenario: The extra time at its cap
- **GIVEN** the pairs `initial = 1` with `max = 60`, and `initial = 3600` with `max = 86400`
- **WHEN** a test asks the wait policy 1000 times for each attempt from 4 up to the largest attempt number, with each pair
- **THEN** no wait is outside the bounds above, and no call panics
- **AND** for attempts from 4 on, the extra time reaches above 15% of the wait in at least one draw

### Requirement: A world stops after its last attempt
An attempt SHALL count as failed unless its connection stays up for `reconnect.max` seconds. A world SHALL stop after `reconnect.attempts` failed attempts in a row. Then it SHALL go to the state `disconnected` and record a disconnect event whose text names the count. `reconnect.attempts = 0` SHALL mean no limit. A connection that stays up for `reconnect.max` seconds SHALL start the count and the wait again from attempt 1.

#### Scenario: The last attempt
- **GIVEN** a world on a paused clock with `reconnect.attempts = 3`, whose game refuses every connect
- **WHEN** the world connects and the clock runs until the world stops
- **THEN** the game saw exactly 4 connects: the first and 3 attempts
- **AND** the journal holds a disconnect event that contains `3 attempts`
- **AND** the world is in the state `disconnected`

#### Scenario: A game that closes each connection after the login
- **GIVEN** a world on a paused clock with a login and `reconnect.attempts = 3`, whose game closes each connection 2 seconds after the login line
- **WHEN** the world connects and the clock runs until the world stops
- **THEN** the game saw exactly 4 connects
- **AND** the journal holds a disconnect event that contains `3 attempts`

#### Scenario: No limit
- **GIVEN** a world on a paused clock with `reconnect.attempts = 0` and `reconnect.max = 60`, whose game refuses every connect
- **WHEN** the clock runs through 40 attempts
- **THEN** the world is in the state `reconnecting`

#### Scenario: A stable connection starts the count again
- **GIVEN** a world on a paused clock with `reconnect.initial = 1` and `reconnect.max = 60`, which reconnected after 3 failed attempts
- **WHEN** the connection stays up for 60 seconds, and then the game closes it
- **THEN** the wait before the next attempt is at most 1.05 seconds

#### Scenario: A short connection keeps the count
- **GIVEN** the same world, which reconnected after 3 failed attempts
- **WHEN** the connection stays up for 59 seconds, and then the game closes it
- **THEN** the wait before the next attempt is at least 8 seconds

### Requirement: Connect and Disconnect act on a world that waits
While a world is in the state `reconnecting`, `Connect` SHALL start an attempt at once and start the count again. `Disconnect` SHALL end the wait, and the world SHALL go to the state `disconnected`.

#### Scenario: Connect during a wait
- **GIVEN** a world with `reconnect.initial = 1` in the state `reconnecting` before attempt 5, whose game refuses every connect
- **WHEN** a client calls `Connect` for the world
- **THEN** the fake game gets a connection before the clock passes 1 second
- **AND** the wait after that refused connect is at most 1.05 seconds

#### Scenario: Disconnect during a wait
- **GIVEN** a world in the state `reconnecting`
- **WHEN** a client calls `Disconnect` for the world, and the clock passes 120 seconds
- **THEN** the world is in the state `disconnected`
- **AND** the fake game gets no new connection
