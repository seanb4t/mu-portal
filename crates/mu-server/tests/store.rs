//! The configuration store: the client resolution, the revision and the reload.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::sync::{Arc, Mutex as StdMutex};

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
fn open(root: &Path) -> Arc<Store> {
    let check = Box::new(|entry: Option<&SnapEntry>| match entry {
        Some(SnapEntry::File(bytes)) if bytes.windows(3).any(|w| w == b"bad") => {
            Err("server.toml: telemetry: bad".to_owned())
        }
        _ => Ok(()),
    });
    let opened = Store::open(
        Some(root),
        DirSnapshot::take(root).unwrap(),
        check,
        Box::new(|_| {}),
    )
    .unwrap();
    Arc::new(opened.store)
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
    let reported = Arc::new(StdMutex::new(Vec::new()));
    let sink = Arc::clone(&reported);
    let opened = Store::open(
        Some(dir.path()),
        DirSnapshot::take(dir.path()).unwrap(),
        Box::new(|_| Ok(())),
        Box::new(move |notice| sink.lock().unwrap().push(notice.path.clone())),
    )
    .unwrap();
    assert_eq!(
        (
            opened.worlds.len(),
            opened
                .notices
                .iter()
                .map(|n| n.path.as_str())
                .collect::<Vec<_>>(),
            reported.lock().unwrap().clone()
        ),
        (
            1,
            vec!["clients/extra.toml"],
            vec!["clients/extra.toml".to_owned()]
        )
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
        Box::new(|_| {}),
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

// The writes of the store.

use mu_server::write::{FieldEdit, edit_field};

#[tokio::test]
async fn a_write_that_makes_a_bad_layer_is_refused() {
    let dir = world_dir();
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"show\"\n",
    );
    let store = open(dir.path());
    // The file changes after the server read it, and no reload ran. So the store still holds a
    // good layer, and the text on the disk is a bad one.
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "[server]\nconnect_wait_seconds = 2\n",
    );
    let error = store
        .use_layout("laptop", "status", None)
        .await
        .expect_err("the edited text is a bad layer")
        .to_string();
    assert!(error.contains("clients/devices/laptop.toml"), "{error}");
    let text = fs::read_to_string(dir.path().join("clients/devices/laptop.toml")).unwrap();
    assert_eq!(text, "[server]\nconnect_wait_seconds = 2\n");
}

#[tokio::test]
async fn a_use_sets_the_layout_of_the_device() {
    let dir = world_dir();
    let store = open(dir.path());
    store
        .use_layout("laptop", "status", None)
        .await
        .expect("the use");
    assert_eq!(laptop(&store).layout_name, "status");
}

/// One line of a device layer that the edit must keep.
fn kept_line() -> impl Strategy<Value = String> {
    prop_oneof![
        "# [a-z ]{0,20}",
        Just(String::new()),
        Just("echo = \"show\"".to_owned()),
        Just("echo = \"hide\" # quiet".to_owned()),
    ]
}

proptest! {
    #[test]
    fn a_device_write_keeps_each_other_line(
        top in prop::collection::vec(kept_line(), 0..6),
        keys in prop::collection::vec("\"alt\\+[a-z]\" = \"focus-next\"", 0..3),
        entries in prop::collection::vec("[b-z][a-z]{0,6} = \"focus\"", 0..3),
        key in prop_oneof![Just("alpha/aria"), Just("alpha")],
        set_layout in any::<bool>(),
    ) {
        let mut lines = top;
        if !keys.is_empty() {
            lines.push("[keys]".to_owned());
            lines.extend(keys);
        }
        if !entries.is_empty() {
            lines.push("[world-layouts]".to_owned());
            lines.extend(entries);
        }
        let mut text = lines.join("\n");
        text.push('\n');
        prop_assume!(text.parse::<toml_edit::DocumentMut>().is_ok());
        let path: &[&str] = if set_layout { &["layout"] } else { &["world-layouts", key] };
        let set = edit_field(&text, path, &FieldEdit::Set("status".to_owned()));
        prop_assert!(set.is_ok(), "{:?}", set);
        let set = set.expect("checked");
        // Each line of the old text is in the new text, in its order.
        let mut rest = set.lines();
        for line in text.lines() {
            prop_assert!(rest.any(|each| each == line), "{:?} is gone from {:?}", line, set);
        }
        // The removal gives the old lines back. It also removes a table that it empties.
        let removed = edit_field(&set, path, &FieldEdit::Remove).expect("the removal");
        let wanted: Vec<&str> = text.lines().filter(|line| !line.is_empty()).collect();
        let found: Vec<&str> = removed.lines().filter(|line| !line.is_empty()).collect();
        prop_assert_eq!(found, wanted);
    }
}

