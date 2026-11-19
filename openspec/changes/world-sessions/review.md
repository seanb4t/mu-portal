---
verdict: approve-with-notes
phase: plan
reviewed_at: 2026-11-19T18:20:40+00:00
head: 42c490126b69c4fd643395340e27409ea483bf2d
round: 2
next_id: 48
lenses:
  spec-lint: ok
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
  verify: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-37 | test-coverage | should-fix | in-change | delta scenario 'A connection that is lost' has no matrix row | review > absent-row — "A connection that is lost" | add the row with its covering task or test |  |  | unmet-requirement |
| R-38 | test-coverage | should-fix | in-change | delta scenario 'A mode with no keepalive' has no matrix row | review > absent-row — "A mode with no keepalive" | add the row with its covering task or test |  |  | unmet-requirement |
| R-39 | test-coverage | should-fix | in-change | delta scenario 'A first wait equal to the cap' has no matrix row | review > absent-row — "A first wait equal to the cap" | add the row with its covering task or test |  |  | unmet-requirement |
| R-40 | test-coverage | should-fix | in-change | delta scenario 'A secret that is gone at a reconnect' has no matrix row | review > absent-row — "A secret that is gone at a reconnect" | add the row with its covering task or test |  |  | unmet-requirement |
| R-41 | test-coverage | should-fix | in-change | delta scenario 'A silent game' has no matrix row | review > absent-row — "A silent game" | add the row with its covering task or test |  |  | unmet-requirement |
| R-42 | test-coverage | should-fix | in-change | delta scenario 'A send during the quiet second' has no matrix row | review > absent-row — "A send during the quiet second" | add the row with its covering task or test |  |  | unmet-requirement |
| R-43 | test-coverage | should-fix | in-change | delta scenario 'No keepalive before the login' has no matrix row | review > absent-row — "No keepalive before the login" | add the row with its covering task or test |  |  | unmet-requirement |
| R-44 | test-coverage | should-fix | in-change | delta scenario 'A game that closes each connection after the login' has no matrix row | review > absent-row — "A game that closes each connection after the login" | add the row with its covering task or test |  |  | unmet-requirement |
| R-45 | test-coverage | should-fix | in-change | delta scenario 'No limit' has no matrix row | review > absent-row — "No limit" | add the row with its covering task or test |  |  | unmet-requirement |
| R-46 | test-coverage | should-fix | in-change | delta scenario 'A short connection keeps the count' has no matrix row | review > absent-row — "A short connection keeps the count" | add the row with its covering task or test |  |  | unmet-requirement |
| R-47 | test-coverage | should-fix | in-change | delta scenario 'A failed connect at the start' has no matrix row | review > absent-row — "A failed connect at the start" | add the row with its covering task or test |  |  | unmet-requirement |
| R-15 | verify | should-fix | in-change | REGRESSED: the fix 5b0f3ae026beba48cc7a36ed5c229139adb07a12 of R-15 did not hold. Connect in a wait must start the count again, but the scenario only asserts a connection within 1 second; an implementation that keeps the old attempt number passes it. | specs/world-connection/spec.md > Scenario: Connect during a wait — "the wait after that refused connect is at most 1.05 seconds" | fix it again | regression |  | unmet-requirement |

## Repeats

None.

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-31 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |
| R-32 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |
| R-33 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |
| R-34 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |
| R-35 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |
| R-36 | suggestion | deferred | Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. | https://github.com/seanb4t/mu-portal/issues/103 |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | a32c34daafa7a31407dcf7c92e0d67910d3a1522 | e439794dc29725eff48a8d3a420dd26c7e3a50aa73bbb92763cd879d5bf2abc8 | changes-requested | 36 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 42c490126b69c4fd643395340e27409ea483bf2d | 88f74e9b2b52e841a79fbefd78846432b739436477847f7668a3172514bc07d4 | approve-with-notes | 11 | 0 | 1 | 27 | 0 |  |

## Judge

