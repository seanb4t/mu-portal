//! `update` for keys, mouse presses, client events, the probe result and late terminal replies.

use mu_client::ClientEvent;
use mu_proto::{Event, EventKind, Source};
use std::num::NonZeroU16;
use std::time::Duration;

use mu_tui::probe::{Outcome, PENDING_MAX};
use mu_tui::store::CAPACITY;
use mu_tui::width::Profile;
use mu_tui::{Effect, Model, Msg, Position, update, view};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::CellDiffOption;
use termina::Event as TermEvent;
use termina::event::{KeyCode, Modifiers, MouseButton, MouseEvent, MouseEventKind};

fn model() -> Model {
    Model::new(["alpha/aria".to_owned(), "beta/bryn".to_owned()])
}

use support::{press, type_text};

mod support;

#[test]
fn ctrl_c_returns_quit_and_keeps_the_model() {
    let mut model = model();
    type_text(&mut model, "hi");
    let effects = press(&mut model, KeyCode::Char('c'), Modifiers::CONTROL);
    assert_eq!(effects, [Effect::Quit]);
    assert_eq!(
        (model.focused(), model.input().as_str()),
        (Some("alpha/aria"), "hi")
    );
}

#[test]
fn ctrl_n_moves_the_focus_with_no_effect() {
    let mut model = model();
    let effects = press(&mut model, KeyCode::Char('n'), Modifiers::CONTROL);
    assert_eq!(effects, []);
    assert_eq!(model.focused(), Some("beta/bryn"));
}

#[test]
fn ctrl_p_moves_the_focus_back_with_no_effect() {
    let mut model = Model::new(["alpha/aria", "beta/bryn", "gamma/gale"].map(str::to_owned));
    let effects = press(&mut model, KeyCode::Char('p'), Modifiers::CONTROL);
    assert_eq!((effects, model.focused()), (vec![], Some("gamma/gale")));
}

#[test]
fn enter_returns_send_and_clears_the_input_line() {
    let mut model = model();
    type_text(&mut model, "look");
    let effects = press(&mut model, KeyCode::Enter, Modifiers::NONE);
    assert_eq!(
        effects,
        [Effect::Send {
            world: "alpha/aria".to_owned(),
            text: "look".to_owned()
        }]
    );
    assert_eq!(model.input(), "");
}

fn game_lines(model: &mut Model, lines: std::ops::Range<usize>) {
    for n in lines {
        let event = Event {
            world: "alpha/aria".to_owned(),
            kind: EventKind::EVENT_KIND_LINE.into(),
            source: Source::SOURCE_GAME.into(),
            text: n.to_string(),
            ..Default::default()
        };
        assert_eq!(update(model, Msg::Client(ClientEvent::Event(event))), []);
    }
}

#[test]
fn press_after_a_trim() {
    let mut model = model();
    game_lines(&mut model, 0..CAPACITY);
    let mut terminal = Terminal::new(TestBackend::new(20, 5)).unwrap();
    terminal.draw(|frame| view(&mut model, frame)).unwrap();
    game_lines(&mut model, 0..CAPACITY);
    let press = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 0,
        row: 1,
        modifiers: Modifiers::NONE,
    };
    assert_eq!(
        update(&mut model, Msg::Terminal(TermEvent::Mouse(press))),
        []
    );
    let pane = model.pane(model.focused_pane()).unwrap();
    let oldest = Position {
        line: pane.store().first_id(),
        byte: 0,
    };
    assert_eq!(pane.selection(), Some((oldest, oldest)));
}

/// Feeds `bytes` to `model` as the Termina parser gives them.
fn feed(model: &mut Model, bytes: &[u8]) {
    let mut parser = termina::Parser::default();
    parser.parse(bytes, false);
    for event in std::iter::from_fn(|| parser.pop()) {
        update(model, Msg::Terminal(event));
    }
}

#[test]
fn late_graphics_reply_does_not_reach_the_input_line() {
    let mut model = model();
    probe(&mut model, &incomplete());
    feed(&mut model, b"\x1b_Gi=31;OK\x1b\\\x1b[?62;22c");
    type_text(&mut model, "hi");
    assert_eq!(model.input(), "hi");
}

