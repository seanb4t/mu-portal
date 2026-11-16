//! The loop of the terminal client: keys, the event stream and the replies of calls.

use std::io;

use crossterm::event::{Event as TermEvent, EventStream};
use futures::StreamExt as _;
use mu_client::PortalClient;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;

use crate::app::{Action, App, Reply};

/// Runs the client on the alternate screen until Ctrl+C. The terminal comes back on exit,
/// and on a panic through the hook of `ratatui`.
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
    let mut terminal = ratatui::try_init()?;
    let result = event_loop(&mut terminal, &mut app, &client).await;
    ratatui::restore();
    result
}

async fn event_loop(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    client: &PortalClient,
) -> io::Result<()> {
    let mut keys = EventStream::new();
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
