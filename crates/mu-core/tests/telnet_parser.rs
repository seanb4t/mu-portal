//! Tests of the telnet parser.

use miniz_oxide::deflate::compress_to_vec_zlib;
use mu_core::telnet::{
    AO, AYT, BRK, DM, DO, DONT, EC, EL, EOR, Event, GA, IAC, IP, MAX_SUBNEGOTIATION, NOP, Parser,
    Policy, ProtocolError, SB, SE, Side, WILL, WONT, escape, opt,
};
use proptest::prelude::*;

use support::reads;

mod support;

/// The policy of the tests: GMCP and MCCP2 on the side of the game.
fn policy() -> Policy {
    Policy::none().accept(Side::Remote, &[opt::GMCP, opt::MCCP2])
}

/// The events of `parser` with adjacent data joined, and the bytes that it sent.
fn drain(parser: &mut Parser) -> (Vec<Event>, Vec<u8>) {
    let mut events: Vec<Event> = Vec::new();
    while let Some(event) = parser.poll_event() {
        match (events.last_mut(), event) {
            (Some(Event::Data(last)), Event::Data(more)) => last.extend(more),
            (_, event) => events.push(event),
        }
    }
    (events, parser.take_output())
}

/// Reads each part of `parts` as one read, and gives the joined events and the sent bytes.
fn run_reads(mut parser: Parser, parts: &[&[u8]]) -> (Vec<Event>, Vec<u8>) {
    let mut events = Vec::new();
    let mut output = Vec::new();
    for part in parts {
        parser.receive(part);
        let (more, sent) = drain(&mut parser);
        events.extend(more);
        output.extend(sent);
    }
    (join(events), output)
}

fn run(bytes: &[u8]) -> (Vec<Event>, Vec<u8>) {
    run_reads(Parser::new(policy()), &[bytes])
}

fn join(events: Vec<Event>) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::new();
    for event in events {
        match (out.last_mut(), event) {
            (Some(Event::Data(last)), Event::Data(more)) => last.extend(more),
            (_, event) => out.push(event),
        }
    }
    out
}

fn data(bytes: &[u8]) -> Event {
    Event::Data(bytes.to_vec())
}

/// A parser where MCCP2 is on for the game.
fn mccp2_parser() -> Parser {
    let mut parser = Parser::new(policy());
    parser.receive(&[IAC, WILL, opt::MCCP2]);
    drain(&mut parser);
    parser
}

const START: [u8; 5] = [IAC, SB, opt::MCCP2, IAC, SE];

fn is_bad_stream(event: &Event) -> bool {
    matches!(event, Event::Error(ProtocolError::BadCompressedStream(_)))
}

#[test]
fn escaped_byte_in_data() {
    assert_eq!(run(&[b'a', IAC, IAC, b'b']).0, [data(&[b'a', IAC, b'b'])]);
}

#[test]
fn byte_255_in_a_subnegotiation() {
    let mut parser = Parser::new(policy());
    parser.send_subnegotiation(opt::GMCP, &[b'a', IAC, b'b']);
    assert_eq!(
        parser.take_output(),
        [IAC, SB, opt::GMCP, b'a', IAC, IAC, b'b', IAC, SE]
    );
}

#[test]
fn ayt_before_data() {
    assert_eq!(run(&[IAC, AYT, b'x']).0, [Event::Command(AYT), data(b"x")]);
}

#[test]
fn each_two_byte_command() {
    for command in [NOP, DM, BRK, IP, AO, AYT, EC, EL, GA, EOR] {
        assert_eq!(
            run(&[IAC, command, b'x']).0,
            [Event::Command(command), data(b"x")],
            "{command}"
        );
    }
}

#[test]
fn se_outside_a_subnegotiation() {
    assert_eq!(run(&[IAC, SE, b'x']).0, [Event::Command(SE), data(b"x")]);
}

#[test]
fn byte_that_is_no_command() {
    assert_eq!(run(&[IAC, 100, b'x']).0, [Event::Command(100), data(b"x")]);
}

#[test]
fn accepted_option() {
    assert_eq!(
        run(&[IAC, WILL, opt::GMCP]),
        (
            vec![Event::Enabled(Side::Remote, opt::GMCP)],
            vec![IAC, DO, opt::GMCP]
        )
    );
}

