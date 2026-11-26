//! `update` for keys, mouse presses, client events, the probe result and late terminal replies.

use mu_client::ClientEvent;
use mu_proto::{Event, EventKind, Source};
use std::num::NonZeroU16;
use std::time::Duration;

use mu_tui::probe::{Outcome, PENDING_MAX};
use mu_tui::store::CAPACITY;
use mu_tui::width::Profile;
use mu_tui::{Effect, Model, Msg, Position, Timer, update, view};
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
    update(model, Msg::Config(Ok(Box::new(reply))));
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
    update(
        &mut model,
        Msg::Config(Ok(Box::new(reply("h2", "r2", &[])))),
    );
    let stale = revision(&model).map(str::to_owned);
    update(
        &mut model,
        Msg::Config(Ok(Box::new(reply("h3", "r3", &[])))),
    );
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
                        Msg::Config(Ok(Box::new(reply(&format!("h{n}"), &format!("r{n}"), &[])))),
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
            vec![Effect::Wait {
                after: Duration::from_millis(100),
                then: Timer::RetryConfig,
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
    update(
        &mut model,
        Msg::Config(Ok(Box::new(reply("h2", "r2", &[])))),
    );
    store_hash(&mut model, "h3");
    effects.extend(update(
        &mut model,
        Msg::Config(Err("unavailable".to_owned())),
    ));
    let millis: Vec<Option<u128>> = effects
        .iter()
        .map(|effect| match effect {
            Effect::Wait { after, .. } => Some(after.as_millis()),
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
    update(
        &mut model,
        Msg::Config(Ok(Box::new(reply("h2", "r2", &[])))),
    );
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
            vec![Effect::Wait {
                after: Duration::from_millis(100),
                then: Timer::RetryConfig,
            }],
            0,
            1
        )
    );
}

#[test]
fn a_hash_back_to_the_applied_one_resets_the_failures() {
    let mut model = configured(reply("h1", "r1", &[]));
    store_hash(&mut model, "h2");
    update(&mut model, Msg::Config(Err("unavailable".to_owned())));
    update(&mut model, Msg::RetryConfig);
    store_hash(&mut model, "h1");
    store_hash(&mut model, "h3");
    let failed = update(&mut model, Msg::Config(Err("unavailable".to_owned())));
    assert_eq!(
        (failed, notices_of(&model, "unavailable")),
        (
            vec![Effect::Wait {
                after: Duration::from_millis(100),
                then: Timer::RetryConfig,
            }],
            2
        )
    );
}

// The world state, the times and the tick of the clock.

use mu_proto::{GetClientConfigResponse, NamedLayout};
use proptest::prelude::*;

const CLASSIC: &[&str] = &["output", "tabs above main", "editor below main"];
const STATUS: &[&str] = &[
    "output",
    "tabs above main",
    "editor below main",
    "statusline below editor",
];

fn texts(lines: &[&str]) -> Vec<String> {
    lines.iter().map(ToString::to_string).collect()
}

/// A configuration with the layout `lines`, the layout of each key of `worlds`, and the two
/// lists of segments.
fn layout_config(
    lines: &[&str],
    worlds: &[(&str, &[&str])],
    left: &[&str],
    right: &[&str],
) -> GetClientConfigResponse {
    GetClientConfigResponse {
        store_hash: "h".to_owned(),
        revision: format!("{lines:?}{worlds:?}{left:?}{right:?}"),
        echo: "dim".to_owned(),
        layout_name: "test".to_owned(),
        layout: texts(lines),
        tab_label: "{world}".to_owned(),
        statusline_left: texts(left),
        statusline_right: texts(right),
        world_layouts: worlds
            .iter()
            .map(|(key, lines)| NamedLayout {
                key: (*key).to_owned(),
                name: "other".to_owned(),
                lines: texts(lines),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

/// Gives `config` to the model through `update`, and returns the effects of the reply.
fn configure(model: &mut Model, config: GetClientConfigResponse) -> Vec<Effect> {
    store_hash(model, &config.store_hash);
    update(model, Msg::Config(Ok(Box::new(config))))
}

/// A model of two worlds with the layout `status` and these segments. The tick of the clock
/// waits.
fn status_model(left: &[&str], right: &[&str]) -> Model {
    let mut model = model();
    configure(&mut model, layout_config(STATUS, &[], left, right));
    model
}

fn world_event(kind: EventKind, source: Source, time_ms: i64) -> Msg {
    Msg::Client(ClientEvent::Event(Event {
        world: "alpha/aria".to_owned(),
        kind: kind.into(),
        source: source.into(),
        time_ms,
        ..Default::default()
    }))
}

fn tick() -> Effect {
    Effect::Wait {
        after: mu_tui::TICK,
        then: Timer::Tick,
    }
}

fn status_row(model: &Model) -> String {
    mu_tui::statusline::row(model, 80)
}

#[test]
fn a_connect_event_sets_the_connect_time() {
    let mut model = status_model(&["connected"], &[]);
    update(
        &mut model,
        world_event(
            EventKind::EVENT_KIND_CONNECTED,
            Source::SOURCE_SYSTEM,
            10_000,
        ),
    );
    update(&mut model, Msg::Tick(13_000));
    assert_eq!(status_row(&model), "0:03");
}

#[test]
fn a_disconnect_event_clears_the_connect_time() {
    let mut model = status_model(&["connected"], &[]);
    update(
        &mut model,
        world_event(
            EventKind::EVENT_KIND_CONNECTED,
            Source::SOURCE_SYSTEM,
            10_000,
        ),
    );
    update(
        &mut model,
        world_event(
            EventKind::EVENT_KIND_DISCONNECTED,
            Source::SOURCE_SYSTEM,
            12_000,
        ),
    );
    update(&mut model, Msg::Tick(13_000));
    assert_eq!(status_row(&model), "");
}

#[test]
fn a_send_from_any_client_sets_the_send_time() {
    let mut model = status_model(&[], &["idle"]);
    update(
        &mut model,
        world_event(EventKind::EVENT_KIND_INPUT, Source::SOURCE_PLAYER, 5_000),
    );
    update(&mut model, Msg::Tick(8_000));
    assert!(
        status_row(&model).ends_with("0:03"),
        "{:?}",
        status_row(&model)
    );
}

#[test]
fn a_connect_event_lists_again() {
    let mut model = model();
    let effects = update(
        &mut model,
        world_event(
            EventKind::EVENT_KIND_CONNECTED,
            Source::SOURCE_SYSTEM,
            10_000,
        ),
    );
    assert_eq!(effects, [Effect::ListWorlds]);
}

#[test]
fn a_reset_clears_the_times() {
    let mut model = status_model(&["connected"], &["idle"]);
    update(
        &mut model,
        world_event(
            EventKind::EVENT_KIND_CONNECTED,
            Source::SOURCE_SYSTEM,
            10_000,
        ),
    );
    update(
        &mut model,
        world_event(EventKind::EVENT_KIND_INPUT, Source::SOURCE_PLAYER, 11_000),
    );
    update(
        &mut model,
        Msg::Client(ClientEvent::Reset {
            instance_id: "i".to_owned(),
        }),
    );
    update(&mut model, Msg::Tick(13_000));
    assert_eq!(status_row(&model), "");
}

#[test]
fn a_tick_moves_the_idle_time() {
    let mut model = status_model(&[], &["idle"]);
    update(
        &mut model,
        world_event(EventKind::EVENT_KIND_INPUT, Source::SOURCE_PLAYER, 20_000),
    );
    update(&mut model, Msg::Tick(23_000));
    assert!(
        status_row(&model).ends_with("0:03"),
        "{:?}",
        status_row(&model)
    );
    update(&mut model, Msg::Tick(24_000));
    assert!(
        status_row(&model).ends_with("0:04"),
        "{:?}",
        status_row(&model)
    );
}

#[test]
fn no_tick_without_a_time_segment() {
    let mut model = model();
    assert_eq!(
        configure(
            &mut model,
            layout_config(CLASSIC, &[], &["world"], &["idle"])
        ),
        []
    );
    assert_eq!(update(&mut model, Msg::Tick(1_000)), []);
    // The layout `status` with no time segment asks for no tick.
    assert_eq!(
        configure(
            &mut model,
            layout_config(STATUS, &[], &["world"], &["typed"])
        ),
        []
    );
}

#[test]
fn the_first_clock_read_starts_no_second_tick() {
    let mut model = model();
    assert_eq!(
        configure(
            &mut model,
            layout_config(STATUS, &[], &["world"], &["idle"])
        ),
        [tick()]
    );
    // The wait of the tick is pending. The first read of the clock sets the time only.
    assert_eq!(update(&mut model, Msg::Clock(5_000)), []);
    assert_eq!(press(&mut model, KeyCode::Char('a'), Modifiers::NONE), []);
    // The end of the one wait starts the next one.
    assert_eq!(update(&mut model, Msg::Tick(6_000)), [tick()]);
}

#[test]
fn one_tick_waits_at_a_time() {
    let mut model = model();
    assert_eq!(
        configure(
            &mut model,
            layout_config(STATUS, &[], &["world"], &["idle"])
        ),
        [tick()]
    );
    assert_eq!(press(&mut model, KeyCode::Char('a'), Modifiers::NONE), []);
    assert_eq!(update(&mut model, Msg::Tick(1_000)), [tick()]);
}

#[test]
fn a_new_layout_with_a_time_segment_starts_the_tick() {
    let mut model = model();
    assert_eq!(
        configure(
            &mut model,
            layout_config(CLASSIC, &[], &["world"], &["idle"])
        ),
        []
    );
    assert_eq!(
        configure(
            &mut model,
            layout_config(STATUS, &[], &["world"], &["idle"])
        ),
        [tick()]
    );
}

#[test]
fn a_focus_change_to_a_status_layout_starts_the_tick() {
    let mut model = model();
    let config = layout_config(CLASSIC, &[("beta/bryn", STATUS)], &["world"], &["idle"]);
    assert_eq!(configure(&mut model, config), []);
    assert_eq!(
        press(&mut model, KeyCode::Char('n'), Modifiers::CONTROL),
        [tick()]
    );
}

#[test]
fn a_tick_takes_the_time_of_the_clock() {
    let message = mu_tui::ended(Timer::Tick, || 42_000);
    assert!(matches!(message, Msg::Tick(42_000)), "{message:?}");
    let retry = mu_tui::ended(Timer::RetryConfig, || 42_000);
    assert!(matches!(retry, Msg::RetryConfig), "{retry:?}");
}

/// One step of the property test of the input focus.
#[derive(Clone, Debug)]
enum FocusStep {
    Layout(usize),
    EditorNext,
    WorldNext,
    WorldPrev,
}

const EDITOR_LAYOUTS: &[&[&str]] = &[
    &["output", "editor below main"],
    &[
        "output",
        "editor below main",
        "editor as scratch right of editor",
    ],
    &[
        "output",
        "editor as scratch below main",
        "editor as notes right of scratch",
        "editor right of notes",
    ],
];

fn focus_step() -> impl Strategy<Value = FocusStep> {
    prop_oneof![
        (0..EDITOR_LAYOUTS.len()).prop_map(FocusStep::Layout),
        Just(FocusStep::EditorNext),
        Just(FocusStep::WorldNext),
        Just(FocusStep::WorldPrev),
    ]
}

proptest! {
    #[test]
    fn the_input_focus_is_an_editor_of_the_plan(
        steps in prop::collection::vec(focus_step(), 0..40),
        other in 0..EDITOR_LAYOUTS.len(),
    ) {
        let mut model = model();
        for step in steps {
            match step {
                FocusStep::Layout(index) => {
                    // The world `beta/bryn` has a layout of its own.
                    let config = layout_config(
                        EDITOR_LAYOUTS[index],
                        &[("beta/bryn", EDITOR_LAYOUTS[other])],
                        &["world"],
                        &[],
                    );
                    configure(&mut model, config);
                }
                FocusStep::EditorNext => {
                    press(&mut model, KeyCode::Char('e'), Modifiers::ALT);
                }
                FocusStep::WorldNext => {
                    press(&mut model, KeyCode::Char('n'), Modifiers::CONTROL);
                }
                FocusStep::WorldPrev => {
                    press(&mut model, KeyCode::Char('p'), Modifiers::CONTROL);
                }
            }
            let input = model.input_id();
            prop_assert!(
                model
                    .layout()
                    .holds(mu_config::layout::BlockKind::Editor, &input),
                "{} is no editor of {:?}",
                input,
                model.layout().lines()
            );
        }
    }
}

// The commands of the layout and the save.

use mu_config::Choice;

/// The choice of the entry of the world `alpha/aria`.
fn aria_entry() -> Choice {
    Choice::Entry("alpha/aria".parse().expect("a world key"))
}

/// Types `text` and presses Enter, and returns the effects of the Enter.
fn run_command(model: &mut Model, text: &str) -> Vec<Effect> {
    type_text(model, text);
    press(model, KeyCode::Enter, Modifiers::NONE)
}

fn laptop() -> Model {
    model().with_device("laptop")
}

#[test]
fn a_block_argument_lists_the_ids_of_the_plan() {
    let mut model = laptop();
    configure(
        &mut model,
        layout_config(EDITOR_LAYOUTS[1], &[], &["world"], &[]),
    );
    assert_eq!(
        mu_tui::candidates(&model, mu_tui::ArgKind::Block),
        ["main", "editor", "scratch"]
    );
}

#[test]
fn a_layout_argument_lists_the_layouts() {
    let mut model = laptop();
    assert_eq!(
        mu_tui::candidates(&model, mu_tui::ArgKind::Layout),
        ["classic", "focus", "sidebar", "status"]
    );
    update(
        &mut model,
        Msg::Layouts {
            next: false,
            names: Ok(vec!["classic".to_owned(), "mine".to_owned()]),
        },
    );
    assert_eq!(
        mu_tui::candidates(&model, mu_tui::ArgKind::Layout),
        ["classic", "mine"]
    );
}

#[test]
fn a_save_from_a_built_in_layout_takes_the_device_name() {
    let mut model = laptop();
    let effects = run_command(&mut model, "/pane hide tabs");
    assert_eq!(
        effects,
        [Effect::SaveLayout {
            name: "laptop".to_owned(),
            lines: vec!["output".to_owned(), "editor below main".to_owned()],
            device: "laptop".to_owned(),
            then: Some(Choice::Default),
        }]
    );
    assert_eq!(model.layout_name(), "laptop");
}

#[test]
fn a_save_from_the_built_in_layout_of_a_world_sets_its_entry() {
    let mut model = laptop();
    let config = layout_config(CLASSIC, &[("alpha/aria", CLASSIC)], &["world"], &[]);
    let mut config = config;
    "classic".clone_into(&mut config.world_layouts[0].name);
    configure(&mut model, config);
    let effects = run_command(&mut model, "/pane hide tabs");
    assert_eq!(
        effects,
        [Effect::SaveLayout {
            name: "laptop.alpha.aria".to_owned(),
            lines: vec!["output".to_owned(), "editor below main".to_owned()],
            device: "laptop".to_owned(),
            then: Some(aria_entry()),
        }]
    );
    // The other world keeps the layout of the field `layout`.
    press(&mut model, KeyCode::Char('n'), Modifiers::CONTROL);
    assert_eq!(model.layout().lines().len(), 3);
}

#[test]
fn a_save_to_a_player_layout_keeps_its_name() {
    let mut model = laptop();
    let mut config = layout_config(&["output", "editor below main"], &[], &["world"], &[]);
    "mine".clone_into(&mut config.layout_name);
    configure(&mut model, config);
    let effects = run_command(&mut model, "/place tabs at top");
    assert_eq!(
        effects,
        [Effect::SaveLayout {
            name: "mine".to_owned(),
            lines: vec![
                "output".to_owned(),
                "editor below main".to_owned(),
                "tabs at top".to_owned()
            ],
            device: "laptop".to_owned(),
            then: None,
        }]
    );
}

#[test]
fn a_failed_save_puts_the_layout_back() {
    let mut model = laptop();
    let mut config = layout_config(CLASSIC, &[], &["world"], &[]);
    "classic".clone_into(&mut config.layout_name);
    configure(&mut model, config);
    run_command(&mut model, "/pane hide tabs");
    assert_eq!(model.layout_name(), "laptop");
    let effects = update(
        &mut model,
        Msg::LayoutSaved(Err("clients/devices/laptop.toml: bad".to_owned())),
    );
    // The screen shows what the server holds, and the client asks for it again.
    assert_eq!(effects, [Effect::FetchConfig]);
    assert_eq!(model.layout().lines(), CLASSIC);
    assert_eq!(model.layout_name(), "classic");
    assert_eq!(notices_of(&model, "clients/devices/laptop.toml"), 1);
}

#[test]
fn a_change_after_a_failed_save_sets_the_choice_again() {
    let mut model = laptop();
    let mut config = layout_config(CLASSIC, &[], &["world"], &[]);
    "classic".clone_into(&mut config.layout_name);
    configure(&mut model, config);
    run_command(&mut model, "/pane hide tabs");
    update(&mut model, Msg::LayoutSaved(Err("down".to_owned())));
    // The layout is the built-in one again, so the next save sets the choice of the device.
    let effects = run_command(&mut model, "/pane hide tabs");
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::SaveLayout { name, then: Some(Choice::Default), .. }] if name == "laptop"
        ),
        "{effects:?}"
    );
}

#[test]
fn a_failed_save_with_no_configuration_gives_the_default_layout() {
    let mut model = laptop();
    run_command(&mut model, "/pane hide tabs");
    let effects = update(&mut model, Msg::LayoutSaved(Err("down".to_owned())));
    assert_eq!(effects, [Effect::FetchConfig]);
    assert_eq!(model.layout_name(), "classic");
    assert_eq!(model.layout().lines(), CLASSIC);
}

#[test]
fn a_device_with_no_good_layout_name_does_not_save() {
    let mut model = model().with_device("focus");
    let effects = run_command(&mut model, "/pane hide tabs");
    assert_eq!(effects, []);
    assert_eq!(model.layout().lines(), ["output", "editor below main"]);
    assert_eq!(notices_of(&model, "/layout save"), 1);
}

#[test]
fn layout_list_asks_the_server() {
    let mut model = laptop();
    assert_eq!(
        run_command(&mut model, "/layout list"),
        [Effect::ListLayouts { next: false }]
    );
}

#[test]
fn a_list_for_show_makes_no_use_call() {
    let mut model = laptop();
    let effects = update(
        &mut model,
        Msg::Layouts {
            next: false,
            names: Ok(vec!["classic".to_owned(), "focus".to_owned()]),
        },
    );
    assert_eq!(effects, []);
}

#[test]
fn the_next_layout_wraps() {
    let mut model = laptop();
    let names = vec!["classic".to_owned(), "focus".to_owned()];
    let next = |model: &mut Model| {
        update(
            model,
            Msg::Layouts {
                next: true,
                names: Ok(names.clone()),
            },
        )
    };
    let use_of = |name: &str| Effect::UseLayout {
        device: "laptop".to_owned(),
        name: Some(name.to_owned()),
        choice: Choice::Default,
    };
    assert_eq!(next(&mut model), [use_of("focus")]);
    let mut config = layout_config(&["output", "editor below main"], &[], &["world"], &[]);
    "focus".clone_into(&mut config.layout_name);
    configure(&mut model, config);
    assert_eq!(next(&mut model), [use_of("classic")]);
}

#[test]
fn layout_here_sets_the_entry_of_the_world_in_focus() {
    let mut model = laptop();
    assert_eq!(
        run_command(&mut model, "/layout here focus"),
        [Effect::UseLayout {
            device: "laptop".to_owned(),
            name: Some("focus".to_owned()),
            choice: aria_entry(),
        }]
    );
    assert_eq!(
        run_command(&mut model, "/layout here"),
        [Effect::UseLayout {
            device: "laptop".to_owned(),
            name: None,
            choice: aria_entry(),
        }]
    );
}

#[test]
fn a_list_that_says_disconnected_clears_the_connect_time() {
    let mut model = status_model(&["state", "connected"], &[]);
    update(
        &mut model,
        world_event(
            EventKind::EVENT_KIND_CONNECTED,
            Source::SOURCE_SYSTEM,
            40_000,
        ),
    );
    update(&mut model, Msg::Tick(100_000));
    assert_eq!(status_row(&model), "connected | 1:00");
    // The list says that the world is not connected, and no disconnect event came.
    let world = |id: &str| mu_proto::World {
        id: id.to_owned(),
        state: mu_proto::WorldState::WORLD_STATE_DISCONNECTED.into(),
        ..Default::default()
    };
    update(
        &mut model,
        Msg::Listed(Ok(vec![world("alpha/aria"), world("beta/bryn")])),
    );
    assert_eq!(status_row(&model), "disconnected");
}

#[test]
fn a_failed_save_puts_back_the_last_good_layout_of_the_server() {
    let mut model = laptop();
    let mut good = layout_config(CLASSIC, &[], &["world"], &[]);
    "classic".clone_into(&mut good.layout_name);
    configure(&mut model, good);
    // The next reply holds a layout that this client cannot read, so the layout stays.
    let mut bad = layout_config(&["bogus"], &[], &["world"], &[]);
    "h2".clone_into(&mut bad.store_hash);
    store_hash(&mut model, "h2");
    update(&mut model, Msg::Config(Ok(Box::new(bad))));
    assert_eq!(model.layout().lines(), CLASSIC);
    run_command(&mut model, "/pane hide tabs");
    update(&mut model, Msg::LayoutSaved(Err("down".to_owned())));
    assert_eq!(model.layout().lines(), CLASSIC);
    assert_eq!(model.layout_name(), "classic");
    // The notice of the bad layout shows one time, not again at the restore.
    assert_eq!(notices_of(&model, "the bad layout"), 1);
}

// The routing of lines and notices by the streams of each event.

use mu_config::layout::{BlockId, Layout};
use mu_proto::World;

fn strings(lines: &[&str]) -> Vec<String> {
    lines.iter().map(ToString::to_string).collect()
}

/// A configuration with the layout `lines`, and the layout of each key of `worlds`.
fn layouts(lines: &[&str], worlds: &[(&str, &[&str])]) -> GetClientConfigResponse {
    GetClientConfigResponse {
        revision: format!("{lines:?}{worlds:?}"),
        echo: "show".to_owned(),
        layout_name: "test".to_owned(),
        layout: strings(lines),
        world_layouts: worlds
            .iter()
            .map(|(key, lines)| NamedLayout {
                key: (*key).to_owned(),
                name: format!("of-{}", key.replace('/', "-")),
                lines: strings(lines),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

/// Gives `model` the layout `lines`, and the layout of each key of `worlds`.
fn with_layouts(model: &mut Model, lines: &[&str], worlds: &[(&str, &[&str])]) {
    let config = layouts(lines, worlds);
    model.apply_config(config.clone());
    assert_eq!(model.config(), Some(&config));
}

/// A notice that a reset keeps: a failed list of the layouts with the reason `reason`.
fn kept_notice(model: &mut Model, reason: &str) {
    update(
        model,
        Msg::Layouts {
            next: false,
            names: Err(reason.to_owned()),
        },
    );
}

/// A line of the game of `world` with the streams `streams`.
fn line_of(world: &str, text: &str, streams: &[&str]) -> Msg {
    Msg::Client(ClientEvent::Event(Event {
        world: world.to_owned(),
        kind: EventKind::EVENT_KIND_LINE.into(),
        source: Source::SOURCE_GAME.into(),
        text: text.to_owned(),
        streams: strings(streams),
        ..Default::default()
    }))
}

fn send_line(model: &mut Model, world: &str, text: &str, streams: &[&str]) {
    update(model, line_of(world, text, streams));
}

fn block(id: &str) -> BlockId {
    BlockId::new(id).expect("an id")
}

/// The rows of the pane of `id` on an 80 by 24 screen, with no trailing spaces, no empty row
/// and no title row.
fn rows_of(model: &mut Model, id: &str) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| view(model, frame)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let solved = mu_tui::solved(model, 80, 24);
    let area = solved
        .pane_of(&block(id))
        .unwrap_or_else(|| panic!("no pane of {id}"))
        .area;
    let title = format!("[{id}]");
    (area.y..area.y + area.height)
        .map(|y| {
            let row: String = (area.x..area.x + area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect();
            row.trim_end().to_owned()
        })
        .filter(|row| !row.is_empty() && !row.starts_with(&title))
        .collect()
}

fn count(rows: &[String], check: impl Fn(&str) -> bool) -> usize {
    rows.iter().filter(|row| check(row)).count()
}

fn notices(rows: &[String]) -> usize {
    count(rows, |row| row.starts_with("-- "))
}

fn focus_next(model: &mut Model) {
    press(model, KeyCode::Char('n'), Modifiers::CONTROL);
}

fn run_line(model: &mut Model, text: &str) {
    type_text(model, text);
    press(model, KeyCode::Enter, Modifiers::NONE);
}

const CHAT_OF_ALL: &[&str] = &[
    "output",
    "editor below main",
    "output as chat of all worlds right of main",
];

const NOTICES_OF_ALL: &[&str] = &[
    "output as main showing main",
    "editor below main",
    "output as notices of all worlds right of main",
];

const NOTICES_BESIDE: &[&str] = &[
    "output as main showing main",
    "editor below main",
    "output as notices right of main",
];

#[test]
fn a_line_goes_to_each_matching_output() {
    let mut model = model();
    with_layouts(
        &mut model,
        &[
            "output",
            "editor below main",
            "output as comms showing chan-*, pages right of main",
            "output as log showing pages below comms",
        ],
        &[],
    );
    send_line(&mut model, "alpha/aria", "Bob pages: hi", &["pages"]);
    let shown = |model: &mut Model, id| count(&rows_of(model, id), |row| row == "Bob pages: hi");
    assert_eq!(
        (
            shown(&mut model, "comms"),
            shown(&mut model, "log"),
            shown(&mut model, "main")
        ),
        (1, 1, 0)
    );
}

#[test]
fn a_line_shows_once_in_one_output() {
    let mut model = model();
    with_layouts(
        &mut model,
        &[
            "output as main showing main, notices, pages",
            "editor below main",
        ],
        &[],
    );
    send_line(
        &mut model,
        "alpha/aria",
        "Bob pages: hi",
        &["pages", "main"],
    );
    let rows = rows_of(&mut model, "main");
    assert_eq!(count(&rows, |row| row == "Bob pages: hi"), 1, "{rows:?}");
}

#[test]
fn two_outputs_show_main() {
    let mut model = model();
    with_layouts(
        &mut model,
        &[
            "output",
            "editor below main",
            "output as log showing main right of main",
        ],
        &[],
    );
    send_line(&mut model, "alpha/aria", "A bell rings.", &["main"]);
    let shown = |model: &mut Model, id| count(&rows_of(model, id), |row| row == "A bell rings.");
    assert_eq!(
        (shown(&mut model, "main"), shown(&mut model, "log")),
        (1, 1)
    );
}

#[test]
fn a_stream_with_no_output_gives_one_notice() {
    let mut model = model();
    send_line(&mut model, "alpha/aria", "[Public] one", &["chan-public"]);
    send_line(&mut model, "alpha/aria", "[Public] two", &["chan-public"]);
    let rows = rows_of(&mut model, "main");
    let named = count(&rows, |row| {
        row.starts_with("-- ") && row.contains("chan-public") && row.contains("alpha/aria")
    });
    assert_eq!(
        (
            named,
            notices(&rows),
            count(&rows, |row| row.starts_with("[Public]"))
        ),
        (1, 1, 0),
        "{rows:?}"
    );
}

#[test]
fn a_line_in_two_streams_gives_a_notice_for_the_stream_with_no_output() {
    let mut model = model();
    send_line(
        &mut model,
        "alpha/aria",
        "Bob pages: hi",
        &["pages", "main"],
    );
    let rows = rows_of(&mut model, "main");
    assert_eq!(
        (
            count(&rows, |row| row == "Bob pages: hi"),
            count(&rows, |row| row.starts_with("-- ") && row.contains("pages"))
        ),
        (1, 1),
        "{rows:?}"
    );
}

#[test]
fn an_event_with_no_streams_goes_to_main() {
    let mut model = model();
    send_line(&mut model, "alpha/aria", "A bell rings.", &[]);
    let rows = rows_of(&mut model, "main");
    assert_eq!(count(&rows, |row| row == "A bell rings."), 1, "{rows:?}");
}

#[test]
fn a_refused_line_notice_belongs_to_its_world() {
    let mut model = model();
    with_layouts(&mut model, NOTICES_BESIDE, &[]);
    update(
        &mut model,
        Msg::Reply(mu_tui::Reply::Refused {
            world: "beta/bryn".to_owned(),
            text: "hi".to_owned(),
            reason: "down".to_owned(),
        }),
    );
    let before = notices(&rows_of(&mut model, "notices"));
    focus_next(&mut model);
    let after = rows_of(&mut model, "notices");
    assert_eq!(
        (before, after.last().map(String::as_str)),
        (0, Some("-- down")),
        "{after:?}"
    );
}

#[test]
fn a_notice_of_no_world_has_the_owner_client() {
    let mut model = Model::new([]);
    with_layouts(&mut model, NOTICES_OF_ALL, &[]);
    run_line(&mut model, "hi");
    let rows = rows_of(&mut model, "notices");
    assert_eq!(
        count(&rows, |row| row.starts_with("client: -- ")),
        1,
        "{rows:?}"
    );
}

#[test]
fn a_kept_notice_keeps_its_owner() {
    let mut model = model();
    kept_notice(&mut model, "nowhere/nobody");
    focus_next(&mut model);
    update(&mut model, reset());
    let of_beta = notices(&rows_of(&mut model, "main"));
    focus_next(&mut model);
    let of_alpha = rows_of(&mut model, "main");
    assert_eq!(
        (
            of_beta,
            count(&of_alpha, |row| row.contains("nowhere/nobody"))
        ),
        (0, 1),
        "{of_alpha:?}"
    );
}

#[test]
fn the_fallback_layout_is_valid() {
    assert!(Layout::parse(mu_tui::FALLBACK_LAYOUT).is_ok());
}

#[test]
fn a_prompt_stays_in_main() {
    let mut model = model();
    with_layouts(
        &mut model,
        &[
            "output",
            "editor below main",
            "output as log showing main right of main",
        ],
        &[],
    );
    update(
        &mut model,
        Msg::Client(ClientEvent::Event(Event {
            world: "alpha/aria".to_owned(),
            kind: EventKind::EVENT_KIND_PROMPT.into(),
            source: Source::SOURCE_GAME.into(),
            text: "HP 10>".to_owned(),
            streams: strings(&["main"]),
            ..Default::default()
        })),
    );
    let shown = |model: &mut Model, id| count(&rows_of(model, id), |row| row == "HP 10>");
    assert_eq!(
        (shown(&mut model, "main"), shown(&mut model, "log")),
        (1, 0)
    );
}

#[test]
fn a_pane_of_all_worlds_is_shared() {
    let mut model = model();
    with_layouts(&mut model, CHAT_OF_ALL, &[]);
    send_line(&mut model, "alpha/aria", "one", &["chat"]);
    send_line(&mut model, "beta/bryn", "two", &["chat"]);
    assert_eq!(
        rows_of(&mut model, "chat"),
        ["alpha/aria: one", "beta/bryn: two"]
    );
}

#[test]
fn a_pane_of_all_worlds_stays_at_a_focus_change() {
    let mut model = model();
    with_layouts(&mut model, CHAT_OF_ALL, &[]);
    send_line(&mut model, "alpha/aria", "one", &["chat"]);
    focus_next(&mut model);
    assert_eq!(rows_of(&mut model, "chat"), ["alpha/aria: one"]);
}

#[test]
fn a_notice_in_a_pane_of_all_worlds_has_its_world() {
    let mut model = model();
    with_layouts(&mut model, NOTICES_OF_ALL, &[]);
    run_line(&mut model, "/focus nowhere/nobody");
    let rows = rows_of(&mut model, "notices");
    assert_eq!(
        count(&rows, |row| row.starts_with("alpha/aria: -- ")),
        1,
        "{rows:?}"
    );
}

#[test]
fn a_focus_change_replays_no_notice() {
    let mut model = model();
    kept_notice(&mut model, "down");
    focus_next(&mut model);
    let of_beta = notices(&rows_of(&mut model, "main"));
    focus_next(&mut model);
    let of_alpha = notices(&rows_of(&mut model, "main"));
    assert_eq!((of_beta, of_alpha), (0, 1));
}

/// A model before its first list of the worlds, with the width notice of no world.
fn starting_with_a_notice() -> Model {
    let mut model = Model::starting();
    probe(&mut model, &incomplete());
    model
}

fn list(model: &mut Model) {
    let worlds = ["alpha/aria", "beta/bryn"].map(|id| World {
        id: id.to_owned(),
        ..Default::default()
    });
    update(model, Msg::Listed(Ok(worlds.to_vec())));
    assert_eq!(model.focused(), Some("alpha/aria"));
}

#[test]
fn the_first_focus_carries_the_notices_of_no_world() {
    let mut model = starting_with_a_notice();
    list(&mut model);
    focus_next(&mut model);
    let of_beta = notices(&rows_of(&mut model, "main"));
    focus_next(&mut model);
    let of_alpha = rows_of(&mut model, "main");
    assert_eq!(
        (of_beta, count(&of_alpha, |row| row == WIDTH_NOTICE)),
        (0, 1),
        "{of_alpha:?}"
    );
}

#[test]
fn a_replay_skips_a_pane_that_holds_the_notice() {
    let mut model = Model::starting();
    with_layouts(&mut model, NOTICES_OF_ALL, &[]);
    probe(&mut model, &incomplete());
    list(&mut model);
    let rows = rows_of(&mut model, "notices");
    assert_eq!(
        count(&rows, |row| row.contains("-- the terminal gave no")),
        1,
        "{rows:?}"
    );
}

#[test]
fn the_first_focus_fills_a_shared_pane_that_lacks_the_notice() {
    let mut model = Model::starting();
    with_layouts(
        &mut model,
        &["output", "editor below main"],
        &[("alpha/aria", NOTICES_OF_ALL)],
    );
    probe(&mut model, &incomplete());
    list(&mut model);
    let rows = rows_of(&mut model, "notices");
    assert_eq!(
        (
            count(&rows, |row| row
                .starts_with("alpha/aria: -- the terminal gave no")),
            count(&rows, |row| row.contains(": -- "))
        ),
        (1, 1),
        "{rows:?}"
    );
}

#[test]
fn a_reset_clears_the_panes_of_all_worlds() {
    let mut model = model();
    with_layouts(&mut model, CHAT_OF_ALL, &[]);
    send_line(&mut model, "alpha/aria", "one", &["chat"]);
    update(&mut model, reset());
    assert_eq!(rows_of(&mut model, "chat"), [] as [String; 0]);
}

#[test]
fn a_reset_keeps_the_pane_of_no_world() {
    let mut model = Model::new([]);
    run_line(&mut model, "hi");
    let before = rows_of(&mut model, "main");
    update(&mut model, reset());
    assert_eq!((rows_of(&mut model, "main"), before.len()), (before, 1));
}

#[test]
fn a_reset_shows_a_notice_of_no_world_once() {
    let mut model = Model::new([]);
    probe(&mut model, &incomplete());
    update(&mut model, reset());
    update(&mut model, reset());
    let rows = rows_of(&mut model, "main");
    assert_eq!(count(&rows, |row| row == WIDTH_NOTICE), 1, "{rows:?}");
}

#[test]
fn a_reset_replays_a_notice_by_the_layout_of_its_owner() {
    let mut model = model();
    with_layouts(
        &mut model,
        &["output", "editor below main"],
        &[("alpha/aria", NOTICES_BESIDE)],
    );
    kept_notice(&mut model, "nowhere/nobody");
    update(&mut model, reset());
    assert_eq!(
        (
            notices(&rows_of(&mut model, "notices")),
            notices(&rows_of(&mut model, "main"))
        ),
        (1, 0)
    );
}

#[test]
fn a_reset_puts_each_kept_notice_back_once() {
    let mut model = model();
    kept_notice(&mut model, "nowhere/nobody");
    kept_notice(&mut model, "nowhere/else");
    update(&mut model, reset());
    update(&mut model, reset());
    let rows = rows_of(&mut model, "main");
    assert_eq!(
        (
            count(&rows, |row| row.contains("nowhere/nobody")),
            count(&rows, |row| row.contains("nowhere/else"))
        ),
        (1, 1),
        "{rows:?}"
    );
}

#[test]
fn the_focused_pane_is_main_of_the_world_in_focus() {
    let mut model = model();
    let of_alpha = model.focused_pane();
    focus_next(&mut model);
    assert_eq!(
        (
            Some(of_alpha) != model.pane_of_block(&block("main")),
            Some(model.focused_pane()) == model.pane_of_block(&block("main"))
        ),
        (true, true)
    );
}

#[test]
fn a_world_with_another_layout_keeps_its_line() {
    let mut model = model();
    with_layouts(
        &mut model,
        &["output", "editor below main"],
        &[("alpha/aria", CHAT_OF_ALL)],
    );
    focus_next(&mut model);
    send_line(&mut model, "alpha/aria", "one", &["chat"]);
    focus_next(&mut model);
    assert_eq!(rows_of(&mut model, "chat"), ["alpha/aria: one"]);
}

#[test]
fn a_stream_with_no_output_gives_no_notice_after_a_reset() {
    let mut model = model();
    send_line(&mut model, "alpha/aria", "[Public] one", &["chan-public"]);
    update(&mut model, reset());
    let rows = rows_of(&mut model, "main");
    assert_eq!(
        count(&rows, |row| row.contains("chan-public")),
        0,
        "{rows:?}"
    );
}

/// The rows of the pane of `main`, with no whitespace, in one string.
fn packed_main(model: &mut Model) -> String {
    rows_of(model, "main").concat().split_whitespace().collect()
}

/// Whether the last hint in the pane of `main` names the stream `stream`.
fn last_hint_names(model: &mut Model, stream: &str) -> bool {
    let packed = packed_main(model);
    let start = packed.rfind("--nooutput").expect("a hint");
    packed[start..].contains(&format!("thestream{stream},"))
}

#[test]
fn the_oldest_stream_with_no_output_gives_its_hint_again() {
    let mut model = model();
    for number in 0..=mu_tui::NO_OUTPUT_MAX {
        send_line(
            &mut model,
            "alpha/aria",
            "a line",
            &[&format!("chan-{number}")],
        );
    }
    // The pair of chan-1 is still in the memory, and the pair of chan-0 is not.
    send_line(&mut model, "alpha/aria", "a line", &["chan-1"]);
    let kept = last_hint_names(&mut model, &format!("chan-{}", mu_tui::NO_OUTPUT_MAX));
    send_line(&mut model, "alpha/aria", "a line", &["chan-0"]);
    assert_eq!((kept, last_hint_names(&mut model, "chan-0")), (true, true));
}
