# Proposal

## Why

The implementation review of `output-pane` (issue #112) and its suite review (#113) found places where the code and the archived design differ. In some places the code is right and the design text is stale. In other places the code falls short of the design. The probe can wait longer than its bound. The probe changes the model outside `update`. The probe parses by hand the replies that Termina already parses. The archive keeps the old design as history, so this change records the corrected design and brings the code to it.

The change is tracked by issue #112.

## What Changes

- The probe result enters the model as a `Msg`, so only `update` changes the model (R-88).
- The probe keeps its wait inside its bound of 500 milliseconds (R-91).
- The probe frames its input itself, by the rules of ECMA-48, and gives each complete CSI to `termina::Parser` to decode (R-93). It decodes only the kitty graphics reply (APC) itself. The hand CSI decoder goes away, and the reply type becomes `probe::Answer` (QUAL-22).
- The probe reads past an Alt+`[` that the user presses during the probe (QUAL-3).
- An APC with no end no longer stalls the probe (SEC-6, the stall part).
- An error and the restore after it can both fail, in the probe, in the event loop and in `screen::open`. Then the result keeps the first error, with the restore error in its message (QUAL-12).
- `Caps` stops repeating the probe outcome. It drops the cell size, which nothing in `src` reads (QUAL-21).
- The pseudo-terminal helper has one wait on the screen, and `wait_rows` uses it (R-74).
- The snapshot test of the first view also asserts the cells that its scenario names (R-86).
- The design records what the code does:
  - the action `ScrollPage(i32)` (R-87)
  - graphics from a kitty reply, with or without DA1 (R-89)
  - the read from stdin or `/dev/tty`, with VMIN 0 and VTIME 1 (R-90)
  - `rustix` with the `termios` feature in `mu-tui` (R-92)
- The decision pages `decisions/character-width.md` and `decisions/terminal-client-architecture.md` follow the corrected design.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. No requirement changes, so `.openspec.yaml` sets `skip_specs: true`. Each scenario of `terminal-client` and `output-pane` keeps its test.

## Impact

- Code: `crates/mu-tui/src/probe.rs`, `run.rs`, `update.rs`, `model.rs`, `pane.rs`, `screen.rs` and `lib.rs`, and the example `crates/mu-tui/examples/image-pane.rs`.
- Tests: `crates/mu-tui/tests/probe.rs`, `update.rs`, `pane.rs`, `screen.rs` and `app.rs`, and `crates/mu-cli/tests/support/process.rs`.
- Dependencies: none new. `mu-tui` already uses `termina`, and `rustix` with `termios`.
- Docs: `decisions/character-width.md` and `decisions/terminal-client-architecture.md`.

Not in this change:

- A forged or garbled reply that skews widths or the graphics flag (the other part of SEC-6). Only the local terminal and the keys of the user reach the probe. No game text reaches it, so no trust boundary is crossed. A bound on the widths also changes the requirement "The client measures character widths at startup".
- The other items of #113. They are in #120, #121, #122, #123, #124 and #13.
- A kitty graphics reply that comes after the wait. It still reaches Termina as keys, and `update` drops it until the late DA1, as the archived design says.
