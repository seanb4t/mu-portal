//! The loop of the terminal client: keys, the event stream and the replies of calls.

use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use futures::StreamExt as _;
use mu_client::PortalClient;
use termina::escape::osc::{Osc, Selection};
use termina::{EventReader, EventStream};
use tokio::sync::mpsc;

use crate::kitty;
use crate::model::Model;
use crate::probe;
use crate::screen::{self, Screen};
use crate::update::{Effect, Msg, Reply, update};
use crate::view::view;

/// Runs the client on the alternate screen until the Quit action or the end of terminal input.
/// The terminal comes back on exit,
/// and on a panic through the panic hook of Termina.
///
/// # Errors
///
/// An I/O error of the terminal.
pub async fn run(client: PortalClient) -> io::Result<()> {
    let mut model = Model::from_listing(client.list_worlds().await);
    // The probe sets mode 2027, so a panic before its replies turns the mode off.
    let grapheme_mode = Arc::new(AtomicBool::new(true));
    let (mut terminal, reader) = screen::open(&grapheme_mode)?;
    let probed = probe::run(terminal.backend_mut(), probe::WAIT);
    let result = match probed {
        Ok((mut outcome, input)) => {
            grapheme_mode.store(outcome.resets_grapheme_mode(), Ordering::Relaxed);
            let open = std::mem::take(&mut outcome.open);
            // The probe result goes first, then the input that came during the probe, then the
            // bytes of a cut answer. The input came before those bytes.
            let early = std::iter::once(Msg::Probe(outcome))
                .chain(input.into_iter().map(Msg::Terminal))
                .chain((!open.is_empty()).then_some(Msg::CutReply(open)))
                .collect();
            let result = event_loop(&mut terminal, reader, early, &mut model, &client).await;
            if model.keeps_grapheme_mode() {
                grapheme_mode.store(false, Ordering::Relaxed);
            }
            result
        }
        Err(error) => Err(error),
    };
    // Dropping the backend returns the terminal to cooked mode.
    let left = screen::leave(
        terminal.backend_mut(),
        grapheme_mode.load(Ordering::Relaxed),
    );
    screen::finish(result, left)
}

async fn event_loop(
    terminal: &mut Screen,
    reader: EventReader,
    mut early: VecDeque<Msg>,
    model: &mut Model,
    client: &PortalClient,
) -> io::Result<()> {
    let mut terminal_events = EventStream::new(reader, |_| true);
    let mut events = Box::pin(client.events());
    let (replies, mut reply) = mpsc::unbounded_channel();
    loop {
        // No frame is drawn while an early message waits.
        let msg = if let Some(msg) = early.pop_front() {
            msg
        } else {
            terminal.draw(|frame| view(model, frame))?;
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
