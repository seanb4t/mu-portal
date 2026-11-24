//! The configuration calls of the API over the store.

use std::fs;
use std::path::Path;

use connectrpc::ErrorCode;
use futures::{FutureExt as _, StreamExt as _};
use mu_core::world::DirSnapshot;
use mu_server::service::{client_config, watch_stream};
use mu_server::store::Store;
use tokio::sync::watch;

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn open(root: &Path) -> Store {
    Store::open(
        Some(root),
        DirSnapshot::take(root).unwrap(),
        Box::new(|_| Ok(())),
        Box::new(|_| {}),
    )
    .unwrap()
    .store
}

fn refused(kind: &str, os: &str, device: &str) -> (ErrorCode, String) {
    let dir = tempfile::tempdir().unwrap();
    let error = client_config(&open(dir.path()).client(), kind, os, device).unwrap_err();
    (error.code, error.message.unwrap_or_default())
}

#[test]
fn a_reply_carries_the_hash_of_its_layers() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "clients/global.toml", "echo = \"hide\"\n");
    let part = open(dir.path()).client();
    let reply = client_config(&part, "tui", "linux", "laptop").unwrap();
    assert_eq!(
        (reply.store_hash.as_str(), reply.echo.as_str()),
        (part.store_hash(), "hide")
    );
}

#[test]
fn a_bad_device_name_in_a_call() {
    let (code, message) = refused("tui", "linux", "My Laptop");
    assert!(
        code == ErrorCode::InvalidArgument && message.contains("My Laptop"),
        "{message}"
    );
}

#[test]
fn a_bad_kind_in_a_call() {
    let (code, message) = refused("gui", "linux", "laptop");
    assert!(
        code == ErrorCode::InvalidArgument && message.contains("gui"),
        "{message}"
    );
}

#[test]
fn a_bad_os_in_a_call() {
    let (code, message) = refused("tui", "windows", "laptop");
    assert!(
        code == ErrorCode::InvalidArgument && message.contains("windows"),
        "{message}"
    );
}

#[tokio::test]
async fn a_watch_sends_the_current_hash() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "clients/global.toml", "echo = \"hide\"\n");
    let store = open(dir.path());
    let (_stop, stopping) = watch::channel(false);
    let mut stream = watch_stream(store.watch(), stopping);
    let first = stream.next().await.unwrap().unwrap();
    assert_eq!(first.store_hash, store.client().store_hash());
}

#[tokio::test]
async fn a_reload_sends_a_new_hash() {
    let dir = tempfile::tempdir().unwrap();
    let store = open(dir.path());
    let (_stop, stopping) = watch::channel(false);
    let mut stream = watch_stream(store.watch(), stopping);
    let first = stream.next().await.unwrap().unwrap();
    write(dir.path(), "clients/global.toml", "echo = \"hide\"\n");
    store.reload().await;
    let second = stream.next().await.unwrap().unwrap();
    assert!(
        second.store_hash != first.store_hash && second.store_hash == store.client().store_hash(),
        "{first:?} {second:?}"
    );
}

#[tokio::test]
async fn a_reload_with_no_change_sends_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let store = open(dir.path());
    let (_stop, stopping) = watch::channel(false);
    let mut stream = watch_stream(store.watch(), stopping);
    stream.next().await.unwrap().unwrap();
    write(dir.path(), "games/alpha.toml", "extends = \"tinymux\"\n");
    store.reload().await;
    assert!(stream.next().now_or_never().is_none());
}

#[tokio::test]
async fn a_fetch_on_a_new_hash_gets_the_new_revision() {
    let dir = tempfile::tempdir().unwrap();
    let store = open(dir.path());
    let before = client_config(&store.client(), "tui", "linux", "laptop").unwrap();
    let (_stop, stopping) = watch::channel(false);
    let mut stream = watch_stream(store.watch(), stopping);
    stream.next().await.unwrap().unwrap();
    write(
        dir.path(),
        "clients/devices/laptop.toml",
        "echo = \"show\"\n",
    );
    store.reload().await;
    let hash = stream.next().await.unwrap().unwrap().store_hash;
    let after = client_config(&store.client(), "tui", "linux", "laptop").unwrap();
    assert!(
        after.store_hash == hash && after.revision != before.revision && after.echo == "show",
        "{hash} {after:?}"
    );
}

#[tokio::test]
async fn a_watch_ends_at_shutdown() {
    let dir = tempfile::tempdir().unwrap();
    let store = open(dir.path());
    let (stop, stopping) = watch::channel(false);
    let mut stream = watch_stream(store.watch(), stopping);
    stream.next().await.unwrap().unwrap();
    stop.send_replace(true);
    assert!(stream.next().await.is_none());
}
