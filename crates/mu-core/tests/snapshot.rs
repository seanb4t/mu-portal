//! The snapshot of a configuration directory.

use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::Path;
use std::process::Command;

use mu_core::world::{DirSnapshot, LINKED_DIRS_MAX, SnapEntry};

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
    fs::write(path, text).expect("write");
}

fn take(root: &Path) -> DirSnapshot {
    DirSnapshot::take(root).expect("the snapshot")
}

fn is_error(entry: Option<&SnapEntry>) -> bool {
    matches!(entry, Some(SnapEntry::Error { .. }))
}

#[test]
fn a_broken_link_under_clients_is_kept() {
    let dir = tempfile::tempdir().expect("dir");
    fs::create_dir_all(dir.path().join("clients/devices")).expect("mkdir");
    symlink(
        dir.path().join("nothing.toml"),
        dir.path().join("clients/devices/laptop.toml"),
    )
    .expect("link");
    assert!(is_error(
        take(dir.path()).get("clients/devices/laptop.toml")
    ));
}

#[test]
fn a_loop_of_links_ends() {
    let dir = tempfile::tempdir().expect("dir");
    let devices = dir.path().join("clients/devices");
    fs::create_dir_all(&devices).expect("mkdir");
    symlink(&devices, devices.join("a")).expect("link a");
    symlink(&devices, devices.join("b")).expect("link b");
    let snapshot = take(dir.path());
    assert!(
        is_error(snapshot.get("clients/devices/a")) && is_error(snapshot.get("clients/devices/b"))
    );
}

#[test]
fn a_link_to_an_unreadable_directory_keeps_its_path() {
    let dir = tempfile::tempdir().expect("dir");
    let locked = dir.path().join("locked");
    fs::create_dir(&locked).expect("mkdir");
    fs::create_dir_all(dir.path().join("clients/devices")).expect("mkdir");
    symlink(&locked, dir.path().join("clients/devices/x")).expect("link");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("chmod");
    let readable = fs::read_dir(&locked).is_ok();
    let snapshot = take(dir.path());
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o700)).expect("chmod back");
    assert!(
        !readable,
        "the mode 000 does not keep this user out: run the tests as a user that is not root"
    );
    assert!(is_error(snapshot.get("clients/devices/x")));
}

#[test]
fn a_link_to_a_sibling_directory_is_walked() {
    let dir = tempfile::tempdir().expect("dir");
    write(dir.path(), "characters/alpha/aria.toml", "host = \"a\"\n");
    symlink(
        dir.path().join("characters/alpha"),
        dir.path().join("characters/alpha2"),
    )
    .expect("link");
    assert_eq!(
        take(dir.path()).get("characters/alpha2/aria.toml"),
        Some(&SnapEntry::File(b"host = \"a\"\n".to_vec()))
    );
}

#[test]
fn a_fifo_is_not_read() {
    let dir = tempfile::tempdir().expect("dir");
    fs::create_dir_all(dir.path().join("clients/devices")).expect("mkdir");
    let status = Command::new("mkfifo")
        .arg(dir.path().join("clients/devices/laptop.toml"))
        .status()
        .expect("mkfifo");
    assert!(status.success());
    assert_eq!(
        take(dir.path()).get("clients/devices/laptop.toml"),
        Some(&SnapEntry::NotRegular)
    );
}

#[test]
fn every_file_under_clients_and_the_server_file_are_kept() {
    let dir = tempfile::tempdir().expect("dir");
    write(dir.path(), "clients/notes.txt", "x");
    write(dir.path(), "server.toml", "y");
    write(dir.path(), "notes.txt", "z");
    let snapshot = take(dir.path());
    assert_eq!(
        (
            snapshot.get("clients/notes.txt").is_some(),
            snapshot.server_file(),
            snapshot.get("notes.txt")
        ),
        (true, Some(&SnapEntry::File(b"y".to_vec())), None)
    );
}

#[test]
fn a_root_that_is_a_file_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("not-a-directory");
    fs::write(&file, "").unwrap();
    let error = DirSnapshot::take(&file).unwrap_err().to_string();
    assert!(error.contains(&file.display().to_string()), "{error}");
}

