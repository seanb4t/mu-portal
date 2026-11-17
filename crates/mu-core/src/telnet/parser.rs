//! A telnet parser with no I/O inside it.
//!
//! [`Parser::receive`] takes the bytes of one read of the game. The parser queues [`Event`]s
//! and the bytes to send back. It negotiates options with the Q method, reads
//! subnegotiations, and inflates MCCP2, which can start inside a read.

use std::collections::VecDeque;

use miniz_oxide::inflate::stream::{InflateState, inflate};
use miniz_oxide::{DataFormat, MZError, MZFlush, MZStatus};

use super::q::{self, Q};
use super::{DO, DONT, IAC, MAX_SUBNEGOTIATION, SB, SE, WILL, WONT, escape, opt};

/// The end of the connection that performs an option.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Side {
    /// The client performs the option: the game sent DO, and the client sent WILL.
    Local,
    /// The game performs the option: the game sent WILL, and the client sent DO.
    Remote,
}

/// A break of the protocol by the game.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ProtocolError {
    /// `IAC` and a byte other than SE or IAC inside a subnegotiation. The parser drops the
    /// payload and reads the byte as a command.
    #[error("the subnegotiation of option {option} has no IAC SE")]
    UnterminatedSubnegotiation {
        /// The option of the dropped subnegotiation.
        option: u8,
    },
    /// `IAC SB IAC`: a subnegotiation with no option.
    #[error("a subnegotiation has no option")]
    EmptySubnegotiation,
    /// A payload longer than [`MAX_SUBNEGOTIATION`].
    #[error("the subnegotiation of option {option} is longer than {MAX_SUBNEGOTIATION} bytes")]
    SubnegotiationTooLong {
        /// The option of the dropped subnegotiation.
        option: u8,
    },
    /// The game answered a request that the client never sent.
    #[error("the game answered a request for option {option} that the client did not send")]
    UnrequestedAnswer {
        /// The side of the option.
        side: Side,
        /// The option.
        option: u8,
    },
    /// The compressed stream is not valid zlib. The parser sends `DONT MCCP2` and drops each
    /// byte until the game answers `WONT MCCP2`.
    #[error("the compressed stream is not valid zlib: {0}")]
    BadCompressedStream(String),
}

impl ProtocolError {
    /// A stable code for the error, for a caller that acts on one kind of error.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnterminatedSubnegotiation { .. } => "subnegotiation-unterminated",
            Self::EmptySubnegotiation => "subnegotiation-empty",
            Self::SubnegotiationTooLong { .. } => "subnegotiation-too-long",
            Self::UnrequestedAnswer { .. } => "unrequested-answer",
            Self::BadCompressedStream(_) => "bad-compressed-stream",
        }
    }
}

/// What the parser read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    /// Data bytes, with `IAC IAC` read as one byte 255. CR, LF and NUL pass through.
    Data(Vec<u8>),
    /// A two-byte command, for example GA, EOR, NOP or AYT, with its code.
    Command(u8),
    /// An option turned on.
    Enabled(Side, u8),
    /// An option turned off.
    Disabled(Side, u8),
    /// A subnegotiation: the option and the payload.
    Subnegotiation(u8, Vec<u8>),
    /// The game answered a mark with `WILL TIMING-MARK` or `WONT TIMING-MARK`.
    Mark,
    /// The compressed stream started.
    CompressionStarted,
    /// The compressed stream ended. The bytes after it are plain.
    CompressionEnded,
    /// The game broke the protocol.
    Error(ProtocolError),
}

/// The options that the client accepts on each side.
#[derive(Clone, Debug)]
pub struct Policy {
    local: [bool; 256],
    remote: [bool; 256],
}

impl Policy {
    /// A policy that accepts no option.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            local: [false; 256],
            remote: [false; 256],
        }
    }

    /// This policy, and also `options` on `side`.
    #[must_use]
    pub fn accept(mut self, side: Side, options: &[u8]) -> Self {
        let table = match side {
            Side::Local => &mut self.local,
            Side::Remote => &mut self.remote,
        };
        for &option in options {
            table[usize::from(option)] = true;
        }
        self
    }

    /// Whether the policy accepts `option` on `side`.
    #[must_use]
    pub const fn accepts(&self, side: Side, option: u8) -> bool {
        match side {
            Side::Local => self.local[option as usize],
            Side::Remote => self.remote[option as usize],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Data,
    Iac,
    Negotiation(u8),
    SbOption,
    Sb,
    SbIac,
}

/// A subnegotiation that is open.
#[derive(Debug, Default)]
struct Payload {
    option: u8,
    bytes: Vec<u8>,
    too_long: bool,
}

/// The telnet parser of one connection.
pub struct Parser {
    policy: Policy,
    local: [Q; 256],
    remote: [Q; 256],
    state: State,
    payload: Payload,
    data: Vec<u8>,
    /// Each event, with the length of the output when the parser read it.
    events: VecDeque<(Event, usize)>,
    output: Vec<u8>,
    /// Where a reply goes in the output: right after the bytes that the parser sent before
    /// the last polled event. `None` puts a reply at the end.
    cursor: Option<usize>,
    inflater: Option<Box<InflateState>>,
    /// After a bad compressed stream: the count of bytes of `IAC WONT MCCP2` that the parser
    /// found so far. `None` while the parser reads normally.
    recovery: Option<usize>,
}

impl std::fmt::Debug for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parser")
            .field("state", &self.state)
            .field("compressed", &self.inflater.is_some())
            .field("recovery", &self.recovery)
            .finish_non_exhaustive()
    }
}

