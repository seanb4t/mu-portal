//! The configuration of the worlds, through `mu world list` and `mu world show`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use support::{BASE_GAME, ConfigDir as Config, fails_naming, mu_command, mu_with, stderr, stdout};

mod support;

impl Config {
    /// `mu world list --config <dir>`.
    fn list(&self) -> Output {
        mu_with(&["world", "list", "--config", self.arg()], &[])
    }

    /// `mu world show <id> --config <dir>`.
    fn show(&self, id: &str) -> Output {
        mu_with(&["world", "show", id, "--config", self.arg()], &[])
    }

    fn arg(&self) -> &str {
        self.path().to_str().unwrap()
    }
}

/// The base game with `extra` lines, and without each top-level key in `drop`.
fn game(drop: &[&str], extra: &str) -> String {
    let mut text = String::new();
    for line in BASE_GAME.lines() {
        if !drop.iter().any(|key| line.starts_with(&format!("{key} "))) {
            text.push_str(line);
            text.push('\n');
        }
    }
    text.push_str(extra);
    text
}

/// Asserts that `output` succeeded, and returns its stdout.
#[track_caller]
fn succeeds(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(output));
    stdout(output)
}

/// Asserts that a line of `text` is `start`, or starts with `start` and a space.
#[track_caller]
fn has_line(text: &str, start: &str) {
    assert!(
        text.lines()
            .any(|line| line == start || line.starts_with(&format!("{start} "))),
        "a line starts with {start}: {text}"
    );
}

/// The ids of the entries of `array` in the output of `show`.
fn ids(text: &str, array: &str) -> Vec<String> {
    let table: toml::Table = text.parse().expect("show prints TOML");
    table
        .get(array)
        .and_then(toml::Value::as_array)
        .map_or_else(Vec::new, |entries| {
            entries
                .iter()
                .map(|entry| entry["id"].as_str().unwrap().to_owned())
                .collect()
        })
}

/// Runs `mu world list` in `cwd` with the test environment, then `env`.
fn list_in(cwd: &Path, env: &[(&str, Option<&str>)]) -> Output {
    let mut command = mu_command(&["world", "list"], env);
    command.current_dir(cwd).output().unwrap()
}

/// The base directory under `root` at `relative`.
fn base_at(root: &Path, relative: &str) -> Config {
    Config::at(root, relative).with_base()
}

/// A directory under `root` at `relative` with the world `beta/bryn` only.
fn beta_at(root: &Path, relative: &str) -> Config {
    Config::at(root, relative)
        .file("games/beta.toml", BASE_GAME)
        .file("characters/beta/bryn.toml", "")
}

// The configuration directory.

#[test]
fn flag_wins_over_the_environment() {
    let root = tempfile::tempdir().unwrap();
    let env = base_at(root.path(), "env");
    let flag = beta_at(root.path(), "flag");
    let output = mu_with(
        &["world", "list", "--config", flag.arg()],
        &[("MU_CONFIG", Some(env.arg()))],
    );
    assert_eq!(succeeds(&output), "beta/bryn\n");
}

#[test]
fn default_directory() {
    let root = tempfile::tempdir().unwrap();
    base_at(root.path(), "mu");
    let output = list_in(
        root.path(),
        &[
            ("MU_CONFIG", None),
            ("XDG_CONFIG_HOME", root.path().to_str()),
        ],
    );
    assert_eq!(succeeds(&output), "alpha/aria\n");
}

#[test]
fn directory_under_home() {
    let root = tempfile::tempdir().unwrap();
    base_at(root.path(), ".config/mu");
    let output = list_in(
        root.path(),
        &[
            ("MU_CONFIG", None),
            ("XDG_CONFIG_HOME", Some("")),
            ("HOME", root.path().to_str()),
        ],
    );
    assert_eq!(succeeds(&output), "alpha/aria\n");
}

#[test]
fn empty_mu_config() {
    let root = tempfile::tempdir().unwrap();
    base_at(root.path(), "mu");
    let output = list_in(
        root.path(),
        &[
            ("MU_CONFIG", Some("")),
            ("XDG_CONFIG_HOME", root.path().to_str()),
        ],
    );
    assert_eq!(succeeds(&output), "alpha/aria\n");
}

#[test]
fn relative_xdg_config_home() {
    let root = tempfile::tempdir().unwrap();
    beta_at(root.path(), "xdg/mu");
    base_at(root.path(), "home/.config/mu");
    let home = root.path().join("home");
    let output = list_in(
        root.path(),
        &[
            ("MU_CONFIG", None),
            ("XDG_CONFIG_HOME", Some("xdg")),
            ("HOME", home.to_str()),
        ],
    );
    assert_eq!(succeeds(&output), "alpha/aria\n");
}

