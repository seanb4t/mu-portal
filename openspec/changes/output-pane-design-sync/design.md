# Design

## Context

The archived design of `output-pane` is in `openspec/changes/archive/2026-11-20-output-pane/design.md`. Its parts D2 and D8 and its Risks describe the skeleton and the width probe. The review rows of #112 name each place where the code at `main` differs from that text. See proposal.md for the list.

The probe today is in `crates/mu-tui/src/probe.rs`. It sets VMIN 0 and VTIME 1 on the terminal, writes every query in one write, and reads until DA1 or the deadline. A hand parser finds the CSI replies and the APC reply. Each other byte goes back to `run.rs`, which feeds it to a `termina::Parser` and gives the events to `update` before the first frame. `run.rs` then calls `Model::set_caps`, `Model::await_late_replies` and `Model::notice` itself.

## Goals / Non-Goals

**Goals:**

- One rule for the model: only `update` changes it.
- One parser of CSI input: Termina's.
- A probe that keeps its bound, and that a stray key or a broken sequence cannot stall.
- Design text and decision pages that match the code.

**Non-Goals:**

- Any change to what the user sees. Each scenario of `terminal-client` and `output-pane` keeps its test and its result.
- A check of the values in a reply. See proposal.md, "Not in this change".

## Decisions

### D1. The probe result is a `Msg`

`Msg` gets the variant `Probe(probe::Outcome)`. `run.rs` gives it to `update` first, then the early input events, then it draws the first frame. On `Msg::Probe`, `update` sets the caps of every pane from the outcome. If the outcome is not complete, it waits for late replies. If the outcome is not measured, it adds the width notice. These are the three calls that `run.rs` makes today.

The flag for mode 2027 at exit stays in `run.rs`. It is state of the terminal for the panic hook, not state of the model.

Alternative: keep the calls in `run.rs` and change the model docs to allow them. That leaves two places that change the model. The decision page "Terminal client architecture" already says that a reply to a terminal query is a `Msg`.

Enforced by: a unit test of `update` for each of the three effects of `Msg::Probe`, and review. No test can prove that `run.rs` calls no setter of the model.

### D2. The probe keeps its bound with a read that cannot end past the deadline

A read with VTIME 1 can block for one tick of 100 milliseconds. The loop starts a read only while the time now plus one tick is not past the deadline. So the last read ends by the deadline, and a silent terminal costs at most 500 milliseconds. The probe can stop up to one tick early. The width rule needs DA1. So a reply in that last tick is a late reply, as a reply after the deadline is today.

The read loop takes the read and the clock as arguments. The probe passes the terminal read and `Instant::now`. A unit test passes a fake read that adds one tick to a fake clock and returns no bytes.

Alternatives:

- `poll` with the time left, then a read that does not block. This needs the `event` feature of `rustix` and a second way to wait. VTIME already gives the wait.
- VTIME 0 in a loop. It spins the processor for up to 500 milliseconds.

Enforced by: the unit test with the fake clock. It asserts that the fake time at the return is not past the bound.

### D3. Termina parses the CSI replies, and the probe parses only the APC reply

