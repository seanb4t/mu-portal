# Design

## Context

See proposal.md for the reason. The engine already has the seams that this change fills:

- `Transport` in `crates/mu-core/src/transport.rs` opens a byte stream for a world. Only `FakeTransport` implements it.
- `WorldSetup::without_transport` gives each configured world no transport, so `Connect` fails with `PortalError::NoTransport`.
- The session in `crates/mu-core/src/session.rs` holds the state of one world as `Conn::Down`, `Connecting` or `Up`. It sends the login in `on_connect` as soon as the stream opens. The `Connect` command handler opens the `world.connection` span.
- `SessionParts.password` is read at the start through `load_password`, which refuses a masked or multi-line password. The `world-config` spec says that the server never reads the secret of a configured world at the start.
- `TelnetFramer` and `TelnetSettings` exist from the telnet-engine change, but no world uses them. Every world uses `LineFramer`.
- `Portal::connect_all` sends `Connect` to every world at the start. Configured worlds then fail with `NoTransport`, and nothing reports it.
- `ResolvedWorld` holds `keepalive` with `command` and `seconds`, but nothing sends it.

The quirks of the three server families come from the telnet spike: decision page `telnet-engine`, issue #7, pull request #32. The transcripts are already test data in `crates/mu-core/tests/fixtures/telnet`. Pull request #32 has one review thread, about the `[TTABLE ]` prefix of a CHARSET request. The telnet-engine change owns that parser, and this change does not touch it.

## Goals / Non-Goals

**Goals:**

- The login moment is a strategy behind a trait. A later change then adds a banner trigger with no change to the session loop.
- The keepalive and the reconnect are each one enum in `mu-core`. The configuration and the session use the same type, so each kind exists once.
- Every timer runs on the tokio clock, so the paused clock tests each scenario with no real wait.
- The new TCP fake game does not use the framer of the engine. A defect that is the same on both sides then cannot pass.

**Non-Goals:**