judge: 2 questions, 0.000 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| The fake world answers | 2.2 | covered |  |
| The fake world answers a mark in order | 2.2 | covered |  |
| The flag is hidden | 2.2 | covered |  |
| A missing password variable | 5.1 | covered |  |
| A fake world does not reconnect | 6.2 | covered |  |
| Two characters on one game | 9.1 | covered |  |
| An unknown world | 9.1 | covered |  |
| A world that is not connected | 9.1 | covered |  |
| A write that fails | 9.1 | covered |  |
| A world that is still connecting | 9.1 | covered |  |
| A world that waits to reconnect | 5.4, 7.2, 8.2 | covered |  |
| An empty line | 9.1 | covered |  |
| A carriage return in the text | 9.1 | covered |  |
| A line break in the text | 9.1 | covered |  |
| Two worlds connect | 7.1 | covered |  |
| A configured world waits | 7.1 | covered |  |
| A configured world that connects at the start | 7.1 | covered |  |
| A configured world connects on request | 7.1 | covered |  |
| A wait outlasts the quiet before the login | 5.2 | covered |  |
| A wait on a world that waits to reconnect | 5.4 | covered |  |
| Each built-in type reads as quirks | 3.3 | partial |  |
| A global preference is not hidden by a type | 9.1 | covered |  |
| A user type with a built-in name | 9.1 | covered |  |
| An unknown field | 9.1 | covered |  |
| A port out of range | 9.1 | covered |  |
| A template with no password | 9.1 | covered |  |
| The case of a character set | 9.1 | covered |  |
| A character set that the engine cannot read | 9.1 | covered |  |
| An unknown mode | 3.1 | covered |  |
| The edges of each range | 3.1 | covered |  |
| The edges of the attempts | 3.1 | covered |  |
| A line break in a line field | 9.1 | covered |  |
| A plugin sets a field of the connection | 3.3 | covered |  |
| A plugin sets a preference | 9.1 | covered |  |
| A world with no host or port | 9.1 | covered |  |
| A login name with no secret | 9.1 | covered |  |
| A login secret with no name | 9.1 | covered |  |
| Half a keepalive | 3.2 | covered |  |
| Half a keepalive that is off | 3.2 | covered |  |
| A world that logs in by hand | 5.2, 9.1 | covered |  |
| The defaults | 3.1 | covered |  |
| A first wait over the cap | 3.1 | covered |  |
| A command under its call | 9.1 | covered |  |
| A command of the server | 9.1 | covered |  |
| A command that fails | 8.1 | covered |  |
| The connection of a fake world | 8.1 | covered |  |
| A connect that fails | 8.1 | covered |  |
| The game closes the connection | 6.2, 8.1 | covered |  |
| A command past the depth limit | 9.1 | covered |  |
| A command over the rate | 9.1 | covered |  |
| Commands at the rate | 9.1 | covered |  |
| A world that never connects | 8.2 | covered |  |
| The state at the stop | 8.2 | covered |  |
| The keepalive lines | 8.2 | covered |  |
| A world reaches its game | 6.1 | covered |  |
| Nothing listens on the port | 6.1 | covered |  |
| A connect that hangs | 5.4 | covered |  |
| A TLS world | 6.1 | covered |  |
| A Latin-1 game | 6.1 | covered |  |
| The window size of the world | 6.1 | covered |  |
| A missing variable | 6.1 | covered |  |
| The login line hides the password | 6.1 | covered |  |
| A banner and then quiet | 5.2 | covered |  |
| A game that never goes quiet | 5.2 | covered |  |
| Idle | 5.3 | covered |  |
| Interval | 5.3 | covered |  |
| Off | 5.3 | covered |  |
| The journal holds no keepalive | 5.3 | covered |  |
| The player disconnects | 6.2 | covered |  |
| Reconnect off | 6.2 | covered |  |
| The waits of a game that refuses every connect | 5.4 | covered |  |
| The extra time at its cap | 4.3 | covered |  |
| The last attempt | 5.4 | covered |  |
| A stable connection starts the count again | 5.4 | covered |  |
| Connect during a wait | 5.4 | covered |  |
| Disconnect during a wait | 5.4 | covered |  |

### Not run

Plan phase: no verification runs.

## Verification runs

None.

