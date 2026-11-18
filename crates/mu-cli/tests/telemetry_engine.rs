//! The telemetry of the engine under the layers of `mu server`, with in-memory exporters.

use std::sync::Arc;

use futures::StreamExt as _;
use mu_cli::telemetry::Providers;
use mu_core::{
    Command, Event, EventKind, Journal as _, MemoryJournal, NewEvent, ResolvedWorld, Seq, Source,
    WorldSetup,
};
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::InMemoryMetricExporter;
use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData};
use opentelemetry_sdk::trace::{InMemorySpanExporter, SpanData};
use tokio::sync::watch;
use tracing_subscriber::layer::SubscriberExt as _;

use engine::{id, idle, on, portal, started, wait_event, world};

#[path = "../../mu-core/tests/support/mod.rs"]
mod engine;

const PW: &str = "s3cret-pw";

/// The providers of the in-memory exporters, and the exporters.
struct Memory {
    providers: Providers,
    spans: InMemorySpanExporter,
    metrics: InMemoryMetricExporter,
}

impl Memory {
    fn new() -> Self {
        let spans = InMemorySpanExporter::default();
        let metrics = InMemoryMetricExporter::default();
        let providers = Providers::with_exporters(Some(spans.clone()), Some(metrics.clone()), "mu");
        Self {
            providers,
            spans,
            metrics,
        }
    }

    /// The subscriber of the layers of the export, with no `fmt` layer.
    fn subscriber(&self) -> impl tracing::Subscriber + Send + Sync + use<> {
        tracing_subscriber::registry().with(self.providers.layers())
    }

    /// Runs `read` on the last export of the metric `name`, after a flush.
    fn last_metric<R>(&self, name: &str, read: impl FnOnce(&MetricData<u64>) -> R) -> Option<R> {
        let provider = self.providers.meter_provider().unwrap();
        provider.force_flush().unwrap();
        let exports = self.metrics.get_finished_metrics().unwrap();
        let metric = exports
            .iter()
            .flat_map(opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics)
            .flat_map(opentelemetry_sdk::metrics::data::ScopeMetrics::metrics)
            .filter(|metric| metric.name() == name)
            .last()?;
        let AggregatedMetrics::U64(data) = metric.data() else {
            panic!("{name} is not a u64 metric: {:?}", metric.data());
        };
        Some(read(data))
    }

    /// The value of the point of the sum or gauge `name` whose attributes hold `attributes`, in
    /// its last export.
    fn value(&self, name: &str, attributes: &[(&str, &str)]) -> Option<u64> {
        let wanted: Vec<_> = attributes
            .iter()
            .map(|(key, value)| KeyValue::new(key.to_string(), value.to_string()))
            .collect();
        let holds = |have: Vec<&KeyValue>| wanted.iter().all(|pair| have.contains(&pair));
        self.last_metric(name, |data| match data {
            MetricData::Sum(sum) => sum
                .data_points()
                .find(|point| holds(point.attributes().collect()))
                .map(opentelemetry_sdk::metrics::data::SumDataPoint::value),
            MetricData::Gauge(gauge) => gauge
                .data_points()
                .find(|point| holds(point.attributes().collect()))
                .map(opentelemetry_sdk::metrics::data::GaugeDataPoint::value),
            other => panic!("{name} is not a sum or a gauge: {other:?}"),
        })?
    }

    /// The count and the sum of the histogram `name`, in its last export.
    fn histogram(&self, name: &str) -> Option<(u64, u64)> {
        self.last_metric(name, |data| {
            let MetricData::Histogram(histogram) = data else {
                panic!("{name} is not a histogram");
            };
            histogram.data_points().fold((0, 0), |(count, sum), point| {
                (count + point.count(), sum + point.sum())
            })
        })
    }

    /// Each span that ended so far.
    fn finished_spans(&self) -> Vec<SpanData> {
        let provider = self.providers.tracer_provider().unwrap();
        provider.force_flush().unwrap();
        self.spans.get_finished_spans().unwrap()
    }
}

