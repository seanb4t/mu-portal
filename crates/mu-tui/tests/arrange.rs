//! Arrange mode: the keys, the letters and the drag.

use mu_client::ClientEvent;
use mu_config::layout::BlockId;
use mu_proto::{Event, EventKind, GetClientConfigResponse, Source};
use mu_tui::{Effect, Model, Msg, Then, update, view};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Modifier;
use termina::Event as TermEvent;
use termina::event::{KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseEventKind};

fn game(world: &str, text: &str) -> Msg {
    Msg::Client(ClientEvent::Event(Event {
        world: world.to_owned(),
        kind: EventKind::EVENT_KIND_LINE.into(),
        source: Source::SOURCE_GAME.into(),
        text: text.to_owned(),
        ..Default::default()
    }))
}

/// Two worlds on the device `laptop`, with the layout `classic`.
fn laptop() -> Model {
    let mut app =
        Model::new(["alpha/aria".to_owned(), "beta/bryn".to_owned()]).with_device("laptop");
    update(&mut app, game("alpha/aria", "Welcome, aria."));
    draw(&mut app);
    app
}

/// Two worlds on the device `laptop`, with the player layout `mine` of `lines`.
fn laptop_with(lines: &[&str]) -> Model {
    let mut app = laptop();
    app.apply_config(GetClientConfigResponse {
        revision: format!("{lines:?}"),
        echo: "dim".to_owned(),
        layout_name: "mine".to_owned(),
        layout: lines.iter().map(ToString::to_string).collect(),
        tab_label: "{world}".to_owned(),
        statusline_left: vec!["world".to_owned()],
        ..Default::default()
    });
    draw(&mut app);
    app
}

fn draw_sized(app: &mut Model, width: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, 24)).unwrap();
    terminal.draw(|frame| view(app, frame)).unwrap();
    terminal.backend().buffer().clone()
}

/// An 80 by 24 screen.
fn draw(app: &mut Model) -> Buffer {
    draw_sized(app, 80)
}

