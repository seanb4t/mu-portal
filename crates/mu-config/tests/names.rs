//! The names of a client.

use std::collections::BTreeMap;

use mu_config::{
    Choice, DeviceName, GameName, Kind, LayoutName, Os, WorldId, WorldKey, layout_for,
};

#[test]
fn a_device_name_of_64() {
    assert!(DeviceName::new(&"a".repeat(64)).is_ok());
}

#[test]
fn a_device_name_of_65() {
    let name = "a".repeat(65);
    let error = DeviceName::new(&name).expect_err("65").to_string();
    assert!(error.contains(&name), "{error}");
}

#[test]
fn an_empty_device_name() {
    let error = DeviceName::new("").expect_err("empty").to_string();
    assert!(error.starts_with("device "), "{error}");
}

#[test]
fn an_uppercase_device_name() {
    assert!(DeviceName::new("Laptop").is_err());
}

#[test]
fn an_unknown_kind() {
    let error = Kind::parse("gui").expect_err("gui").to_string();
    assert!(error.contains("gui") && error.contains("tui"), "{error}");
}

#[test]
fn an_unknown_os() {
    let error = Os::parse("windows").expect_err("windows").to_string();
    assert!(
        error.contains("windows") && error.contains("macos"),
        "{error}"
    );
}

#[test]
fn the_names_read_back() {
    assert_eq!(
        (
            Kind::parse("tui").ok(),
            Os::parse("macos").ok(),
            Os::parse("linux").ok()
        ),
        (Some(Kind::Tui), Some(Os::Macos), Some(Os::Linux))
    );
}

#[test]
fn a_list_of_names() {
    let lists = [
        mu_config::listed::<&str>(&[]),
        mu_config::listed(&["a"]),
        mu_config::listed(&["a", "b"]),
        mu_config::listed(&["a", "b", "c"]),
    ];
    assert_eq!(lists, ["", "a", "a and b", "a, b and c"]);
}

#[test]
fn a_game_key_is_good() {
    assert_eq!(
        "alpha".parse(),
        Ok(WorldKey::Game(GameName::new("alpha").expect("a game")))
    );
}

#[test]
fn a_world_key_is_good() {
    let world: WorldId = "alpha/aria".parse().expect("a world id");
    assert_eq!("alpha/aria".parse(), Ok(WorldKey::World(world)));
}

#[test]
fn a_key_with_an_empty_part() {
    for text in ["", "alpha/", "/aria", "a/b/c"] {
        let error = text.parse::<WorldKey>().expect_err(text).to_string();
        assert!(error.contains(&format!("{text:?}")), "{error}");
    }
}

#[test]
fn a_key_with_an_uppercase_letter() {
    assert!("Alpha".parse::<WorldKey>().is_err());
    assert!("alpha/Aria".parse::<WorldKey>().is_err());
}

#[test]
fn a_layout_name_of_a_built_in_layout() {
    let error = LayoutName::new("classic")
        .expect_err("built in")
        .to_string();
    assert!(
        error.contains("classic") && error.contains("built-in"),
        "{error}"
    );
}

#[test]
fn a_layout_name_of_one_dot() {
    assert!(LayoutName::new(".").is_err());
    assert!(LayoutName::new("..").is_err());
    assert!(LayoutName::new("a.b").is_ok());
}

fn entries(keys: &[&str]) -> BTreeMap<String, &'static str> {
    keys.iter()
        .map(|key| ((*key).to_owned(), "focus"))
        .collect()
}

fn aria() -> WorldId {
    "alpha/aria".parse().expect("a world id")
}

#[test]
fn the_world_entry_comes_first() {
    assert_eq!(
        layout_for(Some(&aria()), &entries(&["alpha", "alpha/aria"])),
        Choice::Entry(WorldKey::World(aria()))
    );
}

#[test]
fn the_game_entry_comes_second() {
    assert_eq!(
        layout_for(Some(&aria()), &entries(&["alpha", "beta/bryn"])),
        Choice::Entry(WorldKey::Game(GameName::new("alpha").expect("a game")))
    );
}

#[test]
fn the_default_comes_last() {
    assert_eq!(
        layout_for(Some(&aria()), &entries(&["beta"])),
        Choice::Default
    );
}

#[test]
fn no_world_gives_the_default() {
    assert_eq!(layout_for(None, &entries(&["alpha"])), Choice::Default);
}

#[test]
fn a_world_key_of_129_characters_is_read() {
    let key = format!("{}/{}", "a".repeat(64), "b".repeat(64));
    assert!(key.parse::<WorldKey>().is_ok());
}

#[test]
fn a_world_key_with_a_part_of_65_characters_is_refused() {
    let key = format!("{}/{}", "a".repeat(64), "b".repeat(65));
    let text = key
        .parse::<WorldKey>()
        .expect_err("a long part")
        .to_string();
    assert!(text.contains("64"), "{text}");
}

#[test]
fn a_part_has_at_most_64_characters() {
    assert!(mu_config::is_part(&"a".repeat(64)));
    assert!(!mu_config::is_part(&"a".repeat(65)));
    // A world id, a game name and a world key hold the limit of a part.
    let long = format!("{}/b", "a".repeat(65));
    assert!(long.parse::<WorldId>().is_err());
    assert!(long.parse::<WorldKey>().is_err());
    assert!(GameName::new(&"a".repeat(65)).is_none());
    assert_eq!(mu_config::WORLD_KEY_MAX, 129);
}

#[test]
fn a_game_name_follows_the_rule_of_a_part() {
    assert!(GameName::new("alpha").is_some());
    assert!(GameName::new("Not A Game/..").is_none());
}
