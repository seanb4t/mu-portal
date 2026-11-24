//! The scenarios of the config-store capability, run on the built binary.

use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::Path;
use std::process::Command;

use support::ConfigDir;
use support::process::{ServerBuilder, ServerProcess};

mod support;

/// Starts a server over `dir` and waits until it serves.
async fn start(dir: &Path) -> ServerProcess {
    ServerBuilder::new().arg("--config").arg(dir).start().await
}

/// The lines of stderr that contain `text`.
fn lines_with(server: &ServerProcess, text: &str) -> Vec<String> {
    server
        .stderr()
        .into_iter()
        .filter(|line| line.contains(text))
        .collect()
}

/// Starts a server over `dir`, and asserts one line of stderr that holds each of `texts` and an
/// API call that succeeds.
async fn one_notice(dir: &Path, texts: &[&str]) {
    let server = start(dir).await;
    let lines = lines_with(&server, texts[0]);
    assert!(
        lines.len() == 1 && texts.iter().all(|text| lines[0].contains(text)),
        "{:?}",
        server.stderr()
    );
    server.client().list_worlds().await.unwrap();
}

/// Sets the mode 000 on `path`. False when this process can still list or read it, as root can.
fn locked(path: &Path) -> bool {
    fs::set_permissions(path, fs::Permissions::from_mode(0o000)).unwrap();
    let locked = fs::read_dir(path).is_err();
    if !locked {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    locked
}

#[tokio::test]
async fn a_bad_device_layer_at_start() {
    let dir = ConfigDir::base().file("clients/devices/laptop.toml", "echo = \"loud\"\n");
    let server = start(dir.path()).await;
    let lines = lines_with(&server, "clients/devices/laptop.toml");
    assert!(
        lines.len() == 1 && lines[0].contains("echo"),
        "{:?}",
        server.stderr()
    );
    let worlds = server.client().list_worlds().await.unwrap();
    assert!(worlds.iter().any(|world| world.id == "alpha/aria"));
}

#[tokio::test]
async fn a_file_that_is_not_a_layer() {
    let dir = ConfigDir::empty().file("clients/extra.toml", "echo = \"show\"\n");
    one_notice(dir.path(), &["clients/extra.toml"]).await;
}

#[tokio::test]
async fn a_device_file_with_a_bad_name() {
    let dir = ConfigDir::empty().file("clients/devices/Laptop.toml", "echo = \"show\"\n");
    one_notice(dir.path(), &["clients/devices/Laptop.toml"]).await;
}

#[tokio::test]
async fn a_layer_for_an_unknown_kind() {
    let dir = ConfigDir::empty().file("clients/kinds/tiu.toml", "echo = \"show\"\n");
    let server = start(dir.path()).await;
    let lines = lines_with(&server, "clients/kinds/tiu.toml");
    assert!(
        lines.len() == 1 && lines[0].contains("\"tiu\""),
        "{:?}",
        server.stderr()
    );
}

#[tokio::test]
async fn a_layer_for_an_unknown_os() {
    let dir = ConfigDir::empty().file("clients/os/windows.toml", "echo = \"show\"\n");
    let server = start(dir.path()).await;
    let lines = lines_with(&server, "clients/os/windows.toml");
    assert!(
        lines.len() == 1 && lines[0].contains("\"windows\""),
        "{:?}",
        server.stderr()
    );
}

#[tokio::test]
async fn a_link_to_nothing_under_clients() {
    let dir = ConfigDir::empty();
    fs::create_dir_all(dir.path().join("clients/devices")).unwrap();
    symlink(
        dir.path().join("nothing.toml"),
        dir.path().join("clients/devices/laptop.toml"),
    )
    .unwrap();
    one_notice(dir.path(), &["clients/devices/laptop.toml"]).await;
}

#[tokio::test]
async fn a_file_that_is_not_toml() {
    let dir = ConfigDir::empty().file("clients/notes.txt", "notes\n");
    one_notice(dir.path(), &["clients/notes.txt"]).await;
}

#[tokio::test]
async fn a_loop_of_links_under_clients() {
    let dir = ConfigDir::empty();
    let devices = dir.path().join("clients/devices");
    fs::create_dir_all(&devices).unwrap();
    symlink(&devices, devices.join("a")).unwrap();
    symlink(&devices, devices.join("b")).unwrap();
    one_notice(dir.path(), &["clients/devices/a"]).await;
}

#[tokio::test]
async fn an_unlistable_clients_directory() {
    let dir = ConfigDir::empty();
    let clients = dir.path().join("clients");
    fs::create_dir(&clients).unwrap();
    if !locked(&clients) {
        return;
    }
    let server = start(dir.path()).await;
    fs::set_permissions(&clients, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        lines_with(&server, "clients").len(),
        1,
        "{:?}",
        server.stderr()
    );
    server.client().list_worlds().await.unwrap();
}

#[tokio::test]
async fn a_link_to_an_unreadable_directory_under_clients() {
    // The locked directory is outside the configuration directory: an unreadable directory
    // among the world files stops the start.
    let dir = ConfigDir::empty();
    let outside = tempfile::tempdir().unwrap();
    let locked_dir = outside.path().join("locked");
    fs::create_dir(&locked_dir).unwrap();
    fs::create_dir_all(dir.path().join("clients/devices")).unwrap();
    symlink(&locked_dir, dir.path().join("clients/devices/x")).unwrap();
    if !locked(&locked_dir) {
        return;
    }
    let server = start(dir.path()).await;
    fs::set_permissions(&locked_dir, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        !lines_with(&server, "clients/devices/x").is_empty(),
        "{:?}",
        server.stderr()
    );
    server.client().list_worlds().await.unwrap();
}

#[tokio::test]
async fn a_control_character_in_a_layer_name() {
    let dir = ConfigDir::empty().file("clients/devices/lap\u{1b}.toml", "echo = \"show\"\n");
    let server = start(dir.path()).await;
    let stderr = server.stderr().join("\n");
    assert!(
        !stderr.contains('\u{1b}') && stderr.contains('\u{fffd}'),
        "{stderr:?}"
    );
}

#[tokio::test]
async fn a_line_break_in_a_layer_name() {
    let dir = ConfigDir::empty().file("clients/devices/lap\n.toml", "echo = \"show\"\n");
    let server = start(dir.path()).await;
    assert_eq!(
        lines_with(&server, "clients/devices/lap").len(),
        1,
        "{:?}",
        server.stderr()
    );
}

#[tokio::test]
async fn a_line_break_in_a_binding() {
    let dir = ConfigDir::empty().file("clients/global.toml", "[keys]\n\"alt+\\n\" = \"quit\"\n");
    let server = start(dir.path()).await;
    assert_eq!(
        lines_with(&server, "clients/global.toml").len(),
        1,
        "{:?}",
        server.stderr()
    );
}

#[tokio::test]
async fn a_fifo_under_clients() {
    let dir = ConfigDir::empty();
    fs::create_dir_all(dir.path().join("clients/devices")).unwrap();
    let status = Command::new("mkfifo")
        .arg(dir.path().join("clients/devices/laptop.toml"))
        .status()
        .unwrap();
    assert!(status.success());
    one_notice(dir.path(), &["clients/devices/laptop.toml"]).await;
}

#[tokio::test]
async fn a_bad_world_still_stops_the_start() {
    let dir = ConfigDir::base().file("characters/alpha/aria.toml", "port = 0\n");
    let output = ServerBuilder::new()
        .arg("--config")
        .arg(dir.path())
        .output()
        .await;
    assert_eq!(output.status.code(), Some(1));
}
