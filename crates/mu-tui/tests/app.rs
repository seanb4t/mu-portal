//! The scenarios of the terminal-client capability on `update` and `view`, drawn on a
//! `TestBackend`.

use mu_client::ClientEvent;
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
    let mut app = Model::from_listing(Ok(worlds.to_vec()));
    assert_eq!(top(&mut app), "[alpha/aria] beta/bryn");
}

const LIST_FAILED: &str = "-- could not list the worlds: unavailable: no server";

fn failed_listing() -> Model {
    Model::from_listing(Err("unavailable: no server".to_owned()))
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
        let mut app = Model::from_listing(Ok(vec![world]));
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
    Model::from_listing(Ok(vec![
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
    let rows = screen(&mut app);
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
                prop_assert!(history.len() <= 1000);
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
        text in any::<String>(),
        line_break in "[^\\p{Cc}]{1,16}",
    ) {
        let rows = mu_tui::echo_rows(&text, Some(&line_break));
        prop_assert!(rows[0].starts_with("> "));
        let parts: Vec<&str> = rows
            .iter()
            .enumerate()
            .map(|(index, row)| row.strip_prefix(if index == 0 { "> " } else { "  " }).unwrap())
            .collect();
        let cleaned = mu_tui::echo_rows(&text, None).remove(0);
        prop_assert_eq!(parts.join(&line_break), cleaned.strip_prefix("> ").unwrap());
    }
}
