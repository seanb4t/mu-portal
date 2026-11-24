//! The configuration store: the client resolution, the revision and the reload.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use mu_config::{DeviceName, Echo, Kind, Os};
use mu_core::world::{DirSnapshot, SnapEntry};
use mu_server::store::{ClientConfig, ClientPart, Content, Store, framed};
use proptest::prelude::*;

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn part(root: &Path) -> ClientPart {
    ClientPart::new(&DirSnapshot::take(root).unwrap())
}

fn client(root: &Path, device: &str) -> ClientConfig {
    part(root).resolve(Kind::Tui, Os::Linux, &DeviceName::new(device).unwrap())
}

fn revision(root: &Path, device: &str) -> String {
    client(root, device).revision
}

fn source<'a>(config: &'a ClientConfig, path: &str) -> &'a str {
    config.sources.get(path).map_or("", String::as_str)
}

fn binding<'a>(config: &'a ClientConfig, key: &str) -> Option<&'a str> {
    config
        .bindings
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, entry)| entry.as_str())
}

/// Sets the mode 000 on `path`, and fails the test if the mode does not keep this process out,
/// as for root.
fn lock(path: &Path) {
    fs::set_permissions(path, fs::Permissions::from_mode(0o000)).unwrap();
    let locked = if path.is_dir() {
        fs::read_dir(path).is_err()
    } else {
        fs::read(path).is_err()
    };
    if !locked {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    assert!(
        locked,
        "the mode 000 does not keep this user out: run the tests as a user that is not root"
    );
}

#[test]
fn the_device_wins() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "clients/global.toml", "echo = \"show\"\n");
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"hide\"\n",
    );
    let config = client(dir.path(), "laptop");
    assert_eq!(
        (config.echo, source(&config, "echo")),
        (Echo::Hide, "clients/devices/laptop.toml")
    );
}

#[test]
fn an_unset_echo_keeps_the_earlier_echo() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "clients/global.toml", "echo = \"hide\"\n");
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "[keys]\n\"alt+n\" = \"focus-next\"\n",
    );
    let config = client(dir.path(), "laptop");
    assert_eq!(
        (config.echo, source(&config, "echo")),
        (Echo::Hide, "clients/global.toml")
    );
}

#[test]
fn the_defaults_are_the_source_without_a_layer() {
    let dir = tempfile::tempdir().unwrap();
    let config = client(dir.path(), "laptop");
    assert_eq!(
        (config.echo, source(&config, "echo")),
        (Echo::Dim, "default")
    );
}

#[test]
fn keys_merge_key_by_key() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "clients/global.toml",
        "[keys]\n\"alt+n\" = \"focus-next\"\n\"alt+p\" = \"focus-prev\"\n",
    );
    write(
        dir.path(),
        "clients/os/linux.toml",
        "[keys]\n\"alt+p\" = \"none\"\n",
    );
    let config = client(dir.path(), "laptop");
    assert_eq!(
        [
            (binding(&config, "alt+n"), source(&config, "keys.alt+n")),
            (binding(&config, "alt+p"), source(&config, "keys.alt+p")),
        ],
        [
            (Some("focus-next"), "clients/global.toml"),
            (Some("none"), "clients/os/linux.toml"),
        ]
    );
}

#[test]
fn two_names_across_layers_are_one_key() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "clients/global.toml",
        "[keys]\n\"ctrl+shift+z\" = \"quit\"\n",
    );
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "[keys]\n\"shift+ctrl+z\" = \"none\"\n",
    );
    let config = client(dir.path(), "laptop");
    assert_eq!(
        (
            config.bindings.clone(),
            source(&config, "keys.ctrl+shift+z")
        ),
        (
            vec![("ctrl+shift+z".to_owned(), "none".to_owned())],
            "clients/devices/laptop.toml"
        )
    );
}

#[test]
fn the_key_period_merges_as_one_key() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "clients/global.toml",
        "[keys]\n\"alt+.\" = \"quit\"\n",
    );
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "[keys]\n\"alt+period\" = \"none\"\n",
    );
    let config = client(dir.path(), "laptop");
    assert_eq!(
        (config.bindings.clone(), source(&config, "keys.alt+period")),
        (
            vec![("alt+period".to_owned(), "none".to_owned())],
            "clients/devices/laptop.toml"
        )
    );
}

