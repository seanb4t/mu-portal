//! The scenarios of the terminal-client capability on `update` and `view`, drawn on a
//! `TestBackend`.

use mu_client::ClientEvent;
use mu_proto::{Event, EventKind, Source, World};
use mu_tui::convert::convert;
use mu_tui::history::HISTORY_LIMIT;
use mu_tui::{Effect, Model, Msg, Reply, update, view};
use proptest::prelude::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::{Buffer, Cell};
use ratatui::style::{Color, Modifier, Style};
use termina::Event as TermEvent;
use termina::event::{KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseEventKind};

use support::{type_lines, type_text};

mod support;

fn app(worlds: &[&str]) -> Model {
    Model::new(worlds.iter().map(|world| (*world).to_owned()))
}

/// A model of `run`, before its first list of the worlds ends.
fn starting() -> Model {
    Model::starting()
}

/// A model of `run` after its first list of the worlds ends with `listed`.
fn after_list(listed: Result<Vec<World>, String>) -> Model {
    let mut app = starting();
    assert_eq!(update(&mut app, Msg::Listed(listed)), []);
    app
}

/// An 80 by 24 screen.
fn draw(app: &mut Model) -> Buffer {
    draw_sized(app, 24)
}

/// A screen 80 cells wide and `height` rows high.
fn draw_sized(app: &mut Model, height: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(80, height)).unwrap();
    terminal.draw(|frame| view(app, frame)).unwrap();
    terminal.backend().buffer().clone()
}

/// The rows of `buffer`, with no trailing spaces.
fn text_rows(buffer: &Buffer) -> Vec<String> {
    (0..buffer.area.height)
        .map(|y| {
            let row: String = (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect();
            row.trim_end().to_owned()
        })
        .collect()
}

/// The rows of an 80 by 24 screen, with no trailing spaces.
fn screen(app: &mut Model) -> Vec<String> {
    text_rows(&draw(app))
}

/// The cells of the first row of the screen that starts with `text`, one for each char of
/// `text`.
fn cells_of(app: &mut Model, text: &str) -> Vec<Cell> {
    let buffer = draw(app);
    let y = screen(app)
        .iter()
        .position(|row| row.starts_with(text))
        .unwrap_or_else(|| panic!("no row starts with {text:?}"));
    let y = u16::try_from(y).unwrap();
    (0..u16::try_from(text.chars().count()).unwrap())
        .map(|x| buffer[(x, y)].clone())
        .collect()
}

/// The foreground color and the modifiers of each cell.
fn looks(cells: &[Cell]) -> Vec<(Color, Modifier)> {
    cells.iter().map(|cell| (cell.fg, cell.modifier)).collect()
}

fn plain(count: usize) -> Vec<(Color, Modifier)> {
    vec![(Color::Reset, Modifier::empty()); count]
}

fn top(app: &mut Model) -> String {
    screen(app).remove(0)
}

/// The rows between the top row and the input line, with no empty row.
fn rows(app: &mut Model) -> Vec<String> {
    let screen = screen(app);
    screen[1..screen.len() - 1]
        .iter()
        .filter(|row| !row.is_empty())
        .cloned()
        .collect()
}

fn event(world: &str, source: Source, text: &str) -> ClientEvent {
    let kind = match source {
        Source::SOURCE_PLAYER => EventKind::EVENT_KIND_INPUT,
        Source::SOURCE_CREDENTIAL => EventKind::EVENT_KIND_LOGIN,
        _ => EventKind::EVENT_KIND_LINE,
    };
    ClientEvent::Event(Event {
        world: world.to_owned(),
        kind: kind.into(),
        source: source.into(),
        text: text.to_owned(),
        ..Default::default()
    })
}

fn game(world: &str, text: &str) -> ClientEvent {
    event(world, Source::SOURCE_GAME, text)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, Modifiers::NONE)
}

/// Presses Up, which runs no effect.
fn up(app: &mut Model) {
    assert_eq!(press(app, key(KeyCode::Up)), []);
}

/// Presses Down, which runs no effect.
fn down(app: &mut Model) {
    assert_eq!(press(app, key(KeyCode::Down)), []);
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), Modifiers::CONTROL)
}

/// Presses `key` and returns the effects.
fn press(app: &mut Model, key: KeyEvent) -> Vec<Effect> {
    update(app, Msg::Terminal(TermEvent::Key(key)))
}

/// Gives `item` to the model. Its only effect can be a list of the worlds.
fn on_event(app: &mut Model, item: ClientEvent) {
    let effects = update(app, Msg::Client(item));
    assert!(
        effects.iter().all(|effect| *effect == Effect::ListWorlds),
        "{effects:?}"
    );
}

fn on_reply(app: &mut Model, reply: Reply) {
    assert_eq!(update(app, Msg::Reply(reply)), []);
}

/// Two worlds, each with its greeting.
fn two_worlds() -> Model {
    let mut app = app(&["alpha/aria", "beta/bryn"]);
    on_event(&mut app, game("alpha/aria", "Welcome, aria."));
    on_event(&mut app, game("beta/bryn", "Welcome, bryn."));
    app
}

#[test]
fn first_view() {
    let mut app = two_worlds();
    assert!(
        top(&mut app).starts_with("[alpha/aria] beta/bryn"),
        "{}",
        top(&mut app)
    );
    assert_eq!(rows(&mut app), ["Welcome, aria."]);
    // The editor takes only the bottom row.
    type_text(&mut app, "look");
    assert_eq!(screen(&mut app)[22..], ["", "look"]);
}

#[test]
fn first_view_snapshot() {
    let mut app = two_worlds();
    type_text(&mut app, "look");
    let mut terminal = Terminal::new(TestBackend::new(40, 6)).unwrap();
    terminal.draw(|frame| view(&mut app, frame)).unwrap();
    let rows = text_rows(terminal.backend().buffer());
    assert!(rows[0].starts_with("[alpha/aria] beta/bryn"), "{}", rows[0]);
    assert!(rows[1..5].iter().any(|row| row == "Welcome, aria."));
    assert!(rows.iter().all(|row| row != "Welcome, bryn."));
    insta::assert_snapshot!(terminal.backend());
}

#[test]
fn world_order() {
    let mut app = app(&["alpha-2/cy", "alpha/bea", "alpha/aria"]);
    assert_eq!(top(&mut app), "[alpha/aria] alpha/bea alpha-2/cy");
}

#[test]
fn no_worlds() {
    assert_eq!(top(&mut app(&[])), "no worlds");
}

#[test]
fn listed_worlds_are_the_worlds() {
    let worlds = ["beta/bryn", "alpha/aria"].map(|id| World {
        id: id.to_owned(),
        ..Default::default()
    });
    let mut app = after_list(Ok(worlds.to_vec()));
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn");
}

const LIST_FAILED: &str = "-- could not list the worlds: unavailable: no server";

fn failed_listing() -> Model {
    after_list(Err("unavailable: no server".to_owned()))
}

#[test]
fn failed_list_of_worlds_shows_its_error() {
    let mut app = failed_listing();
    assert_eq!(top(&mut app), "no worlds");
    assert_eq!(rows(&mut app), [LIST_FAILED]);
}

#[test]
fn notice_stays_when_the_first_world_event_comes() {
    let mut app = failed_listing();
    on_event(&mut app, game("alpha/aria", "hello"));
    assert_eq!(rows(&mut app), [LIST_FAILED, "hello"]);
}

#[test]
fn notice_shows_once_after_a_reset_that_follows_the_first_world_event() {
    let mut app = failed_listing();
    on_event(&mut app, game("alpha/aria", "hello"));
    on_event(
        &mut app,
        ClientEvent::Reset {
            instance_id: "i".to_owned(),
        },
    );
    assert_eq!(rows(&mut app), [LIST_FAILED]);
}

#[test]
fn line_prefixes() {
    let mut app = app(&["alpha/aria"]);
    on_event(
        &mut app,
        event("alpha/aria", Source::SOURCE_CREDENTIAL, "connect aria ****"),
    );
    on_event(
        &mut app,
        event("alpha/aria", Source::SOURCE_PLAYER, "hello"),
    );
    on_event(&mut app, game("alpha/aria", "You say, \"hello\""));
    assert_eq!(
        rows(&mut app),
        ["-- connect aria ****", "> hello", "You say, \"hello\""]
    );
}

#[test]
fn control_characters_are_replaced() {
    let mut app = app(&["alpha/aria"]);
    on_event(
        &mut app,
        game("alpha/aria", "\u{1b}[2Jhi\tthere\u{7}\u{7f}\u{9b}~\u{a0}."),
    );
    on_reply(
        &mut app,
        Reply::Refused {
            world: "alpha/aria".to_owned(),
            text: String::new(),
            reason: "bad\u{1b}]0;title\u{7}".to_owned(),
        },
    );
    assert_eq!(
        rows(&mut app),
        [
            "hi there\u{fffd}\u{fffd}\u{fffd}~\u{a0}.",
            "-- bad\u{fffd}]0;title\u{fffd}"
        ]
    );
}

#[test]
fn control_characters_are_replaced_in_every_part() {
    let world = "alpha/a\u{1b}x";
    let mut app = app(&[world]);
    on_event(&mut app, event(world, Source::SOURCE_PLAYER, "hi\u{7}"));
    on_event(&mut app, event(world, Source::SOURCE_SYSTEM, "up\u{9b}"));
    on_event(
        &mut app,
        event(world, Source::SOURCE_CREDENTIAL, "pw\u{7f}"),
    );
    assert_eq!(top(&mut app), "[alpha/a\u{fffd}x]");
    assert_eq!(
        rows(&mut app),
        ["> hi\u{fffd}", "-- up\u{fffd}", "-- pw\u{fffd}"]
    );
}

#[test]
fn a_line_goes_to_the_world_in_focus() {
    let mut app = two_worlds();
    press(&mut app, ctrl('n'));
    type_text(&mut app, "hi");
    let action = press(&mut app, key(KeyCode::Enter));
    assert_eq!(
        action,
        [Effect::Send {
            world: "beta/bryn".to_owned(),
            text: "hi".to_owned()
        }]
    );
    assert_eq!(app.input(), "");
}

#[test]
fn an_empty_line() {
    let mut app = two_worlds();
    let action = press(&mut app, key(KeyCode::Enter));
    assert_eq!(
        action,
        [Effect::Send {
            world: "alpha/aria".to_owned(),
            text: String::new()
        }]
    );
}

#[test]
fn enter_with_no_world() {
    let mut app = app(&[]);
    type_text(&mut app, "hi");
    assert_eq!(press(&mut app, key(KeyCode::Enter)), []);
    let rows = rows(&mut app);
    assert!(
        rows.iter()
            .any(|row| row.starts_with("-- ") && row.contains("no world")),
        "{rows:?}"
    );
    assert_eq!(app.input(), "hi");
}

#[test]
fn a_refused_line() {
    let mut app = two_worlds();
    type_text(&mut app, "hi");
    assert_ne!(press(&mut app, key(KeyCode::Enter)), [] as [Effect; 0]);
    on_reply(
        &mut app,
        Reply::Refused {
            world: "alpha/aria".to_owned(),
            text: "hi".to_owned(),
            reason: "alpha/aria is not connected".to_owned(),
        },
    );
    assert_eq!(
        rows(&mut app).last().unwrap(),
        "-- alpha/aria is not connected"
    );
    assert_eq!(app.input(), "hi");
    assert_eq!(screen(&mut app).last().unwrap(), "hi");
}

#[test]
fn a_refused_line_after_more_text() {
    let mut app = two_worlds();
    type_text(&mut app, "hi");
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "hi")]
    );
    type_text(&mut app, "next");
    on_reply(&mut app, refused("alpha/aria", "hi"));
    assert_eq!(app.input(), "hi");
    up(&mut app);
    assert_eq!(app.input(), "next");
}

#[test]
fn ctrl_n_moves_focus() {
    let mut app = two_worlds();
    press(&mut app, ctrl('n'));
    assert_eq!(top(&mut app), "alpha/aria [beta/bryn]");
    assert_eq!(rows(&mut app), ["Welcome, bryn."]);
}

#[test]
fn ctrl_p_wraps_around() {
    let mut app = two_worlds();
    press(&mut app, ctrl('p'));
    assert_eq!(top(&mut app), "alpha/aria [beta/bryn]");
}

#[test]
fn ctrl_n_wraps_at_the_end() {
    let mut app = two_worlds();
    press(&mut app, ctrl('n'));
    press(&mut app, ctrl('n'));
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn");
}

#[test]
fn keys_with_no_world() {
    let mut app = app(&[]);
    assert_eq!(press(&mut app, ctrl('n')), []);
    assert_eq!(press(&mut app, ctrl('p')), []);
    assert_eq!(top(&mut app), "no worlds");
}

#[test]
fn one_world_keeps_focus() {
    let mut app = app(&["alpha/aria"]);
    press(&mut app, ctrl('n'));
    press(&mut app, ctrl('p'));
    assert_eq!(top(&mut app), "[alpha/aria]");
}

#[test]
fn activity_mark() {
    let mut app = app(&["alpha/aria", "beta/bryn"]);
    on_event(&mut app, game("beta/bryn", "You say, \"ping\""));
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn*");
    press(&mut app, ctrl('n'));
    assert_eq!(top(&mut app), "alpha/aria [beta/bryn]");
}

#[test]
fn lost_stream_resumes() {
    let mut app = two_worlds();
    on_event(&mut app, ClientEvent::Lost);
    assert!(top(&mut app).contains("disconnected"));
    on_event(&mut app, ClientEvent::Resumed);
    on_event(&mut app, game("alpha/aria", "You say, \"after the cut\""));
    let rows = rows(&mut app);
    assert_eq!(
        rows.iter()
            .filter(|row| *row == "You say, \"after the cut\"")
            .count(),
        1
    );
    assert!(!top(&mut app).contains("disconnected"));
}

#[test]
fn resumed_clears_disconnected() {
    let mut app = two_worlds();
    on_event(&mut app, ClientEvent::Lost);
    assert!(top(&mut app).contains("disconnected"));
    on_event(&mut app, ClientEvent::Resumed);
    assert!(!top(&mut app).contains("disconnected"));
}

