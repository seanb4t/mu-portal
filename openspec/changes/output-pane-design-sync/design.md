# Design

## Context

The archived design of `output-pane` is in `openspec/changes/archive/2026-11-20-output-pane/design.md`. Its parts D2 and D8 and its Risks describe the skeleton and the width probe. The review rows of #112 name each place where the code at `main` differs from that text. See proposal.md for the list.

The probe today is in `crates/mu-tui/src/probe.rs`. It sets VMIN 0 and VTIME 1 on the terminal, writes every query in one write, and reads until DA1 or the deadline. A hand parser finds the CSI replies and the APC reply. Each other byte goes back to `run.rs`, which feeds it to a `termina::Parser`. `run.rs` then calls `Model::set_caps`, `Model::await_late_replies` and `Model::notice` itself. The event loop draws a frame before it takes each message, so today the first frame comes before the early input events.

The tests of the probe are in `crates/mu-tui/tests/probe.rs`. They use the public `probe::parse` and `probe::Reply`. The tests of late replies and notices are in `crates/mu-tui/tests/update.rs`. They call `await_late_replies` and `notice` from outside the crate.

## Goals / Non-Goals

**Goals:**

- One rule for the model: only `update` changes it.
- The probe decodes CSI replies with Termina, and frames the input itself.
- A probe that keeps its bound, and that a stray key or a broken sequence cannot stall.
- Design text and decision pages that match the code.

**Non-Goals:**

- Any change to what the user sees. Each scenario of `terminal-client` and `output-pane` keeps its test and its result.
- A check of the values in a reply. See proposal.md, "Not in this change".

## Decisions

### D1. The probe result is a `Msg`

`Msg` gets the variant `Probe(probe::Outcome)`. `run.rs` puts it first in the queue of early messages, before the early input events. On `Msg::Probe`, `update` does three things:

- It sets the caps of every pane from the outcome.
- If the outcome is not complete, it waits for late replies.
- If the outcome is not measured, it adds the width notice.

While an early message waits in the queue, the event loop draws no frame. So `Msg::Probe` and the early events come before the first frame, and the first frame has the caps.

`Model::set_caps`, `Model::await_late_replies` and `Model::notice` become private to the crate. The tests in `tests/update.rs` reach the late state and the notice through `Msg::Probe`.

The flag for mode 2027 at exit stays in `run.rs`. It is state of the terminal for the panic hook, not state of the model.

Alternative: keep the calls in `run.rs` and change the model docs to allow them. That leaves two places that change the model. The decision page "Terminal client architecture" already says that a reply to a terminal query is a `Msg`.

Enforced by: the tests of `Msg::Probe` in `tests/update.rs`, and the crate-private setters. Code in the crate can still call a setter, so review also checks `run.rs`.

### D2. The probe keeps its bound with a read that cannot end past the deadline

A read with VTIME 1 can block for one tick of 100 milliseconds. One constant holds the tick in tenths of a second. The VTIME value and the `Duration` of the tick both come from it.

The loop starts a read only while the time now plus one tick is not past the deadline. So the last read ends by the deadline, and a silent terminal costs at most 500 milliseconds. The probe can stop up to one tick early. The width rule needs DA1. So a reply in that last tick is a late reply, as a reply after the deadline is today.

The read loop is a public function that takes the read and the clock as arguments. The probe passes the terminal read and `Instant::now`. A test passes a fake read that moves a fake clock.

Alternatives:

- `poll` with the time left, then a read that does not block. This needs the `event` feature of `rustix` and a second way to wait. VTIME already gives the wait.
- VTIME 0 in a loop. It spins the processor for up to 500 milliseconds.

Enforced by: `silent_terminal_keeps_the_bound`. Its fake read returns no bytes and moves the fake clock one tick, plus 1 millisecond on the first read. The old condition then returns past the bound, so the test fails before the fix.

### D3. The probe frames the input, and Termina decodes each CSI

The probe cuts its input into frames by the rules of ECMA-48:

