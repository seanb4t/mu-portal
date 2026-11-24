//! The scenarios of the config-store capability, run on the built binary.

use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::Path;
use std::process::{Command, Output};

use support::process::{ServerBuilder, ServerProcess, server_url};
use support::{BASE_GAME, ConfigDir, mu_async, stderr, stdout};

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
    lock(&clients);
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
    // The locked directory is outside the configuration directory, and the link to it is
    // under clients/.
    let dir = ConfigDir::empty();
    let outside = tempfile::tempdir().unwrap();
    let locked_dir = outside.path().join("locked");
    fs::create_dir(&locked_dir).unwrap();
    fs::create_dir_all(dir.path().join("clients/devices")).unwrap();
    symlink(&locked_dir, dir.path().join("clients/devices/x")).unwrap();
    lock(&locked_dir);
    let server = start(dir.path()).await;
    fs::set_permissions(&locked_dir, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        lines_with(&server, "clients/devices/x").len(),
        1,
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

/// The URL of the socket of `server`.
fn url(server: &ServerProcess) -> String {
    server_url(server.socket())
}

/// Runs `mu config show` on `server` for the device `device`, on the OS of the test.
async fn show(server: &ServerProcess, device: &str) -> Output {
    mu_async(
        &[
            "config",
            "show",
            "--server",
            &url(server),
            "--device",
            device,
        ],
        &[],
    )
    .await
}

/// Runs `mu config reload` on `server`.
async fn reload(server: &ServerProcess) -> Output {
    mu_async(&["config", "reload", "--server", &url(server)], &[]).await
}

/// Whether stdout of `output` holds the line `line`.
fn has_line(output: &Output, line: &str) -> bool {
    stdout(output).lines().any(|each| each == line)
}

/// The first line of stdout.
fn first_line(output: &Output) -> String {
    stdout(output).lines().next().unwrap_or_default().to_owned()
}

/// The OS of the test, as its layer file names it.
fn os() -> &'static str {
    mu_config::Os::current().name()
}

/// Starts a server over `dir` and runs `mu config show` for `laptop`.
async fn show_dir(dir: &ConfigDir) -> Output {
    let server = start(dir.path()).await;
    show(&server, "laptop").await
}

/// Asserts that stderr of `output` holds one line with each of `texts`, and that the
/// command succeeds.
fn one_line_with(output: &Output, texts: &[&str]) {
    let stderr = stderr(output);
    let lines: Vec<&str> = stderr
        .lines()
        .filter(|line| texts.iter().all(|text| line.contains(text)))
        .collect();
    assert!(
        lines.len() == 1 && output.status.code() == Some(0),
        "{stderr}"
    );
}

/// Asserts that stderr of `output` holds each of `texts`.
fn stderr_has(output: &Output, texts: &[&str]) {
    let stderr = stderr(output);
    assert!(texts.iter().all(|text| stderr.contains(text)), "{stderr}");
}

#[tokio::test]
async fn no_clients_directory() {
    let output = show_dir(&ConfigDir::base()).await;
    assert!(
        output.status.code() == Some(0) && has_line(&output, "echo = \"dim\" # default"),
        "{}",
        stdout(&output)
    );
}

