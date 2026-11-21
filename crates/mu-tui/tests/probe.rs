//! The framer of the probe input, the read loop of the probe, and what the answers give.

use std::cell::Cell;
use std::io;
use std::time::{Duration, Instant};

use mu_tui::probe::{
    Answer, GRAPHEMES, Outcome, PENDING_MAX, Skip, TICK, WAIT, flush, frame, is_set, read_answers,
};
use mu_tui::width::{Class, Rule};
use termina::Event;
use termina::escape::csi::{DecModeSetting, DecPrivateMode, DecPrivateModeCode};
use termina::event::{KeyCode, KeyEvent, Modifiers};

const GRAPHEME_MODE: DecPrivateMode = DecPrivateMode::Code(DecPrivateModeCode::GraphemeClustering);

fn mode(setting: DecModeSetting) -> Answer {
    Answer::Mode {
        mode: GRAPHEME_MODE,
        setting,
    }
}

fn key(c: char, modifiers: Modifiers) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(c), modifiers))
}

/// What the framer gave for `reads`, and the bytes that it still holds.
#[derive(Default)]
struct Framing {
    answers: Vec<Answer>,
    events: Vec<Event>,
    pending: Vec<u8>,
    skip: Skip,
}

/// Frames each of `reads` in turn, as the probe does.
fn framed(reads: &[&[u8]]) -> Framing {
    let mut framing = Framing::default();
    for read in reads {
        let step = frame(&mut framing.pending, &mut framing.skip, read);
        framing.answers.extend(step.answers);
        framing.events.extend(step.events);
    }
    framing
}

#[test]
fn replies_parse() {
    let framing = framed(&[b"\x1b[?2027;2$y\x1b[1;3R\x1b_Gi=31;OK\x1b\\\x1b[?62;22c"]);
    assert_eq!(
        framing.answers,
        [
            mode(DecModeSetting::Reset),
            Answer::Cursor { row: 1, col: 3 },
            Answer::Graphics(true),
            Answer::Da1,
        ]
    );
}

#[test]
fn replies_come_from_termina() {
    let each = [
        (b"\x1b[12;5R".as_slice(), Answer::Cursor { row: 12, col: 5 }),
        (b"\x1b[?2027;2$y", mode(DecModeSetting::Reset)),
        (b"\x1b[?62;22c", Answer::Da1),
    ];
    for (reply, answer) in each {
        let framing = framed(&[reply]);
        assert_eq!((framing.answers, framing.events), (vec![answer], vec![]));
    }
}

#[test]
fn graphics_error_is_no_graphics() {
    let framing = framed(&[b"\x1b_Gi=31;ENOTSUPPORTED:no\x1b\\"]);
    assert_eq!(framing.answers, [Answer::Graphics(false)]);
}

#[test]
fn graphics_reply_of_another_id_is_dropped() {
    let framing = framed(&[b"\x1b_Gi=7;OK\x1b\\"]);
    assert_eq!((framing.answers, framing.events), (vec![], vec![]));
}

#[test]
fn keys_during_the_probe_are_early_events() {
    let framing = framed(&[b"a\x1b[1;2Rb\x1b[Ac"]);
    assert_eq!(
        framing.events,
        [
            key('a', Modifiers::NONE),
            key('b', Modifiers::NONE),
            Event::Key(KeyCode::Up.into()),
            key('c', Modifiers::NONE),
        ]
    );
}

#[test]
fn alt_bracket_before_a_reply() {
    let framing = framed(&[b"\x1b[\x1b[12;5R"]);
    assert_eq!(
        (framing.events, framing.answers),
        (
            vec![key('[', Modifiers::ALT)],
            vec![Answer::Cursor { row: 12, col: 5 }]
        )
    );
}

#[test]
fn question_csi_before_a_reply() {
    let framing = framed(&[b"\x1b[?\x1b[12;5R\x1b[?62c"]);
    assert_eq!(
        framing.answers,
        [Answer::Cursor { row: 12, col: 5 }, Answer::Da1]
    );
}

/// A clock that only the reads of [`FakeClock::read`] move. `reads` is their count.
struct FakeClock {
    start: Instant,
    elapsed: Cell<Duration>,
    reads: Cell<usize>,
}