/// A prompt event of the game: text with no line end.
fn prompt(world: &str, text: &str) -> ClientEvent {
    ClientEvent::Event(Event {
        world: world.to_owned(),
        kind: EventKind::EVENT_KIND_PROMPT.into(),
        source: Source::SOURCE_GAME.into(),
        text: text.to_owned(),
        ..Default::default()
    })
}

#[test]
fn press_on_the_unfinished_line() {
    let mut app = app(&["alpha/aria"]);
    on_event(&mut app, game("alpha/aria", "Welcome, aria."));
    on_event(&mut app, prompt("alpha/aria", "Name: "));
    assert_eq!(screen(&mut app)[22], "Name:");
    let press = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 1,
        row: 22,
        modifiers: Modifiers::NONE,
    };
    assert_eq!(update(&mut app, Msg::Terminal(TermEvent::Mouse(press))), []);
    assert_eq!(app.pane(app.focused_pane()).unwrap().selection(), None);
}

#[test]
fn ctrl_c_quits() {
    assert_eq!(press(&mut app(&["alpha/aria"]), ctrl('c')), [Effect::Quit]);
}

/// The view of `text` on the screen: a tab as a space and each other control character as
/// U+FFFD.
fn shown(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\t' => ' ',
            c if c.is_control() => '\u{fffd}',
            c => c,
        })
        .collect()
}

proptest! {
    #[test]
    fn no_control_character_is_drawn(
        character in "[a\\x00-\\x1f\\x7f\\u{80}-\\u{9f}]{1,10}",
        line in "[a \\x00-\\x1f\\x7f\\u{80}-\\u{9f}]{0,40}",
        source in prop::sample::select(vec![
            Source::SOURCE_PLAYER,
            Source::SOURCE_SYSTEM,
            Source::SOURCE_CREDENTIAL,
        ]),
    ) {
        let world = format!("alpha/{character}");
        let mut app = app(&[&world]);
        on_event(&mut app, event(&world, source, &line));
        let prefix = if source == Source::SOURCE_PLAYER { "> " } else { "-- " };
        let want = format!("{prefix}{}", shown(&line));
        let drawn = rows(&mut app).first().cloned().unwrap_or_default();
        prop_assert_eq!(drawn, want.trim_end());
        let top_want = format!("[alpha/{}]", shown(&character));
        prop_assert_eq!(top(&mut app), top_want.trim_end());
    }
}

proptest! {
    #[test]
    fn game_line_draws_no_control_character(
        line in "[a; \x1b\x07\x00-\x1f\x7f\u{80}-\u{9f}\\[\\]0-9m]{0,40}",
    ) {
        let mut app = app(&["alpha/aria"]);
        on_event(&mut app, game("alpha/aria", &line));
        let drawn = rows(&mut app).first().cloned().unwrap_or_default();
        prop_assert!(!drawn.chars().any(char::is_control), "{:?}", drawn);
    }
}

#[test]
fn game_line_drops_escape_sequences() {
    let drawn: Vec<String> = [
        "a\x1b[31mb\x1b[0mc",
        "a\x1b[2Jb",
        "x\x1b]0;title\x07y",
        "a\x07b",
        "a\tb",
    ]
    .into_iter()
    .map(|line| {
        let mut app = app(&["alpha/aria"]);
        on_event(&mut app, game("alpha/aria", line));
        rows(&mut app).first().cloned().unwrap_or_default()
    })
    .collect();
    assert_eq!(drawn, ["abc", "ab", "xy", "a\u{FFFD}b", "a b"]);
}

#[test]
fn colors() {
    let mut app = app(&["alpha/aria"]);
    on_event(
        &mut app,
        game("alpha/aria", "\u{1b}[1;31mred\u{1b}[0m plain"),
    );
    assert_eq!(rows(&mut app), ["red plain"]);
    let cells = cells_of(&mut app, "red plain");
    assert_eq!(
        (looks(&cells[..3]), looks(&cells[3..])),
        (vec![(Color::Red, Modifier::BOLD); 3], plain(6))
    );
}

#[test]
fn other_escape_sequences() {
    let mut app = app(&["alpha/aria"]);
    on_event(&mut app, game("alpha/aria", "before"));
    on_event(
        &mut app,
        game("alpha/aria", "a\u{1b}]0;title\u{7}b\u{1b}[2Jc"),
    );
    assert_eq!(rows(&mut app), ["before", "abc"]);
}

#[test]
fn color_across_two_lines() {
    let mut app = app(&["alpha/aria"]);
    on_event(&mut app, game("alpha/aria", "\u{1b}[32mgreen"));
    on_event(&mut app, game("alpha/aria", "still\u{1b}[0m done"));
    let green = vec![(Color::Green, Modifier::empty()); 5];
    let still = cells_of(&mut app, "still done");
    assert_eq!(
        (
            looks(&cells_of(&mut app, "green")),
            looks(&still[..5]),
            looks(&still[5..])
        ),
        (green.clone(), green, plain(5))
    );
}

#[test]
fn indexed_and_24_bit_colors() {
    let mut app = app(&["alpha/aria"]);
    on_event(
        &mut app,
        game("alpha/aria", "\u{1b}[38;5;208mA\u{1b}[38;2;10;20;30mB"),
    );
    let cells = cells_of(&mut app, "AB");
    assert_eq!(
        (cells[0].fg, cells[1].fg),
        (Color::Indexed(208), Color::Rgb(10, 20, 30))
    );
}

/// The raw game lines of the `aresmush` and `rhostmush` transcripts, with their escape
/// sequences, after the line that names the command with `--raw` that made them.
const TRANSCRIPT_LINES: &str = include_str!("data/transcript-lines.txt");

#[test]
fn transcript_lines_convert() {
    let (header, lines) = TRANSCRIPT_LINES.split_once('\n').unwrap();
    assert!(header.starts_with("# made by: "), "{header}");
    let mut style = Style::default();
    let mut spans = 0;
    for line in lines.lines() {
        let converted = convert(line, &mut style);
        assert!(
            !converted.text.chars().any(char::is_control),
            "{:?}",
            converted.text
        );
        spans += converted.spans.len();
    }
    assert!(spans > 0);
}

#[test]
fn prompt_with_no_line_end() {
    let mut app = app(&["alpha/aria"]);
    on_event(&mut app, prompt("alpha/aria", "Name: "));
    assert_eq!(screen(&mut app)[22], "Name:");
}

#[test]
fn read_ends_inside_an_escape() {
    let mut app = app(&["alpha/aria"]);
    on_event(&mut app, prompt("alpha/aria", "ab\u{1b}[3"));
    on_event(&mut app, game("alpha/aria", "ab\u{1b}[31mred"));
    assert_eq!(rows(&mut app), ["abred"]);
    let cells = cells_of(&mut app, "abred");
    assert_eq!(
        (looks(&cells[..2]), looks(&cells[2..])),
        (plain(2), vec![(Color::Red, Modifier::empty()); 3])
    );
}

#[test]
fn prompt_does_not_carry_its_style() {
    let mut app = app(&["alpha/aria"]);
    on_event(&mut app, prompt("alpha/aria", "\u{1b}[31mName: "));
    on_event(&mut app, game("alpha/aria", "plain"));
    assert_eq!(looks(&cells_of(&mut app, "plain")), plain(5));
}

#[test]
fn open_osc() {
    let mut app = app(&["alpha/aria"]);
    on_event(&mut app, game("alpha/aria", "a\u{1b}]0;title"));
    on_event(&mut app, game("alpha/aria", "next"));
    assert_eq!(rows(&mut app), ["a", "next"]);
}

#[test]
fn reset_clears_each_pane() {
    let mut app = app(&["alpha/aria"]);
    on_event(&mut app, game("alpha/aria", "\u{1b}[31mred"));
    on_event(&mut app, prompt("alpha/aria", "Name: "));
    draw(&mut app);
    let press = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 0,
        row: 1,
        modifiers: Modifiers::NONE,
    };
    assert_eq!(update(&mut app, Msg::Terminal(TermEvent::Mouse(press))), []);
    assert!(app.pane(app.focused_pane()).unwrap().selection().is_some());
    on_event(
        &mut app,
        ClientEvent::Reset {
            instance_id: "i".to_owned(),
        },
    );
    on_event(&mut app, game("alpha/aria", "after"));
    assert_eq!(rows(&mut app), ["after"]);
    let pane = app.pane(app.focused_pane()).unwrap();
    assert_eq!(
        (
            pane.selection(),
            pane.anchor(),
            looks(&cells_of(&mut app, "after"))
        ),
        (None, None, plain(5))
    );
}

/// `alpha/aria` with the game lines `1` to `count`, drawn once.
fn numbered(count: u32) -> Model {
    let mut app = app(&["alpha/aria"]);
    for n in 1..=count {
        on_event(&mut app, game("alpha/aria", &n.to_string()));
    }
    draw(&mut app);
    app
}

/// The rows of the pane on an 80 by 24 screen, with no scroll bar column.
fn pane_rows(app: &mut Model) -> Vec<String> {
    let screen = screen(app);
    screen[1..screen.len() - 1]
        .iter()
        .map(|row| {
            row.chars()
                .take(79)
                .collect::<String>()
                .trim_end()
                .to_owned()
        })
        .collect()
}

fn wheel(app: &mut Model, kind: MouseEventKind) -> Vec<Effect> {
    let mouse = MouseEvent {
        kind,
        column: 10,
        row: 10,
        modifiers: Modifiers::NONE,
    };
    update(app, Msg::Terminal(TermEvent::Mouse(mouse)))
}

#[test]
fn back_and_forward() {
    let mut app = numbered(40);
    assert_eq!(press(&mut app, key(KeyCode::PageUp)), []);
    let back = pane_rows(&mut app);
    assert_eq!(
        (back[0].as_str(), back.contains(&"40".to_owned())),
        ("1", false)
    );
    assert_eq!(press(&mut app, key(KeyCode::PageDown)), []);
    assert_eq!(pane_rows(&mut app).last().unwrap(), "40");
}

#[test]
fn new_line_while_scrolled_back() {
    let mut app = numbered(40);
    press(&mut app, key(KeyCode::PageUp));
    let before = pane_rows(&mut app);
    on_event(&mut app, game("alpha/aria", "ping"));
    assert_eq!(pane_rows(&mut app), before);
}

#[test]
fn enter_returns_to_the_bottom() {
    let mut app = numbered(40);
    press(&mut app, key(KeyCode::PageUp));
    type_text(&mut app, "hi");
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [Effect::Send {
            world: "alpha/aria".to_owned(),
            text: "hi".to_owned()
        }]
    );
    on_event(&mut app, event("alpha/aria", Source::SOURCE_PLAYER, "hi"));
    on_event(&mut app, game("alpha/aria", "You say, \"hi\""));
    assert_eq!(pane_rows(&mut app).last().unwrap(), "You say, \"hi\"");
}

#[test]
fn mouse_wheel_scrolls_three_rows() {
    let mut app = numbered(40);
    assert_eq!(
        (
            pane_rows(&mut app)[0].as_str(),
            wheel(&mut app, MouseEventKind::ScrollUp)
        ),
        ("19", vec![])
    );
    let rows = pane_rows(&mut app);
    assert_eq!((rows[0].as_str(), rows[21].as_str()), ("16", "37"));
}

#[test]
fn back_to_a_scrolled_world() {
    let mut app = two_worlds();
    for n in 1..=40 {
        on_event(&mut app, game("alpha/aria", &n.to_string()));
    }
    draw(&mut app);
    press(&mut app, key(KeyCode::PageUp));
    let before = pane_rows(&mut app);
    press(&mut app, ctrl('n'));
    draw(&mut app);
    press(&mut app, ctrl('p'));
    assert_eq!(pane_rows(&mut app), before);
}

#[test]
fn page_up_in_a_pane_of_one_row() {
    let mut app = app(&["alpha/aria"]);
    for n in 1..=5 {
        on_event(&mut app, game("alpha/aria", &n.to_string()));
    }
    let first = |app: &mut Model| text_rows(&draw_sized(app, 3))[1].chars().next();
    assert_eq!(first(&mut app), Some('5'));
    assert_eq!(press(&mut app, key(KeyCode::PageUp)), []);
    assert_eq!(first(&mut app), Some('4'));
}

#[test]
fn page_keys_in_a_pane_of_no_rows() {
    let mut app = numbered(5);
    draw_sized(&mut app, 2);
    assert_eq!(
        (
            press(&mut app, key(KeyCode::PageUp)),
            press(&mut app, key(KeyCode::PageDown))
        ),
        (vec![], vec![])
    );
    type_text(&mut app, "hi");
    let rows = text_rows(&draw_sized(&mut app, 2));
    assert_eq!(
        (rows[0].starts_with("[alpha/aria]"), rows[1].as_str()),
        (true, "hi")
    );
}

/// A mouse report of the left button at the cell `column`, `row`.
fn left(app: &mut Model, kind: MouseEventKind, column: u16, row: u16) -> Vec<Effect> {
    let mouse = MouseEvent {
        kind,
        column,
        row,
        modifiers: Modifiers::NONE,
    };
    update(app, Msg::Terminal(TermEvent::Mouse(mouse)))
}

/// The number of reversed cells of the pane, with no cell of the top row or the input line.
fn reversed_cells(app: &mut Model) -> usize {
    let buffer = draw(app);
    (1..23)
        .flat_map(|y| (0..80).map(move |x| (x, y)))
        .filter(|&at| buffer[at].modifier.contains(Modifier::REVERSED))
        .count()
}

#[test]
fn click_clears_selection() {
    let mut app = app(&["alpha/aria"]);
    on_event(&mut app, game("alpha/aria", "You say, \"hello\""));
    draw(&mut app);
    left(&mut app, MouseEventKind::Down(MouseButton::Left), 10, 1);
    left(&mut app, MouseEventKind::Drag(MouseButton::Left), 14, 1);
    let copied = left(&mut app, MouseEventKind::Up(MouseButton::Left), 14, 1);
    assert_eq!(
        (copied, reversed_cells(&mut app)),
        (vec![Effect::ClipboardWrite("hello".to_owned())], 5)
    );
    left(&mut app, MouseEventKind::Down(MouseButton::Left), 3, 1);
    let clicked = left(&mut app, MouseEventKind::Up(MouseButton::Left), 3, 1);
    assert_eq!((clicked, reversed_cells(&mut app)), (vec![], 0));
}