#[test]
fn a_bad_layer_adds_nothing() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "clients/global.toml", "echo = \"show\"\n");
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"hide\"\n[keys]\n\"alt+n\" = \"fly\"\n",
    );
    let config = client(dir.path(), "laptop");
    assert_eq!(
        (
            config.echo,
            binding(&config, "alt+n"),
            config.notices.len(),
            config.notices[0].path.as_str()
        ),
        (Echo::Show, None, 1, "clients/devices/laptop.toml")
    );
}

#[test]
fn a_bad_layer_of_another_device_gives_no_notice() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"loud\"\n",
    );
    assert_eq!(client(dir.path(), "desk").notices, []);
}

#[test]
fn a_notice_holds_no_control_character() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "clients/glo\u{1b}bal.toml", "echo = \"show\"\n");
    let notices = part(dir.path()).notices();
    assert!(
        notices.len() == 1 && !notices[0].text.chars().any(char::is_control),
        "{notices:?}"
    );
}

#[test]
fn a_device_file_with_a_bad_name_is_a_notice() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "clients/devices/Laptop.toml",
        "echo = \"show\"\n",
    );
    let notices = part(dir.path()).notices();
    assert_eq!(
        notices.iter().map(|n| n.path.as_str()).collect::<Vec<_>>(),
        ["clients/devices/Laptop.toml"]
    );
}

#[test]
fn the_revision_ignores_other_devices() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"hide\"\n",
    );
    let before = revision(dir.path(), "laptop");
    write(dir.path(), "clients/devices/desk.toml", "echo = \"show\"\n");
    assert_eq!(revision(dir.path(), "laptop"), before);
}

#[test]
fn a_changed_layer_changes_the_revision() {
    let dir = tempfile::tempdir().unwrap();
    let before = revision(dir.path(), "laptop");
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"hide\"\n",
    );
    assert_ne!(revision(dir.path(), "laptop"), before);
}

#[test]
fn a_bad_layer_changes_the_revision() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"hide\"\n",
    );
    let before = revision(dir.path(), "laptop");
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"loud\"\n",
    );
    assert_ne!(revision(dir.path(), "laptop"), before);
}

#[test]
fn the_revision_frames_each_layer() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "clients/global.toml", "echo = \"show\"\n");
    write(dir.path(), "clients/kinds/tui.toml", "[keys]\n");
    let before = revision(dir.path(), "laptop");
    write(
        dir.path(),
        "clients/global.toml",
        "echo = \"show\"\n[keys]\n",
    );
    write(dir.path(), "clients/kinds/tui.toml", "");
    assert_ne!(revision(dir.path(), "laptop"), before);
}

#[test]
fn a_layer_that_holds_the_next_entry_changes_the_revision() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "clients/global.toml", "echo = \"show\"\n");
    write(dir.path(), "clients/kinds/tui.toml", "[keys]\n");
    let before = revision(dir.path(), "laptop");
    write(
        dir.path(),
        "clients/global.toml",
        "echo = \"show\"\nclients/kinds/tui.toml\0[keys]\n",
    );
    fs::remove_file(dir.path().join("clients/kinds/tui.toml")).unwrap();
    assert_ne!(revision(dir.path(), "laptop"), before);
}

#[test]
fn an_unreadable_layer_changes_the_revision() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "clients/devices/laptop.toml", "");
    let before = revision(dir.path(), "laptop");
    let path = dir.path().join("clients/devices/laptop.toml");
    lock(&path);
    let after = client(dir.path(), "laptop");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(
        after.revision != before && after.notices.len() == 1,
        "{after:?}"
    );
}

#[test]
fn an_unlistable_clients_directory_changes_the_revision() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("clients")).unwrap();
    let before = revision(dir.path(), "laptop");
    let clients = dir.path().join("clients");
    lock(&clients);
    let after = client(dir.path(), "laptop");
    fs::set_permissions(&clients, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        after.revision != before && after.notices.iter().any(|notice| notice.path == "clients"),
        "{after:?}"
    );
}