impl Parser {
    /// A parser with every option off.
    #[must_use]
    pub fn new(policy: Policy) -> Self {
        Self {
            policy,
            local: [Q::No; 256],
            remote: [Q::No; 256],
            state: State::Data,
            payload: Payload::default(),
            data: Vec::new(),
            events: VecDeque::new(),
            output: Vec::new(),
            cursor: None,
            inflater: None,
            recovery: None,
        }
    }

    /// Reads the bytes of one read. The data of the read ends as one event at the end.
    pub fn receive(&mut self, mut input: &[u8]) {
        self.cursor = None;
        while !input.is_empty() {
            if let Some(found) = self.recovery {
                input = self.recover(found, input);
            } else if self.inflater.is_some() {
                input = self.inflate(input);
            } else {
                let used = self.parse(input, true);
                input = &input[used..];
            }
        }
        self.flush_data();
    }

    /// The next event, in the order that the parser read it. A reply that the caller sends
    /// before the next poll goes after the bytes that the parser sent before this event.
    pub fn poll_event(&mut self) -> Option<Event> {
        let (event, position) = self.events.pop_front()?;
        self.cursor = Some(position);
        Some(event)
    }

    /// The bytes to send to the game, and clears them.
    pub fn take_output(&mut self) -> Vec<u8> {
        let taken = std::mem::take(&mut self.output);
        for (_, position) in &mut self.events {
            *position = position.saturating_sub(taken.len());
        }
        self.cursor = None;
        taken
    }

    /// Whether `option` is on for `side`.
    #[must_use]
    pub fn is_enabled(&self, side: Side, option: u8) -> bool {
        self.q(side, option) == Q::Yes
    }

    /// Asks the game to turn an option on or off.
    pub fn request(&mut self, side: Side, option: u8, enable: bool) {
        let (next, send) = q::request(self.q(side, option), enable);
        self.set_q(side, option, next);
        if let Some(positive) = send {
            self.send_negotiation(side, option, positive);
        }
    }

    /// Queues a subnegotiation, with each byte 255 of `payload` doubled.
    pub fn send_subnegotiation(&mut self, option: u8, payload: &[u8]) {
        self.write(&[&[IAC, SB, option][..], &escape(payload), &[IAC, SE]].concat());
    }

    /// Adds `bytes` to the output at the cursor, or at the end.
    fn write(&mut self, bytes: &[u8]) {
        let Some(at) = self.cursor else {
            // Data read before this reply comes before it in the order of events.
            self.flush_data();
            self.output.extend_from_slice(bytes);
            return;
        };
        self.output.splice(at..at, bytes.iter().copied());
        self.cursor = Some(at + bytes.len());
        for (_, position) in &mut self.events {
            *position += bytes.len();
        }
    }

    fn q(&self, side: Side, option: u8) -> Q {
        match side {
            Side::Local => self.local[usize::from(option)],
            Side::Remote => self.remote[usize::from(option)],
        }
    }

    fn set_q(&mut self, side: Side, option: u8, next: Q) {
        match side {
            Side::Local => self.local[usize::from(option)] = next,
            Side::Remote => self.remote[usize::from(option)] = next,
        }
    }

    fn send_negotiation(&mut self, side: Side, option: u8, positive: bool) {
        let verb = match (side, positive) {
            (Side::Local, true) => WILL,
            (Side::Local, false) => WONT,
            (Side::Remote, true) => DO,
            (Side::Remote, false) => DONT,
        };
        self.write(&[IAC, verb, option]);
    }

    fn emit(&mut self, event: Event) {
        self.flush_data();
        self.events.push_back((event, self.output.len()));
    }

    fn flush_data(&mut self) {
        if !self.data.is_empty() {
            let data = std::mem::take(&mut self.data);
            self.events
                .push_back((Event::Data(data), self.output.len()));
        }
    }

    /// Parses plain bytes and gives the count that it used. If `may_compress` is set, it
    /// stops right after the `IAC SE` that starts MCCP2.
    fn parse(&mut self, input: &[u8], may_compress: bool) -> usize {
        for (index, &byte) in input.iter().enumerate() {
            match self.state {
                State::Data => match byte {
                    IAC => self.state = State::Iac,
                    _ => self.data.push(byte),
                },
                State::Iac => self.command(byte),
                State::Negotiation(verb) => {
                    self.state = State::Data;
                    self.negotiate(verb, byte);
                }
                State::SbOption => match byte {
                    IAC => {
                        self.state = State::Iac;
                        self.emit(Event::Error(ProtocolError::EmptySubnegotiation));
                    }
                    option => {
                        self.payload = Payload {
                            option,
                            ..Payload::default()
                        };
                        self.state = State::Sb;
                    }
                },
                State::Sb => match byte {
                    IAC => self.state = State::SbIac,
                    _ => self.payload_push(byte),
                },
                State::SbIac => match byte {
                    SE => {
                        self.state = State::Data;
                        if self.end_subnegotiation(may_compress) {
                            return index + 1;
                        }
                    }
                    IAC => {
                        self.state = State::Sb;
                        self.payload_push(IAC);
                    }
                    _ => {
                        let option = self.payload.option;
                        self.payload = Payload::default();
                        self.emit(Event::Error(ProtocolError::UnterminatedSubnegotiation {
                            option,
                        }));
                        self.command(byte);
                    }
                },
            }
        }
        input.len()
    }