#[test]
fn alt_underscore_with_no_reply_after_it_keeps_the_next_key() {
    let mut model = model();
    probe(&mut model, &incomplete());
    press(&mut model, KeyCode::Char('_'), Modifiers::ALT);
    type_text(&mut model, "hi");
    assert_eq!(model.input(), "hi");
}

#[test]
fn reply_after_da1_is_input() {
    let mut model = model();
    probe(&mut model, &incomplete());
    feed(&mut model, b"\x1b[?62;22c\x1b_Gx\x1b\\");
    assert_eq!(model.input(), "Gx");
}

/// Sends the probe result with no DA1, then `bytes` as the cut answer.
fn cut_at(model: &mut Model, bytes: &[u8]) {
    probe(model, &incomplete());
    update(model, Msg::CutReply(bytes.to_vec()));
}

#[test]
fn cut_da1_ends_the_wait() {
    let mut model = model();
    cut_at(&mut model, b"\x1b[?62;2");
    type_text(&mut model, "2");
    press(&mut model, KeyCode::Char('c'), Modifiers::NONE);
    assert_eq!(model.input(), "");
    feed(&mut model, b"\x1b_Gx\x1b\\");
    assert_eq!(model.input(), "Gx");
}

#[test]
fn cut_cursor_reply_is_not_typed() {
    let mut model = model();
    cut_at(&mut model, b"\x1b[12;");
    type_text(&mut model, "5");
    press(&mut model, KeyCode::Char('R'), Modifiers::SHIFT);
    assert_eq!(model.input(), "");
}

#[test]
fn other_key_drops_a_cut_reply() {
    let mut model = model();
    cut_at(&mut model, b"\x1b[1");
    type_text(&mut model, "2");
    assert_eq!(model.input(), "");
    press(&mut model, KeyCode::Char('n'), Modifiers::CONTROL);
    type_text(&mut model, "c");
    assert_eq!(
        (model.focused(), model.input().as_str()),
        (Some("beta/bryn"), "c")
    );
}

#[test]
fn lone_esc_then_bracket_continues_a_cut_reply() {
    let mut model = model();
    cut_at(&mut model, b"\x1b");
    type_text(&mut model, "[?62");
    press(&mut model, KeyCode::Char('c'), Modifiers::NONE);
    assert_eq!(model.input(), "");
    feed(&mut model, b"\x1b_Gx\x1b\\");
    assert_eq!(model.input(), "Gx");
}

#[test]
fn cut_reply_past_the_cap_is_dropped() {
    let mut model = model();
    let held = [b"\x1b[".as_slice(), &vec![b'1'; PENDING_MAX - 2]].concat();
    cut_at(&mut model, &held);
    type_text(&mut model, "1");
    press(&mut model, KeyCode::Char('c'), Modifiers::NONE);
    assert_eq!(model.input(), "1c");
}

#[test]
fn early_keys_come_before_a_cut_reply() {
    let mut model = model();
    probe(&mut model, &incomplete());
    type_text(&mut model, "x");
    update(&mut model, Msg::CutReply(b"\x1b[12;".to_vec()));
    type_text(&mut model, "5");
    press(&mut model, KeyCode::Char('R'), Modifiers::SHIFT);
    assert_eq!(model.input(), "x");
}

#[test]
fn graphics_reply_cut_at_its_end() {
    let mut model = model();
    probe(&mut model, &incomplete());
    press(&mut model, KeyCode::Char('_'), Modifiers::ALT);
    type_text(&mut model, "Gi=31;OK");
    press(&mut model, KeyCode::Escape, Modifiers::NONE);
    press(&mut model, KeyCode::Char('\\'), Modifiers::NONE);
    assert_eq!(model.input(), "");
}

#[test]
fn lone_esc_then_underscore_starts_a_graphics_reply() {
    let mut model = model();
    cut_at(&mut model, b"\x1b");
    type_text(&mut model, "_Gi=31;OK");
    press(&mut model, KeyCode::Char('\\'), Modifiers::ALT);
    type_text(&mut model, "hi");
    assert_eq!(model.input(), "hi");
}

