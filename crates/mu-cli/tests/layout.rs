//! The verb `mu layout` and the layout calls of the server, run on the built binary.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::process::Output;

use support::process::{ServerBuilder, ServerProcess, server_url};
use support::{ConfigDir, mu_async, stderr, stdout};

mod support;

/// Starts a server over `dir` and waits until it serves.
async fn start(dir: &Path) -> ServerProcess {
    ServerBuilder::new().arg("--config").arg(dir).start().await
}

/// Runs `mu layout` with `args` on `server`.
async fn layout(server: &ServerProcess, args: &[&str]) -> Output {
    let url = server_url(server.socket());
    let mut all = vec!["layout"];
    all.extend(args);
    all.extend(["--server", &url]);
    mu_async(&all, &[]).await
}

/// The content of a layout file with `lines`.
fn layout_file<S: AsRef<str>>(lines: &[S]) -> String {
    let quoted: Vec<String> = lines
        .iter()
        .map(|line| format!("{:?}", line.as_ref()))
        .collect();
    format!("place = [{}]\n", quoted.join(", "))
}

/// A configuration directory with the layout file `mine` of `lines`.
fn mine<S: AsRef<str>>(lines: &[S]) -> ConfigDir {
    ConfigDir::empty().file("clients/layouts/mine.toml", &layout_file(lines))
}

/// Starts a server over a directory with the bad layout file `mine` of `lines`, runs
/// `mu layout show mine`, and asserts one line of stderr that holds each of `texts`, and the
/// exit status 1.
async fn bad_layout<S: AsRef<str>>(lines: &[S], texts: &[&str]) {
    let dir = mine(lines);
    let server = start(dir.path()).await;
    let output = layout(&server, &["show", "mine"]).await;
    let stderr = stderr(&output);
    let matching: Vec<&str> = stderr
        .lines()
        .filter(|line| texts.iter().all(|text| line.contains(text)))
        .collect();
    assert!(
        matching.len() == 1 && output.status.code() == Some(1) && stdout(&output).is_empty(),
        "{stderr}"
    );
}

const FILE: &str = "clients/layouts/mine.toml";

#[tokio::test]
async fn the_list_with_one_player_layout() {
    let dir = mine(&["output", "editor below output"]);
    let server = start(dir.path()).await;
    let output = layout(&server, &["list"]).await;
    let text = stdout(&output);
    let names: Vec<&str> = text
        .lines()
        .filter(|line| {
            line.chars()
                .next()
                .is_some_and(|first| first.is_ascii_lowercase())
        })
        .collect();
    assert_eq!(
        names,
        [
            "classic (built-in)",
            "focus (built-in)",
            "mine",
            "sidebar (built-in)",
            "status (built-in)"
        ],
        "{text}"
    );
    assert!(stderr(&output).is_empty(), "{}", stderr(&output));
}

#[tokio::test]
async fn the_list_with_a_bad_layout_file() {
    let dir = mine(&["output"]);
    let server = start(dir.path()).await;
    let output = layout(&server, &["list"]).await;
    assert!(!stdout(&output).contains("mine"), "{}", stdout(&output));
    let stderr = stderr(&output);
    let matching: Vec<&str> = stderr
        .lines()
        .filter(|line| line.contains(FILE) && line.contains("editor"))
        .collect();
    assert_eq!(matching.len(), 1, "{stderr}");
}

#[tokio::test]
async fn the_picture_of_classic() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(&server, &["show", "classic"]).await;
    let text = stdout(&output);
    let place = |word: &str| {
        text.lines()
            .position(|line| line.starts_with('│') && line.contains(word))
            .unwrap_or_else(|| panic!("no row of the picture holds {word}: {text}"))
    };
    assert!(
        place("tabs") < place("output") && place("output") < place("editor"),
        "{text}"
    );
}

#[tokio::test]
async fn the_numbers_of_the_lines() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(&server, &["show", "classic"]).await;
    let text = stdout(&output);
    for line in [
        "1. output",
        "2. tabs above output",
        "3. editor below output",
    ] {
        assert!(text.lines().any(|each| each == line), "{text}");
    }
}