#[tokio::test]
async fn a_device_with_no_file() {
    let dir = ConfigDir::empty().file("clients/devices/laptop.toml", "echo = \"hide\"\n");
    let server = start(dir.path()).await;
    let output = show(&server, "desk").await;
    assert!(
        has_line(&output, "echo = \"dim\" # default") && stderr(&output).is_empty(),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
}

#[tokio::test]
async fn the_device_wins() {
    let dir = ConfigDir::empty()
        .file("clients/global.toml", "echo = \"show\"\n")
        .file("clients/devices/laptop.toml", "echo = \"hide\"\n");
    let output = show_dir(&dir).await;
    assert!(
        has_line(&output, "echo = \"hide\" # clients/devices/laptop.toml"),
        "{}",
        stdout(&output)
    );
}

#[tokio::test]
async fn keys_merge_key_by_key() {
    let dir = ConfigDir::empty()
        .file(
            "clients/global.toml",
            "[keys]\n\"alt+n\" = \"focus-next\"\n\"alt+p\" = \"focus-prev\"\n",
        )
        .file(
            &format!("clients/os/{}.toml", os()),
            "[keys]\n\"alt+p\" = \"none\"\n",
        );
    let output = show_dir(&dir).await;
    assert!(
        has_line(&output, "\"alt+n\" = \"focus-next\" # clients/global.toml")
            && has_line(
                &output,
                &format!("\"alt+p\" = \"none\" # clients/os/{}.toml", os())
            ),
        "{}",
        stdout(&output)
    );
}

#[tokio::test]
async fn two_names_of_one_key_in_two_layers() {
    let dir = ConfigDir::empty()
        .file(
            "clients/global.toml",
            "[keys]\n\"ctrl+shift+z\" = \"quit\"\n",
        )
        .file(
            "clients/devices/laptop.toml",
            "[keys]\n\"shift+ctrl+z\" = \"none\"\n",
        );
    let output = show_dir(&dir).await;
    let text = stdout(&output);
    let bindings: Vec<&str> = text.lines().filter(|line| line.starts_with('"')).collect();
    assert!(
        bindings == ["\"ctrl+shift+z\" = \"none\" # clients/devices/laptop.toml"]
            && stderr(&output).is_empty(),
        "{bindings:?}"
    );
}

#[tokio::test]
async fn one_bad_binding_skips_its_whole_layer() {
    let dir = ConfigDir::empty()
        .file("clients/global.toml", "echo = \"show\"\n")
        .file(
            "clients/devices/laptop.toml",
            "echo = \"hide\"\n[keys]\n\"alt+n\" = \"fly\"\n",
        );
    let output = show_dir(&dir).await;
    assert!(
        has_line(&output, "echo = \"show\" # clients/global.toml")
            && !stdout(&output).contains("alt+n"),
        "{}",
        stdout(&output)
    );
}

#[tokio::test]
async fn a_layer_without_echo_keeps_the_earlier_echo() {
    let dir = ConfigDir::empty()
        .file("clients/global.toml", "echo = \"hide\"\n")
        .file(
            "clients/devices/laptop.toml",
            "[keys]\n\"alt+n\" = \"focus-next\"\n",
        );
    let output = show_dir(&dir).await;
    assert!(
        has_line(&output, "echo = \"hide\" # clients/global.toml"),
        "{}",
        stdout(&output)
    );
}

#[tokio::test]
async fn a_binding_of_a_dot() {
    let dir = ConfigDir::empty().file("clients/global.toml", "[keys]\n\"alt+.\" = \"quit\"\n");
    let output = show_dir(&dir).await;
    assert!(
        has_line(&output, "\"alt+period\" = \"quit\" # clients/global.toml"),
        "{}",
        stdout(&output)
    );
}

#[tokio::test]
async fn a_notice_for_a_bad_layer_that_applies() {
    let dir = ConfigDir::empty().file("clients/devices/laptop.toml", "echo = \"loud\"\n");
    one_line_with(
        &show_dir(&dir).await,
        &["clients/devices/laptop.toml", "echo"],
    );
}

#[tokio::test]
async fn no_notice_for_a_bad_layer_of_another_device() {
    let dir = ConfigDir::empty().file("clients/devices/laptop.toml", "echo = \"loud\"\n");
    let server = start(dir.path()).await;
    let output = show(&server, "desk").await;
    assert_eq!(stderr(&output), "");
}

#[tokio::test]
async fn show_filters_a_control_character_in_a_notice() {
    let dir = ConfigDir::empty().file(
        "clients/global.toml",
        "[keys]\n\"alt+\\u001b\" = \"quit\"\n",
    );
    let output = show_dir(&dir).await;
    let stderr = stderr(&output);
    assert!(
        !stderr.contains('\u{1b}') && stderr.contains('\u{fffd}'),
        "{stderr:?}"
    );
}

#[tokio::test]
async fn the_same_files_give_the_same_revision() {
    let dir = ConfigDir::empty().file("clients/devices/laptop.toml", "echo = \"hide\"\n");
    let first = show_dir(&dir).await;
    let second = show_dir(&dir).await;
    assert!(
        first_line(&first).starts_with("# revision ") && first_line(&first) == first_line(&second),
        "{} {}",
        first_line(&first),
        first_line(&second)
    );
}

#[tokio::test]
async fn a_changed_layer_gives_a_new_revision() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let before = show(&server, "laptop").await;
    let dir = dir.file("clients/devices/laptop.toml", "echo = \"hide\"\n");
    reload(&server).await;
    let after = show(&server, "laptop").await;
    assert_ne!(first_line(&before), first_line(&after));
    drop(dir);
}

