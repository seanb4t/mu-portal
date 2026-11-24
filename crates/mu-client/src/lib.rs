//! API client of `mu`, with the resume of the event stream.

use std::path::PathBuf;
use std::time::Duration;

use backon::{BackoffBuilder as _, ExponentialBackoff, ExponentialBuilder};
use connectrpc::ConnectError;
use connectrpc::client::{ClientConfig, Http2Connection, ServerStream, SharedHttp2Connection};
use futures::{Stream, StreamExt as _};
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
    /// The store hash of the client layers, from the watch of the configuration: the current
    /// hash at each open of the watch, then each new one.
    StoreHash(String),
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

type Hashes = ServerStream<
    <Transport as connectrpc::client::ClientTransport>::ResponseBody,
    proto::__buffa::view::WatchConfigResponseView<'static>,
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
        Ok(self.list().await?.worlds)
    }

    /// The configuration directory that the server read its worlds from, or `None` for a
    /// server with no directory.
    ///
    /// # Errors
    ///
    /// A [`ClientError`] if the call fails.
    pub async fn config_dir(&self) -> Result<Option<String>, ClientError> {
        Ok(self.list().await?.config_dir)
    }

    /// The reply of one `ListWorlds` call.
    async fn list(&self) -> Result<proto::ListWorldsResponse, ClientError> {
        let reply = self
            .inner
            .list_worlds(proto::ListWorldsRequest::default())
            .await?;
        Ok(reply.into_owned())
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

    /// The configuration of the client of `kind`, `os` and `device`.
    ///
    /// # Errors
    ///
    /// A [`ClientError`] if the call fails, such as `invalid_argument` for a bad name.
    pub async fn client_config(
        &self,
        kind: &str,
        os: &str,
        device: &str,
    ) -> Result<proto::GetClientConfigResponse, ClientError> {
        let request = proto::GetClientConfigRequest {
            kind: kind.to_owned(),
            os: os.to_owned(),
            device: device.to_owned(),
            ..Default::default()
        };
        Ok(self.inner.get_client_config(request).await?.into_owned())
    }

    /// Asks the server to read its configuration directory again.
    ///
    /// # Errors
    ///
    /// A [`ClientError`] if the call fails.
    pub async fn reload_config(&self) -> Result<proto::ReloadConfigResponse, ClientError> {
        let request = proto::ReloadConfigRequest::default();
        Ok(self.inner.reload_config(request).await?.into_owned())
    }

    /// One watch of the store hash, with no resume: the current hash, then each new one.
    ///
    /// # Errors
    ///
    /// A [`ClientError`] if the call fails.
    pub async fn watch_config(&self) -> Result<HashStream, ClientError> {
        let request = proto::WatchConfigRequest::default();
        Ok(HashStream(self.inner.watch_config(request).await?))
    }

    /// One stream from `from_seq`, with no resume: the `Hello`, then each event.
    ///
    /// # Errors
    ///
    /// A [`ClientError`] if the call fails.
    pub async fn subscribe(&self, from_seq: u64) -> Result<EventStream, ClientError> {
        let request = proto::SubscribeRequest {
            from_seq,
            ..Default::default()
        };
        Ok(EventStream(self.inner.subscribe(request).await?))
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
            reopen: Reopen::new(),
        };
        futures::stream::unfold(resume, |mut resume| async move {
            let event = resume.next().await;
            Some((event, resume))
        })
    }

    /// The store hashes of the client layers as [`ClientEvent::StoreHash`], with no end. After
    /// an error or a clean end, the watch waits by [`backoff`] and opens again, and its first
    /// hash is the current one.
    pub fn config_hashes(&self) -> impl Stream<Item = ClientEvent> + Send + 'static {
        let client = self.clone();
        watch(move || {
            let client = client.clone();
            async move { Ok(client.watch_config().await?.into_stream()) }
        })
    }
}

/// The stream of one open of a watch.
type HashItems = std::pin::Pin<Box<dyn Stream<Item = Result<String, ClientError>> + Send>>;

/// Each hash of the streams that `open` gives, with no end. A failed open, an error and a clean
/// end each open again after a wait by [`backoff`].
fn watch<O, F>(open: O) -> impl Stream<Item = ClientEvent> + Send
where
    O: FnMut() -> F + Send,
    F: Future<Output = Result<HashItems, ClientError>> + Send,
{
    let state = (open, None::<HashItems>, Reopen::new());
    futures::stream::unfold(state, |(mut open, mut stream, mut reopen)| async move {
        loop {
            let Some(items) = stream.as_mut() else {
                reopen.pause().await;
                match open().await {
                    Ok(items) => stream = Some(items),
                    Err(_) => reopen.failed(),
                }
                continue;
            };
            if let Some(Ok(hash)) = items.next().await {
                reopen.reached();
                return Some((ClientEvent::StoreHash(hash), (open, stream, reopen)));
            }
            stream = None;
            reopen.failed();
        }
    })
}

/// The wait before a stream opens again.
struct Reopen {
    backoff: ExponentialBackoff,
    /// The next open waits first.
    wait: bool,
}

