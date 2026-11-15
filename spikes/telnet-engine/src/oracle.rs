//! A common view of the events of this engine and of libmudtelnet, for differential tests.
//!
//! libmudtelnet is the reference. Its caller must inflate MCCP2 data itself, so
//! [`reference`] does that the way a client on top of libmudtelnet does.

use bytes::Bytes;
use libmudtelnet::Parser;
use libmudtelnet::compatibility::{CompatibilityEntry, CompatibilityTable};
use libmudtelnet::events::TelnetEvents;
use miniz_oxide::inflate::stream::{InflateState, inflate};
use miniz_oxide::{DataFormat, MZError, MZFlush, MZStatus};

use crate::{Engine, Event, Policy, Side, opt};

/// The events that both parsers report, with adjacent data merged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Common {
    Data(Vec<u8>),
    Command(u8),
    Subnegotiation(u8, Vec<u8>),
    CompressionStarted,
    CompressionEnded,
    Error,
}

fn push(out: &mut Vec<Common>, item: Common) {
    if let (Common::Data(new), Some(Common::Data(last))) = (&item, out.last_mut()) {
        last.extend_from_slice(new);
        return;
    }
    if item != Common::Data(Vec::new()) {
        out.push(item);
    }
}

/// The options that the differential streams negotiate. All of them are on before the stream
/// starts, in both parsers.
pub const OPTIONS: [u8; 4] = [opt::GMCP, opt::MSSP, opt::TTYPE, opt::MCCP2];

pub fn engine_policy() -> Policy {
    Policy::none()
        .accept(Side::Local, &OPTIONS)
        .accept(Side::Remote, &OPTIONS)
}

/// Runs the reads through this engine. MCCP2 is on before the first read.
pub fn ours(reads: &[&[u8]]) -> Vec<Common> {
    let mut engine = Engine::new(engine_policy());
    engine.receive(&[crate::IAC, crate::WILL, opt::MCCP2]);
    while engine.poll_event().is_some() {}
    let mut out = Vec::new();
    for read in reads {
        engine.receive(read);
        while let Some(event) = engine.poll_event() {
            let item = match event {
                Event::Data(d) => Common::Data(d),
                Event::Command(c) => Common::Command(c),
                Event::Subnegotiation(o, p) => Common::Subnegotiation(o, p),
                Event::CompressionStarted => Common::CompressionStarted,
                Event::CompressionEnded => Common::CompressionEnded,
                Event::Error(_) => Common::Error,
                Event::Enabled(..) | Event::Disabled(..) => continue,
            };
            push(&mut out, item);
        }
    }
    out
}

/// Runs the reads through libmudtelnet, with every option of [`OPTIONS`] on at both ends.
pub fn reference(reads: &[&[u8]]) -> Vec<Common> {
    let mut table = CompatibilityTable::new();
    for o in OPTIONS {
        table.set_option(o, CompatibilityEntry::new(true, true, true, true));
    }
    let mut parser = Parser::with_support(table);
    let mut inflater: Option<Box<InflateState>> = None;
    let mut out = Vec::new();
    for read in reads {
        let mut pending: Vec<Bytes> = Vec::new();
        if inflater.is_some() {
            pending.push(Bytes::copy_from_slice(read));
        } else {
            feed(&mut parser, read, &mut out, &mut pending, &mut inflater);
        }
        while let Some(compressed) = pending.pop() {
            let mut rest: &[u8] = &compressed;
            let mut buf = [0u8; 16 * 1024];
            while let Some(state) = inflater.as_mut() {
                let r = inflate(state, rest, &mut buf, MZFlush::None);
                rest = &rest[r.bytes_consumed..];
                let plain = buf[..r.bytes_written].to_vec();
                feed_compressed(&mut parser, &plain, &mut out);
                match r.status {
                    Ok(MZStatus::StreamEnd) => {
                        inflater = None;
                        push(&mut out, Common::CompressionEnded);
                        let tail = rest.to_vec();
                        feed(&mut parser, &tail, &mut out, &mut pending, &mut inflater);
                        break;
                    }
                    Ok(_) if rest.is_empty() && r.bytes_written < buf.len() => break,
                    Ok(_) => {}
                    Err(MZError::Buf) if rest.is_empty() => break,
                    Err(_) => {
                        inflater = None;
                        push(&mut out, Common::Error);
                    }
                }
            }
        }
    }
    out
}

/// Feeds inflated bytes. A second MCCP2 start inside the compressed stream does nothing.
fn feed_compressed(parser: &mut Parser, bytes: &[u8], out: &mut Vec<Common>) {
    let mut ignored = Vec::new();
    let mut already = Some(InflateState::new_boxed(DataFormat::Zlib));
    feed(parser, bytes, out, &mut ignored, &mut already);
}