- TLS (#58), a login moment that matches a banner (a later strategy), detection of a refused login, and Docker tests of real games (#82).
- The items under Not tested on the `telnet-engine` decision page. One is the cause of the TinyMUX stall after UTF-8. The `tinymux` type uses ISO-8859-1, so the stall does not occur.
- The other items on that list: PennMUSH and RhostMUSH (not Phase 1 targets), MCCP3, MXP and MSDP.
- Servers with a configuration other than the default, and long sessions. Only the owner's first evening covers them.
- The suggestions of the plan review, R-31 to R-36. Issue #103 tracks them.

## Decisions

### 1. The session gets a fourth connection state, `Waiting`

`Conn` gains `Waiting { attempt, until }`. The session loop already selects on the requests and on `progress(&mut conn)`. For `Waiting`, `progress` sleeps until `until` and then starts the attempt. A `Connect` request in `Waiting` starts a connect at once and sets the count of failed attempts to 0. A `Connect` call is not an attempt. After a failed `Connect`, the world therefore waits `wait(1)`, as after the first connect. A `Disconnect` request moves to `Down`. `Portal::stop` sends a `Disconnect`, so no attempt starts after the stop of the server. A control frame in `Waiting` gets its answer at once, as in `Down`.

`WorldState` gains `Reconnecting`. The `watch` channel of the state carries the state and, for `Reconnecting`, the wall-clock time of the next attempt. At the start of the wait, the session computes that time from the tokio deadline. Only the API reads it.

One function of the session starts every connect: the `Connect` handler and each attempt from `Waiting` call it. It reads the secret (decision 7), opens the `world.connection` span, and wraps the connect of the transport in one `tokio::time::timeout` of 10 seconds. The timeout therefore holds for every transport, the test transports included, and its expiry is the cause `Timeout`.

Alternative: a separate reconnect task per world that sends `Connect` after a sleep. That task races with the requests of the player, and the session does not know the state `reconnecting`. Rejected.

Enforced by: engine tests on the paused clock. They run the scenarios of "A world reconnects after a drop", "A connect that hangs" and "Connect and Disconnect act on a world that waits".

### 2. Why a world goes down decides whether it reconnects

`Session::down` takes a cause: `ConnectFailed`, `Timeout`, `Lost`, `Closed` or `Requested(Origin)`. A `Disconnect` command gives `Requested` with the origin that it already carries. `Portal::stop` sends its `Disconnect` with `Origin::System`.

| Cause | Reconnects | `mu.connection.end` | Span status |
|---|---|---|---|
| `ConnectFailed` | yes | `connect_failed` | `error` |
| `Timeout` | yes | `timeout` | `error` |
| `Lost` | yes | `lost` | `error` |
| `Closed` | yes | `closed` | none |
| `Requested(Player)`, `Requested(Script)` | no | `player` | none |
| `Requested(System)` | no | `stopped` | none |

A refused TLS connect and a missing secret at a `Connect` call fail the call before any state change, so they never reach `down`. A missing secret at an automatic attempt ends the wait in `Down`, before any span opens. It does not go through `down`, so it has no row in the table.

Enforced by: a unit test of each row of the table, and the scenarios of the reconnect requirement.

### 3. The login moment is a strategy, and the login gates the state `connected`

`LoginMoment` is a trait with two kinds in this change: `Immediate` for a fake world, and `Quiet { quiet: 1 s, cap: 10 s }` for a configured world with a login. It has pure methods over the tokio `Instant`. For example, it gives the deadline of the login from the connect time and the time of the last byte. It holds no timer, so a unit test drives it with numbers.

The login moment counts every byte from the game, telnet negotiation included. Negotiation is part of the noise before a game is ready, and TinyMUX can drop input that arrives during it.

From the open of the stream to the login line, the world stays in the state `connecting`. In that time:

- `Send` and a script command fail with `failed_precondition`, as for any world in `connecting`.
- A control frame stays in the queue. The session writes its mark after the login line, or answers it at `down`.
- No keepalive deadline runs.

At the login line, the world goes to `connected` and the keepalive deadline starts. A world with no login goes to `connected` at the open of the stream. `Immediate` gives a fake world the same order as today, so each current scenario of the fake world stays true with no new wait.

Alternative: one enum for the login moment, matched in the session. A banner strategy then changes the session loop. Rejected, because the owner asked for a strategy pattern to leave room for the banner match.

Enforced by: unit tests of each kind. Also by the paused-clock scenarios of "The login waits until the game is quiet" and "A wait of the engine covers a pending login".

### 4. The keepalive and the reconnect are enums

`mu-core` defines `Keepalive { mode, command, seconds }` with `mode` one of `Idle` and `Interval`, and `Reconnect` with the kinds `Backoff { initial, max, attempts }` and `Off`. Each type has pure methods that give the next deadline. `ResolvedWorld` holds `keepalive: Option<Keepalive>` and `reconnect: Reconnect`, and the session uses the same values. A world whose layers set neither half of the keepalive gets `None`, and so does `keepalive.mode = "off"`. A fake world gets `None` and `Reconnect::Off`.

Enforced by: unit tests of each kind, and the scenarios of the keepalive requirement on the paused clock.

### 5. The wait policy

`wait(k) = min(initial * 2^(k-1), max)` and `extra(k) = uniform(0, min(0.05 * k, 0.20)) * wait(k)`. The extra time only adds, so no wait is shorter than the backoff. The doubling saturates: once `initial * 2^(k-1)` reaches `max`, or the shift passes the width of the integer, the wait is `max`. No `k` overflows.

An attempt counts as failed unless its connection stays up for `max` seconds. After `attempts` failed attempts in a row, the world goes to `Down` with a disconnect event that names the count. `attempts = 0` never stops. A connection that stays up for `max` seconds resets `k` to 1.

The count of short connections is a choice of the owner, made in the plan review. Without it, a game that closes each connection right after a bad login never reaches the cap. The reset after `max` seconds is a design choice of this change. A game that drops each connection early still moves the wait up to the cap.

The policy is one function in `mu-core` on `Reconnect`. The function takes the random draw as an argument, so a test controls it. `backon` does not fit: its jitter adds a random fraction from 0 to 1 of the delay, the same fraction range at each attempt, with no cap. The table "Checked facts" gives the source.

Enforced by: the scenarios of "The wait before each attempt grows" and "A world stops after its last attempt". One of them is a property test over many draws, up to the largest attempt number.

### 6. The TCP transport

`TcpTransport { host, port }` in `mu-core` implements `Transport`. It resolves the host and connects in one future, so a slow name lookup also counts against the timeout of decision 1. It sets `TCP_NODELAY`, because a MU sends short lines. A world with `tls = true` gets no transport. `PortalError::NoTransport` becomes `PortalError::TlsNotSupported`, whose message names the world and TLS, and `mu-server` maps it to `failed_precondition`.

`TcpStream::connect` takes a `(&str, u16)` pair. For a name that is not an IP address, it resolves the name with a blocking task at each call. The table "Checked facts" gives the source.

Enforced by: the scenarios of "A configured world connects over TCP" and "A TLS world cannot connect yet".

### 7. The secret is a source, read at each connect

`SessionParts.password: Option<Secret>` becomes `secret: Option<SecretSource>`. A world with no login has `None`. `SecretSource` has two cases:

- `Fixed(Secret)` for a fake world. The flag reads its password at the start, as today.
- `Store(SecretRef, Arc<dyn SecretStore>)` for a configured world. It reads through `load_password`, so a masked or multi-line password fails as it does for the flag.

The connect function of decision 1 reads the source first. A failed read returns `PortalError::NoSecret`, which names the variable. `NoSecret` replaces `NoPassword` at both checks of the session, and `NoPassword` leaves `PortalError` and its row in `crates/mu-server/src/convert.rs`. At a `Connect` call, the failed read records nothing. At an automatic attempt, the world goes to `Down` with a disconnect event that names the variable. The session keeps the secret of the open connection only, so that `record` can mask the login line.

Enforced by: the scenarios of "The secret is read at each connect".

### 8. The framer of a configured world

`mu-cli` builds `TelnetFramer::new(TelnetSettings { charset, columns, rows, policy })` for each configured world, with the policy of `TelnetSettings::default()`. A fake world keeps `LineFramer`.

Enforced by: the scenarios of "A configured world speaks telnet in its character set and window size".

### 9. Start-up connects

`Portal::connect_all` becomes `Portal::connect_at_start`. It sends `Connect` to each world whose setup asks for it: a fake world, and a configured world with `autoconnect = true`. A `Connect` that fails at the start goes to the log at `WARN` and does not stop the server. An example is a TLS world with `autoconnect`.

Enforced by: the scenarios of "Worlds connect at the start or on request", "A failed connect at the start" among them.

### 10. The configuration fields

The layers gain `autoconnect`, `keepalive.mode` and the table `reconnect`. The resolution builds the `Keepalive` and `Reconnect` values of decision 4. The defaults join `WorldDefaults`, so `mu world show` names the source `default` with no new code path. `mu world show` prints `keepalive.mode` from the layers and the defaults, also for a world with no keepalive.

The quirk schema of the built-in types gets its own keepalive table with `command` and `seconds` only. It no longer reuses `KeepaliveFields`, which gains `mode`. The schema refuses `keepalive.mode`, `autoconnect` and `reconnect`. The rule for a shared plugin adds `autoconnect` to its list.

Enforced by: the scenarios of the `world-config` delta, and a unit test of `parse_quirks` for each refused field.

### 11. The API

`portal.proto` gains `WORLD_STATE_RECONNECTING = 4` and `optional int64 next_attempt_ms = 3` on `World`. The time is in milliseconds since the Unix epoch, like `time_ms` of an event. The terminal client shows the id of a world only, so it needs no change.

Enforced by: the scenario "A world that waits to reconnect" of the world-sessions delta, as a binary test.

### 12. Telemetry (#95)

The connect function of decision 1 opens the `world.connection` span for each attempt. At `down`, the session records `mu.connection.end` and the status from the table of decision 2. A failed command sets the status `error` on its `command` span. The keepalive adds 1 to `mu.keepalive.sent`.

`tracing-opentelemetry` sets the span status from the special field `otel.status_code`, and the API spans in `mu-server` already use it. For a later record of a field, `tracing` needs the field in the declaration of the span. The `world.connection` span therefore declares `mu.connection.end` and `otel.status_code` as `tracing::field::Empty`, and `down` records them.

Enforced by: the scenarios of the telemetry delta, with the fake collector of the telemetry tests.

### 13. The crate `mu-fake-server`

| Part | Origin | Uses the framer of the engine |
|---|---|---|
| In-memory fake game and `FakeTransport` | moved from `crates/mu-core/src/fake.rs`, unchanged | yes, as today |
| TCP fake game | new | no: it reads and writes raw bytes |
| Transcript replay | new | no |

The TCP fake game listens on `127.0.0.1:0` and runs a script of steps: send bytes, wait for bytes, go silent, close. A test reads every byte that the client wrote. Replay takes the bytes of a transcript from the caller. The transcripts then stay in `crates/mu-core/tests/fixtures/telnet`, and nothing in `crates/` reads `spikes/`.

The scripted fake game runs over a TCP socket or over an in-memory `tokio::io::duplex` stream. The binary tests use TCP. The engine tests on the paused clock use the duplex stream. With no work in the runtime, a paused clock jumps to the next timer. A wait for a real socket counts as no work. A timer can then fire before bytes in flight arrive.

`mu-fake-server` depends on `mu-core` for `Transport`, `WorldId` and the line framer. `mu-cli` depends on it for `--fake-world`. The integration tests of `mu-core` also use the fake, so `mu-core` gets `mu-fake-server` as a dev-dependency. Cargo accepts this cycle for integration tests only, so no unit test inside `mu-core` can use the fake.

`ALLOWED_EDGES` in `crates/mu-cli/tests/workspace_boundary.rs` reads every dependency table the same way. An edge from `mu-core` to `mu-fake-server` in that list also allows a normal dependency. The boundary test therefore gains `ALLOWED_DEV_EDGES`, a list of edges that only `[dev-dependencies]` can use. `mu-core → mu-fake-server` goes there, and `mu-fake-server → mu-core` and `mu-cli → mu-fake-server` go into `ALLOWED_EDGES`. The LikeC4 model in `architecture/` gains the crate.

Alternative: keep the fakes in `mu-core` behind a feature. The shipped binary needs the in-memory fake for `--fake-world`, so the feature is always on. Rejected.

Enforced by: the boundary test, `mise run docs:check`, and every test that uses a fake game.

## Checked facts

Task 1.1 read each source on 2026-11-19.

| Fact | Source | Result |
|---|---|---|
| `TcpStream::connect` with `(&str, u16)` resolves the name at each call | tokio 1.54.1, `src/net/addr.rs`, `impl ToSocketAddrsPriv for (&str, u16)` | Holds: it parses an IP address, or else it resolves the name with `spawn_blocking`. |
| `TcpStream::set_nodelay` exists | docs.rs, `tokio::net::TcpStream` | Holds. |
| `tokio::io::duplex` and the paused clock | docs.rs, `tokio::io::duplex`, and tokio 1.54.1, `src/time/clock.rs`, the doc of `pause` | The duplex stream is in memory. With no work in the runtime, the paused clock jumps to the next timer. Decision 13 now gives this reason. |
| The jitter of the exponential builder of `backon` | docs.rs, backon 1.6.0, `src/backoff/exponential.rs` | Does not fit. The jitter adds `current_delay * rng.f32()`, from 0 to 100%, at each attempt. Decision 5 now names a function of its own. |
| `otel.status_code` on a span that is already open | tracing-opentelemetry, `src/layer.rs`, `SpanAttributeVisitor` | Holds for a field that the span declared. Decision 12 now declares the fields as `Empty`. |
| A dev-dependency cycle between two members | The Cargo book, resolver, "dev-dependency cycles" | Holds. Unit tests of the library link two copies with types that do not match, as decision 13 says. |

## Risks / Trade-offs

- [The quiet second is a guess. The spike waited 2.5 s before its login, and nobody tested a real game with 1 s.] → The values are constants of `Quiet`. The owner's first evening is the test. A change of the constant changes only the numbers of the spec scenarios.
- [A player cannot send for up to 10 s after a connect, while the login waits.] → The state `connecting` tells the client why. A world with no login connects at once.
- [`wait_idle` can now wait 10 s for a noisy game.] → It runs only in tests, on the paused clock.
- [A wrong password reconnects again and again.] → Each short connection is a failed attempt. The default cap of 30 stops it.
- [A new state in `ListWorlds` breaks a client that refuses unknown values.] → The only client in the repository reads the id only. Protobuf keeps an unknown enum value as a number.
- [The move of `fake.rs` touches many tests.] → The move is its own task, with no change in behavior, before any new code.

## Migration Plan

No data migrates. A configuration that worked before still resolves, because each new field has a default. A world that connected through `--fake-world` behaves as before.

Rollback is a revert of the change. Before the revert, remove `autoconnect`, `keepalive.mode` and the table `reconnect` from each configuration file. The old binary refuses them as unknown fields and does not start.

## Open Questions

- The exact text of the disconnect event after the last attempt, beyond the count that the spec asks for. It changes no scenario.
