//! The engine on the recorded transcripts of the five server families.

use std::path::PathBuf;

use telnet_engine::client::Client;
use telnet_engine::transcript::Transcript;
use telnet_engine::{Event, Side, opt};

fn load(name: &str) -> Transcript {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("transcripts")
        .join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    Transcript::parse(&text).unwrap()
}

fn replay<'a>(reads: impl Iterator<Item = &'a [u8]>) -> (Client, Vec<Event>) {
    let mut client = Client::new();
    let mut events: Vec<Event> = Vec::new();
    for read in reads {
        for event in client.receive(read) {
            match (event, events.last_mut()) {
                (Event::Data(d), Some(Event::Data(last))) => last.extend(d),
                (event, _) => events.push(event),
            }
        }
    }
    (client, events)
}

const ALL: [&str; 6] = [
    "aresmush.txt",
    "evennia.txt",
    "pennmush.txt",
    "rhostmush.txt",
    "tinymux-latin1.txt",
    "tinymux-utf8.txt",
];

#[test]
fn every_transcript_parses_with_no_protocol_error() {
    for name in ALL {
        let (_, events) = replay(load(name).server_reads());
        let errors: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, Event::Error(_)))
            .collect();
        assert!(errors.is_empty(), "{name}: {errors:?}");
    }
}

#[test]
fn other_read_boundaries_give_the_same_events() {
    for name in ALL {
        let transcript = load(name);
        let (_, recorded) = replay(transcript.server_reads());
        let bytes: Vec<u8> = transcript.server_reads().flatten().copied().collect();
        for size in [1, 2, 3, 7, 64, 1500] {
            let (_, events) = replay(bytes.chunks(size));
            assert_eq!(events, recorded, "{name} in reads of {size} bytes");
        }
    }
}

#[test]
fn the_options_each_server_turns_on() {
    let cases: [(&str, &[u8], &[u8]); 5] = [
        ("aresmush.txt", &[opt::MSSP], &[opt::NAWS, opt::CHARSET]),
        (
            "evennia.txt",
            &[opt::SGA, opt::MSSP, opt::MCCP2, opt::GMCP],
            &[opt::TTYPE, opt::NAWS],
        ),
        (
            "pennmush.txt",
            &[opt::CHARSET, opt::MSSP, opt::GMCP],
            &[opt::TTYPE, opt::NAWS],
        ),
        ("rhostmush.txt", &[], &[opt::TTYPE, opt::NAWS]),
        (
            "tinymux-latin1.txt",
            &[opt::SGA, opt::EOR, opt::CHARSET, opt::MSSP, opt::GMCP],
            &[opt::TTYPE, opt::NAWS, opt::CHARSET],
        ),
    ];
    for (name, remote, local) in cases {
        let (client, _) = replay(load(name).server_reads());
        let on = |side| {
            (0..=255u8)
                .filter(|&o| client.engine.is_enabled(side, o))
                .collect::<Vec<_>>()
        };
        let mut remote = remote.to_vec();
        let mut local = local.to_vec();
        remote.sort_unstable();
        local.sort_unstable();
        assert_eq!(
            on(Side::Remote),
            remote,
            "{name}: options the server performs"
        );
        assert_eq!(
            on(Side::Local),
            local,
            "{name}: options the client performs"
        );
    }
}

#[test]
fn evennia_starts_compression_inside_a_read_and_keeps_it_on() {
    let (client, events) = replay(load("evennia.txt").server_reads());
    assert!(events.contains(&Event::CompressionStarted));
    assert!(!events.contains(&Event::CompressionEnded));
    assert!(client.engine.is_compressed());
    let text: Vec<u8> = events
        .iter()
        .filter_map(|e| {
            if let Event::Data(d) = e {
                Some(d.as_slice())
            } else {
                None
            }
        })
        .flatten()
        .copied()
        .collect();
    let text = String::from_utf8_lossy(&text);
    assert!(
        text.contains("café ✓ 日本 😀"),
        "the UTF-8 line comes through the compressed stream"
    );
}