#[test]
fn no_home_directory() {
    let root = tempfile::tempdir().unwrap();
    base_at(root.path(), ".config/mu");
    let output = list_in(
        root.path(),
        &[
            ("MU_CONFIG", None),
            ("XDG_CONFIG_HOME", None),
            ("HOME", Some("")),
        ],
    );
    assert_eq!(succeeds(&output), "");
}

#[test]
fn missing_directory() {
    let root = tempfile::tempdir().unwrap();
    let none = root.path().join("none");
    let output = mu_with(&["world", "list", "--config", none.to_str().unwrap()], &[]);
    assert_eq!(succeeds(&output), "");
    let error = stderr(&output);
    assert!(
        error.starts_with("mu world: warning: ") && error.contains(none.to_str().unwrap()),
        "{error}"
    );
}

#[test]
fn missing_default_directory_is_quiet() {
    let root = tempfile::tempdir().unwrap();
    let output = list_in(
        root.path(),
        &[
            ("MU_CONFIG", None),
            ("XDG_CONFIG_HOME", root.path().to_str()),
            ("HOME", None),
        ],
    );
    assert_eq!(succeeds(&output), "");
    assert_eq!(stderr(&output), "");
}

#[test]
fn helper_points_mu_config_at_an_empty_directory() {
    let (_, value) = support::test_env()
        .into_iter()
        .find(|(name, _)| name == "MU_CONFIG")
        .expect("the test environment sets MU_CONFIG");
    let dir = PathBuf::from(value.expect("MU_CONFIG has a value"));
    assert_eq!(fs::read_dir(dir).unwrap().count(), 0);
}

// The character files.

#[test]
fn worlds_from_character_files() {
    let config = Config::empty()
        .file("games/alpha.toml", BASE_GAME)
        .file("games/beta.toml", BASE_GAME)
        .file("characters/alpha/aria.toml", "")
        .file("characters/alpha/ann.toml", "")
        .file("characters/beta/bryn.toml", "");
    assert_eq!(
        succeeds(&config.list()),
        "alpha/ann\nalpha/aria\nbeta/bryn\n"
    );
}

#[test]
fn character_with_no_game() {
    let config = Config::base().file("characters/gamma/gil.toml", "");
    fails_naming(
        &config.list(),
        &["characters/gamma/gil.toml", "games/gamma.toml"],
    );
}

#[test]
fn file_name_outside_the_id_rule() {
    let config = Config::base().file("games/Beta.toml", BASE_GAME);
    fails_naming(&config.list(), &["games/Beta.toml"]);
}

#[test]
fn stray_file() {
    for path in ["worlds.toml", "games/.toml", "games/beta.TOML"] {
        let config = Config::base().file(path, "");
        fails_naming(&config.list(), &[path]);
    }
}

#[test]
fn a_stray_character_file() {
    let config = Config::base().file("characters/alpha.toml", "");
    fails_naming(&config.list(), &["characters/alpha.toml"]);
}

#[test]
fn a_stray_game_file() {
    let config = Config::base().file("games/old/alpha.toml", "");
    fails_naming(&config.list(), &["games/old/alpha.toml"]);
}

#[test]
fn a_stray_file_under_a_character() {
    let config = Config::base().file("characters/alpha/aria/x.toml", "");
    fails_naming(&config.list(), &["characters/alpha/aria/x.toml"]);
}

#[test]
fn server_file_makes_no_world() {
    let config = Config::base().file(
        "server.toml",
        "[telemetry]\nendpoint = \"http://127.0.0.1:4318\"\n",
    );
    has_line(&succeeds(&config.list()), "alpha/aria");
}

#[test]
fn the_file_of_the_client() {
    let config = Config::base().file("clients/global.toml", "echo = \"hide\"\n");
    has_line(&succeeds(&config.list()), "alpha/aria");
}

#[test]
fn a_game_named_clients() {
    let config = Config::empty()
        .file("games/clients.toml", BASE_GAME)
        .file("characters/clients/aria.toml", "");
    has_line(&succeeds(&config.list()), "clients/aria");
}

#[test]
fn a_bad_client_layer_does_not_stop_the_worlds() {
    let config = Config::base().file("clients/global.toml", "echo = \"loud\"\n");
    has_line(&succeeds(&config.list()), "alpha/aria");
}

#[test]
fn a_leftover_client_toml() {
    let config = Config::base().file("client.toml", "echo = \"hide\"\n");
    fails_naming(&config.list(), &["client.toml", "clients/global.toml"]);
}

/// `name` with its stem in uppercase makes no world where the file system ignores case, and is
/// a stray file where it does not.
fn top_file_in_another_case(name: &str) {
    let other = name.replace(".toml", "").to_uppercase() + ".toml";
    let config = Config::base().file(&other, "");
    let output = config.list();
    // On a file system that ignores case, the reader of the file takes this file as `name`.
    if config.path().join(name).exists() {
        has_line(&succeeds(&output), "alpha/aria");
    } else {
        fails_naming(&output, &[&other]);
    }
}