## Dropped

None.

## Lens notes

- verify: R-15 is REGRESSED: design decision 1 says a Connect in Waiting starts at once and sets the attempt to 1, and the requirement calls that connect an attempt. If it fails, the next attempt is attempt 2 and wait(2) = 2 * initial = 2 s. But the scenario 'Connect during a wait' now asks for at most 1.05 s, which is wait(1). Say in decision 1 and the requirement whether the connect of a Connect in a wait counts as attempt 1 or as a fresh first connect, and make the bound of the scenario match.
- verify: Decision 1 says the connect function 'reads the secret (decision 6)'. After the renumbering, decision 6 is the TCP transport and the secret is decision 7. Decision 7 names the connect function itself, so behaviour does not change.
- verify: Decision 7 gives SecretSource only Fixed and Store, with no case for a world with no login, and drops the Option of SessionParts.password. A configured world that logs in by hand needs Option<SecretSource> or a third case. crates/mu-core/tests/portal.rs:301 builds a world with a login and no password, and that state has no NoSecret variable to name.
- verify: Decision 2 says a missing secret at an automatic attempt ends the wait in Down before any span opens, but no row of the cause table covers that path, while Session::down takes a cause. The implementer must take this path outside down or add a cause that does not reconnect.
- verify: At head the stopped check sits in the Connect handler (crates/mu-core/src/session.rs:549). Decision 1 moves every connect into one function and does not say whether an automatic attempt checks the stopped flag. Portal::stop still ends each world through its Disconnect, so no world is up when stop returns.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","5. The TCP transport"],"It resolves the host and connects inside one `tokio::time::timeout` of 10 seconds"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The scenario A connect that hangs needs a 10-second timeout on a transport that never ends its connect on a paused clock, but decision 5 puts the timeout inside TcpTransport only, and the engine tests do not use TcpTransport. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-sessions/spec.md","Requirement: A wait of the engine covers a pending login"],"The session SHALL answer a control frame only after the login of its world went out."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | No decision makes a control frame wait for a pending login. Decision 1 covers frames in Waiting only, and the session at head writes the marks of waiting frames right after on_connect, which a deferred login would then precede. |
| R-3 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-connection/spec.md","Requirement: The login waits until the game is quiet"],"Before the login line, the world SHALL send the game only the replies of the telnet protocol."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | app-3: No decision says what a Send, a script command, a keepalive deadline or a mark does while the login is pending. At head Send writes whenever the connection is up, so the rule that only telnet replies go out before the login has no producing decision.; inv-4: The session is Connected and accepts Send and Login as soon as the stream opens, so a player or script line can reach the game during the quiet wait; no decision or task guards writes while the login is pending, only control frames. |
| R-4 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","6. The secret is a source, read at each connect"],"A `Connect` reads the source before it changes the state."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | app-4: Decision 6 reads the secret only on a Connect request. An automatic attempt from Waiting has no caller, and no decision says whether it reads the secret or what a failed read does, though the spec reads it at each connect and forbids a reconnect after a failed read.; inv-10: The secret is read only on a Connect request, before a state change; a reconnect attempt from Waiting starts in progress with no Connect, so no decision says it reads the secret or what a failed read does there, though the spec forbids a reconnect after it. |
| R-5 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","9. The configuration fields"],"`keepalive` as an enum: `Idle(Keepalive)`, `Interval(Keepalive)` or `Off`"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  | severity? should-fix 0.61 | app-5: The keepalive enum of decision 9 has no case for the default mode idle with no command and no seconds, the case of aresmush and of an untyped world. Such a world cannot hold mode idle, so mu world show cannot print mode = idle with the source default.; inv-2: With the default keepalive.mode = idle, a world whose layers set neither keepalive half (every aresmush world) has no value of the enum Idle(Keepalive) \| Interval(Keepalive) \| Off; no rule says it resolves to Off or fails, and an explicit mode = interval with no command is silently accepted. |
| R-6 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/world-connection/spec.md","Requirement: A world stops after its last attempt"],"`reconnect.attempts = 0` SHALL mean no limit."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | app-6: No decision produces reconnect.attempts = 0 as no limit, nor the stop after the last attempt with its disconnect event. Decisions 3 and 4 name the attempts field and the wait only.; inv-7: reconnect.attempts = 0 means no limit, but only the config edge test passes 0; no engine test runs a world with attempts = 0, so a check that stops at once on 0 passes every planned test. |
| R-7 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["design.md","11. Telemetry (#95)"],"Each attempt is a new `Conn::Connecting`, so each attempt already opens its own `world.connection` span."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | Decision 11 says each attempt already opens its own world.connection span because it is a new Conn::Connecting. At head the span is made in the Connect command handler, not by Conn::Connecting, so an attempt that the Waiting timer starts gets no span from that code. |
| R-8 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","Migration Plan"],"Rollback is a revert of the change."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The rollback is a revert only. A configuration that sets autoconnect, keepalive.mode or reconnect then fails as an unknown field, and the server does not start. The plan has no step for that configuration. |
| R-9 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","6. The secret is a source, read at each connect"],"A `Connect` reads the source before it changes the state. A failed read returns a new `PortalError::NoSecret` that names the variable, and records nothing."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  | severity? blocker 0.61 | Decision 6 reads the secret of a configured world at each connect, but names neither load_password nor its checks, so a read through SecretStore::get skips the mask and line-break guards that the fake world gets. |
| R-10 | 1 | 2 | should-fix | deviates-design | drift | ["artifact",["design.md","5. The TCP transport"],"A world with `tls = true` gets no transport. Its `Connect` fails with a new `PortalError::TlsNotSupported`"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  | scope? in-change 0.95 | TlsNotSupported is a second error for a world with no transport. After the change the TLS world is the only world without a transport, so NoTransport and its text 'only fake worlds can connect' stay beside it and say something false. |
| R-11 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","6. The secret is a source, read at each connect"],"A failed read returns a new `PortalError::NoSecret` that names the variable, and records nothing."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | NoSecret is a second error for a login that has no secret. Decision 6 does not say what happens to NoPassword and its two checks in the session, so two errors with two texts cover one failure. |
| R-12 | 1 | 2 | should-fix | deviates-design | drift | ["artifact",["design.md","9. The configuration fields"],"`reconnect` as an enum: `Backoff { initial, max, attempts }` or `Off`"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | Decision 9 adds keepalive and reconnect enums to ResolvedWorld whose cases repeat the kinds of KeepalivePolicy and ReconnectPolicy in decision 3, field for field. A new kind then needs an edit in both, and the map between them in WorldSetup. |
| R-13 | 1 | 2 | should-fix | deviates-design | drift | ["artifact",["design.md","2. Why a world goes down decides whether it reconnects"],"The same cause gives the value of `mu.connection.end`, and `Player` and `Script` both map to `player`."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  | scope? in-change 0.71 | The causes Player, Script and Stopped repeat Origin: a Disconnect already carries Origin::Player, Origin::Script or, from Portal::stop, Origin::System. A second enum for the origin of a disconnect can drift from Origin. |
| R-14 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Scenario: The edges of each range"],"the exit status is 0 for 1 and the top of the range"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The planned edge test sets reconnect.initial = 3600 alone, but the default reconnect.max = 60 and the rule initial <= max reject it, so the scenario's exit status 0 at the top edge cannot hold. |
| R-15 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-connection/spec.md","Requirement: Connect and Disconnect act on a world that waits"],"`Connect` SHALL start an attempt at once and start the count again."] | fixed | 886c13e26c0f562b4f013436302270a8bea0026a |  |  | Connect in a wait must start the count again, but the scenario only asserts a connection within 1 second; an implementation that keeps the old attempt number passes it. |
| R-16 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-connection/spec.md","Requirement: The login waits until the game is quiet"],"A quiet second is 1 second with no byte from the game, from the connect or from the last byte."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The quiet second counts from the connect when the game sends nothing, but the planned tests cover only a banner, a noisy game and the cap; the empty case (login 1 second after the connect) is untested. |
| R-17 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Scenario: A first wait over the cap"],"the game `alpha` sets `reconnect.initial = 120` and `reconnect.max = 60`"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The rule initial <= max has one planned test, 120 over 60; the equal side (initial = max) is never passed, so a check written as initial >= max passes. |
| R-18 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/telemetry/spec.md","Requirement: A span for each connection of a world"],"A span that ends with `connect_failed`, `timeout` or `lost` SHALL have the status `error`."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The status error is required for timeout and lost as well as connect_failed, but the span tests pass only connect_failed and closed, and the cause map test of task 4.3 covers the attribute, not the status; the lost side of the lost/closed edge is untested. |
| R-19 | 1 | 2 | should-fix | in-change | false-test | ["path","crates/mu-core/src/world/layer.rs",206] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The quirk schema reuses KeepaliveFields, so adding mode to that struct makes built-in types accept keepalive.mode; the only planned test reads the shipped types and passes whether or not the schema refuses mode, autoconnect or reconnect. |
| R-20 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","4. The strategies"],"Add unit tests for the next deadline after a connect, after a write and after a keepalive, for each kind."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The keepalive deadline starts at the connect, and keepalive.seconds can be 1 while the login waits up to 10 seconds, so a keepalive line can precede the login line; nothing orders the two deadlines. |
| R-21 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","4. The wait policy"],"`wait(k) = min(initial * 2^(k-1), max)`"] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The wait raises 2 to k-1 while k reaches 10000, or grows without end when attempts = 0; 2^(k-1) passes u32 at k = 33, 3600 * 2^(k-1) passes u64 at k = 54, and the property test stops at k = 50 with initial = 1, so an overflow panic in the session goes untested. |
| R-22 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-connection/spec.md","Requirement: A world stops after its last attempt"],"A connection that stays up for `reconnect.max` seconds SHALL start the count and the wait again from attempt 1."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The reset after a stable connection is tested only at exactly max seconds; no test keeps a connection up for max - 1 seconds and asserts the count does not reset, so a reset on every successful connect also passes. |
| R-23 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","Risks / Trade-offs"],"`reconnect.attempts = 30` by default sets a limit, and the wait grows to `max`."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The design relies on attempts = 30 to end the wrong-password loop, but the spec counts only failed attempts in a row, and a connect that opens and is closed after the login is not a failed attempt; nothing makes a short connection count, so that loop never stops. |
| R-24 | 1 | 2 | should-fix | in-change | next-change | ["artifact",["proposal.md","What Changes"],"**BREAKING** for a client that treats an unknown state as an error."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The proposal marks the new state reconnecting as BREAKING, but no delta spec carries a BREAKING marker. |
| R-25 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","A world has a place to connect"],"After the merge, each world SHALL have `host` and `port`, and `host` SHALL NOT be empty."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The requirement 'A world has a place to connect' has no task line that names its subject. |
| R-26 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Each field is known"],"Each layer SHALL set only the fields of a world."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The requirement 'Each field is known' has no task line with both of its distinctive words; the field list is only partly covered. |
| R-27 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/world-config/spec.md","Each value is in its range"],"The range of `port`, `columns` and `rows` SHALL be 1 to 65535."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | The requirement 'Each value is in its range' has no task line that names both 'value' and 'range'; the ranges appear only as a bare word. |
| R-28 | 1 | 1 | should-fix | in-change | false-test | ["artifact",["tasks.md","3. The configuration fields"],"Make the quirk schema of the built-in types refuse `keepalive.mode`, `autoconnect` and `reconnect`."] | duplicate | R-19 |  |  | Task 3.3 makes the quirk schema refuse the new fields, but its only check passes if the schema stays open to them. |
| R-29 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","8. Start-up connects"],"A `Connect` that fails at the start goes to the log at `WARN` and does not stop the server."] | resolved | resolved in round 2 from fixed 5b0f3ae026beba48cc7a36ed5c229139adb07a12 |  |  | Design 8 says a failed Connect at the start only logs and does not stop the server. No scenario and no task test it. |
| R-30 | 1 | 1 | should-fix | in-change | unmet-requirement | ["artifact",["specs/telemetry/spec.md","A span for each connection of a world"],"A span that ends with `connect_failed`, `timeout` or `lost` SHALL have the status `error`."] | duplicate | R-18 |  |  | The rule that timeout and lost set the span status error has a test for connect_failed only. Nothing checks the status for those causes. |
| R-31 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","5. The TCP transport"],"It sets `TCP_NODELAY`, because a MU sends short lines."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | TCP_NODELAY in decision 5 answers no requirement, non-goal or risk of the change. |
| R-32 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","12. The crate `mu-fake-server`"],"`mu-core \\u2192 mu-fake-server` goes there, and `mu-fake-server \\u2192 mu-core` and `mu-cli \\u2192 mu-fake-server` go into `ALLOWED_EDGES`."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | Decision 12 adds a crate but follows only half of the procedure of the workspace page: it names ALLOWED_EDGES and the LikeC4 model, not the ALLOWED_CRATES row of mu-fake-server or the crate table and the 'two allowlists' text of that page. |
| R-33 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","4. The wait policy"],"`backon` is already in `[workspace.dependencies]` for `mu-client`."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | Decision 4 reuses backon in mu-core but names only [workspace.dependencies]. backon is allowed for mu-client alone, and the workspace page asks for a row in the second allowlist for each crate that uses it. |
| R-34 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","12. The crate `mu-fake-server`"],"The TCP fake game listens on `127.0.0.1:0` and runs a script of steps: send bytes, wait for bytes, go silent, close."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | Decision 12 lists transcript replay as a new part beside the scripted fake game, but a replay is a script whose steps are sends of the recorded reads. The parameter is the list of steps. |
| R-35 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","3. Three strategies, chosen per world in `WorldSetup`"],"For example, `LoginMoment` gives the deadline of the login from the connect time and the time of the last byte."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | The LoginMoment seam sees only the connect time and the time of the last byte, so the banner trigger that the proposal names as the next kind cannot fit it without a change to the seam and to what the session gives it. |
| R-36 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","3. Three strategies, chosen per world in `WorldSetup`"],"`LoginMoment` gives the deadline of the login from the connect time and the time of the last byte."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/103; reason: Suggestions from the world-sessions plan review: refinements that do not block the plan, tracked together for the implementation phase. |  |  | The login deadline over any sequence of byte arrivals is a bound over an unbounded domain: never later than connect + 10 s and never earlier than min(last byte + 1 s, connect + 10 s). |
| R-37 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A connection that is lost"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A connection that is lost' has no matrix row |
| R-38 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A mode with no keepalive"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A mode with no keepalive' has no matrix row |
| R-39 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A first wait equal to the cap"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A first wait equal to the cap' has no matrix row |
| R-40 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A secret that is gone at a reconnect"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A secret that is gone at a reconnect' has no matrix row |
| R-41 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A silent game"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A silent game' has no matrix row |
| R-42 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A send during the quiet second"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A send during the quiet second' has no matrix row |
| R-43 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"No keepalive before the login"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'No keepalive before the login' has no matrix row |
| R-44 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A game that closes each connection after the login"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A game that closes each connection after the login' has no matrix row |
| R-45 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"No limit"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'No limit' has no matrix row |
| R-46 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A short connection keeps the count"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A short connection keeps the count' has no matrix row |
| R-47 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A failed connect at the start"] | refuted | Each scenario has a named test in tasks.md; the matrix lacks rows only because a follow-up round does not rebuild it. R-37: 8.1 lost_connection_span_is_error. R-38: 3.2 mode_with_no_keepalive. R-39: 3.1 first_wait_equal_to_the_cap. R-40: 5.5 secret_gone_at_reconnect_stops. R-41: 5.3 login_after_a_silent_second. R-42: 5.3 send_during_quiet_second_is_refused. R-43: 5.4 no_keepalive_before_login. R-44: 5.5 short_connections_count_as_attempts. R-45: 5.5 attempts_zero_never_stops. R-46: 5.5 short_connection_keeps_the_count. R-47: 7.1 failed_autoconnect_does_not_stop_start. |  |  | delta scenario 'A failed connect at the start' has no matrix row |