#[test]
fn escape_after_alt_underscore_keeps_the_next_key() {
    let mut model = model();
    probe(&mut model, &incomplete());
    press(&mut model, KeyCode::Char('_'), Modifiers::ALT);
    press(&mut model, KeyCode::Escape, Modifiers::NONE);
    type_text(&mut model, "x");
    assert_eq!(model.input(), "x");
}

/// The notice that `Msg::Probe` adds when the outcome has no widths.
const WIDTH_NOTICE: &str =
    "-- the terminal gave no character widths, so wide characters can be out of line";

/// The count of rows of `model` on an 80 by 4 screen that read the width notice.
fn notice_rows(model: &mut Model) -> usize {
    let mut terminal = Terminal::new(TestBackend::new(80, 4)).unwrap();
    terminal.draw(|frame| view(model, frame)).unwrap();
    (0..4)
        .filter(|&y| {
            let row: String = (0..80)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                .collect();
            row.trim_end() == WIDTH_NOTICE
        })
        .count()
}

/// Outcome with no DA1 answer and no measured width.
fn incomplete() -> Outcome {
    Outcome::default()
}

/// Outcome with a DA1 answer and every width measured.
fn complete() -> Outcome {
    Outcome {
        complete: true,
        measured: true,
        ..Outcome::default()
    }
}

fn probe(model: &mut Model, outcome: &Outcome) {
    assert_eq!(update(model, Msg::Probe(outcome.clone())), []);
}

fn reset() -> Msg {
    Msg::Client(ClientEvent::Reset {
        instance_id: "i".to_owned(),
    })
}

#[test]
fn notice_with_no_world_shows_once_after_a_reset() {
    let mut model = Model::new([]);
    probe(&mut model, &Outcome::default());
    update(&mut model, reset());
    assert_eq!(notice_rows(&mut model), 1);
}

#[test]
fn notice_comes_back_after_a_reset() {
    let mut model = model();
    probe(&mut model, &Outcome::default());
    update(&mut model, reset());
    assert_eq!(notice_rows(&mut model), 1);
}

#[test]
fn unmeasured_probe_shows_the_width_notice() {
    let mut model = model();
    probe(&mut model, &Outcome::default());
    assert_eq!(notice_rows(&mut model), 1);
}

#[test]
fn measured_probe_shows_no_width_notice() {
    let mut model = model();
    probe(&mut model, &complete());
    assert_eq!(notice_rows(&mut model), 0);
}

#[test]
fn incomplete_probe_waits_for_late_replies() {
    let mut model = model();
    probe(&mut model, &incomplete());
    feed(&mut model, b"\x1b_Gi=31;OK\x1b\\");
    feed(&mut model, b"\x1b[?62;22c");
    type_text(&mut model, "hi");
    assert_eq!(model.input(), "hi");
}

#[test]
fn complete_probe_does_not_wait_for_late_replies() {
    let mut model = model();
    probe(&mut model, &complete());
    feed(&mut model, b"\x1b_Gx\x1b\\");
    assert_ne!(model.input(), "");
}

/// True if the first cell of the first row that starts with `①` has its width forced to 2.
fn forced_width_of_circled_one(model: &mut Model) -> bool {
    let mut terminal = Terminal::new(TestBackend::new(20, 4)).unwrap();
    terminal.draw(|frame| view(model, frame)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..4)
        .find(|&y| buffer[(0, y)].symbol() == "①")
        .is_some_and(|y| {
            buffer[(0, y)].diff_option == CellDiffOption::ForcedWidth(NonZeroU16::new(2).unwrap())
        })
}

#[test]
fn probe_sets_the_caps_of_every_pane() {
    let mut model = model();
    for world in ["alpha/aria", "beta/bryn"] {
        let event = Event {
            world: world.to_owned(),
            kind: EventKind::EVENT_KIND_LINE.into(),
            source: Source::SOURCE_GAME.into(),
            text: "①x".to_owned(),
            ..Default::default()
        };
        update(&mut model, Msg::Client(ClientEvent::Event(event)));
    }
    probe(
        &mut model,
        &Outcome {
            profile: Profile::learn(&[("①", 2)]),
            ..complete()
        },
    );
    let first = forced_width_of_circled_one(&mut model);
    press(&mut model, KeyCode::Char('n'), Modifiers::CONTROL);
    let second = forced_width_of_circled_one(&mut model);
    assert_eq!((first, second), (true, true));
}

