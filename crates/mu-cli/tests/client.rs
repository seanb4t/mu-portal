//! The scenarios of `mu client` on the built binary, in a pseudo-terminal of 80 by 24.

use std::path::Path;
use std::time::Duration;

use mu_client::PortalClient;
use mu_proto::{EventKind, WorldState};

use mu_fake_server::{Listening, Script, ScriptedGame};
use support::process::{
    Answers, BOUND, PtyClient, PtyOn, Reaper, Relay, SERVING, ServerBuilder, ServerProcess,
    Session, TWO_WORLDS, mu_on_pty, private_tempdir,
};
use support::{ConfigDir, mode, stderr};

mod support;

#[tokio::test]
async fn client_starts_a_server() {
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
async fn two_clients_start_one_server() {
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
async fn client_gives_up_after_five_seconds() {
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

#[test]
fn client_needs_a_terminal_on_stdin() {
    let dir = private_tempdir();
    let run = dir.path().join("run");
    let socket = run.join("mu.sock").display().to_string();
    let output = mu_on_pty(&["client", "--socket", &socket], PtyOn::Stdout, dir.path());
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("must be a terminal"),
        "{}",
        stderr(&output)
    );
    assert!(!run.exists(), "the client created {}", run.display());
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

// client.toml.

/// A configuration directory with `client.toml` that holds `text`.
fn client_file(text: &str) -> ConfigDir {
    ConfigDir::empty().file("client.toml", text)
}

/// A client with `--config <config>` on the server of `session`, after the greeting.
async fn configured(session: &Session, config: &Path, answers: Answers) -> PtyClient {
    let client = PtyClient::start_args(
        session.server.socket(),
        session.server.dir().path(),
        answers,
        &["--config".as_ref(), config.as_os_str()],
        &[],
    );
    client.wait_greeting().await;
    client
}

/// A server with `worlds`, and a client on it with `client.toml` that holds `text`.
async fn with_file(worlds: &[&str], text: &str) -> (Session, PtyClient, ConfigDir) {
    let session = Session::with_worlds(worlds).await;
    let config = client_file(text);
    let client = configured(&session, config.path(), Answers::Default).await;
    (session, client, config)
}

/// Sends `hello` and Enter, and waits for the echo line.
async fn echo_of_hello(client: &mut PtyClient) {
    client.send(b"hello\r");
    client
        .wait_rows("the echo line", |rows| {
            rows.iter().any(|row| row.starts_with("> hello"))
        })
        .await;
}

#[tokio::test]
async fn a_file_that_sets_the_echo() {
    let (_session, mut client, _config) = with_file(&["alpha/aria"], "echo = \"show\"\n").await;
    echo_of_hello(&mut client).await;
    assert_eq!(client.dims("> hello"), [false; 7]);
}

#[tokio::test]
async fn a_file_that_sets_dim() {
    let (_session, mut client, _config) = with_file(&["alpha/aria"], "echo = \"dim\"\n").await;
    echo_of_hello(&mut client).await;
    assert_eq!(client.dims("> hello"), [true; 7]);
}

#[tokio::test]
async fn no_file() {
    let session = Session::with_worlds(&["alpha/aria"]).await;
    let config = ConfigDir::empty();
    let client = configured(&session, config.path(), Answers::Default).await;
    client
        .wait_top("the world", |top| top.contains("[alpha/aria]"))
        .await;
}

/// Starts `mu client --config` on a directory with `client.toml` that holds `text`, with no
/// server, and returns the screen after the exit with status 1. The client never enters the
/// alternate screen.
async fn refused_file(config: &ConfigDir) -> String {
    let dir = private_tempdir();
    let socket = dir.path().join("s.sock");
    let mut client = PtyClient::start_args(
        &socket,
        dir.path(),
        Answers::Default,
        &["--config".as_ref(), config.path().as_os_str()],
        &[],
    );
    let status = client.exit_within(BOUND).await.expect("the client exits");
    assert_eq!(status.exit_code(), 1);
    assert!(
        !client
            .output()
            .windows(8)
            .any(|bytes| bytes == b"\x1b[?1049h"),
        "the client entered the alternate screen"
    );
    assert!(!socket.exists(), "the client started a server");
    client.contents().replace('\n', "")
}

/// Asserts that a client with `client.toml` holding `text` stops, and that its message names
/// each of `names`.
async fn stops_naming(text: &str, names: &[&str]) {
    let screen = refused_file(&client_file(text)).await;
    for name in names {
        assert!(
            screen.contains(name),
            "{name} is not on the screen:\n{screen}"
        );
    }
}

#[tokio::test]
async fn an_unknown_key() {
    stops_naming("colour = \"red\"\n", &["client.toml", "colour"]).await;
}

#[tokio::test]
async fn a_bad_echo_value() {
    stops_naming("echo = \"loud\"\n", &["client.toml", "echo"]).await;
}

#[tokio::test]
async fn a_file_that_cannot_be_read() {
    use std::os::unix::fs::PermissionsExt as _;
    let config = client_file("");
    let path = config.path().join("client.toml");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
    // A user who can read every file, such as root, gets no read error.
    if std::fs::File::open(&path).is_ok() {
        return;
    }
    let screen = refused_file(&config).await;
    assert!(screen.contains("client.toml"), "{screen}");
}

#[tokio::test]
async fn not_toml() {
    stops_naming("echo = \n", &["client.toml:1:"]).await;
}

#[tokio::test]
async fn the_client_takes_the_flag() {
    let dir = private_tempdir();
    let env = ConfigDir::empty().file("client.toml", "echo = \"loud\"\n");
    let flag = ConfigDir::base();
    let run = dir.path().join("run");
    let _reaper = Reaper(run.join("server.log"));
    let socket = run.join("mu.sock");
    let client = PtyClient::start_args(
        &socket,
        dir.path(),
        Answers::Default,
        &["--config".as_ref(), flag.path().as_os_str()],
        &[("MU_CONFIG", Some(env.path().as_os_str()))],
    );
    client
        .wait_top("the world of the flag", |top| top.contains("[alpha/aria]"))
        .await;
    let worlds = PortalClient::unix(&socket).list_worlds().await.unwrap();
    assert_eq!(
        worlds
            .iter()
            .map(|world| world.id.as_str())
            .collect::<Vec<_>>(),
        ["alpha/aria"]
    );
}

#[tokio::test]
async fn the_client_warns_of_a_missing_directory() {
    let session = Session::with_worlds(&["alpha/aria"]).await;
    let missing = private_tempdir().path().join("none");
    let client = configured(&session, &missing, Answers::Default).await;
    client
        .wait_top("the world", |top| top.contains("[alpha/aria]"))
        .await;
    let output = String::from_utf8_lossy(&client.output()).into_owned();
    assert!(output.contains(&missing.display().to_string()), "{output}");
}

#[tokio::test]
async fn the_client_with_no_directory() {
    let session = Session::with_worlds(&["alpha/aria"]).await;
    let mut client = PtyClient::start_args(
        session.server.socket(),
        session.server.dir().path(),
        Answers::Default,
        &[],
        &[
            ("MU_CONFIG", None),
            ("XDG_CONFIG_HOME", None),
            ("HOME", Some("".as_ref())),
        ],
    );
    client.wait_greeting().await;
    echo_of_hello(&mut client).await;
    assert_eq!(client.dims("> hello"), [true; 7]);
}

#[tokio::test]
async fn the_first_list_gives_up() {
    let dir = private_tempdir();
    let socket = dir.path().join("s.sock");
    let listener = tokio::net::UnixListener::bind(&socket).unwrap();
    // Each connection stays open with no answer.
    let held = tokio::spawn(async move {
        let mut open = Vec::new();
        while let Ok((stream, _)) = listener.accept().await {
            open.push(stream);
        }
    });
    let mut client = PtyClient::start(&socket, dir.path());
    // 5 seconds, and a margin for the start of the process.
    let status = client
        .exit_within(Duration::from_secs(8))
        .await
        .expect("the client leaves the first list");
    assert_eq!(status.exit_code(), 1);
    let screen = client.contents().replace('\n', "");
    assert!(screen.contains(&socket.display().to_string()), "{screen}");
    held.abort();
}

// The keys table of client.toml.

const THREE_WORLDS: &[&str] = &["alpha/aria", "beta/bryn", "gamma/gil"];

/// Presses `key` on a client with `[keys]` holding `bindings`, and waits until the top row shows
/// `[<world>]`.
async fn key_gives_focus(worlds: &[&str], bindings: &str, key: &[u8], world: &str) {
    let text = format!("[keys]\n{bindings}\n");
    let (_session, mut client, _config) = with_file(worlds, &text).await;
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
        with_file(TWO_WORLDS, "[keys]\n\"ctrl+n\" = \"none\"\n").await;
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
        with_file(&["alpha/aria"], "[keys]\n\"alt+enter\" = \"none\"\n").await;
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
    let session = Session::with_worlds(&["alpha/aria"]).await;
    let config = client_file("[keys]\n\"ctrl+shift+z\" = \"none\"\n");
    let mut client = configured(&session, config.path(), Answers::Kitty).await;
    client.send(b"h");
    editor_holds(&client, "h").await;
    // Ctrl+Z, then Ctrl+Shift+Z in the kitty protocol, then `x`. The default redo would bring
    // `h` back before the `x`.
    client.send(b"\x1a\x1b[122;6ux");
    editor_holds(&client, "x").await;
}

#[tokio::test]
async fn two_names_of_one_key() {
    stops_naming(
        "[keys]\n\"ctrl+shift+z\" = \"redo\"\n\"shift+ctrl+z\" = \"undo\"\n",
        &["client.toml", "ctrl+shift+z", "shift+ctrl+z"],
    )
    .await;
}

#[tokio::test]
async fn shift_with_a_digit() {
    stops_naming(
        "[keys]\n\"shift+1\" = \"quit\"\n",
        &["client.toml", "shift+1"],
    )
    .await;
}

#[tokio::test]
async fn an_uppercase_key() {
    stops_naming(
        "[keys]\n\"ctrl+N\" = \"quit\"\n",
        &["client.toml", "ctrl+N"],
    )
    .await;
}

#[tokio::test]
async fn an_unknown_entry() {
    stops_naming(
        "[keys]\n\"alt+n\" = \"fly\"\n",
        &["client.toml", "alt+n", "fly"],
    )
    .await;
}

#[tokio::test]
async fn an_unknown_key_name() {
    stops_naming(
        "[keys]\n\"hyper+n\" = \"quit\"\n",
        &["client.toml", "hyper+n"],
    )
    .await;
}

#[tokio::test]
async fn an_entry_that_needs_an_argument() {
    stops_naming(
        "[keys]\n\"alt+f\" = \"focus\"\n",
        &["client.toml", "alt+f", "focus"],
    )
    .await;
}

#[tokio::test]
async fn a_control_character() {
    stops_naming("[keys]\n\"alt+\\u007f\" = \"quit\"\n", &["client.toml"]).await;
}
