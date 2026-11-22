//! The startup probe: the queries, the framer of the input, and what the answers give.
//!
//! The probe writes every query in one write. An answer is the reply of the terminal to a query.
//! The probe asks for the state of mode 2027, sets the mode, writes one grapheme of each width
//! class with a cursor position query after each, asks whether the terminal draws kitty images,
//! and ends with DA1. Every terminal answers DA1, so its answer ends the answers.
//!
//! The probe reads the answers itself, before the Termina event reader starts. The event reader
//! gives the kitty graphics answer as keys, and its parser loses an answer that follows a stray
//! ESC. The probe cuts the input into frames by the byte classes of ECMA-48 and by its own rule for
//! a broken sequence. It gives each complete CSI to a Termina parser alone. It decodes the kitty
//! graphics answer itself.

use std::fs::File;
use std::io::{self, IsTerminal, Write};
use std::os::fd::{AsFd, BorrowedFd};
use std::time::{Duration, Instant};

use rustix::termios::{OptionalActions, SpecialCodeIndex, tcgetattr, tcsetattr};
use termina::escape::csi::{
    Csi, Cursor, DecModeSetting, DecPrivateMode, DecPrivateModeCode, Device, Keyboard, Mode,
};
use termina::event::{KeyCode, KeyEvent, Modifiers};
use termina::{Event, Parser};

use crate::screen;
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

/// The key of the id in the kitty graphics query and its answer.
const GRAPHICS_ID: &str = "i=31";

/// The kitty graphics query: one RGB pixel, sent directly, with the query action.
const GRAPHICS_QUERY: &str = "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\";

/// The query of the kitty keyboard flags. A terminal with the protocol answers `CSI ? flags u`.
const KEYBOARD_QUERY: &str = "\x1b[?u";

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
    out.push_str(KEYBOARD_QUERY);
    out.push_str("\x1b[c");
    out.into_bytes()
}

/// One answer of the terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// A cursor position report, from 1.
    Cursor {
        /// The row.
        row: u16,
        /// The column.
        col: u16,
    },
    /// DECRPM: the state of a DEC private mode.
    Mode {
        /// The mode.
        mode: DecPrivateMode,
        /// The state.
        setting: DecModeSetting,
    },
    /// The answer to the kitty graphics query: true for `OK`.
    Graphics(bool),
    /// The answer to the query of the kitty keyboard flags, so the terminal knows the protocol.
    Keyboard,
    /// Primary device attributes, the last answer.
    Da1,
}

/// What the framer took from the bytes that it got.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Framed {
    /// The answers, in order.
    pub answers: Vec<Answer>,
    /// The input that is not an answer, in order: keys, text and other events.
    pub events: Vec<Event>,
}

const ESC: u8 = 0x1b;

/// Whether `b` is a final byte of a CSI, 0x40 to 0x7E.
#[must_use]
pub const fn is_csi_final(b: u8) -> bool {
    matches!(b, 0x40..=0x7e)
}

/// Whether `b` is a parameter or intermediate byte of a CSI, 0x20 to 0x3F.
#[must_use]
pub const fn is_csi_body(b: u8) -> bool {
    matches!(b, 0x20..=0x3f)
}

/// The most bytes that [`frame`] holds for an open sequence.
pub const PENDING_MAX: usize = 4096;

/// What the framer drops at the start of the next read: the rest of a sequence that passed
/// [`PENDING_MAX`] and was dropped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Skip {
    /// Nothing: the next read starts a frame.
    #[default]
    Nothing,
    /// The rest of a CSI, up to its final byte or a byte that breaks it.
    Csi,
    /// The rest of an APC, up to its ESC `\`.
    Apc,
    /// The rest of an APC after an ESC at the end of the last read. A `\` ends it.
    ApcEsc,
}