#[test]
fn a_file_at_a_layer_directory_changes_the_revision() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("clients/devices")).unwrap();
    let before = revision(dir.path(), "laptop");
    fs::remove_dir(dir.path().join("clients/devices")).unwrap();
    write(dir.path(), "clients/devices", "");
    let after = client(dir.path(), "laptop");
    assert!(
        after.revision != before
            && after
                .notices
                .iter()
                .any(|notice| notice.path == "clients/devices"),
        "{after:?}"
    );
}

#[test]
fn the_same_files_give_the_same_revision_and_store_hash() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"hide\"\n",
    );
    let (first, second) = (client(dir.path(), "laptop"), client(dir.path(), "laptop"));
    assert_eq!(
        (first.revision, first.store_hash),
        (second.revision, second.store_hash)
    );
}

/// One entry of a generated list: a path with bytes, or with the kind of a read error. The
/// small alphabet, with the tag bytes in it, makes two lists with one concatenation likely.
fn entry() -> impl Strategy<Value = (String, Result<Vec<u8>, String>)> {
    (
        "[a\\x00]{0,2}",
        prop_oneof![
            proptest::collection::vec(prop_oneof![Just(0_u8), Just(b'a')], 0..3).prop_map(Ok),
            "[a\\x00]{0,2}".prop_map(Err),
        ],
    )
}

fn input(list: &[(String, Result<Vec<u8>, String>)]) -> Vec<u8> {
    framed(list.iter().map(|(path, content)| {
        (
            path.as_str(),
            match content {
                Ok(bytes) => Content::Read(bytes),
                Err(kind) => Content::Unreadable(kind),
            },
        )
    }))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4096))]

    #[test]
    fn two_layer_lists_give_two_inputs(
        first in proptest::collection::vec(entry(), 0..3),
        second in proptest::collection::vec(entry(), 0..3),
    ) {
        prop_assume!(first != second);
        prop_assert_ne!(input(&first), input(&second));
    }
}

/// A directory with the world `alpha/aria`.
fn world_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "games/alpha.toml",
        "extends = \"tinymux\"\nhost = \"localhost\"\nport = 4201\n",
    );
    write(dir.path(), "characters/alpha/aria.toml", "");
    dir
}

/// The store of `root`, whose check of `server.toml` refuses a file that holds `bad`.
fn open(root: &Path) -> Store {
    let check = Box::new(|entry: Option<&SnapEntry>| match entry {
        Some(SnapEntry::File(bytes)) if bytes.windows(3).any(|w| w == b"bad") => {
            Err("server.toml: telemetry: bad".to_owned())
        }
        _ => Ok(()),
    });
    Store::open(Some(root), DirSnapshot::take(root).unwrap(), check)
        .unwrap()
        .store
}

fn laptop(store: &Store) -> ClientConfig {
    store
        .client()
        .resolve(Kind::Tui, Os::Linux, &DeviceName::new("laptop").unwrap())
}

#[test]
fn open_resolves_the_worlds_and_returns_the_notices() {
    let dir = world_dir();
    write(dir.path(), "clients/extra.toml", "echo = \"show\"\n");
    let opened = Store::open(
        Some(dir.path()),
        DirSnapshot::take(dir.path()).unwrap(),
        Box::new(|_| Ok(())),
    )
    .unwrap();
    assert_eq!(
        (
            opened.worlds.len(),
            opened
                .notices
                .iter()
                .map(|n| n.path.as_str())
                .collect::<Vec<_>>()
        ),
        (1, vec!["clients/extra.toml"])
    );
}

#[test]
fn a_bad_world_stops_open() {
    let dir = world_dir();
    write(dir.path(), "characters/alpha/aria.toml", "port = 0\n");
    let opened = Store::open(
        Some(dir.path()),
        DirSnapshot::take(dir.path()).unwrap(),
        Box::new(|_| Ok(())),
    );
    assert!(opened.is_err());
}

#[tokio::test]
async fn a_reload_swaps_the_client_layers() {
    let dir = world_dir();
    let store = open(dir.path());
    let watch = store.watch();
    let before = laptop(&store);
    write(dir.path(), "clients/global.toml", "echo = \"hide\"\n");
    let reloaded = store.reload().await;
    let after = laptop(&store);
    assert_eq!(
        (
            reloaded.notices.is_empty(),
            watch.has_changed().unwrap(),
            after.echo,
            after.store_hash != before.store_hash
        ),
        (true, true, Echo::Hide, true)
    );
}