#[derive(Clone, Debug)]
enum Step {
    Next,
    Previous,
    Enter,
    Line(usize),
    Reset,
    Lost,
}

const IDS: [&str; 4] = ["alpha/aria", "beta/bryn", "alpha-2/cy", "alpha/bea"];

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        Just(Step::Next),
        Just(Step::Previous),
        Just(Step::Enter),
        (0..IDS.len()).prop_map(Step::Line),
        Just(Step::Reset),
        Just(Step::Lost),
    ]
}

proptest! {
    #[test]
    fn focus_is_always_valid(start in 0..=IDS.len(), steps in proptest::collection::vec(step(), 0..40)) {
        let mut app = app(&IDS[..start]);
        for step in steps {
            match step {
                Step::Next => { press(&mut app, ctrl('n')); }
                Step::Previous => { press(&mut app, ctrl('p')); }
                Step::Enter => { press(&mut app, key(KeyCode::Enter)); }
                Step::Line(n) => on_event(&mut app, game(IDS[n], "line")),
                Step::Reset => on_event(&mut app, ClientEvent::Reset { instance_id: "i".to_owned() }),
                Step::Lost => on_event(&mut app, ClientEvent::Lost),
            }
            let ids = app.world_ids();
            match app.focused() {
                Some(focus) => prop_assert!(ids.contains(&focus)),
                None => prop_assert!(ids.is_empty()),
            }
        }
    }
}

// Ctrl+M, a send of each line, and a join in place.

fn send(world: &str, text: &str) -> Effect {
    Effect::Send {
        world: world.to_owned(),
        text: text.to_owned(),
    }
}

fn refused(world: &str, text: &str) -> Reply {
    Reply::Refused {
        world: world.to_owned(),
        text: text.to_owned(),
        reason: format!("{world} is not connected"),
    }
}

#[test]
fn ctrl_m_sends() {
    let mut app = two_worlds();
    press(&mut app, ctrl('n'));
    type_text(&mut app, "hi");
    assert_eq!(press(&mut app, ctrl('m')), [send("beta/bryn", "hi")]);
    assert_eq!(app.input(), "");
}

/// The reply that the server took `text` in `world`, and the effects of it.
fn sent(app: &mut Model, world: &str, text: &str) -> Vec<Effect> {
    let reply = Reply::Sent {
        world: world.to_owned(),
        text: text.to_owned(),
    };
    update(app, Msg::Reply(reply))
}

fn alt_enter() -> KeyEvent {
    KeyEvent::new(KeyCode::Enter, Modifiers::ALT)
}

fn alt_r() -> KeyEvent {
    KeyEvent::new(KeyCode::Char('r'), Modifiers::ALT)
}

/// True if the screen shows `text`, also across a wrap. The rows join with a space.
fn shows(app: &mut Model, text: &str) -> bool {
    screen(app)
        .iter()
        .map(|row| row.trim_end())
        .collect::<Vec<_>>()
        .join(" ")
        .contains(text)
}

/// True if a row of the screen is a notice that holds `text`.
fn has_notice(app: &mut Model, text: &str) -> bool {
    screen(app)
        .iter()
        .any(|row| row.starts_with("-- ") && row.contains(text))
}

#[test]
fn each_line_is_a_command() {
    for modifiers in [Modifiers::ALT, Modifiers::CONTROL] {
        let mut app = two_worlds();
        type_lines(&mut app, &["one", "", "two"]);
        let enter = KeyEvent::new(KeyCode::Enter, modifiers);
        assert_eq!(press(&mut app, enter), [send("alpha/aria", "one")]);
        assert_eq!(app.input(), "");
        assert_eq!(
            sent(&mut app, "alpha/aria", "one"),
            [send("alpha/aria", "")]
        );
        assert_eq!(
            sent(&mut app, "alpha/aria", ""),
            [send("alpha/aria", "two")]
        );
        assert_eq!(sent(&mut app, "alpha/aria", "two"), []);
    }
}

#[test]
fn focus_moves_between_two_lines() {
    let mut app = two_worlds();
    type_lines(&mut app, &["one", "two"]);
    assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "one")]);
    press(&mut app, ctrl('n'));
    assert_eq!(
        sent(&mut app, "alpha/aria", "one"),
        [send("alpha/aria", "two")]
    );
}

#[test]
fn a_line_that_the_server_refuses() {
    let mut app = two_worlds();
    type_lines(&mut app, &["one", "two", "three"]);
    assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "one")]);
    assert_eq!(
        sent(&mut app, "alpha/aria", "one"),
        [send("alpha/aria", "two")]
    );
    on_reply(&mut app, refused("alpha/aria", "two"));
    assert_eq!(app.input(), "two");
    assert_eq!(sent(&mut app, "alpha/aria", "two"), []);
    assert!(has_notice(&mut app, "not connected (1 in history)"));
    up(&mut app);
    assert_eq!(app.input(), "three");
    up(&mut app);
    assert_eq!(app.input(), "one\ntwo\nthree");
}

#[test]
fn the_last_line_is_refused() {
    let mut app = two_worlds();
    type_lines(&mut app, &["one", "two"]);
    assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "one")]);
    assert_eq!(
        sent(&mut app, "alpha/aria", "one"),
        [send("alpha/aria", "two")]
    );
    type_text(&mut app, "later");
    on_reply(&mut app, refused("alpha/aria", "two"));
    assert_eq!(app.input(), "two");
    up(&mut app);
    assert_eq!(app.input(), "later");
}

#[test]
fn sends_to_a_world_go_one_at_a_time() {
    let mut app = two_worlds();
    type_text(&mut app, "one");
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "one")]
    );
    // The second send waits for the reply of the first one, and the editor is empty.
    type_text(&mut app, "two");
    assert_eq!(press(&mut app, key(KeyCode::Enter)), []);
    assert_eq!(app.input(), "");
    assert_eq!(
        sent(&mut app, "alpha/aria", "one"),
        [send("alpha/aria", "two")]
    );
    assert_eq!(sent(&mut app, "alpha/aria", "two"), []);
}

#[test]
fn send_goes_after_the_lines_that_wait() {
    for second in [key(KeyCode::Enter), alt_enter()] {
        let mut app = two_worlds();
        type_lines(&mut app, &["one", "two"]);
        assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "one")]);
        type_text(&mut app, "next");
        assert_eq!(press(&mut app, second), []);
        assert_eq!(app.input(), "");
        assert_eq!(
            sent(&mut app, "alpha/aria", "one"),
            [send("alpha/aria", "two")]
        );
        assert_eq!(
            sent(&mut app, "alpha/aria", "two"),
            [send("alpha/aria", "next")]
        );
        assert_eq!(sent(&mut app, "alpha/aria", "next"), []);
    }
}

#[test]
fn lines_go_after_a_send_with_the_same_text() {
    let mut app = two_worlds();
    type_text(&mut app, "one");
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "one")]
    );
    type_lines(&mut app, &["one", "two"]);
    assert_eq!(press(&mut app, alt_enter()), []);
    // Each reply moves the sends on by one, also if two sends hold the same text.
    assert_eq!(
        sent(&mut app, "alpha/aria", "one"),
        [send("alpha/aria", "one")]
    );
    assert_eq!(
        sent(&mut app, "alpha/aria", "one"),
        [send("alpha/aria", "two")]
    );
    assert_eq!(sent(&mut app, "alpha/aria", "two"), []);
}

#[test]
fn send_to_another_world_does_not_wait() {
    let mut app = two_worlds();
    type_text(&mut app, "before");
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "before")]
    );
    press(&mut app, ctrl('n'));
    type_lines(&mut app, &["one", "two"]);
    assert_eq!(press(&mut app, alt_enter()), [send("beta/bryn", "one")]);
    // The reply of one world does not move the sends of the other.
    assert_eq!(sent(&mut app, "alpha/aria", "before"), []);
    assert_eq!(
        sent(&mut app, "beta/bryn", "one"),
        [send("beta/bryn", "two")]
    );
}

#[test]
fn a_refused_send_of_enter_with_a_send_after_it() {
    let mut app = two_worlds();
    type_text(&mut app, "one");
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "one")]
    );
    type_text(&mut app, "two");
    assert_eq!(press(&mut app, key(KeyCode::Enter)), []);
    type_text(&mut app, "later");
    on_reply(&mut app, refused("alpha/aria", "one"));
    assert_eq!(app.input(), "one");
    assert!(has_notice(&mut app, "not connected (2 in history)"));
    up(&mut app);
    assert_eq!(app.input(), "two");
    up(&mut app);
    assert_eq!(app.input(), "later");
    // No send of the world waits now, so a new send goes at once.
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "later")]
    );
}

#[test]
fn the_sends_stay_apart() {
    let mut app = two_worlds();
    type_lines(&mut app, &["a", "b"]);
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "a\nb")]
    );
    type_text(&mut app, "c");
    assert_eq!(press(&mut app, key(KeyCode::Enter)), []);
    on_reply(&mut app, refused("alpha/aria", "a\nb"));
    assert_eq!(app.input(), "a\nb");
    // The returned `c` equals the newest entry, so the history keeps no new entry.
    assert!(has_notice(&mut app, "not connected"));
    assert!(!has_notice(&mut app, "in history"));
    up(&mut app);
    assert_eq!(app.input(), "c");
}

#[test]
fn hint_does_not_come_back_after_a_reset() {
    let mut app = listed();
    press(&mut app, ctrl('n'));
    type_lines(&mut app, &["one", "two"]);
    press(&mut app, alt_r());
    assert!(has_notice(&mut app, "sets no line break"));
    on_event(
        &mut app,
        ClientEvent::Reset {
            instance_id: "i".to_owned(),
        },
    );
    assert!(!has_notice(&mut app, "sets no line break"));
}

#[test]
fn line_break_that_the_editor_cannot_hold_is_no_line_break() {
    for line_break in ["", "\u{1b}[2J", "a\nb"] {
        let world = World {
            id: "alpha/aria".to_owned(),
            line_break: Some(line_break.to_owned()),
            ..Default::default()
        };
        let mut app = after_list(Ok(vec![world]));
        type_lines(&mut app, &["one", "two"]);
        press(&mut app, alt_r());
        assert_eq!(app.input(), "one\ntwo", "{line_break:?}");
    }
}

/// A model from a list of worlds: `alpha/aria` with the line break `%r`, and `beta/bryn` with
/// none.
fn listed() -> Model {
    let world = |id: &str, line_break: Option<&str>| World {
        id: id.to_owned(),
        line_break: line_break.map(str::to_owned),
        ..Default::default()
    };
    after_list(Ok(vec![
        world("alpha/aria", Some("%r")),
        world("beta/bryn", None),
    ]))
}

#[test]
fn lines_become_one_line() {
    let mut app = listed();
    type_lines(&mut app, &["one", "two"]);
    assert_eq!(press(&mut app, alt_r()), []);
    assert_eq!(app.input(), "one%rtwo");
    assert_eq!(press(&mut app, ctrl('z')), []);
    assert_eq!(press(&mut app, ctrl('z')), []);
    assert_eq!(app.input(), "one\ntwo");
}

#[test]
fn join_with_no_line_break() {
    let mut app = listed();
    press(&mut app, ctrl('n'));
    type_lines(&mut app, &["one", "two"]);
    assert_eq!(press(&mut app, alt_r()), []);
    assert_eq!(app.input(), "one\ntwo");
    let screen = screen(&mut app);
    assert!(
        screen
            .iter()
            .any(|row| row.starts_with("-- ") && row.contains("line break")),
        "{screen:?}"
    );
    assert_eq!(screen[22..], ["one", "two"]);
}

// Sends that wait: the count, a lost server, and a quit.

/// Alpha with the lines `one`, `two` and `three` sent by Alt+Enter: `one` is with the server,
/// and `two` and `three` wait.
fn three_lines_sent() -> Model {
    let mut app = two_worlds();
    type_lines(&mut app, &["one", "two", "three"]);
    assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "one")]);
    app
}

#[test]
fn top_row_counts_the_sends_that_wait() {
    let mut app = three_lines_sent();
    assert_eq!(top(&mut app), "[alpha/aria] (2 waiting) beta/bryn*");
    sent(&mut app, "alpha/aria", "one");
    assert_eq!(top(&mut app), "[alpha/aria] (1 waiting) beta/bryn*");
    sent(&mut app, "alpha/aria", "two");
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn*");
}

#[test]
fn the_stream_ends_while_lines_wait() {
    let mut app = three_lines_sent();
    type_text(&mut app, "later");
    on_event(&mut app, ClientEvent::Lost);
    assert_eq!(app.input(), "two");
    assert!(has_notice(&mut app, "2 lines that waited went back"));
    assert!(screen(&mut app).join(" ").contains("(2 in history)"));
    up(&mut app);
    assert_eq!(app.input(), "three");
    up(&mut app);
    assert_eq!(app.input(), "later");
    // The send with the server stays: its reply sends nothing more.
    assert_eq!(sent(&mut app, "alpha/aria", "one"), []);
}

#[test]
fn the_server_resets_while_lines_wait() {
    let mut app = three_lines_sent();
    on_event(
        &mut app,
        ClientEvent::Reset {
            instance_id: "i".to_owned(),
        },
    );
    assert_eq!(app.input(), "two");
    assert!(has_notice(&mut app, "the server started again"));
    up(&mut app);
    assert_eq!(app.input(), "three");
}

#[test]
fn a_refusal_after_the_stream_ended() {
    let mut app = three_lines_sent();
    on_event(&mut app, ClientEvent::Lost);
    on_reply(&mut app, refused("alpha/aria", "one"));
    assert_eq!(app.input(), "one");
    up(&mut app);
    assert_eq!(app.input(), "two");
    up(&mut app);
    assert_eq!(app.input(), "three");
}

#[test]
fn quit_with_waiting_sends_asks_again() {
    let mut app = three_lines_sent();
    assert_eq!(press(&mut app, ctrl('c')), []);
    assert!(has_notice(&mut app, "3 lines not sent yet"));
    assert_eq!(press(&mut app, ctrl('c')), [Effect::Quit]);
}

