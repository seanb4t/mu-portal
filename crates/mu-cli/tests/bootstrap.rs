//! The scenarios of the client-bootstrap capability that `mu config show` runs, on the built
//! binary.

use std::path::Path;
use std::process::Output;

use support::process::{ServerBuilder, ServerProcess, private_tempdir};
use support::{ConfigDir, mu_async, stderr, stdout};
use tokio::net::UnixListener;

mod support;

/// Runs `mu config show` with `args` after `show`, and `env`.
async fn show(args: &[&str], env: &[(&str, Option<&str>)]) -> Output {
    let mut all = vec!["config", "show"];
    all.extend_from_slice(args);
    mu_async(&all, env).await
}

/// The URL of `socket`.
fn url(socket: &Path) -> String {
    format!("unix://{}", socket.display())
}

/// A server over `dir` on its own socket.
async fn serve(dir: &ConfigDir) -> ServerProcess {
    ServerBuilder::new()
        .arg("--config")
        .arg(dir.path())
        .start()
        .await
}

/// Asserts the exit status 1 and a stderr that holds each of `texts`.
fn refused(output: &Output, texts: &[&str]) {
    let stderr = stderr(output);
    assert!(
        output.status.code() == Some(1) && texts.iter().all(|text| stderr.contains(text)),
        "{stderr}"
    );
}

/// Asserts that `mu config show --server <url> --device laptop` is refused with `texts`.
async fn refused_url(url: &str, texts: &[&str]) {
    refused(
        &show(&["--server", url, "--device", "laptop"], &[]).await,
        texts,
    );
}

/// Asserts the exit status 0.
fn succeeds(output: &Output) {
    assert_eq!(output.status.code(), Some(0), "{}", stderr(output));
}

/// The host name of the test machine in lowercase.
fn host() -> String {
    rustix::system::uname()
        .nodename()
        .to_string_lossy()
        .to_lowercase()
}

/// Asserts that stdout holds the echo `hide` of the device file of `device`.
fn hides_for(output: &Output, device: &str) {
    let line = format!("echo = \"hide\" # clients/devices/{device}.toml");
    assert!(
        stdout(output).lines().any(|each| each == line),
        "{}{}",
        stdout(output),
        stderr(output)
    );
}

#[tokio::test]
async fn the_server_flag_wins() {
    let server = ServerBuilder::new().start().await;
    let other = server.dir().path().join("env.sock");
    let output = show(
        &["--server", &url(server.socket()), "--device", "laptop"],
        &[("MU_SERVER", Some(&url(&other)))],
    )
    .await;
    succeeds(&output);
}

#[tokio::test]
async fn the_server_variable() {
    let server = ServerBuilder::new().start().await;
    let output = show(
        &["--device", "laptop"],
        &[("MU_SERVER", Some(&url(server.socket())))],
    )
    .await;
    succeeds(&output);
}

/// A server on the default socket of the runtime directory `run`.
async fn default_server(run: &Path) -> ServerProcess {
    ServerBuilder::new()
        .socket_flag(None)
        .env("XDG_RUNTIME_DIR", Some(run))
        .start()
        .await
}

#[tokio::test]
async fn an_empty_server_variable() {
    let run = private_tempdir();
    let _server = default_server(run.path()).await;
    let run = run.path().to_str().unwrap();
    let output = show(
        &["--device", "laptop"],
        &[("XDG_RUNTIME_DIR", Some(run)), ("MU_SERVER", Some(""))],
    )
    .await;
    succeeds(&output);
}

#[tokio::test]
async fn the_default() {
    let run = private_tempdir();
    let _server = default_server(run.path()).await;
    let run = run.path().to_str().unwrap();
    let output = show(&["--device", "laptop"], &[("XDG_RUNTIME_DIR", Some(run))]).await;
    succeeds(&output);
}

#[tokio::test]
async fn mu_socket_does_not_count() {
    let server = ServerBuilder::new().start().await;
    let run = server.dir().path().join("run");
    let output = show(
        &["--device", "laptop"],
        &[
            ("XDG_RUNTIME_DIR", Some(run.to_str().unwrap())),
            ("MU_SOCKET", Some(server.socket().to_str().unwrap())),
        ],
    )
    .await;
    let default = run.join("mu/server.sock");
    refused(&output, &[default.to_str().unwrap()]);
}

#[tokio::test]
async fn a_path_with_a_space() {
    let dir = private_tempdir();
    let socket = dir.path().join("a b/s.sock");
    let server = ServerBuilder::new()
        .socket_flag(Some(socket.clone()))
        .start()
        .await;
    let encoded = url(server.socket()).replace(' ', "%20");
    succeeds(&show(&["--server", &encoded, "--device", "laptop"], &[]).await);
}

#[tokio::test]
async fn another_scheme() {
    refused_url("https://example.com", &["https", "unix"]).await;
}

#[tokio::test]
async fn a_host() {
    refused_url("unix://host/tmp/s.sock", &["unix://host/tmp/s.sock"]).await;
}

#[tokio::test]
async fn a_user() {
    refused_url("unix://me@/tmp/s.sock", &["unix://me@/tmp/s.sock"]).await;
}