fn rows(app: &mut Model) -> Vec<String> {
    let buffer = draw(app);
    (0..buffer.area.height)
        .map(|y| {
            let row: String = (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect();
            row.trim_end().to_owned()
        })
        .collect()
}

fn key(app: &mut Model, code: KeyCode, modifiers: Modifiers) -> Vec<Effect> {
    update(
        app,
        Msg::Terminal(TermEvent::Key(KeyEvent::new(code, modifiers))),
    )
}

/// Presses each char of `keys` as a plain key.
fn keys(app: &mut Model, keys: &str) {
    for typed in keys.chars() {
        assert_eq!(key(app, KeyCode::Char(typed), Modifiers::NONE), []);
    }
}

/// Starts arrange mode with Alt+L.
fn arrange(app: &mut Model) {
    assert_eq!(key(app, KeyCode::Char('l'), Modifiers::ALT), []);
    assert!(app.arranges());
    draw(app);
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

fn block(id: &str) -> BlockId {
    BlockId::new(id).expect("an id")
}

/// The rectangle of the pane of `id` in arrange mode: the first column, the first row, the
/// count of columns and the count of rows, the first two from 0.
fn area_of(app: &Model, id: &str) -> (u16, u16, u16, u16) {
    let area = mu_tui::solved_now(app)
        .pane_of(&block(id))
        .unwrap_or_else(|| panic!("no pane of {id}"))
        .area;
    (area.x, area.y, area.width, area.height)
}

const CLASSIC: &[&str] = &["output", "tabs above output", "editor below output"];
const SIDEBAR: &[&str] = &[
    "output",
    "tabs above output",
    "editor below output",
    "statusline below editor",
    "output as notices right of output, 30%",
];
const NOTICES_BELOW: &[&str] = &[
    "output",
    "editor below output",
    "output as notices below output",
];

#[test]
fn the_letters() {
    let mut app = laptop();
    arrange(&mut app);
    let rows = rows(&mut app);
    assert!(rows[0].starts_with('A'), "{rows:?}");
    assert!(rows[1].starts_with('B'), "{rows:?}");
    assert!(rows[22].starts_with('C'), "{rows:?}");
    assert!(
        rows[23].contains("Enter") && rows[23].contains("Esc"),
        "{rows:?}"
    );
}

#[test]
fn typing_in_arrange_mode() {
    let mut app = laptop();
    keys(&mut app, "draft");
    arrange(&mut app);
    keys(&mut app, "z");
    key(&mut app, KeyCode::Escape, Modifiers::NONE);
    assert!(!app.arranges());
    assert_eq!(app.input(), "draft");
}

#[test]
fn events_still_show_in_arrange_mode() {
    let mut app = laptop();
    arrange(&mut app);
    update(&mut app, game("alpha/aria", "A bell rings."));
    let rows = rows(&mut app);
    assert!(rows.iter().any(|row| row == "A bell rings."), "{rows:?}");
}

#[test]
fn the_tabs_go_below_the_output() {
    let mut app = laptop();
    arrange(&mut app);
    keys(&mut app, "ab");
    key(&mut app, KeyCode::Down, Modifiers::NONE);
    assert_eq!(
        app.layout().lines(),
        ["output", "editor below output", "tabs below output"]
    );
    let rows = rows(&mut app);
    // The row above the editor is the tabs, with the letter of the block on its first cell.
    assert!(rows[21].contains("alpha/aria]"), "{rows:?}");
}

#[test]
fn a_refused_stack() {
    let mut app = laptop();
    arrange(&mut app);
    keys(&mut app, "cb=");
    assert_eq!(app.layout().lines(), CLASSIC);
    let rows = rows(&mut app);
    assert!(rows[23].contains("editor"), "{rows:?}");
}

#[test]
fn add_the_statusline() {
    let mut app = laptop();
    arrange(&mut app);
    keys(&mut app, "+");
    let listed = rows(&mut app)[23].clone();
    assert!(listed.contains("f statusline"), "{listed}");
    keys(&mut app, "fc");
    key(&mut app, KeyCode::Down, Modifiers::NONE);
    assert_eq!(app.layout().lines()[3], "statusline below editor");
    // The statusline is on the row above the keys of the mode.
    assert_eq!(area_of(&app, "statusline"), (0, 22, 80, 1));
}

#[test]
fn undo() {
    let mut app = laptop();
    arrange(&mut app);
    keys(&mut app, "a-");
    assert_eq!(app.layout().lines().len(), 2);
    key(&mut app, KeyCode::Char('z'), Modifiers::CONTROL);
    assert_eq!(app.layout().lines(), CLASSIC);
    assert!(app.arranges());
}

#[test]
fn a_larger_and_a_smaller_block() {
    let mut app = laptop_with(SIDEBAR);
    arrange(&mut app);
    // The letters follow the screen: tabs, output, notices, editor, statusline.
    keys(&mut app, "c>");
    assert_eq!(
        app.layout().lines()[4],
        "output as notices right of output, 35%"
    );
    keys(&mut app, "<<");
    assert_eq!(
        app.layout().lines()[4],
        "output as notices right of output, 25%"
    );
}

#[test]
fn a_larger_block_stops_at_ninety_percent() {
    let mut app = laptop_with(&[
        "output",
        "editor below output",
        "output as notices right of output, 90%",
    ]);
    // A screen of 300 columns has room for the smallest pane beside 90 percent.
    draw_sized(&mut app, 300);
    assert_eq!(key(&mut app, KeyCode::Char('l'), Modifiers::ALT), []);
    draw_sized(&mut app, 300);
    keys(&mut app, "b>");
    assert_eq!(
        app.layout().lines()[2],
        "output as notices right of output, 90%"
    );
}

#[test]
fn a_larger_block_in_a_small_split_keeps_its_size() {
    let mut app = laptop_with(&[
        "output",
        "editor below output",
        "output as notices right of output, 30%",
    ]);
    draw_sized(&mut app, 20);
    assert_eq!(key(&mut app, KeyCode::Char('l'), Modifiers::ALT), []);
    draw_sized(&mut app, 20);
    keys(&mut app, "b>");
    assert_eq!(
        app.layout().lines()[2],
        "output as notices right of output, 30%"
    );
}

#[test]
fn a_letter_bound_to_undo_still_picks() {
    let mut keymap = mu_tui::Keymap::default();
    keymap.bind(
        mu_tui::Chord::plain(KeyCode::Char('a')),
        mu_tui::EntryId::named("undo").expect("the entry"),
    );
    let mut app = laptop().with_keymap(keymap);
    arrange(&mut app);
    keys(&mut app, "ab");
    key(&mut app, KeyCode::Down, Modifiers::NONE);
    assert_eq!(app.layout().lines()[2], "tabs below output");
}

#[test]
fn esc_drops_the_pick() {
    let mut app = laptop();
    arrange(&mut app);
    keys(&mut app, "a");
    key(&mut app, KeyCode::Escape, Modifiers::NONE);
    assert!(app.arranges());
    // With no pick, `-` hides nothing.
    keys(&mut app, "-");
    assert_eq!(app.layout().lines(), CLASSIC);
}

#[test]
fn esc_restores() {
    let mut app = laptop();
    arrange(&mut app);
    keys(&mut app, "a-");
    assert_eq!(app.layout().lines().len(), 2);
    let effects = key(&mut app, KeyCode::Escape, Modifiers::NONE);
    assert_eq!(effects, []);
    assert!(!app.arranges());
    assert_eq!(app.layout().lines(), CLASSIC);
}

#[test]
fn enter_saves() {
    let mut app = laptop();
    arrange(&mut app);
    keys(&mut app, "a-");
    let effects = key(&mut app, KeyCode::Enter, Modifiers::NONE);
    assert_eq!(
        effects,
        [Effect::SaveLayout {
            name: "laptop".to_owned(),
            lines: vec!["output".to_owned(), "editor below output".to_owned()],
            device: "laptop".to_owned(),
            then: Then::SetField,
        }]
    );
    assert!(!app.arranges());
}

#[test]
fn enter_with_no_change_saves_nothing() {
    let mut app = laptop();
    arrange(&mut app);
    assert_eq!(key(&mut app, KeyCode::Enter, Modifiers::NONE), []);
    assert!(!app.arranges());
}

/// Drags from the middle of the pane of `from` to the cell `to`, and releases there.
fn drag_block(app: &mut Model, from: &str, to: (u16, u16)) {
    let (x, y, width, height) = area_of(app, from);
    mouse(
        app,
        MouseEventKind::Down(MouseButton::Left),
        x + width / 2,
        y + height / 2,
    );
    mouse(app, MouseEventKind::Drag(MouseButton::Left), to.0, to.1);
    draw(app);
    mouse(app, MouseEventKind::Up(MouseButton::Left), to.0, to.1);
}

#[test]
fn a_drag_to_the_right_edge() {
    let mut app = laptop_with(NOTICES_BELOW);
    arrange(&mut app);
    let (x, y, width, height) = area_of(&app, "output");
    drag_block(&mut app, "notices", (x + width - 1, y + height / 2));
    assert_eq!(app.layout().lines()[2], "output as notices right of output");
    let (notices, output) = (area_of(&app, "notices"), area_of(&app, "output"));
    assert!(
        notices.0 > output.0 + output.2 - 1,
        "{notices:?} {output:?}"
    );
}

#[test]
fn a_drag_to_the_middle_stacks() {
    let mut app = laptop_with(NOTICES_BELOW);
    arrange(&mut app);
    let (x, y, width, height) = area_of(&app, "output");
    drag_block(&mut app, "notices", (x + width / 2, y + height / 2));
    assert_eq!(app.layout().lines()[2], "output as notices with output");
}

#[test]
fn the_drop_part_is_marked() {
    let mut app = laptop_with(NOTICES_BELOW);
    arrange(&mut app);
    let (x, y, width, height) = area_of(&app, "output");
    let (from_x, from_y, from_width, from_height) = area_of(&app, "notices");
    mouse(
        &mut app,
        MouseEventKind::Down(MouseButton::Left),
        from_x + from_width / 2,
        from_y + from_height / 2,
    );
    let target = (x + width - 1, y + height / 2);
    mouse(
        &mut app,
        MouseEventKind::Drag(MouseButton::Left),
        target.0,
        target.1,
    );
    let buffer = draw(&mut app);
    let marked =
        |column: u16, row: u16| buffer[(column, row)].modifier.contains(Modifier::REVERSED);
    // The right third of the pane has the mark, and its middle has none.
    assert!(marked(target.0, target.1));
    assert!(marked(x + width - width / 3, y + height / 2));
    assert!(!marked(x + width / 2, y + height / 2));
}
