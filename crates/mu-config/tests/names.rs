//! The names of a client.

use mu_config::{DeviceName, Kind, Os};

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