#[tokio::test]
async fn a_layer_of_another_device_leaves_the_revision() {
    let dir = ConfigDir::empty();
    let server = start(dir.path()).await;
    let before = show(&server, "laptop").await;
    let dir = dir.file("clients/devices/desk.toml", "echo = \"hide\"\n");
    reload(&server).await;
    let after = show(&server, "laptop").await;
    assert_eq!(first_line(&before), first_line(&after));
    drop(dir);
}

#[tokio::test]
async fn no_server() {
    let dir = support::process::private_tempdir();
    let socket = dir.path().join("s.sock");
    let url = server_url(&socket);
    let output = mu_async(
        &["config", "show", "--server", &url, "--device", "laptop"],
        &[],
    )
    .await;
    assert!(
        output.status.code() == Some(1)
            && stderr(&output).contains(&socket.display().to_string())
            && !socket.exists(),
        "{}",
        stderr(&output)
    );
}

#[tokio::test]
async fn another_os() {
    let dir = ConfigDir::empty()
        .file("clients/os/linux.toml", "echo = \"hide\"\n")
        .file("clients/os/macos.toml", "echo = \"show\"\n");
    let server = start(dir.path()).await;
    let output = mu_async(
        &[
            "config",
            "show",
            "--server",
            &url(&server),
            "--os",
            "linux",
            "--device",
            "laptop",
        ],
        &[],
    )
    .await;
    assert!(
        has_line(&output, "echo = \"hide\" # clients/os/linux.toml"),
        "{}",
        stdout(&output)
    );
}

#[tokio::test]
async fn an_unknown_os_flag() {
    let output = mu_async(
        &[
            "config",
            "show",
            "--server",
            "unix:///nothing/s.sock",
            "--device",
            "laptop",
            "--os",
            "windows",
        ],
        &[],
    )
    .await;
    assert!(
        output.status.code() == Some(1) && stderr(&output).contains("windows"),
        "{}",
        stderr(&output)
    );
}

/// macOS volumes ignore case by default, so there `Clients` is `clients`.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn a_clients_directory_in_another_case() {
    let dir = ConfigDir::empty().file("Clients/global.toml", "echo = \"hide\"\n");
    assert!(
        dir.path().join("clients").exists(),
        "the volume of the temporary directory does not ignore case"
    );
    let output = show_dir(&dir).await;
    assert!(
        has_line(&output, "echo = \"hide\" # clients/global.toml"),
        "{}",
        stdout(&output)
    );
}

#[tokio::test]
async fn a_reload_applies_a_client_layer() {
    let dir = ConfigDir::base();
    let server = start(dir.path()).await;
    let dir = dir.file("clients/global.toml", "echo = \"hide\"\n");
    let reloaded = reload(&server).await;
    let output = show(&server, "laptop").await;
    assert!(
        reloaded.status.code() == Some(0)
            && has_line(&output, "echo = \"hide\" # clients/global.toml"),
        "{}{}",
        stderr(&reloaded),
        stdout(&output)
    );
    drop(dir);
}

#[tokio::test]
async fn mu_config_refuses_an_open_socket_directory() {
    let dir = ConfigDir::base();
    let server = start(dir.path()).await;
    let socket_dir = server.socket().parent().unwrap().to_owned();
    fs::set_permissions(&socket_dir, fs::Permissions::from_mode(0o755)).unwrap();
    let output = show(&server, "laptop").await;
    fs::set_permissions(&socket_dir, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        output.status.code() == Some(1)
            && stdout(&output).is_empty()
            && stderr(&output).contains(&format!("{} has the mode 0755", socket_dir.display())),
        "{}",
        stderr(&output)
    );
}

#[test]
fn mu_config_filters_what_it_prints() {
    let reply = mu_proto::GetClientConfigResponse {
        revision: "\u{1b}[31m".to_owned(),
        ..Default::default()
    };
    let shown = mu_cli::config_verb::render(&reply);
    let restart = mu_cli::config_verb::restart_lines(&["games/\u{1b}[2Ja.toml".to_owned()]);
    assert!(
        !shown.contains('\u{1b}') && !restart.contains('\u{1b}') && restart.lines().count() == 1,
        "{shown:?} {restart:?}"
    );
}