impl Reopen {
    fn new() -> Self {
        Self {
            backoff: backoff(),
            wait: false,
        }
    }

    /// Waits by the backoff, if the last open failed or the last stream ended.
    async fn pause(&mut self) {
        if self.wait {
            tokio::time::sleep(self.backoff.next().unwrap_or(LONGEST_WAIT)).await;
        }
    }

    /// The open failed or the stream ended, so the next open waits.
    const fn failed(&mut self) {
        self.wait = true;
    }

    /// A message of the open stream came: the next open does not wait, and the backoff starts
    /// again.
    fn reached(&mut self) {
        self.backoff = backoff();
        self.wait = false;
    }
}

/// One stream of [`PortalClient::subscribe`].
pub struct EventStream(Events);

impl EventStream {
    /// The next message. None means a clean end.
    ///
    /// # Errors
    ///
    /// A [`ClientError`] if the stream ends with an error.
    pub async fn next(&mut self) -> Result<Option<proto::SubscribeResponse>, ClientError> {
        Ok(self
            .0
            .message()
            .await?
            .map(|message| message.to_owned_message()))
    }
}

/// One stream of [`PortalClient::watch_config`].
pub struct HashStream(Hashes);

impl HashStream {
    /// The next store hash. None means a clean end.
    ///
    /// # Errors
    ///
    /// A [`ClientError`] if the stream ends with an error.
    pub async fn next(&mut self) -> Result<Option<String>, ClientError> {
        Ok(self
            .0
            .message()
            .await?
            .map(|message| message.to_owned_message().store_hash))
    }

    /// The hashes as a stream that ends after a clean end or an error.
    fn into_stream(self) -> HashItems {
        Box::pin(futures::stream::unfold(Some(self), |hashes| async move {
            let mut hashes = hashes?;
            match hashes.next().await {
                Ok(Some(hash)) => Some((Ok(hash), Some(hashes))),
                Ok(None) => None,
                Err(error) => Some((Err(error), None)),
            }
        }))
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
    reopen: Reopen,
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
                self.reopen.failed();
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
        self.reopen.pause().await;
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
            Err(_) => self.reopen.failed(),
        }
    }

    fn hello(&mut self, instance_id: String) -> Option<ClientEvent> {
        self.reopen.reached();
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

    /// A watch whose opens give `opens` in order: a list of hashes for a stream that ends after
    /// them, or `None` for a failed open. The count of opens comes back in `count`.
    fn scripted(
        opens: Vec<Option<Vec<&'static str>>>,
        count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) -> impl Stream<Item = ClientEvent> {
        let mut opens = opens.into_iter();
        watch(move || {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let next = opens.next();
            async move {
                match next {
                    Some(Some(hashes)) => Ok(Box::pin(futures::stream::iter(
                        hashes.into_iter().map(|hash| Ok(hash.to_owned())),
                    )) as HashItems),
                    Some(None) => Err(ClientError {
                        code: "unavailable",
                        message: String::new(),
                    }),
                    None => Ok(Box::pin(futures::stream::pending()) as HashItems),
                }
            }
        })
    }

    fn hash(text: &str) -> ClientEvent {
        ClientEvent::StoreHash(text.to_owned())
    }

    #[tokio::test(start_paused = true)]
    async fn the_watch_reports_each_hash() {
        let count = std::sync::Arc::default();
        let hashes: Vec<ClientEvent> = scripted(vec![Some(vec!["a", "b"])], count)
            .take(2)
            .collect()
            .await;
        assert_eq!(hashes, [hash("a"), hash("b")]);
    }

    #[tokio::test(start_paused = true)]
    async fn the_watch_opens_again() {
        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let opens = vec![Some(vec!["a"]), None, Some(vec!["b"])];
        let hashes: Vec<ClientEvent> = scripted(opens, std::sync::Arc::clone(&count))
            .take(2)
            .collect()
            .await;
        assert_eq!(
            (hashes, count.load(std::sync::atomic::Ordering::SeqCst)),
            (vec![hash("a"), hash("b")], 3)
        );
    }

    #[tokio::test(start_paused = true)]
    async fn the_watch_waits_before_each_open_after_a_loss() {
        let count = std::sync::Arc::default();
        let opens = vec![Some(vec!["a"]), None, Some(vec!["b"])];
        let start = tokio::time::Instant::now();
        let hashes: Vec<ClientEvent> = scripted(opens, count).take(2).collect().await;
        // The timer of the paused clock rounds each sleep up to the next millisecond.
        let waited = start.elapsed();
        assert_eq!(
            (
                hashes,
                waited >= Duration::from_millis(300),
                waited < Duration::from_millis(310)
            ),
            (vec![hash("a"), hash("b")], true, true)
        );
    }

    #[tokio::test(start_paused = true)]
    async fn backoff_runs_past_four_tries() {
        let waits = waits(100).await;
        assert_eq!(waits.len(), 100);
        assert_eq!(waits[99], LONGEST_WAIT);
    }
}
