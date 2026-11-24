//! The loop of the terminal client: keys, the event stream and the replies of calls.

use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt as _;
use mu_client::PortalClient;
use termina::escape::osc::{Osc, Selection};
use termina::{EventReader, EventStream};
use tokio::sync::mpsc;

use crate::kitty;
use crate::model::Model;
use crate::probe;
use crate::screen::{self, Restore, Screen};
use crate::update::{Effect, Msg, Reply, update};
use crate::view::view;

/// Runs the client with the client configuration `config` of `device` on the alternate screen
/// until the Quit action or the end of terminal input. A new store hash of the watch of the
/// configuration fetches the configuration of the kind `tui`, the OS of the machine and
/// `device` again. The terminal comes back on exit, and on a panic through the panic hook of
/// Termina.
///
/// # Errors
///
/// An I/O error of the terminal.
pub async fn run(
    client: PortalClient,
    device: String,
    config: mu_proto::GetClientConfigResponse,
) -> io::Result<()> {
    let mut model = Model::starting();
    model.apply_config(config);
    // The probe sets mode 2027, so a panic before its replies turns the mode off.
    let restore = Arc::new(Restore::default());
    restore.set_grapheme_mode(true);
    let (mut terminal, reader) = screen::open(&restore)?;
    let probed = probe::run(terminal.backend_mut(), probe::WAIT).and_then(|probed| {
        // Only a terminal that answered the query in time gets the flag. A late answer is a
        // terminal event, which the update drops.
        if probed.0.keyboard == probe::KeyboardProtocol::Kitty {
            screen::push_keyboard_flag(terminal.backend_mut(), &restore)?;
        }
        Ok(probed)
    });
    let result = match probed {
        Ok((mut outcome, input)) => {
            restore.set_grapheme_mode(outcome.resets_grapheme_mode());
            let open = std::mem::take(&mut outcome.open);
            // The probe result goes first, then the input that came during the probe, then the
            // bytes of a cut answer. The input came before those bytes.
            let early = std::iter::once(Msg::Probe(outcome))
                .chain(input.into_iter().map(Msg::Terminal))
                .chain((!open.is_empty()).then_some(Msg::CutReply(open)))
                .collect();
            let result =
                event_loop(&mut terminal, reader, early, &mut model, &client, &device).await;
            if model.keeps_grapheme_mode() {
                restore.set_grapheme_mode(false);
            }
            result
        }
        Err(error) => Err(error),
    };
    // Dropping the backend returns the terminal to cooked mode.
    let left = screen::leave(terminal.backend_mut(), &restore);
    screen::finish(result, left)
}

async fn event_loop(
    terminal: &mut Screen,
    reader: EventReader,
    mut early: VecDeque<Msg>,
    model: &mut Model,
    client: &PortalClient,
    device: &str,
) -> io::Result<()> {
    let mut terminal_events = EventStream::new(reader, |_| true);
    let mut events = Box::pin(client.events());
    let mut hashes = Box::pin(client.config_hashes());
    let (replies, mut reply) = mpsc::unbounded_channel();
    // The first list of the worlds, which `Model::starting` marks as asked.
    spawn_reply(&replies, listed_of(client.clone()));
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
            Some(item) = hashes.next() => Msg::Client(item),
            Some(msg) = reply.recv() => msg,
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
                    spawn_reply(&replies, async move {
                        let call = async {
                            client
                                .send(&world, &text)
                                .await
                                .map_err(|error| error.to_string())
                        };
                        let answer = reply_of(call, world.clone(), text.clone(), CALL_LIMIT).await;
                        Msg::Reply(answer)
                    });
                }
                Effect::Connect { world } => {
                    spawn_reply(&replies, control(client.clone(), world, true));
                }
                Effect::Disconnect { world } => {
                    spawn_reply(&replies, control(client.clone(), world, false));
                }
                Effect::ListWorlds => spawn_reply(&replies, listed_of(client.clone())),
                Effect::FetchConfig => {
                    spawn_reply(&replies, config_of(client.clone(), device.to_owned()));
                }
            }
        }
    }
}

