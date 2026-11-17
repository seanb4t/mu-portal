//! Framers: bytes of the game in, lines and prompts out, and a line in, bytes out.

use crate::telnet::{
    self, DO, EOR, Event, GA, IAC, Parser, Policy, Side, WILL, WONT, charset, opt, ttype,
};

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

/// The partial line of a framer. It splits on LF, drops each CR and cuts a run of bytes with
/// no LF at [`MAX_LINE`]. A full partial line followed by LF is one line.
#[derive(Debug, Default)]
pub(crate) struct LineBuffer {
    partial: Vec<u8>,
}

impl LineBuffer {
    /// Takes one data byte, and gives the bytes of the line that it ends or cuts.
    pub(crate) fn push(&mut self, byte: u8) -> Option<Vec<u8>> {
        match byte {
            b'\n' => Some(self.take()),
            b'\r' => None,
            byte => {
                let cut = (self.partial.len() == MAX_LINE).then(|| self.take());
                self.partial.push(byte);
                cut
            }
        }
    }

    /// The bytes of the partial line.
    pub(crate) fn partial(&self) -> &[u8] {
        &self.partial
    }

    /// The partial line, which this clears.
    pub(crate) fn take(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.partial)
    }
}

/// Which end of the connection a [`LineFramer`] reads for.
#[derive(Clone, Copy, Debug, Default)]
enum Role {
    /// Reads the game, and sends marks to it.
    #[default]
    Client,
    /// Reads the client, and answers its marks.
    Game,
}

impl Role {
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

/// Splits on LF and drops each CR. It reads `IAC IAC` as one escaped data byte. It reads the mark of
/// the game and gives it as [`Inbound::Mark`], and it gives no prompt or other message. Any
/// other IAC sequence stays in the line.
#[derive(Debug, Default)]
pub struct LineFramer {
    buffer: LineBuffer,
    role: Role,
    held: Held,
}

impl LineFramer {
    /// A framer for the client side, with no partial line.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A framer for the game side: it answers the mark of a client.
    pub(crate) fn game() -> Self {
        Self {
            role: Role::Game,
            ..Self::default()
        }
    }

    fn step(&mut self, byte: u8, out: &mut Vec<Inbound>) {
        match (self.held, byte) {
            (Held::Nothing, IAC) => self.held = Held::Iac,
            (Held::Nothing, byte) => self.push(byte, out),
            (Held::Iac, IAC) => {
                self.held = Held::Nothing;
                self.push(IAC, out);
            }
            (Held::Iac, verb) if self.role.reads_verb(verb) => self.held = Held::Verb(verb),
            (Held::Verb(_), opt::TIMING_MARK) => {
                self.held = Held::Nothing;
                out.push(self.role.on_mark());
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
        if let Some(line) = self.buffer.push(byte) {
            out.push(Inbound::Line(String::from_utf8_lossy(&line).into_owned()));
        }
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
        match self.role {
            Role::Client => telnet::MARK.to_vec(),
            Role::Game => Vec::new(),
        }
    }
}

/// The character set of the text of a world.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Charset {
    /// UTF-8. Each invalid sequence becomes a replacement character.
    #[default]
    Utf8,
    /// ISO-8859-1. Each byte is the character with the same number.
    Latin1,
}

impl Charset {
    /// The name that a CHARSET answer uses.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Latin1 => "ISO-8859-1",
        }
    }

    fn decode(self, bytes: &[u8]) -> String {
        match self {
            Self::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
            Self::Latin1 => bytes.iter().copied().map(char::from).collect(),
        }
    }

    fn encode(self, text: &str) -> Vec<u8> {
        match self {
            Self::Utf8 => text.as_bytes().to_vec(),
            Self::Latin1 => text
                .chars()
                .map(|c| u8::try_from(c).unwrap_or(b'?'))
                .collect(),
        }
    }
}

/// The settings of a [`TelnetFramer`]: what the client answers, and the options it accepts.
#[derive(Clone, Debug)]
pub struct TelnetSettings {
    /// The character set of the text, which a CHARSET answer offers.
    pub charset: Charset,
    /// The width of the window, which NAWS sends.
    pub columns: u16,
    /// The height of the window, which NAWS sends.
    pub rows: u16,
    /// The options that the client accepts on each side.
    pub policy: Policy,
}