#[test]
fn another_key_cancels_the_second_quit() {
    let mut app = three_lines_sent();
    press(&mut app, ctrl('c'));
    type_text(&mut app, "x");
    assert_eq!(press(&mut app, ctrl('c')), []);
}

#[test]
fn quit_with_no_waiting_send_leaves_at_once() {
    let mut app = three_lines_sent();
    sent(&mut app, "alpha/aria", "one");
    sent(&mut app, "alpha/aria", "two");
    sent(&mut app, "alpha/aria", "three");
    assert_eq!(press(&mut app, ctrl('c')), [Effect::Quit]);
}

// Lists of the worlds and the line break, then more cases of the sends that wait, the quit
// and a paste during the probe.

/// A client whose first list failed, with `alpha/aria` from its first event.
fn unlisted_world() -> Model {
    let mut app = failed_listing();
    let effects = update(&mut app, Msg::Client(game("alpha/aria", "Welcome, aria.")));
    assert_eq!(effects, [Effect::ListWorlds]);
    app
}

fn alpha_with_line_break() -> Vec<World> {
    vec![World {
        id: "alpha/aria".to_owned(),
        line_break: Some("%r".to_owned()),
        ..Default::default()
    }]
}

#[test]
fn list_takes_the_line_break_of_a_new_world() {
    let mut app = unlisted_world();
    assert_eq!(
        update(&mut app, Msg::Listed(Ok(alpha_with_line_break()))),
        []
    );
    type_lines(&mut app, &["one", "two"]);
    press(&mut app, alt_r());
    assert_eq!(app.input(), "one%rtwo");
}

#[test]
fn unknown_line_break_asks_the_server() {
    let mut app = unlisted_world();
    update(&mut app, Msg::Listed(Err("no server".to_owned())));
    type_lines(&mut app, &["one", "two"]);
    assert_eq!(press(&mut app, alt_r()), [Effect::ListWorlds]);
    assert!(has_notice(&mut app, "does not know the line break"));
    assert!(!has_notice(&mut app, "sets no line break"));
    assert_eq!(app.input(), "one\ntwo");
}

#[test]
fn one_list_at_a_time() {
    let mut app = unlisted_world();
    // The list of the first event is still with the server.
    assert_eq!(press(&mut app, alt_r()), []);
    on_event(&mut app, game("beta/bryn", "Welcome, bryn."));
    assert_eq!(update(&mut app, Msg::Client(game("gamma/cora", "hi"))), []);
}

#[test]
fn reset_asks_for_the_list() {
    let mut app = listed();
    let reset = ClientEvent::Reset {
        instance_id: "i".to_owned(),
    };
    assert_eq!(update(&mut app, Msg::Client(reset)), [Effect::ListWorlds]);
}

#[test]
fn paste_during_the_probe_fills_the_editor() {
    let mut app = two_worlds();
    let mut pending = Vec::new();
    let mut skip = mu_tui::probe::Skip::Nothing;
    let framed = mu_tui::probe::frame(&mut pending, &mut skip, b"\x1b[200~one\rtwo\x1b[201~");
    for event in framed.events {
        assert_eq!(update(&mut app, Msg::Terminal(event)), []);
    }
    assert_eq!(app.input(), "one\ntwo");
}

#[test]
fn alt_r_with_no_world_names_the_line_break() {
    let mut app = failed_listing();
    type_lines(&mut app, &["one", "two"]);
    assert_eq!(press(&mut app, alt_r()), []);
    assert!(has_notice(&mut app, "line break"));
    assert_eq!(app.input(), "one\ntwo");
}

#[test]
fn quit_with_only_the_line_with_the_server_asks_again() {
    let mut app = two_worlds();
    type_text(&mut app, "one");
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "one")]
    );
    assert_eq!(press(&mut app, ctrl('c')), []);
    assert!(has_notice(&mut app, "1 line not sent yet"));
    assert_eq!(press(&mut app, ctrl('c')), [Effect::Quit]);
}

#[test]
fn lines_of_two_worlds_wait() {
    // Each model has its own map of sends, so the 32 runs very likely see both orders of a hash
    // map.
    for _ in 0..32 {
        let mut app = two_worlds();
        press(&mut app, ctrl('n'));
        type_lines(&mut app, &["b1", "b2"]);
        assert_eq!(press(&mut app, alt_enter()), [send("beta/bryn", "b1")]);
        press(&mut app, ctrl('p'));
        type_lines(&mut app, &["a1", "a2"]);
        assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "a1")]);
        on_event(&mut app, ClientEvent::Lost);
        assert_eq!(app.input(), "a2");
        assert!(shows(&mut app, "1 line of alpha/aria, 1 line of beta/bryn"));
        // Each editor was empty, and nothing went into history, so the hint has no count.
        assert!(!shows(&mut app, "in history"), "{:?}", screen(&mut app));
        press(&mut app, ctrl('n'));
        assert_eq!(app.input(), "b2");
    }
}

#[test]
fn notice_of_the_returned_lines_does_not_come_back_after_a_reset() {
    let mut app = three_lines_sent();
    on_event(&mut app, ClientEvent::Lost);
    assert!(has_notice(&mut app, "lines that waited"));
    let reset = ClientEvent::Reset {
        instance_id: "i".to_owned(),
    };
    on_event(&mut app, reset);
    // The reset clears the pane and does not bring hints back, and nothing waits any more, so no
    // line about returned lines is on the screen.
    assert!(!has_notice(&mut app, "lines that waited"));
}

#[test]
fn reset_during_a_list_asks_again_after_its_reply() {
    // The new world asked for a list, and its reply did not come yet.
    let mut app = unlisted_world();
    let reset = ClientEvent::Reset {
        instance_id: "i".to_owned(),
    };
    on_event(&mut app, reset);
    let after = update(&mut app, Msg::Listed(Err("old server".to_owned())));
    assert_eq!(after, [Effect::ListWorlds]);
}

#[test]
fn failed_list_names_its_reason_at_alt_r() {
    let mut app = unlisted_world();
    assert_eq!(
        update(
            &mut app,
            Msg::Listed(Err("unavailable: no server".to_owned()))
        ),
        []
    );
    type_lines(&mut app, &["one", "two"]);
    assert_eq!(press(&mut app, alt_r()), [Effect::ListWorlds]);
    assert!(shows(
        &mut app,
        "the last list of the worlds failed (unavailable: no server)"
    ));
}

#[test]
fn list_adds_a_world_that_the_client_did_not_hold() {
    let mut app = failed_listing();
    let beta = World {
        id: "beta/bryn".to_owned(),
        ..Default::default()
    };
    assert_eq!(update(&mut app, Msg::Listed(Ok(vec![beta]))), []);
    assert!(top(&mut app).contains("beta/bryn"));
}

// Alt and a digit give the focus by place, and Alt+Home and Alt+End scroll to the ends.

fn alt(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, Modifiers::ALT)
}

#[test]
fn the_second_world() {
    let mut app = two_worlds();
    press(&mut app, alt(KeyCode::Char('2')));
    assert_eq!(top(&mut app), "alpha/aria [beta/bryn]");
}

#[test]
fn no_world_at_that_place() {
    let mut app = two_worlds();
    assert_eq!(press(&mut app, alt(KeyCode::Char('3'))), []);
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn*");
}

#[test]
fn back_to_the_first_world() {
    let mut app = two_worlds();
    press(&mut app, ctrl('n'));
    press(&mut app, alt(KeyCode::Char('1')));
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn");
}

#[test]
fn top_and_bottom() {
    let mut app = numbered(40);
    press(&mut app, alt(KeyCode::Home));
    let first = pane_rows(&mut app)[0].clone();
    press(&mut app, alt(KeyCode::End));
    assert_eq!(
        (first.as_str(), pane_rows(&mut app).last().unwrap().as_str()),
        ("1", "40")
    );
}

// Client commands: a text of one line that starts with `/`.

/// Types `text` and presses Enter.
fn enter(app: &mut Model, text: &str) -> Vec<Effect> {
    type_text(app, text);
    press(app, key(KeyCode::Enter))
}

#[test]
fn a_command_does_not_reach_the_game() {
    let mut app = two_worlds();
    assert_eq!(enter(&mut app, "/focus-next"), []);
    assert_eq!(app.input(), "");
}

#[test]
fn several_lines_are_not_a_command() {
    let mut app = two_worlds();
    type_lines(&mut app, &["/quit", "two"]);
    assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "/quit")]);
    assert_eq!(
        sent(&mut app, "alpha/aria", "/quit"),
        [send("alpha/aria", "two")]
    );
}

#[test]
fn an_escaped_slash() {
    let mut app = two_worlds();
    assert_eq!(
        enter(&mut app, "//me waves"),
        [send("alpha/aria", "/me waves")]
    );
}

/// The editor text and whether a notice holds `text`, after `command` and Enter.
fn bad_command(app: &mut Model, command: &str, text: &str) -> (String, bool) {
    let effects = enter(app, command);
    assert_eq!(effects, []);
    (app.input(), has_notice(app, text))
}

#[test]
fn an_unknown_command() {
    let mut app = two_worlds();
    assert_eq!(
        bad_command(&mut app, "/fly", "unknown command"),
        ("/fly".to_owned(), true)
    );
}

#[test]
fn a_missing_world() {
    let mut app = two_worlds();
    assert_eq!(
        bad_command(&mut app, "/focus", "needs a world"),
        ("/focus".to_owned(), true)
    );
}

#[test]
fn an_unknown_world() {
    let mut app = two_worlds();
    assert_eq!(
        bad_command(&mut app, "/focus gamma/gil", "gamma/gil"),
        ("/focus gamma/gil".to_owned(), true)
    );
}

#[test]
fn an_argument_to_an_entry_that_takes_none() {
    let mut app = two_worlds();
    assert_eq!(
        bad_command(&mut app, "/quit now", "takes no argument"),
        ("/quit now".to_owned(), true)
    );
}

#[test]
fn an_editing_entry_is_not_a_command() {
    let mut app = two_worlds();
    assert_eq!(
        bad_command(&mut app, "/undo", "not a command"),
        ("/undo".to_owned(), true)
    );
}

#[test]
fn connect_with_no_world_in_focus() {
    let mut app = app(&[]);
    assert_eq!(
        bad_command(&mut app, "/connect", "no world"),
        ("/connect".to_owned(), true)
    );
}

#[test]
fn a_command_with_no_world_in_focus() {
    let mut app = app(&[]);
    assert_eq!(enter(&mut app, "/help"), []);
    assert!(
        screen(&mut app)
            .iter()
            .any(|row| row.starts_with("-- ") && row.contains("connect"))
    );
}

#[test]
fn focus_by_name() {
    let mut app = two_worlds();
    enter(&mut app, "/focus beta/bryn");
    assert_eq!(top(&mut app), "alpha/aria [beta/bryn]");
}

#[test]
fn a_world_with_a_trailing_space() {
    let mut app = two_worlds();
    enter(&mut app, "/focus beta/bryn ");
    assert_eq!(top(&mut app), "alpha/aria [beta/bryn]");
}

#[test]
fn help_lists_the_entries() {
    let mut app = two_worlds();
    assert_eq!(enter(&mut app, "/help"), []);
    // The screen is high enough for one row for each entry.
    let rows = text_rows(&draw_sized(&mut app, 80));
    let notice = |name: &str| {
        rows.iter()
            .any(|row| row.starts_with("-- ") && row.contains(name))
    };
    assert_eq!((notice("connect"), notice("focus-next")), (true, true));
}

#[test]
fn a_key_runs_connect_on_the_world_in_focus() {
    let app = two_worlds();
    let mut keymap = mu_tui::Keymap::default();
    keymap.bind(
        mu_tui::Chord::with(KeyCode::Char('c'), Modifiers::ALT),
        mu_tui::EntryId::named("connect").unwrap(),
    );
    let mut app = app.with_keymap(keymap);
    assert_eq!(
        press(&mut app, alt(KeyCode::Char('c'))),
        [Effect::Connect {
            world: "alpha/aria".to_owned()
        }]
    );
}

// The input history of each world, and its editor.

/// Types `text` in the world in focus and presses Enter, and the server takes the send.
fn send_and_take(app: &mut Model, world: &str, text: &str) {
    type_text(app, text);
    let effects = press(app, key(KeyCode::Enter));
    assert_eq!(effects, [send(world, &text.replace("//", "/"))]);
    assert_eq!(sent(app, world, &text.replace("//", "/")), []);
}

#[test]
fn a_send_enters_the_history() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "one");
    up(&mut app);
    assert_eq!(app.input(), "one");
}

#[test]
fn a_pose_is_one_entry() {
    let mut app = two_worlds();
    type_lines(&mut app, &["a", "b"]);
    assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "a")]);
    up(&mut app);
    assert_eq!(app.input(), "a\nb");
}

#[test]
fn no_empty_entry() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "one");
    press(&mut app, key(KeyCode::Enter));
    up(&mut app);
    assert_eq!(app.input(), "one");
}

#[test]
fn each_world_has_its_own_history() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "one");
    press(&mut app, ctrl('n'));
    up(&mut app);
    assert_eq!(app.input(), "");
}

#[test]
fn no_repeat() {
    let mut app = two_worlds();
    for text in ["one", "one", "two"] {
        send_and_take(&mut app, "alpha/aria", text);
    }
    assert_eq!(app.history("alpha/aria"), Some(vec!["one", "two"]));
    up(&mut app);
    up(&mut app);
    assert_eq!(app.input(), "one");
    up(&mut app);
    assert_eq!(app.input(), "one");
}

#[test]
fn an_escaped_slash_in_the_history() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "//me waves");
    up(&mut app);
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "/me waves")]
    );
}

#[test]
fn down_with_no_walk() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "one");
    type_text(&mut app, "draft");
    down(&mut app);
    assert_eq!(app.input(), "draft");
    up(&mut app);
    down(&mut app);
    down(&mut app);
    assert_eq!(app.input(), "draft");
}

#[test]
fn up_on_an_empty_history() {
    let mut app = two_worlds();
    type_text(&mut app, "draft");
    up(&mut app);
    assert_eq!(app.input(), "draft");
}