#[test]
fn server_file_in_another_case() {
    top_file_in_another_case("server.toml");
}

/// An unreadable `name` makes no world, and the list does not fail.
fn unreadable_top_file_makes_no_world(name: &str) {
    use std::os::unix::fs::PermissionsExt as _;
    let config = Config::base().file(name, "");
    fs::set_permissions(config.path().join(name), fs::Permissions::from_mode(0o000)).unwrap();
    has_line(&succeeds(&config.list()), "alpha/aria");
}

#[test]
fn an_unreadable_server_file_leaves_world_list() {
    unreadable_top_file_makes_no_world("server.toml");
}

#[test]
fn a_fifo_named_server_toml_leaves_world_list() {
    let config = Config::base();
    let status = std::process::Command::new("mkfifo")
        .arg(config.path().join("server.toml"))
        .status()
        .unwrap();
    assert!(status.success());
    has_line(&succeeds(&config.list()), "alpha/aria");
}

#[test]
fn a_broken_link_named_server_toml_leaves_world_list() {
    let config = Config::base();
    std::os::unix::fs::symlink(
        config.path().join("nothing"),
        config.path().join("server.toml"),
    )
    .unwrap();
    has_line(&succeeds(&config.list()), "alpha/aria");
}

#[test]
fn unreadable_server_file_in_another_case_names_the_error() {
    use std::os::unix::fs::PermissionsExt as _;
    let config = Config::base().file("SERVER.toml", "");
    let path = config.path().join("SERVER.toml");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    assert!(
        fs::File::open(&path).is_err(),
        "the mode 000 does not keep this user out: run the tests as a user that is not root"
    );
    if config.path().join("server.toml").exists() {
        // A volume that ignores case: the file is server.toml, whose read error the world
        // read ignores.
        has_line(&succeeds(&config.list()), "alpha/aria");
    } else {
        fails_naming(&config.list(), &["SERVER.toml", "Permission denied"]);
    }
}

/// A hard link of `name` under another name is a stray file.
fn hard_link_to_a_top_file_is_an_error(name: &str) {
    let config = Config::base().file(name, "");
    std::fs::hard_link(config.path().join(name), config.path().join("worlds.toml")).unwrap();
    fails_naming(&config.list(), &["worlds.toml"]);
}

#[test]
fn hard_link_to_the_server_file_is_an_error() {
    hard_link_to_a_top_file_is_an_error("server.toml");
}

#[test]
fn server_file_that_links_to_global_keeps_the_layer() {
    let config = Config::base().file("global.toml", "rows = 50\n");
    std::os::unix::fs::symlink("global.toml", config.path().join("server.toml")).unwrap();
    has_line(&succeeds(&config.show("alpha/aria")), "rows = 50");
}

#[test]
fn character_named_server_is_a_world() {
    let config = Config::empty()
        .file("games/alpha.toml", BASE_GAME)
        .file("characters/alpha/server.toml", "");
    has_line(&succeeds(&config.list()), "alpha/server");
}

#[test]
fn link_to_nothing_is_an_error() {
    let config = Config::base();
    std::os::unix::fs::symlink(config.path().join("none"), config.path().join("games/beta"))
        .unwrap();
    fails_naming(&config.list(), &["games/beta"]);
}

#[test]
fn other_files_are_skipped() {
    let config = Config::base()
        .file("games/alpha.toml.bak", "not toml")
        .file("README.md", "")
        .file("characters/alpha/notes.txt", "");
    assert_eq!(succeeds(&config.list()), "alpha/aria\n");
}

// Each field is known.

#[test]
fn unknown_field() {
    let config = Config::base().file(
        "games/alpha.toml",
        &game(&[], "hots = \"mux.example.org\"\n"),
    );
    fails_naming(&config.list(), &["games/alpha.toml", "hots"]);
}

#[test]
fn port_out_of_range() {
    let config = Config::base().file("games/alpha.toml", &game(&["port"], "port = 70000\n"));
    fails_naming(&config.list(), &["games/alpha.toml", "port"]);
}

#[test]
fn charset_case() {
    let mut config = Config::base();
    for (name, charset) in [
        ("alpha", "ISO-8859-1"),
        ("beta", "iso-8859-1"),
        ("gamma", "utf-8"),
    ] {
        config = config.file(
            &format!("games/{name}.toml"),
            &game(&[], &format!("charset = \"{charset}\"\n")),
        );
    }
    succeeds(&config.list());
}

#[test]
fn charset_the_engine_cannot_read() {
    let config = Config::base().file("games/alpha.toml", &game(&[], "charset = \"koi8-r\"\n"));
    fails_naming(&config.list(), &["games/alpha.toml", "charset"]);
}