/// The number of files and of limit errors in a snapshot of `links` links to directories that
/// each hold one file.
fn linked_dirs(links: usize) -> (usize, usize) {
    let dir = tempfile::tempdir().expect("dir");
    let devices = dir.path().join("clients/devices");
    fs::create_dir_all(&devices).expect("mkdir");
    for n in 0..links {
        let target = dir.path().join("targets").join(format!("t{n:03}"));
        fs::create_dir_all(&target).expect("mkdir");
        fs::write(target.join("x.toml"), "").expect("write");
        symlink(&target, devices.join(format!("l{n:03}"))).expect("link");
    }
    let snapshot = take(dir.path());
    let files = snapshot
        .clients()
        .filter(|(_, entry)| matches!(entry, SnapEntry::File(_)))
        .count();
    let limits = snapshot
        .entries()
        .filter(|(_, entry)| matches!(entry, SnapEntry::Error { kind, .. } if kind == "limit"))
        .count();
    (files, limits)
}

#[test]
fn the_walk_enters_exactly_the_limit_of_linked_directories() {
    assert_eq!(
        (
            linked_dirs(LINKED_DIRS_MAX),
            linked_dirs(LINKED_DIRS_MAX + 1)
        ),
        ((LINKED_DIRS_MAX, 0), (LINKED_DIRS_MAX, 1))
    );
}

#[test]
fn a_mesh_of_linked_directories_stops_at_the_limit() {
    let dir = tempfile::tempdir().expect("dir");
    let devices = dir.path().join("clients/devices");
    let names = ["a", "b", "c", "d", "e", "f"];
    for name in names {
        fs::create_dir_all(dir.path().join("mesh").join(name)).expect("mkdir");
    }
    fs::create_dir_all(&devices).expect("mkdir");
    for from in names {
        symlink(dir.path().join("mesh").join(from), devices.join(from)).expect("link");
        for to in names.iter().filter(|to| **to != from) {
            symlink(
                dir.path().join("mesh").join(to),
                dir.path().join("mesh").join(from).join(to),
            )
            .expect("link");
        }
    }
    let snapshot = take(dir.path());
    let limits = snapshot
        .entries()
        .filter(|(_, entry)| matches!(entry, SnapEntry::Error { kind, .. } if kind == "limit"))
        .count();
    let walked = snapshot.entries().count();
    assert!(
        limits > 0 && walked <= LINKED_DIRS_MAX * names.len() + limits,
        "{limits} {walked}"
    );
}

#[test]
fn a_fifo_named_server_toml_is_the_server_file() {
    let dir = tempfile::tempdir().expect("dir");
    let status = Command::new("mkfifo")
        .arg(dir.path().join("server.toml"))
        .status()
        .expect("mkfifo");
    assert!(status.success());
    let snapshot = take(dir.path());
    assert!(
        snapshot.server_file() == Some(&SnapEntry::NotRegular)
            && snapshot.world_entries().count() == 0
    );
}

#[test]
fn a_broken_link_named_server_toml_is_the_server_file() {
    let dir = tempfile::tempdir().expect("dir");
    symlink(dir.path().join("nothing"), dir.path().join("server.toml")).expect("link");
    let snapshot = take(dir.path());
    assert!(is_error(snapshot.server_file()) && snapshot.world_entries().count() == 0);
}

#[test]
fn a_fifo_named_server_toml_in_another_case_is_not_opened() {
    let dir = tempfile::tempdir().expect("dir");
    let status = Command::new("mkfifo")
        .arg(dir.path().join("Server.toml"))
        .status()
        .expect("mkfifo");
    assert!(status.success());
    assert!(DirSnapshot::take(dir.path()).is_ok());
}

#[cfg(target_os = "macos")]
#[test]
fn a_broken_link_named_server_toml_in_another_case_is_the_server_file() {
    let dir = tempfile::tempdir().expect("dir");
    symlink(dir.path().join("nothing"), dir.path().join("Server.toml")).expect("link");
    let snapshot = take(dir.path());
    assert!(is_error(snapshot.server_file()) && snapshot.world_entries().count() == 0);
}
