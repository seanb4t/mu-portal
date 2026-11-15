//! MCCP2 decompression, with the compressed stream starting inside a read.

use miniz_oxide::deflate::compress_to_vec_zlib;
use telnet_engine::{Engine, Event, GA, IAC, Policy, ProtocolError, SB, SE, Side, WILL, opt};

const START: [u8; 5] = [IAC, SB, opt::MCCP2, IAC, SE];

fn compressing_engine() -> Engine {
    let mut e = Engine::new(Policy::none().accept(Side::Remote, &[opt::MCCP2]));
    e.receive(&[IAC, WILL, opt::MCCP2]);
    while e.poll_event().is_some() {}
    e
}

fn events(engine: &mut Engine) -> Vec<Event> {
    std::iter::from_fn(|| engine.poll_event()).collect()
}

#[test]
fn compression_that_starts_inside_a_read() {
    let mut e = compressing_engine();
    let read = [
        b"hello".as_slice(),
        &START,
        &compress_to_vec_zlib(&[b'w', IAC, GA], 6),
    ]
    .concat();
    e.receive(&read);
    assert_eq!(
        events(&mut e),
        [
            Event::Data(b"hello".to_vec()),
            Event::CompressionStarted,
            Event::Data(b"w".to_vec()),
            Event::Command(GA),
            Event::CompressionEnded,
        ]
    );
}

#[test]
fn bytes_after_the_end_of_the_stream_are_plain() {
    let mut e = compressing_engine();
    let read = [&START[..], &compress_to_vec_zlib(b"in", 6), b"out"].concat();
    e.receive(&read);
    assert_eq!(
        events(&mut e),
        [
            Event::CompressionStarted,
            Event::Data(b"in".to_vec()),
            Event::CompressionEnded,
            Event::Data(b"out".to_vec()),
        ]
    );
    assert!(!e.is_compressed());
}

#[test]
fn a_bad_stream_stops_compression_with_an_error() {
    let mut e = compressing_engine();
    e.receive(&[&START[..], b"not zlib at all"].concat());
    let got = events(&mut e);
    assert_eq!(got[0], Event::CompressionStarted);
    assert!(matches!(got[1], Event::Error(ProtocolError::Inflate(_))));
    assert!(!e.is_compressed());
}

#[test]
fn a_start_without_negotiation_is_only_a_subnegotiation() {
    let mut e = Engine::new(Policy::none());
    e.receive(&START);
    assert_eq!(
        events(&mut e),
        [Event::Subnegotiation(opt::MCCP2, Vec::new())]
    );
    assert!(!e.is_compressed());
}