#[test]
fn refused_option_twice() {
    let mut parser = Parser::new(policy());
    let bytes = [IAC, WILL, opt::MXP, IAC, WILL, opt::MXP];
    parser.receive(&bytes);
    assert_eq!(
        drain(&mut parser),
        (vec![], vec![IAC, DONT, opt::MXP, IAC, DONT, opt::MXP])
    );
    assert!(!parser.is_enabled(Side::Remote, opt::MXP));
}

#[test]
fn repeated_request() {
    let mut parser = Parser::new(policy());
    parser.receive(&[IAC, WILL, opt::GMCP]);
    drain(&mut parser);
    parser.receive(&[IAC, WILL, opt::GMCP]);
    assert_eq!(drain(&mut parser), (vec![], vec![]));
}

#[test]
fn crossing_requests() {
    let mut parser = Parser::new(policy());
    parser.request(Side::Remote, opt::GMCP, true);
    assert_eq!(parser.take_output(), [IAC, DO, opt::GMCP]);
    parser.receive(&[IAC, WILL, opt::GMCP]);
    assert_eq!(
        drain(&mut parser),
        (vec![Event::Enabled(Side::Remote, opt::GMCP)], vec![])
    );
    parser.receive(&[IAC, WILL, opt::GMCP]);
    assert_eq!(drain(&mut parser), (vec![], vec![]));
}

#[test]
fn refusal_of_a_client_request() {
    let mut parser = Parser::new(policy());
    parser.request(Side::Remote, opt::GMCP, true);
    parser.take_output();
    parser.receive(&[IAC, WONT, opt::GMCP]);
    assert_eq!(drain(&mut parser), (vec![], vec![]));
    assert!(!parser.is_enabled(Side::Remote, opt::GMCP));
}

#[test]
fn unrequested_answer_is_an_error() {
    let mut parser = Parser::new(policy());
    parser.receive(&[IAC, WILL, opt::GMCP]);
    drain(&mut parser);
    parser.request(Side::Remote, opt::GMCP, false);
    parser.take_output();
    parser.receive(&[IAC, WILL, opt::GMCP]);
    assert_eq!(
        drain(&mut parser).0,
        [Event::Error(ProtocolError::UnrequestedAnswer {
            side: Side::Remote,
            option: opt::GMCP
        })]
    );
}

#[test]
fn payload_with_escaped_byte() {
    let bytes = [IAC, SB, opt::GMCP, b'a', IAC, IAC, b'b', IAC, SE];
    assert_eq!(
        run(&bytes).0,
        [Event::Subnegotiation(opt::GMCP, vec![b'a', IAC, b'b'])]
    );
}

/// The bytes of a GMCP subnegotiation of `payload`.
fn subnegotiation(payload: &[u8]) -> Vec<u8> {
    [&[IAC, SB, opt::GMCP][..], &escape(payload), &[IAC, SE]].concat()
}

#[test]
fn payload_at_the_bound() {
    let mut payload = vec![b'a'; MAX_SUBNEGOTIATION - 1];
    payload.push(IAC);
    let bytes = subnegotiation(&payload);
    assert_eq!(bytes[bytes.len() - 4..bytes.len() - 2], [IAC, IAC]);
    assert_eq!(run(&bytes).0, [Event::Subnegotiation(opt::GMCP, payload)]);
}

#[test]
fn payload_over_the_bound() {
    let payload = vec![b'a'; MAX_SUBNEGOTIATION + 1];
    let bytes = [subnegotiation(&payload), b"ok".to_vec()].concat();
    let expected = vec![
        Event::Error(ProtocolError::SubnegotiationTooLong { option: opt::GMCP }),
        data(b"ok"),
    ];
    assert_eq!(run(&bytes).0, expected);
    let parts: Vec<&[u8]> = bytes.chunks(4096).collect();
    assert_eq!(run_reads(Parser::new(policy()), &parts).0, expected);
}

#[test]
fn mark_from_the_game() {
    let mut parser = Parser::new(policy());
    parser.receive(&[IAC, DO, opt::TIMING_MARK, IAC, DO, opt::TIMING_MARK]);
    assert_eq!(
        drain(&mut parser),
        (
            vec![],
            vec![IAC, WONT, opt::TIMING_MARK, IAC, WONT, opt::TIMING_MARK]
        )
    );
    assert!(!parser.is_enabled(Side::Local, opt::TIMING_MARK));
    assert!(!parser.is_enabled(Side::Remote, opt::TIMING_MARK));
}