impl Default for TelnetSettings {
    /// UTF-8, a window of 120 by 40, and the options of a MU client.
    fn default() -> Self {
        Self {
            charset: Charset::Utf8,
            columns: 120,
            rows: 40,
            policy: Policy::none()
                .accept(Side::Local, &[opt::TTYPE, opt::NAWS, opt::CHARSET])
                .accept(
                    Side::Remote,
                    &[
                        opt::ECHO,
                        opt::SGA,
                        opt::EOR,
                        opt::CHARSET,
                        opt::MSSP,
                        opt::MCCP2,
                        opt::GMCP,
                    ],
                ),
        }
    }
}

/// The terminal types that the client gives, in order. It repeats the last one.
const TERMINAL_TYPES: [&str; 3] = ["MU-PORTAL", "XTERM-256COLOR", "MTTS 269"];

/// A framer for a telnet game. It reads the protocol with [`Parser`], answers the terminal
/// type, the window size and the character set, and gives lines, prompts, marks, GMCP
/// messages and protocol errors.
#[derive(Debug)]
pub struct TelnetFramer {
    parser: Parser,
    charset: Charset,
    columns: u16,
    rows: u16,
    buffer: LineBuffer,
    /// The last data byte was a CR, so a NUL right after it is dropped.
    after_cr: bool,
    /// The count of terminal types that the client gave.
    terminal_types: usize,
}

impl TelnetFramer {
    /// A framer with every option off and no partial line.
    #[must_use]
    pub fn new(settings: TelnetSettings) -> Self {
        Self {
            parser: Parser::new(settings.policy),
            charset: settings.charset,
            columns: settings.columns,
            rows: settings.rows,
            buffer: LineBuffer::default(),
            after_cr: false,
            terminal_types: 0,
        }
    }

    /// Adds data bytes to the partial line, and gives each line that they end.
    fn data(&mut self, bytes: &[u8], out: &mut Vec<Inbound>) {
        for &byte in bytes {
            let after_cr = std::mem::replace(&mut self.after_cr, byte == b'\r');
            if after_cr && byte == 0 {
                continue;
            }
            if let Some(line) = self.buffer.push(byte) {
                out.push(Inbound::Line(self.charset.decode(&line)));
            }
        }
    }

    /// Gives the partial line as a prompt, if it is not empty, and clears it.
    fn prompt(&mut self, out: &mut Vec<Inbound>) {
        let text = self.buffer.take();
        if !text.is_empty() {
            out.push(Inbound::Prompt(self.charset.decode(&text)));
        }
    }

    fn subnegotiation(&mut self, option: u8, payload: Vec<u8>, out: &mut Vec<Inbound>) {
        match (option, payload.first()) {
            (opt::GMCP, _) => out.push(Inbound::Message {
                name: "GMCP".to_owned(),
                body: payload,
            }),
            (opt::TTYPE, Some(&ttype::SEND)) if self.parser.is_enabled(Side::Local, opt::TTYPE) => {
                let name = TERMINAL_TYPES[self.terminal_types.min(TERMINAL_TYPES.len() - 1)];
                self.terminal_types += 1;
                let answer = [&[ttype::IS], name.as_bytes()].concat();
                self.parser.send_subnegotiation(opt::TTYPE, &answer);
            }
            (opt::CHARSET, Some(&charset::REQUEST)) => {
                let answer = self.charset_answer(&payload[1..]);
                self.parser.send_subnegotiation(opt::CHARSET, &answer);
            }
            _ => {}
        }
    }

    /// The answer to the list of a CHARSET request, with its optional `[TTABLE ]` prefix.
    fn charset_answer(&self, request: &[u8]) -> Vec<u8> {
        let list = match request.strip_prefix(charset::TTABLE) {
            Some(rest) => rest.get(1..).unwrap_or_default(),
            None => request,
        };
        let accepted = list.split_first().and_then(|(separator, names)| {
            names
                .split(|byte| byte == separator)
                .find(|name| name.eq_ignore_ascii_case(self.charset.name().as_bytes()))
        });
        match accepted {
            Some(name) => [&[charset::ACCEPTED], name].concat(),
            None => vec![charset::REJECTED],
        }
    }

    fn window_size(&mut self) {
        let [width_high, width_low] = self.columns.to_be_bytes();
        let [height_high, height_low] = self.rows.to_be_bytes();
        self.parser
            .send_subnegotiation(opt::NAWS, &[width_high, width_low, height_high, height_low]);
    }
}