#[tokio::test]
async fn a_password() {
    refused_url("unix://me:pw@/tmp/s.sock", &["unix://me:pw@/tmp/s.sock"]).await;
}

#[tokio::test]
async fn a_fragment() {
    let server = ServerBuilder::new().start().await;
    refused_url(&format!("{}#x", url(server.socket())), &["#x"]).await;
}

#[tokio::test]
async fn no_double_slash() {
    refused_url("unix:/tmp/s.sock", &["unix:/tmp/s.sock"]).await;
}

#[tokio::test]
async fn a_relative_path() {
    refused_url("unix:s.sock", &["unix:s.sock"]).await;
}

/// Asserts that the URL of a socket in a new directory with `query` is refused with `texts`.
async fn refused_query(query: &str, texts: &[&str]) {
    let dir = private_tempdir();
    refused_url(
        &format!("{}?{query}", url(&dir.path().join("s.sock"))),
        texts,
    )
    .await;
}

#[tokio::test]
async fn a_wait_of_zero() {
    refused_query("connect_wait=0", &["connect_wait"]).await;
}

#[tokio::test]
async fn a_negative_wait() {
    refused_query("connect_wait=-1", &["connect_wait"]).await;
}

#[tokio::test]
async fn a_wait_that_is_not_a_number() {
    refused_query("connect_wait=nan", &["connect_wait"]).await;
}

#[tokio::test]
async fn an_infinite_wait() {
    refused_query("connect_wait=inf", &["connect_wait"]).await;
}

#[tokio::test]
async fn a_wait_under_one_nanosecond() {
    refused_query("connect_wait=1e-10", &["connect_wait"]).await;
}

#[tokio::test]
async fn an_unknown_option() {
    refused_query("colour=red", &["colour"]).await;
}

#[tokio::test]
async fn an_option_twice() {
    refused_query("connect_wait=1&connect_wait=2", &["connect_wait"]).await;
}

#[tokio::test]
async fn a_short_wait() {
    let dir = private_tempdir();
    let socket = dir.path().join("s.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let held = tokio::spawn(async move {
        let mut open = Vec::new();
        while let Ok((stream, _)) = listener.accept().await {
            open.push(stream);
        }
    });
    let output = show(
        &[
            "--server",
            &format!("{}?connect_wait=0.2", url(&socket)),
            "--device",
            "laptop",
        ],
        &[],
    )
    .await;
    held.abort();
    refused(&output, &[socket.to_str().unwrap(), "0.2"]);
}

#[tokio::test]
async fn the_device_variable() {
    let dir = ConfigDir::empty().file("clients/devices/laptop.toml", "echo = \"hide\"\n");
    let server = serve(&dir).await;
    let output = show(
        &["--server", &url(server.socket())],
        &[("MU_DEVICE", Some("laptop"))],
    )
    .await;
    hides_for(&output, "laptop");
}

#[tokio::test]
async fn the_device_flag_wins() {
    let dir = ConfigDir::empty().file("clients/devices/laptop.toml", "echo = \"hide\"\n");
    let server = serve(&dir).await;
    let output = show(
        &["--server", &url(server.socket()), "--device", "laptop"],
        &[("MU_DEVICE", Some("desk"))],
    )
    .await;
    hides_for(&output, "laptop");
}

#[tokio::test]
async fn an_empty_device_variable() {
    let host = host();
    let dir = ConfigDir::empty().file(&format!("clients/devices/{host}.toml"), "echo = \"hide\"\n");
    let server = serve(&dir).await;
    let output = show(
        &["--server", &url(server.socket())],
        &[("MU_DEVICE", Some(""))],
    )
    .await;
    hides_for(&output, &host);
}

#[tokio::test]
async fn the_host_name() {
    let host = host();
    let dir = ConfigDir::empty().file(&format!("clients/devices/{host}.toml"), "echo = \"hide\"\n");
    let server = serve(&dir).await;
    let output = show(&["--server", &url(server.socket())], &[]).await;
    hides_for(&output, &host);
}

#[tokio::test]
async fn a_bad_name() {
    let dir = private_tempdir();
    let output = show(
        &[
            "--server",
            &url(&dir.path().join("s.sock")),
            "--device",
            "my laptop",
        ],
        &[],
    )
    .await;
    refused(&output, &["my laptop"]);
}

#[tokio::test]
async fn a_name_of_64_characters() {
    let server = serve(&ConfigDir::empty()).await;
    let name = "a".repeat(64);
    succeeds(&show(&["--server", &url(server.socket()), "--device", &name], &[]).await);
}

#[tokio::test]
async fn a_name_of_65_characters() {
    let dir = private_tempdir();
    let name = "a".repeat(65);
    let output = show(
        &[
            "--server",
            &url(&dir.path().join("s.sock")),
            "--device",
            &name,
        ],
        &[],
    )
    .await;
    refused(&output, &[&name]);
}

#[tokio::test]
async fn an_empty_device_flag() {
    let dir = private_tempdir();
    let output = show(
        &["--server", &url(&dir.path().join("s.sock")), "--device", ""],
        &[],
    )
    .await;
    refused(&output, &["device"]);
}
