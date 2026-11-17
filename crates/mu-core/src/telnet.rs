//! The telnet protocol of a game: its codes, and a parser that does no I/O.
//!
//! Every code of telnet lives in this file. The submodules name each code by its constant.

pub mod parser;
pub mod q;

pub use parser::{Event, Parser, Policy, ProtocolError, Side};

/// End of record (RFC 885).
pub const EOR: u8 = 239;
/// End of subnegotiation parameters.
pub const SE: u8 = 240;
/// No operation.
pub const NOP: u8 = 241;
/// The data stream part of a synch.
pub const DM: u8 = 242;
/// Break.
pub const BRK: u8 = 243;
/// Interrupt process.
pub const IP: u8 = 244;
/// Abort output.
pub const AO: u8 = 245;
/// Are you there.
pub const AYT: u8 = 246;
/// Erase character.
pub const EC: u8 = 247;
/// Erase line.
pub const EL: u8 = 248;
/// Go ahead.
pub const GA: u8 = 249;
/// Start of a subnegotiation.
pub const SB: u8 = 250;
/// The sender performs, or wants to perform, an option.
pub const WILL: u8 = 251;
/// The sender does not perform an option.
pub const WONT: u8 = 252;
/// The sender asks the peer to perform an option.
pub const DO: u8 = 253;
/// The sender asks the peer to stop an option.
pub const DONT: u8 = 254;
/// Interpret as command. `IAC IAC` is the data byte 255.
pub const IAC: u8 = 255;

/// The option codes.
pub mod opt {
    /// Echo (RFC 857).
    pub const ECHO: u8 = 1;
    /// Suppress go ahead (RFC 858).
    pub const SGA: u8 = 3;
    /// Status (RFC 859).
    pub const STATUS: u8 = 5;
    /// Timing mark (RFC 860).
    pub const TIMING_MARK: u8 = 6;
    /// Terminal type (RFC 1091).
    pub const TTYPE: u8 = 24;
    /// End of record (RFC 885).
    pub const EOR: u8 = 25;
    /// Window size (RFC 1073).
    pub const NAWS: u8 = 31;
    /// Character set (RFC 2066).
    pub const CHARSET: u8 = 42;
    /// MUD server data protocol.
    pub const MSDP: u8 = 69;
    /// MUD server status protocol.
    pub const MSSP: u8 = 70;
    /// MUD client compression protocol, version 2.
    pub const MCCP2: u8 = 86;
    /// MUD extension protocol.
    pub const MXP: u8 = 91;
    /// Generic MUD communication protocol.
    pub const GMCP: u8 = 201;
}

/// The commands of a TTYPE subnegotiation (RFC 1091).
pub mod ttype {
    /// The answer: the name of a terminal type follows.
    pub const IS: u8 = 0;
    /// The request for the next terminal type.
    pub const SEND: u8 = 1;
}

/// The commands of a CHARSET subnegotiation (RFC 2066).
pub mod charset {
    /// A request with a list of character sets.
    pub const REQUEST: u8 = 1;
    /// The answer that accepts one character set of the list.
    pub const ACCEPTED: u8 = 2;
    /// The answer that accepts none of the list.
    pub const REJECTED: u8 = 3;
    /// The optional prefix of a request, before its version byte.
    pub const TTABLE: &[u8] = b"[TTABLE ]";
}

/// The bytes of a mark: `IAC DO TIMING-MARK`.
pub const MARK: [u8; 3] = [IAC, DO, opt::TIMING_MARK];

/// The largest subnegotiation payload that the parser keeps.
pub const MAX_SUBNEGOTIATION: usize = 1 << 20;

/// `bytes` with each byte 255 doubled, as telnet sends it in data and in a subnegotiation.
#[must_use]
pub fn escape(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for &byte in bytes {
        out.push(byte);
        if byte == IAC {
            out.push(IAC);
        }
    }
    out
}