#[test]
fn unknown_mode() {
    for (field, text) in [
        ("keepalive.mode", "[keepalive]\nmode = \"often\"\n"),
        ("reconnect.mode", "[reconnect]\nmode = \"always\"\n"),
    ] {
        let config = Config::base().file("games/alpha.toml", &game(&[], text));
        fails_naming(&config.list(), &["games/alpha.toml", field]);
    }
}

// Each value is in its range.

/// The text of the game file that sets `field` to `value`.
fn game_with(field: &str, value: u64) -> String {
    match field {
        "keepalive.seconds" => game(
            &[],
            &format!("[keepalive]\ncommand = \"idle\"\nseconds = {value}\n"),
        ),
        "port" => game(&["port"], &format!("port = {value}\n")),
        "reconnect.initial" => game(
            &[],
            &format!("[reconnect]\ninitial = {value}\nmax = 86400\n"),
        ),
        "reconnect.max" => game(&[], &format!("[reconnect]\nmax = {value}\n")),
        "reconnect.attempts" => game(&[], &format!("[reconnect]\nattempts = {value}\n")),
        _ => game(&[], &format!("{field} = {value}\n")),
    }
}

#[test]
fn edges_of_each_range() {
    for (field, top) in [
        ("port", 65_535),
        ("columns", 65_535),
        ("rows", 65_535),
        ("keepalive.seconds", 86_400),
        ("reconnect.initial", 3_600),
        ("reconnect.max", 86_400),
    ] {
        for value in [1, top] {
            let config = Config::base().file("games/alpha.toml", &game_with(field, value));
            assert_eq!(config.list().status.code(), Some(0), "{field} = {value}");
        }
        for value in [0, top + 1] {
            let config = Config::base().file("games/alpha.toml", &game_with(field, value));
            fails_naming(&config.list(), &["games/alpha.toml", field]);
        }
    }
}

#[test]
fn line_break_in_a_line_field() {
    for (field, text) in [
        ("login.name", "[login]\nname = \"Aria\\r\"\n"),
        (
            "login.template",
            "[login]\ntemplate = \"connect {login} {password}\\n\"\n",
        ),
        (
            "keepalive.command",
            "[keepalive]\ncommand = \"idle\\nquit\"\n",
        ),
    ] {
        let config = Config::base().file("games/alpha.toml", &game(&[], text));
        fails_naming(&config.list(), &["games/alpha.toml", field]);
    }
}

#[test]
fn edges_of_the_line_break() {
    let long = "x".repeat(16);
    let accented = "é".repeat(16);
    for value in ["%r", long.as_str(), accented.as_str()] {
        let text = game(&[], &format!("line_break = \"{value}\"\n"));
        let config = Config::base().file("games/alpha.toml", &text);
        assert_eq!(config.list().status.code(), Some(0), "{value}");
    }
    let too_long = "x".repeat(17);
    let too_accented = "é".repeat(17);
    for value in [
        "",
        too_long.as_str(),
        too_accented.as_str(),
        "%\\t",
        "%\\n",
        "%\\r",
        "%\\u0085",
        "%\\u007F",
    ] {
        let text = game(&[], &format!("line_break = \"{value}\"\n"));
        let config = Config::base().file("games/alpha.toml", &text);
        fails_naming(&config.list(), &["games/alpha.toml", "line_break"]);
    }
}

#[test]
fn edges_of_the_attempts() {
    for value in [0, 10_000] {
        let config =
            Config::base().file("games/alpha.toml", &game_with("reconnect.attempts", value));
        succeeds(&config.list());
    }
    let config = Config::base().file("games/alpha.toml", &game_with("reconnect.attempts", 10_001));
    fails_naming(&config.list(), &["games/alpha.toml", "reconnect.attempts"]);
}

#[test]
fn first_wait_over_the_cap() {
    let config = Config::base().file(
        "games/alpha.toml",
        &game(&[], "[reconnect]\ninitial = 120\nmax = 60\n"),
    );
    fails_naming(
        &config.list(),
        &["alpha/aria", "reconnect.initial", "reconnect.max", "120"],
    );
}

#[test]
fn first_wait_over_the_cap_with_reconnect_off() {
    let config = Config::base().file(
        "games/alpha.toml",
        &game(
            &[],
            "[reconnect]\nmode = \"off\"\ninitial = 120\nmax = 60\n",
        ),
    );
    fails_naming(
        &config.list(),
        &["alpha/aria", "reconnect.initial", "reconnect.max"],
    );
}

#[test]
fn first_wait_equal_to_the_cap() {
    let config = Config::base().file(
        "games/alpha.toml",
        &game(&[], "[reconnect]\ninitial = 60\nmax = 60\n"),
    );
    succeeds(&config.list());
}

#[test]
fn template_with_no_password() {
    let config = Config::base().file(
        "games/alpha.toml",
        &game(&[], "[login]\ntemplate = \"connect {login}\"\n"),
    );
    fails_naming(&config.list(), &["games/alpha.toml", "login.template"]);
}

