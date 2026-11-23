//! The scenarios of the server-process capability that need no terminal, run on the built
//! binary.

use std::fs;
use std::os::unix::fs::{DirBuilderExt as _, FileTypeExt as _, PermissionsExt as _};
use std::path::Path;

use mu_client::{ClientEvent, PortalClient};
use mu_proto::__buffa::oneof::subscribe_response::Message;
use mu_proto::EventKind;
use rustix::process::Signal;

use support::process::{
    BOUND, PtyOn, ServerBuilder, TWO_WORLDS, mu_on_pty, private_tempdir, wait_until,
};
use support::{mode, mu_with, stderr};

mod support;

async fn lists_worlds(socket: &Path) {
    let worlds = wait_until(
        BOUND,
        "ListWorlds reply",
        PortalClient::unix(socket).list_worlds(),
    )
    .await;
    assert_eq!(
        worlds.map(|worlds| worlds.len()),
        Ok(0),
        "ListWorlds on {}",
        socket.display()
    );
}

fn text(path: &Path) -> String {
    path.display().to_string()
}

#[tokio::test]
async fn serves_on_named_socket() {
    let server = ServerBuilder::new().start().await;
    let socket = server.dir().path().join("s.sock");
    let lines = server.stderr();
    assert!(lines.iter().any(|line| line.contains(&text(&socket))));
    assert!(lines.iter().all(|line| !line.contains("not implemented")));
    lists_worlds(&socket).await;
}

#[tokio::test]
async fn env_names_socket() {
    let builder = ServerBuilder::new().socket_flag(None);
    let socket = builder.dir().join("env.sock");
    let _server = builder.env("MU_SOCKET", Some(&socket)).start().await;
    lists_worlds(&socket).await;
}

#[tokio::test]
async fn flag_wins_over_env() {
    let builder = ServerBuilder::new();
    let env = builder.dir().join("env.sock");
    let flag = builder.dir().join("flag.sock");
    let _server = builder
        .socket_flag(Some(flag.clone()))
        .env("MU_SOCKET", Some(&env))
        .start()
        .await;
    lists_worlds(&flag).await;
    assert!(!env.exists());
}

#[tokio::test]
async fn default_socket_under_runtime_dir() {
    let builder = ServerBuilder::new().socket_flag(None);
    let dir = builder.dir().to_owned();
    let _server = builder.env("XDG_RUNTIME_DIR", Some(&dir)).start().await;
    lists_worlds(&dir.join("mu/server.sock")).await;
}

#[tokio::test]
async fn empty_runtime_dir_is_ignored() {
    let builder = ServerBuilder::new().socket_flag(None);
    let work = builder.dir().join("work");
    let tmp = builder.dir().join("tmp");
    fs::create_dir(&work).unwrap();
    fs::create_dir(&tmp).unwrap();
    let server = builder
        .current_dir(&work)
        .env("XDG_RUNTIME_DIR", Some(""))
        .env("TMPDIR", Some(&tmp))
        .start()
        .await;
    assert!(server.socket().is_absolute());
    assert!(
        server.socket().starts_with(&tmp),
        "{}",
        text(server.socket())
    );
    assert!(!work.join("mu").exists());
    lists_worlds(server.socket()).await;
}

#[tokio::test]
async fn empty_socket_env_is_ignored() {
    let builder = ServerBuilder::new().socket_flag(None);
    let dir = builder.dir().to_owned();
    let _server = builder
        .env("MU_SOCKET", Some(""))
        .env("XDG_RUNTIME_DIR", Some(&dir))
        .start()
        .await;
    lists_worlds(&dir.join("mu/server.sock")).await;
}

#[tokio::test]
async fn new_socket_dir_is_private() {
    let builder = ServerBuilder::new();
    let socket = builder.dir().join("run/s.sock");
    let _server = builder.socket_flag(Some(socket.clone())).start().await;
    assert_eq!(mode(socket.parent().unwrap()), 0o700);
    assert_eq!(mode(&socket), 0o600);
}

#[test]
fn open_socket_dir_is_refused() {
    for open in [0o740, 0o720, 0o710, 0o704, 0o702, 0o701, 0o755] {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("run");
        fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(open)).unwrap();
        let socket = dir.join("s.sock");
        let output = mu_with(
            &["server", "--socket", &text(&socket)],
            &[("MU_SOCKET", None)],
        );
        assert_eq!(output.status.code(), Some(1), "mode {open:o}");
        assert!(stderr(&output).contains(&text(&dir)), "mode {open:o}");
        assert!(!socket.exists(), "mode {open:o}");
    }
}

#[tokio::test]
async fn second_server_is_refused() {
    let server = ServerBuilder::new().start().await;
    let socket = text(server.socket());
    let second = mu_with(&["server", "--socket", &socket], &[("MU_SOCKET", None)]);
    assert_eq!(second.status.code(), Some(1));
    let second = stderr(&second);
    assert!(
        second.contains(&socket) && second.contains("already"),
        "{second}"
    );
    lists_worlds(server.socket()).await;
}

