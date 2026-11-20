//! The loop of the terminal client: keys, the event stream and the replies of calls.

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use futures::StreamExt as _;
use mu_client::PortalClient;
use ratatui::Terminal;
use ratatui::backend::TerminaBackend;
use termina::escape::csi::{Csi, DecPrivateMode, DecPrivateModeCode, Mode};
use termina::escape::osc::{Osc, Selection};
use termina::{Event as TermEvent, EventReader, EventStream, PlatformTerminal, Terminal as _};
use tokio::sync::mpsc;

use crate::kitty;
use crate::model::Model;
use crate::pane::Caps;
use crate::probe;
use crate::update::{Effect, Msg, Reply, update};
use crate::view::view;

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
    let mut model = Model::new(worlds);
    // The probe sets mode 2027, so a panic before its replies turns the mode off.
    let grapheme_mode = Arc::new(AtomicBool::new(true));
    let (mut terminal, reader) = open(&grapheme_mode)?;
    let probed = probe::run(terminal.backend_mut(), probe::WAIT);
    let result = match probed {
        Ok((outcome, input)) => {
            grapheme_mode.store(outcome.resets_grapheme_mode(), Ordering::Relaxed);
            model.set_caps(Caps {
                profile: outcome.profile,
                graphics: outcome.graphics,
            });
            let mut parser = termina::Parser::default();
            parser.parse(&input, false);
            let early = std::iter::from_fn(|| parser.pop()).collect();
            event_loop(&mut terminal, reader, early, &mut model, &client).await
        }
        Err(error) => Err(error),
    };
    // Dropping the backend returns the terminal to cooked mode.
    let left = leave(
        terminal.backend_mut(),
        grapheme_mode.load(Ordering::Relaxed),
    );
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

/// The modes of mouse capture: press and release, drag, and the SGR format of the reports.
const MOUSE: [DecPrivateModeCode; 3] = [
    DecPrivateModeCode::MouseTracking,
    DecPrivateModeCode::ButtonEventMouse,
    DecPrivateModeCode::SGRMouse,
];

/// Turns off mouse capture, and mode 2027 if `grapheme_mode`, then leaves the alternate screen
/// and shows the cursor.
fn leave(out: &mut impl Write, grapheme_mode: bool) -> io::Result<()> {
    for mode in MOUSE.into_iter().rev() {
        write!(out, "{}", private_mode(mode, false))?;
    }
    if grapheme_mode {
        write!(
            out,
            "{}",
            private_mode(DecPrivateModeCode::GraphemeClustering, false)
        )?;
    }
    write!(
        out,
        "{}{}",
        private_mode(DecPrivateModeCode::ClearAndEnableAlternateScreen, false),
        private_mode(DecPrivateModeCode::ShowCursor, true),
    )?;
    out.flush()
}

/// Enters raw mode and the alternate screen and turns on mouse capture. Sets a panic hook that
/// undoes each, and turns off mode 2027 if `grapheme_mode` holds true.
fn open(grapheme_mode: &Arc<AtomicBool>) -> io::Result<(Screen, EventReader)> {
    let mut platform = PlatformTerminal::new()?;
    platform.enter_raw_mode()?;
    let hook_mode = Arc::clone(grapheme_mode);
    platform.set_panic_hook(move |out| {
        // The panic hook has no caller to report a failed write to.
        let _ = leave(out, hook_mode.load(Ordering::Relaxed));
    });
    write!(
        platform,
        "{}",
        private_mode(DecPrivateModeCode::ClearAndEnableAlternateScreen, true)
    )?;
    for mode in MOUSE {
        write!(platform, "{}", private_mode(mode, true))?;
    }
    platform.flush()?;
    let reader = platform.event_reader();
    Ok((Terminal::new(TerminaBackend::new(platform))?, reader))
}

async fn event_loop(
    terminal: &mut Screen,
    reader: EventReader,
    early: Vec<TermEvent>,
    model: &mut Model,
    client: &PortalClient,
) -> io::Result<()> {
    let mut terminal_events = EventStream::new(reader, |_| true);
    let mut events = Box::pin(client.events());
    let (replies, mut reply) = mpsc::unbounded_channel();
    // The input that came during the probe goes first.
    let mut early = early.into_iter();
    loop {
        terminal.draw(|frame| view(model, frame))?;
        let msg = if let Some(event) = early.next() {
            Msg::Terminal(event)
        } else {
            tokio::select! {
            event = terminal_events.next() => match event {
                Some(Ok(event)) => Msg::Terminal(event),
                Some(Err(error)) => return Err(error),
                None => return Ok(()),
            },
            Some(item) = events.next() => Msg::Client(item),
            Some(answer) = reply.recv() => Msg::Reply(answer),
            }
        };
        for effect in update(model, msg) {
            match effect {
                Effect::Quit => return Ok(()),
                Effect::TransmitImage { image, png } => {
                    let out = terminal.backend_mut();
                    out.write_all(&kitty::transmit(image, &png))?;
                    out.flush()?;
                }
                Effect::ClipboardWrite(text) => {
                    let out = terminal.backend_mut();
                    write!(out, "{}", Osc::SetSelection(Selection::CLIPBOARD, &text))?;
                    out.flush()?;
                }
                Effect::Send { world, text } => {
                    let client = client.clone();
                    let replies = replies.clone();
                    tokio::spawn(async move {
                        let answer = match client.send(&world, &text).await {
                            Ok(_) => Reply::Sent,
                            Err(error) => Reply::Refused {
                                world,
                                text,
                                reason: error.message,
                            },
                        };
                        // The loop holds the receiver until it returns.
                        let _ = replies.send(answer);
                    });
                }
            }
        }
    }
}
