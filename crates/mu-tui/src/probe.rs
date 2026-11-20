//! The startup probe: the queries, the parser of the replies, and what the replies give.
//!
//! The probe writes every query in one write. It asks for the state of mode 2027, sets the
//! mode, writes one grapheme of each width class with a cursor position query after each, asks
//! whether the terminal draws kitty images, and ends with DA1. Every terminal answers DA1, so
//! its reply ends the replies.
//!
//! The probe reads the replies itself, before the Termina event reader starts, because Termina
//! drops some replies and gives the kitty graphics reply as keys.

use std::fs::File;
use std::io::{self, IsTerminal, Write};
use std::os::fd::{AsFd, BorrowedFd};
use std::time::{Duration, Instant};

use rustix::termios::{OptionalActions, SpecialCodeIndex, tcgetattr, tcsetattr};

use crate::width::Profile;

/// One grapheme of each width class, in the order of the cursor position queries.
pub const GRAPHEMES: &[&str] = &[
    "a",
    "é",
    "漢",
    "①",
    "e\u{301}",
    "❤\u{FE0F}",
    "⌚\u{FE0E}",
    "👨\u{200D}👩\u{200D}👧",
    "🇺🇸",
    "👍🏽",
    "1\u{FE0F}\u{20E3}",
    "🫩",
    "\u{200B}",
];

/// The key of the id in the kitty graphics query and its reply.
const GRAPHICS_ID: &str = "i=31";

/// The kitty graphics query: one RGB pixel, sent directly, with the query action.
const GRAPHICS_QUERY: &str = "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\";

/// The DEC private mode that makes the terminal measure grapheme clusters.
pub const GRAPHEME_MODE: u16 = 2027;

/// The bytes of every query, in one write.
#[must_use]
pub fn queries() -> Vec<u8> {
    let mut out = String::from("\x1b[?2027$p\x1b[?2027h\x1b[H");
    for g in GRAPHEMES {
        out.push_str("\r\x1b[2K");
        out.push_str(g);
        out.push_str("\x1b[6n");
    }
    out.push_str("\r\x1b[2K");
    out.push_str(GRAPHICS_QUERY);
    out.push_str("\x1b[c");
    out.into_bytes()
}

/// One reply of the terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// A cursor position report, from 1.
    Cursor {
        /// The row.
        row: u16,
        /// The column.
        col: u16,
    },
    /// DECRPM: the state of a DEC private mode. 0 is not recognized, 1 set, 2 reset, 3
    /// permanently set and 4 permanently reset.
    Mode {
        /// The mode.
        mode: u16,
        /// The state.
        value: u16,
    },
    /// The reply to the kitty graphics query: true for `OK`.
    Graphics(bool),
    /// Primary device attributes, the last reply.
    Da1,
}

/// The replies at the start of a buffer, and the bytes that are not replies.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Parsed {
    /// The replies, in order.
    pub replies: Vec<Reply>,
    /// Each byte that is not part of a reply, in order: keys, a paste or other input.
    pub other: Vec<u8>,
    /// The bytes that the parse used. An incomplete sequence at the end stays.
    pub used: usize,
}

/// Parses the replies in `buf`. A sequence that is not a reply goes to [`Parsed::other`]
/// whole. An incomplete sequence at the end stays in the buffer.
#[must_use]
pub fn parse(buf: &[u8]) -> Parsed {
    let mut parsed = Parsed::default();
    let mut at = 0;
    while at < buf.len() {
        if buf[at] != 0x1b {
            parsed.other.push(buf[at]);
            at += 1;
            continue;
        }
        let (reply, len) = match buf.get(at + 1) {
            None => break,
            Some(b'[') => {
                let Some(end) = buf[at + 2..].iter().position(|b| (0x40..=0x7e).contains(b)) else {
                    break;
                };
                let end = at + 2 + end;
                let params = std::str::from_utf8(&buf[at + 2..end]).unwrap_or("");
                (csi(params, buf[end]), end + 1 - at)
            }
            Some(b'_') => {
                let Some(end) = buf[at + 2..].windows(2).position(|pair| pair == b"\x1b\\") else {
                    break;
                };
                let body = String::from_utf8_lossy(&buf[at + 2..at + 2 + end]);
                (graphics(&body), end + 4)
            }
            Some(_) => (None, 1),
        };
        match reply {
            Some(reply) => parsed.replies.push(reply),
            None => parsed.other.extend_from_slice(&buf[at..at + len]),
        }
        at += len;
    }
    parsed.used = at;
    parsed
}

