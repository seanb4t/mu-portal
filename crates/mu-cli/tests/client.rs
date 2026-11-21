//! The scenarios of `mu client` on the built binary, in a pseudo-terminal of 80 by 24.

use std::time::Duration;

use mu_client::PortalClient;
use mu_proto::{EventKind, WorldState};

use mu_fake_server::{Listening, Script, ScriptedGame};
use support::process::{
    Answers, BOUND, PtyClient, PtyOn, Reaper, Relay, SERVING, ServerBuilder, Session, TWO_WORLDS,
    mu_on_pty, private_tempdir,
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
async fn pose_of_two_lines() {
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
async fn line_break_that_the_world_refuses() {
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
