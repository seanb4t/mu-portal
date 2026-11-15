//! Spike: a telnet engine with no I/O inside it.
//!
//! [`Engine::receive`] takes the bytes of one read from the server. The engine queues
//! [`Event`]s for the caller and the bytes to send back. It handles option negotiation
//! (RFC 1143), subnegotiation and MCCP2 decompression, including a compressed stream that
//! starts in the middle of a read.

pub mod client;
pub mod oracle;
pub mod q;
pub mod transcript;

use std::collections::VecDeque;

use miniz_oxide::inflate::stream::{InflateState, inflate};
use miniz_oxide::{DataFormat, MZError, MZFlush, MZStatus};

use crate::q::{Q, Step};

pub const IAC: u8 = 255;
pub const DONT: u8 = 254;
pub const DO: u8 = 253;
pub const WONT: u8 = 252;
pub const WILL: u8 = 251;
pub const SB: u8 = 250;
pub const GA: u8 = 249;
pub const NOP: u8 = 241;
pub const SE: u8 = 240;
pub const EOR: u8 = 239;

pub mod opt {
    pub const ECHO: u8 = 1;
    pub const SGA: u8 = 3;
    pub const TTYPE: u8 = 24;
    pub const EOR: u8 = 25;
    pub const NAWS: u8 = 31;
    pub const LINEMODE: u8 = 34;
    pub const NEW_ENVIRON: u8 = 39;
    pub const CHARSET: u8 = 42;
    pub const MSDP: u8 = 69;
    pub const MSSP: u8 = 70;
    pub const MCCP2: u8 = 86;
    pub const MCCP3: u8 = 87;
    pub const MSP: u8 = 90;
    pub const MXP: u8 = 91;
    pub const ATCP: u8 = 200;
    pub const GMCP: u8 = 201;
}

/// The largest subnegotiation payload the engine keeps. A longer one is dropped.
pub const MAX_SUBNEGOTIATION: usize = 1 << 20;