#[test]
fn answer_to_a_mark_changes_no_option() {
    for verb in [WILL, WONT] {
        let mut parser = Parser::new(policy());
        parser.receive(&[IAC, verb, opt::TIMING_MARK]);
        assert_eq!(drain(&mut parser), (vec![Event::Mark], vec![]));
        assert!(!parser.is_enabled(Side::Remote, opt::TIMING_MARK));
    }
}

#[test]
fn malformed_framing() {
    assert_eq!(
        run(&[IAC, SB, IAC, SE, b'x']).0,
        [
            Event::Error(ProtocolError::EmptySubnegotiation),
            Event::Command(SE),
            data(b"x")
        ]
    );
    for verb in [WILL, WONT, DO, DONT] {
        let (events, output) = run(&[IAC, SB, opt::GMCP, b'a', IAC, verb, opt::MXP]);
        assert_eq!(
            events,
            [Event::Error(ProtocolError::UnterminatedSubnegotiation {
                option: opt::GMCP
            })],
            "{verb}"
        );
        let reply = match verb {
            WILL => vec![IAC, DONT, opt::MXP],
            DO => vec![IAC, WONT, opt::MXP],
            _ => vec![],
        };
        assert_eq!(output, reply, "{verb}");
    }
    assert_eq!(
        run_reads(Parser::new(policy()), &[&[b'a', IAC], &[GA]]).0,
        [data(b"a"), Event::Command(GA)]
    );
    assert_eq!(run(&[b'a', IAC]).0, [data(b"a")]);
}

#[test]
fn stream_starts_inside_a_read() {
    let bytes = [
        &[IAC, WILL, opt::MCCP2][..],
        &START,
        &compress_to_vec_zlib(b"hello", 6),
    ]
    .concat();
    let (events, output) = run(&bytes);
    assert_eq!(output, [IAC, DO, opt::MCCP2]);
    assert_eq!(
        events,
        [
            Event::Enabled(Side::Remote, opt::MCCP2),
            Event::CompressionStarted,
            data(b"hello"),
            Event::CompressionEnded
        ]
    );
}

#[test]
fn plain_bytes_after_the_stream() {
    let bytes = [&START[..], &compress_to_vec_zlib(b"in", 6), b"out"].concat();
    assert_eq!(
        run_reads(mccp2_parser(), &[&bytes]).0,
        [
            Event::CompressionStarted,
            data(b"in"),
            Event::CompressionEnded,
            data(b"out")
        ]
    );
}

#[test]
fn start_inside_a_stream() {
    let inner = [&START[..], b"x"].concat();
    let bytes = [&START[..], &compress_to_vec_zlib(&inner, 6)].concat();
    assert_eq!(
        run_reads(mccp2_parser(), &[&bytes]).0,
        [
            Event::CompressionStarted,
            Event::Subnegotiation(opt::MCCP2, vec![]),
            data(b"x"),
            Event::CompressionEnded
        ]
    );
}

#[test]
fn second_stream_after_end() {
    let bytes = [
        &START[..],
        &compress_to_vec_zlib(b"a", 6),
        &START,
        &compress_to_vec_zlib(b"b", 6),
    ]
    .concat();
    assert_eq!(
        run_reads(mccp2_parser(), &[&bytes]).0,
        [
            Event::CompressionStarted,
            data(b"a"),
            Event::CompressionEnded,
            Event::CompressionStarted,
            data(b"b"),
            Event::CompressionEnded
        ]
    );
}

#[test]
fn start_while_mccp2_is_off() {
    let bytes = [&START[..], b"plain"].concat();
    assert_eq!(
        run(&bytes).0,
        [Event::Subnegotiation(opt::MCCP2, vec![]), data(b"plain")]
    );
}

#[test]
fn stream_that_is_not_zlib() {
    let first = [&START[..], &[0x00, 0x01, 0x02]].concat();
    let (events, _) = run_reads(mccp2_parser(), &[&first, b"more\n"]);
    assert_eq!(events.len(), 2, "{events:?}");
    assert_eq!(events[0], Event::CompressionStarted);
    assert!(is_bad_stream(&events[1]), "{events:?}");
}