#[test]
fn back_to_the_draft() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "one");
    type_text(&mut app, "draft");
    up(&mut app);
    assert_eq!(app.input(), "one");
    down(&mut app);
    assert_eq!(app.input(), "draft");
}

#[test]
fn up_inside_a_pose_moves_the_cursor() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "one");
    type_lines(&mut app, &["a", "b"]);
    up(&mut app);
    assert_eq!(app.input(), "a\nb");
    // The cursor is after `a` now, so the next char goes there.
    type_text(&mut app, "x");
    assert_eq!(app.input(), "ax\nb");
}

#[test]
fn shift_up_is_not_a_history_key() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "one");
    type_text(&mut app, "draft");
    for (code, modifiers) in [
        (KeyCode::Up, Modifiers::SHIFT),
        (KeyCode::Up, Modifiers::CONTROL),
        (KeyCode::Down, Modifiers::SHIFT),
    ] {
        press(&mut app, KeyEvent::new(code, modifiers));
        assert_eq!(app.input(), "draft", "{code:?} {modifiers:?}");
    }
}

#[test]
fn a_focus_change_keeps_each_editor() {
    let mut app = two_worlds();
    type_text(&mut app, "draft");
    press(&mut app, ctrl('n'));
    assert_eq!(app.input(), "");
    type_text(&mut app, "other");
    press(&mut app, ctrl('p'));
    assert_eq!(app.input(), "draft");
    press(&mut app, ctrl('n'));
    assert_eq!(app.input(), "other");
}

#[test]
fn a_walk_stays_with_its_world() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "one");
    type_text(&mut app, "draft");
    up(&mut app);
    press(&mut app, ctrl('n'));
    press(&mut app, ctrl('p'));
    assert_eq!(app.input(), "one");
    down(&mut app);
    assert_eq!(app.input(), "draft");
}

#[test]
fn a_send_during_a_walk() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "one");
    type_text(&mut app, "draft");
    up(&mut app);
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "one")]
    );
    up(&mut app);
    up(&mut app);
    assert_eq!(app.input(), "draft");
}

#[test]
fn a_world_added_during_a_walk() {
    let mut app = app(&["beta/bryn"]);
    send_and_take(&mut app, "beta/bryn", "one");
    type_text(&mut app, "draft");
    up(&mut app);
    on_event(&mut app, game("alpha/aria", "Welcome, aria."));
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("beta/bryn", "one")]
    );
    assert_eq!(app.history("beta/bryn").unwrap(), ["one", "draft", "one"]);
    assert_eq!(app.history("alpha/aria").unwrap(), [] as [&str; 0]);
}

#[test]
fn a_refusal_during_a_walk_of_the_history() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "old");
    type_text(&mut app, "one");
    press(&mut app, key(KeyCode::Enter));
    type_text(&mut app, "draft");
    up(&mut app);
    on_reply(&mut app, refused("alpha/aria", "one"));
    assert_eq!(app.input(), "one");
    assert!(has_notice(&mut app, "(1 in history)"));
    let history = app.history("alpha/aria").unwrap();
    assert_eq!(history.iter().filter(|entry| **entry == "draft").count(), 1);
    up(&mut app);
    assert_eq!(app.input(), "draft");
}

#[test]
fn a_refusal_out_of_focus() {
    let mut app = two_worlds();
    press(&mut app, ctrl('n'));
    type_lines(&mut app, &["b1", "b2"]);
    assert_eq!(press(&mut app, alt_enter()), [send("beta/bryn", "b1")]);
    press(&mut app, ctrl('p'));
    type_text(&mut app, "typed");
    on_reply(&mut app, refused("beta/bryn", "b1"));
    assert_eq!(app.input(), "typed");
    assert_eq!(app.history("alpha/aria").unwrap(), [] as [&str; 0]);
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn*");
    press(&mut app, ctrl('n'));
    assert!(has_notice(&mut app, "(1 in history)"));
    assert_eq!(app.input(), "b1");
    up(&mut app);
    assert_eq!(app.input(), "b2");
}

#[test]
fn a_refusal_out_of_focus_during_a_walk() {
    let mut app = two_worlds();
    send_and_take(&mut app, "alpha/aria", "old");
    type_text(&mut app, "one");
    press(&mut app, key(KeyCode::Enter));
    type_text(&mut app, "draft");
    up(&mut app);
    press(&mut app, ctrl('n'));
    on_reply(&mut app, refused("alpha/aria", "one"));
    assert_eq!(app.input(), "");
    press(&mut app, ctrl('p'));
    assert_eq!(app.input(), "one");
    assert!(has_notice(&mut app, "(1 in history)"));
    up(&mut app);
    assert_eq!(app.input(), "draft");
}

proptest! {
    #[test]
    fn history_walk_returns_the_draft(
        entries in proptest::collection::vec("[a-z]{1,6}", 0..6),
        draft in "[a-z]{0,6}",
        steps in 0usize..10,
    ) {
        let mut app = app(&["alpha/aria"]);
        for entry in &entries {
            send_and_take(&mut app, "alpha/aria", entry);
        }
        type_text(&mut app, &draft);
        for _ in 0..steps {
            up(&mut app);
        }
        for _ in 0..steps {
            down(&mut app);
        }
        prop_assert_eq!(app.input(), draft);
    }

    #[test]
    fn returned_texts_come_back_in_order(
        texts in proptest::collection::hash_set("[a-z]{1,5}", 2..6),
        held in "[A-Z]{1,5}",
    ) {
        let texts: Vec<String> = texts.into_iter().collect();
        let mut app = two_worlds();
        let lines: Vec<&str> = texts.iter().map(String::as_str).collect();
        type_lines(&mut app, &lines);
        assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", &texts[0])]);
        type_text(&mut app, &held);
        on_reply(&mut app, refused("alpha/aria", &texts[0]));
        prop_assert_eq!(app.input(), texts[0].clone());
        let in_history = format!("({} in history)", texts.len());
        prop_assert!(has_notice(&mut app, &in_history));
        for text in texts[1..].iter().chain([&held]) {
            up(&mut app);
            prop_assert_eq!(&app.input(), text);
        }
    }
}

#[test]
fn returned_texts_past_the_limit_keep_the_draft() {
    let texts: Vec<String> = (0..=HISTORY_LIMIT + 100).map(|n| format!("t{n}")).collect();
    let mut app = two_worlds();
    let lines: Vec<&str> = texts.iter().map(String::as_str).collect();
    type_lines(&mut app, &lines);
    assert_eq!(
        press(&mut app, alt_enter()),
        [send("alpha/aria", &texts[0])]
    );
    type_text(&mut app, "DRAFT");
    on_reply(&mut app, refused("alpha/aria", &texts[0]));
    assert_eq!(app.input(), texts[0]);
    assert!(has_notice(
        &mut app,
        &format!("({HISTORY_LIMIT} in history)")
    ));
    let history = app.history("alpha/aria").unwrap();
    assert_eq!(history.len(), HISTORY_LIMIT);
    assert_eq!(history[0], "DRAFT");
    assert_eq!(history[HISTORY_LIMIT - 1], texts[1]);
}

#[derive(Clone, Debug)]
enum HistoryStep {
    Type(String),
    Newline,
    Enter,
    Up,
    Down,
    Next,
    Previous,
    Accept(usize),
    Refuse(usize),
}

impl HistoryStep {
    /// True for a key that can change the editor of the world in focus.
    const fn edits(&self) -> bool {
        matches!(
            self,
            Self::Type(_) | Self::Newline | Self::Enter | Self::Up | Self::Down
        )
    }
}

fn history_step() -> impl Strategy<Value = HistoryStep> {
    prop_oneof![
        "[a-c]{1,2}".prop_map(HistoryStep::Type),
        Just(HistoryStep::Newline),
        Just(HistoryStep::Enter),
        Just(HistoryStep::Up),
        Just(HistoryStep::Down),
        Just(HistoryStep::Next),
        Just(HistoryStep::Previous),
        any::<usize>().prop_map(HistoryStep::Accept),
        any::<usize>().prop_map(HistoryStep::Refuse),
    ]
}

/// The sends of `effects`, as world and text.
fn sends_of(effects: Vec<Effect>) -> Vec<(String, String)> {
    effects
        .into_iter()
        .filter_map(|effect| match effect {
            Effect::Send { world, text } => Some((world, text)),
            _ => None,
        })
        .collect()
}

proptest! {
    #[test]
    fn history_keeps_its_invariants(steps in proptest::collection::vec(history_step(), 0..60)) {
        let mut app = two_worlds();
        let ids = ["alpha/aria", "beta/bryn"];
        // The sends with the server, as world and text.
        let mut with_server: Vec<(String, String)> = Vec::new();
        for step in steps {
            let before: Vec<Option<String>> = ids.iter().map(|id| app.input_of(id)).collect();
            let focused = app.focused().map(str::to_owned);
            let mut put_back = None;
            match &step {
                HistoryStep::Type(text) => type_text(&mut app, text),
                HistoryStep::Newline => { press(&mut app, ctrl('j')); }
                HistoryStep::Enter => {
                    with_server.extend(sends_of(press(&mut app, key(KeyCode::Enter))));
                }
                HistoryStep::Up => { press(&mut app, key(KeyCode::Up)); }
                HistoryStep::Down => { press(&mut app, key(KeyCode::Down)); }
                HistoryStep::Next => { press(&mut app, ctrl('n')); }
                HistoryStep::Previous => { press(&mut app, ctrl('p')); }
                HistoryStep::Accept(pick) if !with_server.is_empty() => {
                    let (world, text) = with_server.remove(pick % with_server.len());
                    with_server.extend(sends_of(sent(&mut app, &world, &text)));
                }
                HistoryStep::Refuse(pick) if !with_server.is_empty() => {
                    let (world, text) = with_server.remove(pick % with_server.len());
                    on_reply(&mut app, refused(&world, &text));
                    put_back = Some(world);
                }
                HistoryStep::Accept(_) | HistoryStep::Refuse(_) => {}
            }
            for (id, before) in ids.iter().zip(before) {
                let history = app.history(id).unwrap();
                prop_assert!(history.iter().all(|entry| !entry.is_empty()));
                prop_assert!(history.windows(2).all(|pair| pair[0] != pair[1]), "{history:?}");
                if let Some((place, len)) = app.walk(id) {
                    prop_assert!(place < len);
                }
                let typed_here = step.edits() && focused.as_deref() == Some(*id);
                if !typed_here && put_back.as_deref() != Some(*id) {
                    prop_assert_eq!(app.input_of(id), before, "{:?} changed {}", step, id);
                }
            }
        }
    }
}

#[test]
fn a_command_enters_the_history_of_the_world_before_it() {
    let mut app = two_worlds();
    type_text(&mut app, "/focus beta/bryn");
    assert_eq!(press(&mut app, key(KeyCode::Enter)), []);
    assert_eq!(app.focused(), Some("beta/bryn"));
    assert_eq!(
        (
            app.history("alpha/aria").unwrap(),
            app.history("beta/bryn").unwrap()
        ),
        (vec!["/focus beta/bryn"], vec![])
    );
}

// The echo line.

fn player(world: &str, text: &str) -> ClientEvent {
    event(world, Source::SOURCE_PLAYER, text)
}

#[test]
fn the_default_dims() {
    let mut app = two_worlds();
    on_event(&mut app, player("alpha/aria", "hello"));
    let cells = cells_of(&mut app, "> hello");
    assert!(
        cells
            .iter()
            .all(|cell| cell.modifier.contains(Modifier::DIM)),
        "{cells:?}"
    );
}

#[test]
fn show() {
    let mut app = two_worlds().with_echo(mu_tui::Echo::Show);
    on_event(&mut app, player("alpha/aria", "hello"));
    let cells = cells_of(&mut app, "> hello");
    assert!(
        cells
            .iter()
            .all(|cell| !cell.modifier.contains(Modifier::DIM)),
        "{cells:?}"
    );
}

#[test]
fn hide() {
    let mut app = two_worlds().with_echo(mu_tui::Echo::Hide);
    on_event(&mut app, player("alpha/aria", "hello"));
    on_event(&mut app, game("alpha/aria", "You say, \"hello\""));
    assert!(shows(&mut app, "You say, \"hello\""));
    assert!(!screen(&mut app).iter().any(|row| row.starts_with("> ")));
}

#[test]
fn a_hidden_echo_marks_a_world_out_of_focus() {
    let mut app = two_worlds().with_echo(mu_tui::Echo::Hide);
    press(&mut app, ctrl('n'));
    press(&mut app, ctrl('p'));
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn");
    on_event(&mut app, player("beta/bryn", "hello"));
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn*");
}

#[test]
fn the_echo_of_a_pose_of_two_lines() {
    let mut app = listed();
    on_event(&mut app, player("alpha/aria", "one%rtwo"));
    let screen = screen(&mut app);
    let at = screen
        .iter()
        .position(|row| row == "> one")
        .unwrap_or_else(|| panic!("{screen:?}"));
    assert_eq!(screen[at + 1], "  two");
    assert!(
        !screen
            .iter()
            .any(|row| row.starts_with("> ") && row.contains("one%rtwo"))
    );
}

#[test]
fn an_echo_with_no_line_break() {
    // `alpha/aria` of `app` is in no list, and `beta/bryn` of `listed` has no line break.
    for (mut app, world) in [
        (app(&["alpha/aria"]), "alpha/aria"),
        (listed(), "beta/bryn"),
    ] {
        if world == "beta/bryn" {
            press(&mut app, ctrl('n'));
        }
        on_event(&mut app, player(world, "one%rtwo"));
        let rows: Vec<String> = rows(&mut app)
            .into_iter()
            .filter(|row| row.starts_with("> ") || row.starts_with("  "))
            .collect();
        assert_eq!(rows, ["> one%rtwo"], "{world}");
    }
}

