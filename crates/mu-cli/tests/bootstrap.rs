//! The scenarios of the client-bootstrap capability that `mu config show` runs, on the built
//! binary.

use std::path::Path;
use std::process::Output;

use support::process::{ServerBuilder, ServerProcess, private_tempdir, server_url};
use support::{ConfigDir, fails_naming, mu_async, stderr, stdout};
use tokio::net::UnixListener;

mod support;

/// Runs `mu config show` with `args` after `show`, and `env`.
async fn show(args: &[&str], env: &[(&str, Option<&str>)]) -> Output {
    let mut all = vec!["config", "show"];
    all.extend_from_slice(args);
    mu_async(&all, env).await
}

/// A server over `dir` on its own socket.
async fn serve(dir: &ConfigDir) -> ServerProcess {
    ServerBuilder::new()
        .arg("--config")
        .arg(dir.path())
        .start()
        .await
}

/// Asserts that `mu config show --server <url> --device laptop` is refused with the text that
/// names `url` and `reason`.
async fn refused_url(url: &str, reason: &str) {
    fails_naming(
        &show(&["--server", url, "--device", "laptop"], &[]).await,
        &[&format!("{url}: {reason}")],
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

/// Asserts that the command took the host name `host` as its device: the echo of its device
/// file, or, for a host name that is not a device name, a refusal that says to set `--device`.
fn uses_the_host(output: &Output, host: &str) {
    if mu_config::DeviceName::new(host).is_ok() {
        hides_for(output, host);
    } else {
        fails_naming(output, &[host, "set --device"]);
    }
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
        &[
            "--server",
            &server_url(server.socket()),
            "--device",
            "laptop",
        ],
        &[("MU_SERVER", Some(&server_url(&other)))],
    )
    .await;
    succeeds(&output);
}

#[tokio::test]
async fn the_server_variable() {
    let server = ServerBuilder::new().start().await;
    let output = show(
        &["--device", "laptop"],
        &[("MU_SERVER", Some(&server_url(server.socket())))],
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
    fails_naming(&output, &[default.to_str().unwrap()]);
}

#[tokio::test]
async fn a_path_with_a_space() {
    let dir = private_tempdir();
    let socket = dir.path().join("a b/s.sock");
    let server = ServerBuilder::new()
        .socket_flag(Some(socket.clone()))
        .start()
        .await;
    succeeds(
        &show(
            &[
                "--server",
                &server_url(server.socket()),
                "--device",
                "laptop",
            ],
            &[],
        )
        .await,
    );
}

#[tokio::test]
async fn another_scheme() {
    refused_url(
        "https://example.com",
        "the scheme is https: a server URL is unix:// and the path of a socket",
    )
    .await;
}

#[tokio::test]
async fn a_host() {
    refused_url("unix://host/tmp/s.sock", "a server URL has no host").await;
}

#[tokio::test]
async fn a_user() {
    refused_url("unix://me@/tmp/s.sock", "empty host").await;
}

#[tokio::test]
async fn a_password() {
    refused_url("unix://me:pw@/tmp/s.sock", "empty host").await;
}

#[tokio::test]
async fn a_fragment() {
    let server = ServerBuilder::new().start().await;
    refused_url(
        &format!("{}#x", server_url(server.socket())),
        "a server URL has no fragment",
    )
    .await;
}

#[tokio::test]
async fn no_double_slash() {
    refused_url("unix:/tmp/s.sock", "write unix:// and an absolute path").await;
}

#[tokio::test]
async fn a_relative_path() {
    refused_url("unix:s.sock", "write unix:// and an absolute path").await;
}

/// Asserts that the URL of a socket in a new directory with `query` is refused with `reason`.
async fn refused_query(query: &str, reason: &str) {
    let dir = private_tempdir();
    refused_url(
        &format!("{}?{query}", server_url(&dir.path().join("s.sock"))),
        reason,
    )
    .await;
}

/// The reason that refuses the wait `shown`.
fn bad_wait(shown: &str) -> String {
    format!("connect_wait: {shown} is not a finite number of seconds of 1 ns or more")
}

#[tokio::test]
async fn a_wait_of_zero() {
    refused_query("connect_wait=0", &bad_wait("0")).await;
}

#[tokio::test]
async fn a_negative_wait() {
    refused_query("connect_wait=-1", &bad_wait("-1")).await;
}

#[tokio::test]
async fn a_wait_that_is_not_a_number() {
    refused_query("connect_wait=nan", &bad_wait("NaN")).await;
}

#[tokio::test]
async fn an_infinite_wait() {
    refused_query("connect_wait=inf", &bad_wait("inf")).await;
}

#[tokio::test]
async fn a_wait_under_one_nanosecond() {
    refused_query("connect_wait=1e-10", &bad_wait("0.0000000001")).await;
}

#[tokio::test]
async fn an_unknown_option() {
    refused_query(
        "colour=red",
        "colour is no option: the only option is connect_wait",
    )
    .await;
}

#[tokio::test]
async fn an_option_twice() {
    refused_query(
        "connect_wait=1&connect_wait=2",
        "connect_wait is there twice",
    )
    .await;
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
            &format!("{}?connect_wait=0.2", server_url(&socket)),
            "--device",
            "laptop",
        ],
        &[],
    )
    .await;
    held.abort();
    fails_naming(&output, &[socket.to_str().unwrap(), "0.2"]);
}

#[tokio::test]
async fn the_device_variable() {
    let dir = ConfigDir::empty().file("clients/devices/laptop.toml", "echo = \"hide\"\n");
    let server = serve(&dir).await;
    let output = show(
        &["--server", &server_url(server.socket())],
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
        &[
            "--server",
            &server_url(server.socket()),
            "--device",
            "laptop",
        ],
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
        &["--server", &server_url(server.socket())],
        &[("MU_DEVICE", Some(""))],
    )
    .await;
    uses_the_host(&output, &host);
}

#[tokio::test]
async fn the_host_name() {
    let host = host();
    let dir = ConfigDir::empty().file(&format!("clients/devices/{host}.toml"), "echo = \"hide\"\n");
    let server = serve(&dir).await;
    let output = show(&["--server", &server_url(server.socket())], &[]).await;
    uses_the_host(&output, &host);
}

#[tokio::test]
async fn a_bad_name() {
    let dir = private_tempdir();
    let output = show(
        &[
            "--server",
            &server_url(&dir.path().join("s.sock")),
            "--device",
            "my laptop",
        ],
        &[],
    )
    .await;
    fails_naming(&output, &["my laptop"]);
}

#[tokio::test]
async fn a_name_of_64_characters() {
    let server = serve(&ConfigDir::empty()).await;
    let name = "a".repeat(64);
    succeeds(
        &show(
            &["--server", &server_url(server.socket()), "--device", &name],
            &[],
        )
        .await,
    );
}

#[tokio::test]
async fn a_name_of_65_characters() {
    let dir = private_tempdir();
    let name = "a".repeat(65);
    let output = show(
        &[
            "--server",
            &server_url(&dir.path().join("s.sock")),
            "--device",
            &name,
        ],
        &[],
    )
    .await;
    fails_naming(&output, &[&name]);
}

#[tokio::test]
async fn an_empty_device_flag() {
    let dir = private_tempdir();
    let output = show(
        &[
            "--server",
            &server_url(&dir.path().join("s.sock")),
            "--device",
            "",
        ],
        &[],
    )
    .await;
    fails_naming(&output, &["device"]);
}
