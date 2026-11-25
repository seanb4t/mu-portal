//! The scenarios of `mu client` on the built binary, in a pseudo-terminal of 80 by 24.

use std::time::Duration;

use mu_client::PortalClient;
use mu_proto::{EventKind, WorldState};

use mu_fake_server::{Listening, Script, ScriptedGame};
use support::process::{
    Answers, BOUND, PtyClient, PtyOn, Reaper, Relay, SERVING, ServerBuilder, ServerProcess,
    Session, TWO_WORLDS, greeting, mu_on_pty, private_tempdir, server_url,
};
use support::{ConfigDir, mode, mu_async, stderr};

mod support;

#[tokio::test]
async fn a_client_with_no_server_starts_one() {
    let dir = private_tempdir();
    let run = dir.path().join("run");
    let log = run.join("server.log");
    let _reaper = Reaper(log.clone());
    let socket = run.join("mu.sock");
    let mut client = PtyClient::start(&socket, dir.path());
    client
        .wait_top("no worlds", |top| top.trim() == "no worlds")
        .await;
    assert_eq!(mode(&run), 0o700);
    client.send(b"\x03");
    assert!(
        client
            .exit_within(BOUND)
            .await
            .expect("the client exits")
            .success()
    );
    // The server outlives the client that started it.
    assert_eq!(
        PortalClient::unix(&socket)
            .list_worlds()
            .await
            .unwrap()
            .len(),
        0
    );
    let log = std::fs::read_to_string(&log).unwrap();
    assert!(log.lines().any(|line| line.starts_with(SERVING)), "{log}");
}

#[tokio::test]
async fn two_clients_start_at_the_same_time() {
    let dir = private_tempdir();
    let run = dir.path().join("run");
    let log = run.join("server.log");
    let _reaper = Reaper(log.clone());
    let socket = run.join("mu.sock");
    let first = PtyClient::start(&socket, dir.path());
    let second = PtyClient::start(&socket, dir.path());
    for client in [&first, &second] {
        client
            .wait_top("no worlds", |top| top.trim() == "no worlds")
            .await;
    }
    let log = std::fs::read_to_string(&log).unwrap();
    let served: Vec<&str> = log
        .lines()
        .filter(|line| line.starts_with(SERVING))
        .collect();
    assert_eq!(served.len(), 1, "{log}");
    assert!(served[0].contains(&socket.display().to_string()), "{log}");
}

#[tokio::test]
async fn no_server_answers_in_time() {
    let dir = private_tempdir();
    let socket = dir.path().join("s.sock");
    // The held lock makes the server that the client starts refuse to serve.
    let lock = std::fs::File::create(mu_server::lock_path(&socket)).unwrap();
    lock.try_lock().unwrap();
    let mut client = PtyClient::start(&socket, dir.path());
    assert!(
        client.exit_within(Duration::from_secs(4)).await.is_none(),
        "exited before 4 seconds"
    );
    // 5 seconds in all, and a margin for the start of the process.
    let status = client
        .exit_within(Duration::from_secs(3))
        .await
        .expect("the client exits within 7 seconds");
    assert_eq!(status.exit_code(), 1);
    client.wait_line("mu client: ").await;
    let screen = client.contents().replace('\n', "");
    let log = dir.path().join("server.log");
    for name in [socket.display().to_string(), log.display().to_string()] {
        assert!(
            screen.contains(&name),
            "{name} is not on the screen:\n{screen}"
        );
    }
}

/// A server with two worlds, and a client on it that shows the greeting of `alpha/aria`.
async fn attached() -> (Session, PtyClient) {
    let session = Session::with_worlds(TWO_WORLDS).await;
    let client = PtyClient::start(session.server.socket(), session.server.dir().path());
    client.wait_greeting().await;
    (session, client)
}

#[tokio::test]
async fn typed_line_reaches_the_server() {
    let (mut session, mut client) = attached().await;
    client.send(b"hi\r");
    let event = session
        .until("the player event", |event| {
            event.kind == EventKind::EVENT_KIND_INPUT
        })
        .await;
    assert_eq!(
        (event.world.as_str(), event.text.as_str()),
        ("alpha/aria", "hi")
    );
    client
        .wait_rows("an empty input line", |rows| {
            rows.last().is_some_and(|row| row.trim().is_empty())
        })
        .await;
}

#[tokio::test]
async fn empty_enter_reaches_the_server() {
    let (mut session, mut client) = attached().await;
    client.send(b"\r");
    let event = session
        .until("the player event", |event| {
            event.kind == EventKind::EVENT_KIND_INPUT
        })
        .await;
    assert_eq!(
        (event.world.as_str(), event.text.as_str()),
        ("alpha/aria", "")
    );
}

/// Runs `mu client` with the URL of a socket in a new empty directory, and a terminal on `on`
/// only, and asserts the refusal and the empty directory.
fn refused_without_terminal(on: PtyOn) {
    let dir = private_tempdir();
    let url = server_url(&dir.path().join("s.sock"));
    let output = mu_on_pty(&["client", "--server", &url], on, dir.path());
    let stderr = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{on:?}");
    assert!(stderr.contains("terminal"), "{on:?}: {stderr}");
    assert!(!stderr.contains("not implemented"), "{on:?}");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0, "{on:?}");
}

#[test]
fn no_terminal() {
    refused_without_terminal(PtyOn::Stdin);
}

#[test]
fn no_terminal_on_stdin() {
    refused_without_terminal(PtyOn::Stdout);
}

#[test]
fn the_socket_flag() {
    let dir = private_tempdir();
    let socket = dir.path().join("s.sock");
    let output = mu_on_pty(
        &["client", "--socket", socket.to_str().unwrap()],
        PtyOn::Stdin,
        dir.path(),
    );
    assert!(
        output.status.code() == Some(2) && stderr(&output).contains("--socket"),
        "{}",
        stderr(&output)
    );
    assert!(!socket.exists());
}

#[tokio::test]
async fn lost_stream_resumes_through_relay() {
    let server = ServerBuilder::new().fake_worlds(TWO_WORLDS).start().await;
    let relay = Relay::start(&server.dir().path().join("relay.sock"), server.socket());
    let client = PtyClient::start(relay.socket(), server.dir().path());
    client.wait_greeting().await;
    relay.hold();
    assert!(relay.cut() >= 1, "no connection to cut");
    client
        .wait_top("disconnected", |top| top.contains("disconnected"))
        .await;
    relay.release();
    server
        .client()
        .send("alpha/aria", "after the cut")
        .await
        .unwrap();
    let line = "You say, \"after the cut\"";
    client
        .wait_rows("the line after the cut", |rows| {
            rows.iter().any(|row| row.trim() == line) && !rows[0].contains("disconnected")
        })
        .await;
    let rows = client.rows();
    assert_eq!(
        rows.iter().filter(|row| row.trim() == line).count(),
        1,
        "{}",
        client.contents()
    );
}

#[tokio::test]
async fn ctrl_c_quits_client_only() {
    let server = ServerBuilder::new().fake_worlds(TWO_WORLDS).start().await;
    let mut client = PtyClient::start(server.socket(), server.dir().path());
    client.wait_greeting().await;
    client.send(b"\x03");
    let status = client.exit_within(BOUND).await.expect("the client exits");
    assert!(status.success(), "{status:?}");
    let worlds = server.client().list_worlds().await.unwrap();
    assert_eq!(worlds.len(), 2);
    assert!(
        worlds
            .iter()
            .all(|world| world.state == WorldState::WORLD_STATE_CONNECTED)
    );
}

/// A server with the configured world `alpha/aria` on `game`, and a client on it. The game
/// listens and the configuration stays until the last part drops.
async fn on_game(game: &ScriptedGame) -> (Session, PtyClient, (Listening, ConfigDir)) {
    on_game_with(game, "", Answers::Default).await
}

/// [`on_game`], with `more` in the character file and `answers` to the probe of the client.
async fn on_game_with(
    game: &ScriptedGame,
    more: &str,
    answers: Answers,
) -> (Session, PtyClient, (Listening, ConfigDir)) {
    let listening = game.listen().await.unwrap();
    let character = format!("autoconnect = true\n{more}");
    let conf = ConfigDir::tcp_world(listening.addr().port(), &character);
    let session = Session::start(ServerBuilder::new().arg("--config").arg(conf.path())).await;
    let client = PtyClient::start_with(
        session.server.socket(),
        session.server.dir().path(),
        answers,
    );
    (session, client, (listening, conf))
}

/// The character setting of a game that sends UTF-8.
const UTF8: &str = "charset = \"utf-8\"\n";

/// The family emoji `👨‍👩‍👧`, a grapheme of the ZWJ class.
const FAMILY: &str = "👨\u{200D}👩\u{200D}👧";

/// Waits until the row that starts with the man of [`FAMILY`] has `x` at the 0-based column
/// `col`.
async fn x_at(client: &PtyClient, col: u16) {
    let has = |screen: &vt100::Screen, row, col, text: &str| {
        screen
            .cell(row, col)
            .is_some_and(|cell| cell.contents().starts_with(text))
    };
    client
        .wait_screen(&format!("x at column {}", col + 1), |screen| {
            (0..24).any(|row| has(screen, row, 0, "👨") && has(screen, row, col, "x"))
        })
        .await;
}

fn family_game() -> ScriptedGame {
    ScriptedGame::new(
        Script::new()
            .send(format!("{FAMILY}x\r\n").as_bytes())
            .silent(),
    )
}

#[tokio::test]
async fn probe_adds_code_point_widths() {
    let game = family_game();
    let (_session, client, _held) = on_game_with(&game, UTF8, Answers::ZwjWidth(8)).await;
    x_at(&client, 8).await;
}

#[tokio::test]
async fn probe_with_no_answer() {
    let game = family_game();
    let (_session, client, _held) = on_game_with(&game, UTF8, Answers::None).await;
    client
        .wait_top("the world", |top| top.contains("alpha/aria"))
        .await;
    x_at(&client, 2).await;
    client
        .wait_rows("the notice of no widths", |rows| {
            rows.iter()
                .any(|row| row.starts_with("-- the terminal gave no character widths"))
        })
        .await;
}

