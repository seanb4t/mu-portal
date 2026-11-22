//! The scenarios of the terminal-client capability on `update` and `view`, drawn on a
//! `TestBackend`.

use mu_client::{ClientError, ClientEvent};
use mu_proto::{Event, EventKind, Source, World};
use mu_tui::convert::convert;
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
    let mut app = Model::from_listing(Ok(worlds.to_vec()));
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn");
}

const LIST_FAILED: &str = "-- could not list the worlds: unavailable: no server";

fn failed_listing() -> Model {
    Model::from_listing(Err(ClientError {
        code: "unavailable",
        message: "no server".to_owned(),
    }))
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
fn enter_sends_to_world_in_focus() {
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
fn empty_enter_sends_empty_line() {
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
fn refused_line_keeps_input() {
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
fn refused_line_goes_before_the_typed_text() {
    let mut app = two_worlds();
    type_text(&mut app, "hi");
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "hi")]
    );
    type_text(&mut app, "next");
    on_reply(&mut app, refused("alpha/aria", "hi"));
    assert_eq!(app.input(), "hi\nnext");
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
fn refused_line_stops_the_rest() {
    let mut app = two_worlds();
    type_lines(&mut app, &["one", "two", "three"]);
    assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "one")]);
    assert_eq!(
        sent(&mut app, "alpha/aria", "one"),
        [send("alpha/aria", "two")]
    );
    on_reply(&mut app, refused("alpha/aria", "two"));
    assert_eq!(app.input(), "two\nthree");
    assert_eq!(sent(&mut app, "alpha/aria", "two"), []);
    assert!(has_notice(&mut app, "not connected"));
    assert_eq!(screen(&mut app)[22..], ["two", "three"]);
}

#[test]
fn refused_last_line_goes_before_the_typed_text() {
    let mut app = two_worlds();
    type_lines(&mut app, &["one", "two"]);
    assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "one")]);
    assert_eq!(
        sent(&mut app, "alpha/aria", "one"),
        [send("alpha/aria", "two")]
    );
    type_text(&mut app, "later");
    on_reply(&mut app, refused("alpha/aria", "two"));
    assert_eq!(app.input(), "two\nlater");
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
fn refused_send_stops_the_sends_after_it() {
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
    assert_eq!(app.input(), "one\ntwo\nlater");
    // No send of the world waits now, so a new send goes at once.
    press(&mut app, ctrl('u'));
    assert_eq!(
        press(&mut app, key(KeyCode::Enter)),
        [send("alpha/aria", "one\ntwo\n")]
    );
}

#[test]
fn hint_does_not_come_back_after_a_reset() {
    let mut app = listed();
    press(&mut app, ctrl('n'));
    type_lines(&mut app, &["one", "two"]);
    press(&mut app, KeyEvent::new(KeyCode::Char('r'), Modifiers::ALT));
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
        let mut app = Model::from_listing(Ok(vec![world]));
        type_lines(&mut app, &["one", "two"]);
        press(&mut app, KeyEvent::new(KeyCode::Char('r'), Modifiers::ALT));
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
    Model::from_listing(Ok(vec![
        world("alpha/aria", Some("%r")),
        world("beta/bryn", None),
    ]))
}

#[test]
fn lines_become_one_line() {
    let mut app = listed();
    type_lines(&mut app, &["one", "two"]);
    let alt_r = KeyEvent::new(KeyCode::Char('r'), Modifiers::ALT);
    assert_eq!(press(&mut app, alt_r), []);
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
    let alt_r = KeyEvent::new(KeyCode::Char('r'), Modifiers::ALT);
    assert_eq!(press(&mut app, alt_r), []);
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
fn lost_server_puts_the_waiting_lines_back() {
    let mut app = three_lines_sent();
    type_text(&mut app, "later");
    on_event(&mut app, ClientEvent::Lost);
    assert_eq!(app.input(), "two\nthree\nlater");
    assert!(has_notice(&mut app, "2 lines that waited went back"));
    // The send with the server stays: its reply sends nothing more.
    assert_eq!(sent(&mut app, "alpha/aria", "one"), []);
}

#[test]
fn reset_puts_the_waiting_lines_back() {
    let mut app = three_lines_sent();
    on_event(
        &mut app,
        ClientEvent::Reset {
            instance_id: "i".to_owned(),
        },
    );
    assert_eq!(app.input(), "two\nthree");
    assert!(has_notice(&mut app, "the server started again"));
}

#[test]
fn refusal_after_a_loss_puts_back_only_its_line() {
    let mut app = three_lines_sent();
    on_event(&mut app, ClientEvent::Lost);
    on_reply(&mut app, refused("alpha/aria", "one"));
    assert_eq!(app.input(), "one\ntwo\nthree");
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

// The line break of a world that the first list did not hold.

fn alt_r() -> KeyEvent {
    KeyEvent::new(KeyCode::Char('r'), Modifiers::ALT)
}

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
fn lost_server_puts_back_the_lines_in_the_order_of_the_worlds() {
    let mut app = two_worlds();
    press(&mut app, ctrl('n'));
    type_lines(&mut app, &["b1", "b2"]);
    assert_eq!(press(&mut app, alt_enter()), [send("beta/bryn", "b1")]);
    press(&mut app, ctrl('p'));
    type_lines(&mut app, &["a1", "a2"]);
    assert_eq!(press(&mut app, alt_enter()), [send("alpha/aria", "a1")]);
    on_event(&mut app, ClientEvent::Lost);
    assert_eq!(app.input(), "a2\nb2");
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
    // The reset puts back nothing more, so no line about returned lines is on the screen.
    assert!(!has_notice(&mut app, "lines that waited"));
}