// A password is a secret reference.

#[test]
fn password_in_a_file() {
    let config = Config::base().file(
        "characters/alpha/aria.toml",
        "[login]\npassword = \"hunter2\"\n",
    );
    let output = config.list();
    fails_naming(&output, &["characters/alpha/aria.toml", "login.password"]);
    assert!(!stderr(&output).contains("hunter2"));
}

#[test]
fn secret_that_is_not_a_reference() {
    let config = Config::base().file(
        "characters/alpha/aria.toml",
        "[login]\nname = \"Aria\"\nsecret = \"hunter2\"\n",
    );
    let output = config.list();
    fails_naming(&output, &["characters/alpha/aria.toml", "login.secret"]);
    assert!(!stderr(&output).contains("hunter2"));
}

#[test]
fn wrong_type_under_login() {
    for (field, text, value) in [
        ("login.secret", "[login]\nsecret = 5\n", "integer"),
        ("login", "login = \"hunter2\"\n", "hunter2"),
        ("login", "login = [123456]\n", "123456"),
        ("login", "login = [\"a\", \"b\", 123456]\n", "123456"),
    ] {
        let config = Config::base().file("characters/alpha/aria.toml", text);
        let output = config.list();
        fails_naming(&output, &["characters/alpha/aria.toml", field]);
        assert!(!stderr(&output).contains(value), "{}", stderr(&output));
    }
}

#[test]
fn table_written_as_an_array() {
    for (field, text) in [
        (
            "login",
            "login = [\"connect {login} {password}\", \"Aria\", \"env:ARIA_PW\"]\n",
        ),
        ("keepalive", "keepalive = [\"idle\", 60]\n"),
        ("keepalive", "keepalive = [\"idle\", 60, 7]\n"),
    ] {
        let config = Config::base().file("characters/alpha/aria.toml", text);
        fails_naming(&config.list(), &["characters/alpha/aria.toml", field]);
    }
}

#[test]
fn show_does_not_read_the_secret() {
    let config = Config::base().file(
        "characters/alpha/aria.toml",
        "[login]\nname = \"Aria\"\nsecret = \"env:ARIA_PW\"\n",
    );
    let output = mu_with(
        &["world", "show", "alpha/aria", "--config", config.arg()],
        &[("ARIA_PW", None)],
    );
    assert!(succeeds(&output).contains("env:ARIA_PW"));
}

#[test]
fn show_spells_the_modes_as_the_configuration() {
    for (table, mode) in [
        ("keepalive", "interval"),
        ("keepalive", "off"),
        ("reconnect", "off"),
    ] {
        let config = Config::base().file(
            "games/alpha.toml",
            &game(&[], &format!("[{table}]\nmode = \"{mode}\"\n")),
        );
        let shown = succeeds(&config.show("alpha/aria"));
        let parsed: toml::Table = shown.parse().unwrap();
        assert_eq!(parsed[table]["mode"].as_str(), Some(mode), "{shown}");
    }
}

// The merge.

#[test]
fn last_layer_wins() {
    let config = Config::base()
        .file("global.toml", "columns = 100\n")
        .file("games/alpha.toml", &game(&[], "columns = 90\n"))
        .file("characters/alpha/aria.toml", "columns = 80\n");
    has_line(&succeeds(&config.show("alpha/aria")), "columns = 80");
}

#[test]
fn global_default_reaches_each_world() {
    let config = Config::base().file("global.toml", "rows = 50\n");
    has_line(&succeeds(&config.show("alpha/aria")), "rows = 50");
}

#[test]
fn table_merges_field_by_field() {
    let config = Config::base()
        .file(
            "games/alpha.toml",
            &game(&[], "[login]\ntemplate = \"co {login} {password}\"\n"),
        )
        .file(
            "characters/alpha/aria.toml",
            "[login]\nname = \"Aria\"\nsecret = \"env:ARIA_PW\"\n",
        );
    let text = succeeds(&config.show("alpha/aria"));
    let login = text
        .split("[login]\n")
        .nth(1)
        .expect("show has the table login")
        .split("\n\n")
        .next()
        .unwrap();
    has_line(login, "template = \"co {login} {password}\"");
    has_line(login, "name = \"Aria\"");
    has_line(login, "secret = \"env:ARIA_PW\"");
}

// The line break of a type.

/// The base directory with the game `game` on the type `type_`, and its character `character`.
fn with_game(config: Config, game_name: &str, type_: &str, character: &str) -> Config {
    config
        .file(
            &format!("games/{game_name}.toml"),
            &game(&["extends"], &format!("extends = \"{type_}\"\n")),
        )
        .file(&format!("characters/{game_name}/{character}.toml"), "")
}