#[test]
fn other_targets_make_no_span_event() {
    let memory = Memory::new();
    tracing::subscriber::with_default(memory.subscriber(), || {
        let span = tracing::info_span!(target: "mu_server", "call");
        let _entered = span.enter();
        tracing::info!(target: "h2", "a frame");
    });
    let spans = memory.finished_spans();
    assert_eq!(spans.len(), 1, "{spans:?}");
    assert!(spans[0].events.is_empty(), "{:?}", spans[0].events);
}

#[test]
fn metric_event_makes_no_span_event() {
    let memory = Memory::new();
    tracing::subscriber::with_default(memory.subscriber(), || {
        let span = tracing::info_span!(target: "mu_core", "work");
        let _entered = span.enter();
        tracing::event!(
            target: mu_core::METRICS_TARGET,
            tracing::Level::INFO,
            "monotonic_counter.mu.journal.events" = 1_u64,
        );
    });
    let spans = memory.finished_spans();
    assert_eq!(spans.len(), 1, "{spans:?}");
    assert!(spans[0].events.is_empty(), "{:?}", spans[0].events);
}

#[test]
fn metric_field_under_another_target_makes_no_metric() {
    let memory = Memory::new();
    tracing::subscriber::with_default(memory.subscriber(), || {
        tracing::event!(
            target: "mu_core",
            tracing::Level::INFO,
            "monotonic_counter.mu.probe.other" = 1_u64,
        );
        tracing::event!(
            target: mu_core::METRICS_TARGET,
            tracing::Level::INFO,
            "monotonic_counter.mu.probe.metrics" = 1_u64,
        );
    });
    assert_eq!(memory.value("mu.probe.metrics", &[]), Some(1));
    assert_eq!(memory.value("mu.probe.other", &[]), None);
}

#[test]
fn crate_with_a_longer_name_makes_no_span() {
    let memory = Memory::new();
    tracing::subscriber::with_default(memory.subscriber(), || {
        tracing::info_span!(target: "mu_client", "call").in_scope(|| {});
        tracing::info_span!(target: "mu_cli::server", "run").in_scope(|| {});
    });
    let names: Vec<_> = memory
        .finished_spans()
        .into_iter()
        .map(|span| span.name.into_owned())
        .collect();
    assert_eq!(names, ["run"]);
}

fn line(journal: &MemoryJournal, text: &str) -> Event {
    journal.append(NewEvent {
        world: id("alpha/aria"),
        kind: EventKind::Line,
        source: Source::Game,
        text: text.to_owned(),
        depth: 0,
    })
}

/// The in-memory exporters as the default subscriber, a journal of `capacity` events, the
/// sender of the stop, and an event stream from the first event, past its `Hello`.
async fn subscriber(
    capacity: usize,
) -> (
    Memory,
    tracing::subscriber::DefaultGuard,
    Arc<MemoryJournal>,
    watch::Sender<bool>,
    connectrpc::ServiceStream<mu_proto::SubscribeResponse>,
) {
    let memory = Memory::new();
    let default = tracing::subscriber::set_default(memory.subscriber());
    let journal = Arc::new(MemoryJournal::new(capacity).unwrap());
    let (stop, stopping) = watch::channel(false);
    let subscription = journal.subscribe(Seq(1)).unwrap();
    let mut stream =
        mu_server::service::event_stream(subscription, journal.clone(), "id", stopping);
    stream.next().await.unwrap().unwrap();
    (memory, default, journal, stop, stream)
}

#[tokio::test]
async fn lagged_stream_is_counted() {
    let (memory, _default, journal, _stop, mut stream) = subscriber(4).await;
    for n in 1..=5 {
        line(&journal, &format!("line {n}"));
    }
    let error = stream.next().await.unwrap().unwrap_err();
    assert_eq!(error.code, connectrpc::ErrorCode::ResourceExhausted);
    assert_eq!(memory.value("mu.subscriber.lagged", &[]), Some(1));
}

#[tokio::test]
async fn lag_at_the_head_is_zero() {
    let (memory, _default, journal, _stop, mut stream) = subscriber(4).await;
    line(&journal, "the head");
    stream.next().await.unwrap().unwrap();
    assert_eq!(memory.histogram("mu.subscriber.lag"), Some((1, 0)));
}