impl FakeClock {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            elapsed: Cell::new(Duration::ZERO),
            reads: Cell::new(0),
        }
    }

    fn now(&self) -> Instant {
        self.start + self.elapsed.get()
    }

    /// A read that gives `bytes` once, then no bytes. Each read takes one tick, and the first
    /// read takes 1 millisecond more.
    fn read<'a>(&'a self, bytes: &'a [u8]) -> impl FnMut(&mut [u8]) -> io::Result<usize> + 'a {
        move |buf| {
            let first = self.reads.get() == 0;
            self.reads.set(self.reads.get() + 1);
            let slow = if first {
                Duration::from_millis(1)
            } else {
                Duration::ZERO
            };
            self.elapsed.set(self.elapsed.get() + TICK + slow);
            let given = if first { bytes.len() } else { 0 };
            buf[..given].copy_from_slice(&bytes[..given]);
            Ok(given)
        }
    }
}

#[test]
fn open_apc_does_not_hide_da1() {
    let clock = FakeClock::new();
    let (outcome, events) =
        read_answers(clock.read(b"\x1b_\x1b[?62c"), || clock.now(), WAIT).unwrap();
    assert_eq!(
        (outcome.complete, events, clock.reads.get()),
        (true, vec![key('_', Modifiers::ALT)], 1)
    );
}

#[test]
fn cut_reply_is_open_in_the_outcome() {
    let clock = FakeClock::new();
    let (outcome, events) = read_answers(clock.read(b"\x1b[12;"), || clock.now(), WAIT).unwrap();
    assert_eq!(
        (outcome.complete, outcome.open, events),
        (false, b"\x1b[12;".to_vec(), vec![])
    );
}

#[test]
fn silent_terminal_keeps_the_bound() {
    let clock = FakeClock::new();
    read_answers(clock.read(b""), || clock.now(), WAIT).unwrap();
    assert_eq!(
        (clock.elapsed.get() <= WAIT, clock.reads.get()),
        (true, 4),
        "{:?}",
        clock.elapsed.get()
    );
}

#[test]
fn apc_end_split_across_reads() {
    let framing = framed(&[b"\x1b_Gi=31;OK\x1b", b"\\"]);
    assert_eq!(framing.answers, [Answer::Graphics(true)]);
}

#[test]
fn split_reply_across_reads() {
    let framing = framed(&[b"\x1b[12", b";5R"]);
    assert_eq!(framing.answers, [Answer::Cursor { row: 12, col: 5 }]);
}

#[test]
fn utf8_split_across_reads() {
    let framing = framed(&[b"\xc3", b"\xa9"]);
    assert_eq!(framing.events, [key('é', Modifiers::NONE)]);
    let framing = framed(&[b"\xe2\x82", b"\xac"]);
    assert_eq!(framing.events, [key('€', Modifiers::NONE)]);
    let framing = framed(&[b"\xf0\x9f\x98", b"\x80"]);
    assert_eq!(framing.events, [key('😀', Modifiers::NONE)]);
}

#[test]
fn ss3_split_across_reads() {
    let framing = framed(&[b"\x1bO", b"P"]);
    assert_eq!(framing.events, [Event::Key(KeyCode::Function(1).into())]);
}

#[test]
fn long_apc_is_dropped() {
    let long = [b"\x1b_".as_slice(), &[b'x'; PENDING_MAX]].concat();
    let framing = framed(&[&long, b"\x1b[?62c"]);
    assert_eq!(
        (framing.answers, framing.events, framing.pending),
        (vec![Answer::Da1], vec![], vec![])
    );
}

#[test]
fn apc_of_the_cap_is_kept() {
    let apc = [b"\x1b_".as_slice(), &[b'x'; PENDING_MAX - 2]].concat();
    let framing = framed(&[&apc]);
    assert_eq!(
        (framing.pending.len(), framing.skip),
        (PENDING_MAX, Skip::Nothing)
    );
}

#[test]
fn apc_one_byte_past_the_cap_is_dropped() {
    let apc = [b"\x1b_".as_slice(), &[b'x'; PENDING_MAX - 1]].concat();
    let framing = framed(&[&apc]);
    assert_eq!((framing.pending, framing.skip), (vec![], Skip::Apc));
}

