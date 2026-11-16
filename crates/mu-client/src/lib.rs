//! API client of `mu`, with the resume of the event stream.

use std::path::PathBuf;
use std::time::Duration;

use backon::{BackoffBuilder as _, ExponentialBackoff, ExponentialBuilder};
use connectrpc::ConnectError;
use connectrpc::client::{ClientConfig, Http2Connection, ServerStream, SharedHttp2Connection};
use futures::Stream;
use mu_proto as proto;
use proto::__buffa::oneof::subscribe_response::Message;
use thiserror::Error;

/// A failed call. `code` is the name of the `ConnectRPC` code, for example `not_found`.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("{code}: {message}")]
pub struct ClientError {
    /// The name of the code.
    pub code: &'static str,
    /// The message of the server.
    pub message: String,
}

impl From<ConnectError> for ClientError {
    fn from(error: ConnectError) -> Self {
        Self {
            code: error.code.as_str(),
            message: error.message.unwrap_or_default(),
        }
    }
}

/// One item of [`PortalClient::events`].
#[derive(Clone, Debug, PartialEq)]
pub enum ClientEvent {
    /// The stream reached a server instance that this stream did not read before. The
    /// events start again from 1.
    Reset {
        /// The id of the server instance.
        instance_id: String,
    },
    /// The stream is back on the same server instance, with no gap.
    Resumed,
    /// One event, in the order of its number.
    Event(proto::Event),
    /// The stream ended. The client subscribes again by itself.
    Lost,
}

/// The shortest wait between two subscribes.
const FIRST_WAIT: Duration = Duration::from_millis(100);
/// The longest wait between two subscribes.
const LONGEST_WAIT: Duration = Duration::from_secs(5);

/// The waits between two tries: from 100 milliseconds, twice as long each time, to at most
/// 5 seconds, with no end.
#[must_use]
pub fn backoff() -> ExponentialBackoff {
    ExponentialBuilder::new()
        .with_min_delay(FIRST_WAIT)
        .with_max_delay(LONGEST_WAIT)
        .with_factor(2.0)
        .without_max_times()
        .build()
}

type Transport = SharedHttp2Connection;
type Events = ServerStream<
    <Transport as connectrpc::client::ClientTransport>::ResponseBody,
    proto::__buffa::view::SubscribeResponseView<'static>,
>;

/// The client of one server, on its Unix socket.
#[derive(Clone)]
pub struct PortalClient {
    inner: proto::PortalServiceClient<Transport>,
}

impl PortalClient {
    /// A client of the server at `socket`. It connects at the first call.
    #[must_use]
    pub fn unix(socket: impl Into<PathBuf>) -> Self {
        let authority = http::Uri::from_static("http://localhost");
        let connection = Http2Connection::lazy_unix(socket, authority.clone()).shared(1024);
        Self {
            inner: proto::PortalServiceClient::new(connection, ClientConfig::new(authority)),
        }
    }

    /// Each world with its state.
    ///
    /// # Errors
    ///
    /// A [`ClientError`] if the call fails.
    pub async fn list_worlds(&self) -> Result<Vec<proto::World>, ClientError> {
        let reply = self
            .inner
            .list_worlds(proto::ListWorldsRequest::default())
            .await?;
        Ok(reply.into_owned().worlds)
    }

    /// Sends one line to `world`. No number means that a hook gagged the line.
    ///
    /// # Errors
    ///
    /// A [`ClientError`] if the call fails.
    pub async fn send(&self, world: &str, text: &str) -> Result<Option<u64>, ClientError> {
        let request = proto::SendRequest {
            world: world.to_owned(),
            text: text.to_owned(),
            ..Default::default()
        };
        Ok(self.inner.send(request).await?.into_owned().seq)
    }

    /// Connects `world`.
    ///
    /// # Errors
    ///
    /// A [`ClientError`] if the call fails.
    pub async fn connect(&self, world: &str) -> Result<(), ClientError> {
        let request = proto::ConnectRequest {
            world: world.to_owned(),
            ..Default::default()
        };
        self.inner.connect(request).await?;
        Ok(())
    }