#[tokio::test]
async fn a_layout_that_no_file_and_no_built_in_has() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(&server, &["show", "gone"]).await;
    assert!(
        output.status.code() == Some(1) && stderr(&output).contains("gone"),
        "{}",
        stderr(&output)
    );
}

#[tokio::test]
async fn an_open_socket_directory() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let socket_dir = server.socket().parent().unwrap().to_owned();
    fs::set_permissions(&socket_dir, fs::Permissions::from_mode(0o755)).unwrap();
    let output = layout(&server, &["list"]).await;
    fs::set_permissions(&socket_dir, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        output.status.code() == Some(1)
            && stdout(&output).is_empty()
            && stderr(&output).contains("0755"),
        "{}",
        stderr(&output)
    );
}

#[tokio::test]
async fn no_server() {
    let dir = support::process::private_tempdir();
    let socket = dir.path().join("s.sock");
    let url = server_url(&socket);
    let output = mu_async(&["layout", "list", "--server", &url], &[]).await;
    assert!(
        output.status.code() == Some(1) && !socket.exists(),
        "{}",
        stderr(&output)
    );
}

// The scenarios of a bad sentence.

#[tokio::test]
async fn an_unknown_block() {
    bad_layout(
        &["editor", "spawns above editor"],
        &[FILE, "line 2", "spawns"],
    )
    .await;
}

#[tokio::test]
async fn a_block_twice() {
    bad_layout(
        &["editor", "output above editor", "output at top"],
        &["line 3", "output"],
    )
    .await;
}

#[tokio::test]
async fn a_block_that_is_not_placed_yet() {
    bad_layout(&["editor", "tabs above output"], &["line 2", "output"]).await;
}

#[tokio::test]
async fn no_editor() {
    bad_layout(&["output", "tabs above output"], &[FILE, "editor"]).await;
}

#[tokio::test]
async fn a_percent_out_of_range() {
    bad_layout(&["editor", "output above editor, 95%"], &["line 2", "95%"]).await;
}

#[tokio::test]
async fn too_many_lines() {
    bad_layout(&vec!["editor"; 33], &[FILE, "32"]).await;
}

#[tokio::test]
async fn anchors_that_are_not_neighbors() {
    bad_layout(
        &[
            "output",
            "tabs above output",
            "editor below output",
            "output as notices right of tabs and editor",
        ],
        &["line 4", "tabs", "editor"],
    )
    .await;
}

#[tokio::test]
async fn an_id_twice() {
    bad_layout(
        &[
            "editor",
            "output as main above editor",
            "output as main at right",
        ],
        &["line 3", "main"],
    )
    .await;
}

#[tokio::test]
async fn an_id_that_is_a_word_of_the_sentences() {
    bad_layout(
        &["editor", "output as above above editor"],
        &["line 2", "above"],
    )
    .await;
}

#[tokio::test]
async fn the_editor_does_not_stack() {
    bad_layout(
        &["output", "editor with output"],
        &[FILE, "line 2", "editor"],
    )
    .await;
}

#[tokio::test]
async fn a_size_on_the_tabs() {
    bad_layout(
        &["editor", "tabs above editor, 3"],
        &[FILE, "line 2", "tabs"],
    )
    .await;
}

#[tokio::test]
async fn a_size_on_the_first_line() {
    bad_layout(&["output, 50%", "editor below output"], &["line 1"]).await;
}

#[tokio::test]
async fn nine_outputs() {
    let mut lines = vec!["editor".to_owned()];
    lines.extend((0..9).map(|number| format!("output as o{number} above editor")));
    bad_layout(&lines, &[FILE, "output"]).await;
}

#[tokio::test]
async fn a_second_editor_with_no_id() {
    bad_layout(&["editor", "editor above editor"], &["line 2", "editor"]).await;
}

#[tokio::test]
async fn five_editors() {
    let mut lines = vec!["editor".to_owned()];
    lines.extend((1..5).map(|number| format!("editor as e{number} above editor")));
    bad_layout(&lines, &[FILE, "editor"]).await;
}