#[tokio::test]
async fn lag_behind_the_head() {
    let (memory, _default, journal, _stop, mut stream) = subscriber(8).await;
    for n in 1..=3 {
        line(&journal, &format!("line {n}"));
    }
    for _ in 1..=3 {
        stream.next().await.unwrap().unwrap();
    }
    // The three events are 2, 1 and 0 behind the head.
    assert_eq!(memory.histogram("mu.subscriber.lag"), Some((3, 3)));
}

/// A world on a fake game whose handler answers `go` from the game with `count` echoes.
fn burst(count: usize) -> (WorldSetup, mu_core::FakeGame) {
    let handler = on(move |event| {
        if event.kind == EventKind::Line && event.text == "go" {
            (0..count)
                .map(|_| Command::Echo {
                    world: event.world.clone(),
                    text: "x".to_owned(),
                })
                .collect()
        } else {
            Vec::new()
        }
    });
    let (aria, game) = world("alpha/aria", PW);
    (aria.with_handler(handler), game)
}

#[tokio::test(start_paused = true)]
async fn command_past_the_depth_limit() {
    let memory = Memory::new();
    let _default = tracing::subscriber::set_default(memory.subscriber());
    let echo = on(|event| {
        if matches!(event.kind, EventKind::Input | EventKind::Echo) {
            vec![Command::Echo {
                world: event.world.clone(),
                text: "again".to_owned(),
            }]
        } else {
            Vec::new()
        }
    });
    let (aria, _game) = world("alpha/aria", PW);
    let portal = started(vec![aria.with_handler(echo)]).await;
    portal.send(&id("alpha/aria"), "start").await.unwrap();
    wait_event(&portal, |event| event.kind == EventKind::ScriptError).await;
    idle(&portal).await;
    let world = ("mu.world", "alpha/aria");
    assert_eq!(
        memory.value("mu.commands.dropped", &[world, ("mu.drop.reason", "depth")]),
        Some(1)
    );
    assert_eq!(memory.value("mu.script.errors", &[world]), Some(1));
}

#[tokio::test(start_paused = true)]
async fn command_over_the_rate() {
    let memory = Memory::new();
    let _default = tracing::subscriber::set_default(memory.subscriber());
    let (aria, game) = burst(25);
    let portal = started(vec![aria]).await;
    assert!(game.say("go"));
    wait_event(&portal, |event| event.kind == EventKind::ScriptError).await;
    idle(&portal).await;
    assert_eq!(
        memory.value(
            "mu.commands.dropped",
            &[("mu.world", "alpha/aria"), ("mu.drop.reason", "rate")]
        ),
        Some(5)
    );
}

#[tokio::test(start_paused = true)]
async fn commands_at_the_rate() {
    let memory = Memory::new();
    let _default = tracing::subscriber::set_default(memory.subscriber());
    let (aria, game) = burst(20);
    let portal = started(vec![aria]).await;
    assert!(game.say("go"));
    wait_event(&portal, |event| {
        event.kind == EventKind::Line && event.text == "go"
    })
    .await;
    idle(&portal).await;
    assert_eq!(engine::of_kind(&portal, EventKind::Echo).len(), 20);
    assert_eq!(
        memory.value("mu.commands.dropped", &[("mu.world", "alpha/aria")]),
        None
    );
}

#[tokio::test(start_paused = true)]
async fn world_without_transport_is_disconnected() {
    let memory = Memory::new();
    let _default = tracing::subscriber::set_default(memory.subscriber());
    let resolved = ResolvedWorld::with_default_login(id("alpha/aria"), "env:PW".parse().unwrap());
    let portal = portal(vec![WorldSetup::without_transport(resolved)], Vec::new());
    idle(&portal).await;
    assert_eq!(
        memory.value(
            "mu.world.state",
            &[
                ("mu.world", "alpha/aria"),
                ("mu.world.state.name", "disconnected")
            ]
        ),
        Some(1)
    );
}