#[tokio::test]
async fn a_reload_of_a_removed_directory_keeps_the_layers() {
    let dir = ConfigDir::base().file("clients/global.toml", "echo = \"show\"\n");
    let server = start(dir.path()).await;
    let moved = dir.path().with_extension("moved");
    fs::rename(dir.path(), &moved).unwrap();
    let reloaded = reload(&server).await;
    let output = show(&server, "laptop").await;
    fs::rename(&moved, dir.path()).unwrap();
    assert!(
        reloaded.status.code() == Some(1)
            && stderr(&reloaded).contains("the configuration directory does not exist")
            && has_line(&output, "echo = \"show\" # clients/global.toml"),
        "{}{}",
        stderr(&reloaded),
        stdout(&output)
    );
}

#[tokio::test]
async fn the_server_logs_each_notice_of_a_reload() {
    let dir = ConfigDir::base();
    let server = start(dir.path()).await;
    let dir = dir.file("clients/devices/laptop.toml", "echo = \"loud\"\n");
    reload(&server).await;
    server.signal(rustix::process::Signal::TERM);
    let line = server
        .wait_stderr(|line| line.contains("clients/devices/laptop.toml") && line.contains("echo"))
        .await;
    assert!(line.is_some(), "{:?}", server.stderr());
    drop(dir);
}

#[tokio::test]
async fn a_reload_with_a_bad_layer() {
    let dir = ConfigDir::base();
    let server = start(dir.path()).await;
    let dir = dir
        .file("clients/global.toml", "echo = \"show\"\n")
        .file("clients/devices/laptop.toml", "echo = \"loud\"\n");
    let reloaded = reload(&server).await;
    let output = show(&server, "laptop").await;
    let lines: Vec<String> = stderr(&reloaded)
        .lines()
        .filter(|line| line.contains("clients/devices/laptop.toml") && line.contains("echo"))
        .map(str::to_owned)
        .collect();
    assert!(
        lines.len() == 1
            && reloaded.status.code() == Some(1)
            && has_line(&output, "echo = \"show\" # clients/global.toml"),
        "{}{}",
        stderr(&reloaded),
        stdout(&output)
    );
    drop(dir);
}

#[tokio::test]
async fn a_reload_does_not_apply_a_world_change() {
    let dir = ConfigDir::base();
    let server = start(dir.path()).await;
    let dir = dir
        .file(
            "games/beta.toml",
            "extends = \"tinymux\"\nhost = \"localhost\"\nport = 4201\n",
        )
        .file("characters/beta/bryn.toml", "");
    let reloaded = reload(&server).await;
    let worlds = server.client().list_worlds().await.unwrap();
    let ids: Vec<&str> = worlds.iter().map(|world| world.id.as_str()).collect();
    assert!(
        reloaded.status.code() == Some(0)
            && stdout(&reloaded)
                .lines()
                .any(|line| line.contains("characters/beta/bryn.toml") && line.contains("restart"))
            && ids == ["alpha/aria"],
        "{}{ids:?}",
        stdout(&reloaded)
    );
    drop(dir);
}

#[tokio::test]
async fn a_reload_with_a_bad_world_file() {
    let dir = ConfigDir::base();
    let server = start(dir.path()).await;
    let dir = dir.file("characters/alpha/aria.toml", "port = 0\n");
    let reloaded = reload(&server).await;
    let worlds = server.client().list_worlds().await.unwrap();
    assert!(
        reloaded.status.code() == Some(1)
            && stderr(&reloaded)
                .lines()
                .any(|line| line.contains("characters/alpha/aria.toml") && line.contains("port"))
            && worlds.iter().any(|world| world.id == "alpha/aria"),
        "{}",
        stderr(&reloaded)
    );
    drop(dir);
}

#[tokio::test]
async fn a_reload_with_two_bad_world_files() {
    let dir = ConfigDir::base()
        .file("games/beta.toml", BASE_GAME)
        .file("characters/beta/bryn.toml", "");
    let server = start(dir.path()).await;
    let dir = dir
        .file("characters/alpha/aria.toml", "port = 0\n")
        .file("characters/beta/bryn.toml", "port = 0\n");
    let reloaded = reload(&server).await;
    assert_eq!(reloaded.status.code(), Some(1));
    stderr_has(
        &reloaded,
        &["characters/alpha/aria.toml", "characters/beta/bryn.toml"],
    );
    drop(dir);
}

