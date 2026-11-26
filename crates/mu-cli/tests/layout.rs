//! The verb `mu layout` and the layout calls of the server, run on the built binary.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::process::Output;

use support::process::{ServerBuilder, ServerProcess, server_url};
use support::{ConfigDir, layout_file, mu_async, place_of, stderr, stdout};

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
    let dir = mine(&["output", "editor below main"]);
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
        place("tabs") < place("main") && place("main") < place("editor"),
        "{text}"
    );
}

#[tokio::test]
async fn the_numbers_of_the_lines() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(&server, &["show", "classic"]).await;
    let text = stdout(&output);
    for line in ["1. output", "2. tabs above main", "3. editor below main"] {
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
        &["line 3", "main", "`as`"],
    )
    .await;
}

#[tokio::test]
async fn a_block_that_is_not_placed_yet() {
    bad_layout(&["editor", "tabs above main"], &["line 2", "main"]).await;
}

#[tokio::test]
async fn no_editor() {
    bad_layout(&["output", "tabs above main"], &[FILE, "editor"]).await;
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
            "tabs above main",
            "editor below main",
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
    bad_layout(&["output", "editor with main"], &[FILE, "line 2", "editor"]).await;
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
    bad_layout(&["output, 50%", "editor below main"], &["line 1"]).await;
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

// The streams of the outputs.

/// Runs `mu layout show mine` over the layout file of `lines`, and asserts that stdout holds
/// line `number` of the plan as `wanted`.
async fn shown_line<S: AsRef<str>>(lines: &[S], number: usize, wanted: &str) {
    let dir = mine(lines);
    let server = start(dir.path()).await;
    let output = layout(&server, &["show", "mine"]).await;
    let text = stdout(&output);
    let line = format!("{number}. {wanted}");
    assert!(
        text.lines().any(|each| each == line),
        "{text}\n{}",
        stderr(&output)
    );
}

#[tokio::test]
async fn the_editor_alone() {
    bad_layout(&["editor"], &[FILE, "main"]).await;
}

#[tokio::test]
async fn no_main_output() {
    bad_layout(&["editor", "output as side above editor"], &[FILE, "main"]).await;
}

#[tokio::test]
async fn no_output_for_the_notices() {
    bad_layout(
        &["output as main showing main, chan-*", "editor below main"],
        &[FILE, "notices"],
    )
    .await;
}

#[tokio::test]
async fn only_exclusions() {
    bad_layout(
        &[
            "output",
            "editor below main",
            "output as side showing !chan-* right of main",
        ],
        &["line 3", "showing"],
    )
    .await;
}

#[tokio::test]
async fn an_empty_list_of_patterns() {
    bad_layout(
        &[
            "output",
            "editor below main",
            "output as side showing right of main",
        ],
        &["line 3", "showing"],
    )
    .await;
}

#[tokio::test]
async fn a_reserved_pattern() {
    bad_layout(
        &[
            "output",
            "editor below main",
            "output as side showing re:chan right of main",
        ],
        &["line 3", "re:chan"],
    )
    .await;
}

#[tokio::test]
async fn a_plan_keeps_the_patterns() {
    let line = "output as comms showing chan-*, pages right of main, 30%";
    shown_line(&["output", "editor below main", line], 3, line).await;
}

#[tokio::test]
async fn an_id_in_upper_case() {
    shown_line(
        &[
            "output",
            "editor below main",
            "output as Comms showing Chan-*, Pages right of main",
        ],
        3,
        "output as comms showing chan-*, pages right of main",
    )
    .await;
}

#[tokio::test]
async fn patterns_of_all_worlds() {
    let line = "output as comms of all worlds showing chan-*, pages right of main, 30%";
    shown_line(&["output", "editor below main", line], 3, line).await;
}

#[tokio::test]
async fn the_main_output_of_all_worlds() {
    bad_layout(
        &["output as main of all worlds", "editor below main"],
        &["line 1", "main"],
    )
    .await;
}

#[tokio::test]
async fn the_id_all() {
    bad_layout(
        &["output", "editor below main", "output as all right of main"],
        &["line 3", "all"],
    )
    .await;
}

#[tokio::test]
async fn the_scope_after_the_place() {
    bad_layout(
        &[
            "output",
            "editor below main",
            "output as chat right of main of all worlds",
        ],
        &["line 3"],
    )
    .await;
}

// The write calls.

use mu_proto::__buffa::oneof::edit_layout_request::Change as EditChange;
use support::process::{Answers, PtyClient, Session};

fn lines(lines: &[&str]) -> Vec<String> {
    lines.iter().map(ToString::to_string).collect()
}

/// Runs `mu config show --device laptop` on `server`.
async fn show_config(server: &ServerProcess) -> String {
    let url = server_url(server.socket());
    let output = mu_async(
        &["config", "show", "--server", &url, "--device", "laptop"],
        &[],
    )
    .await;
    stdout(&output)
}

fn device_file(dir: &Path) -> std::path::PathBuf {
    dir.join("clients/devices/laptop.toml")
}

#[tokio::test]
async fn a_save_writes_the_file() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    server
        .client()
        .save_layout("mine", lines(&["output", "editor below main"]), None, None)
        .await
        .expect("the save");
    assert_eq!(
        place_of(dir.path(), "mine"),
        ["output", "editor below main"]
    );
    let listed = layout(&server, &["list"]).await;
    assert!(
        stdout(&listed).lines().any(|line| line == "mine"),
        "{}",
        stdout(&listed)
    );
}

