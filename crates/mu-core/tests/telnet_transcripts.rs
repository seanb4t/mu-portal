//! The six transcripts of the telnet spike, read by the parser.

use mu_core::TelnetSettings;
use mu_core::telnet::{Event, Parser, Policy};

use support::{reads, transcript_reads, transcripts};

mod support;

/// The policy that recorded the transcripts.
fn policy() -> Policy {
    TelnetSettings::default().policy
}

/// The events of `parts` read in order, with adjacent data joined.
fn events(parts: &[&[u8]]) -> Vec<Event> {
    let mut parser = Parser::new(policy());
    let mut out: Vec<Event> = Vec::new();
    for part in parts {
        parser.receive(part);
        while let Some(event) = parser.poll_event() {
            match (out.last_mut(), event) {
                (Some(Event::Data(last)), Event::Data(more)) => last.extend(more),
                (_, event) => out.push(event),
            }
        }
    }
    out
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