// A request is checked before the first write.

const BAD_LAYER: &str = "echo = ";

fn lines_of(lines: &[&str]) -> Vec<String> {
    lines.iter().map(ToString::to_string).collect()
}

#[tokio::test]
async fn a_refused_save_writes_no_layout_file() {
    let dir = world_dir();
    write(dir.path(), "clients/devices/laptop.toml", BAD_LAYER);
    let store = open(dir.path());
    let error = store
        .save_layout(
            "mine",
            lines_of(&["output", "editor below main"]),
            Some("laptop"),
            None,
        )
        .await
        .expect_err("the device layer is bad")
        .to_string();
    assert!(error.contains("clients/devices/laptop.toml"), "{error}");
    assert!(!dir.path().join("clients/layouts/mine.toml").exists());
}

#[tokio::test]
async fn a_refused_save_keeps_the_old_layout_file() {
    let dir = world_dir();
    write(dir.path(), "clients/devices/laptop.toml", BAD_LAYER);
    write(
        dir.path(),
        "clients/layouts/mine.toml",
        "place = [\"editor\"]\n",
    );
    let store = open(dir.path());
    store
        .save_layout(
            "mine",
            lines_of(&["output", "editor below main"]),
            Some("laptop"),
            None,
        )
        .await
        .expect_err("the device layer is bad");
    let text = fs::read_to_string(dir.path().join("clients/layouts/mine.toml")).unwrap();
    assert_eq!(text, "place = [\"editor\"]\n");
}

#[tokio::test]
async fn a_refused_edit_of_a_built_in_layout_writes_no_layout_file() {
    use mu_config::layout::{BlockId, Change};
    let dir = world_dir();
    write(dir.path(), "clients/devices/laptop.toml", BAD_LAYER);
    let store = open(dir.path());
    let hide = Change::Hide(BlockId::new("tabs").expect("an id"));
    store
        .edit_layout("classic", hide, Some("laptop"), None)
        .await
        .expect_err("the device layer is bad");
    assert!(
        !dir.path()
            .join("clients/layouts")
            .join("laptop.toml")
            .exists()
    );
}

#[tokio::test]
async fn a_device_file_that_the_server_cannot_read_writes_no_layout_file() {
    let dir = world_dir();
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"show\"\n",
    );
    let store = open(dir.path());
    // The device file becomes a directory after the server read it. The store reads the layer
    // before its first write, so the failure comes before the layout file.
    fs::remove_file(dir.path().join("clients/devices/laptop.toml")).unwrap();
    fs::create_dir(dir.path().join("clients/devices/laptop.toml")).unwrap();
    store
        .save_layout(
            "mine",
            lines_of(&["output", "editor below main"]),
            Some("laptop"),
            None,
        )
        .await
        .expect_err("the device file is a directory");
    assert!(!dir.path().join("clients/layouts/mine.toml").exists());
}

#[tokio::test]
async fn a_save_stores_the_lines_as_mu_writes_them() {
    let dir = world_dir();
    let store = open(dir.path());
    store
        .save_layout(
            "mine",
            lines_of(&["  output  ", "editor   below   main"]),
            None,
            None,
        )
        .await
        .expect("the save");
    let text = fs::read_to_string(dir.path().join("clients/layouts/mine.toml")).unwrap();
    assert!(text.contains("\"output\""), "{text}");
    assert!(text.contains("\"editor below main\""), "{text}");
}