    fn command(&mut self, byte: u8) {
        self.state = State::Data;
        match byte {
            IAC => self.data.push(IAC),
            WILL | WONT | DO | DONT => self.state = State::Negotiation(byte),
            SB => self.state = State::SbOption,
            _ => self.emit(Event::Command(byte)),
        }
    }

    fn payload_push(&mut self, byte: u8) {
        if self.payload.bytes.len() < MAX_SUBNEGOTIATION {
            self.payload.bytes.push(byte);
        } else {
            self.payload.too_long = true;
        }
    }

    /// Gives the finished subnegotiation. Returns true if it starts a compressed stream.
    fn end_subnegotiation(&mut self, may_compress: bool) -> bool {
        let Payload {
            option,
            bytes,
            too_long,
        } = std::mem::take(&mut self.payload);
        if too_long {
            self.emit(Event::Error(ProtocolError::SubnegotiationTooLong {
                option,
            }));
            return false;
        }
        let starts = option == opt::MCCP2
            && bytes.is_empty()
            && may_compress
            && self.is_enabled(Side::Remote, option);
        if starts {
            self.inflater = Some(InflateState::new_boxed(DataFormat::Zlib));
            self.emit(Event::CompressionStarted);
        } else {
            self.emit(Event::Subnegotiation(option, bytes));
        }
        starts
    }

    fn negotiate(&mut self, verb: u8, option: u8) {
        if option == opt::TIMING_MARK {
            match verb {
                WILL | WONT => self.emit(Event::Mark),
                DO => self.send_negotiation(Side::Local, option, false),
                _ => {}
            }
            return;
        }
        let side = match verb {
            WILL | WONT => Side::Remote,
            _ => Side::Local,
        };
        let state = self.q(side, option);
        let step = match verb {
            WILL | DO => q::on_enable(state, self.policy.accepts(side, option)),
            _ => q::on_disable(state),
        };
        self.set_q(side, option, step.next);
        if let Some(positive) = step.reply {
            self.send_negotiation(side, option, positive);
        }
        if step.violation {
            self.emit(Event::Error(ProtocolError::UnrequestedAnswer {
                side,
                option,
            }));
        }
        match step.report {
            Some(true) => self.emit(Event::Enabled(side, option)),
            Some(false) => self.emit(Event::Disabled(side, option)),
            None => {}
        }
    }

    /// Inflates compressed bytes and parses the result. Gives the input after the end of the
    /// stream, which is empty while the stream goes on.
    fn inflate<'a>(&mut self, input: &'a [u8]) -> &'a [u8] {
        let mut rest = input;
        let mut out = vec![0; 16 * 1024];
        loop {
            let Some(state) = self.inflater.as_mut() else {
                return rest;
            };
            let result = inflate(state, rest, &mut out, MZFlush::None);
            rest = &rest[result.bytes_consumed..];
            let written = result.bytes_written;
            if written > 0 {
                self.parse(&out[..written], false);
            }
            match result.status {
                Ok(MZStatus::StreamEnd) => {
                    self.inflater = None;
                    self.emit(Event::CompressionEnded);
                    return rest;
                }
                Ok(_) if rest.is_empty() && written < out.len() => return rest,
                Err(MZError::Buf) if rest.is_empty() => return rest,
                Ok(_) | Err(MZError::Buf) => {}
                Err(error) => {
                    self.inflater = None;
                    self.state = State::Data;
                    self.payload = Payload::default();
                    self.recovery = Some(0);
                    self.emit(Event::Error(ProtocolError::BadCompressedStream(format!(
                        "{error:?}"
                    ))));
                    self.request(Side::Remote, opt::MCCP2, false);
                    return rest;
                }
            }
        }
    }

    /// Drops bytes after a bad compressed stream until `IAC WONT MCCP2`, which it reads as a
    /// negotiation. Gives the input after it, which is plain again.
    fn recover<'a>(&mut self, mut found: usize, input: &'a [u8]) -> &'a [u8] {
        const ANSWER: [u8; 3] = [IAC, WONT, opt::MCCP2];
        for (index, &byte) in input.iter().enumerate() {
            found = if byte == ANSWER[found] {
                found + 1
            } else {
                usize::from(byte == IAC)
            };
            if found == ANSWER.len() {
                self.recovery = None;
                self.negotiate(WONT, opt::MCCP2);
                return &input[index + 1..];
            }
        }
        self.recovery = Some(found);
        &[]
    }
}