fn csi(params: &str, end: u8) -> Option<Reply> {
    let numbers =
        |text: &str| -> Option<Vec<u16>> { text.split(';').map(|n| n.parse().ok()).collect() };
    match end {
        b'R' => match numbers(params)?[..] {
            [row, col] => Some(Reply::Cursor { row, col }),
            _ => None,
        },
        b'y' => {
            let inner = params.strip_prefix('?')?.strip_suffix('$')?;
            match numbers(inner)?[..] {
                [mode, value] => Some(Reply::Mode { mode, value }),
                _ => None,
            }
        }
        b'c' if params.starts_with('?') => Some(Reply::Da1),
        _ => None,
    }
}

fn graphics(body: &str) -> Option<Reply> {
    let (keys, message) = body.strip_prefix('G')?.split_once(';')?;
    keys.split(',')
        .any(|key| key == GRAPHICS_ID)
        .then(|| Reply::Graphics(message == "OK"))
}

/// What the replies of the probe give.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// The width rule of each class.
    pub profile: Profile,
    /// The state of mode 2027 before the probe set it, if the terminal answered.
    pub grapheme_mode: Option<u16>,
    /// True if the terminal draws kitty images.
    pub graphics: bool,
    /// True if DA1 answered, so every reply came.
    pub complete: bool,
}

impl Outcome {
    /// Learns from the replies in order. The cursor position after each grapheme of
    /// [`GRAPHEMES`] gives its width. Before DA1, the replies are not complete, and every class
    /// keeps the `unicode-width` value.
    #[must_use]
    pub fn learn(replies: &[Reply]) -> Self {
        let mut outcome = Self::default();
        let mut cursors = Vec::new();
        for reply in replies {
            match *reply {
                Reply::Cursor { col, .. } => cursors.push(col.saturating_sub(1)),
                Reply::Mode {
                    mode: GRAPHEME_MODE,
                    value,
                } => outcome.grapheme_mode = Some(value),
                Reply::Mode { .. } => {}
                Reply::Graphics(ok) => outcome.graphics = ok,
                Reply::Da1 => outcome.complete = true,
            }
        }
        if outcome.complete {
            let measured: Vec<(&str, u16)> = GRAPHEMES.iter().copied().zip(cursors).collect();
            outcome.profile = Profile::learn(&measured);
        }
        outcome
    }

    /// True if the client turns mode 2027 off at exit: the probe set it, and the terminal did
    /// not give it as set before. With no reply in time, the probe cannot know, and the mode
    /// goes off.
    #[must_use]
    pub const fn resets_grapheme_mode(&self) -> bool {
        !matches!(self.grapheme_mode, Some(1 | 3))
    }
}

/// The longest time that the probe waits for DA1.
pub const WAIT: Duration = Duration::from_millis(500);

/// Writes the queries to `out` and reads the replies from the terminal until DA1 or `wait`.
/// Returns what the replies give, and every byte that is not a reply, for the Termina parser.
/// The terminal must be in raw mode.
///
/// # Errors
///
/// An I/O error of the terminal.
pub fn run(out: &mut impl Write, wait: Duration) -> io::Result<(Outcome, Vec<u8>)> {
    // Termina reads from stdin if it is a terminal, and from `/dev/tty` if it is not.
    let tty = if io::stdin().is_terminal() {
        None
    } else {
        Some(File::open("/dev/tty")?)
    };
    let stdin = io::stdin();
    let fd = tty.as_ref().map_or_else(|| stdin.as_fd(), AsFd::as_fd);
    let raw = tcgetattr(fd)?;
    // With VMIN 0 and VTIME 1, a read returns after at most one tenth of a second.
    let mut timed = raw.clone();
    timed.special_codes[SpecialCodeIndex::VMIN] = 0;
    timed.special_codes[SpecialCodeIndex::VTIME] = 1;
    tcsetattr(fd, OptionalActions::Now, &timed)?;
    let read = write_and_read(fd, out, wait);
    tcsetattr(fd, OptionalActions::Now, &raw)?;
    let (replies, other) = read?;
    Ok((Outcome::learn(&replies), other))
}

fn write_and_read(
    fd: BorrowedFd<'_>,
    out: &mut impl Write,
    wait: Duration,
) -> io::Result<(Vec<Reply>, Vec<u8>)> {
    out.write_all(&queries())?;
    out.flush()?;
    let deadline = Instant::now() + wait;
    let mut pending = Vec::new();
    let mut replies = Vec::new();
    let mut other = Vec::new();
    let mut chunk = [0; 1024];
    while !replies.contains(&Reply::Da1) && Instant::now() < deadline {
        let count = match rustix::io::read(fd, &mut chunk) {
            Ok(count) => count,
            Err(rustix::io::Errno::INTR) => continue,
            Err(error) => return Err(error.into()),
        };
        pending.extend_from_slice(&chunk[..count]);
        let parsed = parse(&pending);
        replies.extend(parsed.replies);
        other.extend(parsed.other);
        pending.drain(..parsed.used);
    }
    other.extend(pending);
    Ok((replies, other))
}