#[test]
fn late_mode_reply_as_set_keeps_mode_2027() {
    let mut model = model();
    probe(&mut model, &incomplete());
    feed(&mut model, b"\x1b[?2027;1$y\x1b[?62;22c");
    assert!(model.keeps_grapheme_mode());
}

#[test]
fn mode_reply_with_no_late_wait_changes_nothing() {
    let mut model = model();
    feed(&mut model, b"\x1b[?2027;1$y");
    assert!(!model.keeps_grapheme_mode());
}

// The candidates of an argument and the place of the cursor in a command.

#[test]
fn world_candidates_follow_the_top_row() {
    let model = Model::new(["beta/bryn".to_owned(), "alpha/aria".to_owned()]);
    assert_eq!(
        mu_tui::candidates(&model, mu_tui::ArgKind::World),
        ["alpha/aria", "beta/bryn"]
    );
}

#[test]
fn parse_knows_the_place_of_the_cursor() {
    use mu_tui::command::{CursorAt, parse};
    let text = "/focus  beta/bryn ";
    let places: Vec<CursorAt> = [0, 6, 7, 8, 17, 18]
        .into_iter()
        .map(|cursor| parse(text, cursor).expect("a command").cursor)
        .collect();
    assert_eq!(
        (parse(text, 0).and_then(|parse| parse.arg), places),
        (
            Some("beta/bryn"),
            vec![
                CursorAt::Name,
                CursorAt::Name,
                CursorAt::Arg,
                CursorAt::Arg,
                CursorAt::Arg,
                CursorAt::Past
            ]
        )
    );
}

// The parse of a command and its checks.

/// The editor text, the effects and the last row of the screen after `command` and Enter.
fn command(text: &str) -> (String, Vec<Effect>, String) {
    let mut model = model();
    type_text(&mut model, text);
    let effects = press(&mut model, KeyCode::Enter, Modifiers::NONE);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("a test terminal");
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("a frame");
    let rows: Vec<String> = terminal
        .backend()
        .buffer()
        .content()
        .chunks(80)
        .map(|row| row.iter().map(ratatui::buffer::Cell::symbol).collect())
        .collect();
    let notice = rows
        .iter()
        .rev()
        .find(|row| row.starts_with("-- "))
        .map_or_else(String::new, |row| row.trim_end().to_owned());
    (model.input(), effects, notice)
}

#[test]
fn connect_with_a_trailing_space_takes_the_world_in_focus() {
    let (input, effects, _) = command("/connect ");
    assert_eq!(
        (input.as_str(), effects),
        (
            "",
            vec![Effect::Connect {
                world: "alpha/aria".to_owned()
            }]
        )
    );
}

#[test]
fn a_slash_alone_is_an_unknown_command() {
    let (input, effects, notice) = command("/");
    assert_eq!(
        (input.as_str(), effects, notice.contains("unknown command")),
        ("/", vec![], true)
    );
}

#[test]
fn a_space_after_the_slash_is_an_unknown_command() {
    let (input, effects, notice) = command("/ focus");
    assert_eq!(
        (input.as_str(), effects, notice.contains("unknown command")),
        ("/ focus", vec![], true)
    );
}

#[test]
fn quit_with_an_argument_is_a_bad_command() {
    let (input, effects, notice) = command("/quit now");
    assert_eq!(
        (
            input.as_str(),
            effects,
            notice.contains("takes no argument")
        ),
        ("/quit now", vec![], true)
    );
}

#[test]
fn ignore_is_not_a_command() {
    let (input, effects, notice) = command("/ignore");
    assert_eq!(
        (input.as_str(), effects, notice.contains("not a command")),
        ("/ignore", vec![], true)
    );
}