#[test]
fn line_break_of_each_built_in_type() {
    let config = with_game(Config::base(), "beta", "aresmush", "bryn");
    let config = with_game(config, "gamma", "evennia", "gil");
    for (id, line) in [
        ("alpha/aria", "line_break = \"%r\""),
        ("beta/bryn", "line_break = \"%r\""),
        ("gamma/gil", "line_break = \"|/\""),
    ] {
        has_line(&succeeds(&config.show(id)), line);
    }
}

#[test]
fn game_overrides_the_line_break_of_its_type() {
    let config = Config::base().file("games/alpha.toml", &game(&[], "line_break = \"%R\"\n"));
    has_line(&succeeds(&config.show("alpha/aria")), "line_break = \"%R\"");
}

#[test]
fn built_in_type_that_extends_another() {
    let config = with_game(Config::base(), "beta", "pennmush", "bryn");
    let config = with_game(config, "gamma", "rhostmush", "gil");
    for id in ["beta/bryn", "gamma/gil"] {
        let text = succeeds(&config.show(id));
        has_line(&text, "charset = \"iso-8859-1\"");
        has_line(&text, "line_break = \"%r\"");
    }
}

// extends.

#[test]
fn user_type_extends_built_in_type() {
    let config = Config::base()
        .file("types/rp.toml", "extends = \"tinymux\"\ncolumns = 78\n")
        .file(
            "games/alpha.toml",
            &game(&["extends"], "extends = \"rp\"\n"),
        );
    let text = succeeds(&config.show("alpha/aria"));
    has_line(&text, "charset = \"iso-8859-1\"");
    has_line(&text, "columns = 78");
}

#[test]
fn unknown_type() {
    let unused = Config::base().file("types/rp.toml", "extends = \"nosuch\"\n");
    fails_naming(&unused.list(), &["types/rp.toml", "nosuch"]);
    let in_game = Config::base().file(
        "games/alpha.toml",
        &game(&["extends"], "extends = \"nosuch\"\n"),
    );
    fails_naming(&in_game.list(), &["games/alpha.toml", "nosuch"]);
}

#[test]
fn game_with_no_type() {
    let config = Config::base().file("games/alpha.toml", &game(&["extends"], ""));
    fails_naming(&config.list(), &["games/alpha.toml", "extends"]);
}

#[test]
fn loop_of_types() {
    let config = Config::base()
        .file("types/a.toml", "extends = \"b\"\n")
        .file("types/b.toml", "extends = \"a\"\n");
    fails_naming(&config.list(), &["types/a.toml", "types/b.toml"]);
}

#[test]
fn type_that_extends_itself() {
    let config = Config::base()
        .file("types/a.toml", "extends = \"a\"\n")
        .file("games/alpha.toml", &game(&["extends"], "extends = \"a\"\n"));
    fails_naming(&config.list(), &["types/a.toml"]);
}

#[test]
fn character_that_names_a_parent() {
    let config = Config::base().file("characters/alpha/aria.toml", "extends = \"beta\"\n");
    fails_naming(&config.list(), &["characters/alpha/aria.toml", "extends"]);
}

// A world has a place to connect.

#[test]
fn world_with_no_host_or_port() {
    let cases = [
        (
            "host",
            Config::base().file("games/alpha.toml", &game(&["host"], "")),
        ),
        (
            "host",
            Config::base().file("characters/alpha/aria.toml", "host = \"\"\n"),
        ),
        (
            "port",
            Config::base().file("games/alpha.toml", &game(&["port"], "")),
        ),
    ];
    for (field, config) in cases {
        fails_naming(&config.list(), &["alpha/aria", field]);
    }
}

#[test]
fn login_name_with_no_secret() {
    let config = Config::base().file("characters/alpha/aria.toml", "[login]\nname = \"Aria\"\n");
    fails_naming(&config.list(), &["alpha/aria", "login.secret"]);
}

#[test]
fn login_secret_with_no_name() {
    let config = Config::base().file(
        "characters/alpha/aria.toml",
        "[login]\nsecret = \"env:ARIA_PW\"\n",
    );
    fails_naming(&config.list(), &["alpha/aria", "login.name"]);
}

#[test]
fn half_a_keepalive() {
    for (text, missing) in [
        ("[keepalive]\ncommand = \"idle\"\n", "keepalive.seconds"),
        ("[keepalive]\nseconds = 60\n", "keepalive.command"),
    ] {
        let config = Config::base().file(
            "games/alpha.toml",
            &game(&["extends"], &format!("extends = \"aresmush\"\n{text}")),
        );
        fails_naming(&config.list(), &["alpha/aria", missing]);
    }
}

#[test]
fn half_keepalive_that_is_off() {
    let config = Config::base().file(
        "games/alpha.toml",
        &game(
            &["extends"],
            "extends = \"aresmush\"\n[keepalive]\nmode = \"off\"\ncommand = \"idle\"\n",
        ),
    );
    succeeds(&config.list());
}