- A CSI is ESC `[`, then bytes from 0x20 to 0x3F, then one final byte from 0x40 to 0x7E.
- An APC is ESC `_`, then a body, then ESC `\`.
- An ESC inside an open CSI or APC ends that sequence as broken, and a new frame starts at that ESC. The one exception is the ESC of ESC `\`, which ends an APC.
- A byte outside 0x20 to 0x3F before the final byte also ends a CSI as broken.
- An ESC, or an open sequence, at the end of a read waits for the next read.

Each frame then goes to one place:

| Frame | Goes to |
|---|---|
| A complete CSI | A `termina::Parser`, alone, with no more input to come. Three event kinds are replies: `Csi::Cursor(ActivePositionReport)`, `Csi::Mode(ReportDecPrivateMode)` and `Csi::Device(DeviceAttributes)`. Each other event is early input. |
| An APC | The parser of the kitty graphics reply in the probe. Termina gives an APC as keys. An APC that is not the graphics reply is dropped. |
| A broken sequence | The probe makes the keys itself. ESC and the byte after it give that key with Alt, for example Alt+`[`. Each other byte gives its own key. |
| Other bytes | A `termina::Parser`, for keys, text and paste. |

At the end of the probe, an open sequence can still wait for more input. Its bytes then go to the early input as the keys of a broken sequence. Today the probe loses them.

`probe::run` returns the outcome and the early events, so `run.rs` makes no parser of its own. The hand CSI decoder goes away. The public type of a probe reply is renamed `probe::Answer`, so the crate has no second `Reply` beside `update::Reply` (QUAL-22). The framer is a public function, so `tests/probe.rs` drives it.

`Outcome` keeps the DECRPM value of mode 2027 as Termina's `DecModeSetting`. One function says whether a setting is set. `Outcome::resets_grapheme_mode` and the late DECRPM arm of `update` both call it.

Alternatives:

- Give every byte to Termina. Termina 0.3.3 loses a reply after a stray ESC `[` (see the checked facts).
- Keep the hand parser and only fix its framing. That keeps a CSI decoder that Termina already has, against the rule to reuse open source first.

Enforced by: the tests of `tests/probe.rs` that D10 names, and the pseudo-terminal scenarios of `terminal-client`.

### D4. The read error wins over the restore error

An error and the error of the restore after it can meet in three places:

- `probe::run`: the read, then the restore of the terminal mode.
- `run::run`: the event loop, then `screen::leave`.
- `screen::open`: `enter`, then `leave` after `enter` fails.

In each place, the result keeps the kind of the first error, and its message also holds the restore error. If only one fails, the result is that one. One public function in `screen` does this, and the three places call it.

The panic hook still drops the error of its restore. A panic hook has no caller to give an error to.

Enforced by: four tests of that function, one for each case: both fail, the read alone fails, the restore alone fails, and both succeed.

### D5. `Caps` comes from the outcome in one place

`Caps` holds the width profile and the graphics flag. The cell size goes, because nothing in `src` reads it. One conversion makes `Caps` from a `probe::Outcome`. `update` and the example `image-pane` both use it.

When #13 sizes an image from the cell, it adds the cell size to `Msg::Probe` and to `Caps`, from `screen::cell`.

Enforced by: review. `Caps` is public with public fields, so the compiler gives no warning for a field that nothing reads.

### D6. The test helper has one wait on the screen

`wait_rows` calls `wait_screen` with a check on the rows of the screen. Both keep their names and their failure note.

Enforced by: the client tests, which pass with no edit.

### D7. The snapshot test also asserts its cells

`first_view_snapshot` asserts what the scenario "The first view" names:

- The top row starts with `[alpha/aria]` and then `beta/bryn`.
- The rows between show `Welcome, aria.`.
- No row shows `Welcome, bryn.`, the line of `beta/bryn`.

The snapshot stays.

Enforced by: the test.

### D8. What the archived design text gets wrong, and what is right

| Row | Archived text | What holds |
|---|---|---|
| R-87 | `ScrollPage(Up\|Down)` | `ScrollPage(i32)`: a signed count of pages |
| R-89 | With no DA1 in time, an image line shows its URL | A kitty reply turns graphics on, with or without DA1. The reply proves support. Only the width profile needs DA1. |
| R-90 | The probe opens `/dev/tty` and polls it | If stdin is a terminal, the probe reads stdin. If not, it opens `/dev/tty`. Termina does the same. It waits with VMIN 0 and VTIME 1 (D2). |
| R-92 | The proposal names no new crate for the probe | `mu-tui` uses `rustix` with the `termios` feature for the timed read. It is in the `mu-tui` row of `ALLOWED_CRATES`. |

The decision page `character-width.md` gets these facts, and D2 to D4. The page `terminal-client-architecture.md` gets D1.

### D9. What enforces each rule

| Rule | Enforced by |
|---|---|
| Only `update` changes the model | the tests of `Msg::Probe`, the crate-private setters, and review |
| The probe waits at most 500 milliseconds | `silent_terminal_keeps_the_bound` |
| A stray ESC or an open sequence does not hide a reply | the framer tests of D10 |
| The read error is not lost | the four tests of D4 |
| No test waits on a clock | review. The probe tests use the fake clock. |
| Each scenario keeps its test | the full test run, and the implementation review |

### D10. The tests of the probe

All tests of the probe stay in `crates/mu-tui/tests/probe.rs`. The 14 tests there move from `parse` and `Reply` to the framer and `Answer`, with the same inputs and the same expected outcome. New tests:

| Test | What it proves |
|---|---|
| `replies_come_from_termina` | A cursor reply, a DECRPM reply and DA1 give their answers. |
| `keys_during_the_probe_are_early_events` | A key between replies is an early event. |
| `alt_bracket_before_a_reply` | ESC `[` then a cursor reply gives Alt+`[` and the reply (QUAL-3). |
| `question_csi_before_a_reply` | ESC `[` `?` then a cursor reply and DA1 gives every reply. |
| `open_apc_does_not_hide_da1` | An open ESC `_` then DA1 gives DA1 (SEC-6). With the fake clock, the probe returns at DA1, not at the deadline. |
| `apc_end_split_across_reads` | A graphics reply cut between its ESC and its `\` still turns graphics on. |
| `split_reply_across_reads` | A cursor reply cut in the middle gives one reply. |
| `silent_terminal_keeps_the_bound` | D2 |

### Facts about third-party tools

Checked in the source of `termina` at the version in `Cargo.lock`:

- A CSI with numbers that ends in `R` gives `Csi::Cursor(ActivePositionReport { line, col })`.
- A CSI that starts with `?` and ends in `y` gives `Csi::Mode(ReportDecPrivateMode)`.
- A CSI that starts with `?` and ends in `c` gives `Csi::Device(DeviceAttributes)`.

Checked by the plan review with a scratch crate on the same version (rows R-1, R-4 and R-15):

- The parser holds a bare ESC `[`. When the next ESC comes, it drops ESC `[` ESC, so the reply after it loses its ESC.
- The parser gives no event for a bare ESC `[`.
- After ESC `[` `?`, the parser reads up to the next `u`, `c`, `n` or `y`. A cursor reply in between is lost.

Not checked yet. Task 2.1 checks it before the work depends on it:

- One complete CSI, given alone to a new parser with no more input to come, gives exactly one event.

## Risks / Trade-offs

- [A legacy key such as Shift+F3 can send `CSI 1;2R`. Termina gives it as a cursor report.] → During the probe, it adds a cursor reply. Then the cursor replies outnumber the graphemes, so every class keeps the `unicode-width` value, and the notice shows. The hand parser does the same today.
- [The probe can stop up to one tick before the deadline.] → A reply in that tick is a late reply. `update` handles it as one.
- [The probe owns a small framer.] → It holds only the framing rules of ECMA-48. Termina still decodes each CSI.
- [The change touches `run.rs`, and #121 also touches it (QUAL-16).] → Task 6.1 checks the state of #121. If #121 merged first, the branch rebases on `main` before its pull request.