/// Adds `read` to `pending` and frames the bytes of `pending` that make whole frames. A frame is
/// a complete CSI, an APC, a broken sequence, or a run of other bytes. A CSI or APC with no end,
/// an ESC with at most one byte after it, and the first bytes of a UTF-8 character stay in
/// `pending` for the next read. `pending` holds at most [`PENDING_MAX`] bytes. If an open CSI or
/// APC grows past that limit, the framer drops its bytes, and `skip` makes it drop each byte of
/// the later reads up to the end of that sequence.
///
/// - A complete CSI goes alone to a Termina parser. A cursor position, DECRPM or DA1 event is an
///   [`Answer`]. Each other event is input.
/// - An APC is the graphics answer, or it is dropped.
/// - An ESC inside an open CSI or APC, or a byte outside the bytes of a CSI, ends the sequence
///   as broken. The keys of its bytes are input, and a new frame starts after them.
/// - Other bytes go to a Termina parser.
pub fn frame(pending: &mut Vec<u8>, skip: &mut Skip, read: &[u8]) -> Framed {
    let read = &read[skipped(skip, read, pending)..];
    pending.extend_from_slice(read);
    let mut framed = Framed::default();
    let mut used = 0;
    let mut at = 0;
    while at < pending.len() {
        let next = pending.get(at + 1);
        if pending[at] != ESC || !matches!(next, None | Some(b'[' | b'_')) {
            at += 1;
            continue;
        }
        framed.events.extend(plain(&pending[used..at]));
        used = at;
        if next.is_none() {
            break;
        }
        let rest = &pending[at..];
        let len = match scan(rest) {
            Scan::Open => break,
            Scan::Csi(len) => {
                csi(&rest[..len], &mut framed);
                len
            }
            Scan::Apc(len) => {
                framed
                    .answers
                    .extend(graphics(&String::from_utf8_lossy(&rest[2..len - 2])));
                len
            }
            Scan::Broken(len) => {
                framed.events.extend(broken(&rest[..len]));
                len
            }
        };
        at += len;
        used = at;
    }
    if used < at {
        let end = at - unfinished_tail(&pending[used..at]);
        framed.events.extend(plain(&pending[used..end]));
        used = end;
    }
    pending.drain(..used);
    if pending.len() > PENDING_MAX {
        *skip = match (pending[1], pending.last()) {
            (b'_', Some(&ESC)) => Skip::ApcEsc,
            (b'_', _) => Skip::Apc,
            _ => Skip::Csi,
        };
        pending.clear();
    }
    framed
}

/// The count of bytes at the start of `read` that belong to the sequence that `skip` drops. Sets
/// `skip` to what is left to drop after `read`. A byte that breaks the sequence is not one of
/// them: a new frame starts at it, and the ESC that ended the sequence in the last read comes
/// back to `pending`.
fn skipped(skip: &mut Skip, read: &[u8], pending: &mut Vec<u8>) -> usize {
    for (at, &byte) in read.iter().enumerate() {
        match (*skip, byte) {
            (Skip::Apc, ESC) => match read.get(at + 1) {
                Some(b'\\') => {
                    *skip = Skip::Nothing;
                    return at + 2;
                }
                Some(_) => {
                    *skip = Skip::Nothing;
                    return at;
                }
                None => *skip = Skip::ApcEsc,
            },
            (Skip::Apc, _) => {}
            (Skip::Csi, _) if is_csi_body(byte) => {}
            (Skip::ApcEsc, b'\\') => {
                *skip = Skip::Nothing;
                return at + 1;
            }
            (Skip::Csi, _) if is_csi_final(byte) => {
                *skip = Skip::Nothing;
                return at + 1;
            }
            (Skip::ApcEsc, _) => {
                *skip = Skip::Nothing;
                pending.push(ESC);
                return at;
            }
            (Skip::Csi | Skip::Nothing, _) => {
                *skip = Skip::Nothing;
                return at;
            }
        }
    }
    read.len()
}

/// The count of bytes at the end of `run` that can need bytes of the next read: an ESC with one
/// byte after it, or the first bytes of a UTF-8 character.
fn unfinished_tail(run: &[u8]) -> usize {
    if run.len() >= 2 && run[run.len() - 2] == ESC {
        return 2;
    }
    for (back, &byte) in run.iter().rev().enumerate().take(3) {
        let width = match byte {
            0x80..=0xbf => continue,
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf7 => 4,
            _ => return 0,
        };
        return if back + 1 < width { back + 1 } else { 0 };
    }
    0
}

/// The keys of the bytes that the framer still holds at the end of the probe, as the keys of a
/// broken sequence.
#[must_use]
pub fn flush(pending: &[u8]) -> Vec<Event> {
    broken(pending)
}

/// How a sequence that starts with ESC `[` or ESC `_` ends.
enum Scan {
    /// It has no end yet.
    Open,
    /// A complete CSI of this many bytes.
    Csi(usize),
    /// A complete APC of this many bytes.
    Apc(usize),
    /// A sequence of this many bytes that another byte ended before its end.
    Broken(usize),
}