#[tokio::test]
async fn late_answer() {
    let session = Session::with_worlds(TWO_WORLDS).await;
    let mut client = PtyClient::start_with(
        session.server.socket(),
        session.server.dir().path(),
        Answers::Late,
    );
    client.wait_greeting().await;
    // The late answers went out before the frame of the greeting, so the client reads them
    // before this key.
    client.send(b"z");
    client
        .wait_rows("only z on the input line", |rows| {
            rows.last().is_some_and(|row| row.trim() == "z")
        })
        .await;
}

#[tokio::test]
async fn colors_on_the_terminal() {
    let game = ScriptedGame::new(
        Script::new()
            .send(b"\x1b[1;31mred\x1b[0m plain\r\n")
            .silent(),
    );
    let (_session, client, _held) = on_game(&game).await;
    client
        .wait_rows("the colored line", |rows| {
            rows.iter().any(|row| row.trim_end() == "red plain")
        })
        .await;
    let looks = client.looks("red plain");
    assert_eq!(
        (&looks[..3], &looks[3..]),
        (
            &[(vt100::Color::Idx(1), true); 3][..],
            &[(vt100::Color::Default, false); 6][..]
        )
    );
}

#[tokio::test]
async fn line_in_two_reads() {
    let game = ScriptedGame::new(
        Script::new()
            .send(b"hel")
            .wait_for(b"x")
            .send(b"lo\r\n")
            .silent(),
    );
    let (_session, mut client, _held) = on_game(&game).await;
    client
        .wait_rows("the unfinished line", |rows| {
            rows.iter().any(|row| row.trim_end() == "hel")
        })
        .await;
    client.send(b"x\r");
    client
        .wait_rows("the finished line", |rows| {
            rows.iter().any(|row| row.trim_end() == "hello")
        })
        .await;
    let rows = client.rows();
    let shown: Vec<&str> = rows
        .iter()
        .map(|row| row.trim_end())
        .filter(|row| row.starts_with("hel"))
        .collect();
    assert_eq!(shown, ["hello"], "{}", client.contents());
}

/// The bytes of an SGR mouse report of the left button at the cell `col`, `row`, counted from
/// 0: a press, a drag or a release.
fn mouse(kind: &str, col: usize, row: usize) -> Vec<u8> {
    let (button, end) = match kind {
        "press" => (0, 'M'),
        "drag" => (32, 'M'),
        _ => (0, 'm'),
    };
    format!("\x1b[<{button};{};{}{end}", col + 1, row + 1).into_bytes()
}

#[tokio::test]
async fn copy_by_drag() {
    let (_session, mut client) = attached().await;
    let said = "You say, \"hello\"";
    client.send(b"hello\r");
    client
        .wait_rows("the answer", |rows| {
            rows.iter().any(|row| row.trim_end() == said)
        })
        .await;
    let row = client
        .rows()
        .iter()
        .position(|row| row.trim_end() == said)
        .unwrap();
    let h = said.find('h').unwrap();
    let o = h + 4;
    client.send(&mouse("press", h, row));
    client.send(&mouse("drag", o, row));
    client.send(&mouse("release", o, row));
    client
        .wait_output("the OSC 52 copy", b"\x1b]52;c;aGVsbG8=\x1b\\")
        .await;
    let want: Vec<bool> = (0..80).map(|col| (h..=o).contains(&col)).collect();
    let row = u16::try_from(row).unwrap();
    client
        .wait_screen("the reversed cells", |screen| {
            (0..80)
                .map(|col| screen.cell(row, col).is_some_and(vt100::Cell::inverse))
                .collect::<Vec<_>>()
                == want
        })
        .await;
}

/// The sequences that turn off mouse capture: SGR reports, drag, and press and release.
const MOUSE_OFF: [&[u8]; 3] = [b"\x1b[?1006l", b"\x1b[?1002l", b"\x1b[?1000l"];

/// Starts a client with `answers`, presses Ctrl+C, and returns every byte that the client wrote
/// after the greeting. The exit from the alternate screen comes after the modes.
async fn output_at_exit(answers: Answers) -> Vec<u8> {
    let session = Session::with_worlds(TWO_WORLDS).await;
    let mut client = PtyClient::start_with(
        session.server.socket(),
        session.server.dir().path(),
        answers,
    );
    client.wait_greeting().await;
    let started = client.output().len();
    client.send(b"\x03");
    assert!(
        client
            .exit_within(BOUND)
            .await
            .expect("the client exits")
            .success()
    );
    client
        .wait_output("the exit from the alternate screen", b"\x1b[?1049l")
        .await;
    client.output().split_off(started)
}

fn holds(output: &[u8], needle: &[u8]) -> bool {
    output.windows(needle.len()).any(|window| window == needle)
}

#[tokio::test]
async fn modes_at_exit() {
    let output = output_at_exit(Answers::Default).await;
    let off: Vec<bool> = MOUSE_OFF
        .iter()
        .chain([&&b"\x1b[?2027l"[..]])
        .map(|sequence| holds(&output, sequence))
        .collect();
    assert_eq!(off, [true; 4], "{}", output.escape_ascii());
}

#[tokio::test]
async fn late_mode_reply_still_turns_mode_2027_off() {
    let output = output_at_exit(Answers::Late).await;
    assert!(holds(&output, b"\x1b[?2027l"), "{}", output.escape_ascii());
}

#[tokio::test]
async fn late_reply_of_mode_2027_as_set_keeps_it_on() {
    let output = output_at_exit(Answers::LateGraphemeModeSet).await;
    assert!(!holds(&output, b"\x1b[?2027l"), "{}", output.escape_ascii());
}

#[tokio::test]
async fn mode_2027_already_on() {
    let output = output_at_exit(Answers::GraphemeModeSet).await;
    let mouse_off: Vec<bool> = MOUSE_OFF
        .iter()
        .map(|sequence| holds(&output, sequence))
        .collect();
    assert_eq!(
        (mouse_off, holds(&output, b"\x1b[?2027l")),
        (vec![true; 3], false),
        "{}",
        output.escape_ascii()
    );
}

#[tokio::test]
async fn failed_start_leaves_the_terminal_as_it_was() {
    let session = Session::with_worlds(TWO_WORLDS).await;
    let mut client = PtyClient::start_sized(
        session.server.socket(),
        session.server.dir().path(),
        Answers::Default,
        (0, 0),
    );
    let status = client.exit_within(BOUND).await.expect("the client exits");
    let output = client.output();
    let entered = holds(&output, b"\x1b[?1049h") || holds(&output, b"\x1b[?1000h");
    assert_eq!(
        (status.success(), entered && !holds(&output, b"\x1b[?1049l")),
        (false, false)
    );
}

// The pose editor.

/// The last `count` rows of `rows`, with no trailing spaces.
fn last_rows(rows: &[String], count: usize) -> Vec<&str> {
    rows[rows.len() - count..]
        .iter()
        .map(|row| row.trim_end())
        .collect()
}

#[tokio::test]
async fn a_pose_of_two_lines() {
    let (_session, mut client) = attached().await;
    // Ctrl+J is the byte LF.
    client.send(b"one\ntwo\r");
    client
        .wait_rows("the joined pose, and an empty editor of one row", |rows| {
            rows.iter().any(|row| row.contains("You say, \"one%rtwo\""))
                && last_rows(rows, 1) == [""]
                && !rows.iter().any(|row| row.trim_end() == "one")
        })
        .await;
}

#[tokio::test]
async fn alt_enter_sends_each_line() {
    let (_session, mut client) = attached().await;
    // Alt+Enter is ESC and CR.
    client.send(b"one\ntwo\x1b\r");
    client
        .wait_rows("an answer to each line, in order", |rows| {
            let at = |text: &str| rows.iter().position(|row| row.contains(text));
            match (at("You say, \"one\""), at("You say, \"two\"")) {
                (Some(one), Some(two)) => one < two && last_rows(rows, 1) == [""],
                _ => false,
            }
        })
        .await;
}

#[tokio::test]
async fn a_line_break_that_the_world_refuses() {
    let game = ScriptedGame::new(Script::new().silent());
    let listening = game.listen().await.unwrap();
    let port = listening.addr().port();
    // A type with no `extends` sets no line break.
    let conf = ConfigDir::tcp_world(port, "autoconnect = true\n")
        .file("types/plain.toml", "")
        .file(
            "games/alpha.toml",
            &format!("extends = \"plain\"\nhost = \"127.0.0.1\"\nport = {port}\n"),
        );
    let session = Session::start(ServerBuilder::new().arg("--config").arg(conf.path())).await;
    let mut client = PtyClient::start(session.server.socket(), session.server.dir().path());
    client
        .wait_top("the world", |top| top.contains("alpha/aria"))
        .await;
    client.send(b"one\ntwo\r");
    client
        .wait_rows("the refusal, and the two lines in the editor", |rows| {
            rows.iter()
                .any(|row| row.starts_with("-- ") && row.contains("line break"))
                && last_rows(rows, 2) == ["one", "two"]
        })
        .await;
}

#[tokio::test]
async fn bracketed_paste_at_the_start_and_at_exit() {
    let session = Session::with_worlds(TWO_WORLDS).await;
    let mut client = PtyClient::start(session.server.socket(), session.server.dir().path());
    client.wait_greeting().await;
    // The greeting is in the first frame with a world, so the mode came before it.
    assert!(holds(&client.output(), b"\x1b[?2004h"));
    assert!(!holds(&client.output(), b"\x1b[?2004l"));
    client.send(b"\x03");
    assert!(client.exit_within(BOUND).await.is_some());
    client
        .wait_output("bracketed paste off", b"\x1b[?2004l")
        .await;
}

#[tokio::test]
async fn paste_writes_no_escape() {
    let (_session, mut client) = attached().await;
    let started = client.output().len();
    client.send(b"\x1b[200~a\x1b[31mb\x07\tc\xc2\x9bd\x1b[201~");
    client
        .wait_rows("the pasted text with no control character", |rows| {
            last_rows(rows, 1) == ["a[31mb  cd"]
        })
        .await;
    let output = client.output().split_off(started);
    assert!(!holds(&output, b"\x1b[31m"), "{}", output.escape_ascii());
}