#[tokio::test]
async fn a_reload_with_a_bad_game_of_two_worlds() {
    let dir = ConfigDir::base().file("characters/alpha/ann.toml", "");
    let server = start(dir.path()).await;
    let dir = dir.file("games/alpha.toml", "extends = \"tinymux\"\nport = 0\n");
    let reloaded = reload(&server).await;
    let lines = stderr(&reloaded)
        .lines()
        .filter(|line| line.contains("games/alpha.toml"))
        .count();
    assert!(
        lines == 1 && reloaded.status.code() == Some(1),
        "{}",
        stderr(&reloaded)
    );
    drop(dir);
}

#[tokio::test]
async fn a_bad_game_does_not_hide_a_bad_character_of_another_game() {
    let dir = ConfigDir::base();
    let server = start(dir.path()).await;
    let dir = dir
        .file("games/alpha.toml", "extends = \"nothing\"\n")
        .file("characters/gamma/gil.toml", "");
    let reloaded = reload(&server).await;
    let text = stderr(&reloaded);
    assert!(
        reloaded.status.code() == Some(1)
            && text.contains("games/alpha.toml")
            && text.contains("characters/gamma/gil.toml")
            && !text.contains("characters/alpha/aria.toml"),
        "{text}"
    );
    drop(dir);
}

#[tokio::test]
async fn a_reload_with_two_stray_files() {
    let dir = ConfigDir::base();
    let server = start(dir.path()).await;
    let dir = dir.file("worlds.toml", "").file("games/Beta.toml", "");
    let reloaded = reload(&server).await;
    assert_eq!(reloaded.status.code(), Some(1));
    stderr_has(&reloaded, &["worlds.toml", "games/Beta.toml"]);
    drop(dir);
}

#[tokio::test]
async fn a_reload_with_a_link_to_nothing_in_games() {
    let dir = ConfigDir::base();
    let server = start(dir.path()).await;
    symlink(
        dir.path().join("nothing.toml"),
        dir.path().join("games/ghost.toml"),
    )
    .unwrap();
    let dir = dir.file("clients/global.toml", "echo = \"hide\"\n");
    let reloaded = reload(&server).await;
    let output = show(&server, "laptop").await;
    assert!(
        reloaded.status.code() == Some(1)
            && stderr(&reloaded).contains("games/ghost.toml")
            && has_line(&output, "echo = \"hide\" # clients/global.toml"),
        "{}{}",
        stderr(&reloaded),
        stdout(&output)
    );
    drop(dir);
}

#[tokio::test]
async fn a_reload_lists_a_changed_server_file() {
    let dir = ConfigDir::base().file("server.toml", "");
    let server = start(dir.path()).await;
    let dir = dir.file("server.toml", "[telemetry]\n");
    let reloaded = reload(&server).await;
    assert!(
        reloaded.status.code() == Some(0)
            && stdout(&reloaded)
                .lines()
                .any(|line| line.contains("server.toml") && line.contains("restart")),
        "{}{}",
        stdout(&reloaded),
        stderr(&reloaded)
    );
    drop(dir);
}

#[tokio::test]
async fn a_second_reload_lists_the_world_change_again() {
    let dir = ConfigDir::base();
    let server = start(dir.path()).await;
    let dir = dir
        .file(
            "games/beta.toml",
            "extends = \"tinymux\"\nhost = \"localhost\"\nport = 4201\n",
        )
        .file("characters/beta/bryn.toml", "");
    reload(&server).await;
    let second = reload(&server).await;
    assert!(
        stdout(&second)
            .lines()
            .any(|line| line.contains("characters/beta/bryn.toml") && line.contains("restart")),
        "{}",
        stdout(&second)
    );
    drop(dir);
}

#[tokio::test]
async fn a_reload_filters_a_control_character_in_a_restart_line() {
    let dir = ConfigDir::base();
    let server = start(dir.path()).await;
    let dir = dir.file("characters/alpha/a\u{1b}\nb.toml", "");
    let reloaded = reload(&server).await;
    let stdout = stdout(&reloaded);
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("restart"))
        .collect();
    assert!(
        !stdout.contains('\u{1b}')
            && lines.len() == 1
            && lines[0].contains("characters/alpha/a\u{fffd}\u{fffd}b.toml"),
        "{stdout:?}"
    );
    drop(dir);
}