fn scan(buf: &[u8]) -> Scan {
    let apc = buf[1] == b'_';
    for (at, &byte) in buf.iter().enumerate().skip(2) {
        match byte {
            ESC if apc && buf.get(at + 1) == Some(&b'\\') => return Scan::Apc(at + 2),
            ESC if apc && at + 1 == buf.len() => return Scan::Open,
            ESC => return Scan::Broken(at),
            _ if apc => {}
            b if is_csi_final(b) => return Scan::Csi(at + 1),
            b if is_csi_body(b) => {}
            _ => return Scan::Broken(at),
        }
    }
    Scan::Open
}

/// The events that a Termina parser gives for one complete CSI, with no more input to come.
/// [`frame`] decodes each CSI with this function, and so does the update of a late answer.
#[must_use]
pub fn decode_csi(bytes: &[u8]) -> Vec<Event> {
    plain(bytes)
}

/// Decodes one complete CSI and sorts its events.
fn csi(bytes: &[u8], framed: &mut Framed) {
    for event in decode_csi(bytes) {
        match event {
            Event::Csi(Csi::Cursor(Cursor::ActivePositionReport { line, col })) => {
                framed.answers.push(Answer::Cursor {
                    row: line.get(),
                    col: col.get(),
                });
            }
            Event::Csi(Csi::Mode(Mode::ReportDecPrivateMode { mode, setting })) => {
                framed.answers.push(Answer::Mode { mode, setting });
            }
            Event::Csi(Csi::Device(Device::DeviceAttributes(()))) => {
                framed.answers.push(Answer::Da1);
            }
            // Only the presence of the answer counts. Termina reads one digit of the flags.
            Event::Csi(Csi::Keyboard(Keyboard::ReportFlags(_))) => {
                framed.answers.push(Answer::Keyboard);
            }
            other => framed.events.push(other),
        }
    }
}

/// The events that a Termina parser gives for `bytes`, with no more input to come.
fn plain(bytes: &[u8]) -> Vec<Event> {
    let mut parser = Parser::default();
    parser.parse(bytes, false);
    std::iter::from_fn(|| parser.pop()).collect()
}

/// The keys of an unfinished sequence. ESC and the byte after it give that key with Alt. A
/// Termina parser gives the keys of the other bytes.
fn broken(bytes: &[u8]) -> Vec<Event> {
    match bytes {
        [ESC, next, rest @ ..] => {
            let alt = KeyEvent::new(KeyCode::Char(char::from(*next)), Modifiers::ALT);
            std::iter::once(Event::Key(alt))
                .chain(plain(rest))
                .collect()
        }
        _ => plain(bytes),
    }
}

fn graphics(body: &str) -> Option<Answer> {
    let (keys, message) = body.strip_prefix('G')?.split_once(';')?;
    keys.split(',')
        .any(|key| key == GRAPHICS_ID)
        .then(|| Answer::Graphics(message == "OK"))
}

/// How the terminal reports keys.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyboardProtocol {
    /// The terminal did not answer the query of the kitty keyboard flags before the probe ended.
    #[default]
    Legacy,
    /// The terminal answered the query, so it knows the kitty keyboard protocol.
    Kitty,
}

/// What the answers of the probe give.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// The width rule of each class.
    pub profile: Profile,
    /// The state of mode 2027 before the probe set it, if the terminal answered.
    pub grapheme_mode: Option<DecModeSetting>,
    /// True if the terminal draws kitty images.
    pub graphics: bool,
    /// The keyboard protocol of the terminal, from the query of the kitty keyboard flags.
    pub keyboard: KeyboardProtocol,
    /// True if DA1 answered, so every answer came.
    pub complete: bool,
    /// True if the cursor answers gave the width of each grapheme of [`GRAPHEMES`]. If false,
    /// every class keeps the `unicode-width` value.
    pub measured: bool,
    /// The bytes of an answer that the deadline cut, when DA1 did not come: an open CSI, or a lone
    /// ESC. The rest of the answer comes as keys.
    pub open: Vec<u8>,
}

impl Outcome {
    /// Learns from the answers in order. The cursor position after each grapheme of
    /// [`GRAPHEMES`] gives its width. Before DA1, the answers are not complete. With no DA1, or
    /// with a count of cursor answers that differs from the count of graphemes, every class keeps
    /// the `unicode-width` value.
    #[must_use]
    pub fn learn(answers: &[Answer]) -> Self {
        let mut outcome = Self::default();
        let mut cursors = Vec::new();
        for answer in answers {
            match *answer {
                Answer::Cursor { col, .. } => cursors.push(col.saturating_sub(1)),
                Answer::Mode {
                    mode: DecPrivateMode::Code(DecPrivateModeCode::GraphemeClustering),
                    setting,
                } => outcome.grapheme_mode = Some(setting),
                Answer::Mode { .. } => {}
                Answer::Graphics(ok) => outcome.graphics = ok,
                Answer::Keyboard => outcome.keyboard = KeyboardProtocol::Kitty,
                Answer::Da1 => outcome.complete = true,
            }
        }
        outcome.measured = outcome.complete && cursors.len() == GRAPHEMES.len();
        if outcome.measured {
            let measured: Vec<(&str, u16)> = GRAPHEMES.iter().copied().zip(cursors).collect();
            outcome.profile = Profile::learn(&measured);
        }
        outcome
    }