/// Waits until the terminal cursor shows at the 0-based `row` and `col`.
async fn cursor_at(client: &PtyClient, row: u16, col: u16) {
    client
        .wait_screen(
            &format!("the cursor at row {row}, column {col}"),
            |screen| !screen.hide_cursor() && screen.cursor_position() == (row, col),
        )
        .await;
}

#[tokio::test]
async fn cursor_after_typing() {
    let (_session, mut client) = attached().await;
    client.send(b"hello");
    cursor_at(&client, 23, 5).await;
}

/// A server with two worlds, and a client in a terminal of 20 columns that shows the greeting.
async fn attached_narrow() -> (Session, PtyClient) {
    let session = Session::with_worlds(TWO_WORLDS).await;
    let client = PtyClient::start_sized(
        session.server.socket(),
        session.server.dir().path(),
        Answers::Default,
        (24, 20),
    );
    client.wait_greeting().await;
    (session, client)
}

#[tokio::test]
async fn cursor_on_a_wrapped_row_after_a_wide_grapheme() {
    let (_session, mut client) = attached_narrow().await;
    client.send(format!("{}漢字", "x".repeat(18)).as_bytes());
    cursor_at(&client, 23, 2).await;
}

#[tokio::test]
async fn cursor_at_a_full_row() {
    let (_session, mut client) = attached_narrow().await;
    client.send("x".repeat(20).as_bytes());
    cursor_at(&client, 23, 19).await;
    client
        .wait_rows("an editor of one row", |rows| {
            rows[22].trim_end() != "x".repeat(20)
        })
        .await;
}

#[tokio::test]
async fn editor_cursor_with_probed_widths() {
    let session = Session::with_worlds(TWO_WORLDS).await;
    let mut client = PtyClient::start_with(
        session.server.socket(),
        session.server.dir().path(),
        Answers::ZwjWidth(8),
    );
    client.wait_greeting().await;
    client.send(format!("{FAMILY}x").as_bytes());
    // The emoji takes 8 cells and `x` one, so the cursor is at column 10, counted from 1.
    cursor_at(&client, 23, 9).await;
}

// The kitty keyboard protocol.

const PUSH_FLAG: &[u8] = b"\x1b[>1u";
const POP_FLAG: &[u8] = b"\x1b[<1u";

#[tokio::test]
async fn terminal_with_kitty_keyboard() {
    let session = Session::with_worlds(TWO_WORLDS).await;
    let mut client = PtyClient::start_with(
        session.server.socket(),
        session.server.dir().path(),
        Answers::Kitty,
    );
    client.wait_greeting().await;
    assert!(holds(&client.output(), PUSH_FLAG));
    assert!(!holds(&client.output(), POP_FLAG));
    client.send(b"\x03");
    assert!(client.exit_within(BOUND).await.is_some());
    client.wait_output("the pop of the flag", POP_FLAG).await;
}

#[tokio::test]
async fn terminal_without_kitty_keyboard() {
    let output = output_at_exit(Answers::Default).await;
    assert_eq!(
        (holds(&output, PUSH_FLAG), holds(&output, POP_FLAG)),
        (false, false),
        "{}",
        output.escape_ascii()
    );
}

#[tokio::test]
async fn shift_enter_with_kitty_keyboard() {
    let session = Session::with_worlds(TWO_WORLDS).await;
    let mut client = PtyClient::start_with(
        session.server.socket(),
        session.server.dir().path(),
        Answers::Kitty,
    );
    client.wait_greeting().await;
    // With the flag, the terminal sends Shift+Enter as CSI 13;2u.
    client.send(b"a\x1b[13;2ub");
    client
        .wait_rows("a and b on two rows of the editor", |rows| {
            last_rows(rows, 2) == ["a", "b"]
        })
        .await;
}

#[tokio::test]
async fn late_kitty_keyboard_answer() {
    let session = Session::with_worlds(TWO_WORLDS).await;
    let mut client = PtyClient::start_with(
        session.server.socket(),
        session.server.dir().path(),
        Answers::LateKitty,
    );
    client.wait_greeting().await;
    client.send(b"z");
    client
        .wait_rows("only z in the editor", |rows| last_rows(rows, 1) == ["z"])
        .await;
    client.send(b"\x03");
    assert!(client.exit_within(BOUND).await.is_some());
    client
        .wait_output("the exit from the alternate screen", b"\x1b[?1049l")
        .await;
    assert!(!holds(&client.output(), PUSH_FLAG));
}

// Client commands on the built binary.

/// True if the server of `session` reports `world` as connected.
async fn connected(session: &Session, world: &str) -> bool {
    session
        .client
        .list_worlds()
        .await
        .unwrap()
        .into_iter()
        .find(|each| each.id == world)
        .expect("the world exists")
        .state
        == WorldState::WORLD_STATE_CONNECTED
}

/// The client of [`attached`] after a test disconnected `alpha/aria` through the API.
async fn alpha_disconnected() -> (Session, PtyClient) {
    let (mut session, client) = attached().await;
    session.client.disconnect("alpha/aria").await.unwrap();
    session
        .kind("alpha/aria", EventKind::EVENT_KIND_DISCONNECTED, 0)
        .await;
    (session, client)
}

/// Sends `text` and Enter to `client`, and waits until the server connects `alpha/aria` and
/// the screen shows its greeting twice.
async fn connects_alpha(text: &[u8]) {
    let (mut session, mut client) = alpha_disconnected().await;
    let after = session
        .kind("alpha/aria", EventKind::EVENT_KIND_DISCONNECTED, 0)
        .await
        .seq;
    client.send(text);
    session
        .kind("alpha/aria", EventKind::EVENT_KIND_CONNECTED, after)
        .await;
    client
        .wait_rows("the greeting after the disconnect", |rows| {
            rows.iter()
                .filter(|row| row.trim() == "Welcome, aria.")
                .count()
                == 2
        })
        .await;
}

#[tokio::test]
async fn connect_the_world_in_focus() {
    connects_alpha(b"/connect\r").await;
}

#[tokio::test]
async fn a_trailing_space_is_no_argument() {
    connects_alpha(b"/connect \r").await;
}

#[tokio::test]
async fn disconnect_a_named_world() {
    let (mut session, mut client) = attached().await;
    client.send(b"/disconnect beta/bryn\r");
    session
        .kind("beta/bryn", EventKind::EVENT_KIND_DISCONNECTED, 0)
        .await;
    assert_eq!(
        (
            connected(&session, "beta/bryn").await,
            connected(&session, "alpha/aria").await
        ),
        (false, true)
    );
}

#[tokio::test]
async fn disconnect_an_unknown_world() {
    let (session, mut client) = attached().await;
    client.send(b"/disconnect gamma/gil\r");
    client
        .wait_rows("a hint that names the world", |rows| {
            rows.iter()
                .any(|row| row.starts_with("-- ") && row.contains("gamma/gil"))
        })
        .await;
    assert!(connected(&session, "alpha/aria").await);
}

#[tokio::test]
async fn a_key_and_a_command_give_the_same_focus() {
    for input in [&b"\x0e"[..], b"/focus-next\r"] {
        let (_session, mut client) = attached().await;
        client.send(input);
        client
            .wait_top("the focus on beta/bryn", |top| top.contains("[beta/bryn]"))
            .await;
    }
}

#[tokio::test]
async fn quit_by_command() {
    let (session, mut client) = attached().await;
    client.send(b"/quit\r");
    let status = client.exit_within(BOUND).await.expect("the client exits");
    assert!(status.success(), "{status:?}");
    assert_eq!(session.client.list_worlds().await.unwrap().len(), 2);
}

/// A client on a relay that stalls after the greeting, with the text `hi` of `alpha/aria`
/// waiting for the server.
async fn a_text_waits() -> (ServerProcess, Relay, PtyClient) {
    let server = ServerBuilder::new().fake_worlds(TWO_WORLDS).start().await;
    let relay = Relay::start(&server.dir().path().join("relay.sock"), server.socket());
    let mut client = PtyClient::start(relay.socket(), server.dir().path());
    client.wait_greeting().await;
    relay.stall();
    client.send(b"hi\r");
    client
        .wait_top("the text that waits", |top| top.contains("alpha/aria"))
        .await;
    (server, relay, client)
}

#[tokio::test]
async fn quit_twice_while_a_text_waits() {
    let (_server, _relay, mut client) = a_text_waits().await;
    client.send(b"/quit\r");
    client
        .wait_rows("the quit hint", |rows| {
            rows.iter()
                .any(|row| row.starts_with("-- ") && row.contains("1 line not sent yet"))
        })
        .await;
    client.send(b"/quit\r");
    let status = client.exit_within(BOUND).await.expect("the client exits");
    assert!(status.success(), "{status:?}");
}

/// True if the last hint of `rows` holds `text`.
fn last_hint_holds(rows: &[String], text: &str) -> bool {
    rows.iter()
        .rev()
        .find(|row| row.starts_with("-- "))
        .is_some_and(|row| row.contains(text))
}

/// Sends `/quit`, then `between`, waits until `settled` holds, clears the editor with Ctrl+U,
/// and sends `/quit` again. Then waits for the hint `waiting` after the last Enter: a client
/// that quit would show no new hint.
async fn quit_cancelled_by(between: &[u8], settled: impl Fn(&[String]) -> bool, waiting: &str) {
    let (_server, _relay, mut client) = a_text_waits().await;
    client.send(b"/quit\r");
    client
        .wait_rows("the first quit hint", |rows| {
            last_hint_holds(rows, "1 line not sent yet")
        })
        .await;
    client.send(between);
    client
        .wait_rows("the text between the quits", settled)
        .await;
    client.send(b"\x15/quit\r");
    client
        .wait_rows("the quit hint after the last Enter", |rows| {
            last_hint_holds(rows, waiting)
        })
        .await;
}