#[test]
fn mode_with_no_keepalive() {
    let config = Config::base().file(
        "games/alpha.toml",
        &game(
            &["extends"],
            "extends = \"aresmush\"\n[keepalive]\nmode = \"interval\"\n",
        ),
    );
    let text = succeeds(&config.show("alpha/aria"));
    let keepalive = lines_under(&text, "keepalive");
    assert_eq!(keepalive.len(), 1, "{text}");
    assert!(keepalive[0].starts_with("mode = \"interval\" "), "{text}");
}

#[test]
fn world_that_logs_in_by_hand() {
    let text = succeeds(&Config::base().show("alpha/aria"));
    assert!(!text.contains("[login]"), "{text}");
}

// Show.

#[test]
fn sources_of_fields() {
    let text = succeeds(&Config::base().show("alpha/aria"));
    text.parse::<toml::Table>().expect("show prints TOML");
    for (field, source) in [
        ("host = ", "# games/alpha.toml"),
        ("charset = ", "# built-in tinymux"),
        ("rows = ", "# default"),
    ] {
        let line = text
            .lines()
            .find(|line| line.starts_with(field))
            .unwrap_or_else(|| panic!("show has {field}: {text}"));
        assert!(line.ends_with(source), "{line}");
    }
}

/// The lines of `text` that are not blank, under the header `[table]`, or before the first
/// header for `""`.
fn lines_under<'a>(text: &'a str, table: &str) -> Vec<&'a str> {
    let header = format!("[{table}]");
    let mut current = "";
    text.lines()
        .filter(|line| !line.is_empty())
        .filter(|line| {
            if line.starts_with('[') {
                current = line;
                return false;
            }
            current == header || (table.is_empty() && current.is_empty())
        })
        .collect()
}

#[test]
fn connection_defaults() {
    let text = succeeds(&Config::base().show("alpha/aria"));
    for (table, line) in [
        ("", "autoconnect = false # default"),
        ("keepalive", "mode = \"idle\" # default"),
        ("reconnect", "mode = \"backoff\" # default"),
        ("reconnect", "initial = 1 # default"),
        ("reconnect", "max = 60 # default"),
        ("reconnect", "attempts = 30 # default"),
    ] {
        assert!(
            lines_under(&text, table).contains(&line),
            "[{table}] has {line}: {text}"
        );
    }
}

#[test]
fn unknown_world() {
    for id in ["alpha/nobody", "Alpha/x"] {
        fails_naming(&Config::base().show(id), &[id]);
    }
}

// Rules and scripts.

#[test]
fn entries_from_each_layer() {
    let config = Config::base()
        .file("global.toml", "[[rules]]\nid = \"hl-page\"\n")
        .file(
            "games/alpha.toml",
            &game(&[], "[[rules]]\nid = \"hl-ooc\"\n"),
        )
        .file(
            "characters/alpha/aria.toml",
            "[[rules]]\nid = \"hl-aria\"\n",
        );
    let text = succeeds(&config.show("alpha/aria"));
    assert_eq!(ids(&text, "rules"), ["hl-page", "hl-ooc", "hl-aria"]);
}

#[test]
fn layer_disables_an_inherited_rule() {
    let config = Config::base()
        .file(
            "global.toml",
            "[[rules]]\nid = \"hl-page\"\n[[rules]]\nid = \"hl-ooc\"\n",
        )
        .file("characters/alpha/aria.toml", "disable = [\"hl-ooc\"]\n");
    let text = succeeds(&config.show("alpha/aria"));
    assert_eq!(ids(&text, "rules"), ["hl-page"]);
}

#[test]
fn layer_replaces_an_inherited_rule() {
    let config = Config::base()
        .file(
            "global.toml",
            "[[rules]]\nid = \"hl-page\"\npattern = \"pages\"\n[[rules]]\nid = \"hl-ooc\"\n",
        )
        .file(
            "games/alpha.toml",
            &game(
                &[],
                "[[rules]]\nid = \"hl-page\"\npattern = \"pages you\"\n",
            ),
        );
    let text = succeeds(&config.show("alpha/aria"));
    assert_eq!(ids(&text, "rules"), ["hl-page", "hl-ooc"]);
    has_line(&text, "pattern = \"pages you\"");
    assert!(
        !text.lines().any(|line| line == "pattern = \"pages\""),
        "{text}"
    );
}

#[test]
fn entry_with_no_id() {
    for rule in ["[[rules]]\npattern = \"x\"\n", "[[rules]]\nid = 1\n"] {
        let config = Config::base().file("games/alpha.toml", &game(&[], rule));
        fails_naming(&config.list(), &["games/alpha.toml", "id"]);
    }
}

#[test]
fn disable_with_no_target() {
    let config = Config::base().file("characters/alpha/aria.toml", "disable = [\"nosuch\"]\n");
    fails_naming(&config.list(), &["characters/alpha/aria.toml", "nosuch"]);
}