#[tokio::test]
async fn a_save_of_a_bad_layout() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let error = server
        .client()
        .save_layout("mine", lines(&["output"]), None, None)
        .await
        .expect_err("no editor")
        .to_string();
    assert!(error.contains("editor"), "{error}");
    assert!(!dir.path().join("clients/layouts/mine.toml").exists());
}

#[tokio::test]
async fn a_save_with_a_built_in_name() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let error = server
        .client()
        .save_layout("classic", lines(&["editor"]), None, None)
        .await
        .expect_err("a built-in name")
        .to_string();
    assert!(error.contains("classic"), "{error}");
}

#[tokio::test]
async fn a_save_with_a_bad_name() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let result = server
        .client()
        .save_layout("../x", lines(&["editor"]), None, None)
        .await;
    assert!(result.is_err());
    assert!(!dir.path().join("clients").exists());
    assert!(!dir.path().join("x.toml").exists());
}

#[tokio::test]
async fn a_save_with_the_name_of_two_dots() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let error = server
        .client()
        .save_layout("..", lines(&["output", "editor below main"]), None, None)
        .await
        .expect_err("two dots")
        .to_string();
    assert!(error.contains(".."), "{error}");
    assert!(!dir.path().join("clients").exists());
}

#[tokio::test]
async fn a_clients_path_that_is_a_file() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let clients = dir.path().join("clients");
    fs::write(&clients, "").unwrap();
    let error = server
        .client()
        .save_layout("mine", lines(&["output", "editor below main"]), None, None)
        .await
        .expect_err("clients is a file")
        .to_string();
    assert!(error.contains("clients"), "{error}");
    assert_eq!(fs::read(&clients).unwrap(), b"");
}

#[tokio::test]
async fn a_device_file_with_a_comment() {
    let dir = ConfigDir::empty().file(
        "clients/devices/laptop.toml",
        "# my laptop\necho = \"show\"\n",
    );
    let server = start(dir.path()).await;
    server
        .client()
        .use_layout("laptop", "status", None)
        .await
        .expect("the use");
    let text = fs::read_to_string(device_file(dir.path())).unwrap();
    for line in ["# my laptop", "echo = \"show\"", "layout = \"status\""] {
        assert!(text.lines().any(|each| each == line), "{text}");
    }
}

#[tokio::test]
async fn no_device_file() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(&server, &["use", "focus", "--device", "laptop"]).await;
    assert!(output.status.success(), "{}", stderr(&output));
    let text = fs::read_to_string(device_file(dir.path())).unwrap();
    assert_eq!(text.trim(), "layout = \"focus\"");
}

#[tokio::test]
async fn an_unknown_layout() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(&server, &["use", "gone", "--device", "laptop"]).await;
    assert!(
        output.status.code() == Some(1) && stderr(&output).contains("gone"),
        "{}",
        stderr(&output)
    );
    assert!(!device_file(dir.path()).exists());
}

#[tokio::test]
async fn a_bad_device_layer() {
    let dir = ConfigDir::empty().file("clients/devices/laptop.toml", "echo = ");
    let server = start(dir.path()).await;
    let error = server
        .client()
        .use_layout("laptop", "status", None)
        .await
        .expect_err("a bad layer")
        .to_string();
    assert!(error.contains("clients/devices/laptop.toml"), "{error}");
    assert_eq!(
        fs::read_to_string(device_file(dir.path())).unwrap(),
        "echo = "
    );
}

