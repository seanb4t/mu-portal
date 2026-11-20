//! Prints the game lines of a telnet transcript, one per line, as the telnet framer gives them.
//!
//! Each `<` row of the transcript holds the hex of one read from the game. The other rows are
//! comments or the bytes of the client, and the framer does not see them.
//!
//! `cargo run -p mu-core --example transcript_lines -- <transcript>...`

use std::io::Write;
use std::{env, fs, io};

use mu_core::{Framer, Inbound, TelnetFramer, TelnetSettings};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut out = io::stdout().lock();
    for path in env::args().skip(1) {
        let mut framer = TelnetFramer::new(TelnetSettings::default());
        for row in fs::read_to_string(&path)?.lines() {
            let Some(read) = row.strip_prefix("< ") else {
                continue;
            };
            let hex = read.split_once(' ').map_or(read, |(_, hex)| hex);
            for inbound in framer.feed(&hex::decode(hex)?) {
                if let Inbound::Line(line) = inbound {
                    writeln!(out, "{line}")?;
                }
            }
        }
    }
    Ok(())
}
