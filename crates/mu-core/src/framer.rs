//! Framers: bytes of the game in, lines and prompts out, and a line in, bytes out.

use crate::telnet::{DO, IAC, WILL, WONT, opt};

/// What a framer gives for the bytes of the game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inbound {
    /// A line, with no line end.
    Line(String),
    /// A prompt: text with no line end that the game flushed.
    Prompt(String),
    /// Bytes to write back to the game, for example a negotiation reply.
    Reply(Vec<u8>),
    /// The game answered a mark of the client: it read every byte that came before the mark.
    Mark,
    /// A message of a side protocol, for example GMCP.
    Message {
        /// The name of the side protocol.
        name: String,
        /// The body of the message.
        body: Vec<u8>,
    },
}

/// Whether `text` holds a CR or an LF, which would split it into two lines to the game.
pub(crate) fn has_line_break(text: &str) -> bool {
    text.contains(['\r', '\n'])
}

/// Makes the framer of each new connection of a world.
pub type NewFramer = std::sync::Arc<dyn Fn() -> Box<dyn Framer> + Send + Sync>;

/// The framing of one connection: how its bytes become lines, and its lines bytes.
pub trait Framer: Send + 'static {
    /// Takes the bytes of one read and returns what they complete.
    fn feed(&mut self, bytes: &[u8]) -> Vec<Inbound>;

    /// The bytes of one line to the game.
    fn encode(&self, line: &str) -> Vec<u8>;

    /// The bytes of a mark to the game. The game answers it with [`Inbound::Mark`]. A framer
    /// that gives no bytes has no mark.
    fn mark(&self) -> Vec<u8> {
        Vec::new()
    }
}

/// The longest line that [`LineFramer`] holds. A longer run of bytes with no LF is cut into
/// lines of this length, so a game cannot grow the buffer without a bound.
pub const MAX_LINE: usize = 64 * 1024;

/// Which end of the connection a [`LineFramer`] reads for.
#[derive(Clone, Copy, Debug, Default)]
enum Side {
    /// Reads the game, and sends marks to it.
    #[default]
    Client,
    /// Reads the client, and answers its marks.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "only LineFramer::game builds it")
    )]
    Game,
}

impl Side {
    /// Whether `IAC verb TIMING-MARK` is a mark for this side.
    fn reads_verb(self, verb: u8) -> bool {
        match self {
            Self::Client => matches!(verb, WILL | WONT),
            Self::Game => verb == DO,
        }
    }

    fn on_mark(self) -> Inbound {
        match self {
            Self::Client => Inbound::Mark,
            Self::Game => Inbound::Reply(vec![IAC, WILL, opt::TIMING_MARK]),
        }
    }
}

/// The bytes of a possible mark that end a read.
#[derive(Clone, Copy, Debug, Default)]
enum Held {
    #[default]
    Nothing,
    Iac,
    Verb(u8),
}

/// Splits on LF and drops each CR. It reads the mark of the game and gives it as
/// [`Inbound::Mark`], and it gives no prompt or other message. Any other IAC sequence stays in
/// the line.
#[derive(Debug, Default)]
pub struct LineFramer {
    partial: Vec<u8>,
    side: Side,
    held: Held,
}

impl LineFramer {
    /// A framer for the client side, with no partial line.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A framer for the game side: it answers the mark of a client.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the fake game is the only user")
    )]
    pub(crate) fn game() -> Self {
        Self {
            side: Side::Game,
            ..Self::default()
        }
    }

    fn step(&mut self, byte: u8, out: &mut Vec<Inbound>) {
        match (self.held, byte) {
            (Held::Nothing, IAC) => self.held = Held::Iac,
            (Held::Nothing, byte) => self.push(byte, out),
            (Held::Iac, verb) if self.side.reads_verb(verb) => self.held = Held::Verb(verb),
            (Held::Verb(_), opt::TIMING_MARK) => {
                self.held = Held::Nothing;
                out.push(self.side.on_mark());
            }
            (Held::Iac, byte) => {
                self.held = Held::Nothing;
                self.push(IAC, out);
                self.step(byte, out);
            }
            (Held::Verb(verb), byte) => {
                self.held = Held::Nothing;
                self.push(IAC, out);
                self.push(verb, out);
                self.step(byte, out);
            }
        }
    }

    fn push(&mut self, byte: u8, out: &mut Vec<Inbound>) {
        match byte {
            b'\n' => out.push(self.take_line()),
            b'\r' => {}
            byte => {
                self.partial.push(byte);
                if self.partial.len() == MAX_LINE {
                    out.push(self.take_line());
                }
            }
        }
    }

    fn take_line(&mut self) -> Inbound {
        let line = std::mem::take(&mut self.partial);
        Inbound::Line(String::from_utf8_lossy(&line).into_owned())
    }
}

