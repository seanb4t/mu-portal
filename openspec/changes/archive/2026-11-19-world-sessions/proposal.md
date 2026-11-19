# Proposal

## Why

The server can connect only a fake world. Each configured world stays `disconnected`, and `Connect` on it fails, because no transport reaches a real game. The owner cannot play TinyMUX, AresMUSH or Evennia until a world connects over TCP, logs in, stays connected and comes back after a drop. Issue #54 tracks this change, and it is step 5 of the Phase 1 plan in #61. It runs beside #55 (output-pane), as the owner decided on 2026-11-18.

## What Changes

- A TCP transport for a configured world. It connects to the host and the port of the world, with a connect timeout, and it resolves the name again at each attempt. A world with `tls = true` fails `Connect` with `failed_precondition` and a message that names the world, until #58 lands.
- A configured world uses the telnet framer with its character set, its window size and the option policy of a MU client. The fake world keeps the line framer.
- The secret of a world is read through the secret store at each connect, never at the start of the server. A missing secret fails that connect and names the variable.
- A login trigger decides the moment of the login line. The default waits until the game sends nothing for a short time after the connect, with a cap. The fake world logs in at once. The trigger is a seam, so a trigger that matches a banner line can come later. Until the login line goes out, the world stays `connecting`, and a send to it fails.
- A keepalive with three modes. The default `idle` sends the command after the set seconds with no write to the game. `interval` sends it every set seconds. `off` sends nothing. The journal does not record a keepalive line, and a metric counts it.
- Automatic reconnect after a drop that the player did not ask for. The wait doubles from `initial` up to `max`, with a jitter that grows with each attempt up to 20% of the wait. The world stops after `attempts` failed tries in a row, 30 by default. A connection that does not stay up for `max` seconds counts as a failed try. `mode = "off"` turns reconnect off. A `Disconnect` from the player, and a world with `tls = true`, never reconnect.
- A new world state `reconnecting`, with the time of the next attempt, in the engine and in `ListWorlds`. **BREAKING** for a client that treats an unknown state as an error. The current client shows the id only.
- A world with `autoconnect = true` connects at the start of the server. The default is `false`. The fake world always connects at the start, as today.
- New world configuration fields: `autoconnect`, `keepalive.mode` and the table `reconnect` (`mode`, `initial`, `max`, `attempts`). With `keepalive.mode = "off"`, a world needs neither `keepalive.command` nor `keepalive.seconds`. A world with neither of them has no keepalive. A built-in type sets none of the new fields, and a shared plugin cannot set `autoconnect`.
- The spans of a world show its failures (#95). A `world.connection` span that ends on a failed connect or a lost connection has the status `error` and carries the reason. A `command` span whose command fails has the status `error`.
- A new crate, `mu-fake-server`, holds the fake games. The in-memory fake game moves into it from `mu-core`. A new fake game listens on TCP and speaks telnet in raw bytes, not through the framer of the engine. It replays recorded transcripts, and it drops or goes silent on command. The transcripts stay where the telnet-engine change put them, in `crates/mu-core/tests/fixtures/telnet`.

## Not in this change

- TLS. #58 builds it.
- A login trigger that matches a banner line, and any detection of a refused login. The owner dropped refusal detection.
- Tests against real games in Docker. #82 keeps that gap open.
- The logging framework (#94) and the OTLP logs signal (#92).
- Client commands such as `/connect`, the statusline and any change to the terminal client beyond what a new state needs. #101 builds them.
- Lua or plugin timers that replace the keepalive. `keepalive.mode = "off"` leaves the room for them.
- Reload of the configuration while the server runs.

## Capabilities

### New Capabilities

- `world-connection`: the connection of a configured world to its game. It covers the transport, the connect timeout, the read of the secret, the login trigger, the keepalive and reconnect.

### Modified Capabilities

- `world-sessions`: a configured world can connect. `ListWorlds` gains the state `reconnecting`. At the start, the server connects the fake worlds and each world with `autoconnect = true`. The login of a world waits for its login trigger.
- `world-config`: the fields `autoconnect`, `keepalive.mode` and `reconnect`, with their defaults and ranges. The rule for `keepalive` changes for the mode `off`. Built-in types and shared plugins get limits on the new fields. The server reads a secret at the start only for a world with `autoconnect = true`.
- `telemetry`: the `world.connection` and `command` spans carry the status `error` on a failure, and the gauge `mu.world.state` knows the state `reconnecting`.

## Impact

- Code: `mu-core` gains the TCP transport, the login trigger, the keepalive and the reconnect policy in the session, and the state `reconnecting`. `fake.rs` leaves `mu-core`. `mu-cli` builds a transport and a telnet framer for each configured world. `mu-server` connects the worlds with `autoconnect` at the start and maps the new state.
- API: `portal.proto` gains `WORLD_STATE_RECONNECTING` and a time of the next attempt on `World`.
- New crate: `mu-fake-server`. The allowlist in `crates/mu-cli/tests/workspace_boundary.rs` and the LikeC4 model in `architecture/` gain it.
- Dependencies: no backoff crate fits the growing jitter, so the policy is a function of its own. `fastrand` gives its random draw, and enters `mu-core` through `[workspace.dependencies]`. The design names the check.
- Issues: closes #54 and #95. Leaves #82 open.
- Docs: the user page of the world configuration describes the new fields.
