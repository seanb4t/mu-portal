//! The `PortalService` of the schema over the `Portal` of the engine.

use std::future::{Future, Ready, ready};
use std::sync::Arc;

use connectrpc::{
    ConnectError, RequestContext, Response, Router, ServiceRequest, ServiceResult, ServiceStream,
};
use futures::StreamExt as _;
use mu_core::{Journal, Lagged, Portal, PortalError, Seq, Subscription, WorldId};
use mu_proto as proto;
use proto::__buffa::oneof::subscribe_response::Message;
use proto::PortalServiceExt as _;
use tokio::sync::watch;
use tracing::{Instrument as _, Span};

use crate::convert;
use mu_config::{DeviceName, Kind, NameError, Os};

use crate::store::{ClientPart, Notice, Reloaded, Store};

/// The API of one server instance.
pub struct PortalApi {
    portal: Portal,
    instance_id: String,
    stopping: watch::Receiver<bool>,
    store: Arc<Store>,
    config_dir: Option<String>,
}

impl PortalApi {
    /// The API over `portal` and the configuration `store`. Each stream ends when `stopping`
    /// turns true. `ListWorlds`
    /// returns `config_dir` as the configuration directory of the server.
    #[must_use]
    pub fn new(
        portal: Portal,
        instance_id: String,
        stopping: watch::Receiver<bool>,
        store: Arc<Store>,
        config_dir: Option<String>,
    ) -> Self {
        Self {
            portal,
            instance_id,
            stopping,
            store,
            config_dir,
        }
    }

    /// The axum router of the API.
    pub fn router(self) -> axum::Router {
        Arc::new(self).register(Router::new()).into_axum_router()
    }
}

fn hello(instance_id: &str, last: Seq) -> proto::SubscribeResponse {
    proto::SubscribeResponse {
        message: Some(Message::Hello(Box::new(proto::Hello {
            instance_id: instance_id.to_owned(),
            last_seq: last.0,
            ..Default::default()
        }))),
        ..Default::default()
    }
}

fn event_message(event: &mu_core::Event) -> proto::SubscribeResponse {
    proto::SubscribeResponse {
        message: Some(Message::Event(Box::new(convert::event(event)))),
        ..Default::default()
    }
}

/// The stream of one subscriber: the `Hello`, then each event. A lag ends it with
/// `resource_exhausted`, and `stopping` ends it with no error. For each event, it records the
/// gap to the last event of `journal` in `mu.subscriber.lag`, and it counts each lag in
/// `mu.subscriber.lagged`. Without a metrics layer, it reads no gap, because the read locks the
/// journal.
pub fn event_stream(
    subscription: Subscription,
    journal: Arc<dyn Journal>,
    instance_id: &str,
    stopping: watch::Receiver<bool>,
) -> ServiceStream<proto::SubscribeResponse> {
    let head = futures::stream::once(ready(Ok(hello(instance_id, subscription.last()))));
    let events = subscription.into_stream().map(move |item| match item {
        Ok(event) => {
            if tracing::enabled!(target: mu_core::METRICS_TARGET, tracing::Level::INFO) {
                let lag = journal.last().0.saturating_sub(event.seq.0);
                tracing::event!(
                    target: mu_core::METRICS_TARGET,
                    tracing::Level::INFO,
                    "histogram.mu.subscriber.lag" = lag,
                );
            }
            Ok(event_message(&event))
        }
        Err(Lagged(n)) => {
            tracing::event!(
                target: mu_core::METRICS_TARGET,
                tracing::Level::INFO,
                "monotonic_counter.mu.subscriber.lagged" = 1_u64,
            );
            Err(ConnectError::resource_exhausted(format!(
                "the subscriber fell {n} events behind; subscribe again from the next number"
            )))
        }
    });
    // The stop ends the stream only when no event is ready, so the events that the server
    // recorded before the stop, for example the disconnect of each world, still go out.
    let events = futures::stream::unfold(
        (Box::pin(events), stopping),
        |(mut events, mut stopping)| async move {
            let item = tokio::select! {
                biased;
                item = events.next() => item?,
                // A dropped sender also means stop.
                _ = stopping.wait_for(|stop| *stop) => return None,
            };
            Some((item, (events, stopping)))
        },
    );
    Box::pin(head.chain(events))
}

/// The configuration of the client of `kind`, `os` and `device` in `part`. A bad name is
/// `invalid_argument`, and its message names it.
///
/// # Errors
///
/// A [`ConnectError`] that names the bad kind, OS or device name.
pub fn client_config(
    part: &ClientPart,
    kind: &str,
    os: &str,
    device: &str,
) -> Result<proto::GetClientConfigResponse, ConnectError> {
    let refused = |error: NameError| ConnectError::invalid_argument(error.to_string());
    let kind = Kind::parse(kind).map_err(refused)?;
    let os = Os::parse(os).map_err(refused)?;
    let device = DeviceName::new(device).map_err(refused)?;
    let config = part.resolve(kind, os, &device);
    Ok(proto::GetClientConfigResponse {
        store_hash: config.store_hash,
        revision: config.revision,
        echo: config.echo.name().to_owned(),
        bindings: config
            .bindings
            .into_iter()
            .map(|(key, entry)| proto::Binding {
                key,
                entry,
                ..Default::default()
            })
            .collect(),
        sources: config.sources.into_iter().collect(),
        notices: config.notices.iter().map(notice).collect(),
        ..Default::default()
    })
}

