//! Spike: crossterm or Termina as the backend of the terminal client.
//!
//! Each binary maps the events of its backend to [`Seen`] and calls [`run`]. Both send the same
//! query bytes and walk the same three checks, so the reports differ only by the backend.

use std::fmt::Write as _;
use std::io::{self, Write};
use std::time::Duration;

use futures::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep_until};

/// One event of a backend, reduced to what the checks read.
#[derive(Debug, Clone)]
pub enum Seen {
    Key { code: String, shift: bool, alt: bool, ctrl: bool },
    Paste(String),
    Reply(String),
    Other(String),
}

/// Turns on bracketed paste and the first kitty keyboard flag (disambiguate).
pub const MODES_ON: &[u8] = b"\x1b[?2004h\x1b[>1u";
/// Pops the kitty keyboard flag and turns off bracketed paste.
pub const MODES_OFF: &[u8] = b"\x1b[<u\x1b[?2004l";

/// The probe of the character-width decision: XTVERSION, DECRQM 2027, one wide grapheme with a
/// cursor position query, and DA1 last.
pub const PROBE: &[u8] = b"\x1b[>q\x1b[?2027$p\r\xe6\x97\xa5\x1b[6n\r\x1b[K\x1b[c";

const CHECK_1_WAIT: Duration = Duration::from_secs(3);
const CHECK_2_WAIT: Duration = Duration::from_secs(30);

/// Runs the three checks and returns the report.
pub async fn run<S>(backend: &str, mut events: S) -> io::Result<String>
where
    S: Stream<Item = io::Result<Seen>> + Unpin,
{
    let (tick_tx, mut ticks) = mpsc::channel::<()>(16);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(50));
        loop {
            interval.tick().await;
            if tick_tx.send(()).await.is_err() {
                break;
            }
        }
    });
    let mut tick_count = 0u64;
    let mut event_count = 0u64;
    let mut out = io::stdout();
    let mut report = String::new();
    writeln!(report, "backend\t{backend}").unwrap();

    say(&mut out, "Check 1: press g, then x at once.")?;
    wait_for(&mut events, &mut ticks, &mut tick_count, &mut event_count, None, |s| {
        matches!(s, Seen::Key { code, .. } if code == "g")
    })
    .await?;
    out.write_all(PROBE)?;
    out.flush()?;
    let mut replies = Vec::new();
    let mut stray = Vec::new();
    let mut x_key = false;
    let mut da1 = false;
    let deadline = Instant::now() + CHECK_1_WAIT;
    while !(x_key && da1) {
        match next(&mut events, &mut ticks, &mut tick_count, &mut event_count, Some(deadline)).await? {
            None => break,
            Some(Seen::Key { code, .. }) if code == "x" => x_key = true,
            Some(Seen::Reply(r)) => {
                da1 |= r.contains("DeviceAttributes");
                replies.push(r);
            }
            Some(other) => stray.push(format!("{other:?}")),
        }
    }
    writeln!(report, "check1.key_x_arrived\t{x_key}").unwrap();
    writeln!(report, "check1.replies_as_events\t{}", replies.len()).unwrap();
    for (name, needle) in [
        ("xtversion", "Dcs"),
        ("decrpm_2027", "ReportDecPrivateMode"),
        ("cursor_position", "ActivePositionReport"),
        ("da1", "DeviceAttributes"),
    ] {
        let seen = replies.iter().any(|r| r.contains(needle));
        writeln!(report, "check1.{name}_as_event\t{seen}").unwrap();
    }
    for r in &replies {
        writeln!(report, "check1.reply\t{r}").unwrap();
    }
    writeln!(report, "check1.stray_events\t{}", stray.len()).unwrap();
    for s in &stray {
        writeln!(report, "check1.stray\t{s}").unwrap();
    }

    say(&mut out, "Check 2: press Shift+Enter.")?;
    let deadline = Instant::now() + CHECK_2_WAIT;
    let enter = wait_for(&mut events, &mut ticks, &mut tick_count, &mut event_count, Some(deadline), |s| {
        matches!(s, Seen::Key { code, .. } if code == "Enter")
    })
    .await?;
    let shift_enter = matches!(enter, Some(Seen::Key { shift: true, .. }));
    writeln!(report, "check2.enter\t{enter:?}").unwrap();
    writeln!(report, "check2.shift_enter\t{shift_enter}").unwrap();

    say(&mut out, "Check 2: paste two or more lines.")?;
    let deadline = Instant::now() + CHECK_2_WAIT;
    let paste = wait_for(&mut events, &mut ticks, &mut tick_count, &mut event_count, Some(deadline), |s| {
        matches!(s, Seen::Paste(_))
    })
    .await?;
    let lines = match &paste {
        Some(Seen::Paste(text)) => text.split(['\r', '\n']).filter(|l| !l.is_empty()).count(),
        _ => 0,
    };
    writeln!(report, "check2.paste_lines_in_one_event\t{lines}").unwrap();

    writeln!(report, "check3.ticks\t{tick_count}").unwrap();
    writeln!(report, "check3.events\t{event_count}").unwrap();
    let check1 = x_key && da1 && stray.is_empty() && replies.len() >= 4;
    let check2 = shift_enter && lines >= 2;
    let check3 = tick_count > 0 && event_count > 0;
    writeln!(report, "pass.check1\t{check1}").unwrap();
    writeln!(report, "pass.check2\t{check2}").unwrap();
    writeln!(report, "pass.check3\t{check3}").unwrap();
    Ok(report)
}

fn say(out: &mut impl Write, text: &str) -> io::Result<()> {
    write!(out, "\r\n{text}\r\n")?;
    out.flush()
}

async fn wait_for<S>(
    events: &mut S,
    ticks: &mut mpsc::Receiver<()>,
    tick_count: &mut u64,
    event_count: &mut u64,
    deadline: Option<Instant>,
    want: impl Fn(&Seen) -> bool,
) -> io::Result<Option<Seen>>
where
    S: Stream<Item = io::Result<Seen>> + Unpin,
{
    while let Some(seen) = next(events, ticks, tick_count, event_count, deadline).await? {
        if want(&seen) {
            return Ok(Some(seen));
        }
    }
    Ok(None)
}

/// Reads the next event while the tick stream runs beside it, which is check 3.
async fn next<S>(
    events: &mut S,
    ticks: &mut mpsc::Receiver<()>,
    tick_count: &mut u64,
    event_count: &mut u64,
    deadline: Option<Instant>,
) -> io::Result<Option<Seen>>
where
    S: Stream<Item = io::Result<Seen>> + Unpin,
{
    let deadline = deadline.unwrap_or_else(|| Instant::now() + Duration::from_secs(3600));
    loop {
        tokio::select! {
            event = events.next() => {
                *event_count += 1;
                return event.transpose();
            }
            Some(()) = ticks.recv() => *tick_count += 1,
            () = sleep_until(deadline) => return Ok(None),
        }
    }
}

/// Writes the report to `--out FILE` when the arguments name one, and to stdout always.
pub fn finish(report: &str) -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(path) = args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)) {
        std::fs::write(path, report)?;
    }
    print!("{report}");
    Ok(())
}