    /// True if the client turns mode 2027 off at exit: the probe set it, and the terminal did
    /// not give it as set before. With no answer in time, the probe cannot know, and the mode
    /// goes off.
    #[must_use]
    pub fn resets_grapheme_mode(&self) -> bool {
        !self.grapheme_mode.is_some_and(is_set)
    }
}

/// True if a DECRPM setting says that the mode is on, now or for good.
#[must_use]
pub const fn is_set(setting: DecModeSetting) -> bool {
    matches!(
        setting,
        DecModeSetting::Set | DecModeSetting::PermanentlySet
    )
}

/// The longest time that the probe waits for DA1.
pub const WAIT: Duration = Duration::from_millis(500);

/// The time that one read of the probe waits, in tenths of a second. It is the VTIME value of
/// the terminal.
const TICK_TENTHS: u8 = 1;

/// The longest time that one read of the probe blocks.
pub const TICK: Duration = Duration::from_millis(TICK_TENTHS as u64 * 100);

/// Writes the queries to `out` and reads the answers from the terminal until DA1 or `wait`.
/// Returns what the answers give, and the input that is not an answer, as events. The terminal
/// must be in raw mode.
///
/// # Errors
///
/// An I/O error of the terminal.
pub fn run(out: &mut impl Write, wait: Duration) -> io::Result<(Outcome, Vec<Event>)> {
    // Termina reads from stdin if it is a terminal, and from `/dev/tty` if it is not.
    let tty = if io::stdin().is_terminal() {
        None
    } else {
        Some(File::open("/dev/tty")?)
    };
    let stdin = io::stdin();
    let fd = tty.as_ref().map_or_else(|| stdin.as_fd(), AsFd::as_fd);
    let raw = tcgetattr(fd)?;
    // With VMIN 0, a read returns after at most `TICK`.
    let mut timed = raw.clone();
    timed.special_codes[SpecialCodeIndex::VMIN] = 0;
    timed.special_codes[SpecialCodeIndex::VTIME] = TICK_TENTHS;
    tcsetattr(fd, OptionalActions::Now, &timed)?;
    let read = write_and_read(fd, out, wait);
    let restored = tcsetattr(fd, OptionalActions::Now, &raw).map_err(io::Error::from);
    screen::finish(read, restored)
}

fn write_and_read(
    fd: BorrowedFd<'_>,
    out: &mut impl Write,
    wait: Duration,
) -> io::Result<(Outcome, Vec<Event>)> {
    out.write_all(&queries())?;
    out.flush()?;
    read_answers(|buf| Ok(rustix::io::read(fd, buf)?), Instant::now, wait)
}

/// Reads with `read` and frames the bytes until DA1 or the time `wait` after the start. A read
/// starts only if it can end by then, so the last tick can stay unread. `now` is the clock.
/// Returns what the answers give, and the input that is not an answer, as events.
///
/// If DA1 did not come, an open CSI or a lone ESC at the end is in [`Outcome::open`]. Every other
/// byte that the framer still holds is input.
///
/// # Errors
///
/// An error of `read` other than an interrupt.
pub fn read_answers(
    mut read: impl FnMut(&mut [u8]) -> io::Result<usize>,
    now: impl Fn() -> Instant,
    wait: Duration,
) -> io::Result<(Outcome, Vec<Event>)> {
    let deadline = now() + wait;
    let mut pending = Vec::new();
    let mut skip = Skip::Nothing;
    let mut answers = Vec::new();
    let mut events = Vec::new();
    let mut chunk = [0; 1024];
    while !answers.contains(&Answer::Da1) && now() + TICK <= deadline {
        let count = match read(&mut chunk) {
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        let framed = frame(&mut pending, &mut skip, &chunk[..count]);
        answers.extend(framed.answers);
        events.extend(framed.events);
    }
    let mut outcome = Outcome::learn(&answers);
    if !outcome.complete && matches!(pending.as_slice(), [ESC] | [ESC, b'[', ..]) {
        outcome.open = pending;
    } else {
        events.extend(flush(&pending));
    }
    Ok((outcome, events))
}