#[tokio::test]
async fn a_use_from_the_command_line() {
    let config = ConfigDir::empty();
    let mut session = Session::start(
        ServerBuilder::new()
            .fake_worlds(&["alpha/aria"])
            .arg("--config")
            .arg(config.path()),
    )
    .await;
    session.welcome("alpha/aria").await;
    let client = PtyClient::start_args(
        session.server.socket(),
        session.server.dir().path(),
        Answers::Default,
        &["--device".as_ref(), "laptop".as_ref()],
        &[],
    );
    client.wait_greeting().await;
    let output = layout(&session.server, &["use", "status", "--device", "laptop"]).await;
    assert!(output.status.success(), "{}", stderr(&output));
    client
        .wait_rows("the statusline", |rows| {
            rows.get(23)
                .is_some_and(|row| row.starts_with("alpha/aria | "))
        })
        .await;
}

#[tokio::test]
async fn a_use_for_one_world() {
    let dir = ConfigDir::empty().file(
        "clients/devices/laptop.toml",
        "# my laptop\nlayout = \"status\"\n",
    );
    let server = start(dir.path()).await;
    let output = layout(
        &server,
        &[
            "use",
            "focus",
            "--world",
            "alpha/aria",
            "--device",
            "laptop",
        ],
    )
    .await;
    assert!(output.status.success(), "{}", stderr(&output));
    let text = fs::read_to_string(device_file(dir.path())).unwrap();
    for line in [
        "# my laptop",
        "layout = \"status\"",
        "[world-layouts]",
        "\"alpha/aria\" = \"focus\"",
    ] {
        assert!(text.lines().any(|each| each == line), "{text}");
    }
}

#[tokio::test]
async fn a_use_for_a_game() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(
        &server,
        &["use", "focus", "--game", "alpha", "--device", "laptop"],
    )
    .await;
    assert!(output.status.success(), "{}", stderr(&output));
    let shown = show_config(&server).await;
    for line in [
        "world-layouts.alpha = \"focus\" # clients/devices/laptop.toml",
        "layout = \"classic\" # default",
    ] {
        assert!(shown.lines().any(|each| each == line), "{shown}");
    }
}

#[tokio::test]
async fn forget_the_layout_of_a_world() {
    let dir = ConfigDir::empty().file(
        "clients/devices/laptop.toml",
        "[world-layouts]\n\"alpha/aria\" = \"focus\"\nalpha = \"status\"\n",
    );
    let server = start(dir.path()).await;
    let output = layout(
        &server,
        &["forget", "--world", "alpha/aria", "--device", "laptop"],
    )
    .await;
    assert!(output.status.success(), "{}", stderr(&output));
    let text = fs::read_to_string(device_file(dir.path())).unwrap();
    assert!(
        text.lines().any(|line| line == "alpha = \"status\"") && !text.contains("alpha/aria"),
        "{text}"
    );
}

#[tokio::test]
async fn a_use_with_a_bad_world_id() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(
        &server,
        &["use", "focus", "--world", "a/b/c", "--device", "laptop"],
    )
    .await;
    assert!(
        output.status.code() == Some(1) && stderr(&output).contains("a/b/c"),
        "{}",
        stderr(&output)
    );
    assert!(!device_file(dir.path()).exists());
}

#[tokio::test]
async fn a_use_with_an_empty_name_and_no_key() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let result = server.client().use_layout("laptop", "", None).await;
    assert!(result.is_err());
    assert!(!device_file(dir.path()).exists());
}

// The edits and the description.

const TWO_LINES: &[&str] = &["output", "editor below main"];

#[tokio::test]
async fn an_edit_places_a_block() {
    let dir = mine(TWO_LINES);
    let server = start(dir.path()).await;
    let output = layout(
        &server,
        &["place", "mine", "output as notices right of main, 30%"],
    )
    .await;
    assert!(output.status.success(), "{}", stderr(&output));
    let shown = layout(&server, &["show", "mine"]).await;
    let text = stdout(&shown);
    assert!(
        text.lines()
            .any(|line| line == "3. output as notices right of main, 30%"),
        "{text}"
    );
    assert_eq!(place_of(dir.path(), "mine").len(), 3);
}

#[tokio::test]
async fn an_edit_of_a_built_in_layout() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(&server, &["hide", "classic", "tabs", "--device", "laptop"]).await;
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(place_of(dir.path(), "laptop"), TWO_LINES);
    let shown = show_config(&server).await;
    assert!(
        shown
            .lines()
            .any(|line| line == "layout = \"laptop\" # clients/devices/laptop.toml"),
        "{shown}"
    );
}