proptest::proptest! {
    #[test]
    fn trailing_spaces_are_no_argument(spaces in 0usize..8, world in proptest::bool::ANY) {
        let (text, expected) = if world {
            (format!("/disconnect beta/bryn{}", " ".repeat(spaces)), "beta/bryn")
        } else {
            (format!("/disconnect{}", " ".repeat(spaces)), "alpha/aria")
        };
        let (input, effects, _) = command(&text);
        proptest::prop_assert_eq!(
            (input.as_str(), effects),
            ("", vec![Effect::Disconnect { world: expected.to_owned() }])
        );
    }
}

// The live change of the client configuration.

/// A client configuration with `hash`, `revision` and a notice for each of `notices`.
fn reply(hash: &str, revision: &str, notices: &[&str]) -> mu_proto::GetClientConfigResponse {
    mu_proto::GetClientConfigResponse {
        store_hash: hash.to_owned(),
        revision: revision.to_owned(),
        echo: "dim".to_owned(),
        notices: notices
            .iter()
            .map(|text| mu_proto::Notice {
                path: "clients/global.toml".to_owned(),
                text: (*text).to_owned(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

/// A model that applied `first`.
fn configured(first: mu_proto::GetClientConfigResponse) -> Model {
    let mut model = model();
    model.apply_config(first);
    model
}

fn store_hash(model: &mut Model, hash: &str) -> Vec<Effect> {
    update(model, Msg::Client(ClientEvent::StoreHash(hash.to_owned())))
}

/// Sends the store hash of `reply`, then `reply`.
fn new_config(model: &mut Model, reply: mu_proto::GetClientConfigResponse) {
    store_hash(model, &reply.store_hash);
    update(model, Msg::Config(Ok(reply)));
}

fn revision(model: &Model) -> Option<&str> {
    model.config().map(|config| config.revision.as_str())
}

/// How many notice lines hold `text`.
fn notices_of(model: &Model, text: &str) -> usize {
    model
        .notices()
        .iter()
        .filter(|line| line.contains(text))
        .count()
}

#[test]
fn the_same_hash_fetches_nothing() {
    let mut model = configured(reply("h1", "r1", &[]));
    assert_eq!(store_hash(&mut model, "h1"), []);
}

#[test]
fn a_new_hash_fetches_the_configuration() {
    let mut model = configured(reply("h1", "r1", &[]));
    assert_eq!(store_hash(&mut model, "h2"), [Effect::FetchConfig]);
}

#[test]
fn a_stale_reply_is_dropped() {
    let mut model = configured(reply("h1", "r1", &[]));
    store_hash(&mut model, "h2");
    store_hash(&mut model, "h3");
    update(&mut model, Msg::Config(Ok(reply("h2", "r2", &[]))));
    let stale = revision(&model).map(str::to_owned);
    update(&mut model, Msg::Config(Ok(reply("h3", "r3", &[]))));
    assert_eq!(
        (stale.as_deref(), revision(&model)),
        (Some("r1"), Some("r3"))
    );
}

#[test]
fn the_same_revision_changes_nothing() {
    let mut model = configured(reply("h1", "r1", &[]));
    new_config(&mut model, reply("h2", "r1", &["clients/global.toml: bad"]));
    assert_eq!(notices_of(&model, "bad"), 0);
}

#[test]
fn a_notice_shows_once_across_two_revisions() {
    let mut model = configured(reply("h1", "r1", &["laptop.toml: bad"]));
    new_config(&mut model, reply("h2", "r2", &["laptop.toml: bad"]));
    assert_eq!(notices_of(&model, "laptop.toml: bad"), 1);
}

#[test]
fn a_notice_shows_again_after_a_fix() {
    let mut model = configured(reply("h1", "r1", &["laptop.toml: bad"]));
    new_config(&mut model, reply("h2", "r2", &[]));
    new_config(&mut model, reply("h3", "r3", &["laptop.toml: bad"]));
    assert_eq!(notices_of(&model, "laptop.toml: bad"), 2);
}

#[test]
fn the_attach_notices_seed_the_last_reply() {
    let mut model = Model::starting();
    model.apply_config(reply("h1", "r1", &["laptop.toml: bad"]));
    let at_attach = notices_of(&model, "laptop.toml: bad");
    new_config(&mut model, reply("h2", "r2", &["laptop.toml: bad"]));
    assert_eq!((at_attach, notices_of(&model, "laptop.toml: bad")), (1, 1));
}

/// One message of the live change: a store hash, or the reply of that hash.
#[derive(Clone, Debug)]
enum Step {
    Hash(u8),
    Reply(u8),
}

fn step() -> impl proptest::strategy::Strategy<Value = Step> {
    use proptest::prelude::*;
    prop_oneof![
        (0_u8..4).prop_map(Step::Hash),
        (0_u8..4).prop_map(Step::Reply)
    ]
}

proptest::proptest! {
    #[test]
    fn the_latest_hash_wins(steps in proptest::collection::vec(step(), 0..24)) {
        let mut model = configured(reply("start", "start", &[]));
        let mut latest = None;
        for each in steps {
            match each {
                Step::Hash(n) => {
                    latest = Some(format!("h{n}"));
                    store_hash(&mut model, &format!("h{n}"));
                }
                Step::Reply(n) => {
                    let before = revision(&model).map(str::to_owned);
                    update(
                        &mut model,
                        Msg::Config(Ok(reply(&format!("h{n}"), &format!("r{n}"), &[]))),
                    );
                    if revision(&model).map(str::to_owned) != before {
                        let applied = model.config().map(|config| config.store_hash.clone());
                        proptest::prop_assert_eq!(applied, latest.clone());
                    }
                }
            }
        }
    }
}

#[test]
fn a_failed_fetch_fetches_again() {
    let mut model = configured(reply("h1", "r1", &[]));
    store_hash(&mut model, "h2");
    let failed = update(&mut model, Msg::Config(Err("unavailable".to_owned())));
    let again = update(&mut model, Msg::RetryConfig);
    assert_eq!(
        (failed, again, notices_of(&model, "unavailable")),
        (
            vec![Effect::RetryConfig {
                after: Duration::from_millis(100)
            }],
            vec![Effect::FetchConfig],
            1
        )
    );
}

#[test]
fn each_failed_fetch_waits_longer_until_an_apply() {
    let mut model = configured(reply("h1", "r1", &[]));
    store_hash(&mut model, "h2");
    let mut effects = Vec::new();
    for _ in 0..3 {
        effects.extend(update(
            &mut model,
            Msg::Config(Err("unavailable".to_owned())),
        ));
        update(&mut model, Msg::RetryConfig);
    }
    update(&mut model, Msg::Config(Ok(reply("h2", "r2", &[]))));
    store_hash(&mut model, "h3");
    effects.extend(update(
        &mut model,
        Msg::Config(Err("unavailable".to_owned())),
    ));
    let millis: Vec<Option<u128>> = effects
        .iter()
        .map(|effect| match effect {
            Effect::RetryConfig { after } => Some(after.as_millis()),
            _ => None,
        })
        .collect();
    assert_eq!(
        (millis, notices_of(&model, "unavailable")),
        (vec![Some(100), Some(200), Some(400), Some(100)], 2)
    );
}

#[test]
fn a_failed_fetch_of_an_applied_hash_fetches_nothing() {
    let mut model = configured(reply("h1", "r1", &[]));
    store_hash(&mut model, "h1");
    let failed = update(&mut model, Msg::Config(Err("unavailable".to_owned())));
    let again = update(&mut model, Msg::RetryConfig);
    assert_eq!((failed, again), (vec![], vec![]));
}

#[test]
fn a_late_failure_of_an_old_fetch_does_not_count() {
    let mut model = configured(reply("h1", "r1", &[]));
    store_hash(&mut model, "h2");
    update(&mut model, Msg::Config(Ok(reply("h2", "r2", &[]))));
    let late = update(&mut model, Msg::Config(Err("old".to_owned())));
    store_hash(&mut model, "h3");
    let failed = update(&mut model, Msg::Config(Err("unavailable".to_owned())));
    assert_eq!(
        (
            late,
            failed,
            notices_of(&model, "old"),
            notices_of(&model, "unavailable")
        ),
        (
            vec![],
            vec![Effect::RetryConfig {
                after: Duration::from_millis(100)
            }],
            0,
            1
        )
    );
}
