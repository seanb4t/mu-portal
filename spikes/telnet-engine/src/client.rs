//! The answers of a MU client to the server, on top of the engine.
//!
//! The recorder uses it so that each server sees a client that offers terminal type (with
//! MTTS), window size, UTF-8, GMCP, MSSP, EOR and MCCP2.

use crate::{Engine, Event, Policy, Side, opt};

const TTYPE_IS: u8 = 0;
const TTYPE_SEND: u8 = 1;
const CHARSET_REQUEST: u8 = 1;
const CHARSET_ACCEPTED: u8 = 2;
const CHARSET_REJECTED: u8 = 3;

/// MTTS bits: ANSI (1), UTF-8 (4), 256 colors (8) and truecolor (256).
const MTTS: &str = "MTTS 269";
const TERMINAL_NAMES: [&str; 3] = ["MU-PORTAL", "XTERM-256COLOR", MTTS];

pub const COLUMNS: u16 = 120;
pub const ROWS: u16 = 40;

pub fn policy() -> Policy {
    Policy::none()
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
        )
}

pub struct Client {
    pub engine: Engine,
    ttype_index: usize,
    charset: Vec<u8>,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    /// A client that accepts UTF-8 in a CHARSET request.
    pub fn new() -> Self {
        Self::with_charset("UTF-8")
    }

    /// A client that accepts the named character set in a CHARSET request, or rejects the
    /// request when the server does not offer it.
    pub fn with_charset(name: &str) -> Self {
        Self {
            engine: Engine::new(policy()),
            ttype_index: 0,
            charset: name.as_bytes().to_vec(),
        }
    }

    /// Feeds one read to the engine, answers what the client must answer, and returns the
    /// events in order.
    pub fn receive(&mut self, bytes: &[u8]) -> Vec<Event> {
        self.engine.receive(bytes);
        let mut events = Vec::new();
        while let Some(event) = self.engine.poll_event() {
            self.answer(&event);
            events.push(event);
        }
        events
    }

    pub fn take_output(&mut self) -> Vec<u8> {
        self.engine.take_output()
    }

    fn answer(&mut self, event: &Event) {
        match event {
            Event::Enabled(Side::Local, opt::NAWS) => self.send_naws(),
            Event::Enabled(Side::Remote, opt::GMCP) => {
                self.engine.send_subnegotiation(
                    opt::GMCP,
                    br#"Core.Hello {"client":"mu-portal","version":"0.0.0"}"#,
                );
                self.engine.send_subnegotiation(
                    opt::GMCP,
                    br#"Core.Supports.Set ["Char 1","Room 1","Comm.Channel 1"]"#,
                );
            }
            Event::Subnegotiation(opt::TTYPE, payload) if payload.first() == Some(&TTYPE_SEND) => {
                let name = TERMINAL_NAMES[self.ttype_index.min(TERMINAL_NAMES.len() - 1)];
                self.ttype_index += 1;
                let mut reply = vec![TTYPE_IS];
                reply.extend_from_slice(name.as_bytes());
                self.engine.send_subnegotiation(opt::TTYPE, &reply);
            }
            Event::Subnegotiation(opt::CHARSET, payload)
                if payload.first() == Some(&CHARSET_REQUEST) =>
            {
                let reply = match offered(&payload[1..], &self.charset) {
                    Some(name) => [&[CHARSET_ACCEPTED][..], name].concat(),
                    None => vec![CHARSET_REJECTED],
                };
                self.engine.send_subnegotiation(opt::CHARSET, &reply);
            }
            _ => {}
        }
    }

    fn send_naws(&mut self) {
        let [c1, c0] = COLUMNS.to_be_bytes();
        let [r1, r0] = ROWS.to_be_bytes();
        self.engine
            .send_subnegotiation(opt::NAWS, &[c1, c0, r1, r0]);
    }
}

/// Finds a character set in a CHARSET REQUEST list (RFC 2066). The list can start with the
/// prefix `[TTABLE ]` (with a space) and a one-byte version. After them, the first byte is the
/// separator.
fn offered<'a>(list: &'a [u8], wanted: &[u8]) -> Option<&'a [u8]> {
    let list = list
        .strip_prefix(b"[TTABLE ]")
        .map_or(list, |rest| rest.get(1..).unwrap_or_default());
    let (&separator, names) = list.split_first()?;
    names
        .split(|&b| b == separator)
        .find(|name| name.eq_ignore_ascii_case(wanted))
}

#[cfg(test)]
mod tests {
    use super::offered;

    #[test]
    fn finds_the_charset_in_a_plain_request() {
        assert_eq!(offered(b";ISO-8859-1;UTF-8", b"UTF-8"), Some(&b"UTF-8"[..]));
    }

    #[test]
    fn skips_the_ttable_prefix_and_its_version() {
        assert_eq!(
            offered(b"[TTABLE ]\x01;ISO-8859-1;UTF-8", b"UTF-8"),
            Some(&b"UTF-8"[..])
        );
    }
}