    /// Disconnects `world`.
    ///
    /// # Errors
    ///
    /// A [`ClientError`] if the call fails.
    pub async fn disconnect(&self, world: &str) -> Result<(), ClientError> {
        let request = proto::DisconnectRequest {
            world: world.to_owned(),
            ..Default::default()
        };
        self.inner.disconnect(request).await?;
        Ok(())
    }

    /// The events from the first one, with no end. After an error or a clean end, the
    /// stream yields [`ClientEvent::Lost`], waits by [`backoff`] and subscribes again from
    /// the next number.
    pub fn events(&self) -> impl Stream<Item = ClientEvent> + Send + 'static {
        let resume = Resume {
            client: self.inner.clone(),
            last: 0,
            instance: None,
            stream: None,
            opened_from: 0,
            quiet_hello: false,
            backoff: backoff(),
            wait: false,
        };
        futures::stream::unfold(resume, |mut resume| async move {
            let event = resume.next().await;
            Some((event, resume))
        })
    }
}

struct Resume {
    client: proto::PortalServiceClient<Transport>,
    last: u64,
    instance: Option<String>,
    stream: Option<Events>,
    opened_from: u64,
    /// The next `Hello` follows a reset that this client made, so it yields nothing.
    quiet_hello: bool,
    backoff: ExponentialBackoff,
    /// The next subscribe waits first.
    wait: bool,
}

impl Resume {
    async fn next(&mut self) -> ClientEvent {
        loop {
            let Some(stream) = self.stream.as_mut() else {
                self.open().await;
                continue;
            };
            let Ok(Some(message)) = stream.message().await else {
                self.stream = None;
                self.wait = true;
                return ClientEvent::Lost;
            };
            let message = message.to_owned_message().message;
            match message {
                Some(Message::Hello(hello)) => {
                    if let Some(event) = self.hello(hello.instance_id) {
                        return event;
                    }
                }
                // A number at or below the last one is already read.
                Some(Message::Event(event)) if event.seq > self.last => {
                    self.last = event.seq;
                    return ClientEvent::Event(*event);
                }
                Some(Message::Event(_)) | None => {}
            }
        }
    }

    /// Subscribes from the next number. A failure waits by the backoff and tries again.
    async fn open(&mut self) {
        if self.wait {
            tokio::time::sleep(self.backoff.next().unwrap_or(LONGEST_WAIT)).await;
        }
        let from = self.last + 1;
        let request = proto::SubscribeRequest {
            from_seq: from,
            ..Default::default()
        };
        match self.client.subscribe(request).await {
            Ok(stream) => {
                self.stream = Some(stream);
                self.opened_from = from;
            }
            Err(_) => self.wait = true,
        }
    }

    fn hello(&mut self, instance_id: String) -> Option<ClientEvent> {
        self.backoff = backoff();
        self.wait = false;
        if self.instance.as_ref() == Some(&instance_id) {
            return (!std::mem::take(&mut self.quiet_hello)).then_some(ClientEvent::Resumed);
        }
        self.instance = Some(instance_id.clone());
        self.last = 0;
        if self.opened_from > 1 {
            // The stream starts past the first event of the new instance, so subscribe from 1.
            self.stream = None;
            self.quiet_hello = true;
        }
        Some(ClientEvent::Reset { instance_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sleeps each wait of the backoff on the paused clock, as the resume does, and returns
    /// each wait.
    async fn waits(count: usize) -> Vec<Duration> {
        let mut backoff = backoff();
        let mut waits = Vec::new();
        for _ in 0..count {
            let wait = backoff.next().unwrap();
            tokio::time::sleep(wait).await;
            waits.push(wait);
        }
        waits
    }

    #[tokio::test(start_paused = true)]
    async fn backoff_schedule() {
        let millis: Vec<u128> = waits(8).await.iter().map(Duration::as_millis).collect();
        assert_eq!(millis, [100, 200, 400, 800, 1600, 3200, 5000, 5000]);
    }

    #[tokio::test(start_paused = true)]
    async fn backoff_runs_past_four_tries() {
        let waits = waits(100).await;
        assert_eq!(waits.len(), 100);
        assert_eq!(waits[99], LONGEST_WAIT);
    }
}
