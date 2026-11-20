//! `termina-check [--out FILE]` runs the three checks on the Termina event stream.

use std::io::{self, Write};

use futures::StreamExt;
use termina::event::{KeyCode, KeyEventKind, Modifiers};
use termina::{Event, EventStream, PlatformTerminal, Terminal};
use terminal_backend::{MODES_OFF, MODES_ON, Seen, finish, run};

fn seen(event: Event) -> Seen {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => Seen::Key {
            code: match key.code {
                KeyCode::Char(c) => c.to_string(),
                other => format!("{other:?}"),
            },
            shift: key.modifiers.contains(Modifiers::SHIFT),
            alt: key.modifiers.contains(Modifiers::ALT),
            ctrl: key.modifiers.contains(Modifiers::CONTROL),
        },
        Event::Paste(text) => Seen::Paste(text),
        Event::Csi(csi) => Seen::Reply(format!("{csi:?}")),
        Event::Dcs(dcs) => Seen::Reply(format!("{dcs:?}")),
        Event::Osc(osc) => Seen::Reply(format!("{osc:?}")),
        other => Seen::Other(format!("{other:?}")),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let mut terminal = PlatformTerminal::new()?;
    terminal.enter_raw_mode()?;
    terminal.write_all(MODES_ON)?;
    terminal.flush()?;
    let events = EventStream::new(terminal.event_reader(), |_| true).map(|e| e.map(seen));
    let result = run("termina 0.3.3", events).await;
    terminal.write_all(MODES_OFF)?;
    terminal.flush()?;
    terminal.enter_cooked_mode()?;
    finish(&result?)
}
