use std::fs::File;
use std::io::{self, Write};
use std::time::Instant;

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::supports_keyboard_enhancement;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui_textarea::Input;

use crate::editor::Editor;
use crate::keys::{Action, route};

const GUIDE: &[&str] = &[
    "Steps (the log records each event):",
    "1. Type hello, press Shift+Enter, type world, press Enter.",
    "2. Type a, press Option+Enter, type b, press Enter.",
    "3. Type c, press Ctrl+J, type d, press Enter.",
    "4. Press Cmd+V to paste the sample pose, then press Enter.",
    "5. Press F2, type x, press Enter, press F2.",
    "6. Switch to an input method (for example Japanese Romaji), type nihongo,",
    "   press Space, then Enter to commit. Watch where the candidate box opens.",
    "   Switch back to the U.S. layout and press Enter to send.",
    "7. Press Ctrl+C.",
    "",
];

struct Log {
    start: Instant,
    out: Option<File>,
}

impl Log {
    fn write(&mut self, kind: &str, detail: &str) {
        if let Some(f) = &mut self.out {
            let ms = self.start.elapsed().as_millis();
            let detail = detail
                .replace('\t', "\\t")
                .replace('\n', "\\n")
                .replace('\r', "\\r");
            let _ = writeln!(f, "{ms}\t{kind}\t{detail}");
        }
    }
}

/// The interactive test: a pose editor, a second one-line input and an
/// output list. Enter sends, Shift+Enter, Alt+Enter or Ctrl+J break the
/// line, F2 moves the focus and Ctrl+C quits.
pub fn run(log_path: Option<String>) -> io::Result<()> {
    let mut log = Log {
        start: Instant::now(),
        out: log_path.map(File::create).transpose()?,
    };
    let mut term = ratatui::init();
    let kitty = supports_keyboard_enhancement().unwrap_or(false);
    execute!(io::stdout(), EnableBracketedPaste)?;
    if kitty {
        execute!(
            io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
    }
    log.write(
        "start",
        &format!(
            "term={} term_program={} kitty_keyboard={kitty}",
            std::env::var("TERM").unwrap_or_default(),
            std::env::var("TERM_PROGRAM").unwrap_or_default()
        ),
    );
    let result = event_loop(&mut term, &mut log, kitty);
    if kitty {
        execute!(io::stdout(), PopKeyboardEnhancementFlags)?;
    }
    execute!(io::stdout(), DisableBracketedPaste)?;
    ratatui::restore();
    result
}

fn event_loop(term: &mut DefaultTerminal, log: &mut Log, kitty: bool) -> io::Result<()> {
    let mut editors = [Editor::new(), Editor::new()];
    editors[0].area.set_block(Block::bordered().title(" pose "));
    editors[1]
        .area
        .set_block(Block::bordered().title(" second input "));
    let mut focus = 0usize;
    let mut sent: Vec<String> = Vec::new();
    let mut last = String::from("none");

    loop {
        term.draw(|f| {
            let pose_rows = editors[0].area.lines().len().clamp(1, 6) as u16 + 2;
            let [guide, out, status, a, b] = Layout::vertical([
                Constraint::Length(GUIDE.len() as u16),
                Constraint::Min(3),
                Constraint::Length(1),
                Constraint::Length(pose_rows),
                Constraint::Length(3),
            ])
            .areas(f.area());
            f.render_widget(Paragraph::new(GUIDE.iter().map(|g| Line::raw(*g)).collect::<Vec<_>>()), guide);
            let lines: Vec<Line> = sent.iter().flat_map(|s| s.lines()).map(Line::raw).collect();
            let skip = lines.len().saturating_sub(out.height as usize);
            f.render_widget(
                Paragraph::new(lines[skip..].to_vec()).wrap(Wrap { trim: false }),
                out,
            );
            f.render_widget(
                Line::raw(format!(
                    "focus={} kitty={kitty} last={last}  (Enter send, Shift/Alt+Enter or Ctrl+J newline, F2 focus, Ctrl+C quit)",
                    ["pose", "second"][focus]
                )),
                status,
            );
            let (first, second) = editors.split_at_mut(1);
            first[0].draw(f, a, focus == 0);
            second[0].draw(f, b, focus == 1);
        })?;

        match event::read()? {
            Event::Key(key) => {
                let action = route(&key);
                last = format!("{:?}+{:?}", key.code, key.modifiers);
                log.write("key", &format!("{key:?} -> {action:?}"));
                match action {
                    Action::Quit => return Ok(()),
                    Action::SwitchFocus => focus = 1 - focus,
                    Action::Newline => editors[focus].area.insert_newline(),
                    Action::Submit => {
                        let text = editors[focus].take();
                        log.write(
                            "submit",
                            &format!(
                                "input={focus} bytes={} lines={} text={text}",
                                text.len(),
                                text.lines().count()
                            ),
                        );
                        if !text.is_empty() {
                            let first = text.lines().next().unwrap_or_default();
                            sent.push(format!(
                                "[{}] {} lines, {} bytes: {first}",
                                ["pose", "second"][focus],
                                text.lines().count(),
                                text.len()
                            ));
                        }
                    }
                    Action::Edit => {
                        editors[focus].area.input(Input::from(key));
                    }
                    Action::Ignore => {}
                }
            }
            Event::Paste(text) => {
                last = format!("Paste({} bytes)", text.len());
                log.write(
                    "paste",
                    &format!(
                        "bytes={} lf={} cr={}",
                        text.len(),
                        text.matches('\n').count(),
                        text.matches('\r').count()
                    ),
                );
                editors[focus].paste(&text);
            }
            Event::Resize(w, h) => log.write("resize", &format!("{w}x{h}")),
            _ => {}
        }
    }
}