#[tokio::test]
async fn an_edit_of_the_built_in_layout_of_one_world() {
    let dir = ConfigDir::empty().file(
        "clients/global.toml",
        "[world-layouts]\n\"alpha/aria\" = \"focus\"\n",
    );
    let server = start(dir.path()).await;
    let output = layout(
        &server,
        &[
            "place",
            "focus",
            "tabs at top",
            "--device",
            "laptop",
            "--world",
            "alpha/aria",
        ],
    )
    .await;
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(place_of(dir.path(), "laptop.alpha.aria").len(), 3);
    let shown = show_config(&server).await;
    for line in [
        "world-layouts.\"alpha/aria\" = \"laptop.alpha.aria\" # clients/devices/laptop.toml",
        "layout = \"classic\" # default",
    ] {
        assert!(shown.lines().any(|each| each == line), "{shown}");
    }
}

#[tokio::test]
async fn a_bad_edit() {
    let dir = mine(TWO_LINES);
    let server = start(dir.path()).await;
    let output = layout(&server, &["hide", "mine", "editor"]).await;
    assert!(
        output.status.code() == Some(1) && stderr(&output).contains("editor"),
        "{}",
        stderr(&output)
    );
    assert_eq!(place_of(dir.path(), "mine"), TWO_LINES);
}

#[tokio::test]
async fn an_insert_from_the_command_line() {
    let dir = mine(TWO_LINES);
    let server = start(dir.path()).await;
    let output = layout(
        &server,
        &["insert", "mine", "2", "output as notices right of main"],
    )
    .await;
    assert!(output.status.success(), "{}", stderr(&output));
    let shown = layout(&server, &["show", "mine"]).await;
    let text = stdout(&shown);
    for line in ["2. output as notices right of main", "3. editor below main"] {
        assert!(text.lines().any(|each| each == line), "{text}");
    }
}

#[tokio::test]
async fn a_delete_by_id() {
    let dir = mine(&[
        "output",
        "editor below main",
        "output as side right of main",
    ]);
    let server = start(dir.path()).await;
    let output = layout(&server, &["delete", "mine", "side"]).await;
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(place_of(dir.path(), "mine"), TWO_LINES);
}

#[tokio::test]
async fn an_edit_with_a_number_past_the_plan() {
    let dir = mine(TWO_LINES);
    let server = start(dir.path()).await;
    let request = mu_proto::EditLayoutRequest {
        name: "mine".to_owned(),
        change: Some(EditChange::Delete("3".to_owned())),
        ..Default::default()
    };
    let error = server
        .client()
        .edit_layout(request)
        .await
        .expect_err("line 3")
        .to_string();
    assert!(error.contains('3'), "{error}");
    assert_eq!(place_of(dir.path(), "mine"), TWO_LINES);
}

#[tokio::test]
async fn describe_gives_each_block() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(
        &server,
        &[
            "describe", "sidebar", "--width", "80", "--height", "24", "--json",
        ],
    )
    .await;
    let blocks: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output))
        .unwrap_or_else(|error| panic!("{error}: {}", stdout(&output)));
    let ids: Vec<&str> = blocks
        .iter()
        .map(|block| block["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["main", "tabs", "editor", "statusline", "notices"]);
    let lines: Vec<u64> = blocks
        .iter()
        .map(|block| block["line"].as_u64().unwrap())
        .collect();
    assert_eq!(lines, [1, 2, 3, 4, 5]);
    let of = |id: &str| blocks.iter().find(|block| block["id"] == id).unwrap();
    assert_eq!(of("notices")["width"], 24);
    assert_eq!(
        (of("tabs")["width"].as_u64(), of("tabs")["height"].as_u64()),
        (Some(80), Some(1))
    );
}

#[tokio::test]
async fn describe_of_an_unknown_layout() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(&server, &["describe", "gone"]).await;
    assert!(
        output.status.code() == Some(1) && stderr(&output).contains("gone"),
        "{}",
        stderr(&output)
    );
}

