//! The six transcripts of the telnet spike, read by the parser.

use mu_core::telnet::{Event, Parser, Policy};
use mu_core::{Framer as _, Inbound, TelnetFramer, TelnetSettings, Transport as _};
use mu_fake_server::{Script, ScriptedGame};
use tokio::io::AsyncReadExt as _;

use support::transcript::transcript_reads;
use support::{parse_reads, reads, transcripts};

mod support;

/// The policy that recorded the transcripts.
fn policy() -> Policy {
    TelnetSettings::default().policy
}

/// The events of `parts` read in order, with adjacent data joined.
fn events(parts: &[&[u8]]) -> Vec<Event> {
    parse_reads(&mut Parser::new(policy()), parts).0
}

#[test]
fn six_transcripts_are_present() {
    assert_eq!(transcripts().len(), 6);
}

#[test]
fn transcripts_have_no_protocol_error() {
    for (name, text) in transcripts() {
        let recorded = transcript_reads(&text);
        let parts: Vec<&[u8]> = recorded.iter().map(Vec::as_slice).collect();
        let found = events(&parts);
        assert!(
            found.iter().any(|event| matches!(event, Event::Data(_))),
            "{name}"
        );
        let errors: Vec<&Event> = found
            .iter()
            .filter(|event| matches!(event, Event::Error(_)))
            .collect();
        assert_eq!(errors, Vec::<&Event>::new(), "{name}");
    }
}

#[test]
fn transcripts_in_other_reads() {
    for (name, text) in transcripts() {
        let recorded = transcript_reads(&text);
        let parts: Vec<&[u8]> = recorded.iter().map(Vec::as_slice).collect();
        let expected = events(&parts);
        let whole = recorded.concat();
        for size in [1, 2, 3, 7, 64, 1500] {
            let points: Vec<usize> = (size..whole.len()).step_by(size).collect();
            assert_eq!(
                events(&reads(&whole, &points)),
                expected,
                "{name} in reads of {size}"
            );
        }
    }
}

/// The lines and the messages of `inbound`. Prompts and replies depend on where a read ends.
fn lines_and_messages(inbound: Vec<Inbound>) -> Vec<Inbound> {
    inbound
        .into_iter()
        .filter(|item| matches!(item, Inbound::Line(_) | Inbound::Message { .. }))
        .collect()
}

#[tokio::test]
async fn replay_reads_each_transcript() {
    for (name, text) in transcripts() {
        let recorded = transcript_reads(&text);
        let mut direct = TelnetFramer::new(TelnetSettings::default());
        let expected: Vec<Inbound> = recorded.iter().flat_map(|read| direct.feed(read)).collect();

        let game = ScriptedGame::new(Script::replay(&recorded).close());
        let mut stream = game.connect(&"alpha/aria".parse().unwrap()).await.unwrap();
        let mut framer = TelnetFramer::new(TelnetSettings::default());
        let mut replayed = Vec::new();
        let mut buffer = vec![0; 4096];
        loop {
            let count = stream.read(&mut buffer).await.unwrap();
            if count == 0 {
                break;
            }
            replayed.extend(framer.feed(&buffer[..count]));
        }

        let expected = lines_and_messages(expected);
        assert!(
            expected.iter().any(|item| matches!(item, Inbound::Line(_))),
            "{name}"
        );
        assert_eq!(lines_and_messages(replayed), expected, "{name}");
    }
}