#[tokio::test]
async fn stale_socket_is_replaced() {
    let builder = ServerBuilder::new();
    let socket = builder.dir().join("s.sock");
    drop(std::os::unix::net::UnixListener::bind(&socket).unwrap());
    assert!(
        fs::symlink_metadata(&socket)
            .unwrap()
            .file_type()
            .is_socket()
    );
    let server = builder.start().await;
    lists_worlds(server.socket()).await;
}

#[test]
fn regular_file_at_socket_path_is_kept() {
    let dir = private_tempdir();
    let socket = dir.path().join("s.sock");
    fs::write(&socket, "keep").unwrap();
    let output = mu_with(
        &["server", "--socket", &text(&socket)],
        &[("MU_SOCKET", None)],
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = stderr(&output);
    assert!(stderr.contains(&text(&socket)), "{stderr}");
    assert_eq!(fs::read_to_string(&socket).unwrap(), "keep");
}

async fn stops_on(signal: Signal) {
    let mut server = ServerBuilder::new().start().await;
    let mut events = Box::pin(server.client().events());
    let first = wait_until(
        BOUND,
        "first stream item",
        futures::StreamExt::next(&mut events),
    )
    .await;
    assert!(
        matches!(first, Some(ClientEvent::Reset { .. })),
        "{first:?}"
    );
    server.signal(signal);
    let end = wait_until(
        BOUND,
        "end of the stream",
        futures::StreamExt::next(&mut events),
    )
    .await;
    assert_eq!(end, Some(ClientEvent::Lost));
    assert!(server.wait_exit().await.success());
    assert!(!server.socket().exists());
}

#[tokio::test]
async fn sigterm_sends_the_disconnects_first() {
    let mut server = ServerBuilder::new().fake_worlds(TWO_WORLDS).start().await;
    let mut stream = server.client().subscribe(0).await.unwrap();
    let mut greeted = 0;
    while greeted < 2 {
        let message = wait_until(BOUND, "a greeting", stream.next())
            .await
            .unwrap();
        if let Some(Message::Event(event)) = message.and_then(|message| message.message) {
            greeted += usize::from(event.text.starts_with("Welcome, "));
        }
    }
    server.signal(Signal::TERM);
    let mut gone = Vec::new();
    while let Ok(Some(message)) = wait_until(BOUND, "the end of the stream", stream.next()).await {
        if let Some(Message::Event(event)) = message.message
            && event.kind == EventKind::EVENT_KIND_DISCONNECTED
        {
            gone.push(event.world);
        }
    }
    gone.sort();
    assert_eq!(gone, ["alpha/aria", "beta/bryn"]);
    assert!(server.wait_exit().await.success());
}

#[tokio::test]
async fn bare_socket_name_is_in_the_working_directory() {
    let builder = ServerBuilder::new().socket_flag(Some("s.sock".into()));
    let dir = builder.dir().to_owned();
    let server = builder.current_dir(&dir).start().await;
    assert_eq!(server.socket(), dir.canonicalize().unwrap().join("s.sock"));
    lists_worlds(server.socket()).await;
}

#[tokio::test]
async fn sigterm_stops_server() {
    stops_on(Signal::TERM).await;
}

#[tokio::test]
async fn sigint_stops_server() {
    stops_on(Signal::INT).await;
}

#[test]
fn client_needs_a_terminal() {
    for on in [PtyOn::Stdin, PtyOn::Stdout] {
        let dir = tempfile::tempdir().unwrap();
        let socket = text(&dir.path().join("s.sock"));
        let output = mu_on_pty(&["client", "--socket", &socket], on, dir.path());
        let stderr = stderr(&output);
        assert_eq!(output.status.code(), Some(1), "{on:?}");
        assert!(stderr.contains("terminal"), "{on:?}: {stderr}");
        assert!(!stderr.contains("not implemented"), "{on:?}");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0, "{on:?}");
    }
}

#[tokio::test]
async fn the_server_names_its_directory() {
    let dir = private_tempdir();
    let real = dir.path().join("a");
    fs::create_dir(&real).unwrap();
    let link = dir.path().join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let mut flag = std::ffi::OsString::from("--config=");
    flag.push(&link);
    let server = ServerBuilder::new().arg(flag).start().await;
    let named = server.client().config_dir().await.unwrap();
    assert_eq!(named, Some(text(&fs::canonicalize(&real).unwrap())));
}

#[tokio::test]
async fn a_server_with_no_directory_names_none() {
    let server = ServerBuilder::new()
        .env("MU_CONFIG", None::<&str>)
        .env("XDG_CONFIG_HOME", None::<&str>)
        .env("HOME", None::<&str>)
        .start()
        .await;
    assert_eq!(server.client().config_dir().await.unwrap(), None);
}
