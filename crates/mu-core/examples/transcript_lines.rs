//! Prints the game lines of a telnet transcript, one per line, as the telnet framer gives them.
//!
//! Each `<` record of the transcript holds the hex of one read from the game. The other records
//! are notes or the bytes of the client, and the framer does not see them.
//!
//! `cargo run -p mu-core --example transcript_lines -- <transcript>...`

use std::io::Write;
use std::{env, fs, io};

use mu_core::{Framer, Inbound, TelnetFramer, TelnetSettings};

#[path = "../tests/support/transcript.rs"]
mod transcript;
use transcript::transcript_reads;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut out = io::stdout().lock();
    for path in env::args().skip(1) {
        let mut framer = TelnetFramer::new(TelnetSettings::default());
        for read in transcript_reads(&fs::read_to_string(&path)?) {
            for inbound in framer.feed(&read) {
                if let Inbound::Line(line) = inbound {
                    writeln!(out, "{line}")?;
                }
            }
        }
    }
    Ok(())
}