#[test]
fn bad_stream_after_data() {
    let mut stream = compress_to_vec_zlib(&b"some data before the break ".repeat(8), 6);
    let last = stream.len() - 1;
    stream[last] ^= 1;
    let bytes = [&START[..], &stream].concat();
    for point in 0..=bytes.len() {
        let (events, _) = run_reads(
            mccp2_parser(),
            &[&bytes[..point], &bytes[point..], b"more\n"],
        );
        let errors = events.iter().filter(|event| is_bad_stream(event)).count();
        assert_eq!(errors, 1, "split at {point}: {events:?}");
        assert!(
            is_bad_stream(events.last().unwrap()),
            "split at {point}: {events:?}"
        );
        assert!(events.contains(&Event::CompressionStarted));
        assert!(
            events.iter().any(|event| matches!(event, Event::Data(_))),
            "split at {point}: {events:?}"
        );
    }
}

/// A part of a random stream: its bytes, and the events and replies that it must give.
#[derive(Clone, Debug)]
struct Part {
    bytes: Vec<u8>,
    events: Vec<Event>,
    output: Vec<u8>,
}

impl Part {
    fn new(bytes: Vec<u8>, events: Vec<Event>, output: Vec<u8>) -> Self {
        Self {
            bytes,
            events,
            output,
        }
    }
}

const COMMANDS: [u8; 11] = [NOP, DM, BRK, IP, AO, AYT, EC, EL, GA, EOR, SE];
/// Options that the policy refuses, so that each negotiation gives a fixed reply.
const REFUSED: [u8; 4] = [opt::ECHO, opt::MXP, opt::MSDP, opt::NAWS];

fn data_part() -> impl Strategy<Value = Part> {
    prop::collection::vec(prop_oneof![any::<u8>(), Just(IAC)], 1..20)
        .prop_map(|bytes| Part::new(escape(&bytes), vec![Event::Data(bytes)], vec![]))
}

fn command_part() -> impl Strategy<Value = Part> {
    prop::sample::select(COMMANDS.to_vec())
        .prop_map(|code| Part::new(vec![IAC, code], vec![Event::Command(code)], vec![]))
}

fn negotiation_part() -> impl Strategy<Value = Part> {
    (
        prop::sample::select(vec![WILL, WONT, DO, DONT]),
        prop::sample::select(REFUSED.to_vec()),
    )
        .prop_map(|(verb, option)| {
            let reply = match verb {
                WILL => vec![IAC, DONT, option],
                DO => vec![IAC, WONT, option],
                _ => vec![],
            };
            Part::new(vec![IAC, verb, option], vec![], reply)
        })
}

fn mark_part() -> impl Strategy<Value = Part> {
    prop::sample::select(vec![WILL, WONT])
        .prop_map(|verb| Part::new(vec![IAC, verb, opt::TIMING_MARK], vec![Event::Mark], vec![]))
}

fn subnegotiation_part() -> impl Strategy<Value = Part> {
    (
        prop::sample::select(vec![opt::GMCP, opt::TTYPE, opt::CHARSET, opt::MSSP]),
        prop::collection::vec(any::<u8>(), 0..20),
    )
        .prop_map(|(option, payload)| {
            let bytes = [&[IAC, SB, option][..], &escape(&payload), &[IAC, SE]].concat();
            Part::new(bytes, vec![Event::Subnegotiation(option, payload)], vec![])
        })
}

fn malformed_part() -> impl Strategy<Value = Part> {
    prop_oneof![
        Just(Part::new(
            vec![IAC, SB, IAC, SE],
            vec![
                Event::Error(ProtocolError::EmptySubnegotiation),
                Event::Command(SE)
            ],
            vec![]
        )),
        (prop::collection::vec(1u8..IAC, 0..5)).prop_map(|payload| {
            let bytes = [&[IAC, SB, opt::GMCP][..], &payload, &[IAC, WILL, opt::MXP]].concat();
            Part::new(
                bytes,
                vec![Event::Error(ProtocolError::UnterminatedSubnegotiation {
                    option: opt::GMCP,
                })],
                vec![IAC, DONT, opt::MXP],
            )
        }),
    ]
}