#[test]
fn long_sequence_tail_is_dropped() {
    let apc = [b"\x1b_".as_slice(), &[b'x'; PENDING_MAX]].concat();
    let framing = framed(&[&apc, b"xxxx", b"\x1b", b"\\", b"\x1b[?62c"]);
    assert_eq!(
        (framing.answers, framing.events, framing.pending),
        (vec![Answer::Da1], vec![], vec![])
    );
    let csi = [b"\x1b[".as_slice(), &[b'1'; PENDING_MAX]].concat();
    let framing = framed(&[&csi, b"11", b"2c", b"\x1b[?62c"]);
    assert_eq!(
        (framing.answers, framing.events, framing.pending),
        (vec![Answer::Da1], vec![], vec![])
    );
}

#[test]
fn dropped_tail_keeps_the_esc_of_the_next_frame() {
    let apc = [b"\x1b_".as_slice(), &[b'x'; PENDING_MAX]].concat();
    let framing = framed(&[&apc, b"xx\x1b", b"[?62c"]);
    assert_eq!(
        (framing.answers, framing.events),
        (vec![Answer::Da1], vec![])
    );
}

#[test]
fn incomplete_reply_stays() {
    let framing = framed(&[b"x\x1b[1;"]);
    assert_eq!(framing.pending, b"\x1b[1;");
}

#[test]
fn incomplete_graphics_reply_stays() {
    let framing = framed(&[b"\x1b_Gi=31;O"]);
    assert_eq!(framing.pending, b"\x1b_Gi=31;O");
}

#[test]
fn open_sequence_at_the_end_becomes_keys() {
    assert_eq!(
        flush(b"\x1b[1"),
        [key('[', Modifiers::ALT), key('1', Modifiers::NONE)]
    );
}

/// The replies of a terminal that gives each probe grapheme the width that `width` returns, with no DA1 if
/// `complete` is false.
fn replies(width: impl Fn(&str) -> u16, complete: bool) -> Vec<Answer> {
    let mut replies: Vec<Answer> = GRAPHEMES
        .iter()
        .map(|g| Answer::Cursor {
            row: 1,
            col: width(g) + 1,
        })
        .collect();
    if complete {
        replies.push(Answer::Da1);
    }
    replies
}

fn eight_wide_zwj(g: &str) -> u16 {
    if Class::of(g) == Class::Zwj { 8 } else { 2 }
}

#[test]
fn widths_come_from_the_cursor_positions() {
    let outcome = Outcome::learn(&replies(eight_wide_zwj, true));
    assert_eq!(outcome.profile.width("👩\u{200D}👩\u{200D}👦"), 8);
}

#[test]
fn replies_with_no_da1_keep_the_default_profile() {
    let outcome = Outcome::learn(&replies(eight_wide_zwj, false));
    assert_eq!(outcome.profile.rule(Class::Zwj), Rule::Unicode);
}

#[test]
fn mode_reset_means_the_probe_turned_it_on() {
    let outcome = Outcome::learn(&[mode(DecModeSetting::Reset), Answer::Da1]);
    assert!(outcome.resets_grapheme_mode());
}

#[test]
fn mode_set_means_the_probe_left_it() {
    let outcome = Outcome::learn(&[mode(DecModeSetting::Set), Answer::Da1]);
    assert!(!outcome.resets_grapheme_mode());
}

#[test]
fn no_mode_reply_means_the_mode_goes_off() {
    assert!(Outcome::learn(&[]).resets_grapheme_mode());
}

#[test]
fn permanently_set_mode_stays() {
    let outcome = Outcome::learn(&[mode(DecModeSetting::PermanentlySet)]);
    assert!(!outcome.resets_grapheme_mode());
}

#[test]
fn cursor_reply_for_each_grapheme_measures_the_widths() {
    assert!(Outcome::learn(&replies(eight_wide_zwj, true)).measured);
}

#[test]
fn extra_cursor_reply_keeps_the_default_profile() {
    let mut extra = replies(eight_wide_zwj, true);
    extra.insert(0, Answer::Cursor { row: 1, col: 3 });
    let outcome = Outcome::learn(&extra);
    assert_eq!(
        (outcome.measured, outcome.profile.rule(Class::Zwj)),
        (false, Rule::Unicode)
    );
}

#[test]
fn only_set_and_permanently_set_are_set() {
    let settings = [
        DecModeSetting::NotRecognized,
        DecModeSetting::Set,
        DecModeSetting::Reset,
        DecModeSetting::PermanentlySet,
        DecModeSetting::PermanentlyReset,
    ];
    assert_eq!(settings.map(is_set), [false, true, false, true, false]);
}
