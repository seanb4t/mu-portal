//! The fields of a client layer.

use std::collections::BTreeSet;

use mu_config::{ClientLayer, FIELD_PATHS, Label, Segment};

#[test]
fn the_defaults_set_each_field_path() {
    let value = toml::Value::try_from(ClientLayer::defaults()).unwrap();
    let fields: BTreeSet<&str> = value
        .as_table()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(fields, FIELD_PATHS.iter().copied().collect());
}

fn label(text: &str) -> Label {
    Label::parse(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn label_error(text: &str) -> String {
    Label::parse(text)
        .expect_err("the label is bad")
        .to_string()
}

#[test]
fn a_label_with_the_number() {
    assert_eq!(
        label("{number}:{character}").render("alpha/aria", 2),
        "2:aria"
    );
}

#[test]
fn the_game_and_the_character() {
    assert_eq!(
        label("{character}@{game}").render("alpha/aria", 1),
        "aria@alpha"
    );
}

#[test]
fn the_default_label_is_the_world() {
    assert_eq!(Label::default(), label(mu_config::LABEL_DEFAULT));
    assert_eq!(Label::default().render("alpha/aria", 1), "alpha/aria");
}

#[test]
fn braces() {
    assert_eq!(label("{{{world}}}").render("alpha/aria", 1), "{alpha/aria}");
}

#[test]
fn an_unknown_label_field() {
    let text = label_error("{mood}");
    assert!(text.contains("mood"), "{text}");
}

#[test]
fn an_open_brace() {
    let text = label_error("{world");
    assert!(text.contains('{'), "{text}");
}

#[test]
fn a_lone_closing_brace() {
    let text = label_error("world}");
    assert!(text.contains('}'), "{text}");
}

#[test]
fn an_empty_label() {
    let text = label_error("");
    assert!(text.contains("1 to 64"), "{text}");
}

#[test]
fn a_label_too_long() {
    let text = label_error(&"a".repeat(65));
    assert!(text.contains("65"), "{text}");
}

#[test]
fn a_label_of_64_characters() {
    assert!(Label::parse(&"a".repeat(64)).is_ok());
}

#[test]
fn a_label_of_64_wide_characters() {
    let text = "é".repeat(64);
    assert_eq!(text.len(), 128);
    assert!(Label::parse(&text).is_ok());
}

#[test]
fn an_unknown_segment() {
    let text = Segment::check(&["mood"]).expect_err("mood").to_string();
    assert!(text.contains("mood"), "{text}");
}

#[test]
fn a_segment_twice() {
    let text = Segment::check(&["idle", "idle"])
        .expect_err("twice")
        .to_string();
    assert!(text.contains("idle"), "{text}");
}

#[test]
fn the_segments_of_a_list() {
    assert_eq!(
        Segment::check(&["world", "state"]),
        Ok(Segment::LEFT_DEFAULT.to_vec())
    );
}
