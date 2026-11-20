//! `crossterm-check [--out FILE]` runs the three checks on the crossterm event stream.

use std::io::{self, Write};

use crossterm::event::{Event, EventStream, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use futures::StreamExt;
use terminal_backend::{MODES_OFF, MODES_ON, Seen, finish, run};

fn seen(event: Event) -> Seen {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => Seen::Key {
            code: match key.code {
                crossterm::event::KeyCode::Char(c) => c.to_string(),
                other => format!("{other:?}"),
            },
            shift: key.modifiers.contains(KeyModifiers::SHIFT),
            alt: key.modifiers.contains(KeyModifiers::ALT),
            ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
        },
        Event::Paste(text) => Seen::Paste(text),
        other => Seen::Other(format!("{other:?}")),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().write_all(MODES_ON)?;
    io::stdout().flush()?;
    let events = EventStream::new().map(|e| e.map(seen));
    let result = run("crossterm 0.29.0", events).await;
    io::stdout().write_all(MODES_OFF)?;
    disable_raw_mode()?;
    finish(&result?)
}