#[tokio::test]
async fn a_line_over_256_characters_is_refused() {
    let dir = world_dir();
    let store = open(dir.path());
    let long = format!("editor{}", " ".repeat(251));
    assert_eq!(long.chars().count(), 257);
    let error = store
        .save_layout("mine", vec![long], None, None)
        .await
        .expect_err("a long line")
        .to_string();
    assert!(error.contains("256"), "{error}");
    assert!(!dir.path().join("clients/layouts/mine.toml").exists());
}

#[tokio::test]
async fn a_key_with_a_part_over_64_characters_is_refused() {
    let dir = world_dir();
    let store = open(dir.path());
    let key = format!("{}/{}", "a".repeat(64), "b".repeat(65));
    let error = store
        .use_layout("laptop", "status", Some(&key))
        .await
        .expect_err("a long part")
        .to_string();
    assert!(error.contains("64"), "{error}");
    assert!(!dir.path().join("clients/devices/laptop.toml").exists());
}

#[tokio::test]
async fn a_removal_of_an_entry_that_is_not_there_is_refused() {
    let dir = world_dir();
    let store = open(dir.path());
    let error = store
        .use_layout("laptop", "", Some("alpha/ari"))
        .await
        .expect_err("no such entry")
        .to_string();
    assert!(error.contains("alpha/ari"), "{error}");
    assert!(!dir.path().join("clients/devices/laptop.toml").exists());
}

#[tokio::test]
async fn a_use_of_a_bad_layout_file_says_what_is_wrong() {
    let dir = world_dir();
    write(
        dir.path(),
        "clients/layouts/mine.toml",
        "place = [\"output\"]\n",
    );
    let store = open(dir.path());
    let error = store
        .use_layout("laptop", "mine", None)
        .await
        .expect_err("a plan with no editor")
        .to_string();
    assert!(error.contains("clients/layouts/mine.toml"), "{error}");
    assert!(error.contains("editor"), "{error}");
}

// A link and the permissions of a file.

#[tokio::test]
async fn a_write_to_a_linked_device_file_keeps_the_link() {
    let dir = world_dir();
    let outside = tempfile::tempdir().expect("a directory");
    let real = outside.path().join("laptop.toml");
    fs::write(&real, "# shared\necho = \"show\"\n").unwrap();
    fs::create_dir_all(dir.path().join("clients/devices")).unwrap();
    let link = dir.path().join("clients/devices/laptop.toml");
    std::os::unix::fs::symlink(&real, &link).expect("a link");
    let store = open(dir.path());
    store
        .use_layout("laptop", "status", None)
        .await
        .expect("the write");
    assert!(link.is_symlink());
    let text = fs::read_to_string(&real).unwrap();
    assert!(text.contains("# shared"), "{text}");
    assert!(text.contains("layout = \"status\""), "{text}");
}

#[tokio::test]
async fn a_write_keeps_the_permissions_of_the_file() {
    let dir = world_dir();
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"show\"\n",
    );
    let file = dir.path().join("clients/devices/laptop.toml");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o640)).unwrap();
    let store = open(dir.path());
    store
        .use_layout("laptop", "status", None)
        .await
        .expect("the write");
    assert_eq!(
        fs::metadata(&file).unwrap().permissions().mode() & 0o777,
        0o640
    );
}

#[test]
fn an_error_of_a_text_that_is_no_toml_holds_no_part_of_the_text() {
    let error = edit_field(
        "echo = \"show\"\npassword hunter2\n",
        &["layout"],
        &FieldEdit::Set("status".to_owned()),
    )
    .expect_err("no TOML");
    assert!(!error.contains("hunter2"), "{error}");
    assert!(error.contains("line 2"), "{error}");
}

// The later checks of a call, a link to something that is no file, and a call that goes away.

