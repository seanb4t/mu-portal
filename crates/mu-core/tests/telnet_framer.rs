//! Tests of the telnet framer.

use miniz_oxide::deflate::compress_to_vec_zlib;
use mu_core::telnet::{
    DO, DONT, EOR, GA, IAC, MAX_SUBNEGOTIATION, SB, SE, WILL, WONT, charset, escape, opt, ttype,
};
use mu_core::{Charset, Framer, Inbound, TelnetFramer, TelnetSettings};
use proptest::prelude::*;

use support::{reads, subnegotiation};

mod support;

/// The line bound of the framers.
const MAX_LINE: usize = mu_core::framer::MAX_LINE;

fn framer() -> TelnetFramer {
    TelnetFramer::new(TelnetSettings::default())
}

fn with_charset(charset: Charset) -> TelnetFramer {
    TelnetFramer::new(TelnetSettings {
        charset,
        ..TelnetSettings::default()
    })
}

fn with_window(columns: u16, rows: u16) -> TelnetFramer {
    TelnetFramer::new(TelnetSettings {
        columns,
        rows,
        ..TelnetSettings::default()
    })
}

/// What `framer` gives for each of `parts`, read in order.
fn feed_all(framer: &mut TelnetFramer, parts: &[&[u8]]) -> Vec<Inbound> {
    parts.iter().flat_map(|part| framer.feed(part)).collect()
}

fn line(text: &str) -> Inbound {
    Inbound::Line(text.to_owned())
}

fn prompt(text: &str) -> Inbound {
    Inbound::Prompt(text.to_owned())
}

fn lines(items: Vec<Inbound>) -> Vec<String> {
    items
        .into_iter()
        .filter_map(|item| match item {
            Inbound::Line(text) => Some(text),
            _ => None,
        })
        .collect()
}

