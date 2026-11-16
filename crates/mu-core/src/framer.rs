//! Framers: bytes of the game in, lines and prompts out, and a line in, bytes out.

/// What a framer gives for the bytes of the game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inbound {
    /// A line, with no line end.
    Line(String),
    /// A prompt: text with no line end that the game flushed.
    Prompt(String),
    /// Bytes to write back to the game, for example a negotiation reply.
    Reply(Vec<u8>),
    /// A message of a side protocol, for example GMCP.
    Message {
        /// The name of the side protocol.
        name: String,
        /// The body of the message.
        body: Vec<u8>,
    },
}

/// The framing of one connection. #52 adds the telnet framer.
pub trait Framer: Send + 'static {
    /// Takes the bytes of one read and returns what they complete.
    fn feed(&mut self, bytes: &[u8]) -> Vec<Inbound>;

    /// The bytes of one line to the game.
    fn encode(&self, line: &str) -> Vec<u8>;
}

/// Splits on LF and drops each CR. It never gives a prompt or a message.
#[derive(Debug, Default)]
pub struct LineFramer {
    partial: Vec<u8>,
}

impl LineFramer {
    /// A framer with no partial line.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Framer for LineFramer {
    fn feed(&mut self, bytes: &[u8]) -> Vec<Inbound> {
        let mut lines = Vec::new();
        for &byte in bytes {
            match byte {
                b'\n' => {
                    let line = std::mem::take(&mut self.partial);
                    lines.push(Inbound::Line(String::from_utf8_lossy(&line).into_owned()));
                }
                b'\r' => {}
                byte => self.partial.push(byte),
            }
        }
        lines
    }

    fn encode(&self, line: &str) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(line.len() + 2);
        bytes.extend_from_slice(line.as_bytes());
        bytes.extend_from_slice(b"\r\n");
        bytes
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

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
    fn line_framer_encodes_crlf() {
        assert_eq!(LineFramer::new().encode("hi"), b"hi\r\n");
    }

    proptest! {
        #[test]
        fn line_framer_ignores_read_splits(
            bytes in prop::collection::vec(prop_oneof![Just(b'\n'), Just(b'\r'), any::<u8>()], 0..200),
            cuts in prop::collection::vec(any::<prop::sample::Index>(), 0..10),
        ) {
            let whole = LineFramer::new().feed(&bytes);
            let mut points: Vec<usize> = cuts.iter().map(|cut| cut.index(bytes.len() + 1)).collect();
            points.sort_unstable();
            let mut framer = LineFramer::new();
            let mut split = Vec::new();
            let mut start = 0;
            for point in points.into_iter().chain([bytes.len()]) {
                split.extend(framer.feed(&bytes[start..point]));
                start = point;
            }
            prop_assert_eq!(split, whole);
        }
    }
}
