# Proposal

## Why

The `mu-core` tests show that something did not happen by calling `settle`, which sleeps on the paused tokio clock until every task is idle. Such a test rests on an idle runtime, not on a state of the engine that the test can observe. The `review-lint` gate `time-based-test` reports each sleep that a diff adds to a test. The portal-skeleton change had to accept that finding on `settle`. Issue #66 tracks the change.

## What Changes

- The engine gets a control frame: a request on the queue of a world. It is not a command, and the journal never records it. The session answers it after it handles every request that came before it.
- When the world is up, the frame also makes a round trip through the game with the telnet option TIMING-MARK (RFC 860). The answer then also covers each line that the game sent before the mark.
- The task of the server handlers publishes the sequence number of the last event it handled.
- `Portal` gets a method that sends rounds of frames to every world and to the server handlers. It returns after a round that adds no event to the journal.
- The fake game answers `IAC DO TIMING-MARK` with `IAC WILL TIMING-MARK`, and the line framer reads that answer as the end of a frame, not as data.
- The test helper `settle` is deleted. Each caller waits for the new method.

## Not in this change

- An API call or a `mu` verb for the frame. Only the engine and its tests use it.
- TIMING-MARK for the production telnet engine and the world-type profiles. The telnet-engine change decides how real games answer it.
- The test cleanups of issues #67, #68 and #69.
- A change to the `time-based-test` gate of `review-lint`.
- The sleeps in `mu-client` and `mu-cli`, in production code and in one unit test of `mu-client`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `world-sessions`: three new requirements: the control frame, the answer to a waiting frame on a drop, and the wait of the engine. The fake world answers TIMING-MARK.

## Impact

- `crates/mu-core/src/session.rs`: a frame request beside the command request, and the answer of a frame after the game's mark.
- `crates/mu-core/src/telnet.rs`: a new crate-private module with the codes of the mark.
- `crates/mu-core/src/framer.rs`: the line framer encodes the mark and reads the answer. `Framer::mark` has a default body, so a framer outside the crate still compiles.
- `crates/mu-core/src/portal.rs`: the wait method, and the progress of the server handlers.
- `crates/mu-core/src/fake.rs`: the fake game answers the mark.
- `crates/mu-core/tests/support/mod.rs`, `portal.rs`, `scripts.rs`: `settle` goes.
- `crates/mu-cli/tests/source_rules.rs`: a new test that reads each Rust file of `mu-core` as tokens. It finds a code of telnet outside `telnet.rs` and a sleep. It sits beside `workspace_boundary.rs` and uses `support::files_under`.
- `crates/mu-cli/Cargo.toml`: the dev-dependency `proc-macro2` gets the feature `span-locations`, which gives the line of a hit. The test also uses `syn`, which `mu-cli` already has.
- No new crate, and `mu-core` gets no new dependency.
