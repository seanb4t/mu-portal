//! The transcript format: a text file with one record for each read or write.
//!
//! ```text
//! # family: pennmush
//! < 0.012 fffb56fffb18
//! > 0.013 fffd56
//! ```
//!
//! `<` is a read from the server and `>` a write to it. The number is seconds since the
//! connect. The hex holds the raw bytes of that one read, so the read boundaries of the real
//! server are kept. Lines that start with `#` are notes.

use std::fmt::Write as _;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    FromServer,
    ToServer,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub direction: Direction,
    pub seconds: f64,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct Transcript {
    pub notes: Vec<String>,
    pub records: Vec<Record>,
}

impl Transcript {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut transcript = Self::default();
        for (n, line) in text.lines().enumerate() {
            let line = line.trim_end();
            if line.is_empty() {
                continue;
            }
            if let Some(note) = line.strip_prefix('#') {
                transcript.notes.push(note.trim().to_owned());
                continue;
            }
            let fail = |what: &str| format!("line {}: {what}", n + 1);
            let mut parts = line.splitn(3, ' ');
            let direction = match parts.next() {
                Some("<") => Direction::FromServer,
                Some(">") => Direction::ToServer,
                _ => return Err(fail("expected < or >")),
            };
            let seconds = parts
                .next()
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| fail("bad time"))?;
            let bytes =
                decode_hex(parts.next().unwrap_or_default()).ok_or_else(|| fail("bad hex"))?;
            transcript.records.push(Record {
                direction,
                seconds,
                bytes,
            });
        }
        Ok(transcript)
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        for note in &self.notes {
            let _ = writeln!(out, "# {note}");
        }
        for r in &self.records {
            let arrow = match r.direction {
                Direction::FromServer => '<',
                Direction::ToServer => '>',
            };
            let _ = writeln!(out, "{arrow} {:.3} {}", r.seconds, encode_hex(&r.bytes));
        }
        out
    }

    /// The reads from the server, in order.
    pub fn server_reads(&self) -> impl Iterator<Item = &[u8]> {
        self.records
            .iter()
            .filter(|r| r.direction == Direction::FromServer)
            .map(|r| r.bytes.as_slice())
    }
}

pub fn encode_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok())
        .collect()
}