fn notice(notice: &Notice) -> proto::Notice {
    proto::Notice {
        path: notice.path.clone(),
        text: notice.text.clone(),
        ..Default::default()
    }
}

/// The reply of a reload.
#[must_use]
pub fn reloaded(reloaded: &Reloaded) -> proto::ReloadConfigResponse {
    proto::ReloadConfigResponse {
        notices: reloaded.notices.iter().map(notice).collect(),
        restart: reloaded.restart.clone(),
        ..Default::default()
    }
}

/// The stream of one watcher: the current store hash, then each new one. `stopping` ends it
/// with no error, and so does the end of the store.
pub fn watch_stream(
    mut parts: watch::Receiver<Arc<ClientPart>>,
    stopping: watch::Receiver<bool>,
) -> ServiceStream<proto::WatchConfigResponse> {
    parts.mark_changed();
    Box::pin(futures::stream::unfold(
        (parts, stopping),
        |(mut parts, mut stopping)| async move {
            let store_hash = tokio::select! {
                biased;
                // A dropped sender also means stop.
                _ = stopping.wait_for(|stop| *stop) => return None,
                changed = parts.changed() => {
                    changed.ok()?;
                    parts.borrow_and_update().store_hash().to_owned()
                }
            };
            let message = proto::WatchConfigResponse {
                store_hash,
                ..Default::default()
            };
            Some((Ok(message), (parts, stopping)))
        },
    ))
}

/// The span of one call of `method`. The caller records `mu.world` and the status where they
/// apply. A failed call also records the name of its Connect error code, never its message.
fn call_span(method: &'static str) -> Span {
    let service = proto::PORTAL_SERVICE_SERVICE_NAME;
    tracing::info_span!(
        "rpc",
        otel.name = format!("{service}/{method}"),
        rpc.system = "connect_rpc",
        rpc.service = service,
        rpc.method = method,
        mu.world = tracing::field::Empty,
        otel.status_code = tracing::field::Empty,
        rpc.connect_rpc.error_code = tracing::field::Empty,
    )
}

/// Marks `span` as failed with `error`.
fn failed(span: &Span, error: &ConnectError) {
    span.record("otel.status_code", "ERROR");
    span.record("rpc.connect_rpc.error_code", error.code.as_str());
}

/// Marks `span` as failed if `result` is an error.
fn ended<T>(span: &Span, result: ServiceResult<T>) -> ServiceResult<T> {
    if let Err(error) = &result {
        failed(span, error);
    }
    result
}

/// Runs `call` in the span of one call of `method`.
async fn traced<T>(
    method: &'static str,
    call: impl Future<Output = ServiceResult<T>>,
) -> ServiceResult<T> {
    let span = call_span(method);
    let result = call.instrument(span.clone()).await;
    ended(&span, result)
}

/// The world of a request, recorded in the current span.
fn named_world(text: &str) -> Result<WorldId, ConnectError> {
    let world = world(text)?;
    Span::current().record("mu.world", tracing::field::display(&world));
    Ok(world)
}

/// The world of a request. A text that is not an id names no world, so it is `not_found` too.
fn world(text: &str) -> Result<WorldId, ConnectError> {
    text.parse()
        .map_err(|_| ConnectError::not_found(format!("unknown world {text:?}")))
}

#[expect(
    refining_impl_trait_reachable,
    reason = "connectrpc generates `impl Encodable` returns; the concrete messages implement it"
)]
impl proto::PortalService for PortalApi {
    fn subscribe(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, proto::SubscribeRequest>,
    ) -> Ready<ServiceResult<ServiceStream<proto::SubscribeResponse>>> {
        // The span ends when this method returns, not with the stream.
        let span = call_span("Subscribe");
        let _entered = span.enter();
        ready(ended(
            &span,
            match self.portal.subscribe(Seq(request.from_seq)) {
                Ok(subscription) => Response::stream_ok(event_stream(
                    subscription,
                    Arc::clone(self.portal.journal()),
                    &self.instance_id,
                    self.stopping.clone(),
                )),
                Err(error @ PortalError::PastEnd { last, .. }) => {
                    let status = convert::status(&error);
                    failed(&span, &status);
                    let items = [Ok(hello(&self.instance_id, last)), Err(status)];
                    Response::stream_ok(Box::pin(futures::stream::iter(items)))
                }
                Err(error) => Err(convert::status(&error)),
            },
        ))
    }

    fn list_worlds(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, proto::ListWorldsRequest>,
    ) -> Ready<ServiceResult<proto::ListWorldsResponse>> {
        let span = call_span("ListWorlds");
        let _entered = span.enter();
        let worlds = self
            .portal
            .worlds()
            .into_iter()
            .map(|(id, status)| convert::world(&id, status, self.portal.line_break(&id)))
            .collect();
        ready(Response::ok(proto::ListWorldsResponse {
            worlds,
            config_dir: self.config_dir.clone(),
            ..Default::default()
        }))
    }