#[tokio::test]
async fn another_command_cancels_the_quit() {
    quit_cancelled_by(
        b"/fly\r",
        |rows| last_hint_holds(rows, "unknown command"),
        "1 line not sent yet",
    )
    .await;
}

#[tokio::test]
async fn a_game_text_cancels_the_quit() {
    quit_cancelled_by(
        b"hello\r",
        |rows| rows.last().is_some_and(|row| row.trim().is_empty()),
        "2 lines not sent yet",
    )
    .await;
}

#[tokio::test]
async fn a_good_command_cancels_the_quit() {
    quit_cancelled_by(
        b"/focus-next\r",
        |rows| rows[0].contains("[beta/bryn]"),
        "1 line not sent yet",
    )
    .await;
}

// The client layers of the server.

/// A server with `worlds` over `config`, and a client on it with `--device laptop`, after the
/// greeting of the first world, with `answers` to the probe.
async fn attached_with(
    worlds: &[&str],
    config: &ConfigDir,
    answers: Answers,
) -> (Session, PtyClient) {
    let mut session = Session::start(
        ServerBuilder::new()
            .fake_worlds(worlds)
            .arg("--config")
            .arg(config.path()),
    )
    .await;
    for world in worlds {
        session.welcome(world).await;
    }
    let client = PtyClient::start_args(
        session.server.socket(),
        session.server.dir().path(),
        answers,
        &["--device".as_ref(), "laptop".as_ref()],
        &[],
    );
    if worlds.is_empty() {
        client
            .wait_top("no worlds", |top| top.trim() == "no worlds")
            .await;
    } else {
        client.wait_greeting().await;
    }
    (session, client)
}

/// A server with `worlds` and `clients/global.toml` that holds `text`, and a client on it.
async fn with_layer(worlds: &[&str], text: &str) -> (Session, PtyClient, ConfigDir) {
    let config = ConfigDir::empty().file("clients/global.toml", text);
    let (session, client) = attached_with(worlds, &config, Answers::Default).await;
    (session, client, config)
}

/// Sends `hello` and Enter, and waits for the reply of the fake game.
async fn hello(client: &mut PtyClient) {
    client.send(b"hello\r");
    client
        .wait_rows("the reply", |rows| {
            rows.iter().any(|row| row.contains("You say, \"hello\""))
        })
        .await;
}

/// Whether a row starts with `> `.
fn has_echo(client: &PtyClient) -> bool {
    client.rows().iter().any(|row| row.starts_with("> "))
}

#[tokio::test]
async fn the_default_dims() {
    let (_session, mut client, _config) = with_layer(&["alpha/aria"], "").await;
    hello(&mut client).await;
    assert_eq!(client.dims("> hello"), [true; 7]);
}

#[tokio::test]
async fn show() {
    let (_session, mut client, _config) = with_layer(&["alpha/aria"], "echo = \"show\"\n").await;
    hello(&mut client).await;
    assert_eq!(client.dims("> hello"), [false; 7]);
}

#[tokio::test]
async fn hide() {
    let (_session, mut client, _config) = with_layer(&["alpha/aria"], "echo = \"hide\"\n").await;
    hello(&mut client).await;
    assert!(!has_echo(&client), "{:?}", client.rows());
}

/// Asserts that the layer `path` with `echo = "hide"` applies to `mu client --device laptop`.
async fn hides_with(path: &str) {
    let config = ConfigDir::empty().file(path, "echo = \"hide\"\n");
    let (_session, mut client) = attached_with(&["alpha/aria"], &config, Answers::Default).await;
    hello(&mut client).await;
    assert!(!has_echo(&client), "{:?}", client.rows());
}

#[tokio::test]
async fn the_kind_layer_applies() {
    hides_with("clients/kinds/tui.toml").await;
}

#[tokio::test]
async fn the_device_layer_applies() {
    hides_with("clients/devices/laptop.toml").await;
}

#[tokio::test]
async fn the_os_layer_applies() {
    hides_with(&format!(
        "clients/os/{}.toml",
        mu_config::Os::current().name()
    ))
    .await;
}

#[tokio::test]
async fn a_bad_device_layer_at_attach() {
    let config = ConfigDir::empty()
        .file("clients/global.toml", "echo = \"show\"\n")
        .file("clients/devices/laptop.toml", "echo = \"loud\"\n");
    let (_session, mut client) = attached_with(&["alpha/aria"], &config, Answers::Default).await;
    hello(&mut client).await;
    let screen: String = client.contents().split_whitespace().collect();
    assert!(
        screen.contains("clients/devices/laptop.toml") && screen.contains("echo"),
        "{screen}"
    );
    assert_eq!(client.dims("> hello"), [false; 7]);
}

#[tokio::test]
async fn the_client_takes_the_flag() {
    let dir = private_tempdir();
    let flag = ConfigDir::base();
    let run = dir.path().join("run");
    let output = support::mu_with(
        &["client", "--config", flag.path().to_str().unwrap()],
        &[("XDG_RUNTIME_DIR", Some(run.to_str().unwrap()))],
    );
    assert!(
        output.status.code() == Some(2) && stderr(&output).contains("--config"),
        "{}",
        stderr(&output)
    );
    assert!(!run.exists());
}

#[tokio::test]
async fn the_client_warns_of_a_missing_directory() {
    let dir = private_tempdir();
    let missing = dir.path().join("none");
    let run = dir.path().join("run");
    let log = run.join("server.log");
    let _reaper = Reaper(log.clone());
    let client = PtyClient::start_args(
        &run.join("s.sock"),
        dir.path(),
        Answers::Default,
        &[],
        &[("MU_CONFIG", Some(missing.as_os_str()))],
    );
    client
        .wait_top("no worlds", |top| top.trim() == "no worlds")
        .await;
    let log = std::fs::read_to_string(&log).unwrap();
    assert!(log.contains(&missing.display().to_string()), "{log}");
}

#[tokio::test]
async fn the_started_server_inherits_mu_config() {
    let dir = private_tempdir();
    let conf = ConfigDir::base();
    let run = dir.path().join("run");
    let _reaper = Reaper(run.join("server.log"));
    let client = PtyClient::start_args(
        &run.join("s.sock"),
        dir.path(),
        Answers::Default,
        &[],
        &[("MU_CONFIG", Some(conf.path().as_os_str()))],
    );
    client
        .wait_top("the world", |top| top.contains("alpha/aria"))
        .await;
}

#[tokio::test]
async fn no_notice_about_another_directory() {
    let dir = private_tempdir();
    let b = dir.path().join("b");
    std::fs::create_dir(&b).unwrap();
    let server = ServerBuilder::new()
        .fake_worlds(&["alpha/aria"])
        .arg("--config")
        .arg(&b)
        .start()
        .await;
    let a = dir.path().join("a");
    let client = PtyClient::start_args(
        server.socket(),
        dir.path(),
        Answers::Default,
        &[],
        &[("MU_CONFIG", Some(a.as_os_str()))],
    );
    client
        .wait_top("the world", |top| top.contains("alpha/aria"))
        .await;
    let screen: String = client.contents().split_whitespace().collect();
    let b: String = b.display().to_string().split_whitespace().collect();
    assert!(!screen.contains(&b), "{screen}");
}

/// A socket in a private directory where a test listens, and gives each connection to `serve`.
fn listening(
    serve: fn(tokio::net::UnixStream, &mut Vec<tokio::net::UnixStream>),
) -> (
    tempfile::TempDir,
    std::path::PathBuf,
    tokio::task::JoinHandle<()>,
) {
    let dir = private_tempdir();
    let socket = dir.path().join("s.sock");
    let listener = tokio::net::UnixListener::bind(&socket).unwrap();
    let task = tokio::spawn(async move {
        let mut open = Vec::new();
        while let Ok((stream, _)) = listener.accept().await {
            serve(stream, &mut open);
        }
    });
    (dir, socket, task)
}

#[tokio::test]
async fn a_server_that_does_not_answer() {
    // Each connection stays open with no answer.
    let (dir, socket, task) = listening(|stream, open| open.push(stream));
    let url = format!("{}?connect_wait=0.2", server_url(&socket));
    let mut client = PtyClient::start_url(&url, dir.path(), Answers::Default, &[], &[]);
    let status = client.exit_within(BOUND).await.expect("the client exits");
    assert_eq!(status.exit_code(), 1);
    client.wait_line("mu client: ").await;
    let screen = client.contents().replace('\n', "");
    assert!(screen.contains(&socket.display().to_string()), "{screen}");
    assert!(screen.contains("0.2 s"), "{screen}");
    task.abort();
}

#[tokio::test]
async fn a_started_server_that_does_not_answer_names_the_last_error() {
    let dir = private_tempdir();
    let conf = ConfigDir::base().file("characters/alpha/aria.toml", "port = 0\n");
    let run = dir.path().join("run");
    let _reaper = Reaper(run.join("server.log"));
    let url = format!("{}?connect_wait=1", server_url(&run.join("s.sock")));
    let mut client = PtyClient::start_url(
        &url,
        dir.path(),
        Answers::Default,
        &[],
        &[("MU_CONFIG", Some(conf.path().as_os_str()))],
    );
    let status = client.exit_within(BOUND).await.expect("the client exits");
    assert_eq!(status.exit_code(), 1);
    client.wait_line("mu client: ").await;
    let screen = client.contents().replace('\n', "");
    assert!(
        screen.contains("the last error was") && screen.contains("server.log"),
        "{screen}"
    );
}

#[tokio::test]
async fn a_server_that_closes_the_connection() {
    let (dir, socket, task) = listening(|stream, _| drop(stream));
    let mut client = PtyClient::start(&socket, dir.path());
    let status = client.exit_within(BOUND).await.expect("the client exits");
    assert_eq!(status.exit_code(), 1);
    client.wait_line("mu client: ").await;
    let screen = client.contents().replace('\n', "");
    assert!(screen.contains(&socket.display().to_string()), "{screen}");
    assert!(
        !dir.path().join("server.log").exists(),
        "the client started a server"
    );
    task.abort();
}

