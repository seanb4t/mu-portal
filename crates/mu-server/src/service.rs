//! The `PortalService` of the schema over the `Portal` of the engine.

use std::future::{Ready, ready};
use std::sync::Arc;

use connectrpc::{
    ConnectError, RequestContext, Response, Router, ServiceRequest, ServiceResult, ServiceStream,
};
use futures::StreamExt as _;
use mu_core::{Lagged, Portal, PortalError, Seq, Subscription, WorldId};
use mu_proto as proto;
use proto::__buffa::oneof::subscribe_response::Message;
use proto::PortalServiceExt as _;
use tokio::sync::watch;

use crate::convert;

/// The API of one server instance.
pub struct PortalApi {
    portal: Portal,
    instance_id: String,
    stopping: watch::Receiver<bool>,
}

impl PortalApi {
    /// The API over `portal`. Each stream ends when `stopping` turns true.
    #[must_use]
    pub fn new(portal: Portal, instance_id: String, stopping: watch::Receiver<bool>) -> Self {
        Self {
            portal,
            instance_id,
            stopping,
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
/// `resource_exhausted`, and `stopping` ends it with no error.
pub fn event_stream(
    subscription: Subscription,
    instance_id: &str,
    stopping: watch::Receiver<bool>,
) -> ServiceStream<proto::SubscribeResponse> {
    let head = futures::stream::once(ready(Ok(hello(instance_id, subscription.last()))));
    // The stop ends the stream only when no event is ready, so the events that the server
    // recorded before the stop, for example the disconnect of each world, still go out.
    let events = futures::stream::unfold(Some(subscription), move |state| {
        let mut stopping = stopping.clone();
        async move {
            let mut subscription = state?;
            let item = tokio::select! {
                biased;
                item = subscription.next() => item?,
                // A dropped sender also means stop.
                _ = stopping.wait_for(|stop| *stop) => return None,
            };
            Some(match item {
                Ok(event) => (Ok(event_message(&event)), Some(subscription)),
                Err(Lagged(n)) => (
                    Err(ConnectError::resource_exhausted(format!(
                        "the subscriber fell {n} events behind; subscribe again from the next number"
                    ))),
                    None,
                ),
            })
        }
    });
    Box::pin(head.chain(events))
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
        ready(match self.portal.subscribe(Seq(request.from_seq)) {
            Ok(subscription) => Response::stream_ok(event_stream(
                subscription,
                &self.instance_id,
                self.stopping.clone(),
            )),
            Err(error @ PortalError::PastEnd { last, .. }) => {
                let items = [
                    Ok(hello(&self.instance_id, last)),
                    Err(convert::status(&error)),
                ];
                Response::stream_ok(Box::pin(futures::stream::iter(items)))
            }
            Err(error) => Err(convert::status(&error)),
        })
    }

    fn list_worlds(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, proto::ListWorldsRequest>,
    ) -> Ready<ServiceResult<proto::ListWorldsResponse>> {
        let worlds = self
            .portal
            .worlds()
            .into_iter()
            .map(|(id, state)| proto::World {
                id: id.to_string(),
                state: convert::state(state).into(),
                ..Default::default()
            })
            .collect();
        ready(Response::ok(proto::ListWorldsResponse {
            worlds,
            ..Default::default()
        }))
    }

    async fn send(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, proto::SendRequest>,
    ) -> ServiceResult<proto::SendResponse> {
        let world = world(request.world)?;
        let seq = self
            .portal
            .send(&world, request.text)
            .await
            .map_err(|error| convert::status(&error))?;
        Response::ok(proto::SendResponse {
            seq: seq.map(|seq| seq.0),
            ..Default::default()
        })
    }

    async fn connect(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, proto::ConnectRequest>,
    ) -> ServiceResult<proto::ConnectResponse> {
        let world = world(request.world)?;
        self.portal
            .connect(&world)
            .await
            .map_err(|error| convert::status(&error))?;
        Response::ok(proto::ConnectResponse::default())
    }

    async fn disconnect(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, proto::DisconnectRequest>,
    ) -> ServiceResult<proto::DisconnectResponse> {
        let world = world(request.world)?;
        self.portal
            .disconnect(&world)
            .await
            .map_err(|error| convert::status(&error))?;
        Response::ok(proto::DisconnectResponse::default())
    }
}

#[cfg(test)]
mod tests {
    use connectrpc::ErrorCode;
    use mu_core::{EventKind, Journal as _, MemoryJournal, NewEvent, Source};

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
        let journal = MemoryJournal::new(4).unwrap();
        let (_stop, stopping) = watch::channel(false);
        let mut slow = event_stream(journal.subscribe(Seq(1)).unwrap(), "id", stopping.clone());
        let mut fast = event_stream(journal.subscribe(Seq(1)).unwrap(), "id", stopping);
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
    async fn stopping_ends_the_stream() {
        let journal = MemoryJournal::new(4).unwrap();
        let (stop, stopping) = watch::channel(false);
        let mut stream = event_stream(journal.subscribe(Seq(1)).unwrap(), "id", stopping);
        assert!(stream.next().await.is_some());
        stop.send_replace(true);
        assert!(stream.next().await.is_none());
    }
}