fn plain_part() -> impl Strategy<Value = Part> {
    prop_oneof![
        4 => data_part(),
        2 => command_part(),
        2 => negotiation_part(),
        1 => mark_part(),
        2 => subnegotiation_part(),
        1 => malformed_part(),
    ]
}

/// A compressed run of plain parts. Its bytes are the start and the zlib stream.
fn compressed_part() -> impl Strategy<Value = Part> {
    prop::collection::vec(plain_part(), 0..5).prop_map(|parts| {
        let inner: Vec<u8> = parts.iter().flat_map(|part| part.bytes.clone()).collect();
        let mut events = vec![Event::CompressionStarted];
        events.extend(parts.iter().flat_map(|part| part.events.clone()));
        events.push(Event::CompressionEnded);
        let output = parts.iter().flat_map(|part| part.output.clone()).collect();
        Part::new(
            [&START[..], &compress_to_vec_zlib(&inner, 6)].concat(),
            events,
            output,
        )
    })
}

/// The last part of a stream: a bad zlib header and any bytes after it, or a lone IAC.
fn last_part() -> impl Strategy<Value = Option<Part>> {
    prop_oneof![
        2 => Just(None),
        1 => prop::collection::vec(any::<u8>(), 0..20).prop_map(|tail| {
            let bytes = [&START[..], &[0x00, 0x01, 0x02], &tail].concat();
            Some(Part::new(
                bytes,
                vec![
                    Event::CompressionStarted,
                    Event::Error(ProtocolError::BadCompressedStream(String::new())),
                ],
                vec![],
            ))
        }),
        1 => Just(Some(Part::new(vec![IAC], vec![], vec![]))),
    ]
}

/// A whole stream: MCCP2 turns on, then the parts, then the optional last part.
fn stream() -> impl Strategy<Value = (Vec<u8>, Vec<Event>, Vec<u8>)> {
    (
        prop::collection::vec(
            prop_oneof![4 => plain_part(), 1 => compressed_part()],
            0..12,
        ),
        last_part(),
    )
        .prop_map(|(parts, last)| {
            let mut bytes = vec![IAC, WILL, opt::MCCP2];
            let mut events = vec![Event::Enabled(Side::Remote, opt::MCCP2)];
            let mut output = vec![IAC, DO, opt::MCCP2];
            for part in parts.into_iter().chain(last) {
                bytes.extend(part.bytes);
                events.extend(part.events);
                output.extend(part.output);
            }
            (bytes, join(events), output)
        })
}

/// The events with the text of a bad stream error removed, which zlib words.
fn without_error_text(events: Vec<Event>) -> Vec<Event> {
    events
        .into_iter()
        .map(|event| match event {
            Event::Error(ProtocolError::BadCompressedStream(_)) => {
                Event::Error(ProtocolError::BadCompressedStream(String::new()))
            }
            other => other,
        })
        .collect()
}

proptest! {
    #[test]
    fn parser_matches_the_oracle((bytes, events, output) in stream()) {
        let (got, sent) = run(&bytes);
        prop_assert_eq!(without_error_text(got), events);
        prop_assert_eq!(sent, output);
    }

    #[test]
    fn parser_ignores_read_splits(
        (bytes, _, _) in stream(),
        cuts in prop::collection::vec(any::<prop::sample::Index>(), 0..12),
    ) {
        let points: Vec<usize> = cuts.iter().map(|cut| cut.index(bytes.len() + 1)).collect();
        let whole = run(&bytes);
        let split = run_reads(Parser::new(policy()), &reads(&bytes, &points));
        prop_assert_eq!(split, whole);
    }

    #[test]
    fn parser_round_trip(
        text in prop::collection::vec(any::<u8>(), 1..200),
        payload in prop::collection::vec(any::<u8>(), 0..200),
    ) {
        let mut sender = Parser::new(policy());
        sender.send_subnegotiation(opt::GMCP, &payload);
        let bytes = [escape(&text), sender.take_output()].concat();
        prop_assert_eq!(
            run(&bytes).0,
            vec![Event::Data(text), Event::Subnegotiation(opt::GMCP, payload)]
        );
    }
}