/// Which end of the connection performs an option.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Side {
    /// This client performs the option: the server sent DO, and the client sent WILL.
    Local,
    /// The server performs the option: the server sent WILL, and the client sent DO.
    Remote,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolError {
    /// IAC followed by a byte other than SE or IAC inside a subnegotiation. The engine drops
    /// the subnegotiation and reads the byte as a command.
    UnterminatedSubnegotiation { option: u8 },
    /// IAC SB IAC SE: a subnegotiation with no option byte.
    EmptySubnegotiation,
    /// A subnegotiation longer than [`MAX_SUBNEGOTIATION`].
    SubnegotiationTooLong { option: u8 },
    /// The server answered a request that this client never made.
    UnrequestedAnswer { side: Side, option: u8 },
    /// The compressed stream is not valid zlib. Compression stops, and the rest of the read is lost.
    Inflate(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    /// Application bytes, with IAC IAC read as one 255 byte. CR, LF and NUL pass through.
    Data(Vec<u8>),
    /// A two-byte command such as GA, EOR, NOP or AYT.
    Command(u8),
    Enabled(Side, u8),
    Disabled(Side, u8),
    Subnegotiation(u8, Vec<u8>),
    CompressionStarted,
    /// The server ended the zlib stream. Bytes after the end are plain again.
    CompressionEnded,
    Error(ProtocolError),
}

/// The options this client accepts on each side.
#[derive(Clone)]
pub struct Policy {
    local: [bool; 256],
    remote: [bool; 256],
}

impl Policy {
    pub fn none() -> Self {
        Self {
            local: [false; 256],
            remote: [false; 256],
        }
    }

    pub fn accept(mut self, side: Side, options: &[u8]) -> Self {
        let table = match side {
            Side::Local => &mut self.local,
            Side::Remote => &mut self.remote,
        };
        for &o in options {
            table[usize::from(o)] = true;
        }
        self
    }

    fn accepts(&self, side: Side, option: u8) -> bool {
        match side {
            Side::Local => self.local[usize::from(option)],
            Side::Remote => self.remote[usize::from(option)],
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

pub struct Engine {
    policy: Policy,
    local: [Q; 256],
    remote: [Q; 256],
    state: State,
    sb_option: u8,
    sb_buffer: Vec<u8>,
    sb_overflow: bool,
    data: Vec<u8>,
    events: VecDeque<Event>,
    output: Vec<u8>,
    inflater: Option<Box<InflateState>>,
}

impl Engine {
    pub fn new(policy: Policy) -> Self {
        Self {
            policy,
            local: [Q::No; 256],
            remote: [Q::No; 256],
            state: State::Data,
            sb_option: 0,
            sb_buffer: Vec::new(),
            sb_overflow: false,
            data: Vec::new(),
            events: VecDeque::new(),
            output: Vec::new(),
            inflater: None,
        }
    }

    /// Takes the bytes of one read. Data is flushed as one event at the end of the call, so a
    /// prompt with no line end reaches the caller at once.
    pub fn receive(&mut self, mut input: &[u8]) {
        while !input.is_empty() {
            if self.inflater.is_some() {
                input = self.inflate(input);
            } else {
                let used = self.parse(input, true);
                input = &input[used..];
            }
        }
        self.flush_data();
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    pub fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.output)
    }

    pub fn is_enabled(&self, side: Side, option: u8) -> bool {
        self.q(side, option) == Q::Yes
    }

    pub fn is_compressed(&self) -> bool {
        self.inflater.is_some()
    }

    /// Asks the server to enable or disable an option.
    pub fn request(&mut self, side: Side, option: u8, enable: bool) {
        let (next, send) = q::request(self.q(side, option), enable);
        self.set_q(side, option, next);
        if let Some(positive) = send {
            self.send_negotiation(side, option, positive);
        }
    }

    /// Queues application bytes, with each 255 byte doubled.
    pub fn send_data(&mut self, data: &[u8]) {
        for &b in data {
            self.output.push(b);
            if b == IAC {
                self.output.push(IAC);
            }
        }
    }

    pub fn send_subnegotiation(&mut self, option: u8, payload: &[u8]) {
        self.output.extend_from_slice(&[IAC, SB, option]);
        self.send_data(payload);
        self.output.extend_from_slice(&[IAC, SE]);
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
        let command = match (side, positive) {
            (Side::Local, true) => WILL,
            (Side::Local, false) => WONT,
            (Side::Remote, true) => DO,
            (Side::Remote, false) => DONT,
        };
        self.output.extend_from_slice(&[IAC, command, option]);
    }

    fn emit(&mut self, event: Event) {
        self.flush_data();
        self.events.push_back(event);
    }

    fn flush_data(&mut self) {
        if !self.data.is_empty() {
            let data = std::mem::take(&mut self.data);
            self.events.push_back(Event::Data(data));
        }
    }

    /// Parses plain bytes. Stops after IAC SE of an MCCP2 start when `may_compress` is true,
    /// and returns the count of bytes it used.
    fn parse(&mut self, input: &[u8], may_compress: bool) -> usize {
        for (i, &b) in input.iter().enumerate() {
            match self.state {
                State::Data => match b {
                    IAC => self.state = State::Iac,
                    _ => self.data.push(b),
                },
                State::Iac => self.command(b),
                State::Negotiation(command) => {
                    self.state = State::Data;
                    self.negotiate(command, b);
                }
                State::SbOption => match b {
                    IAC => {
                        self.state = State::Iac;
                        self.emit(Event::Error(ProtocolError::EmptySubnegotiation));
                    }
                    _ => {
                        self.sb_option = b;
                        self.sb_buffer.clear();
                        self.sb_overflow = false;
                        self.state = State::Sb;
                    }
                },
                State::Sb => match b {
                    IAC => self.state = State::SbIac,
                    _ => self.sb_push(b),
                },
                State::SbIac => match b {
                    SE => {
                        self.state = State::Data;
                        if self.end_subnegotiation() && may_compress {
                            return i + 1;
                        }
                    }
                    IAC => {
                        self.state = State::Sb;
                        self.sb_push(IAC);
                    }
                    _ => {
                        let option = self.sb_option;
                        self.emit(Event::Error(ProtocolError::UnterminatedSubnegotiation {
                            option,
                        }));
                        self.command(b);
                    }
                },
            }
        }
        input.len()
    }

    fn command(&mut self, b: u8) {
        self.state = State::Data;
        match b {
            IAC => self.data.push(IAC),
            WILL | WONT | DO | DONT => self.state = State::Negotiation(b),
            SB => self.state = State::SbOption,
            _ => self.emit(Event::Command(b)),
        }
    }

    fn sb_push(&mut self, b: u8) {
        if self.sb_buffer.len() < MAX_SUBNEGOTIATION {
            self.sb_buffer.push(b);
        } else {
            self.sb_overflow = true;
        }
    }

    /// Reports the finished subnegotiation. Returns true when it starts MCCP2.
    fn end_subnegotiation(&mut self) -> bool {
        let option = self.sb_option;
        if self.sb_overflow {
            self.sb_buffer.clear();
            self.emit(Event::Error(ProtocolError::SubnegotiationTooLong {
                option,
            }));
            return false;
        }
        let payload = std::mem::take(&mut self.sb_buffer);
        if option == opt::MCCP2 && payload.is_empty() && self.is_enabled(Side::Remote, option) {
            if self.inflater.is_none() {
                self.inflater = Some(InflateState::new_boxed(DataFormat::Zlib));
                self.emit(Event::CompressionStarted);
                return true;
            }
            return false;
        }
        self.emit(Event::Subnegotiation(option, payload));
        false
    }

    fn negotiate(&mut self, command: u8, option: u8) {
        let side = match command {
            WILL | WONT => Side::Remote,
            _ => Side::Local,
        };
        let state = self.q(side, option);
        let step: Step = match command {
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

    /// Inflates compressed bytes and parses the result. Returns the input after the end of the
    /// zlib stream, which is empty while the stream goes on.
    fn inflate<'a>(&mut self, input: &'a [u8]) -> &'a [u8] {
        let mut rest = input;
        let mut out = [0u8; 16 * 1024];
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
                Ok(_) => {}
                Err(MZError::Buf) if rest.is_empty() => return rest,
                Err(e) => {
                    self.inflater = None;
                    self.emit(Event::Error(ProtocolError::Inflate(format!("{e:?}"))));
                    return &[];
                }
            }
        }
    }
}