#[tokio::test]
async fn a_reload_with_no_server() {
    let dir = support::process::private_tempdir();
    let socket = dir.path().join("s.sock");
    let url = server_url(&socket);
    let output = mu_async(&["config", "reload", "--server", &url], &[]).await;
    assert!(
        output.status.code() == Some(1)
            && stderr(&output).contains(&socket.display().to_string())
            && !dir.path().join("server.log").exists(),
        "{}",
        stderr(&output)
    );
}

#[tokio::test]
async fn a_server_table() {
    let dir = ConfigDir::empty().file(
        "clients/global.toml",
        "[server]\nconnect_wait_seconds = 2\n",
    );
    one_line_with(&show_dir(&dir).await, &["clients/global.toml", "server"]);
}

#[tokio::test]
async fn an_unknown_field() {
    let dir = ConfigDir::empty().file("clients/global.toml", "colour = \"red\"\n");
    one_line_with(&show_dir(&dir).await, &["clients/global.toml", "colour"]);
}

#[tokio::test]
async fn not_toml() {
    let dir = ConfigDir::empty().file("clients/global.toml", "echo = ");
    one_line_with(&show_dir(&dir).await, &["clients/global.toml:1:"]);
}

#[tokio::test]
async fn a_bad_echo_value() {
    let dir = ConfigDir::empty().file("clients/global.toml", "echo = \"loud\"\n");
    one_line_with(&show_dir(&dir).await, &["clients/global.toml", "echo"]);
}

#[tokio::test]
async fn a_file_that_cannot_be_read() {
    let dir = ConfigDir::empty().file("clients/global.toml", "echo = \"hide\"\n");
    let file = dir.path().join("clients/global.toml");
    lock(&file);
    let output = show_dir(&dir).await;
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
    one_line_with(&output, &["clients/global.toml"]);
}

#[tokio::test]
async fn two_names_of_one_key() {
    let dir = ConfigDir::empty().file(
        "clients/global.toml",
        "[keys]\n\"ctrl+shift+z\" = \"quit\"\n\"shift+ctrl+z\" = \"none\"\n",
    );
    stderr_has(
        &show_dir(&dir).await,
        &["clients/global.toml", "ctrl+shift+z", "shift+ctrl+z"],
    );
}

#[tokio::test]
async fn an_uppercase_key() {
    let dir = ConfigDir::empty().file("clients/global.toml", "[keys]\n\"ctrl+N\" = \"quit\"\n");
    stderr_has(&show_dir(&dir).await, &["clients/global.toml", "ctrl+N"]);
}

#[tokio::test]
async fn a_control_character() {
    let dir = ConfigDir::empty().file(
        "clients/global.toml",
        "[keys]\n\"alt+\\u007f\" = \"quit\"\n",
    );
    one_line_with(&show_dir(&dir).await, &["clients/global.toml"]);
}

#[tokio::test]
async fn shift_with_a_digit() {
    let dir = ConfigDir::empty().file("clients/global.toml", "[keys]\n\"shift+1\" = \"quit\"\n");
    stderr_has(&show_dir(&dir).await, &["clients/global.toml", "shift+1"]);
}

#[tokio::test]
async fn an_unknown_entry() {
    let dir = ConfigDir::empty().file("clients/global.toml", "[keys]\n\"alt+n\" = \"fly\"\n");
    stderr_has(
        &show_dir(&dir).await,
        &["clients/global.toml", "alt+n", "fly"],
    );
}

#[tokio::test]
async fn an_unknown_key_name() {
    let dir = ConfigDir::empty().file("clients/global.toml", "[keys]\n\"hyper+n\" = \"quit\"\n");
    stderr_has(&show_dir(&dir).await, &["clients/global.toml", "hyper+n"]);
}

#[tokio::test]
async fn an_entry_that_needs_an_argument() {
    let dir = ConfigDir::empty().file("clients/global.toml", "[keys]\n\"alt+f\" = \"focus\"\n");
    stderr_has(
        &show_dir(&dir).await,
        &["clients/global.toml", "alt+f", "focus"],
    );
}