#[test]
fn id_twice_in_one_layer() {
    let config = Config::base().file(
        "games/alpha.toml",
        &game(
            &[],
            "[[rules]]\nid = \"hl-page\"\n[[rules]]\nid = \"hl-page\"\n",
        ),
    );
    fails_naming(&config.list(), &["games/alpha.toml", "hl-page"]);
}

#[test]
fn rule_and_script_with_one_id() {
    let two_layers = Config::base()
        .file("global.toml", "[[rules]]\nid = \"greet\"\n")
        .file(
            "games/alpha.toml",
            &game(&[], "[[scripts]]\nid = \"greet\"\n"),
        );
    let one_layer = Config::base().file(
        "games/alpha.toml",
        &game(
            &[],
            "[[rules]]\nid = \"greet\"\n[[scripts]]\nid = \"greet\"\n",
        ),
    );
    for config in [two_layers, one_layer] {
        fails_naming(&config.list(), &["games/alpha.toml", "greet"]);
    }
}

// Built-in types.

#[test]
fn global_preference_is_not_hidden_by_a_type() {
    let config = Config::base().file("global.toml", "columns = 100\n");
    has_line(&succeeds(&config.show("alpha/aria")), "columns = 100");
}

#[test]
fn user_type_with_a_built_in_name() {
    let config = Config::base().file("types/tinymux.toml", "columns = 80\n");
    fails_naming(&config.list(), &["types/tinymux.toml", "tinymux"]);
}

// The output of show parses back.

mod parses_back {
    use std::collections::BTreeMap;

    use mu_core::world::{Entry, LayerOrigin};
    use mu_core::{Keepalive, Login, ResolvedWorld};
    use proptest::prelude::*;

    /// A value of an entry: a string, a nested table, or an array of tables.
    fn value() -> impl Strategy<Value = toml::Value> {
        let leaf = any::<String>().prop_map(toml::Value::from);
        leaf.prop_recursive(2, 12, 3, |inner| {
            let table = prop::collection::btree_map(any::<String>(), inner, 0..3)
                .prop_map(|map| map.into_iter().collect::<toml::Table>());
            prop_oneof![
                table.clone().prop_map(toml::Value::Table),
                prop::collection::vec(table.prop_map(toml::Value::Table), 1..3)
                    .prop_map(toml::Value::Array),
            ]
        })
    }

    /// An entry with the id `id` and keys from any string.
    fn entry() -> impl Strategy<Value = toml::Table> {
        (
            any::<String>(),
            prop::collection::btree_map(any::<String>(), value(), 0..4),
        )
            .prop_map(|(id, map)| {
                let mut table: toml::Table = map.into_iter().collect();
                table.insert("id".to_owned(), id.into());
                table
            })
    }

    fn entries(tables: Vec<toml::Table>) -> Vec<Entry> {
        tables
            .into_iter()
            .map(|table| Entry {
                id: table["id"].as_str().unwrap().to_owned(),
                table,
                origin: LayerOrigin::File("games/alpha.toml".to_owned()),
            })
            .collect()
    }

    proptest! {
        #[test]
        fn show_output_parses_back(
            host in any::<String>(),
            name in "[^\r\n]*",
            template in "[^\r\n]*\\{password\\}[^\r\n]*",
            command in "[^\r\n]*",
            rules in prop::collection::vec(entry(), 0..3),
            scripts in prop::collection::vec(entry(), 0..3),
        ) {
            let world = ResolvedWorld {
                host: host.clone(),
                port: 4201,
                login: Some(Login::new(&template, &name, "env:ARIA_PW".parse().unwrap()).unwrap()),
                keepalive: Some(Keepalive::new(&command, 60).unwrap()),
                rules: entries(rules.clone()),
                scripts: entries(scripts.clone()),
                sources: BTreeMap::new(),
                ..ResolvedWorld::new("alpha/aria".parse().unwrap())
            };
            let text = mu_cli::world::render(&world).unwrap();
            let parsed: toml::Table = text.parse().map_err(|error| {
                TestCaseError::fail(format!("{error}\n{text}"))
            })?;
            prop_assert_eq!(parsed["host"].as_str(), Some(host.as_str()));
            prop_assert_eq!(parsed["login"]["name"].as_str(), Some(name.as_str()));
            prop_assert_eq!(parsed["login"]["template"].as_str(), Some(template.as_str()));
            prop_assert_eq!(parsed["keepalive"]["command"].as_str(), Some(command.as_str()));
            let array = |key: &str| -> Vec<toml::Table> {
                parsed.get(key).and_then(toml::Value::as_array).map_or_else(Vec::new, |values| {
                    values.iter().map(|value| value.as_table().unwrap().clone()).collect()
                })
            };
            prop_assert_eq!(array("rules"), rules);
            prop_assert_eq!(array("scripts"), scripts);
        }
    }
}
