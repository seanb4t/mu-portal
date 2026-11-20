//! The loop of the terminal client: keys, the event stream and the replies of calls.

use std::io::{self, Write};

use futures::StreamExt as _;
use mu_client::PortalClient;
use ratatui::Terminal;
use ratatui::backend::TerminaBackend;
use termina::escape::csi::{Csi, DecPrivateMode, DecPrivateModeCode, Mode};
use termina::{Event as TermEvent, EventReader, EventStream, PlatformTerminal, Terminal as _};
use tokio::sync::mpsc;

use crate::app::{Action, App, Reply};

type Screen = Terminal<TerminaBackend<PlatformTerminal>>;

/// Runs the client on the alternate screen until Ctrl+C. The terminal comes back on exit,
/// and on a panic through the panic hook of Termina.
///
/// # Errors
///
/// An I/O error of the terminal.
pub async fn run(client: PortalClient) -> io::Result<()> {
    // A failed call leaves the list empty, and the events of each world add it.
    let worlds = client.list_worlds().await.map_or_else(
        |_| Vec::new(),
        |worlds| worlds.into_iter().map(|world| world.id).collect(),
    );
    let mut app = App::new(worlds);
    let (mut terminal, reader) = open()?;
    let result = event_loop(&mut terminal, reader, &mut app, &client).await;
    // Dropping the backend returns the terminal to cooked mode.
    let left = leave(terminal.backend_mut());
    result.and(left)
}

fn private_mode(code: DecPrivateModeCode, on: bool) -> Csi {
    let mode = DecPrivateMode::Code(code);
    Csi::Mode(if on {
        Mode::SetDecPrivateMode(mode)
    } else {
        Mode::ResetDecPrivateMode(mode)
    })
}

fn leave(out: &mut impl Write) -> io::Result<()> {
    write!(
        out,
        "{}{}",
        private_mode(DecPrivateModeCode::ClearAndEnableAlternateScreen, false),
        private_mode(DecPrivateModeCode::ShowCursor, true),
    )?;
    out.flush()
}

/// Enters raw mode and the alternate screen, and sets a panic hook that leaves both.
fn open() -> io::Result<(Screen, EventReader)> {
    let mut platform = PlatformTerminal::new()?;
    platform.enter_raw_mode()?;
    platform.set_panic_hook(|out| {
        // The panic hook has no caller to report a failed write to.
        let _ = leave(out);
    });
    write!(
        platform,
        "{}",
        private_mode(DecPrivateModeCode::ClearAndEnableAlternateScreen, true)
    )?;
    platform.flush()?;
    let reader = platform.event_reader();
    Ok((Terminal::new(TerminaBackend::new(platform))?, reader))
}

async fn event_loop(
    terminal: &mut Screen,
    reader: EventReader,
    app: &mut App,
    client: &PortalClient,
) -> io::Result<()> {
    let mut keys = EventStream::new(reader, |_| true);
    let mut events = Box::pin(client.events());
    let (replies, mut reply) = mpsc::unbounded_channel();
    loop {
        terminal.draw(|frame| app.draw(frame))?;
        tokio::select! {
            key = keys.next() => match key {
                Some(Ok(TermEvent::Key(key))) => match app.on_key(key) {
                    Some(Action::Quit) => return Ok(()),
                    Some(Action::Send { world, text }) => {
                        let client = client.clone();
                        let replies = replies.clone();
                        tokio::spawn(async move {
                            let answer = match client.send(&world, &text).await {
                                Ok(_) => Reply::Sent,
                                Err(error) => Reply::Refused { world, text, reason: error.message },
                            };
                            // The loop holds the receiver until it returns.
                            let _ = replies.send(answer);
                        });
                    }
                    None => {}
                },
                Some(Ok(_)) => {}
                Some(Err(error)) => return Err(error),
                None => return Ok(()),
            },
            Some(item) = events.next() => app.on_event(item),
            Some(answer) = reply.recv() => app.on_reply(answer),
        }
    }
}