impl Framer for TelnetFramer {
    fn feed(&mut self, bytes: &[u8]) -> Vec<Inbound> {
        self.parser.receive(bytes);
        let mut out = Vec::new();
        let mut added = false;
        while let Some(event) = self.parser.poll_event() {
            match event {
                Event::Data(data) => {
                    added = true;
                    self.data(&data, &mut out);
                }
                Event::Command(GA | EOR) => {
                    added = false;
                    self.prompt(&mut out);
                }
                Event::Enabled(Side::Local, opt::NAWS) => self.window_size(),
                Event::Subnegotiation(option, payload) => {
                    self.subnegotiation(option, payload, &mut out);
                }
                Event::Mark => out.push(Inbound::Mark),
                Event::Error(error) => out.push(Inbound::Message {
                    name: "telnet".to_owned(),
                    body: format!("{} {error}", error.code()).into_bytes(),
                }),
                Event::Command(_)
                | Event::Enabled(..)
                | Event::Disabled(..)
                | Event::CompressionStarted
                | Event::CompressionEnded => {}
            }
        }
        let reply = self.parser.take_output();
        if !reply.is_empty() {
            out.push(Inbound::Reply(reply));
        }
        if added && !self.buffer.partial().is_empty() {
            out.push(Inbound::Prompt(self.charset.decode(self.buffer.partial())));
        }
        out
    }

    fn encode(&self, line: &str) -> Vec<u8> {
        let mut bytes = telnet::escape(&self.charset.encode(line));
        bytes.extend_from_slice(b"\r\n");
        bytes
    }