#[tokio::test]
async fn the_started_server_finds_the_default_directory() {
    let dir = private_tempdir();
    let xdg = dir.path().join("xdg");
    let run = dir.path().join("run");
    let log = run.join("server.log");
    let _reaper = Reaper(log.clone());
    let mut client = PtyClient::start_args(
        &run.join("s.sock"),
        dir.path(),
        Answers::Default,
        &[],
        &[
            ("MU_CONFIG", None),
            ("XDG_CONFIG_HOME", Some(xdg.as_os_str())),
        ],
    );
    client
        .wait_top("no worlds", |top| top.trim() == "no worlds")
        .await;
    client.send(b"\x03");
    client.exit_within(BOUND).await.expect("the client exits");
    // The server writes any warning before it serves, and the client waited until it served.
    let log = std::fs::read_to_string(&log).unwrap();
    assert!(log.lines().any(|line| line.starts_with(SERVING)), "{log}");
    let xdg = xdg.display().to_string();
    assert!(!log.lines().any(|line| line.contains(&xdg)), "{log}");
}

// The keys table of the client layers.

const THREE_WORLDS: &[&str] = &["alpha/aria", "beta/bryn", "gamma/gil"];

/// Presses `key` on a client with `[keys]` holding `bindings`, and waits until the top row shows
/// `[<world>]`.
async fn key_gives_focus(worlds: &[&str], bindings: &str, key: &[u8], world: &str) {
    let text = format!("[keys]\n{bindings}\n");
    let (_session, mut client, _config) = with_layer(worlds, &text).await;
    client.send(key);
    let focus = format!("[{world}]");
    client
        .wait_top("the focus", |top| top.contains(&focus))
        .await;
}

#[tokio::test]
async fn a_new_binding() {
    key_gives_focus(
        TWO_WORLDS,
        "\"alt+n\" = \"focus-next\"",
        b"\x1bn",
        "beta/bryn",
    )
    .await;
}

#[tokio::test]
async fn the_default_keys_stay() {
    key_gives_focus(
        TWO_WORLDS,
        "\"alt+n\" = \"focus-next\"",
        b"\x0e",
        "beta/bryn",
    )
    .await;
}

#[tokio::test]
async fn a_binding_of_shift_tab() {
    key_gives_focus(
        THREE_WORLDS,
        "\"shift+tab\" = \"focus-prev\"",
        b"\x1b[Z",
        "gamma/gil",
    )
    .await;
}

#[tokio::test]
async fn a_binding_replaces_a_default() {
    key_gives_focus(
        THREE_WORLDS,
        "\"ctrl+n\" = \"focus-prev\"",
        b"\x0e",
        "gamma/gil",
    )
    .await;
}

/// Waits until the editor row, the last row of the screen, is `text`.
async fn editor_holds(client: &PtyClient, text: &str) {
    client
        .wait_rows("the editor", |rows| {
            rows.last().is_some_and(|row| row.trim_end() == text)
        })
        .await;
}

#[tokio::test]
async fn a_removed_binding() {
    let (_session, mut client, _config) =
        with_layer(TWO_WORLDS, "[keys]\n\"ctrl+n\" = \"none\"\n").await;
    client.send(b"\x0ex");
    // The `x` after Ctrl+N shows that the client read the key.
    editor_holds(&client, "x").await;
    assert!(
        client.rows()[0].contains("[alpha/aria]"),
        "{:?}",
        client.rows()
    );
}

#[tokio::test]
async fn none_on_a_key_with_a_modifier() {
    let (mut session, mut client, _config) =
        with_layer(&["alpha/aria"], "[keys]\n\"alt+enter\" = \"none\"\n").await;
    client.send(b"hi\x1b\r!");
    editor_holds(&client, "hi!").await;
    client.send(b"\r");
    let event = session
        .until("the player event", |event| {
            event.kind.as_known() == Some(EventKind::EVENT_KIND_INPUT)
        })
        .await;
    assert_eq!(event.text, "hi!");
}

#[tokio::test]
async fn a_binding_of_ctrl_shift_z() {
    let config = ConfigDir::empty().file(
        "clients/global.toml",
        "[keys]\n\"ctrl+shift+z\" = \"none\"\n",
    );
    let (_session, mut client) = attached_with(&["alpha/aria"], &config, Answers::Kitty).await;
    client.send(b"h");
    editor_holds(&client, "h").await;
    // Ctrl+Z, then Ctrl+Shift+Z in the kitty protocol, then `x`. The default redo would bring
    // `h` back before the `x`.
    client.send(b"\x1a\x1b[122;6ux");
    editor_holds(&client, "x").await;
}

#[tokio::test]
async fn a_key_bound_to_connect_with_no_world_in_focus() {
    let config =
        ConfigDir::empty().file("clients/global.toml", "[keys]\n\"alt+c\" = \"connect\"\n");
    let (_session, mut client) = attached_with(&[], &config, Answers::Default).await;
    client.send(b"\x1bc");
    client
        .wait_rows("the hint", |rows| {
            rows.iter()
                .any(|row| row.starts_with("-- ") && row.contains("no world"))
        })
        .await;
    // The client still reads keys.
    client.send(b"x");
    editor_holds(&client, "x").await;
}

// A live change of the client layers.

