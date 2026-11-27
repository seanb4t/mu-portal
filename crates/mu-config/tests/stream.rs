//! The names of streams, the lists of them, and the selections of the outputs.

use mu_config::layout::BlockId;
use mu_config::stream::{MAIN, NOTICES, Pattern, Scope, Selection, StreamName, Streams};
use proptest::prelude::*;
use support::{glob, id_of};

mod support;

fn name(text: &str) -> StreamName {
    StreamName::new(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn texts(streams: &Streams) -> Vec<&str> {
    streams.names().iter().map(StreamName::as_str).collect()
}

#[test]
fn a_stream_name_is_an_id() {
    assert_eq!(name("chan-public").as_str(), "chan-public");
}

#[test]
fn a_stream_name_in_upper_case() {
    assert_eq!(name("Chan-Public").as_str(), "chan-public");
}

#[test]
fn an_id_in_upper_case() {
    assert_eq!(
        BlockId::new("Comms").map(|id| id.to_string()),
        Ok("comms".to_owned())
    );
}

#[test]
fn a_stream_name_that_is_a_word() {
    assert!(StreamName::new("with").is_err());
}

#[test]
fn a_stream_name_of_32_characters() {
    assert!(StreamName::new(&"a".repeat(32)).is_ok());
}

#[test]
fn a_stream_name_of_33_characters() {
    assert!(StreamName::new(&"a".repeat(33)).is_err());
}

#[test]
fn streams_hold_one_to_eight() {
    let eight = (1..=8).map(|n| name(&format!("s{n}")));
    let nine = (1..=9).map(|n| name(&format!("s{n}")));
    assert_eq!(
        (
            Streams::new(Vec::new()).is_err(),
            Streams::new(eight).is_ok(),
            Streams::new(nine).is_err()
        ),
        (true, true, true)
    );
}

#[test]
fn streams_drop_a_name_twice() {
    let streams = Streams::new([name("pages"), name("main"), name("pages")]).expect("two names");
    assert_eq!(texts(&streams), ["pages", "main"]);
}

#[test]
fn the_main_streams() {
    assert_eq!(texts(&Streams::main()), [MAIN]);
}

#[test]
fn streams_from_the_wire_with_no_name() {
    assert_eq!(Streams::from_wire::<&str>(&[]), Streams::main());
}

#[test]
fn streams_from_the_wire_with_a_bad_name() {
    assert_eq!(Streams::from_wire(&["pages", "9bad"]), Streams::main());
}

#[test]
fn streams_from_the_wire_with_nine_names() {
    let nine: Vec<String> = (1..=9).map(|n| format!("s{n}")).collect();
    assert_eq!(Streams::from_wire(&nine), Streams::main());
}

#[test]
fn streams_from_the_wire_with_a_name_twice() {
    let mut names: Vec<String> = (1..=8).map(|n| format!("s{n}")).collect();
    names.push("s1".to_owned());
    assert_eq!(Streams::from_wire(&names).names().len(), 8);
}

fn pattern(text: &str) -> Pattern {
    Pattern::new(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn selection(texts: &[&str]) -> Selection {
    Selection::new(
        Scope::World,
        texts.iter().map(|text| pattern(text)).collect(),
    )
    .unwrap_or_else(|error| panic!("{texts:?}: {error}"))
}

#[test]
fn a_glob_matches() {
    let chosen = selection(&["chan-*", "page?"]);
    assert_eq!(
        (
            chosen.shows(&name("chan-public")),
            chosen.shows(&name("pages")),
            chosen.shows(&name("main"))
        ),
        (true, true, false)
    );
}

#[test]
fn an_exclusion_wins() {
    let chosen = selection(&["chan-*", "!chan-ooc"]);
    assert_eq!(
        (
            chosen.shows(&name("chan-public")),
            chosen.shows(&name("chan-ooc"))
        ),
        (true, false)
    );
}

#[test]
fn the_order_does_not_matter() {
    let first = selection(&["!chan-ooc", "chan-*"]);
    assert!(!first.shows(&name("chan-ooc")) && first.shows(&name("chan-public")));
}

#[test]
fn the_default_is_the_id() {
    let chosen = Selection::default_for(&id_of("comms"), Scope::World);
    assert_eq!(
        (chosen.shows(&name("comms")), chosen.shows(&name("main"))),
        (true, false)
    );
}

#[test]
fn the_default_of_main() {
    let chosen = Selection::default_for(&id_of("main"), Scope::World);
    assert_eq!(
        (
            chosen.shows(&name("main")),
            chosen.shows(&name("notices")),
            chosen.shows(&name("pages"))
        ),
        (true, true, false)
    );
}

#[test]
fn a_pattern_in_upper_case() {
    assert_eq!(pattern("!Chan-*").to_string(), "!chan-*");
}

#[test]
fn only_exclusions_are_bad() {
    assert!(Selection::new(Scope::World, vec![pattern("!chan-*")]).is_err());
}

#[test]
fn an_empty_list_is_bad() {
    assert!(Selection::new(Scope::World, Vec::new()).is_err());
}

#[test]
fn a_lone_exclusion_is_bad() {
    assert!(Pattern::new("!").is_err());
}

#[test]
fn a_pattern_that_is_a_word_is_bad() {
    assert_eq!(
        (Pattern::new("at").is_err(), Pattern::new("!Right").is_err()),
        (true, true)
    );
}

#[test]
fn a_digit_after_the_first_character_is_good() {
    assert_eq!(
        (
            Pattern::new("chan-1").map(|pattern| pattern.to_string()),
            Pattern::new("!c9*").map(|pattern| pattern.to_string())
        ),
        (Ok("chan-1".to_owned()), Ok("!c9*".to_owned()))
    );
}

#[test]
fn a_pattern_that_starts_with_a_digit_is_bad() {
    assert_eq!(
        (Pattern::new("30").is_err(), Pattern::new("!3*").is_err()),
        (true, true)
    );
}

#[test]
fn a_reserved_pattern_is_bad() {
    assert_eq!(
        (
            Pattern::new("re:chan").is_err(),
            Pattern::new("/chan/").is_err()
        ),
        (true, true)
    );
}

proptest! {
    #[test]
    fn any_order_of_the_patterns_shows_the_same(
        globs in prop::collection::vec(glob(), 1..5),
        seed in any::<u64>(),
        names in prop::collection::vec("[a-c][a-c0-9-]{0,4}", 1..8),
    ) {
        let patterns: Vec<Pattern> = globs.iter().filter_map(|text| Pattern::new(text).ok()).collect();
        let Ok(chosen) = Selection::new(Scope::World, patterns.clone()) else {
            return Ok(());
        };
        let mut turned = patterns;
        let len = turned.len();
        turned.rotate_left(usize::try_from(seed % len as u64).unwrap_or(0));
        turned.reverse();
        let other = Selection::new(Scope::World, turned).expect("the same patterns");
        for text in &names {
            let stream = name(text);
            prop_assert_eq!(chosen.shows(&stream), other.shows(&stream));
        }
    }
}

#[test]
fn the_notices_is_a_stream_name() {
    assert_eq!(StreamName::notices(), name(NOTICES));
}

proptest! {
    #[test]
    fn the_wire_gives_the_same_streams(
        texts in prop::collection::vec("[a-z][a-z0-9-]{0,31}", 1..=8),
    ) {
        let names = texts.iter().filter_map(|text| StreamName::new(text).ok());
        if let Ok(streams) = Streams::new(names) {
            prop_assert_eq!(Streams::from_wire(&streams.to_wire()), streams);
        }
    }
}