#[tokio::test]
async fn an_edit_gives_the_same_file_as_the_command() {
    use support::process::{BOUND, wait_until};
    let plan = ["output", "tabs above main", "editor below main"];
    let with_mine = || mine(&plan).file("clients/global.toml", "layout = \"mine\"\n");
    // The first server has a client, which runs the command.
    let first = with_mine();
    let mut session = Session::start(
        ServerBuilder::new()
            .fake_worlds(&["alpha/aria"])
            .arg("--config")
            .arg(first.path()),
    )
    .await;
    session.welcome("alpha/aria").await;
    let mut client = PtyClient::start_args(
        session.server.socket(),
        session.server.dir().path(),
        Answers::Default,
        &["--device".as_ref(), "laptop".as_ref()],
        &[],
    );
    client.wait_greeting().await;
    let mut hashes = session
        .server
        .client()
        .watch_config()
        .await
        .expect("the watch of the configuration");
    hashes.next().await.expect("the current store hash");
    client.send(b"/pane hide tabs\r");
    wait_until(BOUND, "the save of the client", async {
        while place_of(first.path(), "mine").len() != 2 {
            hashes.next().await.expect("the next store hash");
        }
    })
    .await;
    // The second server gets the same change from the command line.
    let second = with_mine();
    let server = start(second.path()).await;
    let output = layout(&server, &["hide", "mine", "tabs"]).await;
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        place_of(first.path(), "mine"),
        place_of(second.path(), "mine")
    );
    assert_eq!(
        fs::read_to_string(first.path().join("clients/layouts/mine.toml")).unwrap(),
        fs::read_to_string(second.path().join("clients/layouts/mine.toml")).unwrap()
    );
}

#[tokio::test]
async fn a_size_of_fill_from_the_command_line() {
    let dir = mine(&[
        "output",
        "editor below main",
        "output as notices right of main, 30%",
    ]);
    let server = start(dir.path()).await;
    let output = layout(&server, &["size", "mine", "notices", "40%"]).await;
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        place_of(dir.path(), "mine")[2],
        "output as notices right of main, 40%"
    );
    let output = layout(&server, &["size", "mine", "notices", "fill"]).await;
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        place_of(dir.path(), "mine")[2],
        "output as notices right of main"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn a_save_through_a_linked_directory() {
    // The directory `clients` of the configuration directory is a link to a directory.
    let real = mine(&["output", "editor below main"]);
    let config = tempfile::tempdir().expect("a directory");
    std::os::unix::fs::symlink(real.path().join("clients"), config.path().join("clients"))
        .expect("a link");
    let server = start(config.path()).await;
    let output = layout(&server, &["place", "mine", "tabs above main"]).await;
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        place_of(real.path(), "mine"),
        ["output", "editor below main", "tabs above main"]
    );
    // The link stays a link.
    assert!(config.path().join("clients").is_symlink());
}

#[tokio::test]
async fn a_save_with_a_key_sets_the_entry_in_one_call() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    server
        .client()
        .save_layout(
            "mine",
            lines(&["output", "editor below main"]),
            Some("laptop"),
            Some("alpha/aria"),
        )
        .await
        .expect("the save");
    assert_eq!(
        place_of(dir.path(), "mine"),
        ["output", "editor below main"]
    );
    let device = fs::read_to_string(dir.path().join("clients/devices/laptop.toml")).unwrap();
    let device: toml::Table = device.parse().expect("the device layer is TOML");
    assert_eq!(device["world-layouts"]["alpha/aria"].as_str(), Some("mine"));
    assert!(!device.contains_key("layout"), "{device:?}");
}

#[tokio::test]
async fn a_save_with_a_key_and_no_device_is_refused() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let error = server
        .client()
        .save_layout(
            "mine",
            lines(&["output", "editor below main"]),
            None,
            Some("alpha/aria"),
        )
        .await
        .expect_err("a key with no device")
        .to_string();
    assert!(error.contains("device"), "{error}");
    assert!(!dir.path().join("clients/layouts/mine.toml").exists());
}

#[tokio::test]
async fn describe_of_a_bad_layout_file_says_what_is_wrong() {
    // The plan of the file has no editor.
    let dir = mine(&["output"]);
    let server = start(dir.path()).await;
    let output = layout(&server, &["describe", "mine"]).await;
    assert_eq!(output.status.code(), Some(1));
    let text = stderr(&output);
    assert!(text.contains("clients/layouts/mine.toml"), "{text}");
    assert!(text.contains("editor"), "{text}");
}

#[tokio::test]
async fn describe_refuses_a_width_over_65535() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let error = server
        .client()
        .describe_layout("classic", 70_000, 24)
        .await
        .expect_err("a width over 65535")
        .to_string();
    assert!(
        error.contains("70000") && error.contains("65535"),
        "{error}"
    );
}

#[tokio::test]
async fn forget_of_an_entry_that_is_not_there_fails() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let output = layout(
        &server,
        &["forget", "--world", "alpha/ari", "--device", "laptop"],
    )
    .await;
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("alpha/ari"), "{}", stderr(&output));
    assert!(!dir.path().join("clients/devices/laptop.toml").exists());
}