/// The longest time that the client waits for the reply of a send or a list of the worlds. A
/// send waits on the server for at most two writes to the game, of `WRITE_TIMEOUT` (10 s) each
/// in `mu_core::session`. The limit is above that, so the server's own answer comes first when
/// the game is slow.
pub const CALL_LIMIT: Duration = Duration::from_secs(30);

/// The [`Msg::Control`] of a connect of `world` if `connect` is true, else of a disconnect,
/// within [`CALL_LIMIT`].
async fn control(client: PortalClient, world: String, connect: bool) -> Msg {
    let call = async {
        let result = if connect {
            client.connect(&world).await
        } else {
            client.disconnect(&world).await
        };
        result.map_err(|error| error.to_string())
    };
    control_of(call, world.clone(), CALL_LIMIT).await
}

/// The list of the worlds of `client`, within [`CALL_LIMIT`].
async fn listed_of(client: PortalClient) -> Msg {
    let call = async {
        client
            .list_worlds()
            .await
            .map_err(|error| error.to_string())
    };
    Msg::Listed(within(call, CALL_LIMIT, "").await)
}

/// The client configuration of the kind `tui`, the OS of the machine and `device`, within
/// [`CALL_LIMIT`].
async fn config_of(client: PortalClient, device: String) -> Msg {
    let call = async {
        crate::config::fetch(&client, &device)
            .await
            .map_err(|error| error.to_string())
    };
    Msg::Config(within(call, CALL_LIMIT, "").await)
}

/// Runs `call` on a task, and gives its message to the loop.
fn spawn_reply(
    replies: &mpsc::UnboundedSender<Msg>,
    call: impl Future<Output = Msg> + Send + 'static,
) {
    let replies = replies.clone();
    tokio::spawn(async move {
        // The loop holds the receiver until it returns.
        let _ = replies.send(call.await);
    });
}

/// The result of `call`, or a reason if it gives no answer within `limit`.
///
/// # Errors
///
/// The error of `call`, or `the server gave no answer in <seconds> s` followed by `late`.
pub async fn within<T>(
    call: impl Future<Output = Result<T, String>>,
    limit: Duration,
    late: &str,
) -> Result<T, String> {
    timed(call, limit, || {
        format!("the server gave no answer in {} s{late}", limit.as_secs())
    })
    .await
}

/// The answer of `call`, or the text of `late` when `limit` ends first.
///
/// # Errors
///
/// The error of `call`, or the text of `late`.
pub async fn timed<T>(
    call: impl Future<Output = Result<T, String>>,
    limit: Duration,
    late: impl FnOnce() -> String,
) -> Result<T, String> {
    tokio::time::timeout(limit, call)
        .await
        .unwrap_or_else(|_| Err(late()))
}

/// The reply to a send of `text` to `world` that `call` makes. A call that fails is a refusal
/// with its reason. A call with no answer within `limit` is a refusal too, because a send that
/// waits stops each later send of its world.
pub async fn reply_of<T>(
    call: impl Future<Output = Result<T, String>>,
    world: String,
    text: String,
    limit: Duration,
) -> Reply {
    match within(call, limit, ", so the game can have the line or not").await {
        Ok(_) => Reply::Sent { world, text },
        Err(reason) => Reply::Refused {
            world,
            text,
            reason,
        },
    }
}

/// The [`Msg::Control`] of a connect or a disconnect of `world` that `call` makes. A call with
/// no answer within `limit` gives that as its reason.
pub async fn control_of(
    call: impl Future<Output = Result<(), String>>,
    world: String,
    limit: Duration,
) -> Msg {
    let result = within(call, limit, "").await;
    Msg::Control { world, result }
}