impl Framer for LineFramer {
    fn feed(&mut self, bytes: &[u8]) -> Vec<Inbound> {
        let mut out = Vec::new();
        for &byte in bytes {
            self.step(byte, &mut out);
        }
        out
    }

    fn encode(&self, line: &str) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(line.len() + 2);
        bytes.extend_from_slice(line.as_bytes());
        bytes.extend_from_slice(b"\r\n");
        bytes
    }

    fn mark(&self) -> Vec<u8> {
        match self.side {
            Side::Client => vec![IAC, DO, opt::TIMING_MARK],
            Side::Game => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const TM: u8 = opt::TIMING_MARK;
    const FAVORED: [u8; 5] = [IAC, WILL, WONT, DO, TM];

    fn lines(inbound: Vec<Inbound>) -> Vec<String> {
        inbound
            .into_iter()
            .map(|item| match item {
                Inbound::Line(line) => line,
                other => panic!("not a line: {other:?}"),
            })
            .collect()
    }

    #[test]
    fn line_framer_splits_on_lf() {
        assert_eq!(
            lines(LineFramer::new().feed(b"one\ntwo\n\n")),
            ["one", "two", ""]
        );
    }

    #[test]
    fn line_framer_drops_cr() {
        assert_eq!(
            lines(LineFramer::new().feed(b"one\r\nt\rwo\r\n")),
            ["one", "two"]
        );
    }

    #[test]
    fn line_framer_keeps_a_partial_line() {
        let mut framer = LineFramer::new();
        assert_eq!(framer.feed(b"hel"), []);
        assert_eq!(lines(framer.feed(b"lo\nwor")), ["hello"]);
        assert_eq!(lines(framer.feed(b"ld\n")), ["world"]);
    }

    #[test]
    fn line_framer_cuts_a_line_at_the_bound() {
        let mut framer = LineFramer::new();
        let long = vec![b'a'; MAX_LINE + 3];
        let cut = lines(framer.feed(&long));
        assert_eq!(cut.len(), 1);
        assert_eq!(cut[0].len(), MAX_LINE);
        assert_eq!(lines(framer.feed(b"\n")), ["aaa"]);
    }

    #[test]
    fn line_framer_encodes_crlf() {
        assert_eq!(LineFramer::new().encode("hi"), b"hi\r\n");
    }

    /// Feeds `bytes` to `framer` in reads that end at `points`.
    fn feed_split(mut framer: LineFramer, bytes: &[u8], points: &[usize]) -> Vec<Inbound> {
        let mut points = points.to_vec();
        points.sort_unstable();
        let mut out = Vec::new();
        let mut start = 0;
        for point in points.into_iter().chain([bytes.len()]) {
            out.extend(framer.feed(&bytes[start..point]));
            start = point;
        }
        out
    }

    fn framer_for(game: bool) -> LineFramer {
        if game {
            LineFramer::game()
        } else {
            LineFramer::new()
        }
    }

    fn favored_byte() -> impl Strategy<Value = u8> {
        prop::sample::select(FAVORED.to_vec())
    }

    fn line_of(bytes: &[u8]) -> Inbound {
        Inbound::Line(String::from_utf8_lossy(bytes).into_owned())
    }

    #[test]
    fn line_framer_reads_mark_answer() {
        for verb in [WILL, WONT] {
            let bytes = [b"one\n".as_slice(), &[IAC, verb, TM], b"two\n"].concat();
            assert_eq!(
                LineFramer::new().feed(&bytes),
                [line_of(b"one"), Inbound::Mark, line_of(b"two")]
            );
        }
    }

    #[test]
    fn line_framer_keeps_other_iac_bytes() {
        let cases = [
            (LineFramer::new(), [IAC, WILL, 5]),
            (LineFramer::game(), [IAC, WILL, 5]),
            (LineFramer::new(), [IAC, DO, TM]),
            (LineFramer::game(), [IAC, WILL, TM]),
            (LineFramer::game(), [IAC, WONT, TM]),
        ];
        for (framer, sequence) in cases {
            let bytes = [b"a".as_slice(), &sequence, b"b\n"].concat();
            let points: Vec<usize> = (1..bytes.len()).collect();
            assert_eq!(
                feed_split(framer, &bytes, &points),
                [line_of(&bytes[..bytes.len() - 1])]
            );
        }
    }

    #[test]
    fn line_framer_mark_at_the_bound() {
        for verb in [WILL, WONT] {
            let data = vec![b'a'; MAX_LINE - 1];
            let bytes = [data.as_slice(), &[IAC, verb, TM], b"\n"].concat();
            for points in [vec![], vec![MAX_LINE], vec![MAX_LINE + 1]] {
                assert_eq!(
                    feed_split(LineFramer::new(), &bytes, &points),
                    [Inbound::Mark, line_of(&data)]
                );
            }
        }
    }

    #[test]
    fn line_framer_near_miss_at_the_bound() {
        let data = vec![b'a'; MAX_LINE - 1];
        let bytes = [data.as_slice(), &[IAC, WILL, 5], b"\n"].concat();
        let mut first = "a".repeat(MAX_LINE - 1);
        first.push('\u{FFFD}');
        let expected = [Inbound::Line(first), Inbound::Line("\u{FFFD}\u{5}".into())];
        for points in [vec![], vec![MAX_LINE], vec![MAX_LINE + 1]] {
            assert_eq!(feed_split(LineFramer::new(), &bytes, &points), expected);
        }
    }

    #[test]
    fn line_framer_game_answers_do_timing_mark() {
        let bytes = [b"one\n".as_slice(), &[IAC, DO, TM], b"two\n"].concat();
        assert_eq!(
            LineFramer::game().feed(&bytes),
            [
                line_of(b"one"),
                Inbound::Reply(vec![IAC, WILL, TM]),
                line_of(b"two")
            ]
        );
    }

    #[test]
    fn line_framer_client_marks_with_do_timing_mark() {
        assert_eq!(LineFramer::new().mark(), [IAC, DO, TM]);
    }

    #[test]
    fn line_framer_game_sends_no_mark() {
        assert_eq!(LineFramer::game().mark(), []);
    }

    proptest! {
        #[test]
        fn line_framer_ignores_read_splits(
            game in any::<bool>(),
            bytes in prop::collection::vec(
                prop_oneof![Just(b'\n'), Just(b'\r'), any::<u8>(), favored_byte()],
                0..200,
            ),
            cuts in prop::collection::vec(any::<prop::sample::Index>(), 0..10),
        ) {
            let whole = framer_for(game).feed(&bytes);
            let points: Vec<usize> = cuts.iter().map(|cut| cut.index(bytes.len() + 1)).collect();
            prop_assert_eq!(feed_split(framer_for(game), &bytes, &points), whole);
        }

        #[test]
        fn line_framer_lines_stay_in_the_bound(
            game in any::<bool>(),
            tail in prop::collection::vec(favored_byte(), 0..12),
            cuts in prop::collection::vec(any::<prop::sample::Index>(), 0..6),
        ) {
            let prefix = MAX_LINE - 4;
            let bytes = [vec![b'a'; prefix], tail, vec![b'\n']].concat();
            let points: Vec<usize> = cuts.iter().map(|cut| cut.index(bytes.len() + 1)).collect();
            let verbs: &[u8] = if game { &[DO] } else { &[WILL, WONT] };
            let answers = bytes
                .windows(3)
                .filter(|window| window[0] == IAC && verbs.contains(&window[1]) && window[2] == TM)
                .count();
            let mut seen = 0;
            let mut chars = 0;
            for item in feed_split(framer_for(game), &bytes, &points) {
                match item {
                    Inbound::Line(line) => {
                        prop_assert!(line.chars().count() <= MAX_LINE);
                        chars += line.chars().count();
                    }
                    Inbound::Mark | Inbound::Reply(_) => seen += 1,
                    other => prop_assert!(false, "unexpected {other:?}"),
                }
            }
            prop_assert_eq!(seen, answers);
            prop_assert_eq!(chars, bytes.len() - 1 - 3 * answers);
        }

        #[test]
        fn line_framer_marks_are_transparent(
            stream in prop::collection::vec(
                prop_oneof![Just(b'\n'), any::<u8>().prop_map(|byte| byte.min(IAC - 1))],
                0..100,
            ),
            inserts in prop::collection::vec((any::<prop::sample::Index>(), any::<bool>()), 0..5),
            cuts in prop::collection::vec(any::<prop::sample::Index>(), 0..6),
        ) {
            let inserts: Vec<(usize, u8)> = inserts
                .iter()
                .map(|(at, wont)| (at.index(stream.len() + 1), if *wont { WONT } else { WILL }))
                .collect();
            let mut bytes = Vec::new();
            for position in 0..=stream.len() {
                for (_, verb) in inserts.iter().filter(|(at, _)| *at == position) {
                    bytes.extend([IAC, *verb, TM]);
                }
                bytes.extend(stream.get(position));
            }
            let points: Vec<usize> = cuts.iter().map(|cut| cut.index(bytes.len() + 1)).collect();

            let mut expected = Vec::new();
            let mut placed = 0;
            let mut plain = LineFramer::new().feed(&stream).into_iter();
            for (end, _) in stream.iter().enumerate().filter(|(_, byte)| **byte == b'\n') {
                let due = inserts.iter().filter(|(at, _)| *at <= end).count();
                expected.extend(std::iter::repeat_n(Inbound::Mark, due - placed));
                placed = due;
                expected.extend(plain.next());
            }
            expected.extend(std::iter::repeat_n(Inbound::Mark, inserts.len() - placed));

            prop_assert_eq!(feed_split(LineFramer::new(), &bytes, &points), expected);
        }
    }
}