#[tokio::test]
async fn a_reload_with_no_change_sends_no_value() {
    let dir = world_dir();
    let store = open(dir.path());
    let watch = store.watch();
    store.reload().await;
    assert!(!watch.has_changed().unwrap());
}

#[tokio::test]
async fn a_reload_keeps_the_worlds() {
    let dir = world_dir();
    let store = open(dir.path());
    write(
        dir.path(),
        "games/beta.toml",
        "extends = \"tinymux\"\nhost = \"localhost\"\nport = 4201\n",
    );
    write(dir.path(), "characters/beta/bryn.toml", "");
    let reloaded = store.reload().await;
    assert_eq!(
        (reloaded.notices.len(), reloaded.restart),
        (
            0,
            vec![
                "characters/beta/bryn.toml".to_owned(),
                "games/beta.toml".to_owned()
            ]
        )
    );
}

#[tokio::test]
async fn a_reload_reports_a_bad_world_file() {
    let dir = world_dir();
    let store = open(dir.path());
    write(dir.path(), "characters/alpha/aria.toml", "port = 0\n");
    write(dir.path(), "clients/global.toml", "echo = \"hide\"\n");
    let reloaded = store.reload().await;
    assert_eq!(
        (
            reloaded
                .notices
                .iter()
                .map(|n| n.path.as_str())
                .collect::<Vec<_>>(),
            reloaded.notices[0].text.contains("port"),
            laptop(&store).echo
        ),
        (vec!["characters/alpha/aria.toml"], true, Echo::Hide)
    );
}

#[tokio::test]
async fn a_reload_reports_two_bad_world_files() {
    let dir = world_dir();
    write(dir.path(), "characters/alpha/ann.toml", "");
    let store = open(dir.path());
    write(dir.path(), "characters/alpha/aria.toml", "port = 0\n");
    write(dir.path(), "characters/alpha/ann.toml", "port = 0\n");
    let reloaded = store.reload().await;
    assert_eq!(
        reloaded
            .notices
            .iter()
            .map(|n| n.path.as_str())
            .collect::<Vec<_>>(),
        ["characters/alpha/ann.toml", "characters/alpha/aria.toml"]
    );
}

#[tokio::test]
async fn a_reload_reports_a_bad_game_once() {
    let dir = world_dir();
    write(dir.path(), "characters/alpha/ann.toml", "");
    let store = open(dir.path());
    write(
        dir.path(),
        "games/alpha.toml",
        "extends = \"tinymux\"\nport = 0\n",
    );
    let reloaded = store.reload().await;
    assert_eq!(
        reloaded
            .notices
            .iter()
            .map(|n| n.path.as_str())
            .collect::<Vec<_>>(),
        ["games/alpha.toml"]
    );
}

#[tokio::test]
async fn a_reload_reports_two_stray_files() {
    let dir = world_dir();
    let store = open(dir.path());
    write(dir.path(), "worlds.toml", "");
    write(dir.path(), "games/Beta.toml", "");
    let reloaded = store.reload().await;
    assert_eq!(
        reloaded
            .notices
            .iter()
            .map(|n| n.path.as_str())
            .collect::<Vec<_>>(),
        ["games/Beta.toml", "worlds.toml"]
    );
}

#[tokio::test]
async fn a_reload_reports_a_bad_server_file() {
    let dir = world_dir();
    let store = open(dir.path());
    write(dir.path(), "server.toml", "bad\n");
    let reloaded = store.reload().await;
    assert_eq!(
        reloaded
            .notices
            .iter()
            .map(|n| n.path.as_str())
            .collect::<Vec<_>>(),
        ["server.toml"]
    );
}

#[tokio::test]
async fn a_reload_lists_a_changed_server_file() {
    let dir = world_dir();
    write(dir.path(), "server.toml", "");
    let store = open(dir.path());
    write(dir.path(), "server.toml", "[telemetry]\n");
    assert_eq!(store.reload().await.restart, ["server.toml"]);
}

#[tokio::test]
async fn a_second_reload_lists_the_world_change_again() {
    let dir = world_dir();
    let store = open(dir.path());
    write(dir.path(), "characters/alpha/aria.toml", "port = 4202\n");
    store.reload().await;
    assert_eq!(store.reload().await.restart, ["characters/alpha/aria.toml"]);
}
