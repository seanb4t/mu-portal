//! The read boundaries never change what the engine reports or sends.

use proptest::prelude::*;
use telnet_engine::oracle::split;
use telnet_engine::{Engine, Event, client};

fn run(reads: &[&[u8]]) -> (Vec<Event>, Vec<u8>) {
    let mut engine = Engine::new(client::policy());
    let mut events: Vec<Event> = Vec::new();
    for read in reads {
        engine.receive(read);
        while let Some(event) = engine.poll_event() {
            match (event, events.last_mut()) {
                (Event::Data(d), Some(Event::Data(last))) => last.extend(d),
                (event, _) => events.push(event),
            }
        }
    }
    (events, engine.take_output())
}

fn telnet_byte() -> impl Strategy<Value = u8> {
    prop_oneof![
        4 => prop::sample::select(b"ab\r\n\0".to_vec()),
        3 => Just(255u8),
        2 => 239u8..=254,
        2 => prop::sample::select(vec![1u8, 3, 24, 25, 31, 42, 70, 86, 201]),
        1 => any::<u8>(),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    #[test]
    fn split_reads_give_the_same_result(
        bytes in prop::collection::vec(telnet_byte(), 0..200),
        cuts in prop::collection::vec(any::<usize>(), 0..20),
    ) {
        prop_assert_eq!(run(&split(&bytes, &cuts)), run(&[&bytes]));
    }
}