fn feed(
    parser: &mut Parser,
    bytes: &[u8],
    out: &mut Vec<Common>,
    pending: &mut Vec<Bytes>,
    inflater: &mut Option<Box<InflateState>>,
) {
    if bytes.is_empty() {
        return;
    }
    for event in parser.receive(bytes) {
        match event {
            TelnetEvents::DataReceive(d) => push(out, Common::Data(d.to_vec())),
            TelnetEvents::IAC(c) => push(out, Common::Command(c.command)),
            TelnetEvents::Subnegotiation(s) if s.option == opt::MCCP2 && s.buffer.is_empty() => {}
            TelnetEvents::Subnegotiation(s) => {
                push(out, Common::Subnegotiation(s.option, s.buffer.to_vec()));
            }
            TelnetEvents::DecompressImmediate(rest) => {
                if inflater.is_none() {
                    *inflater = Some(InflateState::new_boxed(DataFormat::Zlib));
                    push(out, Common::CompressionStarted);
                    pending.push(rest);
                }
            }
            TelnetEvents::Negotiation(_) | TelnetEvents::DataSend(_) => {}
        }
    }
}

/// A piece of a generated server stream.
#[derive(Clone, Debug)]
pub enum Token {
    Data(Vec<u8>),
    /// GA, EOR or NOP: the commands that libmudtelnet reads as two bytes.
    Prompt(u8),
    Subnegotiation(u8, Vec<u8>),
    /// IAC SB MCCP2 IAC SE. Every token after it is compressed in one zlib stream.
    StartCompression,
    /// Ends the zlib stream. Every token after it is plain again.
    EndCompression,
    /// IAC IAC inside data.
    EscapedIac,
    /// A two-byte command other than GA, EOR and NOP, for example AYT or DM.
    OtherCommand(u8),
    /// A subnegotiation whose payload holds an escaped IAC.
    SubnegotiationWithIac(u8, Vec<u8>),
}

/// Encodes tokens as the bytes a server sends.
pub fn encode(tokens: &[Token]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut compressing: Option<Vec<u8>> = None;
    for token in tokens {
        let target = compressing.as_mut().unwrap_or(&mut out);
        match token {
            Token::Data(d) => target.extend_from_slice(d),
            Token::Prompt(c) | Token::OtherCommand(c) => {
                target.extend_from_slice(&[crate::IAC, *c])
            }
            Token::EscapedIac => target.extend_from_slice(&[crate::IAC, crate::IAC]),
            Token::Subnegotiation(o, p) | Token::SubnegotiationWithIac(o, p) => {
                target.extend_from_slice(&[crate::IAC, crate::SB, *o]);
                for &b in p {
                    target.push(b);
                    if b == crate::IAC {
                        target.push(crate::IAC);
                    }
                }
                target.extend_from_slice(&[crate::IAC, crate::SE]);
            }
            Token::StartCompression => {
                if compressing.is_none() {
                    out.extend_from_slice(&[
                        crate::IAC,
                        crate::SB,
                        opt::MCCP2,
                        crate::IAC,
                        crate::SE,
                    ]);
                    compressing = Some(Vec::new());
                }
            }
            Token::EndCompression => {
                if let Some(plain) = compressing.take() {
                    out.extend(miniz_oxide::deflate::compress_to_vec_zlib(&plain, 6));
                }
            }
        }
    }
    if let Some(plain) = compressing.take() {
        out.extend(compress_open(&plain));
    }
    out
}

/// Compresses bytes as the start of a zlib stream that stays open, with a sync flush at the
/// end, the way a MUD server sends output while compression is on.
pub fn compress_open(plain: &[u8]) -> Vec<u8> {
    use miniz_oxide::deflate::core::{CompressorOxide, create_comp_flags_from_zip_params};
    use miniz_oxide::deflate::stream::deflate;

    let mut compressor = CompressorOxide::new(create_comp_flags_from_zip_params(6, 1, 0));
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    let mut rest = plain;
    loop {
        let r = deflate(&mut compressor, rest, &mut buf, MZFlush::Sync);
        rest = &rest[r.bytes_consumed..];
        out.extend_from_slice(&buf[..r.bytes_written]);
        if rest.is_empty() && r.bytes_written < buf.len() {
            return out;
        }
    }
}

/// The events a correct parser reports for the tokens, with no chunking.
pub fn expected(tokens: &[Token]) -> Vec<Common> {
    let mut out = Vec::new();
    let mut compressing = false;
    for token in tokens {
        match token {
            Token::Data(d) => push(&mut out, Common::Data(d.clone())),
            Token::EscapedIac => push(&mut out, Common::Data(vec![crate::IAC])),
            Token::Prompt(c) | Token::OtherCommand(c) => push(&mut out, Common::Command(*c)),
            Token::Subnegotiation(o, p) | Token::SubnegotiationWithIac(o, p) => {
                push(&mut out, Common::Subnegotiation(*o, p.clone()));
            }
            Token::StartCompression if !compressing => {
                compressing = true;
                push(&mut out, Common::CompressionStarted);
            }
            Token::EndCompression if compressing => {
                compressing = false;
                push(&mut out, Common::CompressionEnded);
            }
            Token::StartCompression | Token::EndCompression => {}
        }
    }
    out
}

/// Splits bytes at the given cut points, which are taken modulo the length and sorted.
pub fn split<'a>(bytes: &'a [u8], cuts: &[usize]) -> Vec<&'a [u8]> {
    if bytes.is_empty() {
        return Vec::new();
    }
    let mut points: Vec<usize> = cuts.iter().map(|c| c % bytes.len()).collect();
    points.push(0);
    points.push(bytes.len());
    points.sort_unstable();
    points.dedup();
    points.windows(2).map(|w| &bytes[w[0]..w[1]]).collect()
}
