//! The state of the terminal client, with no I/O: events, replies and keys go in, actions
//! and a frame come out.

use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mu_client::ClientEvent;
use mu_proto::{Event, Source};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui_textarea::{CursorMove, TextArea};

/// The lines that the client keeps for each world.
const KEPT_LINES: usize = 2000;

/// What the loop must do after a key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Send `text` to `world`.
    Send {
        /// The world in focus.
        world: String,
        /// The text of the input line.
        text: String,
    },
    /// Leave the client.
    Quit,
}

/// The result of an [`Action::Send`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    /// The server took the line.
    Sent,
    /// The server refused the line.
    Refused {
        /// The world of the line.
        world: String,
        /// The text of the line, which goes back to the input line.
        text: String,
        /// Why the server refused it.
        reason: String,
    },
}

struct World {
    id: String,
    lines: VecDeque<String>,
    unseen: bool,
}

impl World {
    fn new(id: String) -> Self {
        Self {
            id,
            lines: VecDeque::new(),
            unseen: false,
        }
    }

    fn push(&mut self, line: String) {
        if self.lines.len() == KEPT_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }
}

/// The order of the worlds: by the game part, then by the character part.
fn order(id: &str) -> (&str, &str) {
    id.split_once('/').unwrap_or((id, ""))
}

/// The terminal client.
pub struct App {
    worlds: Vec<World>,
    /// An index into `worlds`. None if and only if there is no world.
    focus: Option<usize>,
    /// Lines for no world, shown when no world has the focus.
    notices: Vec<String>,
    input: TextArea<'static>,
    lost: bool,
}

impl App {
    /// A client with these worlds, in their order. The first world has the focus.
    #[must_use]
    pub fn new(worlds: impl IntoIterator<Item = String>) -> Self {
        let mut app = Self {
            worlds: Vec::new(),
            focus: None,
            notices: Vec::new(),
            input: input_line(String::new()),
            lost: false,
        };
        for id in worlds {
            app.world(&id);
        }
        app.focus = (!app.worlds.is_empty()).then_some(0);
        app
    }

    /// The id of the world in focus.
    #[must_use]
    pub fn focused(&self) -> Option<&str> {
        self.focus
            .and_then(|index| self.worlds.get(index))
            .map(|world| world.id.as_str())
    }

    /// The ids of the worlds, in their order.
    #[must_use]
    pub fn world_ids(&self) -> Vec<&str> {
        self.worlds.iter().map(|world| world.id.as_str()).collect()
    }

    /// The text of the input line.
    #[must_use]
    pub fn input(&self) -> &str {
        self.input.lines().first().map_or("", String::as_str)
    }

    /// The index of `id`, which this adds in its order if it is new.
    fn world(&mut self, id: &str) -> usize {
        match self
            .worlds
            .binary_search_by(|world| order(&world.id).cmp(&order(id)))
        {
            Ok(index) => index,
            Err(index) => {
                self.worlds.insert(index, World::new(id.to_owned()));
                match self.focus {
                    Some(focus) if focus >= index => self.focus = Some(focus + 1),
                    None => self.focus = Some(index),
                    Some(_) => {}
                }
                index
            }
        }
    }

    /// Takes one item of the event stream.
    pub fn on_event(&mut self, item: ClientEvent) {
        match item {
            ClientEvent::Reset { .. } => {
                self.lost = false;
                for world in &mut self.worlds {
                    world.lines.clear();
                    world.unseen = false;
                }
            }
            ClientEvent::Resumed => self.lost = false,
            ClientEvent::Lost => self.lost = true,
            ClientEvent::Event(event) => {
                self.lost = false;
                let index = self.world(&event.world);
                let unseen = self.focus != Some(index);
                let world = &mut self.worlds[index];
                world.push(line(&event));
                world.unseen |= unseen;
            }
        }
    }

    /// Takes the reply of a call.
    pub fn on_reply(&mut self, reply: Reply) {
        let Reply::Refused {
            world,
            text,
            reason,
        } = reply
        else {
            return;
        };
        let notice = format!("-- {reason}");
        match self.worlds.iter_mut().find(|each| each.id == world) {
            Some(world) => world.push(notice),
            None => self.notices.push(notice),
        }
        self.input = input_line(text);
    }

    /// Takes one key. Returns what the loop must do, if anything.
    pub fn on_key(&mut self, key: KeyEvent) -> Option<Action> {
        if key.kind == KeyEventKind::Release {
            return None;
        }
        let control = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('c') if control => return Some(Action::Quit),
            KeyCode::Char('n') if control => self.move_focus(true),
            KeyCode::Char('p') if control => self.move_focus(false),
            KeyCode::Enter => return self.enter(),
            _ => {
                self.input.input(key);
            }
        }
        None
    }

    fn enter(&mut self) -> Option<Action> {
        let Some(world) = self.focused().map(str::to_owned) else {
            self.notices
                .push("-- no world has the focus, so the line goes nowhere".to_owned());
            return None;
        };
        let text = self.input().to_owned();
        self.input = input_line(String::new());
        Some(Action::Send { world, text })
    }

    fn move_focus(&mut self, forward: bool) {
        let count = self.worlds.len();
        let Some(focus) = self.focus.filter(|focus| *focus < count) else {
            return;
        };
        let next = if forward {
            if focus + 1 == count { 0 } else { focus + 1 }
        } else if focus == 0 {
            count - 1
        } else {
            focus - 1
        };
        self.focus = Some(next);
        self.worlds[next].unseen = false;
    }

    fn top_row(&self) -> String {
        let mut row = if self.worlds.is_empty() {
            "no worlds".to_owned()
        } else {
            self.worlds
                .iter()
                .enumerate()
                .map(|(index, world)| {
                    let mark = if world.unseen { "*" } else { "" };
                    if self.focus == Some(index) {
                        format!("[{}]{mark}", world.id)
                    } else {
                        format!("{}{mark}", world.id)
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        };
        if self.lost {
            row.push_str("  disconnected");
        }
        row
    }

    /// Draws the top row, the lines of the world in focus and the input line.
    pub fn draw(&self, frame: &mut Frame) {
        let [top, body, bottom] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(frame.area());
        frame.render_widget(Paragraph::new(self.top_row()), top);

        let lines: Vec<&str> = match self.focus.and_then(|index| self.worlds.get(index)) {
            Some(world) => world.lines.iter().map(String::as_str).collect(),
            None => self.notices.iter().map(String::as_str).collect(),
        };
        let height = usize::from(body.height);
        let shown: Vec<Line> = lines[lines.len().saturating_sub(height)..]
            .iter()
            .map(|line| Line::raw(*line))
            .collect();
        frame.render_widget(Paragraph::new(shown), body);
        frame.render_widget(&self.input, bottom);
    }
}

fn input_line(text: String) -> TextArea<'static> {
    let mut input = TextArea::new(vec![text]);
    input.set_cursor_line_style(Style::default());
    input.move_cursor(CursorMove::End);
    input
}

/// The text of an event on the screen: a game line as it is, a player line after `> `, and
/// a line of the system or a credential after `-- `.
fn line(event: &Event) -> String {
    match event.source.as_known() {
        Some(Source::SOURCE_PLAYER) => format!("> {}", event.text),
        Some(Source::SOURCE_SYSTEM | Source::SOURCE_CREDENTIAL) => format!("-- {}", event.text),
        _ => event.text.clone(),
    }
}
