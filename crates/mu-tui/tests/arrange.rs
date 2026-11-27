//! Arrange mode: the keys, the letters and the drag.

use mu_proto::GetClientConfigResponse;
use mu_tui::{Effect, Model, Msg, update, view};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Modifier;
use termina::Event as TermEvent;
use termina::event::{KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseEventKind};

use support::{block, game_line, run_command, sidebar_lines};

mod support;

fn game(world: &str, text: &str) -> Msg {
    game_line(world, text, &[])
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

/// The rectangle of the pane of `id` in arrange mode: the first column, the first row, the
/// count of columns and the count of rows, the first two from 0.
fn area_of(app: &Model, id: &str) -> (u16, u16, u16, u16) {
    let area = mu_tui::solved_now(app)
        .pane_of(&block(id))
        .unwrap_or_else(|| panic!("no pane of {id}"))
        .area;
    (area.x, area.y, area.width, area.height)
}

const CLASSIC: &[&str] = &["output", "tabs above main", "editor below main"];
const NOTICES_BELOW: &[&str] = &[
    "output",
    "editor below main",
    "output as notices below main",
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
        ["output", "editor below main", "tabs below main"]
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
    let mut app = laptop_with(sidebar_lines());
    arrange(&mut app);
    // The letters follow the screen: tabs, output, notices, editor, statusline.
    keys(&mut app, "c>");
    assert_eq!(
        app.layout().lines()[4],
        "output as notices right of main, 35%"
    );
    keys(&mut app, "<<");
    assert_eq!(
        app.layout().lines()[4],
        "output as notices right of main, 25%"
    );
}

#[test]
fn a_larger_block_stops_at_ninety_percent() {
    let mut app = laptop_with(&[
        "output",
        "editor below main",
        "output as notices right of main, 80%",
    ]);
    // A screen of 300 columns has room for the smallest pane beside 90 percent.
    draw_sized(&mut app, 300);
    assert_eq!(key(&mut app, KeyCode::Char('l'), Modifiers::ALT), []);
    draw_sized(&mut app, 300);
    // The first step shows that the key changes the block.
    keys(&mut app, "b>");
    assert_eq!(
        app.layout().lines()[2],
        "output as notices right of main, 85%"
    );
    // Three more steps ask for 100 percent.
    keys(&mut app, ">>>");
    assert_eq!(
        app.layout().lines()[2],
        "output as notices right of main, 90%"
    );
}

#[test]
fn a_larger_block_in_a_small_split_keeps_its_size() {
    let mut app = laptop_with(&[
        "output",
        "editor below main",
        "output as notices right of main, 30%",
    ]);
    draw_sized(&mut app, 20);
    assert_eq!(key(&mut app, KeyCode::Char('l'), Modifiers::ALT), []);
    draw_sized(&mut app, 20);
    keys(&mut app, "b>");
    assert_eq!(
        app.layout().lines()[2],
        "output as notices right of main, 30%"
    );
    // The last row says why, and the refused step is no step of the undo list.
    let shown = rows(&mut app);
    assert!(shown[23].contains("no room"), "{shown:?}");
    // The same keys on a wider screen change the block, so the pick is on it.
    draw_sized(&mut app, 80);
    keys(&mut app, ">");
    assert_eq!(
        app.layout().lines()[2],
        "output as notices right of main, 35%"
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
    assert_eq!(app.layout().lines()[2], "tabs below main");
    let rows = rows(&mut app);
    assert!(rows[21].contains("alpha/aria]"), "{rows:?}");
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
            lines: vec!["output".to_owned(), "editor below main".to_owned()],
            device: "laptop".to_owned(),
            then: Some(mu_config::Choice::Default),
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
    let (x, y, width, height) = area_of(&app, "main");
    drag_block(&mut app, "notices", (x + width - 1, y + height / 2));
    assert_eq!(app.layout().lines()[2], "output as notices right of main");
    let (notices, output) = (area_of(&app, "notices"), area_of(&app, "main"));
    assert!(
        notices.0 > output.0 + output.2 - 1,
        "{notices:?} {output:?}"
    );
}

#[test]
fn a_drag_to_the_middle_stacks() {
    let mut app = laptop_with(NOTICES_BELOW);
    arrange(&mut app);
    let (x, y, width, height) = area_of(&app, "main");
    drag_block(&mut app, "notices", (x + width / 2, y + height / 2));
    assert_eq!(app.layout().lines()[2], "output as notices with main");
}

#[test]
fn the_drop_part_is_marked() {
    let mut app = laptop_with(NOTICES_BELOW);
    arrange(&mut app);
    let (x, y, width, height) = area_of(&app, "main");
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

const ANCHORED: &[&str] = &[
    "output",
    "output as notices right of main",
    "tabs above notices",
    "editor at bottom",
];

#[test]
fn a_change_that_writes_the_plan_again_gives_a_notice_at_the_save() {
    let mut app = laptop_with(ANCHORED);
    arrange(&mut app);
    // The letters follow the screen: main, tabs, notices, editor.
    keys(&mut app, "c-");
    assert!(
        !app.layout()
            .lines()
            .iter()
            .any(|line| line.contains("notices"))
    );
    let effects = key(&mut app, KeyCode::Enter, Modifiers::NONE);
    assert!(
        matches!(effects.as_slice(), [Effect::SaveLayout { .. }]),
        "{effects:?}"
    );
    assert!(
        app.notices().iter().any(|line| line.contains("plan")),
        "{:?}",
        app.notices()
    );
}

#[test]
fn a_new_configuration_ends_arrange_mode() {
    let mut app = laptop_with(&["output", "editor below main"]);
    arrange(&mut app);
    keys(&mut app, "b");
    let other = ["output", "tabs above main", "editor below main"];
    app.apply_config(GetClientConfigResponse {
        revision: "other".to_owned(),
        echo: "dim".to_owned(),
        layout_name: "other".to_owned(),
        layout: other.iter().map(ToString::to_string).collect(),
        tab_label: "{world}".to_owned(),
        ..Default::default()
    });
    assert!(!app.arranges());
    // Esc has no older layout to put under the new name.
    key(&mut app, KeyCode::Escape, Modifiers::NONE);
    assert_eq!(app.layout_name(), "other");
    assert_eq!(app.layout().lines(), other);
}

#[test]
fn a_border_that_cannot_move_says_why() {
    let mut app = laptop_with(&[
        "output as main showing main",
        "editor below main",
        "output as notices right of main, 50%",
    ]);
    draw_sized(&mut app, 20);
    let divider = area_of(&app, "notices").0 - 1;
    mouse(
        &mut app,
        MouseEventKind::Down(MouseButton::Left),
        divider,
        5,
    );
    mouse(
        &mut app,
        MouseEventKind::Drag(MouseButton::Left),
        divider - 3,
        5,
    );
    let effects = mouse(
        &mut app,
        MouseEventKind::Up(MouseButton::Left),
        divider - 3,
        5,
    );
    assert_eq!(effects, []);
    // The pane of `notices` is narrow, so the hint wraps: the test reads its words.
    let shown = rows_sized(&mut app, 20).join(" ");
    let plain = shown.replace('│', " ");
    let words: Vec<&str> = plain.split_whitespace().collect();
    assert!(words.join(" ").contains("has no room"), "{shown}");
}

/// The rows of a screen of `width` columns, with no trailing spaces.
fn rows_sized(app: &mut Model, width: u16) -> Vec<String> {
    let buffer = draw_sized(app, width);
    (0..buffer.area.height)
        .map(|y| {
            let row: String = (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect();
            row.trim_end().to_owned()
        })
        .collect()
}

/// The configuration of [`laptop_with`] for `lines`.
fn mine_config(lines: &[&str]) -> GetClientConfigResponse {
    GetClientConfigResponse {
        revision: format!("{lines:?}"),
        echo: "dim".to_owned(),
        layout_name: "mine".to_owned(),
        layout: lines.iter().map(ToString::to_string).collect(),
        tab_label: "{world}".to_owned(),
        statusline_left: vec!["world".to_owned()],
        ..Default::default()
    }
}

#[test]
fn a_configuration_with_the_same_revision_keeps_arrange_mode() {
    let lines = ["output", "editor below main"];
    let mut app = laptop_with(&lines);
    arrange(&mut app);
    // The fetch after a failed save can bring the same configuration again.
    app.apply_config(mine_config(&lines));
    assert!(app.arranges());
    assert!(
        !app.notices()
            .iter()
            .any(|line| line.contains("arrange mode ended"))
    );
}

#[test]
fn a_failed_save_ends_arrange_mode_with_its_own_reason() {
    let mut app = laptop_with(&["output", "editor below main"]);
    arrange(&mut app);
    let effects = update(&mut app, Msg::LayoutSaved(Err("down".to_owned())));
    assert_eq!(effects, [Effect::FetchConfig]);
    assert!(!app.arranges());
    let ended: Vec<&str> = app
        .notices()
        .into_iter()
        .filter(|line| line.contains("arrange mode ended"))
        .collect();
    assert_eq!(ended.len(), 1, "{:?}", app.notices());
    assert!(ended[0].contains("did not take the change"), "{ended:?}");
    assert!(!ended[0].contains("configuration changed"), "{ended:?}");
}

#[test]
fn a_focus_move_ends_arrange_mode_and_puts_the_layout_back() {
    let lines = ["output", "tabs above main", "editor below main"];
    let mut app = laptop_with(&lines);
    arrange(&mut app);
    // The letters follow the screen: tabs, main, editor. The key `-` hides the tabs.
    keys(&mut app, "a-");
    assert_ne!(app.layout().lines(), lines);
    update(
        &mut app,
        Msg::Run {
            entry: mu_tui::EntryId::named("focus-next").expect("an entry"),
            arg: None,
        },
    );
    assert!(!app.arranges());
    // The layout is one layout for both worlds here, and it is the one from before the mode.
    assert_eq!(app.layout().lines(), lines);
    assert!(
        app.notices()
            .iter()
            .any(|line| line.contains("the focus moved") && line.contains("arrange mode ended")),
        "{:?}",
        app.notices()
    );
}

#[test]
fn a_move_keeps_the_selection() {
    let mut app = laptop_with(&[
        "output",
        "editor below main",
        "output as comms of all worlds showing chan-*, !chan-ooc below main",
    ]);
    arrange(&mut app);
    let (x, y, width, height) = area_of(&app, "main");
    drag_block(&mut app, "comms", (x + width - 1, y + height / 2));
    assert_eq!(
        app.layout().lines()[2],
        "output as comms of all worlds showing chan-*, !chan-ooc right of main"
    );
}

/// Types `text` and presses Enter.
#[test]
fn a_place_command_keeps_the_streams() {
    let mut app = laptop_with(&[
        "output",
        "editor below main",
        "output as comms of all worlds showing chan-* below main",
    ]);
    run_command(&mut app, "/place output as comms right of main");
    assert_eq!(
        app.layout().lines()[2],
        "output as comms of all worlds showing chan-* right of main"
    );
}