/// Writes `text` to `path` under `config`.
fn write_layer(config: &ConfigDir, path: &str, text: &str) {
    let path = config.path().join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// Runs `mu config reload` on the server of `socket`.
async fn reload(socket: &std::path::Path) {
    support::mu_async(&["config", "reload", "--server", &server_url(socket)], &[]).await;
}

/// Waits until the screen, with no whitespace, holds each of `texts`.
async fn wait_notice(client: &PtyClient, texts: &[&str]) {
    client
        .wait_rows("the notice", |rows| {
            let screen: String = rows.concat().split_whitespace().collect();
            texts.iter().all(|text| screen.contains(text))
        })
        .await;
}

/// Writes the binding Alt+N to `focus-next` for `laptop`, and a bad `clients/global.toml`. The
/// notice of the bad file shows when the client applied the configuration with the binding.
fn bind_alt_n(config: &ConfigDir) {
    write_layer(
        config,
        "clients/devices/laptop.toml",
        "[keys]\n\"alt+n\" = \"focus-next\"\n",
    );
    write_layer(config, "clients/global.toml", "colour = \"red\"\n");
}

/// Waits for the notice of [`bind_alt_n`], presses Alt+N and waits for the focus of
/// `beta/bryn`.
async fn alt_n_moves_the_focus(client: &mut PtyClient) {
    wait_notice(client, &["clients/global.toml", "colour"]).await;
    client.send(b"\x1bn");
    client
        .wait_top("the focus", |top| top.contains("[beta/bryn]"))
        .await;
}

#[tokio::test]
async fn a_new_binding_works_without_a_restart() {
    let config = ConfigDir::empty();
    let (session, mut client) = attached_with(TWO_WORLDS, &config, Answers::Default).await;
    bind_alt_n(&config);
    reload(session.server.socket()).await;
    alt_n_moves_the_focus(&mut client).await;
}

#[tokio::test]
async fn a_reload_while_the_stream_is_down() {
    let config = ConfigDir::empty();
    let server = ServerBuilder::new()
        .fake_worlds(TWO_WORLDS)
        .arg("--config")
        .arg(config.path())
        .start()
        .await;
    let relay = Relay::start(&server.dir().path().join("relay.sock"), server.socket());
    let mut client = PtyClient::start_args(
        relay.socket(),
        server.dir().path(),
        Answers::Default,
        &["--device".as_ref(), "laptop".as_ref()],
        &[],
    );
    client.wait_greeting().await;
    relay.hold();
    assert!(relay.cut() >= 1, "no connection to cut");
    client
        .wait_top("disconnected", |top| top.contains("disconnected"))
        .await;
    bind_alt_n(&config);
    reload(server.socket()).await;
    relay.release();
    client
        .wait_top("connected", |top| !top.contains("disconnected"))
        .await;
    alt_n_moves_the_focus(&mut client).await;
}

#[tokio::test]
async fn a_reload_across_a_server_restart() {
    let config = ConfigDir::empty();
    let (mut session, mut client) = attached_with(TWO_WORLDS, &config, Answers::Default).await;
    session.server.signal(rustix::process::Signal::TERM);
    session.server.wait_exit().await;
    bind_alt_n(&config);
    let dir = std::sync::Arc::clone(session.server.dir());
    let _server = ServerBuilder::new()
        .in_dir(&dir)
        .fake_worlds(TWO_WORLDS)
        .arg("--config")
        .arg(config.path())
        .start()
        .await;
    client
        .wait_top("connected", |top| !top.contains("disconnected"))
        .await;
    alt_n_moves_the_focus(&mut client).await;
}

#[tokio::test]
async fn a_bad_layer_after_a_reload() {
    let config = ConfigDir::empty().file("clients/devices/laptop.toml", "echo = \"hide\"\n");
    let (session, mut client) = attached_with(&["alpha/aria"], &config, Answers::Default).await;
    write_layer(&config, "clients/devices/laptop.toml", "echo = \"loud\"\n");
    reload(session.server.socket()).await;
    wait_notice(&client, &["clients/devices/laptop.toml", "echo"]).await;
    hello(&mut client).await;
    assert_eq!(client.dims("> hello"), [true; 7]);
}

#[tokio::test]
async fn a_clients_directory_that_turns_unlistable() {
    use std::os::unix::fs::PermissionsExt as _;
    let config = ConfigDir::empty().file("clients/global.toml", "");
    let clients = config.path().join("clients");
    let (session, client) = attached_with(&["alpha/aria"], &config, Answers::Default).await;
    std::fs::set_permissions(&clients, std::fs::Permissions::from_mode(0o000)).unwrap();
    if std::fs::read_dir(&clients).is_ok() {
        std::fs::set_permissions(&clients, std::fs::Permissions::from_mode(0o755)).unwrap();
        panic!(
            "the mode 000 does not keep this user out: run the tests as a user that is not root"
        );
    }
    reload(session.server.socket()).await;
    client
        .wait_rows("the notice", |rows| {
            rows.iter()
                .any(|row| row.starts_with("-- ") && row.contains("clients"))
        })
        .await;
    std::fs::set_permissions(&clients, std::fs::Permissions::from_mode(0o755)).unwrap();
}

// The layout of the screen (client-layout).

/// The content of a layout file with `lines`.
fn layout_file(lines: &[&str]) -> String {
    let quoted: Vec<String> = lines.iter().map(|line| format!("{line:?}")).collect();
    format!("place = [{}]\n", quoted.join(", "))
}

/// A configuration directory whose `clients/global.toml` names the layout `mine` of `lines`, and
/// then holds `rest`.
fn layout_dir(lines: &[&str], rest: &str) -> ConfigDir {
    ConfigDir::empty()
        .file("clients/layouts/mine.toml", &layout_file(lines))
        .file("clients/global.toml", &format!("layout = \"mine\"\n{rest}"))
}

/// A server with `worlds` over `config`, and a client on it. A row of the client holds the
/// greeting of the first world, beside the text of other panes.
async fn attached_showing(worlds: &[&str], config: &ConfigDir) -> (Session, PtyClient) {
    let mut session = Session::start(
        ServerBuilder::new()
            .fake_worlds(worlds)
            .arg("--config")
            .arg(config.path()),
    )
    .await;
    for world in worlds {
        session.welcome(world).await;
    }
    let client = PtyClient::start_args(
        session.server.socket(),
        session.server.dir().path(),
        Answers::Default,
        &["--device".as_ref(), "laptop".as_ref()],
        &[],
    );
    let first = greeting(worlds[0]);
    client
        .wait_rows("the greeting", |rows| {
            rows.iter().any(|row| row.contains(&first))
        })
        .await;
    (session, client)
}

/// A server with `worlds` and the layout `lines`, and a client on it that shows the greeting.
async fn with_layout(worlds: &[&str], lines: &[&str]) -> (Session, PtyClient, ConfigDir) {
    let config = layout_dir(lines, "");
    let (session, client) = attached_showing(worlds, &config).await;
    (session, client, config)
}

/// Waits until row `number`, from 1, passes `check`.
async fn wait_row(client: &PtyClient, number: usize, what: &str, check: impl Fn(&str) -> bool) {
    client
        .wait_rows(what, |rows| {
            rows.get(number - 1).is_some_and(|row| check(row))
        })
        .await;
}

/// The column of the first `needle` in row `number`, both from 1.
fn column_of(client: &PtyClient, number: usize, needle: &str) -> Option<usize> {
    let rows = client.rows();
    let row = rows.get(number - 1)?;
    let at = row.find(needle)?;
    Some(row[..at].chars().count() + 1)
}

#[tokio::test]
async fn no_tabs() {
    let (mut session, mut client, _config) =
        with_layout(TWO_WORLDS, &["output", "editor below output"]).await;
    client.send(b"\x0e");
    client
        .wait_rows("the greeting of beta/bryn", |rows| {
            rows.iter().any(|row| row.trim() == greeting("beta/bryn"))
        })
        .await;
    client.send(b"hi\r");
    let event = session
        .until("the player event", |event| {
            event.kind == EventKind::EVENT_KIND_INPUT
        })
        .await;
    assert_eq!(
        (event.world.as_str(), event.text.as_str()),
        ("beta/bryn", "hi")
    );
    assert!(
        client.rows().iter().all(|row| !row.contains("alpha/aria")),
        "{}",
        client.contents()
    );
}

#[tokio::test]
async fn the_editor_alone() {
    // The client starts with the default layout, so the greeting shows when it knows its world.
    let (mut session, mut client, config) = with_layer(&["alpha/aria"], "").await;
    let config = config
        .file("clients/layouts/mine.toml", &layout_file(&["editor"]))
        .file("clients/global.toml", "layout = \"mine\"\n");
    let output = mu_async(
        &[
            "config",
            "reload",
            "--server",
            &server_url(session.server.socket()),
        ],
        &[],
    )
    .await;
    assert!(output.status.success(), "{}", stderr(&output));
    client
        .wait_rows("a screen with the editor alone", |rows| {
            rows.iter().all(|row| row.trim().is_empty())
        })
        .await;
    client.send(b"hi\r");
    let event = session
        .until("the player event", |event| {
            event.kind == EventKind::EVENT_KIND_INPUT
        })
        .await;
    assert_eq!(
        (event.world.as_str(), event.text.as_str()),
        ("alpha/aria", "hi")
    );
    drop(config);
}

#[tokio::test]
async fn the_output_above_the_tabs() {
    let (_session, client, _config) = with_layout(
        &["alpha/aria"],
        &["output", "tabs below output", "editor below tabs"],
    )
    .await;
    wait_row(&client, 23, "the tabs", |row| {
        row.starts_with("[alpha/aria]")
    })
    .await;
    let rows = client.rows();
    assert!(
        rows[..22]
            .iter()
            .any(|row| row.trim() == greeting("alpha/aria")),
        "{}",
        client.contents()
    );
}

#[tokio::test]
async fn a_block_beside_another() {
    let (_session, client, _config) = with_layout(
        &["alpha/aria"],
        &[
            "output",
            "tabs above output",
            "editor below output",
            "output as notices right of output",
        ],
    )
    .await;
    wait_row(&client, 1, "the tabs", |row| {
        row.starts_with("[alpha/aria]")
    })
    .await;
    // The divider is in column 41, and the tabs row has none.
    assert_eq!(column_of(&client, 1, "│"), None);
    assert_eq!(column_of(&client, 3, "│"), Some(41));
    let greeting_row = (2..=23)
        .find(|row| column_of(&client, *row, &greeting("alpha/aria")).is_some())
        .expect("the greeting shows");
    assert_eq!(
        column_of(&client, greeting_row, &greeting("alpha/aria")),
        Some(1)
    );
}

#[tokio::test]
async fn the_statusline_at_the_bottom() {
    let (_session, client, _config) = with_layout(
        &["alpha/aria"],
        &[
            "output",
            "editor below output",
            "output as notices right of output",
            "statusline at bottom",
        ],
    )
    .await;
    wait_row(&client, 24, "the statusline", |row| {
        row.starts_with("alpha/aria | ")
    })
    .await;
    assert_eq!(column_of(&client, 24, "│"), None);
}

#[tokio::test]
async fn notices_behind_the_output() {
    let (_session, client, _config) = with_layout(
        &["alpha/aria"],
        &[
            "output",
            "editor below output",
            "output as notices with output",
        ],
    )
    .await;
    wait_row(&client, 1, "the title row", |row| {
        row.trim() == "[output] notices"
    })
    .await;
    assert!(
        client
            .rows()
            .iter()
            .any(|row| row.trim() == greeting("alpha/aria")),
        "{}",
        client.contents()
    );
}

#[tokio::test]
async fn an_id_in_a_later_line() {
    let (_session, client, _config) = with_layout(
        &["alpha/aria"],
        &[
            "output",
            "editor below output",
            "output as side right of output",
            "tabs above side",
        ],
    )
    .await;
    wait_row(&client, 1, "the tabs beside the title", |row| {
        row.starts_with("[output]")
    })
    .await;
    assert_eq!(column_of(&client, 1, "[alpha/aria]"), Some(42));
}

#[tokio::test]
async fn a_percent() {
    let (_session, client, _config) = with_layout(
        &["alpha/aria"],
        &[
            "output",
            "editor below output",
            "output as notices right of output, 25%",
        ],
    )
    .await;
    assert_eq!(column_of(&client, 2, "│"), Some(60));
}

#[tokio::test]
async fn a_count_of_cells() {
    let (_session, client, _config) = with_layout(
        &["alpha/aria"],
        &[
            "output",
            "editor below output",
            "output as notices below output, 5",
        ],
    )
    .await;
    // The pane of the notices takes the 5 rows above the editor, and its first row is its title.
    wait_row(&client, 19, "the title of the notices", |row| {
        row.trim() == "[notices]"
    })
    .await;
    assert_eq!(client.rows()[0].trim(), "[output]");
}

/// Types `count` lines with Ctrl+J between them.
fn type_lines(client: &mut PtyClient, count: usize) {
    for number in 1..=count {
        if number > 1 {
            client.send(b"\x0a");
        }
        client.send(format!("line{number}").as_bytes());
    }
}

#[tokio::test]
async fn the_editor_stops_at_its_max() {
    let (_session, mut client, _config) =
        with_layout(&["alpha/aria"], &["output", "editor below output, max 3"]).await;
    type_lines(&mut client, 5);
    wait_row(&client, 24, "the last line", |row| row.trim() == "line5").await;
    let rows = client.rows();
    assert_eq!(
        [rows[21].trim(), rows[22].trim(), rows[23].trim()],
        ["line3", "line4", "line5"]
    );
    assert!(!rows[20].contains("line"), "{}", client.contents());
}

#[tokio::test]
async fn the_sidebar_layout() {
    let config = ConfigDir::empty().file("clients/global.toml", "layout = \"sidebar\"\n");
    let (_session, client) = attached_showing(&["alpha/aria"], &config).await;
    wait_row(&client, 24, "the statusline", |row| {
        row.starts_with("alpha/aria | ")
    })
    .await;
    assert!(
        client.rows()[0].starts_with("[alpha/aria]"),
        "{}",
        client.contents()
    );
    assert_eq!(column_of(&client, 2, "│"), Some(56));
}

#[tokio::test]
async fn a_reload_changes_the_layout() {
    let (session, mut client, config) = with_layer(&["alpha/aria"], "").await;
    client.send(b"draft");
    wait_row(&client, 24, "the typed text", |row| row.trim() == "draft").await;
    let config = config.file("clients/global.toml", "layout = \"status\"\n");
    let output = mu_async(
        &[
            "config",
            "reload",
            "--server",
            &server_url(session.server.socket()),
        ],
        &[],
    )
    .await;
    assert!(output.status.success(), "{}", stderr(&output));
    wait_row(&client, 24, "the statusline", |row| {
        row.starts_with("alpha/aria | ")
    })
    .await;
    assert_eq!(client.rows()[22].trim(), "draft");
    drop(config);
}

#[tokio::test]
async fn the_default_layout_puts_the_tabs_first() {
    let (_session, client) =
        attached_with(&["alpha/aria"], &ConfigDir::empty(), Answers::Default).await;
    assert!(
        client.rows()[0].starts_with("[alpha/aria]"),
        "{}",
        client.contents()
    );
}

#[tokio::test]
async fn a_label_with_the_number() {
    let (_session, client, _config) =
        with_layer(TWO_WORLDS, "[tabs]\nlabel = \"{number}:{character}\"\n").await;
    client
        // The world with no focus can have the mark of unseen lines.
        .wait_top("the labels", |top| top.starts_with("[1:aria] 2:bryn"))
        .await;
}

#[tokio::test]
async fn the_game_and_the_character() {
    let (_session, client, _config) =
        with_layer(&["alpha/aria"], "[tabs]\nlabel = \"{character}@{game}\"\n").await;
    client
        .wait_top("the label", |top| top.trim() == "[aria@alpha]")
        .await;
}

#[tokio::test]
async fn braces() {
    let (_session, client, _config) =
        with_layer(&["alpha/aria"], "[tabs]\nlabel = \"{{{character}}}\"\n").await;
    client
        .wait_top("the label", |top| top.trim() == "[{aria}]")
        .await;
}

/// Whether `text` is a time from `0:00` to `0:09`.
fn is_a_young_time(text: &str) -> bool {
    text.strip_prefix("0:0")
        .is_some_and(|rest| rest.len() == 1 && rest.bytes().all(|byte| byte.is_ascii_digit()))
}

#[tokio::test]
async fn the_default_segments() {
    let (_session, client, _config) = with_layer(&["alpha/aria"], "layout = \"status\"\n").await;
    wait_row(&client, 24, "the statusline", |row| {
        row.starts_with("alpha/aria | connected")
    })
    .await;
}

#[tokio::test]
async fn the_state_after_a_disconnect() {
    let (_session, mut client, _config) =
        with_layer(&["alpha/aria"], "layout = \"status\"\n").await;
    wait_row(&client, 24, "the connected state", |row| {
        row.starts_with("alpha/aria | connected")
    })
    .await;
    client.send(b"/disconnect\r");
    wait_row(&client, 24, "the disconnected state", |row| {
        row.starts_with("alpha/aria | disconnected")
    })
    .await;
}

#[tokio::test]
async fn the_idle_time_after_a_send() {
    let (_session, mut client, _config) = with_layer(
        &["alpha/aria"],
        "layout = \"status\"\n[statusline]\nleft = []\nright = [\"idle\"]\n",
    )
    .await;
    client.send(b"hi\r");
    wait_row(&client, 24, "the idle time", |row| {
        row.split_whitespace().last().is_some_and(is_a_young_time)
    })
    .await;
}

#[tokio::test]
async fn the_connected_time() {
    let (_session, client, _config) = with_layer(
        &["alpha/aria"],
        "layout = \"status\"\n[statusline]\nleft = [\"connected\"]\nright = []\n",
    )
    .await;
    wait_row(&client, 24, "the connected time", |row| {
        is_a_young_time(row.trim())
    })
    .await;
}

// The layout of the world in focus.

/// A server with the two worlds and `[world-layouts]` with `entries`, and a client on it.
async fn with_world_layouts(entries: &str) -> (Session, PtyClient, ConfigDir) {
    with_layer(TWO_WORLDS, &format!("[world-layouts]\n{entries}")).await
}

#[tokio::test]
async fn a_world_with_its_own_layout() {
    let (_session, client, _config) = with_world_layouts("\"alpha/aria\" = \"status\"\n").await;
    wait_row(&client, 24, "the statusline", |row| {
        row.starts_with("alpha/aria | ")
    })
    .await;
    assert!(
        client.rows()[0].starts_with("[alpha/aria]"),
        "{}",
        client.contents()
    );
}

#[tokio::test]
async fn the_focus_moves_to_a_world_with_no_entry() {
    let (_session, mut client, _config) = with_world_layouts("\"alpha/aria\" = \"status\"\n").await;
    wait_row(&client, 24, "the statusline", |row| {
        row.starts_with("alpha/aria | ")
    })
    .await;
    client.send(b"\x0e");
    client
        .wait_top("the focus on beta/bryn", |top| top.contains("[beta/bryn]"))
        .await;
    client.send(b"x");
    wait_row(&client, 24, "the editor on the bottom row", |row| {
        row.trim() == "x"
    })
    .await;
}

#[tokio::test]
async fn a_layout_for_a_game() {
    let (_session, client, _config) = with_world_layouts("alpha = \"focus\"\n").await;
    assert!(
        client
            .rows()
            .iter()
            .all(|row| !row.contains("[alpha/aria]")),
        "{}",
        client.contents()
    );
}

#[tokio::test]
async fn the_world_entry_comes_before_the_game_entry() {
    let (_session, client, _config) =
        with_world_layouts("alpha = \"focus\"\n\"alpha/aria\" = \"classic\"\n").await;
    client
        .wait_top("the tabs", |top| top.starts_with("[alpha/aria]"))
        .await;
}

#[tokio::test]
async fn no_world_in_focus_takes_the_default_layout() {
    let (_session, client, _config) = with_layer(
        &[],
        "layout = \"status\"\n[world-layouts]\nalpha = \"focus\"\n",
    )
    .await;
    wait_row(&client, 24, "the statusline", |row| {
        row.trim() == "no world"
    })
    .await;
}

#[tokio::test]
async fn a_world_keeps_its_text_across_a_change_of_layout() {
    let (_session, mut client, _config) = with_world_layouts("\"alpha/aria\" = \"status\"\n").await;
    client.send(b"draft");
    wait_row(&client, 23, "the typed text", |row| row.trim() == "draft").await;
    client.send(b"\x0e");
    client
        .wait_top("the focus on beta/bryn", |top| top.contains("[beta/bryn]"))
        .await;
    client.send(b"\x10");
    wait_row(&client, 23, "the text of alpha/aria", |row| {
        row.trim() == "draft"
    })
    .await;
    wait_row(&client, 24, "the statusline", |row| {
        row.starts_with("alpha/aria | ")
    })
    .await;
}

#[tokio::test]
async fn the_input_focus_after_a_change_of_the_world() {
    let config = ConfigDir::empty()
        .file(
            "clients/layouts/mine.toml",
            &layout_file(&[
                "output",
                "editor below output",
                "editor as scratch right of editor",
            ]),
        )
        .file(
            "clients/global.toml",
            "[world-layouts]\n\"alpha/aria\" = \"mine\"\n",
        );
    let (_session, mut client) = attached_with(TWO_WORLDS, &config, Answers::Default).await;
    // Alt+E gives the input focus to `scratch`, which the layout of beta/bryn does not hold.
    client.send(b"\x1be");
    client.send(b"\x0e");
    client
        .wait_top("the focus on beta/bryn", |top| top.contains("[beta/bryn]"))
        .await;
    client.send(b"x");
    wait_row(&client, 24, "the editor of beta/bryn", |row| {
        row.trim() == "x"
    })
    .await;
}

// The saves of the layout from the client (layout-editing).

use support::process::wait_until;

/// The output of `mu config show --device laptop` on the server of `session`.
async fn config_of(session: &Session) -> String {
    let output = mu_async(
        &[
            "config",
            "show",
            "--server",
            &server_url(session.server.socket()),
            "--device",
            "laptop",
        ],
        &[],
    )
    .await;
    support::stdout(&output)
}

/// Types `keys` into the client, then waits until the configuration of `laptop` passes
/// `check`. It reads the configuration again after each new store hash of the server.
async fn after_save(
    session: &Session,
    client: &mut PtyClient,
    keys: &[u8],
    what: &str,
    check: impl Fn(&str) -> bool,
) -> String {
    let mut hashes = session
        .server
        .client()
        .watch_config()
        .await
        .expect("the watch of the configuration");
    hashes.next().await.expect("the current store hash");
    client.send(keys);
    wait_until(BOUND, what, async {
        loop {
            let shown = config_of(session).await;
            if check(&shown) {
                return shown;
            }
            hashes.next().await.expect("the next store hash");
        }
    })
    .await
}

fn has_config_line(shown: &str, line: &str) -> bool {
    shown.lines().any(|each| each == line)
}

/// The lines of the layout file `name` under `dir`.
fn place_of(dir: &std::path::Path, name: &str) -> Vec<String> {
    let text = std::fs::read_to_string(dir.join(format!("clients/layouts/{name}.toml")))
        .unwrap_or_else(|error| panic!("the layout file {name}: {error}"));
    let file: toml::Table = text.parse().expect("the layout file is TOML");
    file["place"]
        .as_array()
        .expect("place is a list")
        .iter()
        .map(|line| line.as_str().expect("a line is text").to_owned())
        .collect()
}

#[tokio::test]
async fn a_save_from_a_built_in_layout() {
    let (session, mut client, config) = with_layer(&["alpha/aria"], "").await;
    let shown = after_save(
        &session,
        &mut client,
        b"/pane hide tabs\r",
        "the layout of the device",
        |shown| has_config_line(shown, "layout = \"laptop\" # clients/devices/laptop.toml"),
    )
    .await;
    assert!(shown.contains("#   1. output"), "{shown}");
    assert_eq!(
        place_of(config.path(), "laptop"),
        ["output", "editor below output"]
    );
}

#[tokio::test]
async fn a_save_to_a_player_layout() {
    let config = layout_dir(&["output", "editor below output"], "");
    let (session, mut client) = attached_showing(&["alpha/aria"], &config).await;
    after_save(
        &session,
        &mut client,
        b"/place tabs at top\r",
        "the third line of the layout",
        |shown| shown.contains("#   3. tabs at top"),
    )
    .await;
    assert_eq!(
        place_of(config.path(), "mine"),
        ["output", "editor below output", "tabs at top"]
    );
    assert!(!config.path().join("clients/devices/laptop.toml").exists());
}

#[tokio::test]
async fn a_second_client_gets_the_change() {
    let config = layout_dir(&["output", "tabs above output", "editor below output"], "");
    let (session, mut first) = attached_showing(TWO_WORLDS, &config).await;
    let second = PtyClient::start_args(
        session.server.socket(),
        session.server.dir().path(),
        Answers::Default,
        &["--device".as_ref(), "desk".as_ref()],
        &[],
    );
    second
        .wait_top("the tabs of the second client", |top| {
            top.starts_with("[alpha/aria]")
        })
        .await;
    first.send(b"/pane hide tabs\r");
    second
        .wait_top("no tabs in the second client", |top| {
            !top.contains("[alpha/aria]")
        })
        .await;
}

#[tokio::test]
async fn the_server_refuses() {
    let config = ConfigDir::empty().file("clients/devices/laptop.toml", "echo = ");
    let (_session, mut client) = attached_showing(&["alpha/aria"], &config).await;
    client.send(b"/pane hide tabs\r");
    client
        .wait_rows("the notice of the refused save", |rows| {
            rows.iter().any(|row| row.starts_with("-- "))
                && rows
                    .iter()
                    .any(|row| row.contains("clients/devices/laptop.toml"))
                && !rows[0].starts_with("[alpha/aria]")
        })
        .await;
}

#[tokio::test]
async fn a_save_from_the_built_in_layout_of_one_world() {
    let (session, mut client, config) = with_world_layouts("\"alpha/aria\" = \"classic\"\n").await;
    let shown = after_save(
        &session,
        &mut client,
        b"/pane hide tabs\r",
        "the entry of the world",
        |shown| {
            has_config_line(
                shown,
                "world-layouts.\"alpha/aria\" = \"laptop.alpha.aria\" # clients/devices/laptop.toml",
            )
        },
    )
    .await;
    assert!(
        has_config_line(&shown, "layout = \"classic\" # default"),
        "{shown}"
    );
    assert_eq!(
        place_of(config.path(), "laptop.alpha.aria"),
        ["output", "editor below output"]
    );
}

#[tokio::test]
async fn the_other_world_keeps_its_layout() {
    let (session, mut client, _config) = with_world_layouts("\"alpha/aria\" = \"classic\"\n").await;
    after_save(
        &session,
        &mut client,
        b"/pane hide tabs\r",
        "the entry of the world",
        |shown| shown.contains("laptop.alpha.aria"),
    )
    .await;
    client.send(b"\x0e");
    client
        .wait_top("the tabs of beta/bryn", |top| top.contains("[beta/bryn]"))
        .await;
}

/// A server with `worlds` over `config`, and a client with the device name `device` on it.
async fn attached_as(worlds: &[&str], config: &ConfigDir, device: &str) -> (Session, PtyClient) {
    let mut session = Session::start(
        ServerBuilder::new()
            .fake_worlds(worlds)
            .arg("--config")
            .arg(config.path()),
    )
    .await;
    for world in worlds {
        session.welcome(world).await;
    }
    let client = PtyClient::start_args(
        session.server.socket(),
        session.server.dir().path(),
        Answers::Default,
        &["--device".as_ref(), device.as_ref()],
        &[],
    );
    client.wait_greeting().await;
    (session, client)
}

/// The rows as one text, with a space between two rows. A long notice wraps over several rows.
fn joined(rows: &[String]) -> String {
    rows.iter()
        .map(|row| row.trim_end())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Runs `/pane hide tabs` and waits for the notice that names `/layout save`. The screen shows
/// no tabs, and the server holds no layout file.
async fn not_saved(config: &ConfigDir, device: &str) {
    let (_session, mut client) = attached_as(&["alpha/aria"], config, device).await;
    client.send(b"/pane hide tabs\r");
    client
        .wait_rows("the notice of a layout that is not saved", |rows| {
            joined(rows).contains("/layout save") && !rows[0].starts_with("[alpha/aria]")
        })
        .await;
    assert!(!config.path().join("clients/layouts").exists());
}

#[tokio::test]
async fn a_name_that_is_too_long() {
    let config = ConfigDir::empty().file(
        "clients/global.toml",
        "[world-layouts]\n\"alpha/aria\" = \"classic\"\n",
    );
    not_saved(&config, &"a".repeat(60)).await;
}

#[tokio::test]
async fn a_device_with_a_built_in_name() {
    not_saved(&ConfigDir::empty(), "focus").await;
}

#[tokio::test]
async fn use_a_built_in_layout() {
    let (session, mut client, _config) = with_layer(&["alpha/aria"], "").await;
    let shown = after_save(
        &session,
        &mut client,
        b"/layout use status\r",
        "the layout of the device",
        |shown| shown.contains("layout = \"status\""),
    )
    .await;
    assert!(
        has_config_line(&shown, "layout = \"status\" # clients/devices/laptop.toml"),
        "{shown}"
    );
    wait_row(&client, 24, "the statusline", |row| {
        row.starts_with("alpha/aria | ")
    })
    .await;
}

#[tokio::test]
async fn save_under_a_name() {
    let (session, mut client, config) = with_layer(&["alpha/aria"], "").await;
    after_save(
        &session,
        &mut client,
        b"/layout save evening\r",
        "the layout of the device",
        |shown| shown.contains("layout = \"evening\""),
    )
    .await;
    assert_eq!(
        place_of(config.path(), "evening"),
        ["output", "tabs above output", "editor below output"]
    );
}

#[tokio::test]
async fn a_save_over_a_built_in_name() {
    let (_session, mut client, config) = with_layer(&["alpha/aria"], "").await;
    client.send(b"/layout save focus\r");
    client
        .wait_rows("the notice of the refused name", |rows| {
            rows.iter()
                .any(|row| row.starts_with("-- ") && row.contains("focus"))
        })
        .await;
    assert!(!config.path().join("clients/layouts/focus.toml").exists());
}

#[tokio::test]
async fn a_layout_for_this_world() {
    let (session, mut client, _config) = with_layer(TWO_WORLDS, "").await;
    let shown = after_save(
        &session,
        &mut client,
        b"/layout here focus\r",
        "the entry of the world",
        |shown| shown.contains("world-layouts"),
    )
    .await;
    assert!(
        has_config_line(
            &shown,
            "world-layouts.\"alpha/aria\" = \"focus\" # clients/devices/laptop.toml"
        ),
        "{shown}"
    );
    client
        .wait_top("no tabs", |top| !top.contains("[alpha/aria]"))
        .await;
}

#[tokio::test]
async fn remove_the_layout_of_this_world() {
    let (session, mut client, _config) = with_layer(TWO_WORLDS, "").await;
    after_save(
        &session,
        &mut client,
        b"/layout here focus\r",
        "the entry of the world",
        |shown| shown.contains("world-layouts"),
    )
    .await;
    client
        .wait_top("no tabs", |top| !top.contains("[alpha/aria]"))
        .await;
    after_save(
        &session,
        &mut client,
        b"/layout here\r",
        "no entry of the world",
        |shown| !shown.contains("world-layouts"),
    )
    .await;
    client
        .wait_top("the tabs", |top| top.starts_with("[alpha/aria]"))
        .await;
}

#[tokio::test]
async fn no_world_in_focus_for_layout_here() {
    let (_session, mut client, config) = with_layer(&[], "").await;
    client.send(b"/layout here focus\r");
    client
        .wait_rows("the notice", |rows| {
            rows.iter()
                .any(|row| row.starts_with("-- ") && row.contains("world"))
        })
        .await;
    assert!(!config.path().join("clients/devices/laptop.toml").exists());
}

#[tokio::test]
async fn an_unknown_layout_for_this_world() {
    let (_session, mut client, config) = with_layer(&["alpha/aria"], "").await;
    client.send(b"/layout here gone\r");
    client
        .wait_rows("the notice", |rows| {
            rows.iter()
                .any(|row| row.starts_with("-- the server did not save"))
                && joined(rows).contains("gone")
        })
        .await;
    assert!(!config.path().join("clients/devices/laptop.toml").exists());
}

#[tokio::test]
async fn a_use_behind_an_entry() {
    let (session, mut client, _config) = with_world_layouts("\"alpha/aria\" = \"focus\"\n").await;
    let shown = after_save(
        &session,
        &mut client,
        b"/layout use status\r",
        "the layout of the device",
        |shown| shown.contains("layout = \"status\""),
    )
    .await;
    assert!(shown.contains("layout = \"status\""), "{shown}");
    client
        .wait_rows("the notice that names /layout here", |rows| {
            rows.iter().any(|row| row.contains("/layout here"))
                && !rows[0].starts_with("[alpha/aria]")
        })
        .await;
}

#[tokio::test]
async fn the_next_layout_of_a_world_entry() {
    let (session, mut client, _config) = with_layer(
        &["alpha/aria"],
        "[keys]\n\"alt+l\" = \"layout-next\"\n[world-layouts]\n\"alpha/aria\" = \"classic\"\n",
    )
    .await;
    let shown = after_save(
        &session,
        &mut client,
        b"\x1bl",
        "the next layout of the entry",
        |shown| shown.contains("world-layouts.\"alpha/aria\" = \"focus\""),
    )
    .await;
    assert!(
        has_config_line(&shown, "layout = \"classic\" # default"),
        "{shown}"
    );
}

#[tokio::test]
async fn arrange_mode_saves_through_the_server() {
    let (session, mut client, config) = with_layer(&["alpha/aria"], "").await;
    // Alt+L starts the mode, and its last row shows the keys.
    client.send(b"\x1bl");
    wait_row(&client, 24, "the keys of arrange mode", |row| {
        row.contains("Enter") && row.contains("Esc")
    })
    .await;
    // The tabs are block A, and the output is block B: A, B and Down put the tabs below it.
    client.send(b"ab\x1b[B");
    wait_row(&client, 22, "the tabs above the editor", |row| {
        row.contains("alpha/aria]")
    })
    .await;
    after_save(
        &session,
        &mut client,
        b"\r",
        "the layout of the device",
        |shown| has_config_line(shown, "layout = \"laptop\" # clients/devices/laptop.toml"),
    )
    .await;
    assert_eq!(
        place_of(config.path(), "laptop"),
        ["output", "editor below output", "tabs below output"]
    );
}