The probe splits its input in two. An APC, from ESC `_` to ESC `\`, goes to the parser of the kitty graphics reply in the probe. Every other byte goes to a `termina::Parser`. From the events of that parser, the probe takes three kinds as replies:

- `Csi::Cursor(ActivePositionReport)`
- `Csi::Mode(ReportDecPrivateMode)`
- `Csi::Device(DeviceAttributes)`

Each other event is early input. `probe::run` returns the outcome and the early events, so `run.rs` makes no parser of its own. The hand CSI parser and the public `probe::Reply` go away. A private type in the probe holds the replies for `Outcome::learn`.

Termina gives the APC reply as keys, so the probe must keep its APC parser. `update` already matches the DECRPM and DA1 events of Termina for the late replies.

Alternative: keep the hand parser. It repeats a parser that the client already depends on, against the rule to reuse open source first.

Enforced by: the unit tests of the probe, which feed bytes and assert the outcome and the early events, and the pseudo-terminal scenarios of `terminal-client`.

### D4. An ESC ends an open sequence

In the input of the probe, an ESC that comes before the end of an open CSI or APC ends that sequence. The bytes before the ESC go to the early input as they are, and the parse starts again at the new ESC. Thus:

- Alt+`[` gives ESC `[`. A reply that follows it starts with ESC, so the reply still counts, and Alt+`[` reaches the input line (QUAL-3).
- An ESC `_` with no end cannot hide the DA1 reply after it, because DA1 starts with ESC (SEC-6).

Task 2.1 reads how `termina::Parser` treats an ESC inside an open CSI. It also reads what the flag for more input does with a sequence that a read splits. If Termina already ends the sequence at the ESC, the probe relies on it. If not, the split of D3 cuts the input at that ESC before Termina sees it.

Enforced by: a unit test with Alt+`[` before a cursor reply, and a unit test with an open ESC `_` before DA1. With the fake clock of D2, the second test asserts that the probe returns at DA1, not at the deadline.

### D5. The read error wins over the restore error

If the read fails and the restore of the terminal mode also fails, the probe returns an error of the kind of the read error. Its message holds both errors. If only one fails, the probe returns that one. A small function takes both results and gives the one result, so a unit test covers it with no terminal.

Enforced by: the unit test of that function.

### D6. `Caps` holds what the pane reads

`Caps` holds the width profile and the graphics flag. The cell size goes, because nothing in `src` reads it. The example `image-pane` takes the cell size from `screen::cell` itself. When #13 sizes an image from the cell, it adds the field back with a reader.

Enforced by: the compiler. A field that nothing reads gives a dead code warning, and the lints deny warnings.

### D7. The test helper has one wait on the screen

`wait_rows` calls `wait_screen` with a check on the rows of the screen. Both keep their names and their failure note.

Enforced by: the client tests, which pass with no edit.

### D8. The snapshot test also asserts its cells

`first_view_snapshot` asserts what the scenario "The first view" names. The top row starts with `[alpha/aria]` and then `beta/bryn`, and the rows between show `Welcome, aria.`. The snapshot stays.

Enforced by: the test.

### D9. What the archived design text gets wrong, and what is right

| Row | Archived text | What holds |
|---|---|---|
| R-87 | `ScrollPage(Up\|Down)` | `ScrollPage(i32)`: a signed count of pages |
| R-89 | With no DA1 in time, an image line shows its URL | A kitty reply turns graphics on, with or without DA1. The reply proves support. Only the width profile needs DA1. |
| R-90 | The probe opens `/dev/tty` and polls it | If stdin is a terminal, the probe reads stdin. If not, it opens `/dev/tty`. Termina does the same. It waits with VMIN 0 and VTIME 1 (D2). |
| R-92 | The proposal names no new crate for the probe | `mu-tui` uses `rustix` with the `termios` feature for the timed read. It is in the `mu-tui` row of `ALLOWED_CRATES`. |

The decision page `character-width.md` gets these facts and D2 to D4. The page `terminal-client-architecture.md` gets D1.

### What enforces each rule

| Rule | Enforced by |
|---|---|
| Only `update` changes the model | the unit tests of D1, and review |
| The probe waits at most 500 milliseconds | the unit test with the fake clock (D2) |
| A stray ESC or an open sequence does not hide a reply | the unit tests of D4 |
| The read error is not lost | the unit test of D5 |
| No test waits on a clock | review. The probe tests use the fake clock. |
| Each scenario keeps its test | the full test run, and the implementation review |

### Facts about third-party tools

Checked in the source of `termina` at the version in `Cargo.lock`:

- A CSI with numbers that ends in `R` gives `Csi::Cursor(ActivePositionReport { line, col })`.
- A CSI that starts with `?` and ends in `y` gives `Csi::Mode(ReportDecPrivateMode)`.
- A CSI that starts with `?` and ends in `c` gives `Csi::Device(DeviceAttributes)`.

Not checked yet. Task 2.1 reads the source and the docs before the work depends on them:

- How `termina::Parser` treats an ESC inside an open CSI.
- What its flag for more input does with a sequence that a read splits.

## Risks / Trade-offs

- [A legacy key such as Shift+F3 can send `CSI 1;2R`. Termina gives it as a cursor report.] → During the probe, it adds a cursor reply. Then the count of cursor replies differs from the count of graphemes, so every class keeps the `unicode-width` value and the notice shows. The hand parser does the same today.
- [The probe can stop up to one tick before the deadline.] → A reply in that tick is a late reply. `update` handles it as one.
- [The change touches `run.rs`, and #121 also touches it (QUAL-16).] → The edits are in different lines. The second pull request to merge rebases on the first.