/// The reply of `framer` to `bytes`, after the parts before it.
fn reply_to(framer: &mut TelnetFramer, bytes: &[u8]) -> Vec<u8> {
    framer
        .feed(bytes)
        .into_iter()
        .filter_map(|item| match item {
            Inbound::Reply(reply) => Some(reply),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn line_to_the_game() {
    assert_eq!(framer().encode("hi"), b"hi\r\n");
}

#[test]
fn line_ends() {
    assert_eq!(
        lines(framer().feed(b"one\r\ntwo\nthree\n")),
        ["one", "two", "three"]
    );
}

#[test]
fn cr_nul_across_reads() {
    assert_eq!(lines(feed_all(&mut framer(), &[b"a\r", b"\0b\n"])), ["ab"]);
}

#[test]
fn long_run_with_no_line_end() {
    let bytes = [vec![b'a'; MAX_LINE + 3], vec![b'\n']].concat();
    assert_eq!(
        lines(framer().feed(&bytes)),
        ["a".repeat(MAX_LINE), "aaa".to_owned()]
    );
}

#[test]
fn line_of_exactly_the_bound() {
    for end in [b"\n".as_slice(), b"\r\n"] {
        let bytes = [vec![b'a'; MAX_LINE], end.to_vec()].concat();
        assert_eq!(lines(framer().feed(&bytes)), ["a".repeat(MAX_LINE)]);
    }
}

#[test]
fn latin1_line_from_the_game() {
    assert_eq!(
        with_charset(Charset::Latin1).feed(b"caf\xe9\n"),
        [line("café")]
    );
}

#[test]
fn utf8_line_from_the_game() {
    assert_eq!(
        with_charset(Charset::Utf8).feed(b"caf\xc3\xa9\n\xe9\n"),
        [line("café"), line("\u{FFFD}")]
    );
}

#[test]
fn latin1_line_to_the_game() {
    let expected = [&b"caf\xe9 "[..], &[IAC, IAC], b" ?\r\n"].concat();
    assert_eq!(with_charset(Charset::Latin1).encode("café ÿ ✓"), expected);
}

#[test]
fn prompt_with_no_ga() {
    assert_eq!(framer().feed(b"Name: "), [prompt("Name: ")]);
}

#[test]
fn prompt_with_eor() {
    let bytes = [&b"HP 10> "[..], &[IAC, EOR]].concat();
    assert_eq!(framer().feed(&bytes), [prompt("HP 10> ")]);
}

#[test]
fn cr_or_nul_alone_gives_no_prompt() {
    let mut framer = framer();
    assert_eq!(framer.feed(b"Name: "), [prompt("Name: ")]);
    assert_eq!(framer.feed(b"\r"), []);
    assert_eq!(framer.feed(b"\0"), []);
}

#[test]
fn bare_cr_in_a_line_to_the_game() {
    assert_eq!(framer().encode("a\rb"), b"a\r\0b\r\n");
    assert_eq!(framer().encode("a\r"), b"a\r\0\r\n");
}

#[test]
fn prompt_with_ga() {
    let bytes = [&b"HP 10> "[..], &[IAC, GA], b"more"].concat();
    assert_eq!(framer().feed(&bytes), [prompt("HP 10> "), prompt("more")]);
}

#[test]
fn line_split_across_reads() {
    assert_eq!(
        feed_all(&mut framer(), &[b"hel", b"lo\n"]),
        [prompt("hel"), line("hello")]
    );
}

#[test]
fn read_that_ends_at_a_line_end() {
    let mut framer = framer();
    assert_eq!(framer.feed(b"hel"), [prompt("hel")]);
    assert_eq!(framer.feed(b"lo\n"), [line("hello")]);
}

#[test]
fn ga_after_a_line_end() {
    let bytes = [&b"ok\n"[..], &[IAC, GA]].concat();
    assert_eq!(framer().feed(&bytes), [line("ok")]);
}

#[test]
fn negotiation_alone() {
    let mut framer = framer();
    assert_eq!(framer.feed(b"Name: "), [prompt("Name: ")]);
    assert_eq!(
        framer.feed(&[IAC, WILL, opt::GMCP]),
        [Inbound::Reply(vec![IAC, DO, opt::GMCP])]
    );
}

#[test]
fn same_prompt_after_a_line() {
    assert_eq!(
        feed_all(&mut framer(), &[b"> ", b"ok\n> "]),
        [prompt("> "), line("> ok"), prompt("> ")]
    );
}

#[test]
fn two_marks_in_a_row() {
    let framer = framer();
    let mark = vec![IAC, DO, opt::TIMING_MARK];
    assert_eq!([framer.mark(), framer.mark()], [mark.clone(), mark]);
}

#[test]
fn answer_to_a_mark() {
    for verb in [WILL, WONT] {
        let bytes = [&b"one\n"[..], &[IAC, verb, opt::TIMING_MARK], b"two\n"].concat();
        assert_eq!(
            framer().feed(&bytes),
            [line("one"), Inbound::Mark, line("two")]
        );
    }
}

#[test]
fn gmcp_message() {
    let mut framer = framer();
    framer.feed(&[IAC, WILL, opt::GMCP]);
    assert_eq!(
        framer.feed(&subnegotiation(opt::GMCP, b"Logged.In")),
        [Inbound::Message {
            name: "GMCP".to_owned(),
            body: b"Logged.In".to_vec()
        }]
    );
}

#[test]
fn four_ttype_requests() {
    let mut framer = framer();
    framer.feed(&[IAC, DO, opt::TTYPE]);
    let request = subnegotiation(opt::TTYPE, &[ttype::SEND]);
    let answers: Vec<Vec<u8>> = (0..4).map(|_| reply_to(&mut framer, &request)).collect();
    let expected: Vec<Vec<u8>> = ["MU-PORTAL", "XTERM-256COLOR", "MTTS 269", "MTTS 269"]
        .iter()
        .map(|name| subnegotiation(opt::TTYPE, &[&[ttype::IS], name.as_bytes()].concat()))
        .collect();
    assert_eq!(answers, expected);
}

#[test]
fn ttype_request_while_ttype_is_off() {
    let request = subnegotiation(opt::TTYPE, &[ttype::SEND]);
    assert_eq!(reply_to(&mut framer(), &request), []);
    let before_do = [&request[..], &[IAC, DO, opt::TTYPE]].concat();
    assert_eq!(reply_to(&mut framer(), &before_do), [IAC, WILL, opt::TTYPE]);
}

#[test]
fn ttype_answer_in_the_order_of_the_read() {
    let request = subnegotiation(opt::TTYPE, &[ttype::SEND]);
    let bytes = [
        &[IAC, DO, opt::TTYPE][..],
        &request,
        &[IAC, DONT, opt::TTYPE],
    ]
    .concat();
    let expected = [
        &[IAC, WILL, opt::TTYPE][..],
        &subnegotiation(opt::TTYPE, &[&[ttype::IS][..], b"MU-PORTAL"].concat()),
        &[IAC, WONT, opt::TTYPE],
    ]
    .concat();
    assert_eq!(reply_to(&mut framer(), &bytes), expected);
}

#[test]
fn naws_turns_on() {
    let expected = [
        &[IAC, WILL, opt::NAWS][..],
        &subnegotiation(opt::NAWS, &[0, 120, 0, 40]),
    ]
    .concat();
    assert_eq!(
        reply_to(&mut with_window(120, 40), &[IAC, DO, opt::NAWS]),
        expected
    );
}

#[test]
fn size_with_byte_255() {
    let expected = [
        &[IAC, WILL, opt::NAWS][..],
        &[IAC, SB, opt::NAWS, 0, IAC, IAC, 0, 40, IAC, SE],
    ]
    .concat();
    assert_eq!(
        reply_to(&mut with_window(u16::from(IAC), 40), &[IAC, DO, opt::NAWS]),
        expected
    );
}

/// The reply of a UTF-8 framer to a CHARSET request with `body` after the command.
fn charset_reply(body: &[u8]) -> Vec<u8> {
    let request = [&[charset::REQUEST][..], body].concat();
    reply_to(&mut framer(), &subnegotiation(opt::CHARSET, &request))
}

fn charset_answer(answer: &[u8]) -> Vec<u8> {
    subnegotiation(opt::CHARSET, answer)
}

#[test]
fn charset_is_offered() {
    assert_eq!(
        charset_reply(b";ISO-8859-1;utf-8"),
        charset_answer(&[&[charset::ACCEPTED][..], b"utf-8"].concat())
    );
}

#[test]
fn charset_request_with_ttable() {
    let body = [charset::TTABLE, &[1], b";UTF-8"].concat();
    assert_eq!(
        charset_reply(&body),
        charset_answer(&[&[charset::ACCEPTED][..], b"UTF-8"].concat())
    );
}

#[test]
fn charset_is_not_offered() {
    assert_eq!(
        charset_reply(b";ISO-8859-1"),
        charset_answer(&[charset::REJECTED])
    );
}

#[test]
fn ttable_prefix_with_no_version() {
    assert_eq!(
        charset_reply(charset::TTABLE),
        charset_answer(&[charset::REJECTED])
    );
}

#[test]
fn empty_charset_list() {
    for body in [Vec::new(), [charset::TTABLE, &[1]].concat()] {
        assert_eq!(
            charset_reply(&body),
            charset_answer(&[charset::REJECTED]),
            "{body:?}"
        );
    }
}

fn telnet_message(item: &Inbound) -> Option<String> {
    match item {
        Inbound::Message { name, body } if name == "telnet" => {
            Some(String::from_utf8_lossy(body).into_owned())
        }
        _ => None,
    }
}

#[test]
fn subnegotiation_over_the_bound() {
    let mut framer = framer();
    framer.feed(&[IAC, WILL, opt::GMCP]);
    let items = framer.feed(&subnegotiation(
        opt::GMCP,
        &vec![b'a'; MAX_SUBNEGOTIATION + 1],
    ));
    let messages: Vec<String> = items.iter().filter_map(telnet_message).collect();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(
        messages[0].starts_with("subnegotiation-too-long "),
        "{messages:?}"
    );
    assert!(messages[0].contains(&opt::GMCP.to_string()), "{messages:?}");
    assert!(
        !items
            .iter()
            .any(|item| matches!(item, Inbound::Message { name, .. } if name == "GMCP"))
    );
}

#[test]
fn bad_compressed_stream() {
    let mut framer = framer();
    framer.feed(&[IAC, WILL, opt::MCCP2]);
    let first = [&subnegotiation(opt::MCCP2, &[])[..], &[0x00, 0x01, 0x02]].concat();
    let items = feed_all(&mut framer, &[&first, b"more\n"]);
    let messages: Vec<String> = items.iter().filter_map(telnet_message).collect();
    assert_eq!(messages.len(), 1, "{items:?}");
    assert!(
        messages[0].starts_with("bad-compressed-stream "),
        "{messages:?}"
    );
    assert!(
        !items
            .iter()
            .any(|item| matches!(item, Inbound::Line(_) | Inbound::Prompt(_)))
    );
}

#[test]
fn compressed_lines() {
    let mut framer = framer();
    framer.feed(&[IAC, WILL, opt::MCCP2]);
    let bytes = [
        &subnegotiation(opt::MCCP2, &[])[..],
        &compress_to_vec_zlib(b"one\ntwo\n", 6),
    ]
    .concat();
    assert_eq!(lines(framer.feed(&bytes)), ["one", "two"]);
}

#[test]
fn latin1_round_trip() {
    for code in (0..256_u16).map(|code| u8::try_from(code).unwrap()) {
        let character = char::from(code);
        if character == '\r' || character == '\n' {
            continue;
        }
        let bytes = with_charset(Charset::Latin1).encode(&character.to_string());
        assert_eq!(
            with_charset(Charset::Latin1).feed(&bytes),
            [Inbound::Line(character.to_string())],
            "{code}"
        );
    }
}

/// A byte of a random stream, with the line ends, CR NUL and GA favored.
fn stream_byte() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        4 => any::<u8>().prop_map(|byte| escape(&[byte])),
        2 => Just(b"\n".to_vec()),
        1 => Just(b"\r\0".to_vec()),
        1 => Just(b"\r\n".to_vec()),
        1 => Just(vec![IAC, GA]),
        1 => Just(vec![IAC, WILL, opt::TIMING_MARK]),
    ]
}

proptest! {
    #[test]
    fn telnet_framer_lines_ignore_read_splits(
        pieces in prop::collection::vec(stream_byte(), 0..200),
        cuts in prop::collection::vec(any::<prop::sample::Index>(), 0..12),
    ) {
        let bytes = pieces.concat();
        let points: Vec<usize> = cuts.iter().map(|cut| cut.index(bytes.len() + 1)).collect();
        let whole = lines(framer().feed(&bytes));
        prop_assert_eq!(lines(feed_all(&mut framer(), &reads(&bytes, &points))), whole);
    }

    #[test]
    fn framer_survives_any_payload(
        option in prop::sample::select(vec![opt::CHARSET, opt::TTYPE, opt::GMCP]),
        payload in prop::collection::vec(any::<u8>(), 0..64),
    ) {
        let mut framer = framer();
        framer.feed(&[IAC, DO, opt::TTYPE, IAC, WILL, opt::GMCP]);
        framer.feed(&subnegotiation(option, &payload));
        prop_assert_eq!(lines(framer.feed(b"ok\n")), vec!["ok".to_owned()]);
    }
}
