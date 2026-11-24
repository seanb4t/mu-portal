//! The fields of a client layer.

use std::collections::BTreeSet;

use mu_config::{ClientLayer, FIELD_PATHS};

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