proptest! {
    #[test]
    fn echo_rows_join_back(
        text in prop_oneof![any::<String>(), "[a-z \\t\\x01\\x1b\\x7f\\u{10EEEE}é]{0,40}"],
        line_break in "[^\\p{Cc}]{1,16}",
    ) {
        let rows = mu_tui::echo_rows(&text, Some(&line_break));
        prop_assert!(rows[0].starts_with("> "));
        let parts: Vec<&str> = rows
            .iter()
            .enumerate()
            .map(|(index, row)| row.strip_prefix(if index == 0 { "> " } else { "  " }).unwrap())
            .collect();
        // The screen shows a tab as a space, and each other control character as U+FFFD.
        let cleaned: String = text
            .chars()
            .map(|c| match c {
                '\t' => ' ',
                '\u{10EEEE}' => '\u{FFFD}',
                c if c.is_control() => '\u{FFFD}',
                c => c,
            })
            .collect();
        prop_assert_eq!(parts.join(&line_break), cleaned);
    }
}

// The first list of the worlds.

fn world_of(id: &str) -> World {
    World {
        id: id.to_owned(),
        ..Default::default()
    }
}

fn list_ends(app: &mut Model, listed: Result<Vec<World>, String>) {
    assert_eq!(update(app, Msg::Listed(listed)), []);
}

#[test]
fn before_the_first_list() {
    let mut app = starting();
    assert_eq!(top(&mut app), "listing worlds");
    list_ends(&mut app, Ok(vec![world_of("alpha/aria")]));
    assert_eq!(top(&mut app), "[alpha/aria]");
}

#[test]
fn a_send_before_the_first_list() {
    let mut app = starting();
    type_text(&mut app, "hi");
    assert_eq!(press(&mut app, key(KeyCode::Enter)), []);
    assert!(has_notice(&mut app, "no world"), "{:?}", rows(&mut app));
    assert_eq!(app.input(), "hi");
}

#[test]
fn text_typed_before_the_first_list() {
    let mut app = starting();
    type_text(&mut app, "hi");
    list_ends(&mut app, Ok(vec![world_of("alpha/aria")]));
    assert_eq!(app.input(), "hi");
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [Effect::Send {
            world: "alpha/aria".to_owned(),
            text: "hi".to_owned()
        }]
    );
}

#[test]
fn up_before_the_first_list() {
    let mut app = starting();
    type_text(&mut app, "hi");
    press(&mut app, key(KeyCode::Up));
    press(&mut app, key(KeyCode::Down));
    assert_eq!(app.input(), "hi");
}

#[test]
fn an_event_before_the_first_list() {
    let mut app = starting();
    on_event(&mut app, game("alpha/aria", "Welcome, aria."));
    assert_eq!(top(&mut app), "listing worlds");
    type_text(&mut app, "hi");
    assert_eq!(press(&mut app, key(KeyCode::Enter)), []);
    assert!(has_notice(&mut app, "no world"), "{:?}", rows(&mut app));
}

#[test]
fn a_focus_key_before_the_first_list() {
    let mut app = starting();
    on_event(&mut app, game("alpha/aria", "Welcome, aria."));
    press(&mut app, alt(KeyCode::Char('1')));
    type_text(&mut app, "hi");
    assert_eq!(press(&mut app, key(KeyCode::Enter)), []);
    assert!(has_notice(&mut app, "no world"), "{:?}", rows(&mut app));
}

#[test]
fn a_failed_first_list_after_an_event() {
    let mut app = starting();
    on_event(&mut app, game("alpha/aria", "Welcome, aria."));
    list_ends(&mut app, Err("boom".to_owned()));
    assert_eq!(top(&mut app), "[alpha/aria]");
}

#[test]
fn a_notice_before_the_first_list() {
    let mut app = starting();
    let outcome = mu_tui::probe::Outcome {
        complete: true,
        ..Default::default()
    };
    assert_eq!(update(&mut app, Msg::Probe(outcome)), []);
    list_ends(&mut app, Ok(vec![world_of("alpha/aria")]));
    assert_eq!(top(&mut app), "[alpha/aria]");
    assert!(
        has_notice(&mut app, "the terminal gave no character widths"),
        "{:?}",
        rows(&mut app)
    );
}

#[test]
fn the_first_list_fails() {
    let mut app = starting();
    list_ends(&mut app, Err("boom".to_owned()));
    assert_eq!(top(&mut app), "no worlds");
    assert!(has_notice(&mut app, "boom"), "{:?}", rows(&mut app));
}

#[test]
fn a_reset_during_the_first_list() {
    let mut app = starting();
    let reset = ClientEvent::Reset {
        instance_id: "i".to_owned(),
    };
    assert_eq!(update(&mut app, Msg::Client(reset)), []);
    let alpha = || Ok(vec![world_of("alpha/aria")]);
    assert_eq!(update(&mut app, Msg::Listed(alpha())), [Effect::ListWorlds]);
    assert_eq!(top(&mut app), "listing worlds");
    list_ends(&mut app, alpha());
    assert_eq!(top(&mut app), "[alpha/aria]");
}

#[test]
fn an_entry_that_needs_a_world_runs_with_none() {
    let mut app = two_worlds();
    let focus = mu_tui::EntryId::named("focus").unwrap();
    assert_eq!(
        update(
            &mut app,
            Msg::Run {
                entry: focus,
                arg: None
            }
        ),
        []
    );
    assert!(
        has_notice(&mut app, "focus needs a world"),
        "{:?}",
        screen(&mut app)
    );
    let next = mu_tui::EntryId::named("focus-next").unwrap();
    let arg = Some("beta/bryn".to_owned());
    assert_eq!(update(&mut app, Msg::Run { entry: next, arg }), []);
    assert!(
        has_notice(&mut app, "focus-next takes no argument"),
        "{:?}",
        screen(&mut app)
    );
}

#[test]
fn the_hint_of_returned_lines_counts_what_history_kept() {
    let mut app = two_worlds();
    type_lines(&mut app, &["x", "y", "y"]);
    assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "x")]);
    type_text(&mut app, "y");
    on_event(&mut app, ClientEvent::Lost);
    // The held `y` goes into history, and the second returned `y` equals it, so Up has one.
    assert!(has_notice(&mut app, "2 lines that waited"));
    let joined = screen(&mut app).join(" ");
    assert!(joined.contains("(1 in history)"), "{joined}");
}

#[test]
fn a_focus_command_before_the_first_list() {
    let mut app = starting();
    on_event(&mut app, game("alpha/aria", "Welcome, aria."));
    type_text(&mut app, "/focus alpha/aria");
    assert_eq!(press(&mut app, key(KeyCode::Enter)), []);
    assert!(
        has_notice(&mut app, "the worlds are not listed yet"),
        "{:?}",
        rows(&mut app)
    );
}

// The layout of the screen.

use mu_config::layout::{BUILT_IN, BlockId, Layout, preview};
use mu_proto::{GetClientConfigResponse, NamedLayout};

fn lines_of(lines: &[&str]) -> Vec<String> {
    lines.iter().map(ToString::to_string).collect()
}