    fn mark(&self) -> Vec<u8> {
        telnet::MARK.to_vec()
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
    fn line_framer_line_of_exactly_the_bound() {
        for end in [b"\n".as_slice(), b"\r\n"] {
            let bytes = [vec![b'a'; MAX_LINE], end.to_vec()].concat();
            assert_eq!(
                lines(LineFramer::new().feed(&bytes)),
                ["a".repeat(MAX_LINE)]
            );
        }
    }

    #[test]
    fn line_buffer_splits_on_lf_and_drops_cr() {
        let mut buffer = LineBuffer::default();
        let found: Vec<Vec<u8>> = b"a\rb\nc"
            .iter()
            .filter_map(|byte| buffer.push(*byte))
            .collect();
        assert_eq!(found, [b"ab".to_vec()]);
        assert_eq!(buffer.partial(), b"c");
    }

    #[test]
    fn line_buffer_cuts_on_the_byte_after_the_bound() {
        let mut buffer = LineBuffer::default();
        for _ in 0..MAX_LINE {
            assert_eq!(buffer.push(b'a'), None);
        }
        assert_eq!(buffer.push(b'\r'), None);
        assert_eq!(buffer.push(b'b'), Some(vec![b'a'; MAX_LINE]));
        assert_eq!(buffer.take(), b"b");
        assert_eq!(buffer.partial(), b"");
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

    /// The marks and the data bytes of `bytes`, with `IAC IAC` read as one data byte.
    fn count_marks(bytes: &[u8], game: bool) -> (usize, usize) {
        let verbs: &[u8] = if game { &[DO] } else { &[WILL, WONT] };
        let (mut marks, mut data, mut rest) = (0, 0, bytes);
        while !rest.is_empty() {
            let used = match rest {
                [IAC, verb, TM, ..] if verbs.contains(verb) => {
                    marks += 1;
                    3
                }
                [IAC, IAC, ..] => {
                    data += 1;
                    2
                }
                _ => {
                    data += 1;
                    1
                }
            };
            rest = &rest[used..];
        }
        (marks, data)
    }

    fn favored_byte() -> impl Strategy<Value = u8> {
        prop::sample::select(FAVORED.to_vec())
    }

    /// A byte of a stream: LF, CR, any byte, and the favored bytes.
    fn stream_byte() -> impl Strategy<Value = u8> {
        prop_oneof![Just(b'\n'), Just(b'\r'), any::<u8>(), favored_byte()]
    }

    fn line_of(bytes: &[u8]) -> Inbound {
        Inbound::Line(String::from_utf8_lossy(bytes).into_owned())
    }

    /// The lines that end in `stream`, which holds data bytes only: split at LF, with no CR.
    fn data_lines(stream: &[u8]) -> Vec<Inbound> {
        stream
            .split_inclusive(|byte| *byte == b'\n')
            .filter(|piece| piece.ends_with(b"\n"))
            .map(|piece| {
                let data: Vec<u8> = piece[..piece.len() - 1]
                    .iter()
                    .copied()
                    .filter(|byte| *byte != b'\r')
                    .collect();
                line_of(&data)
            })
            .collect()
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
            (LineFramer::new(), [IAC, WILL, opt::STATUS]),
            (LineFramer::game(), [IAC, WILL, opt::STATUS]),
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
    fn line_framer_reads_escaped_iac() {
        let answer = Inbound::Reply(vec![IAC, WILL, TM]);
        // Each case: the framer, the bytes read, the data bytes of the line, what the framer answers.
        let cases = [
            (LineFramer::new(), vec![IAC, IAC], vec![IAC], vec![]),
            (LineFramer::game(), vec![IAC, IAC], vec![IAC], vec![]),
            (
                LineFramer::new(),
                vec![IAC, IAC, WILL, TM],
                vec![IAC, WILL, TM],
                vec![],
            ),
            (
                LineFramer::game(),
                vec![IAC, IAC, DO, TM],
                vec![IAC, DO, TM],
                vec![],
            ),
            (
                LineFramer::new(),
                vec![IAC, IAC, IAC, WONT, TM],
                vec![IAC],
                vec![Inbound::Mark],
            ),
            (
                LineFramer::game(),
                vec![IAC, IAC, IAC, DO, TM],
                vec![IAC],
                vec![answer],
            ),
        ];
        for (framer, sequence, data, answers) in cases {
            let bytes = [b"a".as_slice(), &sequence, b"b\n"].concat();
            let line = [b"a".as_slice(), &data, b"b"].concat();
            let expected = [answers, vec![line_of(&line)]].concat();
            let points: Vec<usize> = (1..bytes.len()).collect();
            assert_eq!(feed_split(framer, &bytes, &points), expected);
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
        let bytes = [data.as_slice(), &[IAC, WILL, opt::STATUS], b"\n"].concat();
        let mut first = "a".repeat(MAX_LINE - 1);
        first.push('\u{FFFD}');
        let expected = [
            Inbound::Line(first),
            Inbound::Line(format!("\u{FFFD}{}", char::from(opt::STATUS))),
        ];
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
            bytes in prop::collection::vec(stream_byte(), 0..200),
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
            let (answers, data) = count_marks(&bytes[..bytes.len() - 1], game);
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
            prop_assert_eq!(chars, data);
        }

        #[test]
        fn line_framer_marks_are_transparent(
            stream in prop::collection::vec(stream_byte(), 0..100),
            inserts in prop::collection::vec((any::<prop::sample::Index>(), any::<bool>()), 0..5),
            cuts in prop::collection::vec(any::<prop::sample::Index>(), 0..6),
        ) {
            let inserts: Vec<(usize, u8)> = inserts
                .iter()
                .map(|(at, wont)| (at.index(stream.len() + 1), if *wont { WONT } else { WILL }))
                .collect();
            // The stream holds data bytes. On the wire, each byte 255 is `IAC IAC`.
            let mut bytes = Vec::new();
            for position in 0..=stream.len() {
                for (_, verb) in inserts.iter().filter(|(at, _)| *at == position) {
                    bytes.extend([IAC, *verb, TM]);
                }
                match stream.get(position) {
                    Some(&IAC) => bytes.extend([IAC, IAC]),
                    Some(&byte) => bytes.push(byte),
                    None => {}
                }
            }
            let points: Vec<usize> = cuts.iter().map(|cut| cut.index(bytes.len() + 1)).collect();

            let mut expected = Vec::new();
            let mut placed = 0;
            let mut complete = data_lines(&stream).into_iter();
            for (end, _) in stream.iter().enumerate().filter(|(_, byte)| **byte == b'\n') {
                let due = inserts.iter().filter(|(at, _)| *at <= end).count();
                expected.extend(std::iter::repeat_n(Inbound::Mark, due - placed));
                placed = due;
                expected.extend(complete.next());
            }
            expected.extend(std::iter::repeat_n(Inbound::Mark, inserts.len() - placed));

            prop_assert_eq!(feed_split(LineFramer::new(), &bytes, &points), expected);
        }
    }
}
