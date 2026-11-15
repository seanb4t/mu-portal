//! A ConnectRPC server with an event log that a client resumes from a
//! sequence number, and a relay that cuts live connections.

#![allow(refining_impl_trait_internal, refining_impl_trait_reachable)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use connectrpc::{
    ConnectError, RequestContext, Response, Router, ServiceRequest, ServiceResult, ServiceStream,
};
use futures::StreamExt;
use tokio::sync::{broadcast, watch};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

pub mod proto {
    connectrpc::include_generated!();
}

pub mod relay;

pub use proto::mu::spike::v1::*;

/// Live events a slow subscriber can fall behind by before the server ends
/// its stream.
pub const LIVE_BUFFER: usize = 1024;

struct LogInner {
    events: Vec<Event>,
    live: broadcast::Sender<Event>,
}

/// The event log. `seq` starts at 1 and has no gaps.
#[derive(Clone)]
pub struct Log {
    inner: Arc<Mutex<LogInner>>,
    subscribers: Arc<watch::Sender<usize>>,
    versions: Arc<Mutex<Vec<http::Version>>>,
}

impl Default for Log {
    fn default() -> Self {
        Self::new()
    }
}

impl Log {
    pub fn new() -> Self {
        let (live, _) = broadcast::channel(LIVE_BUFFER);
        Self {
            inner: Arc::new(Mutex::new(LogInner {
                events: Vec::new(),
                live,
            })),
            subscribers: Arc::new(watch::channel(0).0),
            versions: Arc::default(),
        }
    }

    pub fn append(&self, text: String) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        let seq = inner.events.len() as u64 + 1;
        let event = Event {
            seq,
            text,
            ..Default::default()
        };
        inner.events.push(event.clone());
        let _ = inner.live.send(event);
        seq
    }

    /// The events from `from_seq`, then each new one. The backlog and the
    /// live receiver come from one lock, so the stream has no gap and no
    /// duplicate.
    pub fn stream_from(&self, from_seq: u64) -> ServiceStream<Event> {
        let (backlog, live) = {
            let inner = self.inner.lock().unwrap();
            let start = from_seq.saturating_sub(1) as usize;
            let backlog = inner.events.get(start..).unwrap_or_default().to_vec();
            (backlog, inner.live.subscribe())
        };
        let guard = SubscriberGuard::new(self.subscribers.clone());
        let live = BroadcastStream::new(live).map(|item| match item {
            Ok(event) => Ok(event),
            Err(BroadcastStreamRecvError::Lagged(n)) => Err(ConnectError::resource_exhausted(
                format!("subscriber fell {n} events behind; resume from the last seq"),
            )),
        });
        let stream = futures::stream::iter(backlog.into_iter().map(Ok))
            .chain(live)
            .map(move |item| {
                let _ = &guard;
                item
            });
        Box::pin(stream)
    }

    /// The HTTP version of each request the server received, in order.
    pub fn http_versions(&self) -> Vec<http::Version> {
        self.versions.lock().unwrap().clone()
    }

    /// The number of open Subscribe streams on the server.
    pub fn subscribers(&self) -> watch::Receiver<usize> {
        self.subscribers.subscribe()
    }
}

struct SubscriberGuard(Arc<watch::Sender<usize>>);

impl SubscriberGuard {
    fn new(count: Arc<watch::Sender<usize>>) -> Self {
        count.send_modify(|n| *n += 1);
        Self(count)
    }
}

impl Drop for SubscriberGuard {
    fn drop(&mut self) {
        self.0.send_modify(|n| *n -= 1);
    }
}

pub struct Portal {
    pub log: Log,
}

impl PortalService for Portal {
    async fn send(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, SendRequest>,
    ) -> ServiceResult<SendResponse> {
        let seq = self.log.append(request.text.to_string());
        Response::ok(SendResponse {
            seq,
            ..Default::default()
        })
    }

    async fn subscribe(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, SubscribeRequest>,
    ) -> ServiceResult<ServiceStream<Event>> {
        Response::stream_ok(self.log.stream_from(request.from_seq))
    }
}

pub fn app(log: Log) -> axum::Router {
    let versions = log.versions.clone();
    Arc::new(Portal { log })
        .register(Router::new())
        .into_axum_router()
        .layer(axum::middleware::map_request(
            move |req: axum::extract::Request| {
                versions.lock().unwrap().push(req.version());
                async move { req }
            },
        ))
}

/// Where a server or a relay listens.
#[derive(Clone, Debug)]
pub enum Endpoint {
    Tcp(std::net::SocketAddr),
    Unix(PathBuf),
}

/// Serves the API on `listener` until the task is dropped.
pub async fn serve_tcp(listener: tokio::net::TcpListener, log: Log) -> std::io::Result<()> {
    axum::serve(listener, app(log)).await
}

pub async fn serve_unix(listener: tokio::net::UnixListener, log: Log) -> std::io::Result<()> {
    axum::serve(listener, app(log)).await
}