/// A client configuration with the layout `lines`, the layout of each key of `worlds`, and the
/// two lists of segments.
fn config_with(
    lines: &[&str],
    worlds: &[(&str, &[&str])],
    label: &str,
    left: &[&str],
    right: &[&str],
) -> GetClientConfigResponse {
    GetClientConfigResponse {
        revision: format!("{lines:?}{worlds:?}{label}{left:?}{right:?}"),
        echo: "dim".to_owned(),
        layout_name: "test".to_owned(),
        layout: lines_of(lines),
        tab_label: label.to_owned(),
        statusline_left: lines_of(left),
        statusline_right: lines_of(right),
        world_layouts: worlds
            .iter()
            .map(|(key, lines)| NamedLayout {
                key: (*key).to_owned(),
                name: format!("layout-of-{}", key.replace('/', "-")),
                lines: lines_of(lines),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

/// Gives `app` the layout `lines`.
fn with_layout(app: &mut Model, lines: &[&str]) {
    app.apply_config(config_with(
        lines,
        &[],
        "{world}",
        &["world", "state"],
        &["idle"],
    ));
}

fn block(id: &str) -> BlockId {
    BlockId::new(id).expect("an id")
}

/// The rectangle of the pane of `id` on an 80 by 24 screen, as `(first column, last column,
/// first row, last row)`, each from 1.
fn rect_of(app: &Model, id: &str) -> (u16, u16, u16, u16) {
    let solved = mu_tui::solved(app, 80, 24);
    let area = solved
        .pane_of(&block(id))
        .unwrap_or_else(|| panic!("no pane of {id}"))
        .area;
    (
        area.x + 1,
        area.x + area.width,
        area.y + 1,
        area.y + area.height,
    )
}

/// The rows of the pane of `id` on an 80 by 24 screen, with no trailing spaces and no empty
/// row.
fn block_rows(app: &mut Model, id: &str) -> Vec<String> {
    let buffer = draw(app);
    let (left, right, top, bottom) = rect_of(app, id);
    (top - 1..bottom)
        .map(|y| {
            let row: String = (left - 1..right).map(|x| buffer[(x, y)].symbol()).collect();
            row.trim_end().to_owned()
        })
        .filter(|row| !row.is_empty())
        .collect()
}

const TWO_EDITORS: &[&str] = &[
    "output",
    "editor below main",
    "editor as scratch right of editor",
];

const SIDE_NOTICES: &[&str] = &[
    "output as main showing main",
    "editor below main",
    "output as notices right of main",
];

const STATUS: &[&str] = &[
    "output",
    "tabs above main",
    "editor below main",
    "statusline below editor",
];

#[test]
fn each_block_has_a_draw() {
    assert_eq!(mu_tui::drawn_blocks(), mu_tui::catalog_blocks());
}

#[test]
fn the_picture_of_each_built_in_layout() {
    let pictures: Vec<String> = BUILT_IN
        .iter()
        .map(|each| {
            let layout = each.layout().expect("valid");
            format!("{}\n{}", each.name, preview(&layout, 40, 12).join("\n"))
        })
        .collect();
    insta::assert_snapshot!(pictures.join("\n\n"));
}

#[test]
fn a_percent_and_a_count_of_cells() {
    let layout = Layout::parse(&[
        "output",
        "editor below main",
        "output as notices right of main, 25%",
        "output as log below main, 5",
    ])
    .expect("valid");
    insta::assert_snapshot!(preview(&layout, 40, 16).join("\n"));
}

#[test]
fn one_text_pane_has_no_title_row() {
    let mut app = two_worlds();
    assert_eq!(block_rows(&mut app, "main"), ["Welcome, aria."]);
}

#[test]
fn two_text_panes_have_title_rows() {
    let mut app = two_worlds();
    with_layout(&mut app, SIDE_NOTICES);
    assert_eq!(block_rows(&mut app, "main")[0], "[main]");
    assert_eq!(block_rows(&mut app, "notices"), ["[notices]"]);
}

#[test]
fn a_percent_in_a_narrow_terminal() {
    let mut app = app(&["alpha/aria"]);
    with_layout(
        &mut app,
        &[
            "output",
            "editor below main",
            "output as notices right of main, 10%",
        ],
    );
    let solved = mu_tui::solved(&app, 40, 24);
    assert_eq!(
        solved
            .pane_of(&block("notices"))
            .expect("a pane")
            .area
            .width,
        10
    );
}

#[test]
fn a_block_beside_two_rows_gets_its_columns() {
    let mut app = app(&["alpha/aria"]);
    with_layout(
        &mut app,
        &[
            "output",
            "tabs above main",
            "editor below main",
            "statusline below editor",
            "output as notices right of main and editor, 33%",
        ],
    );
    assert_eq!(rect_of(&app, "notices"), (55, 80, 2, 23));
    assert_eq!(rect_of(&app, "editor"), (1, 53, 23, 23));
    assert_eq!(rect_of(&app, "tabs"), (1, 80, 1, 1));
    assert_eq!(rect_of(&app, "statusline"), (1, 80, 24, 24));
}

#[test]
fn the_editor_after_the_notices_gets_its_rows() {
    let mut app = app(&["alpha/aria"]);
    with_layout(
        &mut app,
        &[
            "output",
            "output as notices right of main",
            "editor below main",
        ],
    );
    assert_eq!(rect_of(&app, "notices"), (42, 80, 1, 24));
    assert_eq!(rect_of(&app, "editor"), (1, 40, 24, 24));
}

#[test]
fn the_editor_before_the_notices_gets_its_columns() {
    let mut app = app(&["alpha/aria"]);
    with_layout(&mut app, SIDE_NOTICES);
    assert_eq!(rect_of(&app, "editor"), (1, 80, 24, 24));
    assert_eq!(rect_of(&app, "notices"), (42, 80, 1, 23));
}

// The streams.

fn notice_lines(rows: &[String]) -> Vec<&String> {
    rows.iter().filter(|row| row.starts_with("-- ")).collect()
}

#[test]
fn one_output_shows_the_notices_too() {
    let mut app = two_worlds();
    enter(&mut app, "/focus nowhere/nobody");
    let rows = block_rows(&mut app, "main");
    assert!(rows.contains(&"Welcome, aria.".to_owned()), "{rows:?}");
    assert_eq!(notice_lines(&rows).len(), 1, "{rows:?}");
}

#[test]
fn an_output_for_the_notices() {
    let mut app = two_worlds();
    with_layout(&mut app, SIDE_NOTICES);
    enter(&mut app, "/focus nowhere/nobody");
    let side = block_rows(&mut app, "notices");
    assert_eq!(notice_lines(&side).len(), 1, "{side:?}");
    let main = block_rows(&mut app, "main");
    assert!(main.contains(&"Welcome, aria.".to_owned()), "{main:?}");
    assert!(notice_lines(&main).is_empty(), "{main:?}");
}

#[test]
fn an_output_with_no_stream_yet() {
    let mut app = two_worlds();
    with_layout(
        &mut app,
        &[
            "output",
            "editor below main",
            "output as channels right of main",
        ],
    );
    assert_eq!(block_rows(&mut app, "channels"), ["[channels]"]);
}

#[test]
fn the_first_output_merges_in_the_order_of_arrival() {
    let mut app = two_worlds();
    enter(&mut app, "/focus nowhere/nobody");
    on_event(&mut app, game("alpha/aria", "A bell rings."));
    let rows = block_rows(&mut app, "main");
    assert_eq!(rows.len(), 3, "{rows:?}");
    assert_eq!(rows[0], "Welcome, aria.");
    assert!(rows[1].starts_with("-- "), "{rows:?}");
    assert_eq!(rows[2], "A bell rings.");
}

#[test]
fn an_output_that_comes_later_starts_with_new_lines() {
    let mut app = two_worlds().with_device("laptop");
    // The bad command stays in the editor, so Ctrl+U removes it before the next command.
    enter(&mut app, "/focus nowhere/nobody");
    press(&mut app, ctrl('u'));
    enter(&mut app, "/place output as notices right of main");
    assert_eq!(block_rows(&mut app, "notices"), ["[notices]"]);
    let main = block_rows(&mut app, "main");
    assert!(
        main.iter()
            .any(|row| row.starts_with("-- ") && row.contains("nowhere/nobody")),
        "{main:?}"
    );
}

#[test]
fn a_notice_shows_under_the_world_in_focus_only() {
    let mut app = two_worlds();
    enter(&mut app, "/focus nowhere/nobody");
    assert_eq!(press(&mut app, ctrl('n')), []);
    let rows = block_rows(&mut app, "main");
    assert!(rows.contains(&"Welcome, bryn.".to_owned()), "{rows:?}");
    assert!(
        rows.iter().all(|row| !row.contains("nowhere/nobody")),
        "{rows:?}"
    );
}

#[test]
fn a_layout_with_no_editor_from_the_server() {
    let mut app = two_worlds();
    with_layout(&mut app, &["output"]);
    assert!(
        top(&mut app).starts_with("[alpha/aria]"),
        "{}",
        top(&mut app)
    );
    let rows = rows(&mut app);
    assert!(
        rows.iter()
            .any(|row| row.starts_with("-- ") && row.contains("layout")),
        "{rows:?}"
    );
}

#[test]
fn a_new_layout_keeps_the_scroll_and_the_text() {
    let mut app = numbered(60);
    press(&mut app, key(KeyCode::PageUp));
    type_text(&mut app, "draft");
    let before = rows(&mut app)[0].clone();
    with_layout(&mut app, STATUS);
    assert_eq!(app.input(), "draft");
    assert_eq!(rows(&mut app)[0], before);
}

// Several editors.

fn two_editors() -> Model {
    let mut app = two_worlds();
    with_layout(&mut app, TWO_EDITORS);
    app
}

fn text_of(app: &Model, world: &str, editor: &str) -> String {
    app.text_of(Some(world), &block(editor)).unwrap_or_default()
}

#[test]
fn two_editors_keep_two_texts() {
    let mut app = two_editors();
    type_text(&mut app, "one");
    assert_eq!(press(&mut app, alt(KeyCode::Char('e'))), []);
    type_text(&mut app, "two");
    assert_eq!(text_of(&app, "alpha/aria", "editor"), "one");
    assert_eq!(text_of(&app, "alpha/aria", "scratch"), "two");
    assert_eq!(block_rows(&mut app, "editor"), ["one"]);
    assert_eq!(block_rows(&mut app, "scratch"), ["two"]);
}

#[test]
fn enter_sends_the_editor_with_the_input_focus() {
    let mut app = two_editors();
    type_text(&mut app, "one");
    press(&mut app, alt(KeyCode::Char('e')));
    type_text(&mut app, "two");
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [Effect::Send {
            world: "alpha/aria".to_owned(),
            text: "two".to_owned()
        }]
    );
    assert_eq!(text_of(&app, "alpha/aria", "editor"), "one");
    assert_eq!(text_of(&app, "alpha/aria", "scratch"), "");
}

#[test]
fn one_text_for_each_world() {
    let mut app = two_editors();
    press(&mut app, alt(KeyCode::Char('e')));
    type_text(&mut app, "two");
    press(&mut app, ctrl('n'));
    assert_eq!(app.input(), "");
    press(&mut app, ctrl('p'));
    assert_eq!(app.input(), "two");
    assert_eq!(text_of(&app, "beta/bryn", "scratch"), "");
}

#[test]
fn the_key_moves_the_input_focus() {
    let mut app = two_editors();
    assert_eq!(app.input_id(), block("editor"));
    press(&mut app, alt(KeyCode::Char('e')));
    assert_eq!(app.input_id(), block("scratch"));
    press(&mut app, alt(KeyCode::Char('e')));
    assert_eq!(app.input_id(), block("editor"));
}

#[test]
fn a_click_moves_the_input_focus() {
    let mut app = two_editors();
    draw(&mut app);
    let (left, _, top, _) = rect_of(&app, "scratch");
    let click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: left,
        row: top - 1,
        modifiers: Modifiers::NONE,
    };
    assert_eq!(update(&mut app, Msg::Terminal(TermEvent::Mouse(click))), []);
    assert_eq!(app.input_id(), block("scratch"));
}

#[test]
fn one_editor() {
    let mut app = two_worlds();
    assert_eq!(press(&mut app, alt(KeyCode::Char('e'))), []);
    assert_eq!(app.input_id(), block("editor"));
}

#[test]
fn a_hidden_editor_keeps_its_text() {
    let mut app = two_editors();
    press(&mut app, alt(KeyCode::Char('e')));
    type_text(&mut app, "two");
    with_layout(&mut app, &["output", "editor below main"]);
    assert_eq!(app.input_id(), block("editor"));
    with_layout(&mut app, TWO_EDITORS);
    assert_eq!(text_of(&app, "alpha/aria", "scratch"), "two");
}

#[test]
fn the_editors_share_the_history() {
    let mut app = two_editors();
    type_text(&mut app, "one");
    press(&mut app, key(KeyCode::Enter));
    press(&mut app, alt(KeyCode::Char('e')));
    up(&mut app);
    assert_eq!(text_of(&app, "alpha/aria", "scratch"), "one");
}

#[test]
fn two_editors_side_by_side() {
    let mut app = two_editors();
    type_lines(&mut app, &["one", "two", "three"]);
    assert_eq!(rect_of(&app, "editor"), (1, 40, 22, 24));
    assert_eq!(rect_of(&app, "scratch"), (42, 80, 22, 24));
}

#[test]
fn the_shorter_max_does_not_stop_the_taller_editor() {
    let mut app = two_worlds();
    with_layout(
        &mut app,
        &[
            "output",
            "editor below main",
            "editor as scratch right of editor, max 2",
        ],
    );
    type_lines(&mut app, &["one", "two", "three"]);
    assert_eq!(rect_of(&app, "editor"), (1, 40, 22, 24));
    assert_eq!(rect_of(&app, "scratch"), (42, 80, 22, 24));
}

// The tab label.

#[test]
fn a_label_keeps_the_unseen_mark() {
    let mut app = two_worlds();
    app.apply_config(config_with(
        BUILT_IN[0].lines,
        &[],
        "{character}",
        &["world"],
        &[],
    ));
    on_event(&mut app, game("beta/bryn", "A bell rings."));
    assert_eq!(top(&mut app), "[aria] bryn*");
}

#[test]
fn a_label_drops_a_control_character_of_the_id() {
    let mut app = app(&["alpha/ar\u{7}ia"]);
    let top = top(&mut app);
    assert!(
        top.starts_with("[alpha/ar") && !top.chars().any(char::is_control),
        "{top:?}"
    );
}

// The statusline.

/// A model of two worlds with the layout `status` and these two lists of segments.
fn status_app(left: &[&str], right: &[&str]) -> Model {
    let mut app = two_worlds();
    app.apply_config(config_with(STATUS, &[], "{world}", left, right));
    app
}

fn kind_event(world: &str, kind: EventKind, time_ms: i64) -> ClientEvent {
    ClientEvent::Event(Event {
        world: world.to_owned(),
        kind: kind.into(),
        source: Source::SOURCE_SYSTEM.into(),
        time_ms,
        ..Default::default()
    })
}

fn bottom_row(app: &mut Model) -> String {
    screen(app).remove(23)
}

#[test]
fn the_typed_count() {
    let mut app = status_app(&["typed"], &[]);
    type_text(&mut app, "héllo");
    assert_eq!(bottom_row(&mut app), "5");
}

#[test]
fn a_narrow_statusline() {
    let mut app = status_app(&["world"], &["state"]);
    on_event(
        &mut app,
        kind_event("alpha/aria", EventKind::EVENT_KIND_CONNECTED, 0),
    );
    assert_eq!(mu_tui::statusline::row(&app, 12), "al connected");
}

#[test]
fn no_connected_time_while_disconnected() {
    let mut app = status_app(&["world", "connected"], &[]);
    // The effects are the list of the worlds and the tick of the clock.
    update(
        &mut app,
        Msg::Client(kind_event(
            "alpha/aria",
            EventKind::EVENT_KIND_DISCONNECTED,
            0,
        )),
    );
    assert_eq!(bottom_row(&mut app), "alpha/aria");
}

#[test]
fn no_world_in_the_statusline() {
    let mut app = app(&[]);
    with_layout(&mut app, STATUS);
    assert_eq!(bottom_row(&mut app), "no world");
}

#[test]
fn a_time_under_an_hour() {
    assert_eq!(mu_tui::statusline::time(75_000), "1:15");
}

#[test]
fn a_time_over_an_hour() {
    assert_eq!(mu_tui::statusline::time(3_725_000), "1:02:05");
}

#[test]
fn a_time_one_second_under_an_hour() {
    assert_eq!(mu_tui::statusline::time(3_599_000), "59:59");
}

#[test]
fn a_time_of_one_hour() {
    assert_eq!(mu_tui::statusline::time(3_600_000), "1:00:00");
}

#[test]
fn a_time_before_the_clock_is_zero() {
    assert_eq!(mu_tui::statusline::time(-5_000), "0:00");
}

#[test]
fn a_wide_character_is_not_split() {
    let mut app = app(&["alpha/日本"]);
    app.apply_config(config_with(STATUS, &[], "{world}", &["world"], &[]));
    // The id takes 10 columns. With 9 columns, the last wide character does not fit.
    assert_eq!(mu_tui::statusline::row(&app, 10), "alpha/日本");
    assert_eq!(mu_tui::statusline::row(&app, 9), "alpha/日");
}

#[test]
fn the_default_max() {
    let mut app = two_worlds();
    let lines: Vec<String> = (1..=12).map(|number| format!("line{number}")).collect();
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    type_lines(&mut app, &lines);
    assert_eq!(rect_of(&app, "editor"), (1, 80, 17, 24));
}

// The commands of the layout.

const SIDEBAR_LINES: &[&str] = &[
    "output as main showing main",
    "tabs above main",
    "editor below main",
    "statusline below editor",
    "output as notices right of main, 30%",
];

/// Two worlds on the device `laptop`.
fn laptop() -> Model {
    let mut app = two_worlds().with_device("laptop");
    assert_eq!(app.layout_name(), "classic");
    draw(&mut app);
    app
}

/// Two worlds on the device `laptop`, with the player layout `mine` of `lines`.
fn laptop_with(lines: &[&str]) -> Model {
    let mut app = laptop();
    let mut config = config_with(lines, &[], "{world}", &["world", "state"], &["idle"]);
    "mine".clone_into(&mut config.layout_name);
    app.apply_config(config);
    // A layout with a time segment starts the tick of the clock at the next message.
    update(&mut app, Msg::Tick(0));
    draw(&mut app);
    app
}

/// The save of the layout `name` that an effect asks for, with its lines.
fn saved(effects: &[Effect], name: &str) -> Vec<String> {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::SaveLayout {
                name: found, lines, ..
            } if found == name => Some(lines.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no save of {name}: {effects:?}"))
}

/// The column of the divider on row 3 of an 80 by 24 screen, from 1.
fn divider_column(app: &mut Model) -> Option<usize> {
    let rows = screen(app);
    rows[2].chars().position(|c| c == '│').map(|at| at + 1)
}

fn mouse(app: &mut Model, kind: MouseEventKind, column: u16, row: u16) -> Vec<Effect> {
    let event = MouseEvent {
        kind,
        column,
        row,
        modifiers: Modifiers::NONE,
    };
    update(app, Msg::Terminal(TermEvent::Mouse(event)))
}

/// Presses the left button at one cell, drags to another, and releases there. The cells are
/// from 0.
fn drag_from_to(app: &mut Model, from: (u16, u16), to: (u16, u16)) -> Vec<Effect> {
    draw(app);
    mouse(app, MouseEventKind::Down(MouseButton::Left), from.0, from.1);
    mouse(app, MouseEventKind::Drag(MouseButton::Left), to.0, to.1);
    draw(app);
    mouse(app, MouseEventKind::Up(MouseButton::Left), to.0, to.1)
}

/// Whether a notice of the client holds each of `texts`. A narrow pane wraps a notice over
/// several rows, so the check reads the notices and not the rows.
fn notice_with(app: &mut Model, texts: &[&str]) -> bool {
    app.notices()
        .iter()
        .any(|line| line.starts_with("-- ") && texts.iter().all(|text| line.contains(text)))
}

#[test]
fn add_the_notices() {
    let mut app = laptop();
    let effects = enter(&mut app, "/place output as notices right of main, 30%");
    assert_eq!(divider_column(&mut app), Some(56));
    assert_eq!(block_rows(&mut app, "notices"), ["[notices]"]);
    assert_eq!(saved(&effects, "laptop").len(), 4);
}

#[test]
fn move_the_tabs() {
    let mut app = laptop();
    enter(&mut app, "/place tabs below main");
    let rows = screen(&mut app);
    assert!(rows[22].starts_with("[alpha/aria]"), "{rows:?}");
}

#[test]
fn a_bad_sentence() {
    let mut app = laptop();
    let before = app.layout().lines();
    assert_eq!(enter(&mut app, "/place spawns right of main"), []);
    assert!(notice_with(&mut app, &["spawns"]));
    assert_eq!(app.layout().lines(), before);
}

#[test]
fn hide_the_tabs() {
    let mut app = laptop();
    enter(&mut app, "/pane hide tabs");
    assert_eq!(screen(&mut app)[0], "Welcome, aria.");
}

#[test]
fn the_editor_stays() {
    let mut app = laptop();
    assert_eq!(enter(&mut app, "/pane hide editor"), []);
    assert!(notice_with(&mut app, &["editor"]));
    assert_eq!(app.layout().lines().len(), 3);
}

#[test]
fn hide_a_second_editor() {
    let mut app = laptop_with(TWO_EDITORS);
    press(&mut app, alt(KeyCode::Char('e')));
    assert_eq!(app.input_id(), block("scratch"));
    enter(&mut app, "/pane hide scratch");
    type_text(&mut app, "x");
    assert_eq!(app.input_id(), block("editor"));
    assert_eq!(text_of(&app, "alpha/aria", "editor"), "x");
    assert_eq!(app.layout().lines(), ["output", "editor below main"]);
}

#[test]
fn resize_the_notices() {
    let mut app = laptop_with(SIDEBAR_LINES);
    enter(&mut app, "/pane size notices 50%");
    assert_eq!(divider_column(&mut app), Some(40));
}

#[test]
fn a_size_of_fill_removes_the_size() {
    let mut app = laptop_with(SIDEBAR_LINES);
    enter(&mut app, "/pane size notices fill");
    assert_eq!(app.layout().lines()[4], "output as notices right of main");
}

#[test]
fn a_size_of_the_first_block() {
    let mut app = laptop_with(SIDEBAR_LINES);
    assert_eq!(enter(&mut app, "/pane size main 60%"), []);
    assert_eq!(app.layout().lines(), SIDEBAR_LINES);
    assert!(notice_with(&mut app, &["notices"]));
}

#[test]
fn a_resize_changes_one_line() {
    let mut app = laptop_with(SIDEBAR_LINES);
    enter(&mut app, "/pane size notices 40%");
    let lines = app.layout().lines();
    assert_eq!(lines[4], "output as notices right of main, 40%");
    assert_eq!(lines[..4], SIDEBAR_LINES[..4]);
}

#[test]
fn a_hide_of_an_anchor_shows_a_notice() {
    let mut app = laptop_with(&[
        "output",
        "output as notices right of main",
        "tabs above notices",
        "editor at bottom",
    ]);
    enter(&mut app, "/pane hide notices");
    let ids: Vec<String> = app
        .layout()
        .steps()
        .iter()
        .map(|step| step.id.to_string())
        .collect();
    assert_eq!(ids.len(), 3);
    assert!(!ids.contains(&"notices".to_owned()), "{ids:?}");
    assert!(notice_with(&mut app, &["plan"]));
}

#[test]
fn the_next_block_of_a_stack() {
    let mut app = laptop_with(&["output", "editor below main", "output as notices with main"]);
    assert_eq!(screen(&mut app)[0], "[main] notices");
    let next = mu_tui::EntryId::named("pane-next").expect("the entry");
    update(
        &mut app,
        Msg::Run {
            entry: next,
            arg: None,
        },
    );
    assert_eq!(screen(&mut app)[0], "main [notices]");
    update(
        &mut app,
        Msg::Run {
            entry: next,
            arg: None,
        },
    );
    assert_eq!(screen(&mut app)[0], "[main] notices");
}

#[test]
fn list_the_plan() {
    let mut app = laptop();
    assert_eq!(enter(&mut app, "/layout plan"), []);
    let rows = screen(&mut app);
    for line in [
        "-- 1. output",
        "-- 2. tabs above main",
        "-- 3. editor below main",
    ] {
        assert!(rows.iter().any(|row| row == line), "{rows:?}");
    }
}

#[test]
fn insert_in_the_middle() {
    let mut app = laptop_with(&["output", "editor below main", "statusline below editor"]);
    enter(&mut app, "/layout insert 3 output as notices right of main");
    assert_eq!(app.layout().lines()[2], "output as notices right of main");
    assert_eq!(app.layout().lines()[3], "statusline below editor");
    // The notices are beside the output only, and the editor takes the whole width.
    assert_eq!(rect_of(&app, "editor"), (1, 80, 23, 23));
    assert_eq!(rect_of(&app, "notices"), (42, 80, 1, 22));
}

#[test]
fn edit_a_line_by_its_id() {
    let mut app = laptop_with(SIDEBAR_LINES);
    enter(
        &mut app,
        "/layout edit notices output as notices left of main, 20%",
    );
    let lines = app.layout().lines();
    assert_eq!(lines[4], "output as notices left of main, 20%");
    assert_eq!(lines[..4], SIDEBAR_LINES[..4]);
    assert!(rect_of(&app, "notices").1 < rect_of(&app, "main").0);
}

#[test]
fn delete_a_line_that_others_need() {
    let mut app = laptop();
    assert_eq!(enter(&mut app, "/layout delete 1"), []);
    assert_eq!(app.layout().lines().len(), 3);
    assert!(notice_with(&mut app, &["line 2", "main"]));
}

// The mouse.

#[test]
fn a_drag_to_the_left() {
    let mut app = laptop_with(SIDEBAR_LINES);
    assert_eq!(divider_column(&mut app), Some(56));
    let effects = drag_from_to(&mut app, (55, 5), (39, 5));
    assert_eq!(divider_column(&mut app), Some(40));
    assert_eq!(
        saved(&effects, "mine")[4],
        "output as notices right of main, 50%"
    );
}

#[test]
fn a_drag_past_the_smallest_pane() {
    let mut app = laptop_with(SIDEBAR_LINES);
    drag_from_to(&mut app, (55, 5), (77, 5));
    let (left, right, _, _) = rect_of(&app, "notices");
    assert_eq!(right - left + 1, 10);
}

const NOTICES_BELOW: &[&str] = &[
    "output",
    "editor below main",
    "output as notices below main, 50%",
];

fn rows_of(app: &Model, id: &str) -> u16 {
    let (_, _, top, bottom) = rect_of(app, id);
    bottom - top + 1
}

#[test]
fn a_drag_down() {
    let mut app = laptop_with(NOTICES_BELOW);
    let before = rows_of(&app, "notices");
    let title = rect_of(&app, "notices").2 - 1;
    drag_from_to(&mut app, (20, title), (20, title + 4));
    assert_eq!(rows_of(&app, "notices"), before - 4);
}

#[test]
fn a_drag_down_past_the_smallest_pane() {
    let mut app = laptop_with(NOTICES_BELOW);
    let title = rect_of(&app, "notices").2 - 1;
    let all = rows_of(&app, "notices") + rows_of(&app, "main");
    drag_from_to(&mut app, (20, title), (20, 22));
    assert_eq!(rows_of(&app, "notices"), 2);
    assert_eq!(rows_of(&app, "main"), all - 2);
}

#[test]
fn a_click_on_a_title() {
    let mut app = laptop_with(&["output", "editor below main", "output as notices with main"]);
    assert_eq!(screen(&mut app)[0], "[main] notices");
    // The name `notices` takes columns 8 to 14 of the title row `[main] notices`.
    mouse(&mut app, MouseEventKind::Down(MouseButton::Left), 10, 0);
    let effects = mouse(&mut app, MouseEventKind::Up(MouseButton::Left), 10, 0);
    assert_eq!(effects, []);
    assert_eq!(screen(&mut app)[0], "main [notices]");
}

#[test]
fn a_selection_beside_a_divider() {
    let mut app = laptop_with(SIDEBAR_LINES);
    let row = u16::try_from(
        screen(&mut app)
            .iter()
            .position(|row| row.starts_with("Welcome, aria."))
            .expect("the greeting shows"),
    )
    .unwrap();
    let effects = drag_from_to(&mut app, (0, row), (6, row));
    assert_eq!(effects, [Effect::ClipboardWrite("Welcome".to_owned())]);
    assert_eq!(divider_column(&mut app), Some(56));
}

#[test]
fn a_drag_from_a_divider_selects_no_text() {
    let mut app = laptop_with(SIDEBAR_LINES);
    let effects = drag_from_to(&mut app, (55, 5), (50, 5));
    assert!(
        effects
            .iter()
            .all(|effect| !matches!(effect, Effect::ClipboardWrite(_))),
        "{effects:?}"
    );
    assert_eq!(app.pane(app.focused_pane()).unwrap().selection(), None);
}

#[test]
fn a_drag_does_not_drift() {
    let mut app = laptop_with(SIDEBAR_LINES);
    draw(&mut app);
    mouse(&mut app, MouseEventKind::Down(MouseButton::Left), 55, 5);
    for column in [40, 30, 45, 60, 50, 39] {
        mouse(&mut app, MouseEventKind::Drag(MouseButton::Left), column, 5);
        draw(&mut app);
    }
    mouse(&mut app, MouseEventKind::Up(MouseButton::Left), 39, 5);
    assert_eq!(divider_column(&mut app), Some(40));
    assert_eq!(
        app.layout().lines()[4],
        "output as notices right of main, 50%"
    );
}

/// Whether a row of the block `id` starts a notice.
fn has_a_notice_row(app: &mut Model, id: &str) -> bool {
    block_rows(app, id).iter().any(|row| row.starts_with("-- "))
}

#[test]
fn a_refused_line_goes_to_the_notices_block() {
    let mut app = laptop_with(SIDEBAR_LINES);
    type_text(&mut app, "hi");
    assert_ne!(press(&mut app, key(KeyCode::Enter)), [] as [Effect; 0]);
    on_reply(
        &mut app,
        Reply::Refused {
            world: "alpha/aria".to_owned(),
            text: "hi".to_owned(),
            reason: "down".to_owned(),
        },
    );
    assert_eq!(block_rows(&mut app, "notices").last().unwrap(), "-- down");
    assert!(!has_a_notice_row(&mut app, "main"));
}

#[test]
fn a_refused_line_of_another_world_waits_in_its_notices() {
    let mut app = laptop_with(SIDEBAR_LINES);
    type_text(&mut app, "hi");
    assert_ne!(press(&mut app, key(KeyCode::Enter)), [] as [Effect; 0]);
    enter(&mut app, "/focus beta/bryn");
    on_reply(
        &mut app,
        Reply::Refused {
            world: "alpha/aria".to_owned(),
            text: "hi".to_owned(),
            reason: "down".to_owned(),
        },
    );
    // The world in focus shows nothing of it.
    assert!(!has_a_notice_row(&mut app, "notices"));
    assert!(!has_a_notice_row(&mut app, "main"));
    enter(&mut app, "/focus alpha/aria");
    assert_eq!(block_rows(&mut app, "notices").last().unwrap(), "-- down");
    assert!(!has_a_notice_row(&mut app, "main"));
}

#[test]
fn enter_with_no_world_goes_to_the_notices_block() {
    let mut app = app(&[]).with_device("laptop");
    app.apply_config(config_with(SIDEBAR_LINES, &[], "{world}", &["world"], &[]));
    draw(&mut app);
    type_text(&mut app, "hi");
    assert_eq!(press(&mut app, key(KeyCode::Enter)), []);
    assert!(has_a_notice_row(&mut app, "notices"));
    assert!(!has_a_notice_row(&mut app, "main"));
}

// The main output, and a save that changes nothing.

#[test]
fn a_command_that_changes_nothing_saves_nothing() {
    let mut app = laptop();
    assert_eq!(enter(&mut app, "/place tabs above main"), []);
    assert_eq!(app.layout_name(), "classic");
    assert_eq!(app.input(), "");
}

#[test]
fn a_drag_back_to_its_start_saves_nothing() {
    let mut app = laptop_with(SIDEBAR_LINES);
    let name = app.layout_name().to_owned();
    draw(&mut app);
    mouse(&mut app, MouseEventKind::Down(MouseButton::Left), 55, 5);
    mouse(&mut app, MouseEventKind::Drag(MouseButton::Left), 45, 5);
    draw(&mut app);
    assert_ne!(app.layout().lines(), SIDEBAR_LINES);
    mouse(&mut app, MouseEventKind::Drag(MouseButton::Left), 55, 5);
    draw(&mut app);
    assert_eq!(
        mouse(&mut app, MouseEventKind::Up(MouseButton::Left), 55, 5),
        []
    );
    assert_eq!(app.layout().lines(), SIDEBAR_LINES);
    assert_eq!(app.layout_name(), name);
}

// Helpers of the configuration and the editor.

/// A configuration with the player layout `name` of `lines`.
fn named_config(name: &str, lines: &[&str]) -> GetClientConfigResponse {
    let mut config = config_with(lines, &[], "{world}", &["world"], &[]);
    name.clone_into(&mut config.layout_name);
    config
}

/// Clears the editor, types `text` and presses Enter.
fn enter_cleared(app: &mut Model, text: &str) -> Vec<Effect> {
    let typed = app.input().chars().count();
    for _ in 0..typed {
        press(app, key(KeyCode::Backspace));
    }
    enter(app, text)
}

// An edit of the layout and the focus.

#[test]
fn a_focus_move_during_a_drag_ends_the_drag_and_puts_the_layout_back() {
    let mut app = laptop_with(SIDEBAR_LINES);
    draw(&mut app);
    mouse(&mut app, MouseEventKind::Down(MouseButton::Left), 55, 5);
    mouse(&mut app, MouseEventKind::Drag(MouseButton::Left), 45, 5);
    draw(&mut app);
    assert_ne!(app.layout().lines(), SIDEBAR_LINES);
    // The key of the next world moves the focus while the button is down.
    enter_cleared(&mut app, "/focus beta/bryn");
    assert_eq!(app.layout().lines(), SIDEBAR_LINES);
    mouse(&mut app, MouseEventKind::Drag(MouseButton::Left), 40, 5);
    let effects = mouse(&mut app, MouseEventKind::Up(MouseButton::Left), 40, 5);
    assert_eq!(effects, []);
    assert_eq!(app.layout().lines(), SIDEBAR_LINES);
    let rows = block_rows(&mut app, "notices");
    assert!(
        rows.iter().any(|row| row.contains("the drag ended")),
        "{rows:?}"
    );
}

#[test]
fn a_new_configuration_ends_a_drag_and_says_so() {
    let mut app = laptop_with(SIDEBAR_LINES);
    draw(&mut app);
    mouse(&mut app, MouseEventKind::Down(MouseButton::Left), 55, 5);
    mouse(&mut app, MouseEventKind::Drag(MouseButton::Left), 45, 5);
    // The configuration of the server changes while the button is down.
    app.apply_config(named_config("other", SIDEBAR_LINES));
    assert_eq!(app.layout().lines(), SIDEBAR_LINES);
    let effects = mouse(&mut app, MouseEventKind::Up(MouseButton::Left), 45, 5);
    assert_eq!(effects, []);
    let rows = block_rows(&mut app, "notices").join(" ");
    assert!(rows.contains("the drag ended"), "{rows}");
}
