//! The codes of telnet (RFC 854) that the engine uses.

pub const IAC: u8 = 255;
pub const DO: u8 = 253;
pub const WONT: u8 = 252;
pub const WILL: u8 = 251;

pub mod opt {
    pub const TIMING_MARK: u8 = 6;
}