    async fn send(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, proto::SendRequest>,
    ) -> ServiceResult<proto::SendResponse> {
        traced("Send", async {
            let world = named_world(request.world)?;
            let seq = self
                .portal
                .send(&world, request.text)
                .await
                .map_err(|error| convert::status(&error))?;
            Response::ok(proto::SendResponse {
                seq: seq.map(|seq| seq.0),
                ..Default::default()
            })
        })
        .await
    }

    async fn connect(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, proto::ConnectRequest>,
    ) -> ServiceResult<proto::ConnectResponse> {
        traced("Connect", async {
            let world = named_world(request.world)?;
            self.portal
                .connect(&world)
                .await
                .map_err(|error| convert::status(&error))?;
            Response::ok(proto::ConnectResponse::default())
        })
        .await
    }

    async fn get_client_config(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, proto::GetClientConfigRequest>,
    ) -> ServiceResult<proto::GetClientConfigResponse> {
        traced("GetClientConfig", async {
            let reply = client_config(
                &self.store.client(),
                request.kind,
                request.os,
                request.device,
            )?;
            Response::ok(reply)
        })
        .await
    }

    fn watch_config(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, proto::WatchConfigRequest>,
    ) -> Ready<ServiceResult<ServiceStream<proto::WatchConfigResponse>>> {
        // The span ends when this method returns, not with the stream.
        let span = call_span("WatchConfig");
        let _entered = span.enter();
        ready(Response::stream_ok(watch_stream(
            self.store.watch(),
            self.stopping.clone(),
        )))
    }

    async fn reload_config(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, proto::ReloadConfigRequest>,
    ) -> ServiceResult<proto::ReloadConfigResponse> {
        traced("ReloadConfig", async {
            Response::ok(reloaded(&self.store.reload().await))
        })
        .await
    }

    async fn disconnect(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, proto::DisconnectRequest>,
    ) -> ServiceResult<proto::DisconnectResponse> {
        traced("Disconnect", async {
            let world = named_world(request.world)?;
            self.portal
                .disconnect(&world)
                .await
                .map_err(|error| convert::status(&error))?;
            Response::ok(proto::DisconnectResponse::default())
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use connectrpc::ErrorCode;
    use mu_core::{EventKind, MemoryJournal, NewEvent, Source};

    use super::*;

    fn append(journal: &MemoryJournal, text: &str) {
        journal.append(NewEvent {
            world: "alpha/aria".parse().unwrap(),
            kind: EventKind::Line,
            source: Source::Game,
            text: text.to_owned(),
            depth: 0,
        });
    }

    fn text(item: &Result<proto::SubscribeResponse, ConnectError>) -> &str {
        match &item.as_ref().unwrap().message {
            Some(Message::Event(event)) => &event.text,
            other => panic!("not an event: {other:?}"),
        }
    }

    #[tokio::test]
    async fn lagged_stream_ends_with_resource_exhausted() {
        let journal = Arc::new(MemoryJournal::new(4).unwrap());
        let (_stop, stopping) = watch::channel(false);
        let mut slow = event_stream(
            journal.subscribe(Seq(1)).unwrap(),
            journal.clone(),
            "id",
            stopping.clone(),
        );
        let mut fast = event_stream(
            journal.subscribe(Seq(1)).unwrap(),
            journal.clone(),
            "id",
            stopping,
        );
        assert!(matches!(
            slow.next().await.unwrap().unwrap().message,
            Some(Message::Hello(_))
        ));
        assert!(matches!(
            fast.next().await.unwrap().unwrap().message,
            Some(Message::Hello(_))
        ));
        for n in 1..=5 {
            append(&journal, &format!("line {n}"));
            assert_eq!(text(&fast.next().await.unwrap()), format!("line {n}"));
        }
        let error = slow.next().await.unwrap().unwrap_err();
        assert_eq!(error.code, ErrorCode::ResourceExhausted);
        assert!(slow.next().await.is_none());
    }

    #[tokio::test]
    async fn stop_sends_the_ready_events_first() {
        let journal = Arc::new(MemoryJournal::new(4).unwrap());
        let (stop, stopping) = watch::channel(false);
        let mut stream = event_stream(
            journal.subscribe(Seq(1)).unwrap(),
            journal.clone(),
            "id",
            stopping,
        );
        assert!(stream.next().await.is_some());
        append(&journal, "the last line");
        stop.send_replace(true);
        assert_eq!(text(&stream.next().await.unwrap()), "the last line");
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn stopping_ends_the_stream() {
        let journal = Arc::new(MemoryJournal::new(4).unwrap());
        let (stop, stopping) = watch::channel(false);
        let mut stream = event_stream(
            journal.subscribe(Seq(1)).unwrap(),
            journal.clone(),
            "id",
            stopping,
        );
        assert!(stream.next().await.is_some());
        stop.send_replace(true);
        assert!(stream.next().await.is_none());
    }
}