#[tokio::test]
async fn a_save_that_makes_a_good_layer_bad_writes_no_layout_file() {
    let dir = world_dir();
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"show\"\n",
    );
    let store = open(dir.path());
    // The layer on the disk changes after the server read it. The edit of this text gives a
    // bad layer, which only the check after the edit finds.
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "[server]\nconnect_wait_seconds = 2\n",
    );
    store
        .save_layout(
            "mine",
            lines_of(&["output", "editor below main"]),
            Some("laptop"),
            None,
        )
        .await
        .expect_err("the edited text is a bad layer");
    assert!(!dir.path().join("clients/layouts/mine.toml").exists());
}

#[tokio::test]
async fn a_refused_call_makes_no_directory() {
    let dir = world_dir();
    let store = open(dir.path());
    store
        .use_layout("laptop", "", Some("alpha/ari"))
        .await
        .expect_err("no such entry");
    assert!(!dir.path().join("clients").exists());
}

#[tokio::test]
async fn a_removal_of_a_missing_entry_is_refused_for_each_form_of_the_file() {
    // The file has no final newline, CRLF line ends, or an empty table.
    for text in [
        "layout = \"status\"",
        "layout = \"status\"\r\necho = \"show\"\r\n",
        "layout = \"status\"\n\n[world-layouts]\n",
    ] {
        let dir = world_dir();
        write(dir.path(), "clients/devices/laptop.toml", text);
        let store = open(dir.path());
        store
            .use_layout("laptop", "", Some("alpha/aria"))
            .await
            .expect_err("no such entry");
        let after = fs::read_to_string(dir.path().join("clients/devices/laptop.toml")).unwrap();
        assert_eq!(after, text);
    }
}

#[tokio::test]
async fn a_link_to_a_directory_is_not_written() {
    let dir = world_dir();
    let outside = tempfile::tempdir().expect("a directory");
    fs::write(outside.path().join("keep.txt"), "keep").unwrap();
    fs::create_dir_all(dir.path().join("clients/layouts")).unwrap();
    let link = dir.path().join("clients/layouts/mine.toml");
    std::os::unix::fs::symlink(outside.path(), &link).expect("a link");
    let store = open(dir.path());
    let error = store
        .save_layout(
            "mine",
            lines_of(&["output", "editor below main"]),
            None,
            None,
        )
        .await
        .expect_err("a link to a directory")
        .to_string();
    assert!(error.contains("clients/layouts/mine.toml"), "{error}");
    assert!(error.contains("no regular file"), "{error}");
    // The error holds no path outside the configuration directory.
    assert!(
        !error.contains(&outside.path().display().to_string()),
        "{error}"
    );
    assert!(link.is_symlink());
    assert_eq!(
        fs::read_to_string(outside.path().join("keep.txt")).unwrap(),
        "keep"
    );
}

#[tokio::test]
async fn a_link_to_nothing_is_not_written() {
    let dir = world_dir();
    fs::create_dir_all(dir.path().join("clients/layouts")).unwrap();
    let link = dir.path().join("clients/layouts/mine.toml");
    std::os::unix::fs::symlink(dir.path().join("gone.toml"), &link).expect("a link");
    let store = open(dir.path());
    let error = store
        .save_layout(
            "mine",
            lines_of(&["output", "editor below main"]),
            None,
            None,
        )
        .await
        .expect_err("a link to nothing")
        .to_string();
    assert!(error.contains("a link to nothing"), "{error}");
    assert!(!dir.path().join("gone.toml").exists());
}

#[tokio::test]
async fn a_write_whose_caller_goes_away_still_ends_with_a_reload() {
    let dir = world_dir();
    let store = open(dir.path());
    let mut watch = store.watch();
    let before = watch.borrow_and_update().store_hash().to_owned();
    {
        let save = store.save_layout(
            "mine",
            lines_of(&["output", "editor below main"]),
            None,
            None,
        );
        tokio::pin!(save);
        // One poll starts the write. The caller then drops the call, as a call with a
        // deadline that ends does.
        tokio::select! {
            biased;
            _ = &mut save => {}
            () = std::future::ready(()) => {}
        }
    }
    // The store still writes the file and reads the directory again.
    watch.changed().await.expect("the store lives");
    assert_ne!(watch.borrow().store_hash(), before);
    assert!(dir.path().join("clients/layouts/mine.toml").exists());
}
